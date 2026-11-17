import csv
import hashlib
import io
import json
import math
import os
import re
import shutil
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

    W3_PILOT = ROOT / "tools/frame_boards/w3_pilot_0402.py"
    W3_GROUPS = (("Yageo", "RC"), ("Vishay", "CRCW"), ("Yageo", "CC"))

    def w3_samples(self, report):
        return [pn for mfr, series in self.W3_GROUPS for g in report["groups"]
                if g["footprint"] == "CHIP_0402" and (g["mfr"], g["series"]) == (mfr, series) for pn in g["sample"]]

    def assert_w3_pilot_rows(self, rows, samples):
        self.assertEqual([r["pn"] for r in rows], samples)
        for row in rows:
            with self.subTest(pn=row["pn"]):
                self.assertEqual(row["footprint"], "passive::CHIP_0402")
                self.assertEqual(row["placements"], [{"at_mm": row["placements"][0]["at_mm"], "rotate": 0, "side": "top"}])
                self.assertNotIn("mapping", row)
        jb.expand(rows, 3)  # no two parts overlap

    def test_w3_pilot_record_rebuilds_from_its_frozen_inputs(self):
        # Historical record: the 2026-11-07 preview was uploaded from this manifest,
        # so it rebuilds from the frozen inputs and the report and pages they hash,
        # whatever the current lib/passive is.
        board = ROOT / "tools/frame_boards/w3-pilot-0402"
        inputs = json.loads((board / "inputs.json").read_text())
        report_path = ROOT / inputs["report"]["path"]
        self.assertEqual(report_path, ROOT / "tools/sourcing_data/w3_sampling/2026-11-07/groups.json")
        report = json.loads(report_path.read_text())
        for name in ("report", "pages"):
            self.assertEqual(hashlib.sha256((ROOT / inputs[name]["path"]).read_bytes()).hexdigest(),
                             inputs[name]["sha256"])
        pages = Path(inputs["pages"]["path"]).relative_to("tools/sourcing_data")
        self.assertEqual(report["sources"][str(pages)], inputs["pages"]["sha256"])
        self.assertEqual(inputs["dependencies"], {"std": "0.3.0", "passive": report["sources"]["lib/passive"]})
        self.assertEqual(report["sources"]["lib/passive"], "0.2.5")
        self.assertEqual({k: v["version"] for k, v in inputs["packages"].items()}, inputs["dependencies"])
        with tempfile.TemporaryDirectory() as td:
            subprocess.run([sys.executable, "-I", str(self.W3_PILOT), str(Path(td) / "m.json")],
                           check=True, capture_output=True)
            self.assertEqual((Path(td) / "m.json").read_bytes(), (board / "manifest.json").read_bytes())
        manifest = json.loads((board / "manifest.json").read_text())
        self.assertEqual(manifest["dependencies"], inputs["dependencies"])
        self.assertEqual(len(manifest["rows"]), 99)
        self.assert_w3_pilot_rows(manifest["rows"], self.w3_samples(report))

    def test_w3_pilot_frozen_inputs_tampered_are_refused(self):
        inputs = json.loads((ROOT / "tools/frame_boards/w3-pilot-0402/inputs.json").read_text())
        with tempfile.TemporaryDirectory() as td:
            for change in ({"report": dict(inputs["report"], sha256="0" * 64)},
                           {"samples": inputs["samples"][1:]},
                           {"dependencies": {"std": "0.3.0", "passive": "0.2.6"}},
                           {"packages": dict(inputs["packages"], passive=dict(inputs["packages"]["passive"], version="0.2.6"))}):
                with self.subTest(change=sorted(change)):
                    (Path(td) / "i.json").write_text(json.dumps(dict(inputs, **change)))
                    run = subprocess.run([sys.executable, "-I", str(self.W3_PILOT), str(Path(td) / "m.json"),
                                          "--inputs", str(Path(td) / "i.json")], capture_output=True, text=True)
                    self.assertNotEqual(run.returncode, 0)

    def test_w3_pilot_generator_follows_a_fresh_report_on_the_current_library(self):
        # Current generator: a report built now from the current tree, frozen
        # against the current library, gives a board of exactly its samples.
        version = re.search(r'^version\s*=\s*"([^"]+)"', (ROOT / "lib/passive/cohdl.toml").read_text(), re.M).group(1)
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            subprocess.run([sys.executable, "-I", str(ROOT / "tools/w3_groups.py"), "-o", str(td / "report")],
                           check=True, capture_output=True)
            subprocess.run([sys.executable, "-I", str(self.W3_PILOT), "--freeze", str(td / "inputs.json"),
                            "--report", str(td / "report/groups.json")], check=True, capture_output=True)
            subprocess.run([sys.executable, "-I", str(self.W3_PILOT), str(td / "m.json"),
                            "--inputs", str(td / "inputs.json")], check=True, capture_output=True)
            report = json.loads((td / "report/groups.json").read_text())
            manifest = json.loads((td / "m.json").read_text())
            self.assertEqual(manifest["dependencies"], {"std": "0.3.0", "passive": version})
            self.assertEqual(report["sources"]["lib/passive"], version)
            self.assert_w3_pilot_rows(manifest["rows"], self.w3_samples(report))
            # The dated report freezes to the committed inputs only on the library it names.
            run = subprocess.run([sys.executable, "-I", str(self.W3_PILOT), "--freeze", str(td / "dated.json")],
                                 capture_output=True, text=True)
            if version == "0.2.5":
                self.assertEqual(run.returncode, 0, run.stderr)
                self.assertEqual((td / "dated.json").read_bytes(),
                                 (ROOT / "tools/frame_boards/w3-pilot-0402/inputs.json").read_bytes())
            else:
                self.assertNotEqual(run.returncode, 0)
                self.assertIn(f"the report is for passive 0.2.5, the library is {version}", run.stderr)

    @unittest.skipUnless(COHDL.exists() or REQUIRE_BINARY, "target/debug/cohdl is not built")
    def test_w3_pilot_record_builds_from_its_pinned_packages_alone(self):
        # The record must build where neither lib/ nor a package cache offers
        # passive 0.2.5: the compiler is copied out of the repository, so it
        # finds no library root, and COHDL_HOME is empty.  Unpinned, the same
        # build cannot resolve it; pinned, it builds all 99 against the lock
        # of the previewed package, and a package that differs from the lock
        # is refused.
        board = ROOT / "tools/frame_boards/w3-pilot-0402"
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            (td / "bin").mkdir()
            (td / "home").mkdir()
            cohdl = td / "bin/cohdl"
            shutil.copy2(COHDL, cohdl)
            env = dict(os.environ, COHDL_HOME=str(td / "home"), COHDL_REGISTRY="http://127.0.0.1:9")
            build = lambda out, *extra: subprocess.run(
                [sys.executable, "-I", str(ROOT / "tools/jlc_frame_board.py"), str(board / "manifest.json"),
                 "-o", str(td / out), "--cohdl", str(cohdl), "--skip-kicad", *extra],
                cwd=td, env=env, capture_output=True, text=True)
            bare = build("bare")
            self.assertNotEqual(bare.returncode, 0)
            self.assertIn('E1102', bare.stderr)
            self.assertIn('passive = "0.2.5"', bare.stderr)

            pinned = build("pinned", "--packages", str(board / "inputs.json"), "--lock", str(board / "cohdl.lock"))
            self.assertEqual(pinned.returncode, 0, pinned.stderr)
            self.assertIn("generated 99 placements", pinned.stdout)
            self.assertEqual((td / "pinned/cohdl.lock").read_bytes(), (board / "cohdl.lock").read_bytes())
            with open(td / "pinned/out/w3_pilot_0402-lcsc-cpl.csv", newline="") as f:
                self.assertEqual(len(list(csv.DictReader(f))), 99)

            lock = (board / "cohdl.lock").read_text()
            (td / "other.lock").write_text(re.sub(r'(name = "passive"\nversion = "0.2.5"\nhash = "sha256:)\w+',
                                                  r"\g<1>" + "0" * 64, lock))
            other = build("other", "--packages", str(board / "inputs.json"), "--lock", str(td / "other.lock"))
            self.assertNotEqual(other.returncode, 0)
            self.assertIn("E1103", other.stderr)

    def test_pinned_packages_must_be_the_manifests_versions_and_trees(self):
        board = ROOT / "tools/frame_boards/w3-pilot-0402"
        manifest = jb.load_manifest(board / "manifest.json")
        inputs = json.loads((board / "inputs.json").read_text())
        packages = inputs["packages"]
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            for name, change in (("version", dict(packages, passive=dict(packages["passive"], version="0.2.6"))),
                                 ("tree", dict(packages, passive=dict(packages["passive"], tree=packages["std"]["tree"]))),
                                 ("missing", {"std": packages["std"]})):
                with self.subTest(name):
                    (td / "p.json").write_text(json.dumps({"packages": change}))
                    with self.assertRaises(jb.BoardError):
                        jb.vendor_packages(manifest, td / "p.json", board / "cohdl.lock", td / name)

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

    BOARD_4 = ROOT / "tools/frame_boards/preview-board-4/manifest.json"
    FOUR_COPIES = [(0, "top"), (37, "top"), (0, "bottom"), (37, "bottom")]
    CALIBRATION = [(a, side) for side in ("top", "bottom") for a in (0, 90, 180, 270, 37)]

    def test_preview_board_4_manifest_is_its_generators_output_with_confirmed_parts(self):
        with tempfile.TemporaryDirectory() as td:
            subprocess.run(["python3", "-I", str(ROOT / "tools/frame_boards/preview_board_4.py"), str(Path(td) / "m.json")],
                           check=True, capture_output=True)
            self.assertEqual((Path(td) / "m.json").read_bytes(), self.BOARD_4.read_bytes())
        data = ROOT / "tools/sourcing_data"
        pages = {r["c"]: r for r in json.load(open(data / "jlc_catalog/2026-10-30/catalog.json"))}
        pages.update({r["c"]: r for r in json.load(open(data / "lcsc_backfill/2026-11-07-samsung-b1/confirmed.json"))})
        mapped = {r["pn"]: r for r in json.load(open(data / "lcsc_map.json"))["rows"]}
        framed = {r["pn"] for r in json.load(open(data / "jlc_frames.json"))["rows"]}
        aliases = json.load(open(data / "mfr_aliases.json"))["aliases"]
        manifest = json.loads(self.BOARD_4.read_text())
        rows, left_off = manifest["rows"], manifest["left_off"]
        self.assertEqual(len({r["pn"] for r in rows}), len(rows))
        self.assertFalse(set(left_off) & {r["pn"] for r in rows})
        self.assertTrue(all(left_off.values()))
        copies = {"A": self.FOUR_COPIES, "B": self.FOUR_COPIES, "E": self.FOUR_COPIES,
                  "C": self.CALIBRATION, "D": [(0, "top")]}
        for row in rows:
            with self.subTest(row=row["name"]):
                # Every part number is a page already in the repository, with enough
                # stock for five boards; a mapped one takes the map row's reviewed mfr.
                page = pages[row["pn"]]
                self.assertGreaterEqual(page["stock"], max(30, 5 * len(row["placements"])))
                mfr = page["mfr"]
                if row["purpose"] == "E":
                    self.assertNotIn(row["pn"], mapped)
                    self.assertNotIn(row["pn"], framed)
                else:
                    mfr = aliases.get(page["mfr"], page["mfr"])
                    self.assertEqual((mapped[row["pn"]]["mfr"], mapped[row["pn"]]["mpn"]), (mfr, page["mpn"]))
                self.assertEqual((row["mfr"], row["mpn"], row["jlc_package"]), (mfr, page["mpn"], page["package"]))
                self.assertEqual([(p["rotate"], p["side"]) for p in row["placements"]], copies[row["purpose"]])
                self.assertNotIn("mapping", row)
                # Only the bound part is placed as the library ships it.
                self.assertEqual("library_part" in row, row["purpose"] == "B")
        self.assertEqual({p: sum(r["purpose"] == p for r in rows) for p in copies},
                         {"A": 12, "B": 1, "E": 10, "C": 10, "D": 51})
        jb.expand(jb.load_manifest(self.BOARD_4)["rows"], 5)  # no two boxes overlap

    def test_preview_board_4_takes_its_samples_from_the_w3_report_and_mixes_groups_in_each_view(self):
        report_path = ROOT / "tools/sourcing_data/w3_sampling/2026-11-10/groups.json"
        self.assertEqual(hashlib.sha256(report_path.read_bytes()).hexdigest(),
                         "5b0f77f2be21eb8cde5b15171d382338ef23f9e26cf6e7cdb35abc5f31929a2c")
        groups = {" ".join(g[k] for k in ("footprint", "type", "mfr", "series")): g
                  for g in json.loads(report_path.read_text())["groups"]}
        member = {m[0]: name for name, g in groups.items() for m in g["member_rows"]}
        manifest = json.loads(self.BOARD_4.read_text())
        rows = manifest["rows"]
        # D is the report's own sample of the two groups, whole and in its order, but for
        # the two the first package's BOM step found short on 2026-11-11 (the w3.json of the
        # board's preview record; the board's own w3.json is gone now that the record exists).
        sampled = ["CHIP_0402 ChipResistor Vishay CRCW", "CHIP_1210 ChipResistor Yageo RC"]
        self.assertFalse((self.BOARD_4.parent / "w3.json").exists())
        short = json.loads((ROOT / "tools/sourcing_data/jlc_previews/2026-11-11-board-4/w3.json")
                           .read_text())["short_at_preview"]
        self.assertEqual(set(short), {"C470029", "C875102"})
        replaced = {"C470029": "C7276637", "C875102": "C728392"}
        self.assertEqual([r["pn"] for r in rows if r["purpose"] == "D"],
                         [replaced.get(pn, pn) for name in sampled for pn in groups[name]["sample"]])
        self.assertEqual([len(groups[name]["sample"]) for name in sampled], [9, 42])
        self.assertFalse(set(short) & {r["pn"] for r in rows})
        self.assertEqual({r["pn"]: r["replaces"] for r in rows if "replaces" in r}, {new: old for old, new in replaced.items()})
        # Each replacement is the short sample's neighbour among the group's eligible
        # members in MPN order (in stock, not yet tried), and not short itself.
        g = groups["CHIP_1210 ChipResistor Yageo RC"]
        tried = {x["pn"] for key in ("passed", "failed", "not_drawn", "direct") for x in g[key]}
        eligible = [m[0] for m in g["member_rows"] if m[0] not in tried and m[5] >= 30]
        for old, new in replaced.items():
            self.assertEqual(abs(eligible.index(old) - eligible.index(new)), 1)
            self.assertIn(short[old], next(r["note"] for r in rows if r["pn"] == new))
        # A chip row names its report group; a part number no group has says null.
        counted = {"top": {}, "bottom": {}}
        for row in rows:
            chip = row["footprint"].startswith("passive::CHIP_")
            with self.subTest(row=row["name"]):
                self.assertEqual("group" in row, chip)
                if not chip:
                    continue
                self.assertEqual(row["group"], member.get(row["pn"]))
                self.assertEqual(row["group"] is None, row["purpose"] == "E")
                if row["group"]:
                    self.assertTrue(row["group"].startswith(row["footprint"].split("::")[1] + " "))
                if row["purpose"] == "D":
                    self.assertIn(row["group"], row["note"])
            for place in row["placements"]:
                view = counted[place["side"]]
                view[row["group"] or ""] = view.get(row["group"] or "", 0) + 1
        self.assertEqual(manifest["view_groups"], counted)
        # Tony's condition on the W3 median (#104): a view's median must include bodies
        # of other groups. Here every group is a minority of its view's chip bodies.
        self.assertEqual({view: sum(c.values()) for view, c in counted.items()}, {"top": 135, "bottom": 84})
        for view, counts in counted.items():
            named = {g: n for g, n in counts.items() if g}
            with self.subTest(view=view):
                self.assertGreaterEqual(len(named), 9)
                self.assertLess(2 * max(named.values()), sum(counts.values()))
        self.assertEqual(counted["top"]["CHIP_1210 ChipResistor Yageo RC"], 42)
        self.assertEqual(counted["top"]["CHIP_0402 ChipResistor Vishay CRCW"], 19)
        self.assertNotIn("CHIP_1210 ChipResistor Yageo RC", counted["bottom"])

    @unittest.skipUnless(COHDL.exists() or REQUIRE_BINARY, "needs target/debug/cohdl")
    def test_preview_board_4_turns_each_calibration_part_through_five_angles_on_both_sides(self):
        manifest = json.loads(self.BOARD_4.read_text())
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "board"
            subprocess.run(["python3", "-I", str(ROOT / "tools/jlc_frame_board.py"), str(self.BOARD_4), "-o", str(out),
                            "--cohdl", str(COHDL), "--skip-kicad"], check=True, capture_output=True)
            supplier = {r["Designator"]: r for r in csv.DictReader(io.StringIO((out / "out/preview_board_4-lcsc-cpl.csv").read_text()))}
            bom = list(csv.DictReader(io.StringIO((out / "out/preview_board_4-lcsc-bom.csv").read_text())))
            maps = json.loads((out / "out/preview_board_4-lcsc-map.json").read_text())
        self.assertEqual(len(supplier), 243)
        self.assertEqual(len(bom), 84)
        # No row is mapped: the supplier CPL shows each land's raw frame.
        self.assertEqual(len(maps["designators"]), 243)
        self.assertEqual({(tuple(d["at"]), d["rotate"]) for d in maps["designators"].values()}, {(("0mm", "0mm"), 0)})
        by_pn = {r["LCSC Part #"]: r["Designator"].split(",") for r in bom}
        for row in manifest["rows"]:
            placed = sorted((supplier[ref]["Layer"], float(supplier[ref]["Rotation"])) for ref in by_pn[row["pn"]])
            # The assembler's bottom angle is the authored one seen from below: -R.
            expected = sorted(("Top", float(p["rotate"])) if p["side"] == "top" else ("Bottom", float(-p["rotate"] % 360))
                              for p in row["placements"])
            with self.subTest(row=row["name"]):
                self.assertEqual(placed, expected)
                if row["purpose"] == "C":
                    self.assertEqual(expected, [("Bottom", 0.0), ("Bottom", 90.0), ("Bottom", 180.0), ("Bottom", 270.0),
                                                ("Bottom", 323.0), ("Top", 0.0), ("Top", 37.0), ("Top", 90.0),
                                                ("Top", 180.0), ("Top", 270.0)])

    BOARD_5 = ROOT / "tools/frame_boards/preview-board-5/manifest.json"
    # pn: (the candidate rotation, the land it is applied on)
    BOARD_5_CANDIDATES = {"C7519": (270, "esd::FP_SOT_23_6"), "C131941": (180, "load_switch::FP_TI_DBV0006A"),
                          "C189961": (270, "contrib_audio_amp::SOP8P65X490X110N")}

    def test_preview_board_5_places_each_candidate_with_its_rotation_applied(self):
        with tempfile.TemporaryDirectory() as td:
            subprocess.run(["python3", "-I", str(ROOT / "tools/frame_boards/preview_board_5.py"), str(Path(td) / "m.json")],
                           check=True, capture_output=True)
            self.assertEqual((Path(td) / "m.json").read_bytes(), self.BOARD_5.read_bytes())
        data = ROOT / "tools/sourcing_data"
        pages = {r["c"]: r for r in json.load(open(data / "lcsc_backfill/2026-11-07-samsung-b1/confirmed.json"))}
        mapped = {r["pn"]: r for r in json.load(open(data / "lcsc_map.json"))["rows"]}
        framed = {r["pn"]: r for r in json.load(open(data / "jlc_frames.json"))["rows"]}
        raw = {m["ref"]: m for m in json.load(open(data / "jlc_previews/2026-11-11-board-4/measurements.json"))}
        with open(data / "jlc_previews/2026-11-11-board-4/package/preview_board_4-lcsc-bom.csv", encoding="utf-8-sig") as f:
            raw_refs = {r["LCSC Part #"]: r["Designator"].split(",") for r in csv.DictReader(f)}
        manifest = json.loads(self.BOARD_5.read_text())
        self.assertEqual({r["pn"]: (r["mapping"]["rotate_deg"], r["footprint"]) for r in manifest["rows"]},
                         self.BOARD_5_CANDIDATES)
        for row in manifest["rows"]:
            with self.subTest(row=row["name"]):
                page = pages[row["pn"]]
                self.assertGreaterEqual(page["stock"], max(30, 5 * len(row["placements"])))
                self.assertEqual((row["stock"], row["page_fetched"]), (page["stock"], page["fetched_date"][:10]))
                self.assertEqual((mapped[row["pn"]]["mfr"], mapped[row["pn"]]["mpn"]), (row["mfr"], row["mpn"]))
                self.assertEqual([(p["rotate"], p["side"]) for p in row["placements"]], self.FOUR_COPIES)
                self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": row["candidate"]["rotate_deg"]})
                self.assertNotIn("library_part", row)
                # Nothing is bound or recorded from this file. The board's own preview confirmed the candidate,
                # and the one frame row for the part number cites that record with the candidate as mapping.
                self.assertEqual((framed[row["pn"]]["evidence"]["record"], framed[row["pn"]]["mapping"]),
                                 ("tools/sourcing_data/jlc_previews/2026-11-14-board-5/README.md", row["mapping"]))
                # The candidate is what board 4 read raw on its four copies (top 0, top 37, bottom 0, bottom 37),
                # to the nearest quarter turn: a bottom copy reads the mirror of a top one.
                read = [raw[ref]["delta_ccw_from_top"] for ref in raw_refs[row["pn"]]]
                self.assertEqual(list(row["candidate"]["read_raw_deg"].values()), read)
                turn = row["mapping"]["rotate_deg"]
                for value, expected in zip(read, (360 - turn, 360 - turn, turn, turn)):
                    self.assertLessEqual(abs((value - expected + 180) % 360 - 180), 15)
        # The copies stand clear of each other: 16 mm apart, where the largest land's courtyard is under 8 mm across.
        spots = [tuple(p["at_mm"]) + (p["side"],) for r in manifest["rows"] for p in r["placements"]]
        self.assertEqual(len(set(spots)), 12)
        self.assertGreaterEqual(min(math.hypot(a[0] - b[0], a[1] - b[1]) for a in spots for b in spots if a != b), 16)
        jb.expand(jb.load_manifest(self.BOARD_5)["rows"], 5)  # no two boxes overlap

    def test_preview_board_5_supplier_cpl_carries_the_candidate_rotations(self):
        manifest = json.loads(self.BOARD_5.read_text())
        with tempfile.TemporaryDirectory() as td:
            out = Path(td) / "board"
            subprocess.run(["python3", "-I", str(ROOT / "tools/jlc_frame_board.py"), str(self.BOARD_5), "-o", str(out),
                            "--cohdl", str(COHDL), "--skip-kicad"], check=True, capture_output=True)
            supplier = {r["Designator"]: r for r in csv.DictReader(io.StringIO((out / "out/preview_board_5-lcsc-cpl.csv").read_text()))}
            neutral = {r["Designator"]: r for r in csv.DictReader(io.StringIO((out / "out/preview_board_5-cpl.csv").read_text()))}
            bom = list(csv.DictReader(io.StringIO((out / "out/preview_board_5-lcsc-bom.csv").read_text())))
            maps = json.loads((out / "out/preview_board_5-lcsc-map.json").read_text())
        self.assertEqual((len(supplier), len(bom)), (12, 3))
        by_pn = {r["LCSC Part #"]: r["Designator"].split(",") for r in bom}
        for row in manifest["rows"]:
            turn = self.BOARD_5_CANDIDATES[row["pn"]][0]
            refs = by_pn[row["pn"]]
            with self.subTest(row=row["name"]):
                self.assertEqual({(tuple(maps["designators"][ref]["at"]), maps["designators"][ref]["rotate"]) for ref in refs},
                                 {(("0mm", "0mm"), turn)})
                # Top: the authored angle plus the candidate. Bottom: the same seen from below.
                self.assertEqual(sorted((supplier[ref]["Layer"], float(supplier[ref]["Rotation"])) for ref in refs),
                                 sorted([("Top", float(turn)), ("Top", float((37 + turn) % 360)),
                                         ("Bottom", float(turn)), ("Bottom", float((turn - 37) % 360))]))
                # Without the mapping the supplier CPL would be the neutral one: the candidate is what differs.
                self.assertEqual(sorted(float(neutral[ref]["Rotation"]) for ref in refs if neutral[ref]["Layer"] == "Top"),
                                 [0.0, 37.0])

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
