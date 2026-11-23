"""tools/gen_sot23_tvs.py: the committed SOT-23 TVS arrays are what the data builds."""
import collections
import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import gen_sot23_tvs as gen  # noqa: E402


class GenSot23TvsTests(unittest.TestCase):
    def setUp(self):
        self.rows = json.loads(gen.DATA.read_text(encoding="utf-8"))["parts"]

    def test_committed_files_are_what_the_data_builds(self):
        self.assertEqual(gen.main(["--check"]), 0)

    def test_circuits_and_counts(self):
        self.assertEqual(collections.Counter(r["config"] for r in self.rows),
                         {"bidirectional": 8, "unidirectional": 3, "steering": 1, "asymmetric": 1})
        text = gen.OUT.read_text(encoding="utf-8")
        self.assertEqual(text.count("pub part "), 13)
        self.assertEqual(text.count("pub part "), text.count("footprint: sot::SOT3P190X290X130N"))
        self.assertNotIn("supplier lcsc", text)
        self.assertIn("TVS2Asym_SOT23<12V, 7V>", text)
        self.assertNotIn('"SM712"', text)

    def test_another_pin_assignment_or_circuit_is_refused(self):
        rows = copy.deepcopy(self.rows)
        rows[0]["pins"] = {"1": "GND", "2": "IO2", "3": "IO1"}
        with self.assertRaisesRegex(SystemExit, "pins"):
            gen.check(rows)
        rows = copy.deepcopy(self.rows)
        rows[0]["config"] = "unknown"
        with self.assertRaisesRegex(SystemExit, "unknown circuit"):
            gen.check(rows)


if __name__ == "__main__":
    unittest.main()
