import json
import math
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import w3_groups as wg

REPORT = wg.DATA / "w3_sampling/2026-11-07"


class W3GroupsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = wg.build()

    def test_the_committed_report_is_consistent_with_its_recorded_sources(self):
        # Historical record: a dated snapshot, checked only against what it records.
        # Later library, map or frame changes make it older, not wrong, so nothing
        # here compares it with a fresh build.
        committed = json.loads((REPORT / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((REPORT / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        sources = committed["sources"]
        self.assertEqual(sources["lib/passive"], "0.2.5")
        pages_paths = [name for name in sources if name.startswith("lcsc_backfill/")]
        self.assertEqual(set(sources), {"lib/passive", "lcsc_map.json", "jlc_frames.json", *pages_paths})
        self.assertEqual(pages_paths, ["lcsc_backfill/2026-11-06-full/confirmed.json"])
        # The confirmed pages are a run's record, kept as they were: the hash must still hold.
        self.assertEqual(wg._sha256(wg.DATA / pages_paths[0]), sources[pages_paths[0]])
        pages = {p["c"]: p for p in json.loads((wg.DATA / pages_paths[0]).read_text(encoding="utf-8"))}
        rule = committed["rule"]
        for g in committed["groups"]:
            with self.subTest(group=(g["footprint"], g["type"], g["mfr"], g["series"])):
                self.assertEqual(g["package_version"], sources["lib/passive"])
                rows = {row[0]: row for row in g["member_rows"]}
                self.assertEqual(len(rows), g["members"])
                # Each member's stock is its recorded page's.
                self.assertTrue(all(row[5] == pages[pn]["stock"] for pn, row in rows.items()))
                direct = {d["pn"] for d in g["direct"]}
                self.assertEqual(g["required"], max(rule["min_sample"], math.ceil(rule["fraction"] * g["members"])))
                self.assertEqual(g["more_needed"], max(0, g["required"] - len(direct)))
                self.assertLessEqual(len(g["sample"]), g["more_needed"])
                for pn in g["sample"]:
                    self.assertGreaterEqual(rows[pn][5], rule["min_stock"])
                    self.assertNotIn(pn, direct)
                    self.assertNotIn(pn, rule["short_at_preview"])

    def test_a_fresh_report_matches_what_the_current_tree_builds(self):
        # Current generator: built from the current tree and compared with a fresh
        # build, never with the dated report.
        name, version = wg.package_metadata(wg.PASSIVE / "cohdl.toml")
        snapshot = {row["snapshot_sha256"] for row in
                    json.loads((wg.DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]}
        pages_path, _ = wg.confirmed_pages(snapshot.pop())
        self.assertEqual(self.report["sources"], {
            "lib/passive": version,
            "lcsc_map.json": wg._sha256(wg.DATA / "lcsc_map.json"),
            "jlc_frames.json": wg._sha256(wg.DATA / "jlc_frames.json"),
            str(pages_path.relative_to(wg.DATA)): wg._sha256(pages_path),
        })
        self.assertEqual(self.report["rule"]["min_stock"], wg.MIN_STOCK)
        with tempfile.TemporaryDirectory() as td:
            with mock.patch("sys.stdout"):
                self.assertEqual(wg.main(["-o", td]), 0)
            self.assertEqual(wg.main(["-o", td, "--check"]), 0)
            self.assertEqual(json.loads(Path(td, "groups.json").read_text(encoding="utf-8")), self.report)

    def test_groups_split_by_version_type_manufacturer_and_series(self):
        groups = self.report["groups"]
        keys = [(g["package"], g["package_version"], g["footprint"], g["type"], g["mfr"], g["series"]) for g in groups]
        self.assertEqual(len(keys), len(set(keys)))
        for g in groups:
            self.assertTrue(all(row[1].startswith(g["series"]) and wg.series(row[1]) == g["series"]
                                for row in g["member_rows"]), g["series"])
        # The page's package text alone never makes a group: one land, one text, several groups.
        chip_0402 = [g for g in groups if g["footprint"] == "CHIP_0402"]
        self.assertTrue(all("0402" in g["page_package_text"] for g in chip_0402))
        self.assertGreater(len(chip_0402), 1)

    def test_samples_follow_the_draft_rule(self):
        for g in self.report["groups"]:
            rows = {row[0]: row for row in g["member_rows"]}
            direct = {d["pn"] for d in g["direct"]}
            self.assertEqual(g["required"], max(3, math.ceil(0.05 * g["members"])))
            self.assertEqual(g["more_needed"], max(0, g["required"] - len(direct)))
            self.assertLessEqual(len(g["sample"]), g["more_needed"])
            eligible = [pn for pn, row in rows.items() if pn not in direct and row[5] >= wg.MIN_STOCK]
            self.assertEqual(len(g["sample"]), min(g["more_needed"], len(eligible)))
            for pn in g["sample"]:
                # Never a part without stock, never one already checked.
                self.assertGreaterEqual(rows[pn][5], wg.MIN_STOCK)
                self.assertNotIn(pn, direct)
                self.assertNotIn(pn, wg.SHORT_AT_PREVIEW)
            self.assertEqual(g["zero_stock"], sum(row[5] <= 0 for row in rows.values()))

    def test_a_direct_reading_off_zero_pauses_its_group(self):
        frames = json.loads((wg.DATA / "jlc_frames.json").read_text(encoding="utf-8"))
        target = next(row for row in frames["rows"] if row["package"] == "passive" and row["pn"] == "C1525")
        target["mapping"] = {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 90}
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == wg.DATA / "jlc_frames.json":
                return json.dumps(frames)
            return real(path, *args, **kwargs)
        with mock.patch.object(Path, "read_text", read_text):
            report = wg.build()
        paused = [g for g in report["groups"] if g["status"] == "paused"]
        self.assertEqual([(g["footprint"], g["mfr"], g["series"], g["anomalies"]) for g in paused],
                         [("CHIP_0402", "Samsung", "CL", ["C1525"])])

    def test_spread_is_deterministic_and_covers_both_ends(self):
        self.assertEqual(wg.spread(list(range(10)), 3), [0, 4, 9])
        self.assertEqual(wg.spread(list(range(3)), 5), [0, 1, 2])
        self.assertEqual(wg.spread(list(range(10)), 0), [])

    def test_a_sample_short_at_preview_is_replaced_by_its_nearest_neighbour(self):
        self.assertEqual(wg.choose(list(range(10)), 3, lambda i: i == 4), [0, 3, 9])
        self.assertEqual(wg.choose(list(range(10)), 3, lambda i: i in (3, 4)), [0, 5, 9])
        # Ends: 0 is short, 1 is free; 9 is short and 8 is taken by nothing.
        self.assertEqual(wg.choose(list(range(10)), 3, lambda i: i in (0, 9)), [1, 4, 8])
        self.assertEqual(wg.choose(list(range(3)), 3, lambda i: i == 1), [0, 2])


if __name__ == "__main__":
    unittest.main()
