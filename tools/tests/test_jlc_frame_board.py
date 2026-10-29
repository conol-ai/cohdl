import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import jlc_frame_board as jb


class FrameBoardTests(unittest.TestCase):
    def manifest(self):
        return {"name": "demo", "rows": [
            {"name": "u", "footprint": "pkg::FP", "pn": "C1", "bottom": True,
             "width_mm": 4, "height_mm": 6, "pins": ["A", "B"]},
            {"name": "r", "footprint": "pkg::R", "pn": "C2", "width_mm": 2, "height_mm": 2}
        ]}

    def test_grid_has_top_and_bottom_controls_without_overlap(self):
        places, bounds = jb.expand(self.manifest()["rows"], 3)
        self.assertEqual([(p["side"], p["rotate"]) for p in places],
                         [("top", 0), ("bottom", 0), ("bottom", 37), ("top", 0)])
        self.assertLess(bounds[0], 0)

    def test_overlap_is_rejected(self):
        doc = self.manifest()
        doc["rows"][0]["placements"] = [{"at_mm": [0, 0]}]
        doc["rows"][1]["placements"] = [{"at_mm": [0, 0]}]
        with self.assertRaisesRegex(jb.BoardError, "overlap"):
            jb.expand(doc["rows"], 5)

    def test_mocked_toolchain_writes_project_and_fabrication(self):
        calls = []
        def run(argv):
            calls.append(argv)
            if argv[0] == "cohdl":
                out = Path(argv[2]) / "out"
                out.mkdir(exist_ok=True)
                (out / "demo.kicad_pcb").write_text('(kicad_pcb (footprint "pkg-FP" (layer "F.Cu") (at 1 2 0) (property "Reference" "U1")))\n')
                return ""
            if argv[-1] == "--version":
                return "9.0.0\n"
            return ""
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            manifest = root / "in.json"
            manifest.write_text(json.dumps(self.manifest()))
            with mock.patch.object(jb, "run", side_effect=run):
                self.assertEqual(jb.main([str(manifest), "-o", str(root / "board"), "--cohdl", "cohdl"]), 0)
            build = next(c for c in calls if c[0] == "cohdl")
            self.assertEqual(build[-5:], ["--supplier", "lcsc", "--bottom-rotation", "lcsc=mirrored", "--emit", "kicad_pcb"][-5:])
            self.assertIn("C1", (root / "board/CHECKLIST.md").read_text())
            self.assertTrue((root / "board/src/main.cohdl").exists())
            self.assertTrue(any("gerbers" in c for c in calls))
            self.assertTrue(any("drill" in c for c in calls))

    def test_explicit_replay_preserves_fixture_placement_lines(self):
        rows = []
        for name, fp, pn, x in [("q", "transistor::FP_JSCJ_SOT23", "C2146", -26),
                                 ("u", "soic::SOIC8P127X790X216N", "C97521", -8),
                                 ("c6", "passive::CHIP_0603", "C107045", 12),
                                 ("c8", "passive::CHIP_0805", "C107110", 28)]:
            rows.append({"name": name, "footprint": fp, "pn": pn, "placements": [
                {"at_mm": [x, -20]}, {"at_mm": [x, -7], "rotate": 37},
                {"at_mm": [x, 7], "side": "bottom"}, {"at_mm": [x, 20], "rotate": 37, "side": "bottom"}]})
        places, _ = jb.expand(rows, 5)
        self.assertEqual(len(places), 16)
        self.assertEqual(places[-1]["rotate"], 37)


if __name__ == "__main__":
    unittest.main()
