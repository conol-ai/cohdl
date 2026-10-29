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
        self.assertEqual(restored, source)

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


if __name__ == "__main__":
    unittest.main()
