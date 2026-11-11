import io
import json
import re
import sys
import unittest
from decimal import Decimal
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import samsung_b1_alternates as sb

SOURCES = sb.ROOT / "lib/passive/src"


class SamsungB1AlternatesTests(unittest.TestCase):
    def test_the_committed_table_is_the_tools_output(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as stderr:
            self.assertEqual(sb.main(["--check"]), 0, stderr.getvalue())

    def test_decoding_follows_the_catalog(self):
        self.assertEqual(sb.decode("CL10B102KB8NNNC"), ("0603", 50, "X7R", Decimal("1E-9"), "K"))
        self.assertEqual(sb.decode("CL05C100JB5NNNC"), ("0402", 50, "C0G", Decimal("1.0E-11"), "J"))
        self.assertEqual(sb.decode("CL21A475KAQNNNE"), ("0805", 25, "X5R", Decimal("4.7E-6"), "K"))
        self.assertEqual(sb.decode("CL05C1R5BB5NNNC")[3], Decimal("1.5E-12"))
        self.assertIsNone(sb.decode("CL31B102KGFNNNE"))  # voltage code G: no library voltage
        self.assertIsNone(sb.decode("GRM155R71C104KA88D"))

    def test_every_alternate_is_emitted_once_on_its_part(self):
        table = json.loads(sb.OUT.read_text(encoding="utf-8"))
        text = "".join(p.read_text(encoding="utf-8") for p in sorted(SOURCES.glob("capacitors_*.cohdl")))
        for key, row in table["parts"].items():
            with self.subTest(key=key):
                # Once, as a Samsung alternate; a bound one carries its supplier clause.
                self.assertEqual(len(re.findall(r'alt \{ mfr: "Samsung", mpn: "%s"[ ,}]' % re.escape(row["mpn"]), text)), 1)
                self.assertIn(row["category"], ("basic", "preferred"))
        for row in table["unmatched"]:
            self.assertNotIn(f'"{row["mpn"]}"', text)

    def test_a_wrong_tolerance_is_the_reason_whatever_the_combination(self):
        # Review of #83: six +/-20 % numbers were reported under "Yageo primary
        # unconfirmed" by first-match precedence, overstating that bucket by six.
        table = json.loads(sb.OUT.read_text(encoding="utf-8"))
        for row in table["unmatched"]:
            decoded = sb.decode(row["mpn"])
            if decoded is not None and decoded[4] != sb.TOLERANCE[decoded[2]]:
                with self.subTest(mpn=row["mpn"]):
                    self.assertTrue(row["reason"].startswith("tolerance "), row["reason"])
        reasons = [row["reason"] for row in table["unmatched"]]
        self.assertEqual(reasons.count("no library part: its Yageo primary is unconfirmed"), 10)
        self.assertEqual(sum(r.startswith("tolerance ") for r in reasons), 9)
        self.assertEqual(reasons.count("no library part"), 8)
        self.assertEqual(reasons.count("not decoded"), 1)


if __name__ == "__main__":
    unittest.main()
