"""tools/gen_sot23_diodes.py: the committed SOT-23 diodes are what the data builds."""
import collections
import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import gen_sot23_diodes as gen  # noqa: E402


from test_gen_sot23 import bound_as_read  # noqa: E402

class GenSot23DiodesTests(unittest.TestCase):
    def setUp(self):
        self.rows = json.loads(gen.DATA.read_text(encoding="utf-8"))["parts"]

    def test_committed_files_are_what_the_data_builds(self):
        self.assertEqual(gen.main(["--check"]), 0)

    def test_devices_and_counts(self):
        self.assertEqual(collections.Counter(gen.device(r) for r in self.rows), {
            "Zener_SOT23": 15, "DiodePairSeries_SOT23": 5, "Schottky_SOT23": 4,
            "SchottkyPairCommonCathode_SOT23": 3, "SchottkyPairSeries_SOT23": 3,
            "DiodePairCommonCathode_SOT23": 3, "SchottkyPairCommonAnode_SOT23": 2,
            "DiodePairCommonAnode_SOT23": 2, "Diode_SOT23": 2})
        text = gen.OUT.read_text(encoding="utf-8")
        self.assertEqual(text.count("pub part "), len(self.rows))
        self.assertEqual(text.count("pub part "), text.count("footprint: sot::SOT3P190X290X130N"))
        bound, built = bound_as_read("diode")
        self.assertEqual(bound, built)
        # Every diode but 1SS226 (no stock, no reading): the BZX84C Zeners a half turn, 11 others a quarter turn.
        self.assertEqual(len(bound), 38)
        self.assertEqual(sorted(r for _, r in bound).count(180), 7)
        self.assertEqual(sorted(r for _, r in bound).count(90), 11)
        self.assertNotIn("C7502731", {pn for pn, _ in bound})

    def test_a_part_whose_sheet_draws_another_arrangement_is_refused(self):
        rows = copy.deepcopy(self.rows)
        pair = next(r for r in rows if r["config"] == "common_cathode")
        pair["diodes"] = [["3", "1"], ["3", "2"]]
        with self.assertRaisesRegex(SystemExit, "common_cathode arrangement"):
            gen.check(rows)
        rows = copy.deepcopy(self.rows)
        single = next(r for r in rows if r["config"] == "single")
        single["nc"] = []
        with self.assertRaisesRegex(SystemExit, "single arrangement"):
            gen.check(rows)

    def test_zener_rows_are_the_family_sheet_rows(self):
        zeners = {r["mpn"]: r["ratings"] for r in self.rows if r["kind"] == "zener"}
        self.assertEqual((zeners["BZX84C3V3"]["vz_min_v"], zeners["BZX84C3V3"]["vz_max_v"]), ("3.1", "3.5"))
        self.assertEqual((zeners["MMBZ5226B"]["vz_nom_v"], zeners["MMBZ5226B"]["izt_ma"]), ("3.3", "20"))
        self.assertTrue(all(z["pd_mw"] == 300 for z in zeners.values()))


if __name__ == "__main__":
    unittest.main()
