import copy
from decimal import Decimal
import io
import json
import os
import re
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import realign_place as rp
import smt_pos_map as sm


class RealignTests(unittest.TestCase):
    def test_addendum_top_bottom_self_check(self):
        changes = {"p::F": (2_000_000, 1_000_000, 90)}
        top = "design D { layout {\n place x at (10mm, 0mm)\n} }\n"
        bottom = top.replace("0mm)", "0mm) side bottom")
        for source, expected in [
            (top, "place x at (12mm, 1mm) rotate 90"),
            (bottom, "place x at (8mm, 1mm) rotate 270 side bottom"),
        ]:
            edits, errors, _ = rp.realign(source, changes, {"x": "p::F"})
            self.assertFalse(errors)
            for start, end, replacement in reversed(edits):
                source = source[:start] + replacement + source[end:]
            self.assertIn(expected, source)

    def test_forward_reverse_all_sides_and_mounting(self):
        source = """design D {
 layout {
  place U1 at (10mm, 20mm) rotate 37
  place U2 at (30mm, 40mm) rotate 37 side bottom
  place mounting[0] at (1mm, 2mm) side bottom
 }
}
"""
        fps = {"U1": "p::F", "U2": "p::F", "mounting[0]": "p::F"}
        changes = {"p::F": (2 * 10**6, 1 * 10**6, 90)}
        edits, errors, _ = rp.realign(source, changes, fps)
        self.assertFalse(errors)
        forward = source
        for a, b, text in reversed(edits):
            forward = forward[:a] + text + forward[b:]
        self.assertNotEqual(forward, source)
        edits, errors, _ = rp.realign(forward, changes, fps, reverse=True)
        self.assertFalse(errors)
        restored = forward
        for a, b, text in reversed(edits):
            restored = restored[:a] + text + restored[b:]
        # At 37 degrees the inverse carries fixed-point roundoff; every
        # coordinate (bottom included) must return within 1 nm and every
        # angle and side exactly.
        number = r"(-?[0-9.]+)mm"
        statement = r"place (\S+) at \(" + number + ", " + number + r"\)(.*)"
        expected = re.findall(statement, source)
        actual = re.findall(statement, restored)
        self.assertEqual(len(actual), 3)
        for (name, x, y, tail), (name2, x2, y2, tail2) in zip(expected, actual):
            self.assertEqual((name, tail), (name2, tail2))
            self.assertLessEqual(abs(Decimal(x) - Decimal(x2)), Decimal("0.000001"), name)
            self.assertLessEqual(abs(Decimal(y) - Decimal(y2)), Decimal("0.000001"), name)
        self.assertIn("rotate 37 side bottom", restored)
        # CLI --reverse uses exact source snapshots; geometric inverse is tested
        # independently below at cardinal angles without sub-femto roundoff.
        cardinal = "design D { layout {\n place U1 at (10mm, 20mm)\n place U2 at (30mm, 40mm) side bottom\n} }\n"
        edits, errors, _ = rp.realign(cardinal, changes, fps)
        self.assertFalse(errors)
        forward_cardinal = cardinal
        for a, b, text in reversed(edits):
            forward_cardinal = forward_cardinal[:a] + text + forward_cardinal[b:]
        edits, errors, _ = rp.realign(forward_cardinal, changes, fps, reverse=True)
        self.assertFalse(errors)
        for a, b, text in reversed(edits):
            forward_cardinal = forward_cardinal[:a] + text + forward_cardinal[b:]
        self.assertEqual(forward_cardinal, cardinal)

    def test_non_targets_untouched_and_all_unsupported_reported(self):
        source = """subdesign S {
 layout {
  place U1 at (1mm, 2mm)
 }
}
design D {
 layout {
  for i in 0..3 { place U2 at (2mm, 3mm) }
  for j in 0..2 {
   place U4 at (1mm, 2mm)
  }
  place U3 at (X, 3mm)
  place other at (1mm, 2mm)
 }
}
"""
        _, errors, _ = rp.realign(source, {"p::F": (1, 2, 3)}, {"U1": "p::F", "U2": "p::F", "U3": "p::F", "U4": "p::F"})
        self.assertEqual([line for line, _ in errors], [3, 8, 10, 12])

    def test_map_changes_and_conflicts(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "map.json"
            p.write_text(json.dumps({"changes": [{"footprint": "p::F", "dx_mm": 2, "dy_mm": 1, "rotate": 90}]}))
            self.assertEqual(rp.read_changes(p)["p::F"], (2 * 10**6, 10**6, 90))

    def test_cli_reports_file_line_without_writing(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            (root / "src").mkdir()
            src = root / "src/main.cohdl"
            original = "design D { layout {\nplace U1 at (X, 1mm)\n} }\n"
            src.write_text(original)
            (root / "change.json").write_text('{"changes":[{"footprint":"p::F","dx_mm":1}]}')
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(root), str(root / "change.json")]), 1)
            self.assertIn("requires --layout and --board", err.getvalue())
            self.assertEqual(src.read_text(), original)

    def _plain_names_project(self, root, source):
        (root / "src").mkdir()
        (root / "src/main.cohdl").write_text(source)
        (root / "change.json").write_text('{"changes":[{"footprint":"p::F","rotate":90}]}')
        (root / "layout.json").write_text(json.dumps({"placements": [
            {"instance": "D::u_0", "at": [1, 2], "rotate": 0},
            {"instance": "D::v_1", "at": [5, 6], "rotate": 0}]}))
        (root / "board.kicad_pcb").write_text(
            '(kicad_pcb (footprint "p-F" (layer "F.Cu") (at 1 2)) (footprint "p-F" (layer "F.Cu") (at 5 6)))\n')
        return ["--layout", str(root / "layout.json"), "--board", str(root / "board.kicad_pcb")]

    def test_plain_instance_named_like_an_array_element(self):
        # `inst u_0: D` is laid out as u_0 and placed as u_0, not u[0].
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            args = self._plain_names_project(root, "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n")
            with mock.patch("sys.stdout", new=io.StringIO()) as out:
                self.assertEqual(rp.main([str(root), str(root / "change.json")] + args), 0)
            self.assertIn("realigned 2 placements", out.getvalue())
            self.assertIn("place u_0 at (1mm, 2mm) rotate 90\n", (root / "src/main.cohdl").read_text())

    def test_moved_placement_without_a_place_statement_is_an_error(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\n} }\n"
            args = self._plain_names_project(root, original)
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(root), str(root / "change.json")] + args), 1)
            self.assertIn("v_1 uses moved footprint p::F", err.getvalue())
            self.assertEqual((root / "src/main.cohdl").read_text(), original)

    def test_subdesign_member_on_a_moved_footprint_is_an_error(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
            args = self._plain_names_project(root, original)
            layout = json.loads((root / "layout.json").read_text())
            layout["placements"][1]["instance"] = "D::port::xcvr"
            (root / "layout.json").write_text(json.dumps(layout))
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(root), str(root / "change.json")] + args), 1)
            self.assertIn("port::xcvr uses moved footprint p::F but it is placed inside a subdesign", err.getvalue())
            self.assertEqual((root / "src/main.cohdl").read_text(), original)

    def test_generic_subdesign_place_is_diagnosed_not_rewritten(self):
        source = """subdesign Vreg<Cin: Capacitance> {
 layout {
  place U1 at (1mm, 2mm)
 }
}
design D {
 layout {
  place U1 at (5mm, 6mm)
 }
}
"""
        edits, errors, _ = rp.realign(source, {"p::F": (0, 0, 90)}, {"U1": "p::F"}, design="D")
        self.assertEqual(errors, [(3, "place inside subdesign is not rewritten")])
        self.assertEqual(len(edits), 1)
        self.assertEqual(source.count("\n", 0, edits[0][0]) + 1, 8)

    def test_design_body_spans_past_an_earlier_subdesign_use_site_block(self):
        # The design header's own `{` opens its body; the use site's block is
        # nested in it and must not end the design's region early.
        source = """design D {
 subdesign port: CanPort { a: n1 }
 layout {
  place U1 at (5mm, 6mm)
 }
}
"""
        notes = []
        edits, errors, changed = rp.realign(
            source, {"p::F": (0, 0, 90)}, {"U1": "p::F"}, design="D", notes=notes)
        self.assertFalse(errors)
        self.assertFalse(notes)
        self.assertEqual(changed, [("U1", "p::F")])
        self.assertIn("rotate 90", source[:edits[0][0]] + edits[0][2])

    def test_a_laid_out_target_left_unrewritten_fails_the_run(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
            args = self._plain_names_project(root, original)
            real = rp.realign

            def drop_v_1(*a, **k):
                edits, errors, changed = real(*a, **k)
                keep = [i for i, (target, _) in enumerate(changed) if target != "v_1"]
                return [edits[i] for i in keep], errors, [changed[i] for i in keep]

            with mock.patch.object(rp, "realign", drop_v_1), \
                    mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(root), str(root / "change.json")] + args), 1)
            self.assertIn("v_1 uses moved footprint p::F but was not rewritten", err.getvalue())
            self.assertEqual((root / "src/main.cohdl").read_text(), original)

    def test_same_named_place_in_another_design_is_left_alone(self):
        source = """design Other {
 layout {
  place U1 at (1mm, 2mm)
 }
}
design D {
 layout {
  place U1 at (5mm, 6mm)
 }
}
"""
        notes = []
        edits, errors, _ = rp.realign(source, {"p::F": (0, 0, 90)}, {"U1": "p::F"}, design="D", notes=notes)
        self.assertFalse(errors)
        self.assertEqual([line for line, _ in notes], [3])
        self.assertEqual(len(edits), 1)
        self.assertIn("(5mm, 6mm)", source[edits[0][0]:edits[0][1]])

    def test_comments_neither_abort_nor_get_rewritten(self):
        source = """design D {
 layout {
  // place U1 at (9mm, 9mm)
  place U1 at (1mm, 2mm) // locked by the enclosure
 }
}
"""
        edits, errors, _ = rp.realign(source, {"p::F": (0, 0, 90)}, {"U1": "p::F"}, design="D")
        self.assertFalse(errors)
        self.assertEqual(len(edits), 1)
        out = source
        for start, end, text in reversed(edits):
            out = out[:start] + text + out[end:]
        self.assertIn("  // place U1 at (9mm, 9mm)\n", out)
        self.assertIn("  place U1 at (1mm, 2mm) rotate 90 // locked by the enclosure\n", out)

    def test_placement_that_no_longer_matches_the_layout_is_refused(self):
        source = "design D { layout {\nplace U1 at (1mm, 2mm) rotate 90\n} }\n"
        placed = {"U1": (10**15, 2 * 10**15, 0, False)}
        edits, errors, _ = rp.realign(source, {"p::F": (0, 0, 90)}, {"U1": "p::F"}, design="D", placed=placed)
        self.assertEqual(edits, [])
        self.assertIn("no longer matches the build's layout", errors[0][1])

    def test_unknown_change_keys_are_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            p = Path(td) / "map.json"
            p.write_text(json.dumps({"changes": [{"footprint": "p::F", "rotate_deg": 90}]}))
            with self.assertRaisesRegex(ValueError, "unknown key.*rotate_deg"):
                rp.read_changes(p)

    def test_second_forward_run_and_foreign_reverse_are_refused(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td) / "a"
            root.mkdir()
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
            args = self._plain_names_project(root, original)
            cli = [str(root), str(root / "change.json")]
            with mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(rp.main(cli + args), 0)
            once = (root / "src/main.cohdl").read_text()
            self.assertNotEqual(once, original)
            # Same table again: the undo log is not overwritten.
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main(cli + args), 1)
            self.assertIn("already exists", err.getvalue())
            # A fresh undo log does not help: the source no longer equals the
            # stale layout, so the delta is never composed twice.
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main(cli + args + ["--undo-log", str(root / "second.json")]), 1)
            self.assertIn("no longer matches the build's layout", err.getvalue())
            self.assertEqual((root / "src/main.cohdl").read_text(), once)
            # The undo log belongs to project `a`; reversing project `b` fails.
            other = Path(td) / "b"
            (other / "src").mkdir(parents=True)
            (other / "src/main.cohdl").write_text(once)
            with mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(other), str(root / "change.json"), "--reverse"]), 1)
            self.assertIn("belongs to project", err.getvalue())
            with mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(rp.main(cli + ["--reverse"]), 0)
            self.assertEqual((root / "src/main.cohdl").read_text(), original)
            self.assertFalse((root / "change.json.undo.json").exists())

    def test_an_unwritable_undo_log_leaves_the_sources_untouched(self):
        for case in ("missing", "read-only"):
            with self.subTest(case), tempfile.TemporaryDirectory() as td:
                root = Path(td)
                original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
                args = self._plain_names_project(root, original)
                logs = root / "logs"
                if case == "read-only":
                    logs.mkdir()
                    logs.chmod(0o500)
                    if os.access(logs, os.W_OK):
                        self.skipTest("running with write access to read-only directories")
                undo = logs / "undo.json"
                try:
                    with mock.patch("sys.stderr", new=io.StringIO()) as err:
                        self.assertEqual(rp.main([str(root), str(root / "change.json"), *args,
                                                  "--undo-log", str(undo)]), 1)
                finally:
                    if logs.exists():
                        logs.chmod(0o700)
                self.assertIn("realign_place.py:", err.getvalue())
                self.assertEqual((root / "src/main.cohdl").read_text(), original)
                self.assertEqual(list(logs.glob("*")) if logs.exists() else [], [])

    def test_a_failed_source_write_restores_sources_and_drops_the_log(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
            args = self._plain_names_project(root, original)
            real, failed = Path.write_text, []

            def write_text(path, *a, **k):
                if path.suffix == ".cohdl" and not failed:
                    failed.append(path)
                    real(path, "partial", **k)
                    raise OSError("disk full")
                return real(path, *a, **k)

            with mock.patch.object(Path, "write_text", write_text), \
                    mock.patch("sys.stderr", new=io.StringIO()) as err:
                self.assertEqual(rp.main([str(root), str(root / "change.json")] + args), 1)
            self.assertIn("disk full", err.getvalue())
            self.assertEqual((root / "src/main.cohdl").read_text(), original)
            self.assertFalse((root / "change.json.undo.json").exists())
            self.assertFalse((root / "change.json.undo.json.partial").exists())

    def test_a_reverse_that_stopped_part_way_can_be_resumed(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            original = "design D { layout {\nplace u_0 at (1mm, 2mm)\nplace v_1 at (5mm, 6mm)\n} }\n"
            args = self._plain_names_project(root, original)
            cli = [str(root), str(root / "change.json")]
            with mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(rp.main(cli + args), 0)
            # The file is back at its original source but the log remains.
            (root / "src/main.cohdl").write_text(original)
            with mock.patch("sys.stdout", new=io.StringIO()):
                self.assertEqual(rp.main(cli + ["--reverse"]), 0)
            self.assertEqual((root / "src/main.cohdl").read_text(), original)
            self.assertFalse((root / "change.json.undo.json").exists())


if __name__ == "__main__":
    unittest.main()
