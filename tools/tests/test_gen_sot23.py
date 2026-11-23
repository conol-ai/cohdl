"""tools/gen_sot23.py: the committed SOT-23 transistors and MOSFETs are what the data builds."""
import copy
import json
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import gen_sot23  # noqa: E402


class GenSot23Tests(unittest.TestCase):
    def setUp(self):
        self.data = json.loads(gen_sot23.DATA.read_text(encoding="utf-8"))

    def test_committed_files_are_what_the_data_builds(self):
        self.assertEqual(gen_sot23.main(["--check"]), 0)

    def test_every_part_is_on_the_sot23_land_without_a_binding(self):
        for package in ("transistor", "mosfet"):
            text = (ROOT / "lib" / package / "src" / "sot23.cohdl").read_text(encoding="utf-8")
            self.assertEqual(text.count("pub part "), text.count("footprint: sot::SOT3P190X290X130N"))
            self.assertNotIn("supplier", text.split("binding yet")[1])
        counts = {k: sum(r["kind"] == k for r in self.data["parts"]) for k in gen_sot23.DEVICES}
        self.assertEqual(counts, {"npn": 26, "pnp": 19, "nmos": 17, "pmos": 12})

    def _rows(self, change):
        rows = copy.deepcopy(self.data["parts"])
        change(rows)
        return rows

    def test_a_part_whose_sheet_shows_another_pin_order_is_refused(self):
        def swap(rows):
            rows[0]["pins"] = dict(rows[0]["pins"], **{"1": rows[0]["pins"]["2"], "2": rows[0]["pins"]["1"]})
        with self.assertRaisesRegex(SystemExit, "pins"):
            gen_sot23.check(self._rows(swap))

    def test_a_repeated_part_or_a_missing_rating_is_refused(self):
        with self.assertRaisesRegex(SystemExit, "appears twice"):
            gen_sot23.check(self._rows(lambda rows: rows.append(copy.deepcopy(rows[0]))))
        def zero(rows):
            key = "vceo_v" if rows[0]["kind"] in ("npn", "pnp") else "vds_v"
            rows[0]["ratings"][key] = 0
        with self.assertRaisesRegex(SystemExit, "non-positive"):
            gen_sot23.check(self._rows(zero))

    def test_magnitudes_and_units(self):
        self.assertEqual(gen_sot23.literal(0.115, "A"), "115mA")
        self.assertEqual(gen_sot23.literal(-4.2, "A"), "4.2A")
        self.assertEqual(gen_sot23.literal(1000, "mA"), "1A")
        self.assertEqual(gen_sot23.literal(300, "mW"), "300mW")


    def test_preview_boards_15_and_16_place_every_part_four_times(self):
        sys.path.insert(0, str(ROOT / "tools/frame_boards"))
        import sot23_boards
        seen = []
        for board in sot23_boards.BOARDS:
            path = ROOT / "tools/frame_boards" / board.replace("_", "-") / "manifest.json"
            built = sot23_boards.build(board)
            self.assertEqual(json.loads(path.read_text(encoding="utf-8")), built)
            for row in built["rows"]:
                self.assertEqual(len(row["placements"]), 4)
                if board in sot23_boards.REVERIFY:
                    # Boards 18 and 19 place again, with the candidate, what boards 15 to 17 left for manual review.
                    angle = row["candidate"]["rotate_deg"]
                    self.assertIn(angle, (90, 180))
                    self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": angle})
                    self.assertEqual(row["mfr"], "Zhuhai Hongjiacheng Technology")
                    continue
                self.assertNotIn("mapping", row)
                seen.append(row["pn"])
        self.assertEqual([(r["pn"], r["mapping"]["rotate_deg"]) for r in sot23_boards.build("preview_board_18")["rows"]],
                         [("C28646268", 90), ("C28646269", 90), ("C20069147", 90)])
        # Board 19: the 7 BZX84C Zeners a half turn, the other 11 diodes a quarter turn.
        board_19 = {r["pn"]: (r["mpn"], r["mapping"]["rotate_deg"]) for r in sot23_boards.build("preview_board_19")["rows"]}
        self.assertEqual(sorted(pn for pn, (mpn, angle) in board_19.items() if angle == 180),
                         ["C19077467", "C19077468", "C19077469", "C19077470", "C19077471", "C19077472", "C19077473"])
        self.assertTrue(all(mpn.startswith("BZX84C") == (angle == 180) for mpn, angle in board_19.values()))
        self.assertEqual(len(board_19), 18)
        # A reading that is no clean turn stops the build.
        self.assertEqual(sot23_boards.candidate_angle([-81.5, -80.9, 80.5, 80.5]), 90)
        self.assertEqual(sot23_boards.candidate_angle([-163.5, -163.8, 163.6, 162.9]), 180)
        with self.assertRaises(SystemExit):
            sot23_boards.candidate_angle([-81.5, -40.0, 80.5, 80.5])
        with self.assertRaises(SystemExit):
            sot23_boards.candidate_angle([7.0, 8.0, 5.0, 6.0])
        import gen_sot23_diodes
        diodes = json.loads(gen_sot23_diodes.DATA.read_text(encoding="utf-8"))["parts"]
        import gen_sot23_tvs
        tvs = json.loads(gen_sot23_tvs.DATA.read_text(encoding="utf-8"))["parts"]
        expected = [r["c"] for r in self.data["parts"] + diodes + tvs if r["c"] not in sot23_boards.LEFT_OUT]
        self.assertEqual(sorted(seen), sorted(expected))
        self.assertEqual(len(seen), 125)


if __name__ == "__main__":
    unittest.main()
