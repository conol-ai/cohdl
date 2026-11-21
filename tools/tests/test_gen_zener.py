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
                         {"bzt52c": 38, "bzt52b": 33, "mmsz": 47, "mm1z": 39, "mm1w": 50,
                          "mm1z_plain": 38, "mmsz46": 40})
        self.assertNotIn("pub part D_MM1Z43:", text)  # printed twice with different values
        self.assertIn("pub part D_MMSZ4685: Zener<3.6V, 500mW> {", text)
        self.assertIn("IZT 50uA", text)
        self.assertEqual(text.count("pub part D_"), len(rows))
        self.assertEqual(text.count("footprint: sod::SOD3716X135N"), len(rows))
        self.assertEqual(len(re.findall(r'supplier lcsc "C\d+" \}', text)), 98)
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
            "both a part and omitted": lambda d: d["series"][0].update(
                omitted=[{"mpn": d["series"][0]["parts"][0]["mpn"], "reason": "x"}]),
        }
        for message, change in cases.items():
            with self.subTest(message=message):
                d = copy.deepcopy(data())
                change(d)
                with self.assertRaises(SystemExit) as raised:
                    gz.render(d)
                self.assertIn(message, str(raised.exception))

    def test_bound_exactly_the_part_numbers_read_with_the_cathode_at_pad_1(self):
        # Boards 8 and 9 read each part number on four copies with no mapping. Only those
        # read "cathode at pad 1" are bound, with no rotation, each through one row of
        # jlc_frames.json on the land the boards placed, citing its record; the reversed
        # ones wait for board 10, and the undrawn one and the two left out are not bound.
        text = gz.OUT.read_text(encoding="utf-8")
        bound = dict(re.findall(r'mpn: "(\w+)",[^}]*?supplier lcsc "(C\d+)" \}', text))
        self.assertTrue(all("rotate" not in line for line in text.splitlines() if "supplier" in line))
        frames = json.loads((gz.ROOT / "tools/sourcing_data/jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        by_pn = {}
        for row in frames:
            by_pn.setdefault(row["pn"], []).append(row)
        sod = re.search(r'^version = "([^"]+)"', (gz.ROOT / "lib/sod/cohdl.toml").read_text(), re.M)[1]
        readings = {}
        for board in (8, 9):
            record = f"tools/sourcing_data/jlc_previews/2026-11-21-board-{board}"
            manifest = json.loads((gz.ROOT / record / "package/manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["dependencies"]["sod"], sod)
            for part in json.loads((gz.ROOT / record / "readings.json").read_text(encoding="utf-8"))["parts"]:
                readings[part["pn"]] = (part["mpn"], part["reading"], record + "/README.md")
        self.assertEqual(len(readings), 124)
        right = {pn: v for pn, v in readings.items() if v[1] == "cathode at pad 1"}
        self.assertEqual(len(right), 98)
        self.assertEqual(sorted(bound.items()), sorted((mpn, pn) for pn, (mpn, _, _) in right.items()))
        for pn, (mpn, _, record) in right.items():
            with self.subTest(pn=pn):
                [row] = by_pn[pn]
                self.assertEqual((row["package"], row["package_version"], row["footprint"]),
                                 ("sod", sod, "SOD3716X135N"))
                self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0})
                self.assertEqual(row["evidence"]["record"], record)
                self.assertIn(mpn + ", polarity read by eye with no mapping", row["evidence"]["note"])
                self.assertIn("not: a measured angle", row["evidence"]["note"])
        for pn, (_, reading, _) in readings.items():
            if reading != "cathode at pad 1":
                self.assertNotIn(f'"{pn}"', text)

    @unittest.skipUnless(os.environ.get("COHDL_REMEASURE_PREVIEWS"), "set COHDL_REMEASURE_PREVIEWS=1")
    def test_the_body_readings_reproduce(self):
        script = gz.ROOT / "tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py"
        for board in (8, 9):
            record = gz.ROOT / f"tools/sourcing_data/jlc_previews/2026-11-21-board-{board}"
            with self.subTest(board=board), tempfile.TemporaryDirectory() as directory:
                out = Path(directory, "body.json")
                subprocess.run([sys.executable, "-I", "-B", str(script), str(record), str(out)], check=True)
                self.assertEqual(out.read_bytes(), (record / "body.json").read_bytes())

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
