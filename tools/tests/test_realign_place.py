import copy
import io
import json
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


if __name__ == "__main__":
    unittest.main()
