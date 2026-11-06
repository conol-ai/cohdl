import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import datasheet_links as dl

SNAPSHOT = dl.DATA / "datasheet_links/2026-11-07"


class DatasheetLinksTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = dl.build()
        cls.rows = {(r[0], r[1], r[2], r[4]): r for r in cls.report["identities"]}

    def links(self, row):
        return [self.report["links"][i][0] for i, _ in row[6]]

    def test_the_committed_index_records_its_sources(self):
        committed = json.loads((SNAPSHOT / "datasheet_links.json").read_text(encoding="utf-8"))
        self.assertEqual(committed["schema_version"], 1)
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

    def test_distributor_links_are_not_official(self):
        for url, _ in self.report["links"]:
            self.assertTrue(dl._official(url), url)
        row = self.rows[("esd", "ESD_H5VL10B", "primary", "H5VL10B")]
        self.assertNotIn("official", row[5])
        self.assertEqual(row[8], "C7420372")
        self.assertIn("C7420372", self.report["lcsc_datasheets"])

    def test_snapshot_parts_carry_their_lcsc_datasheets(self):
        snapshot = self.report["snapshot"]
        self.assertEqual(len(snapshot), 1334)
        self.assertEqual(sum(1 for r in snapshot if r[0] in self.report["lcsc_datasheets"]), 1332)


if __name__ == "__main__":
    unittest.main()
