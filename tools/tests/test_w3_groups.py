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
BOARD_4 = wg.PREVIEWS / "2026-11-11-board-4"


def before_board_4():
    """The tree as the rule saw it before board 4's preview was recorded (2026-11-11):
    that record is not judged. Its `short_at_preview` list still applies. The tests
    about the pilot and board 3 are about that state; `test_board_4_...` is about
    what the record changes."""
    real = wg.preview_inputs
    return mock.patch.object(wg, "preview_inputs",
                             lambda: {record: meta for record, meta in real().items() if record != BOARD_4})


def group(report, footprint, kind, mfr, series):
    return next(g for g in report["groups"]
                if (g["footprint"], g["type"], g["mfr"], g["series"]) == (footprint, kind, mfr, series))


class W3GroupsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = wg.build()
        with before_board_4():
            cls.before = wg.build()

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
        inputs.update({str(path.relative_to(wg.ROOT / "tools")): wg._sha256(path) for path in wg.board_inputs()})
        self.assertIn("jlc_previews/2026-11-11-board-4/w3.json", inputs)
        self.assertNotIn("frame_boards/preview-board-4/w3.json", inputs)
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
        self.assertEqual(group(self.before, "CHIP_0402", "MLCC", "Samsung", "CL")["status"], "needs_samples")
        with mock.patch.object(Path, "read_text", read_text), before_board_4():
            report = wg.build()
        changed = [(g["footprint"], g["mfr"], g["series"], g["anomalies"]) for g in report["groups"]
                   if g["status"] != group(self.before, g["footprint"], g["type"], g["mfr"], g["series"])["status"]]
        self.assertEqual(changed, [("CHIP_0402", "Samsung", "CL", ["C1525"])])

    def test_the_pilot_samples_the_0402_yageo_rc_group_and_not_the_others(self):
        rc = group(self.before, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("sampled", 49, 49))
        # Every RC sample is within the raw limit too, not only after the bias is taken off.
        self.assertTrue(all(p["raw_offset_mm"] <= p["offset_limit_mm"] for p in rc["passed"]))
        # Not drawn no longer pauses CRCW: its 9 undrawn samples do not count, so it needs 9 more.
        crcw = group(self.before, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["failed"]), ("needs_samples", 40, []))
        self.assertEqual(len(crcw["not_drawn"]), 9)
        self.assertTrue(all(c["status"] == "unverified" for n in crcw["not_drawn"] for c in n["outcomes"]))
        self.assertEqual(crcw["more_needed"], 9)
        # CC stays paused: C113781 is left for manual review (10.5 deg), whatever C576731 is.
        cc = group(self.before, "CHIP_0402", "MLCC", "Yageo", "CC")
        self.assertEqual(cc["status"], "paused")
        self.assertEqual([f["pn"] for f in cc["failed"]], ["C113781"])
        self.assertEqual([n["pn"] for n in cc["not_drawn"]], ["C576731"])

    def test_a_group_whose_only_fault_is_an_undrawn_sample_is_not_paused(self):
        # Board 3: C874860 is not drawn at any of its four placements; C309589 passes.
        r1210 = group(self.before, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], [n["pn"] for n in r1210["not_drawn"]]), ("needs_samples", ["C874860"]))
        self.assertNotIn("C874860", r1210["sample"])
        # 2512 also has C878096 off its residual limit, so it stays paused.
        r2512 = group(self.before, "CHIP_2512", "ChipResistor", "Yageo", "RC")
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
        with mock.patch.object(Path, "read_text", read_text), before_board_4():
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
        with mock.patch.object(Path, "read_text", read_text), before_board_4():
            report = wg.build()
        rc = group(report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual(rc["status"], "paused")
        self.assertEqual([f["pn"] for f in rc["failed"]], ["C138033"])

    def test_short_at_preview_lives_in_the_dated_record(self):
        records = {pn for meta in wg.preview_inputs().values() for pn in meta["short_at_preview"]}
        self.assertEqual(records, {"C185414", "C185430", "C470029", "C875102"})
        self.assertNotIn("C185414", (wg.ROOT / "tools/w3_groups.py").read_text(encoding="utf-8"))

    def test_a_board_short_at_its_bom_step_hands_the_fact_to_its_preview_record(self):
        # Board 4's first package (2026-11-11) stopped at the BOM step with no preview, so
        # the board's own w3.json carried the two numbers. Its second package was previewed
        # the same day; the record's w3.json holds the list now, and the board's is removed.
        self.assertEqual(wg.board_inputs(), {})
        self.assertFalse((wg.BOARDS / "preview-board-4/w3.json").exists())
        record = wg.PREVIEWS / "2026-11-11-board-4"
        self.assertEqual(list(wg.PREVIEWS.glob("*board-4*")), [record])
        meta = wg.preview_inputs()[record]
        short = meta["short_at_preview"]
        self.assertEqual(set(short), {"C470029", "C875102"})
        self.assertTrue(all(why.startswith("2026-11-11 preview-board-4 first package 1cd1883d ") for why in short.values()))
        # The note says the fact is the first package's, not a reading of this record's preview.
        self.assertIn("first package", meta["note"])
        self.assertIn("took no placement preview", meta["note"])
        # Neither number is on the package this record previewed.
        bom = (record / meta["bom"]).read_text(encoding="utf-8")
        self.assertFalse(any(pn in bom for pn in short))
        self.assertEqual(set(wg.SHORT_AT_PREVIEW), {"C185414", "C185430", "C470029", "C875102"})
        self.assertNotIn("C470029", (wg.ROOT / "tools/w3_groups.py").read_text(encoding="utf-8"))
        # A report built before the record's readings counted passes over both and takes each
        # one's nearest eligible neighbour (the package this record previewed); the committed
        # 2026-11-10 report, a dated snapshot, still names them.
        fresh = group(self.before, "CHIP_1210", "ChipResistor", "Yageo", "RC")["sample"]
        dated = group(json.loads((CURRENT / "groups.json").read_text(encoding="utf-8")),
                      "CHIP_1210", "ChipResistor", "Yageo", "RC")["sample"]
        self.assertEqual([(old, new) for old, new in zip(dated, fresh) if old != new],
                         [("C470029", "C7276637"), ("C875102", "C728392")])
        # Without the board's list the same tree chooses the dated sample.
        with mock.patch.object(wg, "SHORT_AT_PREVIEW", {pn: wg.SHORT_AT_PREVIEW[pn] for pn in ("C185414", "C185430")}), \
                before_board_4():
            self.assertEqual(group(wg.build(), "CHIP_1210", "ChipResistor", "Yageo", "RC")["sample"], dated)

    def test_board_4_as_the_rule_judges_its_record(self):
        # The 2026-11-11 preview of board 4, read by measure_preview.py as the rule takes it.
        # (The record's README shows that the readings include each part's silkscreen text;
        # that is evidence for a decision about the reading, and changes nothing here.)
        record = str((BOARD_4 / "README.md").relative_to(wg.ROOT))

        def here(items):
            return sorted(x["pn"] for x in items if any(c["record"] == record for c in x["outcomes"]))
        # D, Yageo RC 1210: 42 samples, 28 passed, none failed, 14 not drawn. Not paused.
        r1210 = group(self.report, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], len(r1210["passed"]), r1210["failed"], r1210["more_needed"]),
                         ("needs_samples", 29, [], 14))
        self.assertEqual(sum(record in p["records"] for p in r1210["passed"]), 28)
        self.assertEqual(len(here(r1210["not_drawn"])), 14)
        self.assertEqual(len(r1210["not_drawn"]), 15)
        self.assertIn("C7276637", here(r1210["not_drawn"]))  # one of the two replacements
        # D, Vishay CRCW 0402: 9 samples, 7 passed, C482153 left for manual review, C2085363 not
        # drawn. The two calibration numbers fail at their turned copies. Paused.
        crcw = group(self.report, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["more_needed"]), ("paused", 45, 4))
        self.assertEqual(here(crcw["failed"]), ["C481918", "C482097", "C482153"])
        self.assertEqual(here(crcw["not_drawn"]), ["C2085363"])
        self.assertEqual(len(crcw["not_drawn"]), 10)
        # C, Yageo RC 0402: the three calibration numbers the pilot passed fail here, so the
        # group the pilot sampled is paused while this record counts.
        rc = group(self.report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("paused", 46, 49))
        self.assertEqual(here(rc["failed"]), ["C138033", "C185445", "C60491"])
        self.assertEqual(group(self.before, "CHIP_0402", "ChipResistor", "Yageo", "RC")["status"], "sampled")
        # Every group this record judges, and what it is afterwards.
        judged = {(g["footprint"], g["mfr"], g["series"]): g["status"] for g in self.report["groups"]
                  if here(g["failed"]) or here(g["not_drawn"]) or any(record in p["records"] for p in g["passed"])}
        self.assertEqual(judged, {
            ("CHIP_0201", "Yageo", "RC"): "paused", ("CHIP_0402", "Vishay", "CRCW"): "paused",
            ("CHIP_0402", "Yageo", "RC"): "paused", ("CHIP_0402", "Samsung", "CL"): "paused",
            ("CHIP_0402", "Yageo", "CC"): "paused", ("CHIP_0603", "Samsung", "CL"): "paused",
            ("CHIP_0603", "Yageo", "CC"): "paused", ("CHIP_1210", "Yageo", "RC"): "needs_samples",
            ("CHIP_2010", "Yageo", "RC"): "paused", ("CHIP_2512", "Yageo", "RC"): "paused"})

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
