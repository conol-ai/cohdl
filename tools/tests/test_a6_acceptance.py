"""Opt-in real compiler acceptance: ``A6_ACCEPTANCE=1 python3 -I ...``."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
TOOLS = ROOT / "tools"


def run(argv, cwd=ROOT):
    result = subprocess.run(argv, cwd=cwd, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
    if result.returncode:
        raise AssertionError("command failed: %s\n%s" % (argv, result.stdout))
    return result.stdout


@unittest.skipUnless(os.environ.get("A6_ACCEPTANCE") == "1", "set A6_ACCEPTANCE=1 to run compiler acceptance")
class CompilerAcceptance(unittest.TestCase):
    def test_three_examples_round_trip_pads_and_bottom_preview(self):
        cohdl = ROOT / "target/debug/cohdl"
        self.assertTrue(cohdl.exists())
        with tempfile.TemporaryDirectory() as td:
            tmp = Path(td)
            # Three independent examples: two project builds plus one source-only
            # top/bottom literal design using the existing map fixture's bindings.
            examples = [ROOT / "tests/fixtures/rfc034/bottom-preview",
                        ROOT / "tests/fixtures/rfc034/map-integration"]
            for i, example in enumerate(examples):
                project = tmp / ("example%d" % i)
                shutil.copytree(example, project)
                supplier = "acme"
                manifest = project / "cohdl.toml"
                if "std =" not in manifest.read_text():
                    manifest.write_text(manifest.read_text() + '\n[dependencies]\nstd = "0.3.0"\n')
                run([str(cohdl), "build", str(project), "--supplier", supplier, "--bottom-rotation", supplier + "=mirrored", "--emit", "kicad_pcb"])
                name = "bottom_preview" if i == 0 else "map_integration"
                board = project / ("out/%s.kicad_pcb" % name)
                original_board = board.read_bytes()
                original_sources = {p.name: p.read_bytes() for p in (project / "src").glob("*.cohdl")}
                changes = tmp / ("changes%d.json" % i)
                changes.write_text(json.dumps({"changes": [{"package": name, "footprint": name + "::FP_A" if i else "transistor::FP_JSCJ_SOT23", "dx_mm": 2, "dy_mm": 1, "rotate": 90}]}))
                if i == 0:
                    # Use the two mapped types, not just the first one.
                    changes.write_text(json.dumps({"changes": [
                        {"footprint": "transistor::FP_JSCJ_SOT23", "dx_mm": 2, "dy_mm": 1, "rotate": 90},
                        {"footprint": "soic::SOIC8P127X790X216N", "dx_mm": -1, "dy_mm": 2, "rotate": 270}]}))
                map_path = project / ("out/%s-%s-map.json" % (name, supplier))
                run(["python3", "-I", str(TOOLS / "realign_place.py"), str(project), str(changes), "--map", str(map_path)])
                run([str(cohdl), "build", str(project), "--supplier", supplier, "--bottom-rotation", supplier + "=mirrored", "--emit", "kicad_pcb"])
                self.assertEqual(board.read_bytes(), original_board, "physical board must be byte-stable")
                run(["python3", "-I", str(TOOLS / "realign_place.py"), str(project), str(changes), "--map", str(map_path), "--reverse"])
                for source_name, content in original_sources.items():
                    self.assertEqual((project / "src" / source_name).read_bytes(), content)
            # A third, source-only example checks array/mounting forms through the
            # same realign API used by the CLI, with exact inverse text.
            import sys
            sys.path.insert(0, str(TOOLS))
            import realign_place as rp
            source = "design D { layout {\n place U[0] at (1mm, 2mm)\n place mounting[0] at (3mm, 4mm) rotate 37 side bottom\n} }\n"
            changes = {"p::F": (2_000_000, 1_000_000, 90)}
            edits, errors, _ = rp.realign(source, changes, {"U[0]": "p::F", "mounting[0]": "p::F"})
            self.assertFalse(errors)
            forward = source
            for start, end, text in reversed(edits): forward = forward[:start] + text + forward[end:]
            edits, errors, _ = rp.realign(forward, changes, {"U[0]": "p::F", "mounting[0]": "p::F"}, True)
            self.assertFalse(errors)
            for start, end, text in reversed(edits): forward = forward[:start] + text + forward[end:]
            self.assertEqual(forward, source)
            # Explicit-coordinate replay: template keeps the independently
            # previewed declarations, generator supplies lcsc and the board files.
            replay = tmp / "replay.json"
            rows = []
            for name, fp, pn, x in [("q", "transistor::FP_JSCJ_SOT23", "C2146", -26), ("u", "soic::SOIC8P127X790X216N", "C97521", -8), ("c6", "passive::CHIP_0603", "C107045", 12), ("c8", "passive::CHIP_0805", "C107110", 28)]:
                rows.append({"name": name, "footprint": fp, "pn": pn, "placements": [{"at_mm": [x, -20]}, {"at_mm": [x, -7], "rotate": 37}, {"at_mm": [x, 7], "side": "bottom"}, {"at_mm": [x, 20], "rotate": 37, "side": "bottom"}]})
            replay.write_text(json.dumps({"name": "bottom_preview", "rows": rows}))
            output = tmp / "bottom-preview-replay"
            run(["python3", "-I", str(TOOLS / "jlc_frame_board.py"), str(replay), "-o", str(output), "--template", str(examples[0]), "--cohdl", str(cohdl), "--skip-kicad"])
            self.assertEqual((output / "out/bottom_preview.kicad_pcb").read_bytes(), (ROOT / "tests/fixtures/rfc034/bottom_preview.kicad_pcb").read_bytes())
            actual_cpl = (output / "out/bottom_preview-lcsc-cpl.csv").read_bytes()
            expected_cpl = (examples[0] / "handoff/cpl-acme-reference.csv").read_bytes()
            self.assertEqual(actual_cpl, expected_cpl)
            actual_map = json.loads((output / "out/bottom_preview-lcsc-map.json").read_text())
            expected_map = json.loads((ROOT / "tests/fixtures/rfc034/bottom_preview-acme-map.json").read_text())
            for row in actual_map["designators"].values():
                row["pn"] = "A-" + row["pn"][1:]
            self.assertEqual(actual_map, expected_map)


if __name__ == "__main__":
    unittest.main()
