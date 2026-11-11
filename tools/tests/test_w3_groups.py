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
CURRENT = wg.DATA / "w3_sampling/2026-11-10"
PILOT = wg.PREVIEWS / "2026-11-07-w3-pilot-0402"


def group(report, footprint, kind, mfr, series):
    return next(g for g in report["groups"]
                if (g["footprint"], g["type"], g["mfr"], g["series"]) == (footprint, kind, mfr, series))


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
        inputs = {str((record / name).relative_to(wg.DATA)): wg._sha256(record / name)
                  for record, meta in wg.preview_inputs().items()
                  for name in ("w3.json", meta["bom"], meta["checks"], meta["measurements"])}
        self.assertEqual(self.report["sources"], {
            "lib/passive": version,
            "lcsc_map.json": wg._sha256(wg.DATA / "lcsc_map.json"),
            "jlc_frames.json": wg._sha256(wg.DATA / "jlc_frames.json"),
            str(pages_path.relative_to(wg.DATA)): wg._sha256(pages_path),
            **inputs,
        })
        self.assertEqual({k: v for k, v in self.report["rule"].items() if k != "short_at_preview"},
                         wg.w3_rule.RULE)
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

    def test_the_current_dated_report_is_consistent_with_its_recorded_sources(self):
        # The 2026-11-10 report under rule w3-2026-11-10b: like the 2026-11-07 one, a
        # dated snapshot, checked against what it records rather than a fresh build.
        committed = json.loads((CURRENT / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((CURRENT / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        self.assertEqual(committed["schema_version"], 3)
        self.assertEqual(committed["rule"]["version"], "w3-2026-11-10b")
        for name, digest in committed["sources"].items():
            if name.startswith("jlc_previews/") or name.startswith("lcsc_backfill/"):
                self.assertEqual(wg._sha256(wg.DATA / name), digest, name)

    def test_samples_follow_the_rule(self):
        for g in self.report["groups"]:
            rows = {row[0]: row for row in g["member_rows"]}
            direct = {d["pn"] for d in g["direct"]}
            passed = {p["pn"] for p in g["passed"]}
            failed = {f["pn"] for f in g["failed"]}
            not_drawn = {n["pn"] for n in g["not_drawn"]}
            self.assertFalse(passed & failed or passed & not_drawn or failed & not_drawn)
            # Not drawn: every copy either passed or not drawn, and at least one not drawn.
            for n in g["not_drawn"]:
                self.assertTrue(any(c["not_drawn"] for c in n["outcomes"]))
                self.assertTrue(all(c["ok"] or c["not_drawn"] for c in n["outcomes"]))
            self.assertEqual(g["required"], max(3, math.ceil(0.05 * g["members"])))
            self.assertEqual(g["more_needed"], max(0, g["required"] - len(passed)))
            self.assertLessEqual(len(g["sample"]), g["more_needed"])
            self.assertEqual(g["status"] == "paused", bool(g["failed"] or g["anomalies"]))
            for p in g["passed"]:
                self.assertTrue(wg.w3_rule.sample_ok(p))
            tried = direct | passed | failed | not_drawn
            eligible = [pn for pn, row in rows.items() if pn not in tried and row[5] >= wg.MIN_STOCK]
            self.assertEqual(len(g["sample"]), min(g["more_needed"], len(eligible)))
            for pn in g["sample"]:
                # Never a part without stock, never one already checked.
                self.assertGreaterEqual(rows[pn][5], wg.MIN_STOCK)
                self.assertNotIn(pn, tried)
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
        self.assertEqual(group(self.report, "CHIP_0402", "MLCC", "Samsung", "CL")["status"], "needs_samples")
        with mock.patch.object(Path, "read_text", read_text):
            report = wg.build()
        changed = [(g["footprint"], g["mfr"], g["series"], g["anomalies"]) for g in report["groups"]
                   if g["status"] != group(self.report, g["footprint"], g["type"], g["mfr"], g["series"])["status"]]
        self.assertEqual(changed, [("CHIP_0402", "Samsung", "CL", ["C1525"])])

    def test_the_pilot_samples_the_0402_yageo_rc_group_and_not_the_others(self):
        rc = group(self.report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("sampled", 49, 49))
        # Every RC sample is within the raw limit too, not only after the bias is taken off.
        self.assertTrue(all(p["raw_offset_mm"] <= p["offset_limit_mm"] for p in rc["passed"]))
        # Not drawn no longer pauses CRCW: its 9 undrawn samples do not count, so it needs 9 more.
        crcw = group(self.report, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["failed"]), ("needs_samples", 40, []))
        self.assertEqual(len(crcw["not_drawn"]), 9)
        self.assertTrue(all(c["status"] == "unverified" for n in crcw["not_drawn"] for c in n["outcomes"]))
        self.assertEqual(crcw["more_needed"], 9)
        # CC stays paused: C113781 is left for manual review (10.5 deg), whatever C576731 is.
        cc = group(self.report, "CHIP_0402", "MLCC", "Yageo", "CC")
        self.assertEqual(cc["status"], "paused")
        self.assertEqual([f["pn"] for f in cc["failed"]], ["C113781"])
        self.assertEqual([n["pn"] for n in cc["not_drawn"]], ["C576731"])

    def test_a_group_whose_only_fault_is_an_undrawn_sample_is_not_paused(self):
        # Board 3: C874860 is not drawn at any of its four placements; C309589 passes.
        r1210 = group(self.report, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], [n["pn"] for n in r1210["not_drawn"]]), ("needs_samples", ["C874860"]))
        self.assertNotIn("C874860", r1210["sample"])
        # 2512 also has C878096 off its residual limit, so it stays paused.
        r2512 = group(self.report, "CHIP_2512", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r2512["status"], [f["pn"] for f in r2512["failed"]]), ("paused", ["C878096"]))

    def pilot_with_uniform_offset(self, offset_y):
        """The pilot's readings with every drawn body moved to (0, OFFSET_Y): the
        reviewer's counterexample, where the board median takes the whole offset off."""
        meta = json.loads((PILOT / "w3.json").read_text(encoding="utf-8"))
        readings = json.loads((PILOT / meta["measurements"]).read_text(encoding="utf-8"))
        for m in readings:
            if "offset_xy_mm" in m.get("body", {}):
                m["body"]["offset_xy_mm"] = [0.0, offset_y]
                m["body"]["offset_limit_mm"] = 0.127
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == PILOT / meta["measurements"]:
                return json.dumps(readings)
            return real(path, *args, **kwargs)
        with mock.patch.object(Path, "read_text", read_text):
            return wg.build()

    def test_a_uniform_offset_within_the_raw_limit_passes(self):
        rc = group(self.pilot_with_uniform_offset(-0.12), "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual(rc["status"], "sampled")
        pilot = [p for p in rc["passed"] if p["records"] == [str((PILOT / "README.md").relative_to(wg.ROOT))]]
        self.assertEqual(len(pilot), 44)
        self.assertTrue(all((p["raw_offset_mm"], p["residual_offset_mm"]) == (0.12, 0.0) for p in pilot))

    def test_a_uniform_offset_beyond_the_raw_limit_pauses_the_group(self):
        # 0.13 mm against 0.127: the residual is 0 for every sample, so only the raw check fails them.
        rc = group(self.pilot_with_uniform_offset(-0.13), "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual(rc["status"], "paused")
        self.assertEqual(len(rc["failed"]), 44)
        for f in rc["failed"]:
            (c,) = f["outcomes"]
            self.assertEqual((c["reading"]["raw_offset_mm"], c["reading"]["residual_offset_mm"]), (0.13, 0.0))

    def test_readings_lapse_when_a_land_file_changes(self):
        real = wg._sha256

        def sha256(path):
            return "0" * 64 if path == wg.ROOT / "lib/passive/src/footprints.cohdl" else real(path)
        with mock.patch.object(wg, "_sha256", sha256):
            self.assertEqual(wg.judged_members(), {})

    def test_a_reading_outside_the_sample_limits_fails_its_member(self):
        meta = json.loads((PILOT / "w3.json").read_text(encoding="utf-8"))
        readings = json.loads((PILOT / meta["measurements"]).read_text(encoding="utf-8"))
        target = next(m for m in readings if m.get("body", {}).get("delta_mod_180") == 9.9)
        target["body"]["delta_mod_180"] = 10.5
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == PILOT / meta["measurements"]:
                return json.dumps(readings)
            return real(path, *args, **kwargs)
        with mock.patch.object(Path, "read_text", read_text):
            report = wg.build()
        rc = group(report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual(rc["status"], "paused")
        self.assertEqual([f["pn"] for f in rc["failed"]], ["C138033"])

    def test_short_at_preview_lives_in_the_dated_record(self):
        self.assertEqual(set(wg.SHORT_AT_PREVIEW), {"C185414", "C185430"})
        self.assertNotIn("C185414", (wg.ROOT / "tools/w3_groups.py").read_text(encoding="utf-8"))

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


class W3RuleTests(unittest.TestCase):
    """The reviewer's counterexample on the rule itself: 49 samples all off by the
    same amount leave no residual, so only the raw check can tell centred from not."""

    def readings(self, offset):
        offsets = [(0.0, offset)] * 49
        bias = wg.w3_rule.board_bias(offsets)
        return [wg.w3_rule.sample_reading(0.0, o, 0.127, bias) for o in offsets]

    def test_a_uniform_offset_within_the_raw_limit_passes(self):
        readings = self.readings(0.12)
        self.assertEqual({(r["raw_offset_mm"], r["residual_offset_mm"]) for r in readings}, {(0.12, 0.0)})
        self.assertTrue(all(wg.w3_rule.sample_ok(r) for r in readings))

    def test_a_uniform_offset_beyond_the_raw_limit_fails_on_the_raw_check(self):
        readings = self.readings(0.13)
        self.assertEqual({(r["raw_offset_mm"], r["residual_offset_mm"]) for r in readings}, {(0.13, 0.0)})
        self.assertFalse(any(wg.w3_rule.sample_ok(r) for r in readings))
        # The residual alone would have passed them: that is the gap the raw check closes.
        rule = dict(wg.w3_rule.RULE, offset_raw_fraction=float("inf"))
        with mock.patch.object(wg.w3_rule, "RULE", rule):
            self.assertTrue(all(wg.w3_rule.sample_ok(r) for r in readings))

    def test_a_reading_whose_offsets_do_not_add_up_fails(self):
        reading = wg.w3_rule.sample_reading(0.0, (0.0, 0.06), 0.127, (0.0, 0.0))
        self.assertTrue(wg.w3_rule.sample_ok(reading))
        self.assertFalse(wg.w3_rule.sample_ok(dict(reading, residual_offset_mm=0.0)))
        self.assertFalse(wg.w3_rule.sample_ok(dict(reading, raw_offset_mm=0.0)))


if __name__ == "__main__":
    unittest.main()
