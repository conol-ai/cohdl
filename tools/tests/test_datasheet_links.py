import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import datasheet_links as dl

SNAPSHOT = dl.DATA / "datasheet_links/2026-11-07"

# A package README shared by several vendors' parts, like lib/connectors/docs/README.md.
SHARED_README = [
    "| `raspberry-pi-pico-datasheet.pdf` | Pico | [Raspberry Pi](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) |",
    "| `samtec-ssw.pdf` | Samtec `SSW-103-22-SM-D-VS` | [Samtec](https://suddendocs.samtec.com/prints/ssw-dvs.pdf) |",
    "| `molex-43045.pdf` | Molex `43045-0212` | [Molex](https://www.molex.com/salesdrawingpdf/430450612_sd.pdf) |",
    "| External specification | Micro-Fit 3.0 | [Molex PDF](https://www.molex.com/productspecificationpdf/PS-43045-001.pdf) |",
    "| `hirose-fh12.evidence.md` | Hirose `FH12-26S-0.5SH(55)` | [Hirose](https://www.hirose.com/product/p/CL0586-0576-2-55) |",
    "| `hirose-ufl.evidence.md` | Hirose `U.FL-R-SMT-1(10)` | [mirror](https://mm.digikey.com/docus/8942/HIROS08829-1.pdf) |",
    "- Pico design files: [ZIP](https://pip.raspberrypi.com/documents/RP-008379-DS).",
]


def shared(*extra):
    return {("connectors", "docs/README.md"): {"local": True, "sha256": None, "readme_lines": [],
                                               "record_lines": SHARED_README + list(extra)}}


def identity(mfr, mpn):
    return {"package": "connectors", "part": "P", "entry": "primary", "mfr": mfr, "mpn": mpn,
            "docs": ["docs/README.md"]}


class DatasheetLinksTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = dl.build()
        cls.rows = {(r[0], r[1], r[2], r[4]): r for r in cls.report["identities"]}

    def links(self, row):
        return [self.report["links"][i][0] for i, *_ in row[6]]

    def test_the_committed_index_records_its_sources(self):
        committed = json.loads((SNAPSHOT / "datasheet_links.json").read_text(encoding="utf-8"))
        self.assertEqual(committed["schema_version"], 2)
        self.assertIn("lcsc_map.json", committed["sources"])
        self.assertEqual(committed["identity_columns"], self.report["identity_columns"])
        self.assertEqual((SNAPSHOT / "datasheet_links.md").read_text(encoding="utf-8"), dl.render(committed))

    def test_every_library_identity_is_indexed_once(self):
        keys = [(r[0], r[1], r[2], r[3], r[4]) for r in self.report["identities"]]
        self.assertEqual(len(keys), len(set(keys)))
        counted = sum(sum(c.values()) for c in self.report["counts"].values())
        self.assertEqual(counted, len(keys))

    def test_an_alternate_does_not_inherit_the_primarys_datasheet(self):
        alt = self.rows[("passive", "R_1R_F_0201", "alt", "CRCW02011R00FKED")]
        self.assertTrue(self.links(alt))
        self.assertTrue(all("vishay.com" in url for url in self.links(alt)))
        primary = self.rows[("passive", "R_1R_F_0201", "primary", "RC0201FR-071RL")]
        self.assertTrue(all("yageo" in url for url in self.links(primary)))
        # Samsung alternates cite only the Yageo datasheets, so they get no official link.
        samsung = self.rows[("passive", "C_100p_100V_C0G_0201", "alt", "CL03C101JC31IN")]
        self.assertEqual(samsung[5], "local_only")

    def test_a_reel_suffix_still_finds_the_part_line(self):
        row = self.rows[("@st/stm32", "MCU_STM32C011D6Y6TR", "primary", "STM32C011D6Y6TR")]
        self.assertEqual(row[5], "official_mpn")
        self.assertIn("https://www.st.com/resource/en/datasheet/stm32c011d6.pdf", self.links(row))

    def test_only_the_manufacturers_own_domain_is_official(self):
        for row in self.report["identities"]:
            for i, match, label, _ in row[6]:
                url = self.report["links"][i][0]
                self.assertIn(match, ("mpn", "manufacturer"), (row[:5], url))
                self.assertEqual(label, dl.label(url, row[3]), (row[:5], url))
                if label == "official":
                    self.assertTrue(dl._on(dl._host(url), dl.MANUFACTURER_DOMAINS[row[3]]), (row[:5], url))
                if match == "manufacturer":
                    self.assertEqual(label, "official", (row[:5], url))
        row = self.rows[("esd", "ESD_H5VL10B", "primary", "H5VL10B")]
        self.assertNotIn("official", row[5])
        self.assertEqual(row[8], "C7420372")
        self.assertIn("C7420372", self.report["lcsc_datasheets"])

    def test_a_shared_readme_gives_a_part_only_its_own_vendors_links(self):
        # 76a5987 attached every URL of a small record to every part citing it.
        jst = identity("JST", "S2B-XH-A(LF)(SN)")
        self.assertEqual(dl.links_for(jst, shared()), [])
        self.assertEqual(dl.classify(dl.links_for(jst, shared()), None, False), "none")
        line = "| `jst-xh.pdf` | JST XH series | <https://www.jst-mfg.com/product/pdf/eng/eXH.pdf> |"
        links = dl.links_for(jst, shared(line))
        self.assertEqual([(l["url"], l["match"], l["label"]) for l in links],
                         [("https://www.jst-mfg.com/product/pdf/eng/eXH.pdf", "manufacturer", "official")])
        molex = dl.links_for(identity("Molex", "43045-0212"), shared(line))
        self.assertEqual(sorted((l["url"].rsplit("/", 1)[1], l["match"]) for l in molex),
                         [("430450612_sd.pdf", "mpn"), ("PS-43045-001.pdf", "manufacturer")])
        # In the repository: the JST battery connector cites only the shared connectors README.
        row = self.rows[("connectors", "CONN_BAT_2P", "primary", "S2B-XH-A(LF)(SN)")]
        self.assertFalse(row[5].startswith("official"), row[5])
        self.assertTrue(all("jst-mfg.com" in url for url in self.links(row)), self.links(row))

    def test_an_empty_mpn_never_matches(self):
        self.assertEqual(dl.mpn_forms(""), [])
        self.assertEqual(dl.mpn_forms("  "), [])
        self.assertEqual(dl.links_for(identity("Generic", ""), shared()), [])
        for part in ("CONN_MIC_2P", "CONN_SPK_2P"):
            row = self.rows[("connectors", part, "primary", "")]
            self.assertEqual(row[6], [])
            self.assertEqual(row[5], "none")
        for row in self.report["identities"]:
            if not row[4].strip():
                self.assertNotIn("mpn", [match for _, match, *_ in row[6]], row[:5])

    def test_a_mirror_naming_the_mpn_is_not_official(self):
        # In the repository: the U.FL receptacle's only line naming it links a DigiKey mirror.
        row = self.rows[("connectors", "CON_HIROSE_UFL_R_SMT_1_10", "primary", "U.FL-R-SMT-1(10)")]
        self.assertEqual(row[5], "unverified")
        ufl, fh12 = identity("Hirose Electric", "U.FL-R-SMT-1(10)"), identity("Hirose Electric", "FH12-26S-0.5SH(55)")
        # The Hirose FH12 row is on hirose.com but is about the FH12 part, not U.FL.
        links = dl.links_for(ufl, shared(), dl.part_namer([ufl, fh12]))
        self.assertEqual([(l["url"], l["match"], l["label"]) for l in links],
                         [("https://mm.digikey.com/docus/8942/HIROS08829-1.pdf", "mpn", "mirror")])
        self.assertEqual(dl.classify(links, None, False), "unverified")
        for url in ("https://datasheet.lcsc.com/x.pdf", "https://www.alldatasheet.com/hirose.pdf",
                    "https://web.archive.org/web/1/https://www.hirose.com/a.pdf",
                    "https://raw.githubusercontent.com/hirose/a.pdf"):
            self.assertEqual(dl.label(url, "Hirose Electric"), "mirror", url)
        self.assertEqual(dl.label("https://www.hirose.com/a.pdf", "Hirose Electric"), "official")
        self.assertEqual(dl.label("https://www.hirose.com/a.pdf", "JST"), "unverified")
        self.assertEqual(dl.label("https://example-hirose.com/a.pdf", "Hirose Electric"), "unverified")

    def test_a_catalog_row_belongs_to_its_own_part(self):
        # In the repository: the ESP32-C3 does not take the ESP32-S3's datasheet from the shared catalog.
        row = self.rows[("@espressif/esp32", "ESP32_C3", "primary", "ESP32-C3")]
        self.assertNotIn("https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf",
                         self.links(row))
        record = ["| `ESP32-C3` | <https://www.espressif.com/esp32-c3_datasheet_en.pdf> |",
                  "| `ESP32-S3` | <https://www.espressif.com/esp32-s3_datasheet_en.pdf> |",
                  "- Selector: <https://products.espressif.com/api>"]
        docs = {("connectors", "docs/README.md"): {"local": True, "sha256": None, "readme_lines": [],
                                                   "record_lines": record}}
        c3, s3 = identity("Espressif", "ESP32-C3"), identity("Espressif", "ESP32-S3")
        links = dl.links_for(c3, docs, dl.part_namer([c3, s3]))
        self.assertEqual(sorted((l["url"], l["match"]) for l in links),
                         [("https://products.espressif.com/api", "manufacturer"),
                          ("https://www.espressif.com/esp32-c3_datasheet_en.pdf", "mpn")])

    def test_snapshot_parts_carry_their_lcsc_datasheets(self):
        snapshot = self.report["snapshot"]
        self.assertEqual(len(snapshot), 1334)
        self.assertEqual(sum(1 for r in snapshot if r[0] in self.report["lcsc_datasheets"]), 1332)


if __name__ == "__main__":
    unittest.main()
