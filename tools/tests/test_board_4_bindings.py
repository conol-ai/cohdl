"""passive 0.2.8: the seven part numbers bound on the readings of preview board 4."""
import csv
import json
import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RECORD = ROOT / "tools/sourcing_data/jlc_previews/2026-11-11-board-4"
README = "tools/sourcing_data/jlc_previews/2026-11-11-board-4/README.md"
ANGLE_LIMIT = 10.0  # docs/library-supplier-plan.md: axis along the pads within 10 deg, mod 180

# pn: (row id, land, source file, part, entry, manufacturer part, readings file)
BOUND = {
    "C107039": ("pb4-chip0603-c107039", "CHIP_0603", "capacitors_0603", "C_1n5_50V_C0G_0603", "primary", "CC0603JRNPO9BN152", 2),
    "C113780": ("pb4-chip0402-c113780", "CHIP_0402", "capacitors_0402", "C_1n_50V_C0G_0402", "primary", "CC0402JRNPO9BN102", 2),
    "C107006": ("pb4-chip0402-c107006", "CHIP_0402", "capacitors_0402", "C_330p_50V_C0G_0402", "primary", "CC0402JRNPO9BN331", 2),
    "C19666": ("pb4-chip0603-c19666", "CHIP_0603", "capacitors_0603", "C_4u7_16V_X5R_0603", "alt", "CL10A475KO8NNNC", 2),
    "C106255": ("pb4-chip0402-c106255", "CHIP_0402", "capacitors_0402", "C_470n_10V_X5R_0402", "primary", "CC0402KRX5R6BB474", 2),
    "C97018": ("pb4-fplwpn201610u2016metric-c97018", "FP_L_WPN201610U_2016Metric", "inductors", "IND_4U7_WPN201610U",
               "primary", "WPN201610U4R7MT", "model"),
    "C337893": ("pb4-fpl08062016metric-c337893", "FP_L_0806_2016Metric", "inductors", "IND_2U2", "primary",
                "DFE201612E-2R2M=P2", "model"),
}
# Members of the sampled group CHIP_0402 / ChipResistor / Yageo / RC that board 4 did not place.
STILL_INFERRED = ("C477807", "C137990")
READINGS = {2: "remeasure-2026-11-11-method-2.json", "model": "body-model.json"}


def rows():
    return json.loads((ROOT / "tools/sourcing_data/jlc_frames.json").read_text(encoding="utf-8"))["rows"]


def copies():
    """{pn: (land, [ref, ...])} from the BOM of the package that was previewed."""
    with (RECORD / "package/preview_board_4-lcsc-bom.csv").open(encoding="utf-8-sig", newline="") as handle:
        return {row["LCSC Part #"]: (row["Footprint"], [ref.strip() for ref in row["Designator"].split(",")])
                for row in csv.DictReader(handle)}


def spoken(value):
    return "0.0" if abs(value) < 0.05 else "%+.1f" % value


class Board4BindingsTests(unittest.TestCase):
    def test_each_bound_number_has_its_own_row_on_the_board_4_record(self):
        by_pn = {}
        for row in rows():
            if row["package"] == "passive":
                self.assertEqual(row["package_version"], "0.2.8", row["id"])
                by_pn.setdefault(row["pn"], []).append(row)
        zero = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}
        for pn, (row_id, land, *_) in BOUND.items():
            with self.subTest(pn=pn):
                self.assertEqual(len(by_pn[pn]), 1)
                row = by_pn[pn][0]
                self.assertEqual((row["id"], row["footprint"], row["supplier"], row["mapping"], row["measured"]),
                                 (row_id, land, "lcsc", zero, zero))
                self.assertIsNone(row.get("sampled"))
                self.assertEqual((row["evidence"]["method"], row["evidence"]["date"], row["evidence"]["record"]),
                                 ("preview", "2026-11-11", README))
                self.assertIn("known mod 180 only", row["evidence"]["note"])

    def test_every_copy_on_the_record_lies_along_its_land_and_centred(self):
        notes = {row["pn"]: row["evidence"]["note"] for row in rows() if row["id"].startswith("pb4-")}
        placed = copies()
        board = {fp["ref"]: fp for fp in json.loads((RECORD / "package/board.json").read_text(encoding="utf-8"))["fps"]}
        for pn, (_, land, _, _, _, _, how) in BOUND.items():
            with self.subTest(pn=pn):
                footprint, refs = placed[pn]
                self.assertEqual((footprint, len(refs)), (land, 4))
                read = {row["ref"]: row for row in json.loads((RECORD / READINGS[how]).read_text(encoding="utf-8"))}
                self.assertEqual(sorted(read[ref]["side"] for ref in refs), ["bottom", "bottom", "top", "top"])
                self.assertEqual(sorted(board[ref]["angle"] % 180 for ref in refs), [0.0, 0.0, 37.0, 37.0])
                bodies = []
                for ref in refs:
                    self.assertIsNot(read[ref].get("drawn"), False, ref)
                    body = read[ref]["body"]
                    self.assertLessEqual(abs(body["delta_mod_180"]), ANGLE_LIMIT, ref)
                    self.assertLessEqual(body["offset_mm"], body["offset_limit_mm"], ref)
                    if how == 2:  # the tool's own verdict, by the method the note names
                        self.assertEqual((body["method"], read[ref]["body_ok"]), (2, True), ref)
                    bodies.append(body)
                # The note quotes the record's numbers.
                angles = [body["delta_mod_180"] for body in bodies]
                self.assertIn("(%s; top and bottom, 0 and 37 deg)" % ", ".join(refs), notes[pn])
                self.assertIn("4 of 4 copies (%s to %s deg mod 180" % (spoken(min(angles)), spoken(max(angles))), notes[pn])
                self.assertIn("(within %.3f mm, limit %.3f)" % (max(b["offset_mm"] for b in bodies),
                                                              min(b["offset_limit_mm"] for b in bodies)), notes[pn])
                self.assertIn(READINGS[how], notes[pn])
        # The inductor drawn as one block says what its reading does not show.
        self.assertIn("one light block over both pads", notes["C97018"])
        self.assertIn("and no more", notes["C97018"])
        self.assertIn("a terminal on each pad", notes["C337893"])

    def test_the_library_binds_each_number_on_the_entry_the_map_names(self):
        identity = {row["pn"]: (row["mfr"], row["mpn"]) for row in
                    json.loads((ROOT / "tools/sourcing_data/lcsc_map.json").read_text(encoding="utf-8"))["rows"]}
        self.assertIn('version = "0.2.8"', (ROOT / "lib/passive/cohdl.toml").read_text(encoding="utf-8"))
        for pn, (_, land, source, part, entry, mpn, _) in BOUND.items():
            with self.subTest(pn=pn):
                self.assertEqual(identity[pn][1], mpn)
                text = (ROOT / "lib/passive/src" / (source + ".cohdl")).read_text(encoding="utf-8")
                block = re.search(r"^pub part %s:.*?^}" % re.escape(part), text, re.M | re.S).group(0)
                clause = re.search(r'%s \{ mfr: "%s", mpn: "%s"[^}]*\}' % (entry, re.escape(identity[pn][0]), re.escape(mpn)),
                                   block).group(0)
                self.assertIn('supplier lcsc "%s"' % pn, clause)
                self.assertEqual(block.count("supplier lcsc"), 1)
                if entry == "primary":
                    self.assertIn("footprint: %s" % land, clause)
        self.assertEqual(sum(text.count('supplier lcsc "') for text in
                             (path.read_text(encoding="utf-8") for path in (ROOT / "lib/passive/src").glob("*.cohdl"))), 886)

    def test_the_sampled_group_is_as_it_was(self):
        inferred = [row for row in rows() if row.get("sampled")]
        self.assertEqual(len(inferred), 798)
        self.assertEqual({row["sampled"]["group"] for row in inferred}, {"passive-0.2.8-CHIP_0402-ChipResistor-Yageo-RC"})
        by_pn = {row["pn"]: row for row in inferred}
        placed = copies()
        for pn in STILL_INFERRED:
            self.assertIn(pn, by_pn)
            self.assertIsNone(by_pn[pn]["evidence"])
            self.assertNotIn(pn, placed)  # board 4 did not place them: they have no reading of their own
        self.assertFalse(set(BOUND) & set(by_pn))


if __name__ == "__main__":
    unittest.main()
