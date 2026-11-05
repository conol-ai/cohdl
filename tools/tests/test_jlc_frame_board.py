import csv
import io
import json
import math
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import check_cpl
import jlc_frame_board as jb

ROOT = Path(__file__).resolve().parents[2]
COHDL = ROOT / "target/debug/cohdl"
REQUIRE_BINARY = os.environ.get("COHDL_REQUIRE_BINARY") == "1"


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

    def test_a_library_row_places_the_library_part_and_its_binding(self):
        doc = self.manifest()
        doc["rows"][0]["library_part"] = "pkg::BOUND_U"
        with tempfile.TemporaryDirectory() as td:
            manifest = Path(td) / "in.json"
            manifest.write_text(json.dumps(doc))
            places = jb.emit_project(jb.load_manifest(manifest), Path(td) / "board", 3)
            source = (Path(td) / "board/src/main.cohdl").read_text()
        self.assertIn("inst u_0: pkg::BOUND_U", source)
        self.assertNotIn("P_u", source)
        self.assertIn('pub part P_r: pkg::r {\n    primary { mfr: "unknown", mpn: "r", footprint: pkg::R, supplier lcsc "C2" }', source)
        # A board-local mapping would not be the library's binding.
        doc["rows"][0]["mapping"] = {"rotate_deg": 90}
        with tempfile.TemporaryDirectory() as td:
            manifest = Path(td) / "in.json"
            manifest.write_text(json.dumps(doc))
            with self.assertRaisesRegex(jb.BoardError, "mapping is refused with library_part"):
                jb.load_manifest(manifest)

        # The supplier BOM must carry the row's part number for every copy.
        with tempfile.TemporaryDirectory() as td:
            lock, bom = Path(td) / "design.lock", Path(td) / "bom.csv"
            lock.write_text('[designators]\n' + "".join(
                '"FramePreview::u_%d" = "U%d"\n' % (i, i + 1) for i in range(3)) + '"FramePreview::r_0" = "R1"\n')
            bom.write_text('Comment,Designator,Footprint,LCSC Part #\n"x","U1,U2,U3","FP","C1"\n"y","R1","R","C2"\n')
            jb.require_library_bindings(bom, lock, "FramePreview", places)
            bom.write_text('Comment,Designator,Footprint,LCSC Part #\n"x","U1,U2","FP","C1"\n"x","U3","FP","C9"\n')
            with self.assertRaisesRegex(jb.BoardError, "U3 .u. is C9 in the supplier BOM, not C1"):
                jb.require_library_bindings(bom, lock, "FramePreview", places)

    @unittest.skipUnless(COHDL.exists() or REQUIRE_BINARY, "target/debug/cohdl is not built")
    def test_lcsc_batch_1_board_uses_the_shipped_bindings(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "board"
            subprocess.run(["python3", "-I", str(ROOT / "tools/jlc_frame_board.py"),
                            str(ROOT / "tools/frame_boards/lcsc-batch-1/manifest.json"), "-o", str(out),
                            "--cohdl", str(COHDL), "--skip-kicad"], check=True, capture_output=True)
            neutral = list(csv.DictReader(io.StringIO((out / "out/lcsc_batch_1-cpl.csv").read_text())))
            supplier = {r["Designator"]: r for r in csv.DictReader(io.StringIO((out / "out/lcsc_batch_1-lcsc-cpl.csv").read_text()))}
            bom = (out / "out/lcsc_batch_1-lcsc-bom.csv").read_text()
        self.assertEqual(len(neutral), 24)
        self.assertIn('"STM32H743VIH6TR","U17,U18,U19,U20"', bom)
        turned = {"U%d" % i for i in range(1, 13)}  # LQFP100 x2 and LQFP208: rotate 270
        for row in neutral:
            ref, angle = row["Designator"], float(row["Rotation"])
            delta = 270 if ref in turned else 0
            # Top: authoring angle plus the mapping. Bottom: the assembler's
            # mirrored reading of the composed KiCad angle.
            expected = (angle + delta) % 360 if row["Layer"] == "Top" else (180 - angle + delta) % 360
            with self.subTest(ref=ref):
                self.assertEqual(float(supplier[ref]["Rotation"]), expected)
                self.assertEqual(supplier[ref]["Mid X"], row["Mid X"])

    def test_c2_board_3_manifest_is_its_generators_output_with_confirmed_parts(self):
        committed = ROOT / "tools/frame_boards/c2-board-3/manifest.json"
        with tempfile.TemporaryDirectory() as td:
            subprocess.run(["python3", "-I", str(ROOT / "tools/frame_boards/c2_board_3.py"), str(Path(td) / "m.json")],
                           check=True, capture_output=True)
            self.assertEqual((Path(td) / "m.json").read_bytes(), committed.read_bytes())
        data = ROOT / "tools/sourcing_data"
        pages = {r["c"]: r for r in json.load(open(data / "jlc_catalog/2026-10-30/catalog.json"))}
        pages.update({r["c"]: r for r in json.load(open(data / "lcsc_backfill/2026-10-30-sample200/confirmed.json"))})
        mapped = {r["pn"]: r for r in json.load(open(data / "lcsc_map.json"))["rows"]}
        aliases = json.load(open(data / "mfr_aliases.json"))["aliases"]
        manifest = json.loads(committed.read_text())
        rows, dropped = manifest["rows"], manifest["dropped"]
        self.assertEqual(len({r["pn"] for r in rows}), len(rows))
        # The passive batch 2 board shares this preview: each of its part numbers is
        # here, or listed as dropped with a reason.
        passives = json.loads((ROOT / "tools/frame_boards/lcsc-batch-2-passive/manifest.json").read_text())["rows"]
        self.assertLessEqual({r["pn"] for r in passives}, {r["pn"] for r in rows} | set(dropped))
        self.assertFalse(set(dropped) & {r["pn"] for r in rows})
        self.assertTrue(all(dropped.values()))
        for row in rows:
            with self.subTest(row=row["name"]):
                # Every part number is a confirmed page, with that page's identity;
                # a mapped one takes the map row's mfr, the page's after alias review.
                page = pages[row["pn"]]
                mfr = aliases.get(page["mfr"], page["mfr"])
                if row["pn"] in mapped:
                    self.assertEqual(mapped[row["pn"]]["mfr"], mfr)
                else:
                    mfr = page["mfr"]
                self.assertEqual((row["mfr"], row["mpn"], row["jlc_package"]), (mfr, page["mpn"], page["package"]))
                if row["note"].startswith("map row"):
                    self.assertEqual(mapped[row["pn"]]["mpn"], row["mpn"])
                # Four copies, or one top copy where stock allows only five parts.
                sides = [(p["rotate"], p["side"]) for p in row["placements"]]
                self.assertIn(sides, ([(0, "top"), (37, "top"), (0, "bottom"), (37, "bottom")], [(0, "top")]))
                self.assertNotIn("mapping", row)

    @unittest.skipUnless(COHDL.exists() or REQUIRE_BINARY, "target/debug/cohdl is not built")
    def test_c2_board_3_board_carries_no_supplier_mapping(self):
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "board"
            subprocess.run(["python3", "-I", str(ROOT / "tools/jlc_frame_board.py"),
                            str(ROOT / "tools/frame_boards/c2-board-3/manifest.json"), "-o", str(out),
                            "--cohdl", str(COHDL), "--skip-kicad"], check=True, capture_output=True)
            neutral = list(csv.DictReader(io.StringIO((out / "out/c2_frame_board_3-cpl.csv").read_text())))
            supplier = {r["Designator"]: r for r in csv.DictReader(io.StringIO((out / "out/c2_frame_board_3-lcsc-cpl.csv").read_text()))}
            bom = list(csv.DictReader(io.StringIO((out / "out/c2_frame_board_3-lcsc-bom.csv").read_text())))
        self.assertEqual(len(neutral), 109)
        self.assertEqual(len(bom), 49)
        for row in neutral:
            angle = float(row["Rotation"])
            expected = angle if row["Layer"] == "Top" else (180 - angle) % 360
            with self.subTest(ref=row["Designator"]):
                self.assertEqual(float(supplier[row["Designator"]]["Rotation"]), expected)
                self.assertEqual(supplier[row["Designator"]]["Mid X"], row["Mid X"])

    def test_cpl_check_compares_position_side_and_top_angle(self):
        cpl = ("Designator,Mid X,Mid Y,Layer,Rotation\n"
               "U1,1.0000mm,2.0000mm,Top,37\nU2,3.0000mm,4.0000mm,Bottom,217\n")
        kicad = "Ref,Val,Package,PosX,PosY,Rot,Side\nU1,x,p,1.0,2.0,37.0,top\nU2,x,p,3.0,4.0,143.0,bottom\n"
        self.assertEqual(check_cpl.compare_positions(kicad, cpl), [])
        moved = kicad.replace("1.0,2.0,37.0", "1.001,2.0,38.0").replace("bottom", "top")
        bad = check_cpl.compare_positions(moved + "U3,x,p,0,0,0,top\n", cpl)
        self.assertEqual(bad, ["U1: position 1.0,2.0 against KiCad 1.001,2.0",
                               "U1: angle 37 against KiCad 38.0",
                               "U2: side Bottom against KiCad top",
                               "U3: only in the KiCad export"])

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
