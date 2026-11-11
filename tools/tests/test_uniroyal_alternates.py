import contextlib
import hashlib
import io
import json
import sys
import tempfile
import unittest
from decimal import Decimal
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import gen_passive
import uniroyal_alternates as ur

SOURCES = ur.ROOT / "lib/passive/src"

# The ten part numbers the OpenWalkieTalkie Rev B order places, with what each must decode to.
OPENWALKIETALKIE = {
    "0402WGF1003TCE": ("100000", "R_100K_F_0402"),
    "0402WGF1001TCE": ("1000", "R_1K_F_0402"),
    "0402WGF1004TCE": ("1000000", "R_1M_F_0402"),
    "0402WGF0000TCE": ("0", "R_0R_J_0402"),
    "0402WGF2202TCE": ("22000", "R_22K_F_0402"),
    "0402WGF5101TCE": ("5100", "R_5K1_F_0402"),
    "0402WGF1002TCE": ("10000", "R_10K_F_0402"),
    "0402WGF220JTCE": ("22", "R_22R_F_0402"),
    "0402WGF3301TCE": ("3300", "R_3K3_F_0402"),
    "0402WGF1803TCE": ("180000", "R_180K_F_0402"),
}


def reencode(mpn):
    decoded = gen_passive.uniroyal_decode(mpn)
    return gen_passive.uniroyal_mpn(decoded["size"], decoded["tolerance"], decoded["mantissa"], decoded["decade"])


class DecodeTests(unittest.TestCase):
    def test_fields(self):
        self.assertEqual(
            gen_passive.uniroyal_decode("0402WGF1002TCE"),
            {"size": "0402", "tolerance": "F", "power": "62.5mW", "ohms": Decimal(10000), "mantissa": 100, "decade": 4},
        )
        for mpn, size, power, ohms in (
            ("0603WAF4022T5E", "0603", "100mW", "40200"),
            ("0805W8F499JT5E", "0805", "125mW", "49.9"),
            ("0603WAF220KT5E", "0603", "100mW", "2.2"),
            ("1206W4F100LT5E", "1206", "250mW", "0.1"),
            ("0603WAF1005T5E", "0603", "100mW", "10000000"),
            ("0805W8F1000T5E", "0805", "125mW", "100"),
        ):
            with self.subTest(mpn=mpn):
                decoded = gen_passive.uniroyal_decode(mpn)
                self.assertEqual((decoded["size"], decoded["power"], decoded["tolerance"]), (size, power, "F"))
                self.assertEqual(decoded["ohms"], Decimal(ohms))
                self.assertEqual(ur.ohms_text(decoded["ohms"]), ohms)

    def test_zero_ohm_has_no_grid_point(self):
        decoded = gen_passive.uniroyal_decode("0603WAF0000T5E")
        self.assertEqual((decoded["ohms"], decoded["mantissa"], decoded["decade"]), (Decimal(0), None, None))

    def test_the_openwalkietalkie_numbers_decode_to_their_library_parts(self):
        verified = gen_passive.load_verified()
        for mpn, (ohms, part) in OPENWALKIETALKIE.items():
            with self.subTest(mpn=mpn):
                result = ur.classify(mpn, verified)
                self.assertEqual((result["ohms"], result["part"], result["size"], result["tolerance"]),
                                 (ohms, part, "0402", "1%"))
                self.assertEqual(reencode(mpn), mpn)
        self.assertEqual({mpn for _, mpn, _ in ur.OPENWALKIETALKIE_REV_B}, set(OPENWALKIETALKIE))

    def test_every_decoded_snapshot_number_round_trips(self):
        rows = ur.catalog_rows()
        decoded = [row for row in rows if gen_passive.uniroyal_decode(row["mpn"]) is not None]
        self.assertEqual((len(rows), len(decoded)), (295, 289))
        for row in decoded:
            with self.subTest(mpn=row["mpn"]):
                self.assertEqual(reencode(row["mpn"]), row["mpn"])
                # The one attribute of the part the snapshot itself records.
                self.assertEqual(gen_passive.uniroyal_decode(row["mpn"])["size"], row["package"])

    def test_malformed_input_is_refused(self):
        for mpn in (
            "", "0402", "0402WGF1002TC", "0402WGF1002TCEE", " 0402WGF1002TCE", "0402WGF1002TCE ",
            "0402WGF1002TCE\n", "0402wgf1002tce", "0402WGF10O2TCE", "0402WGF1.02TCE", "0402WGF-102TCE",
            "0402WGF１002TCE", "0603WAF1602T5E (SMTonly)", "RC0402FR-0710KL", "CL05B104KO5NNNC",
            None, 4021002, b"0402WGF1002TCE", ["0402WGF1002TCE"],
        ):
            with self.subTest(mpn=mpn):
                self.assertIsNone(gen_passive.uniroyal_decode(mpn))

    def test_the_datasheets_own_example(self):
        # p.2 §3: "(Example: 0805 1/8W ±5% 10KΩ T/R-5000)" spelled 0805 W8 J 0103 T 5 E.
        decoded = gen_passive.uniroyal_decode("0805W8J0103T5E")
        self.assertEqual(decoded, {"size": "0805", "tolerance": "J", "power": "125mW", "ohms": Decimal(10000),
                                   "mantissa": 100, "decade": 4})
        self.assertEqual(gen_passive.UNIROYAL_TOLERANCE["J"][0], "5%")
        self.assertEqual(reencode("0805W8J0103T5E"), "0805W8J0103T5E")
        result = ur.classify("0805W8J0103T5E", gen_passive.load_verified())
        self.assertEqual((result["list"], result["part"], result["ohms"], result["tolerance"]),
                         ("alternates", "R_10K_J_0805", "10000", "5%"))

    def test_each_size_takes_the_datasheets_power_and_reel_codes(self):
        # p.2 §2.2 (wattage codes) and §3 (packing quantity per size); p.5 §6 (rating per size).
        for mpn, size, power in (
            ("0201WMF1002TEE", "0201", "50mW"),     # WM = 1/20 W, E = 15,000
            ("0402WGF1002TCE", "0402", "62.5mW"),   # WG = 1/16 W, C = 10,000
            ("0603WAF1002T5E", "0603", "100mW"),    # WA = 1/10 W, 5 = 5,000
            ("0805W8F1002T5E", "0805", "125mW"),    # W8 = 1/8 W
            ("1206W4F1002T5E", "1206", "250mW"),    # W4 = 1/4 W
            ("1210W2F1002T5E", "1210", "500mW"),    # W2 = 1/2 W
            ("201007F1002T4E", "2010", "750mW"),    # 07 = 3/4 W, 4 = 4,000
            ("25121WF1002T4E", "2512", "1W"),       # 1W = 1 W
        ):
            with self.subTest(mpn=mpn):
                decoded = gen_passive.uniroyal_decode(mpn)
                self.assertEqual((decoded["size"], decoded["power"], decoded["ohms"]), (size, power, Decimal(10000)))
                self.assertEqual(reencode(mpn), mpn)
        self.assertEqual(set(gen_passive.UNIROYAL_SIZE), set(gen_passive.R_POWER))

    def test_value_forms_follow_the_tolerance(self):
        # p.2 §3: at 5% a 0 then two significant digits; at 1% three significant digits.
        for mpn, ohms, mantissa, decade in (
            ("0402WGJ0103TCE", "10000", 100, 4),
            ("0402WGJ0472TCE", "4700", 470, 3),
            ("0402WGJ0100TCE", "10", 100, 1),
            ("0402WGJ091JTCE", "9.1", 910, 0),
            ("0402WGJ010JTCE", "1", 100, 0),
            ("0402WGJ0106TCE", "10000000", 100, 7),
            ("0402WGF1005TCE", "10000000", 100, 7),
            ("0402WGF4991TCE", "4990", 499, 3),
        ):
            with self.subTest(mpn=mpn):
                decoded = gen_passive.uniroyal_decode(mpn)
                self.assertEqual((ur.ohms_text(decoded["ohms"]), decoded["mantissa"], decoded["decade"]),
                                 (ohms, mantissa, decade))
                self.assertEqual(reencode(mpn), mpn)
        for mpn in ("0402WGJ1002TCE", "0402WGF0103TCE", "0402WGJ0010TCE", "0402WGJ1000TCE"):
            with self.subTest(mpn=mpn):
                self.assertIsNone(gen_passive.uniroyal_decode(mpn))

    def test_the_resistance_range_of_each_size_bounds_the_value(self):
        # p.5 §6, the ±1% and ±5% columns at the rating decoded for the size.
        for mpn, inside in (
            ("0201WMF100KTEE", True), ("0201WMF999JTEE", True), ("0201WMF100LTEE", False),   # from 1 ohm
            ("0402WGF100LTCE", True), ("0402WGF999MTCE", False),                             # from 0.1 ohm
            ("0603WAF100MT5E", True), ("0603WAF999NT5E", False),                             # from 0.01 ohm
            ("0805W8F100LT5E", True), ("0805W8F500MT5E", False),                             # from 0.1 ohm at 1/8 W
            ("1206W4F100LT5E", True), ("1206W4F500MT5E", False),
            ("25121WF100MT4E", True), ("25121WF100NT4E", False),
            ("0402WGF1005TCE", True), ("0402WGF1015TCE", False), ("0402WGF1006TCE", False),  # to 10 Mohm
            ("0402WGJ0106TCE", True), ("0402WGJ0116TCE", False),
        ):
            with self.subTest(mpn=mpn):
                self.assertEqual(gen_passive.uniroyal_decode(mpn) is not None, inside)

    def test_codes_outside_the_datasheet_or_the_library_are_refused(self):
        for mpn, why in (
            ("0402WGD1002TCE", "D (0.5%) is in the datasheet, not a library grade"),
            ("0402WGG0103TCE", "G (2%) is in the datasheet, not a library grade"),
            ("0402WGG1002TCE", "G (2%) is in the datasheet, not a library grade"),
            ("0402WGK1002TCE", "tolerance letter the datasheet does not define"),
            ("0105WHF1002TDE", "size 01005 is in the datasheet, not a library size"),
            ("181207F1002T4E", "size 1812 is in the datasheet, not a library size"),
            ("1218W4F1002T5E", "size the datasheet does not list"),
            ("0805W4F500MT5E", "0805 at 1/4 W is the low-resistance rating, not the library's"),
            ("1206W3F500MT5E", "1206 at 1/3 W is the low-resistance rating, not the library's"),
            ("2512W1F1002T4E", "1 W is spelled 1W"),
            ("2010W7F1002T4E", "3/4 W is spelled 07"),
            ("0201WMF1002TCE", "reel code of another size"),
            ("0402WAF1002TCE", "power code of another size"),
            ("0603WGF1002T5E", "power code of another size"),
            ("0402XGF1002TCE", "fifth code is not W"),
            ("0402WGF1002T5E", "reel code of another size"),
            ("0603WAF1002TCE", "reel code of another size"),
            ("0402WGF1002TDE", "reel code of another size"),
            ("0402WGF1002RCE", "packaging type other than T"),
            ("0402WGF1002TCX", "special feature other than E"),
            ("0402WGF1009TCE", "power of ten 9"),
            ("0402WGF100RTCE", "power-of-ten letter R"),
            ("0402WGF10R0TCE", "R as a decimal point"),
            ("0402WGF000JTCE", "zero with a submultiple letter"),
            ("0402WGF0001TCE", "zero digits with a power of ten"),
            ("0402WGJ0000TCE", "the jumper lettered J: not in the datasheet and not seen in the catalog"),
            ("4D03WGJ0472T5E", "resistor array"),
            ("4D02WGJ0103TCE", "resistor array"),
        ):
            with self.subTest(mpn=mpn, why=why):
                self.assertIsNone(gen_passive.uniroyal_decode(mpn))

    def test_encoding_refuses_what_decoding_refuses(self):
        for args in (("0402", "D", 100, 4), ("0402", "F", 99, 4), ("0402", "F", 1000, 4),
                     ("0402", "F", 100, 9), ("0402", "F", 100, -2), ("0402", "F", 101, 7),
                     ("0402", "J", 102, 4), ("0402", "J", 110, 7), ("0402", "J", None, None)):
            with self.subTest(args=args), self.assertRaises(ValueError):
                gen_passive.uniroyal_mpn(*args)
        with self.assertRaises(KeyError):
            gen_passive.uniroyal_mpn("1812", "F", 100, 4)

    def test_decoded_power_is_the_librarys_for_every_size(self):
        for size, (_, power, _, _) in gen_passive.UNIROYAL_SIZE.items():
            self.assertEqual(power, gen_passive.R_POWER[size])
            self.assertEqual(ur.milliwatts(power) > 0, True)
        self.assertEqual((ur.milliwatts("62.5mW"), ur.milliwatts("1W")), (Decimal("62.5"), Decimal(1000)))

    def test_every_library_resistor_value_has_one_spelling(self):
        # Each grid point of each grade encodes, and decodes back to itself, in every size whose range holds it.
        for size in gen_passive.UNIROYAL_SIZE:
            for tol in gen_passive.TOLERANCES:
                for mantissa, decade in gen_passive.resistor_values(tol):
                    mpn = gen_passive.uniroyal_mpn(size, tol, mantissa, decade)
                    decoded = gen_passive.uniroyal_decode(mpn)
                    self.assertEqual((decoded["size"], decoded["tolerance"], decoded["mantissa"], decoded["decade"]),
                                     (size, tol, mantissa, decade), mpn)


class DryRunTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = json.loads(ur.OUT.read_text(encoding="utf-8"))
        cls.text = {size: (SOURCES / f"resistors_{size}.cohdl").read_text(encoding="utf-8")
                    for size in gen_passive.R_POWER}

    def test_the_committed_report_is_the_tools_output(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as stderr:
            self.assertEqual(ur.main(["--check"]), 0, stderr.getvalue())

    def test_counts(self):
        counts = self.report["counts"]
        self.assertEqual(
            {name: counts[name] for name in ("snapshot_rows", "alternates", "zero_ohm", "new_values", "not_decoded")},
            {"snapshot_rows": 295, "alternates": 284, "zero_ohm": 4, "new_values": 1, "not_decoded": 6},
        )
        for name in ("alternates", "zero_ohm", "new_values", "not_decoded"):
            self.assertEqual(counts[name], len(self.report[name]))
        self.assertEqual(sum(counts["alternates_by_size"].values()), counts["alternates"])
        self.assertEqual(counts["openwalkietalkie_rev_b"],
                         {"numbers": 10, "placements": 43, "decoded": 10, "in_snapshot": 9})

    def test_each_alternate_names_an_emitted_part_with_the_same_parameters(self):
        seen = set()
        for row in self.report["alternates"]:
            with self.subTest(mpn=row["mpn"]):
                decoded = gen_passive.uniroyal_decode(row["mpn"])
                literal = gen_passive.ohm_literal(decoded["mantissa"], decoded["decade"])
                size = row["size"]
                # The part's own declaration states the decoded resistance, tolerance and case size.
                self.assertIn(f"pub part {row['part']}: ChipResistor<{literal}, {row['tolerance']}>[R{size}] {{",
                              self.text[size])
                self.assertEqual(Decimal(row["ohms"]), decoded["ohms"])
                self.assertIn(row["category"], ("basic", "preferred"))
                self.assertNotIn(row["part"], seen)
                seen.add(row["part"])

    def test_zero_ohm_rows_name_the_librarys_jumper(self):
        for row in self.report["zero_ohm"]:
            with self.subTest(mpn=row["mpn"]):
                self.assertEqual(row["part"], f"R_0R_J_{row['size']}")
                self.assertIn(f"pub part {row['part']}: ChipResistor<0ohm, 5%>[R{row['size']}] {{", self.text[row["size"]])

    def test_jumper_ratings_are_the_datasheets_table(self):
        # p.5 §7. The library's jumper part states 0 ohm and 5%; the datasheet states neither a tolerance
        # nor a part number for a jumper, so the zero_ohm rows stay apart for a person to decide.
        ratings = self.report["jumper_ratings"]
        self.assertEqual(set(ratings), set(gen_passive.R_POWER))
        self.assertEqual(ratings["0402"], {"resistance": "<50mΩ", "rated_current": "1A", "max_overload_current": "2A"})
        self.assertEqual({size: r["rated_current"] for size, r in ratings.items()},
                         {"0201": "0.5A", "0402": "1A", "0603": "1A", "0805": "2A", "1206": "2A",
                          "1210": "2A", "2010": "2A", "2512": "2A"})
        self.assertEqual({r["resistance"] for r in ratings.values()}, {"<50mΩ"})
        self.assertEqual({row["size"] for row in self.report["zero_ohm"]}, {"0402", "0603", "0805", "1206"})

    def test_new_values_have_no_library_part(self):
        self.assertEqual([(row["mpn"], row["ohms"]) for row in self.report["new_values"]], [("1206W4F100LT5E", "0.1")])
        self.assertNotIn("R_0R1_F_1206", self.text["1206"])

    def test_refused_rows_are_the_arrays_and_the_suffixed_number(self):
        self.assertEqual(
            sorted(row["mpn"] for row in self.report["not_decoded"]),
            ["0603WAF1602T5E (SMTonly)", "4D02WGJ0103TCE", "4D03WGJ0102T5E", "4D03WGJ0103T5E",
             "4D03WGJ0471T5E", "4D03WGJ0472T5E"],
        )

    def test_the_library_and_the_sourcing_tables_are_untouched(self):
        # A dry run: no UNI-ROYAL part number is emitted, mapped or aliased.
        every = "".join(path.read_text(encoding="utf-8") for path in sorted(SOURCES.glob("*.cohdl")))
        lcsc_map = (ur.ROOT / "tools/sourcing_data/lcsc_map.json").read_text(encoding="utf-8")
        aliases = json.loads((ur.ROOT / "tools/sourcing_data/mfr_aliases.json").read_text(encoding="utf-8"))["aliases"]
        self.assertNotIn(ur.CATALOG_MFR, aliases)
        for name in ("alternates", "zero_ohm", "new_values"):
            for row in self.report[name]:
                self.assertNotIn(f'"{row["mpn"]}"', every)
                self.assertNotIn(f'"{row["mpn"]}"', lcsc_map)
        self.assertNotIn("UNI-ROYAL", every)
        self.assertNotIn("uniroyal", Path(gen_passive.__file__).read_text(encoding="utf-8").split("def main()")[1])

    def test_the_openwalkietalkie_rows(self):
        rows = {row["mpn"]: row for row in self.report["openwalkietalkie_rev_b"]}
        self.assertEqual(set(rows), set(OPENWALKIETALKIE))
        for mpn, (ohms, part) in OPENWALKIETALKIE.items():
            self.assertEqual((rows[mpn]["ohms"], rows[mpn]["part"]), (ohms, part))
        self.assertEqual([mpn for mpn, row in rows.items() if not row["in_snapshot"]], ["0402WGF1803TCE"])
        self.assertEqual([mpn for mpn, row in rows.items() if row["list"] != "alternates"], ["0402WGF0000TCE"])


def page(c, mpn, description):
    """A synthetic page in the shape parse reads: one schema.org Product block."""
    product = {"@context": "https://schema.org", "@type": "Product", "sku": c, "mpn": mpn, "description": description}
    crumbs = {"@context": "https://schema.org", "@type": "BreadcrumbList", "itemListElement": []}
    return "".join(f'<script type="application/ld+json">{json.dumps(block, ensure_ascii=False)}</script>'
                   for block in (crumbs, product))


class ValidatePagesTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.cache = Path(self.tmp.name)

    def store(self, row, html, url=None):
        url = url or ur.ORIGIN + row["partdetail"]
        digest = hashlib.sha256((ur.ORIGIN + row["partdetail"]).encode()).hexdigest()
        body = html.encode("utf-8")
        (self.cache / f"{digest}.body").write_bytes(body)
        (self.cache / f"{digest}.json").write_text(json.dumps(
            {"url": url, "sha256": hashlib.sha256(body).hexdigest(), "snapshot_date": "2026-01-01T00:00:00Z"}))

    @staticmethod
    def row(c, mpn, package="0402"):
        return {"c": c, "mpn": mpn, "package": package, "partdetail": f"/partdetail/1-{mpn}/{c}/{c}"}

    def test_attributes_are_read_from_the_parts_own_description(self):
        html = page("C1", "0402WGF1002TCE", "0402WGF1002TCE - -55℃~+155℃ 10kΩ 50V 62.5mW Thick Film Resistor ±1% ±100ppm/℃ 0402")
        self.assertEqual(ur.page_attributes(html, "C1", "0402WGF1002TCE"),
                         {"ohms": Decimal(10000), "tolerance": Decimal(1), "milliwatts": Decimal("62.5")})
        self.assertIsNone(ur.page_attributes(html, "C2", "0402WGF1002TCE"))
        self.assertIsNone(ur.page_attributes(html, "C1", "0402WGF1003TCE"))
        self.assertIsNone(ur.page_attributes("<html></html>", "C1", "0402WGF1002TCE"))
        for text, ohms in (("100mΩ 250mW ±1%", "0.1"), ("49.9Ω 100mW ±1%", "49.9"), ("1.5MΩ 0.25W ±1%", "1500000")):
            self.assertEqual(ur.page_attributes(page("C1", "X", text), "C1", "X")["ohms"], Decimal(ohms))
        self.assertEqual(ur.page_attributes(page("C1", "X", "1kΩ 0.25W ±1%"), "C1", "X")["milliwatts"], Decimal(250))
        # A field stated twice, or not at all, is not read.
        self.assertIsNone(ur.page_attributes(page("C1", "X", "1kΩ 2kΩ 100mW ±1%"), "C1", "X")["ohms"])
        self.assertIsNone(ur.page_attributes(page("C1", "X", "1kΩ ±1%"), "C1", "X")["milliwatts"])

    def test_each_outcome_is_counted(self):
        good = self.row("C1", "0402WGF1002TCE")
        wrong_value = self.row("C2", "0402WGF1003TCE")
        wrong_tolerance = self.row("C3", "0402WGF2202TCE")
        wrong_power = self.row("C4", "0402WGF220JTCE")
        wrong_package = self.row("C5", "0402WGF1001TCE", package="0603")
        missing = self.row("C6", "0402WGF1004TCE")
        tampered = self.row("C7", "0402WGF5101TCE")
        unreadable = self.row("C8", "0402WGF3301TCE")
        refused = self.row("C9", "4D03WGJ0472T5E", package="0603x4")
        self.store(good, page("C1", good["mpn"], "10kΩ 50V 62.5mW ±1% ±100ppm/℃ 0402"))
        self.store(wrong_value, page("C2", wrong_value["mpn"], "10kΩ 62.5mW ±1%"))
        self.store(wrong_tolerance, page("C3", wrong_tolerance["mpn"], "22kΩ 62.5mW ±5%"))
        self.store(wrong_power, page("C4", wrong_power["mpn"], "22Ω 100mW ±1%"))
        self.store(wrong_package, page("C5", wrong_package["mpn"], "1kΩ 62.5mW ±1%"))
        self.store(tampered, page("C7", tampered["mpn"], "5.1kΩ 62.5mW ±1%"), url="https://example.invalid/other")
        self.store(unreadable, page("C8", unreadable["mpn"], "62.5mW ±1%"))
        rows = [good, wrong_value, wrong_tolerance, wrong_power, wrong_package, missing, tampered, unreadable, refused]
        result = ur.validate_pages(self.cache, rows)
        self.assertEqual((result["rows"], result["match"]), (9, 1))
        self.assertEqual({item["pn"]: item["differs"] for item in result["mismatch"]},
                         {"C2": ["resistance"], "C3": ["tolerance"], "C4": ["power"], "C5": ["package"]})
        self.assertEqual(result["no_page"], ["C6", "C7"])
        self.assertEqual(result["unreadable"], ["C8"])
        self.assertEqual([item["pn"] for item in result["not_decoded"]], ["C9"])
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            ur.print_validation(result)
        self.assertIn("1 of 8", out.getvalue())
        self.assertEqual(sorted(p.name for p in self.cache.iterdir() if not p.name.endswith((".body", ".json"))), [])

    def test_a_mismatch_fails_the_command(self):
        with mock.patch.object(ur, "validate_pages", return_value={
            "rows": 1, "not_decoded": [], "no_page": [], "unreadable": [], "match": 0,
            "mismatch": [{"pn": "C2", "mpn": "0402WGF1003TCE", "differs": ["resistance"]}],
        }), contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(ur.main(["--validate-pages", str(self.cache)]), 1)

    def test_a_missing_page_fails_the_command(self):
        # An empty cache: no decoded row can be compared, so the check has not passed.
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ur.main(["--validate-pages", str(self.cache)]), 1)
        self.assertIn("0 of 289", out.getvalue())
        self.assertIn("289 have no cached page", out.getvalue())


if __name__ == "__main__":
    unittest.main()
