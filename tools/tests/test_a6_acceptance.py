"""Real compiler acceptance; runs by default when target/debug/cohdl exists."""
import json
import math
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOOLS = ROOT / "tools"
COHDL = ROOT / "target/debug/cohdl"
sys.path.insert(0, str(TOOLS))
from jlc_frame_board import board_sexpr, children
from smt_pos_map import round_nm


def run(argv, cwd=ROOT):
    result = subprocess.run(argv, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode:
        raise AssertionError("command failed: %s\n%s" % (argv, result.stdout))
    return result.stdout


def nm(value):
    return int((Decimal(value) * 1_000_000).to_integral_value())


def pads(path, footprint=None):
    board = board_sexpr(path.read_text(encoding="utf-8"))
    result = {}
    for fp in children(board, "footprint"):
        if footprint and fp[1] != footprint.replace("::", "-"):
            continue
        ref = next(p[2] for p in children(fp, "property") if p[1] == "Reference")
        at = next(c for c in children(fp, "at"))
        anchor = nm(at[1]), nm(at[2])
        angle = float(at[3]) if len(at) > 3 else 0
        bottom = next(c for c in children(fp, "layer"))[1] == "B.Cu"
        cosine, sine = math.cos(math.radians(angle)), math.sin(math.radians(angle))
        for pad_index, pad in enumerate(children(fp, "pad")):
            local = next(c for c in children(pad, "at"))
            x, y = nm(local[1]), nm(local[2])
            absolute = (anchor[0] + round_nm(x * cosine + y * sine), anchor[1] + round_nm(y * cosine - x * sine))
            result[(ref, pad[1], pad_index)] = absolute
    return result


def assert_pads_equal(test, actual, expected, tolerance_nm=0):
    test.assertEqual(actual.keys(), expected.keys())
    for key in expected:
        delta = tuple(abs(a - b) for a, b in zip(actual[key], expected[key]))
        test.assertLessEqual(max(delta), tolerance_nm, "%s moved by %s nm" % (key, delta))


def change(tmp, name, footprint):
    path = tmp / (name + ".json")
    path.write_text(json.dumps({"changes": [{"footprint": footprint, "dx_mm": 1, "dy_mm": 2, "rotate": 90}]}))
    return path


def realign(project, name, changes, reverse=False):
    argv = ["python3", "-I", str(TOOLS / "realign_place.py"), str(project), str(changes)]
    if reverse:
        argv.append("--reverse")
    else:
        argv += ["--layout", str(project / ("out/%s-layout.json" % name)), "--board", str(project / ("out/%s.kicad_pcb" % name))]
    output = run(argv)
    match = re.search(r"realigned (\d+) placements", output)
    if not match:
        raise AssertionError(output)
    return int(match.group(1))


@unittest.skipUnless(COHDL.exists(), "target/debug/cohdl is not built")
class CompilerAcceptance(unittest.TestCase):
    def test_three_examples_round_trip(self):
        cases = [
            ("joint-motor-controller", "connectors::headers::micro_fit_3::FP_Molex_43045_0212", 2),
            ("rpi-pico2", "passive::CHIP_0201", 2),
            ("sf32-miniboard", "soic::SOIC8P127X790X216N", 1),
        ]
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            for name, footprint, expected in cases:
                with self.subTest(example=name):
                    project = tmp / name
                    shutil.copytree(ROOT / "examples" / name, project)
                    run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
                    original = {p.relative_to(project): p.read_bytes() for p in (project / "src").glob("*.cohdl")}
                    changes = change(tmp, name, footprint)
                    self.assertEqual(realign(project, name, changes), expected)
                    self.assertTrue(any(p.read_bytes() != original[path] for path, p in ((p.relative_to(project), p) for p in (project / "src").glob("*.cohdl"))))
                    self.assertEqual(realign(project, name, changes, True), expected)
                    for path, content in original.items():
                        self.assertEqual((project / path).read_bytes(), content)
                    print("A6 round-trip %s: %d placements" % (name, expected))

    def test_three_examples_geometric_inverse(self):
        # The CLI --reverse restores the forward run's snapshot; this checks
        # that the geometric inverse alone also returns every source byte.
        import realign_place as rp
        cases = [
            ("joint-motor-controller", "connectors::headers::micro_fit_3::FP_Molex_43045_0212", 2),
            ("rpi-pico2", "passive::CHIP_0201", 2),
            ("sf32-miniboard", "soic::SOIC8P127X790X216N", 1),
        ]
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            for name, footprint, expected in cases:
                with self.subTest(example=name):
                    project = tmp / name
                    shutil.copytree(ROOT / "examples" / name, project)
                    run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
                    changes = rp.read_changes(change(tmp, name, footprint))
                    sources = {p: p.read_text(encoding="utf-8") for p in (project / "src").glob("*.cohdl")}
                    targets = {m.group(1) for s in sources.values()
                               for m in re.finditer(r"\bplace\s+(" + rp.TARGET + r")\s+at\b", s)}
                    footprints = rp.compiler_footprints(project / ("out/%s-layout.json" % name),
                                                        project / ("out/%s.kicad_pcb" % name), targets, changes)
                    count = 0
                    for path, source in sources.items():
                        edits, errors, _ = rp.realign(source, changes, footprints)
                        self.assertFalse(errors)
                        forward = source
                        for start, end, text in reversed(edits):
                            forward = forward[:start] + text + forward[end:]
                        count += len(edits)
                        edits, errors, _ = rp.realign(forward, changes, footprints, reverse=True)
                        self.assertFalse(errors)
                        for start, end, text in reversed(edits):
                            forward = forward[:start] + text + forward[end:]
                        self.assertEqual(forward, source, path)
                    self.assertEqual(count, expected)

    def test_vendored_joint_rigid_transform_preserves_pads(self):
        name = "joint-motor-controller"
        footprint = "connectors::headers::micro_fit_3::FP_Molex_43045_0212"
        with tempfile.TemporaryDirectory() as td:
            project = Path(td) / name
            shutil.copytree(ROOT / "examples" / name, project)
            run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
            board = project / ("out/%s.kicad_pcb" % name)
            before = pads(board, footprint)
            self.assertEqual(len(before), 8)
            vendored = project / "deps/connectors"
            shutil.copytree(ROOT / "lib/connectors", vendored)
            library = vendored / "src/headers/micro_fit_3.cohdl"
            source = library.read_text()
            start = source.index("pub footprint FP_Molex_43045_0212 {")
            end = source.index("\n}\n", start) + 3
            moved = (TOOLS / "tests/fixtures/a6/moved_micro_fit_3.cohdl.inc").read_text()
            library.write_text(source[:start] + moved + source[end:])
            changes = change(Path(td), name, footprint)
            self.assertEqual(realign(project, name, changes), 2)
            (project / "cohdl.lock").unlink(missing_ok=True)
            run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
            after = pads(board, footprint)
            self.assertEqual(after, before)
            print("A6 vendored joint: compared %d pads" % len(before))

    def test_top_bottom_37_rigid_transform_preserves_pads(self):
        with tempfile.TemporaryDirectory() as td:
            project = Path(td) / "rigid"
            shutil.copytree(TOOLS / "tests/fixtures/a6/rigid", project)
            run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
            board = project / "out/a6_rigid.kicad_pcb"
            before = pads(board, "a6_rigid::FP")
            self.assertEqual(len(before), 6)
            source_path = project / "src/main.cohdl"
            source = source_path.read_text()
            start, end = source.index("pub footprint FP {"), source.index("\n}\n", source.index("pub footprint FP {")) + 3
            source_path.write_text(source[:start] + (project / "moved.inc").read_text() + source[end:])
            changes = change(Path(td), "rigid", "a6_rigid::FP")
            self.assertEqual(realign(project, "a6_rigid", changes), 3)
            run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
            assert_pads_equal(self, pads(board, "a6_rigid::FP"), before, tolerance_nm=1)
            print("A6 rigid top/bottom/37: compared %d pads" % len(before))

    def test_bottom_preview_generated_source_pads_and_cpl(self):
        with tempfile.TemporaryDirectory() as td:
            output = Path(td) / "bottom"
            run(["python3", "-I", str(TOOLS / "jlc_frame_board.py"), str(TOOLS / "tests/fixtures/a6/bottom-preview.json"), "-o", str(output), "--cohdl", str(COHDL), "--skip-kicad"])
            actual_board = output / "out/bottom_preview.kicad_pcb"
            expected_board = ROOT / "tests/fixtures/rfc034/bottom_preview.kicad_pcb"
            before, after = pads(expected_board), pads(actual_board)
            self.assertEqual(len(after), 60)
            self.assertEqual(after, before)
            actual_cpl = (output / "out/bottom_preview-lcsc-cpl.csv").read_bytes()
            expected_cpl = (ROOT / "tests/fixtures/rfc034/bottom-preview/handoff/cpl-acme-reference.csv").read_bytes()
            self.assertEqual(actual_cpl.splitlines(), expected_cpl.splitlines())
            self.assertEqual(actual_cpl, expected_cpl)
            self.assertNotIn("template", (TOOLS / "jlc_frame_board.py").read_text())
            print("A6 bottom-preview: compared %d pads and %d CPL rows" % (len(after), len(actual_cpl.splitlines()) - 1))

    def test_unsupported_targets_report_file_and_line(self):
        with tempfile.TemporaryDirectory() as td:
            project = Path(td) / "rigid"
            shutil.copytree(TOOLS / "tests/fixtures/a6/rigid", project)
            run([str(COHDL), "build", str(project), "--emit", "kicad_pcb"])
            changes = change(Path(td), "rigid", "a6_rigid::FP")
            source_path = project / "src/main.cohdl"
            source_path.write_text(source_path.read_text().replace("place top at (-20mm, 0mm)", "place top at (X, 0mm)"))
            result = subprocess.run(["python3", "-I", str(TOOLS / "realign_place.py"), str(project), str(changes), "--layout", str(project / "out/a6_rigid-layout.json"), "--board", str(project / "out/a6_rigid.kicad_pcb")], text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
            self.assertNotEqual(result.returncode, 0)
            self.assertRegex(result.stdout, r"main\.cohdl:20:")

    def test_for_subdesign_and_reach_in_diagnostics(self):
        import realign_place as rp
        source = '''subdesign S {
 layout {
  place inner at (0mm, 0mm)
 }
}
design D {
 layout {
  for i in 0..2 {
   place looped at (1mm, 0mm)
  }
  place motor.inner at (2mm, 0mm)
 }
}
'''
        fps = {"inner": "p::F", "looped": "p::F"}
        _, errors, _ = rp.realign(source, {"p::F": (1_000_000, 2_000_000, 90)}, fps)
        self.assertEqual([(line, message) for line, message in errors], [
            (3, "place inside subdesign is not rewritten"),
            (9, "place in a for loop is not rewritten"),
            (11, "subdesign reach-in place is not rewritten"),
        ])


if __name__ == "__main__":
    unittest.main()
