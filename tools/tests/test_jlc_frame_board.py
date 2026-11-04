import json
import math
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

    def test_turned_boxes_set_spacing_overlap_and_outline(self):
        # A long, thin part: at 37 degrees its pads reach far past its
        # unrotated height.
        row = {"name": "long", "footprint": "pkg::L", "pn": "C3", "bottom": True,
               "width_mm": 40, "height_mm": 2}
        places, bounds = jb.expand([row], 5)
        boxes = []
        for p in places:
            a = math.radians(p["rotate"])
            corners = [(p["x"] + dx * math.cos(a) - dy * math.sin(a), p["y"] + dx * math.sin(a) + dy * math.cos(a))
                       for dx in (-20, 20) for dy in (-1, 1)]
            xs, ys = [c[0] for c in corners], [c[1] for c in corners]
            boxes.append((min(xs), min(ys), max(xs), max(ys)))
            for x, y in corners:
                self.assertTrue(bounds[0] < x < bounds[2] and bounds[1] < y < bounds[3], (p["rotate"], x, y, bounds))
        # The turned boxes keep the full spacing between rows.
        for upper, lower in zip(boxes, boxes[1:]):
            self.assertGreaterEqual(lower[1] - upper[3], 5 - 1e-9)

        # An explicit pair that overlaps only once turned is refused...
        explicit = dict(row, bottom=False, placements=[{"at_mm": [0, 0]}, {"at_mm": [0, 8], "rotate": 37}])
        with self.assertRaisesRegex(jb.BoardError, "overlap"):
            jb.expand([explicit], 5)
        # ...unless the rectangle is declared to cover every rotation.
        square = dict(explicit, width_mm=7, height_mm=7, covers_rotation=True)
        jb.expand([square], 5)

    def test_committed_frame_boards_mark_their_boxes_as_covering_rotation(self):
        # Their generators size each box from the courtyard's reach, so turning
        # it again would move the outline of boards already uploaded.
        for manifest in sorted((Path(__file__).resolve().parents[1] / "frame_boards").glob("*/manifest.json")):
            with self.subTest(manifest.parent.name):
                rows = json.loads(manifest.read_text(encoding="utf-8"))["rows"]
                self.assertTrue(all(r.get("covers_rotation") is True for r in rows))
                jb.expand(jb.load_manifest(manifest)["rows"], 5)

    def test_mocked_toolchain_writes_project_and_fabrication(self):
        calls = []
        origin = True
        def run(argv):
            calls.append(argv)
            if argv[0] == "cohdl":
                out = Path(argv[2]) / "out"
                out.mkdir(exist_ok=True)
                # The designator prefix comes from the device, not the row name.
                setup = "(setup (aux_axis_origin -4 9.5))" if origin else "(setup)"
                (out / "demo.kicad_pcb").write_text('(kicad_pcb %s (footprint "pkg-FP" (layer "F.Cu") (at 1 2 0) (property "Reference" "Q7")))\n' % setup)
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
            # Gerbers, drill files and the CPL share the compiler's place origin.
            self.assertIn("--use-drill-file-origin", next(c for c in calls if "gerbers" in c))
            drill = next(c for c in calls if "drill" in c)
            self.assertEqual(drill[drill.index("--drill-origin") + 1], "plot")

            # A board without that origin is refused before anything is plotted.
            origin = False
            calls.clear()
            with mock.patch.object(jb, "run", side_effect=run):
                self.assertEqual(jb.main([str(manifest), "-o", str(root / "old"), "--cohdl", "cohdl"]), 1)
            self.assertFalse(any("gerbers" in c for c in calls))

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
