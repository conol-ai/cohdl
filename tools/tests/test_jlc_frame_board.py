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
                # The designator prefix comes from the device, not the row name.
                (out / "demo.kicad_pcb").write_text('(kicad_pcb (setup (pad_to_mask_clearance 0)) (footprint "pkg-FP" (layer "F.Cu") (at 1 2 0) (property "Reference" "Q7")))\n')
                (out / "demo-layout.json").write_text(json.dumps({"board_outline": {
                    "start": [-4, -3], "segments": [{"to": [20, -3]}, {"to": [20, 9.5]}, {"to": [-4, 9.5]}]}}))
                (Path(argv[2]) / "design.lock").write_text('[designators]\n"FramePreview::u_0" = "Q7"\n')
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
            self.assertIn("| Q7 | C1 | top | 0 |", (root / "board/CHECKLIST.md").read_text())
            self.assertTrue((root / "board/src/main.cohdl").exists())
            # Gerbers, drill files and the CPL share the outline's lower-left corner.
            self.assertIn("(aux_axis_origin -4 9.5)", (root / "board/out/demo.kicad_pcb").read_text())
            self.assertIn("--use-drill-file-origin", next(c for c in calls if "gerbers" in c))
            drill = next(c for c in calls if "drill" in c)
            self.assertEqual(drill[drill.index("--drill-origin") + 1], "plot")

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
