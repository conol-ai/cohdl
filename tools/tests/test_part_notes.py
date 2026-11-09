import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import part_notes as pn

SHA = "ef8d376f2ec356e29172eb9e053819a0ebdcc576dba7fc9ab0505c568427920f"
IDENTITY = {
    "schema": "part-note/1", "device": "ldo::AP2112K_3V3", "parts": ["ldo::LDO_AP2112K_3V3"],
    "identities": [{"mfr": "Diodes Incorporated", "mpn": "AP2112K-3.3TRG1", "lcsc": "C51118"}],
    "sources": [{"id": "ds", "title": "AP2112", "revision": "DS39724 Rev. 2-2",
                 "url": "https://www.diodes.com/datasheet/download/AP2112.pdf", "sha256": SHA,
                 "local": "lib/ldo/docs/diodes-ap2112.pdf"}],
    "status": "draft",
}
PINS = [("1", "VIN"), ("2", "GND"), ("3", "EN"), ("4", "NC"), ("5", "VOUT")]


def note(identity=None, pins=None, cite="ds p. 1"):
    identity = IDENTITY if identity is None else identity
    pins = PINS if pins is None else pins
    rows = "\n".join(f"| {pad} | {name} | {name} | in | f | {cite} |" for pad, name in pins)
    return (f"# AP2112K-3.3 — LDO\n\n## Identity\n\n```json\n{json.dumps(identity)}\n```\n\n"
            "## Summary\n\nx\n\n"
            f"## Key parameters\n\n| Parameter | Min | Typ | Max | Unit | Conditions | Source |\n|---|---|---|---|---|---|---|\n"
            f"| VOUT | | 3.3 | | V | | {cite} |\n\n"
            f"## Absolute maximum ratings\n\n| Parameter | Limit | Unit | Source |\n|---|---|---|---|\n| VIN | 6.5 | V | {cite} |\n\n"
            f"## Recommended operating conditions\n\n| Parameter | Min | Max | Unit | Source |\n|---|---|---|---|---|\n"
            f"| VIN | 2.5 | 6 | V | {cite} |\n\n"
            f"## Pins\n\n| Pad | Library pin | Datasheet name | Direction | Function | Source |\n|---|---|---|---|---|---|\n{rows}\n")


class PartNotesTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.dirs = pn.package_dirs()
        rows = json.loads((pn.ROOT / "tools/sourcing_data/lcsc_map.json").read_text(encoding="utf-8"))["rows"]
        cls.lcsc = {(r["mfr"], r["mpn"]): r["pn"] for r in rows}

    def check(self, text):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "note.md"
            path.write_text(text, encoding="utf-8")
            return pn.check(path, self.dirs, self.lcsc)

    def test_a_complete_note_passes(self):
        self.assertEqual(self.check(note()), [])

    def test_the_pin_table_must_match_the_device(self):
        swapped = [("1", "VOUT"), ("2", "GND"), ("3", "EN"), ("4", "NC"), ("5", "VIN")]
        self.assertEqual(self.check(note(pins=swapped)),
                         ["pad 1 is VIN in the library, VOUT in the note", "pad 5 is VOUT in the library, VIN in the note"])
        self.assertIn("pad 4 (NC) is missing from the pin table", self.check(note(pins=PINS[:3] + PINS[4:])))
        self.assertIn("pad 6 is not a pad of AP2112K_3V3", self.check(note(pins=PINS + [("6", "X")])))

    def test_every_table_row_cites_a_page_of_a_listed_source(self):
        problems = self.check(note(cite="datasheet"))
        self.assertEqual(len(problems), 3 + 5)
        self.assertTrue(all("cites no page" in p for p in problems))
        self.assertIn("Pins: cites an unknown source in 'other p. 3'", self.check(note(cite="other p. 3")))

    def test_identity_and_sources_are_checked(self):
        wrong = dict(IDENTITY, identities=[{"mfr": "Diodes Incorporated", "mpn": "AP2112K-3.3TRG1", "lcsc": "C1"}])
        self.assertEqual(self.check(note(wrong)), ["AP2112K-3.3TRG1: lcsc C1, but lcsc_map.json maps C51118"])
        bad_sha = dict(IDENTITY, sources=[dict(IDENTITY["sources"][0], sha256="0" * 64)])
        self.assertIn(f"source ds: lib/ldo/docs/diodes-ap2112.pdf does not hash to {'0' * 64}", self.check(note(bad_sha)))
        other = dict(IDENTITY, parts=["ldo::NO_SUCH_PART"])
        self.assertIn("no part ldo::NO_SUCH_PART", self.check(note(other)))
        placeholder = dict(IDENTITY, status="<draft>")
        self.assertIn("a <placeholder> is left in Identity", self.check(note(placeholder)))
        self.assertIn("a template comment is left", self.check(note() + "\n<!-- todo -->\n"))

    def test_a_series_note_matches_its_pattern_over_each_variant(self):
        series = {"schema": "part-note/1", "device": "passive::ChipResistor", "parts_pattern": "R_.*_0402",
                  "variants": ["R0402", "R0603"],
                  "identities": [{"mfr": "Yageo", "mpn_pattern": r"RC0402[FJ]R-07.+L"}],
                  "sources": IDENTITY["sources"], "status": "draft"}
        self.assertEqual(self.check(note(series, pins=[("1", "A"), ("2", "B")])), [])
        series["identities"] = [{"mfr": "Yageo", "mpn_pattern": "XX.*"}]
        self.assertEqual(self.check(note(series, pins=[("1", "A"), ("2", "B")])), ["no Yageo entry matches XX.*"])

    def test_contact_named_pads_are_compared(self):
        usb = {"schema": "part-note/1", "device": "usb::USB_C_Receptacle_2_0",
               "parts": ["usb::USB_C_HRO_TYPE_C_31_M_12"],
               "identities": [{"mfr": "HRO", "mpn": "TYPE-C-31-M-12", "lcsc": "C165948"}],
               "sources": IDENTITY["sources"], "status": "draft"}
        pins = [(p, "GND") for p in ("A1", "A12", "B1", "B12")] + [(p, "VBUS") for p in ("A4", "A9", "B4", "B9")] + [
            ("A5", "CC1"), ("B5", "CC2"), ("A6", "DP"), ("B6", "DP"), ("A7", "DN"), ("B7", "DN"),
            ("A8", "SBU1"), ("B8", "SBU2")] + [(f"SH{i}", "SHIELD") for i in range(1, 5)]
        self.assertEqual(self.check(note(usb, pins=pins)), [])
        swapped = [(p, {"CC1": "CC2", "CC2": "CC1"}.get(n, n)) for p, n in pins]
        self.assertEqual(self.check(note(usb, pins=swapped)),
                         ["pad A5 is CC1 in the library, CC2 in the note", "pad B5 is CC2 in the library, CC1 in the note"])

    def test_mechanical_pads_are_read(self):
        self.assertEqual(pn.PAD.findall("1, 2, MP, EP, SH4"), ["1", "2", "MP", "EP", "SH4"])

    def test_a_part_may_name_its_device_by_path(self):
        _, parts = pn.declarations(self.dirs["@espressif/esp32"])
        self.assertEqual(parts["ESP32_S3_WROOM_1U_N4"][0], "ESP32_S3_WROOM_1")
        self.assertEqual(parts["ESP32_S3_WROOM_1_N8"][0], "ESP32_S3_WROOM_1")

    def test_the_committed_notes_pass(self):
        with mock.patch.object(sys, "stderr", io.StringIO()) as stderr, mock.patch.object(sys, "stdout", io.StringIO()):
            self.assertEqual(pn.main(["check"]), 0, stderr.getvalue())


if __name__ == "__main__":
    unittest.main()
