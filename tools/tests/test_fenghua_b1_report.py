import json
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import fenghua_b1_report as fh  # noqa: E402


class FenghuaReportTests(unittest.TestCase):
    def test_the_ordering_code_decodes_as_the_specification_reads(self):
        self.assertEqual(fh.decode("0805CG102K500NT")[:3], ("0805", 50.0, "C0G"))
        size, volts, diel, farads, tol = fh.decode("0402B104K160NT")
        self.assertEqual((size, volts, diel, float(farads), tol), ("0402", 16.0, "X7R", 1e-7, "K"))
        self.assertEqual(fh.decode("0603X106M6R3NT")[1], 6.3)
        self.assertEqual(float(fh.decode("0402CG1R0C500NT")[3]), 1e-12)
        self.assertEqual(fh.decode("1005CG0R5C250NT")[0], "01005")
        self.assertIsNone(fh.decode("0402BT104K160NT"))  # X7T: no library dielectric
        self.assertIsNone(fh.decode("0402CG6R0C500NT (SMTonly)"))
        self.assertIsNone(fh.decode("CMI201209U2R2KT"))

    def test_the_committed_report_is_what_the_tool_builds_and_writes_nothing_else(self):
        run = subprocess.run([sys.executable, "-I", str(ROOT / "tools/fenghua_b1_report.py"), "--check"],
                             capture_output=True, text=True)
        self.assertEqual(run.returncode, 0, run.stderr)
        report = json.loads(fh.OUT.read_text(encoding="utf-8"))
        self.assertTrue(report["_provenance"]["status"].startswith("dry run"))
        self.assertEqual((len(report["matched"]), len(report["unmatched"]), len(report["not_mlcc"])), (31, 26, 3))
        # A dry run: no generator reads it and the library names no Fenghua part number.
        self.assertNotIn("fenghua_b1", (ROOT / "tools/gen_passive.py").read_text(encoding="utf-8"))
        lib = "".join(p.read_text(encoding="utf-8") for p in (ROOT / "lib/passive/src").glob("*.cohdl"))
        for row in report["matched"].values():
            self.assertNotIn(row["mpn"], lib)
        # Each matched part number carries the tolerance the library pairs with its dielectric.
        for key, row in report["matched"].items():
            self.assertEqual(fh.decode(row["mpn"])[4], fh.TOLERANCE[key.split("|")[2]])


if __name__ == "__main__":
    unittest.main()
