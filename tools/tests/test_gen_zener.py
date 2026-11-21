import copy
import io
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import gen_zener as gz

COMPILER = gz.ROOT / "target" / "debug" / "cohdl"


def data():
    return json.loads(gz.DATA.read_text(encoding="utf-8"))


class GenZenerTests(unittest.TestCase):
    def test_the_committed_source_is_the_generators_output(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as stderr:
            self.assertEqual(gz.main(["--check"]), 0, stderr.getvalue())

    def test_one_part_per_sheet_row_on_the_shared_land(self):
        d = data()
        text = gz.OUT.read_text(encoding="utf-8")
        rows = [r for s in d["series"] for r in s["parts"]]
        self.assertEqual({s["key"]: len(s["parts"]) for s in d["series"]},
                         {"bzt52c": 38, "bzt52b": 33, "mmsz": 47, "mm1z": 39, "mm1w": 50})
        self.assertEqual(text.count("pub part D_"), len(rows))
        self.assertEqual(text.count("footprint: sod::SOD3716X135N"), len(rows))
        self.assertNotIn("supplier", re.sub(r"//.*", "", text))
        self.assertIn('pub part D_BZT52C5V6: Zener<5.6V, 350mW> {', text)
        self.assertIn('pub part D_MMSZ5232B: Zener<5.6V, 500mW> {', text)
        self.assertIn('pub part D_MM1W330: Zener<330V, 1W> {', text)
        self.assertIn('pub part D_MM1Z2V0B: Zener<2V, 500mW> {', text)
        self.assertIn("required Cathode: 1 [passive]", text)

    def test_literals(self):
        self.assertEqual(gz.literal("5.60", "V"), "5.6V")
        self.assertEqual(gz.literal("10", "V"), "10V")
        self.assertEqual(gz.literal("75.0", "V"), "75V")
        self.assertEqual(gz.power(350), "350mW")
        self.assertEqual(gz.power(1000), "1W")

    def test_it_fails_closed(self):
        cases = {
            "twice": lambda d: d["series"][0]["parts"].append(dict(d["series"][0]["parts"][0])),
            "is not min < nom < max": lambda d: d["series"][0]["parts"][0].update(vz_max_v="1.0"),
            "encodes": lambda d: d["series"][0]["parts"][0].update(vz_nom_v="2.2", vz_max_v="2.3"),
            "numbering": lambda d: d["series"][0]["parts"][0].update(mpn="BZT52X2V0"),
            "no parts": lambda d: d["series"][1].update(parts=[]),
        }
        for message, change in cases.items():
            with self.subTest(message=message):
                d = copy.deepcopy(data())
                change(d)
                with self.assertRaises(SystemExit) as raised:
                    gz.render(d)
                self.assertIn(message, str(raised.exception))

    @unittest.skipUnless(COMPILER.exists() or os.environ.get("COHDL_REQUIRE_BINARY"),
                         "needs target/debug/cohdl")
    def test_the_output_is_in_canonical_form(self):
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "zener.cohdl").write_text(gz.render(data()), encoding="utf-8")
            result = subprocess.run([str(COMPILER), "fmt", directory, "--check"],
                                    capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
