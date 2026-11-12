"""The bindings read on a real board (jlc_previews/2026-11-12-owt-rev-c): three ICs placed with a
rotation, and seven chip parts and two inductors of `passive` placed with none."""

import contextlib
import csv
import hashlib
import io
import json
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "frame_boards"))
import measure_preview as mp

ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tools" / "sourcing_data"
RECORD = DATA / "jlc_previews" / "2026-11-12-owt-rev-c"
BOARD_4 = DATA / "jlc_previews" / "2026-11-11-board-4"

# frame row id: (package, version, footprint, part number, rotate, board footprint id,
#                library source, placements on the real board)
ROWS = {
    "owtc-fpsot236-c7519": (
        "esd", "0.1.3", "FP_SOT_23_6", "C7519", 270, "esd-FP_SOT_23_6",
        "lib/esd/src/usblc6.cohdl", ["U20"]),
    "owtc-fptidbv0006a-c131941": (
        "load-switch", "0.2.1", "FP_TI_DBV0006A", "C131941", 180, "load_switch-FP_TI_DBV0006A",
        "lib/load-switch/src/tps22918.cohdl", ["U2", "U10"]),
    "owtc-sop8p65x490x110n-c189961": (
        "@contrib/audio-amp", "0.2.1", "SOP8P65X490X110N", "C189961", 270,
        "contrib_audio_amp-SOP8P65X490X110N", "lib/@contrib/audio-amp/src/ns4150b.cohdl", ["U1"]),
}

PASSIVE_VERSION = "0.2.8"
# frame row id: (footprint, part number, manufacturer, manufacturer part, library source, library part,
#                entry kind, placements on the real board, board 4 footprint or None when not on it)
PASSIVE = {
    "owtc-chip0603-c107039": (
        "CHIP_0603", "C107039", "Yageo", "CC0603JRNPO9BN152", "capacitors_0603.cohdl", "C_1n5_50V_C0G_0603",
        "primary", ["C2"], "CHIP_0603"),
    "owtc-chip0402-c113780": (
        "CHIP_0402", "C113780", "Yageo", "CC0402JRNPO9BN102", "capacitors_0402.cohdl", "C_1n_50V_C0G_0402",
        "primary", ["C6", "C32", "C33", "C34", "C35", "C36", "C37", "C38", "C39", "C50"], "CHIP_0402"),
    "owtc-chip0402-c107006": (
        "CHIP_0402", "C107006", "Yageo", "CC0402JRNPO9BN331", "capacitors_0402.cohdl", "C_330p_50V_C0G_0402",
        "primary", ["C8", "C9", "C10"], "CHIP_0402"),
    "owtc-chip0603-c19666": (
        "CHIP_0603", "C19666", "Samsung", "CL10A475KO8NNNC", "capacitors_0603.cohdl", "C_4u7_16V_X5R_0603",
        "alt", ["C18", "C40", "C41", "C48"], "CHIP_0603"),
    "owtc-chip0402-c106255": (
        "CHIP_0402", "C106255", "Yageo", "CC0402KRX5R6BB474", "capacitors_0402.cohdl", "C_470n_10V_X5R_0402",
        "primary", ["C25"], "CHIP_0402"),
    "owtc-chip0402-c477807": (
        "CHIP_0402", "C477807", "Yageo", "RC0402FR-0786K6L", "resistors_0402.cohdl", "R_86K6_F_0402",
        "primary", ["R31"], None),
    "owtc-chip0402-c137990": (
        "CHIP_0402", "C137990", "Yageo", "RC0402FR-073K6L", "resistors_0402.cohdl", "R_3K6_F_0402",
        "primary", ["R37"], None),
    "owtc-fplwpn201610u2016metric-c97018": (
        "FP_L_WPN201610U_2016Metric", "C97018", "Sunlord", "WPN201610U4R7MT", "inductors.cohdl",
        "IND_4U7_WPN201610U", "primary", ["L3"], "FP_L_WPN201610U_2016Metric"),
    "owtc-fpl08062016metric-c337893": (
        "FP_L_0806_2016Metric", "C337893", "Murata", "DFE201612E-2R2M=P2", "inductors.cohdl", "IND_2U2",
        "primary", ["L4", "L5"], "FP_L_0806_2016Metric"),
}
W3_RECORD = DATA / "w3_records" / "2026-11-10-chip0402-yageo-rc.json"
# Part numbers the same preview read on `passive` lands that are NOT bound from it: the ten
# UNI-ROYAL resistors (no library entry yet) and two MLCCs with no identity row in lcsc_map.json.
NOT_BOUND = ["C11702", "C25741", "C25768", "C25092", "C26083", "C25905", "C25744", "C25890", "C17168", "C25760",
             "C15849", "C97946"]


def off_pad_1(delta):
    """Degrees between the pin 1 mark and pad 1, 0..180."""
    return min(delta % 360, 360 - delta % 360)


class OwtRevCRecordTests(unittest.TestCase):
    def setUp(self):
        frames = json.loads((DATA / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        self.frames = {row["id"]: row for row in frames}
        self.extract = {fp["ref"]: fp for fp in json.loads((RECORD / "board-extract.json").read_text())["fps"]}
        self.readings = {row["ref"]: row for row in json.loads((RECORD / "measurements.json").read_text())}

    def test_each_row_is_the_binding_and_cites_this_record(self):
        for row_id, (package, version, footprint, pn, rotate, _, source, _) in ROWS.items():
            with self.subTest(row=row_id):
                row = self.frames[row_id]
                self.assertEqual(
                    (row["package"], row["package_version"], row["footprint"], row["supplier"], row["pn"]),
                    (package, version, footprint, "lcsc", pn))
                mapping = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate}
                self.assertEqual((row["mapping"], row["measured"]), (mapping, mapping))
                self.assertEqual(row["evidence"]["record"], str((RECORD / "README.md").relative_to(ROOT)))
                self.assertIn(str((BOARD_4 / "README.md").relative_to(ROOT)), row["evidence"]["note"])
                self.assertIn(f'supplier lcsc "{pn}" rotate {rotate} }}', (ROOT / source).read_text(encoding="utf-8"))
                manifest = (ROOT / source).parents[1] / "cohdl.toml"
                self.assertIn(f'version = "{version}"', manifest.read_text(encoding="utf-8"))

    def test_with_the_mapping_applied_pin_1_reads_on_pad_1(self):
        cpl = (RECORD / "lcsc-cpl-rows.csv").read_text().splitlines()
        sent = {line.split(",")[0]: int(line.split(",")[4]) for line in cpl[1:]}
        refs = []
        for row_id, (_, _, _, _, rotate, fpid, _, placements) in ROWS.items():
            for ref in placements:
                with self.subTest(row=row_id, ref=ref):
                    fp, reading = self.extract[ref], self.readings[ref]
                    self.assertEqual(fp["fpid"], fpid)
                    self.assertEqual(len(reading["dots"]), 1)
                    # 15 deg: a mark drawn beside a SOT-23-6 lead reads 10-12 deg off; a wrong
                    # quarter reads 78 deg or more.
                    self.assertLess(off_pad_1(reading["delta_ccw_from_top"]), 15)
                    # The CPL row that was previewed carries the mapping: top is the board angle
                    # plus rotate, bottom (mirrored convention) 180 minus the board angle plus rotate.
                    base = 180 - fp["angle"] if fp["bottom"] else fp["angle"]
                    self.assertEqual(sent[ref], (base + rotate) % 360)
                    refs.append(ref)
        # Every placement of the record that is not on a `passive` land is one of these.
        others = sorted(ref for ref, fp in self.extract.items() if not fp["fpid"].startswith("passive-"))
        self.assertEqual(sorted(refs), others)
        self.assertEqual(sorted(self.readings), sorted(self.extract))

    def body_of(self, ref):
        """A passive placement's method 2 body reading and whether the tool passes it."""
        reading = self.readings[ref]
        if mp.is_chip(self.extract[ref]):
            return reading["body"], reading["body_ok"]
        self.assertNotIn("body", reading)   # measure reads a body on a CHIP land only
        body = json.loads((RECORD / "inductor-bodies.json").read_text())[ref]
        return body, body["ok"]

    def test_each_passive_row_is_a_zero_binding_on_the_entry_the_map_names(self):
        identity = {(row["mfr"], row["mpn"]): row["pn"]
                    for row in json.loads((DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]}
        zero = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}
        manifest = (ROOT / "lib/passive/cohdl.toml").read_text(encoding="utf-8")
        self.assertIn(f'version = "{PASSIVE_VERSION}"', manifest)
        for row_id, (footprint, pn, mfr, mpn, source, part, kind, _, _) in PASSIVE.items():
            with self.subTest(row=row_id):
                row = self.frames[row_id]
                self.assertEqual(
                    (row["package"], row["package_version"], row["footprint"], row["supplier"], row["pn"]),
                    ("passive", PASSIVE_VERSION, footprint, "lcsc", pn))
                self.assertEqual((row["mapping"], row["measured"], row["exception"]), (zero, zero, None))
                self.assertIsNone(row.get("sampled"))
                self.assertIsNone(row["evidence_ref"])
                self.assertEqual(row["evidence"]["record"], str((RECORD / "README.md").relative_to(ROOT)))
                self.assertIn("mod 180 only", row["evidence"]["note"])
                self.assertEqual(identity[(mfr, mpn)], pn)
                # The one row for this land and part number, so the generator's lookup is unambiguous.
                self.assertEqual([r["id"] for r in self.frames.values()
                                  if (r["package"], r["footprint"], r["pn"]) == ("passive", footprint, pn)], [row_id])
                text = (ROOT / "lib/passive/src" / source).read_text(encoding="utf-8")
                block = text[text.index(f"pub part {part}:"):]
                block = block[:block.index("\n}\n")]
                entry = next(e for e in re.findall(r"\b(?:primary|alt) \{[^}]*\}", block) if f'mpn: "{mpn}"' in e)
                self.assertTrue(entry.startswith(kind + " {"), entry)
                self.assertIn(f'mfr: "{mfr}"', entry)
                self.assertRegex(entry, rf'supplier lcsc "{pn}" \}}$')

    def test_each_passive_placement_reads_along_its_pads_and_centred(self):
        bom = (RECORD / "lcsc-bom-rows.csv").read_text().splitlines()[1:]
        placed = {}
        for line in bom:
            comment, refs, footprint, pn = next(iter(csv.reader([line])))
            placed[pn] = (footprint, refs.split(","))
        cpl = {line.split(",")[0]: line.split(",") for line in (RECORD / "lcsc-cpl-rows.csv").read_text().splitlines()[1:]}
        claimed = []
        for row_id, (footprint, pn, _, _, _, _, _, refs, _) in PASSIVE.items():
            self.assertEqual(placed[pn], (footprint, refs))
            for ref in refs:
                with self.subTest(row=row_id, ref=ref):
                    fp = self.extract[ref]
                    self.assertEqual(fp["fpid"], f"passive-{footprint}")
                    self.assertEqual(len([p for p in fp["pads"] if p["num"]]), 2)
                    body, ok = self.body_of(ref)
                    self.assertEqual(body["method"], 2)
                    self.assertTrue(ok and mp.body_ok(body))
                    self.assertEqual(self.readings[ref]["dots"], [])           # no pin 1 mark of its own
                    # Far inside the tool's limits (10 deg, the land's offset limit): a quarter turn
                    # reads 90 deg, and a model off by one pad reads half the pad pitch.
                    self.assertLessEqual(abs(body["delta_mod_180"]), 1.0)
                    self.assertLessEqual(body["offset_mm"], 0.03)
                    self.assertLess(body["offset_mm"], 0.25 * body["offset_limit_mm"])
                    # Placed with no mapping: the CPL row is the board's own angle (bottom mirrored).
                    base = 180 - fp["angle"] if fp["bottom"] else fp["angle"]
                    self.assertEqual(int(cpl[ref][4]) % 360, base % 360)
                    self.assertEqual(cpl[ref][3], "Bottom" if fp["bottom"] else "Top")
                    claimed.append(ref)
        on_passive = sorted(ref for ref, fp in self.extract.items() if fp["fpid"].startswith("passive-"))
        self.assertEqual(sorted(claimed), on_passive)
        self.assertEqual(len(claimed), 24)

    def test_board_4_read_seven_of_the_passive_part_numbers_the_same_with_no_mapping(self):
        with (BOARD_4 / "package" / "preview_board_4-lcsc-bom.csv").open(encoding="utf-8-sig", newline="") as handle:
            bom = {row["LCSC Part #"]: row for row in csv.DictReader(handle)}
        chips = {row["ref"]: row for row in json.loads((BOARD_4 / "remeasure-2026-11-11-method-2.json").read_text())}
        model = json.loads((BOARD_4 / "body-model.json").read_text())
        seen = 0
        for row_id, (footprint, pn, _, _, _, _, _, _, board_4) in PASSIVE.items():
            with self.subTest(row=row_id):
                note = self.frames[row_id]["evidence"]["note"]
                if board_4 is None:
                    self.assertNotIn(pn, bom)
                    self.assertNotIn("board-4", note)
                    continue
                seen += 1
                self.assertIn(str((BOARD_4 / "README.md").relative_to(ROOT)), note)
                self.assertEqual(bom[pn]["Footprint"], board_4)
                refs = bom[pn]["Designator"].split(",")
                self.assertEqual(len(refs), 4)
                if footprint.startswith("CHIP_"):
                    for ref in refs:
                        body = chips[ref]["body"]
                        self.assertTrue(chips[ref]["body_ok"] and body["method"] == 2)
                        self.assertLessEqual(abs(body["delta_mod_180"]), 3.0)
                        self.assertLessEqual(body["offset_mm"], 0.03)
        self.assertEqual(seen, 7)

    def test_the_two_w3_members_are_bound_by_their_own_rows_not_by_inference(self):
        record = json.loads(W3_RECORD.read_text(encoding="utf-8"))
        group, = record["groups"]
        self.assertEqual((group["package_version"], group["footprint"], group["series"]),
                         (PASSIVE_VERSION, "CHIP_0402", "RC"))
        samples = {s["pn"] for s in group["samples"]}
        inferred = {row["pn"] for row in self.frames.values() if (row.get("sampled") or {}).get("group") == group["id"]}
        self.assertEqual((len(group["members"]), len(samples), len(group["zero_stock"]), len(inferred)), (969, 49, 122, 796))
        for pn in ("C477807", "C137990"):
            with self.subTest(pn=pn):
                self.assertIn(pn, group["members"])
                self.assertNotIn(pn, samples)
                self.assertNotIn(pn, inferred)
                rows = [row for row in self.frames.values() if row["pn"] == pn]
                self.assertEqual(len(rows), 1)
                self.assertIsNotNone(rows[0]["evidence"])
        # Every other member with stock is a sample or inferred, as before.
        self.assertEqual(len(group["members"]) - len(group["zero_stock"]) - len(samples) - len(inferred), 2)

    def test_the_part_numbers_not_bound_from_this_preview_have_no_row_and_no_binding_from_it(self):
        own = str((RECORD / "README.md").relative_to(ROOT))
        cited = {row["pn"] for row in self.frames.values() if (row.get("evidence") or {}).get("record") == own}
        self.assertEqual(cited, {values[3] for values in ROWS.values()} | {values[1] for values in PASSIVE.values()})
        identity = {row["pn"] for row in json.loads((DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]}
        sources = "".join(path.read_text(encoding="utf-8") for path in sorted((ROOT / "lib/passive/src").glob("*.cohdl")))
        readme = (ROOT / "lib/passive/docs/README.md").read_text(encoding="utf-8")
        for pn in NOT_BOUND:
            with self.subTest(pn=pn):
                self.assertNotIn(pn, identity)
                self.assertNotIn(f'supplier lcsc "{pn}"', sources)
                self.assertIn(pn, readme)

    def test_board_4_read_the_same_rotation_with_no_mapping(self):
        fps = {fp["ref"]: fp for fp in json.loads((BOARD_4 / "package" / "board.json").read_text())["fps"]}
        readings = json.loads((BOARD_4 / "measurements.json").read_text())
        for row_id, (_, _, _, _, rotate, fpid, _, _) in ROWS.items():
            with self.subTest(row=row_id):
                copies = [row for row in readings if fps[row["ref"]]["fpid"] == fpid]
                self.assertEqual(sorted(row["side"] for row in copies), ["bottom", "bottom", "top", "top"])
                for row in copies:
                    delta = row["delta_ccw_from_top"]
                    quarter = round(delta / 90) * 90 % 360
                    self.assertLess(abs(delta - round(delta / 90) * 90), 15)
                    # The rotate that cancels a reading is 360 minus it on top and the reading
                    # itself from below.
                    self.assertEqual(quarter if row["side"] == "bottom" else (360 - quarter) % 360, rotate)

    def test_the_readme_hashes_and_the_source_hashes_match_the_files(self):
        readme = (RECORD / "README.md").read_text(encoding="utf-8")
        listed = dict(re.findall(r"^\| `([^`]+)` \| `([0-9a-f]{64})` \|$", readme, re.M))
        files = sorted(str(p.relative_to(RECORD)) for p in RECORD.rglob("*") if p.is_file() and p.name != "README.md")
        self.assertEqual(sorted(listed), files)
        self.assertEqual(len(files), 18)
        for name, digest in listed.items():
            with self.subTest(file=name):
                self.assertEqual(hashlib.sha256((RECORD / name).read_bytes()).hexdigest(), digest)
        source = dict(reversed(line.split("  ")) for line in (RECORD / "source-SHA256SUMS").read_text().splitlines())
        for name in ("exports/top.png", "exports/bottom.png", "crops/U1-top.png", "crops/U2-bottom.png",
                     "crops/U10-bottom.png", "crops/U20-top.png", "crops/L3-bottom.png", "crops/L4-bottom.png",
                     "crops/L5-bottom.png"):
            self.assertEqual(listed[name], source[name])

    @unittest.skipUnless(sys.platform == "darwin" and shutil.which("sips"), "needs macOS sips to convert the exports")
    def test_the_exports_calibrate_on_the_board_and_reproduce_the_readings(self):
        with tempfile.TemporaryDirectory() as td:
            for side in ("top", "bottom"):
                subprocess.run(["sips", "-s", "format", "bmp", str(RECORD / "exports" / f"{side}.png"),
                                "--out", str(Path(td) / f"{side}.bmp")], check=True, capture_output=True)
            # The board's bounding box in each export, as the source record's features.json has it.
            for side, box in (("top", (162, 168, 2263, 3804)), ("bottom", (9, 168, 2110, 3804))):
                view = mp.View(str(Path(td) / f"{side}.bmp"), (0.0, 0.0, 1.0, 1.0), side == "bottom")
                self.assertEqual((view.left, view.top, view.right, view.low), box, side)
            out = Path(td) / "out.json"
            with contextlib.redirect_stdout(io.StringIO()):
                mp.measure(str(RECORD / "board-extract.json"), str(Path(td) / "top.bmp"),
                           str(Path(td) / "bottom.bmp"), str(out))
            self.assertEqual(out.read_bytes(), (RECORD / "measurements.json").read_bytes())
            # The inductors: the same method 2 body reading, which `measure` runs on CHIP lands only.
            extract = json.loads((RECORD / "board-extract.json").read_text())
            outline = mp.outline(extract)
            views = {False: mp.View(str(Path(td) / "top.bmp"), outline, False),
                     True: mp.View(str(Path(td) / "bottom.bmp"), outline, True)}
            bodies = {}
            for fp in extract["fps"]:
                if fp["fpid"].startswith("passive-FP_L_"):
                    reading = mp.body(views[fp["bottom"]], fp)
                    bodies[fp["ref"]] = dict(reading, ok=mp.body_ok(reading))
            bodies = {ref: bodies[ref] for ref in sorted(bodies, key=lambda ref: int(ref[1:]))}
            self.assertEqual(sorted(bodies), ["L3", "L4", "L5"])
            self.assertEqual(json.dumps(bodies, indent=1) + "\n", (RECORD / "inductor-bodies.json").read_text())

    def test_the_record_names_no_local_path(self):
        for path in sorted(p for p in RECORD.rglob("*") if p.is_file()):
            data = path.read_bytes()
            for needle in (b"/Users/", b"/home/", b"/private/"):
                self.assertNotIn(needle, data, f"{path.relative_to(ROOT)} names {needle.decode()}")


if __name__ == "__main__":
    unittest.main()
