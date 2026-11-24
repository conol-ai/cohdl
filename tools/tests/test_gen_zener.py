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
        self.assertEqual(len(re.findall(r'supplier lcsc "C\d+"( rotate 180)? \}', text)), 123)
        self.assertIn('pub part D_BZT52C5V6: Zener<5.6V, 350mW> {', text)
        self.assertIn('pub part D_MMSZ5232B: Zener<5.6V, 500mW> {', text)
        self.assertIn('pub part D_MM1W330: Zener<330V, 1W> {', text)
        self.assertIn('pub part D_MM1Z2V0B: Zener<2V, 500mW> {', text)
        self.assertIn("required Cathode: 1 [passive]", text)

    def test_the_sod323_and_sod523_parts_sit_on_the_land_their_sheet_names(self):
        d = json.loads(gz.DATA_SMALL.read_text(encoding="utf-8"))
        text = gz.OUT_SMALL.read_text(encoding="utf-8")
        self.assertEqual({s["key"]: (len(s["parts"]), s["package"]["footprint"], s["power_mw"]) for s in d["series"]},
                         {"bzt52c_s": (38, "sod::SOD2613X110N", 200), "bzt52b_s": (31, "sod::SOD2613X110N", 200),
                          "mm3z": (38, "sod::SOD2613X110N", 200), "mmsz_s": (35, "sod::SOD2613X110N", 200),
                          "bzx584c": (31, "sod::SOD1608X70N", 150)})
        self.assertEqual(text.count("pub part D_"), 173)
        self.assertEqual(text.count("footprint: sod::SOD2613X110N"), 142)
        self.assertEqual(text.count("footprint: sod::SOD1608X70N"), 31)
        self.assertNotIn("pub device", text)  # the device is zener.cohdl's
        self.assertEqual(text.count("supplier lcsc"), 25)  # board 20, below
        self.assertIn("pub part D_BZT52C5V6S: Zener<5.6V, 200mW> {", text)
        self.assertIn("pub part D_BZX584C3V3: Zener<3.3V, 150mW> {", text)
        self.assertIn("pub part D_MMSZ5231BS: Zener<5.1V, 200mW> {", text)
        # The two transcription corrections keep what the sheet printed.
        rows = {r["mpn"]: r for s in d["series"] for r in s["parts"]}
        self.assertEqual(rows["BZT52B3V0S"]["printed_mpn"], "BZT52B3VOS")
        self.assertEqual((rows["MM3Z56"]["tc_max_mv_per_c"], rows["MM3Z56"]["printed"]),
                         ("63.8", {"tc_max_mv_per_c": "63..8"}))
        # MM3Z orders ZZT before IZT; the comment reads the named columns.
        self.assertIn("VZ 1.91-2.09V (nom 2.0V) at IZT 5mA; ZZT 100ohm; IR 150uA at 1.0V; marking WY", text)

    def test_every_b1_sod323_and_sod523_zener_is_a_part(self):
        catalog = json.loads((gz.ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json").read_text(
            encoding="utf-8"))
        b1 = {r["mpn"]: r["package"] for r in catalog if r["mfr"] == "hongjiacheng"
              and r["package"] in ("SOD-323", "SOD-523") and re.match(r"(BZT52|MM3Z|MMSZ|BZX)", r["mpn"])}
        self.assertEqual(len(b1), 71)
        d = json.loads(gz.DATA_SMALL.read_text(encoding="utf-8"))
        land = {r["mpn"]: s["package"]["footprint"] for s in d["series"] for r in s["parts"]}
        for mpn, package in b1.items():
            with self.subTest(mpn=mpn):
                self.assertEqual(land[mpn], {"SOD-323": "sod::SOD2613X110N", "SOD-523": "sod::SOD1608X70N"}[package])

    def test_preview_board_20_places_every_b1_part_four_times_on_its_land(self):
        sys.path.insert(0, str(gz.ROOT / "tools/frame_boards"))
        import zener_small_boards
        built = zener_small_boards.build("preview_board_20")
        path = gz.ROOT / "tools/frame_boards/preview-board-20/manifest.json"
        self.assertEqual(json.loads(path.read_text(encoding="utf-8")), built)
        self.assertEqual(len(built["rows"]), 71)
        self.assertEqual(built["dependencies"]["diode"],
                         re.search(r'^version = "([^"]+)"', (gz.ROOT / "lib/diode/cohdl.toml").read_text(), re.M)[1])
        for row in built["rows"]:
            self.assertEqual(len(row["placements"]), 4)
            self.assertNotIn("mapping", row)
            self.assertEqual(row["footprint"], zener_small_boards.LANDS[row["jlc_package"]])

    def test_preview_board_21_reverifies_board_20s_reversed_part_numbers_with_rotate_180(self):
        sys.path.insert(0, str(gz.ROOT / "tools/frame_boards"))
        import zener_small_boards
        built = zener_small_boards.build("preview_board_21")
        path = gz.ROOT / "tools/frame_boards/preview-board-21/manifest.json"
        self.assertEqual(json.loads(path.read_text(encoding="utf-8")), built)
        record = "tools/sourcing_data/jlc_previews/2026-11-24-board-20"
        reversed_ = {p["pn"] for p in json.loads((gz.ROOT / record / "readings.json").read_text(encoding="utf-8"))["parts"]
                     if p["reading"] == "reversed"}
        self.assertEqual({row["pn"] for row in built["rows"]}, reversed_)
        self.assertEqual(len(reversed_), 13)
        for row in built["rows"]:
            self.assertEqual(len(row["placements"]), 4)
            self.assertEqual(row["footprint"], "sod::SOD1608X70N")
            self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 180})
            self.assertEqual(row["candidate"], {"rotate_deg": 180, "record": record + "/README.md"})

    def test_an_mpn_in_both_data_files_is_refused(self):
        small = json.loads(gz.DATA_SMALL.read_text(encoding="utf-8"))
        small["series"][0]["parts"].append(dict(data()["series"][0]["parts"][0]))
        real = gz.DATA_SMALL.read_text
        with mock.patch.object(type(gz.DATA_SMALL), "read_text",
                               lambda self, **kw: json.dumps(small) if self == gz.DATA_SMALL else real.__func__(self, **kw)):
            with self.assertRaises(SystemExit) as raised:
                gz.main(["--check"])
        self.assertIn("both data files", str(raised.exception))

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
        # Boards 8 and 9 read each part number on four copies with no mapping; board 10 read
        # again, with rotate 180 applied, the 25 they read reversed. A part number is bound
        # only where every copy showed the cathode at pad 1: with no rotation from boards 8
        # and 9, with rotate 180 from board 10. Each binding goes through one row of
        # jlc_frames.json on the land the boards placed, citing its record. The undrawn
        # one and the four left out are not bound.
        text = gz.OUT.read_text(encoding="utf-8")
        bound = {mpn: (pn, rotate) for mpn, pn, rotate in re.findall(
            r'mpn: "(\w+)",[^}]*?supplier lcsc "(C\d+)"( rotate 180)? \}', text)}
        self.assertEqual(text.count("supplier lcsc"), len(bound))
        frames = json.loads((gz.ROOT / "tools/sourcing_data/jlc_frames.json").read_text(encoding="utf-8"))["rows"]
        by_pn = {}
        for row in frames:
            by_pn.setdefault(row["pn"], []).append(row)
        sod = re.search(r'^version = "([^"]+)"', (gz.ROOT / "lib/sod/cohdl.toml").read_text(), re.M)[1]

        def readings(board):
            record = f"tools/sourcing_data/jlc_previews/2026-11-21-board-{board}"
            manifest = json.loads((gz.ROOT / record / "package/manifest.json").read_text(encoding="utf-8"))
            self.assertEqual(manifest["dependencies"]["sod"], sod)
            parts = json.loads((gz.ROOT / record / "readings.json").read_text(encoding="utf-8"))["parts"]
            return {p["pn"]: (p["mpn"], p["reading"], record + "/README.md") for p in parts}

        first = {**readings(8), **readings(9)}
        again = readings(10)
        self.assertEqual(len(first), 124)
        reversed_ = {pn for pn, v in first.items() if v[1] == "reversed"}
        self.assertEqual(len(reversed_), 25)
        self.assertEqual(set(again), reversed_)
        expected = {pn: (mpn, 0, record) for pn, (mpn, reading, record) in first.items()
                    if reading == "cathode at pad 1"}
        expected.update({pn: (mpn, 180, record) for pn, (mpn, reading, record) in again.items()
                         if reading == "cathode at pad 1"})
        self.assertEqual(len(expected), 123)
        self.assertEqual(bound, {mpn: (pn, " rotate 180" if rotate else "")
                                 for pn, (mpn, rotate, _) in expected.items()})
        for pn, (mpn, rotate, record) in expected.items():
            with self.subTest(pn=pn):
                [row] = by_pn[pn]
                self.assertEqual((row["package"], row["package_version"], row["footprint"]),
                                 ("sod", sod, "SOD3716X135N"))
                self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate})
                self.assertEqual(row["evidence"]["record"], record)
                applied = "rotate 180 applied" if rotate else "no mapping"
                self.assertIn(f"{mpn}, polarity read by eye with {applied}", row["evidence"]["note"])
                self.assertIn("not: a measured angle", row["evidence"]["note"])
        for pn in set(first) - set(expected):
            self.assertNotIn(f'"{pn}"', text)

    def test_board_20_binds_the_mapped_part_numbers_read_with_the_cathode_at_pad_1(self):
        # Board 20 read the 71 SOD-323 and SOD-523 part numbers on four copies each with no
        # mapping. A part number is bound only where every copy showed the cathode at pad 1 AND
        # a reviewed backfill gave it a map row; the 13 BZX584C read reversed wait for a
        # re-verify board, and the unmapped wait for their map rows.
        record = "tools/sourcing_data/jlc_previews/2026-11-24-board-20"
        text = gz.OUT_SMALL.read_text(encoding="utf-8")
        bound = dict(re.findall(r'mpn: "(\w+)",[^}]*?supplier lcsc "(C\d+)" \}', text))
        self.assertEqual(text.count("supplier lcsc"), len(bound))
        parts = json.loads((gz.ROOT / record / "readings.json").read_text(encoding="utf-8"))["parts"]
        self.assertEqual(len(parts), 71)
        self.assertEqual(sorted(p["mpn"] for p in parts if p["reading"] == "reversed"),
                         sorted("BZX584C" + v for v in "10 11 12 13 15 18 39 3V3 3V9 4V3 5V1 5V6 9V1".split()))
        mapped = {r["pn"] for r in json.loads((gz.ROOT / "tools/sourcing_data/lcsc_map.json")
                                              .read_text(encoding="utf-8"))["rows"]}
        expected = {p["mpn"]: p["pn"] for p in parts if p["reading"] == "cathode at pad 1" and p["pn"] in mapped}
        self.assertEqual(len(expected), 25)
        self.assertEqual(bound, expected)
        sod = re.search(r'^version = "([^"]+)"', (gz.ROOT / "lib/sod/cohdl.toml").read_text(), re.M)[1]
        manifest = json.loads((gz.ROOT / record / "package/manifest.json").read_text(encoding="utf-8"))
        self.assertEqual(manifest["dependencies"]["sod"], sod)
        frames = [r for r in json.loads((gz.ROOT / "tools/sourcing_data/jlc_frames.json")
                                        .read_text(encoding="utf-8"))["rows"] if r["pn"] in set(expected.values())]
        self.assertEqual(len(frames), 25)
        land = {p["pn"]: p["land"].split("::")[1] for p in parts}
        for row in frames:
            with self.subTest(pn=row["pn"]):
                self.assertEqual((row["package"], row["package_version"], row["footprint"]), ("sod", sod, land[row["pn"]]))
                self.assertEqual(row["mapping"], {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0})
                self.assertEqual(row["evidence"]["record"], record + "/README.md")
                self.assertIn("polarity read by eye with no mapping", row["evidence"]["note"])

    @unittest.skipUnless(os.environ.get("COHDL_REMEASURE_PREVIEWS"), "set COHDL_REMEASURE_PREVIEWS=1")
    def test_the_body_readings_reproduce(self):
        script = gz.ROOT / "tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py"
        for board in ("2026-11-21-board-8", "2026-11-21-board-9", "2026-11-21-board-10", "2026-11-24-board-20"):
            record = gz.ROOT / "tools/sourcing_data/jlc_previews" / board
            with self.subTest(board=board), tempfile.TemporaryDirectory() as directory:
                out = Path(directory, "body.json")
                subprocess.run([sys.executable, "-I", "-B", str(script), str(record), str(out)], check=True)
                self.assertEqual(out.read_bytes(), (record / "body.json").read_bytes())

    @unittest.skipUnless(COMPILER.exists() or os.environ.get("COHDL_REQUIRE_BINARY"),
                         "needs target/debug/cohdl")
    def test_the_output_is_in_canonical_form(self):
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "zener.cohdl").write_text(gz.render(data()), encoding="utf-8")
            Path(directory, "zener_sod323_sod523.cohdl").write_text(
                gz.render_small(json.loads(gz.DATA_SMALL.read_text(encoding="utf-8"))), encoding="utf-8")
            result = subprocess.run([str(COMPILER), "fmt", directory, "--check"],
                                    capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
