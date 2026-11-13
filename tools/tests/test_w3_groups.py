import csv
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
# Dated snapshots under earlier rules; none is compared with a fresh build.
DATED = {"2026-11-10": (3, "w3-2026-11-10b"), "2026-11-11": (4, "w3-2026-11-11")}
CURRENT = wg.DATA / "w3_sampling/2026-11-10"
METHOD_2 = wg.DATA / "w3_sampling/2026-11-11-method-2"
BOARD_3 = wg.PREVIEWS / "2026-11-05-c2-board-3"
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


def method_1():
    """The rule as it read chip bodies until 2026-11-11: `measure_preview.py` method 1,
    the model together with the silkscreen in its window, and the run's own check of
    each body. The tests of what the records said then are about that reading."""
    return mock.patch.dict(wg.w3_rule.RULE, body_method=1)


def group(report, footprint, kind, mfr, series):
    return next(g for g in report["groups"]
                if (g["footprint"], g["type"], g["mfr"], g["series"]) == (footprint, kind, mfr, series))


class W3GroupsTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.report = wg.build()
        with before_board_4():
            cls.before = wg.build()
        with method_1():
            cls.report_1 = wg.build()
            with before_board_4():
                cls.before_1 = wg.build()

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
                  for name in ("w3.json", meta["bom"], meta["checks"], "w3-readings.json",
                               "remeasure-2026-11-11-method-2.json")}
        self.assertFalse(any(name.endswith(("/measurements.json", "/remeasure-2026-11-10.json")) for name in inputs))
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

    def test_the_later_dated_reports_are_consistent_with_their_recorded_sources(self):
        # The 2026-11-10 report (rule w3-2026-11-10b) and the 2026-11-11 one (rule
        # w3-2026-11-11): like the 2026-11-07 one, dated snapshots, checked against
        # what they record rather than a fresh build. Their lcsc_map.json and
        # jlc_frames.json hashes are those of their day: frame rows merged since make
        # a report older, not wrong, so those two are not compared with the tree.
        for day, (schema, rule) in DATED.items():
            with self.subTest(report=day):
                folder = wg.DATA / "w3_sampling" / day
                committed = json.loads((folder / "groups.json").read_text(encoding="utf-8"))
                self.assertEqual((folder / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
                self.assertEqual(committed["schema_version"], schema)
                self.assertEqual(committed["rule"]["version"], rule)
                for name, digest in committed["sources"].items():
                    if name.startswith("jlc_previews/") or name.startswith("lcsc_backfill/"):
                        self.assertEqual(wg._sha256(wg.DATA / name), digest, name)

    def test_the_2026_11_11_report_records_the_two_follow_up_conditions(self):
        committed = json.loads((wg.DATA / "w3_sampling/2026-11-11/groups.json").read_text(encoding="utf-8"))
        rule = committed["rule"]
        self.assertEqual((rule["bias_min_other_bodies"], rule["bias_other_share_above"],
                          rule["not_drawn_max_share"], rule["not_drawn_excess"]), (3, 0.5, 0.1, "pause"))
        self.assertEqual({view.split("/")[-2] + view.split("README.md")[1]: made["bodies"]
                          for view, made in committed["views"].items()},
                         {"2026-11-05-c2-board-3#bottom": 12, "2026-11-05-c2-board-3#top": 39,
                          "2026-11-07-w3-pilot-0402#top": 89})
        status = {(g["footprint"], g["mfr"], g["series"]): g["status"] for g in committed["groups"]}
        self.assertEqual(status[("CHIP_0402", "Yageo", "RC")], "sampled")
        self.assertEqual(status[("CHIP_0402", "Vishay", "CRCW")], "paused")
        self.assertEqual(sorted(status.values()).count("paused"), 6)
        self.assertFalse(any(g["bias_unsupported"] for g in committed["groups"]))

    def test_the_method_2_report_is_what_the_tree_builds_under_the_proposed_rule(self):
        # The 2026-11-11 report under the proposed rule w3-2026-11-11-m2 (chip bodies read
        # by method 2). A dated snapshot like the others, checked against what it records.
        committed = json.loads((METHOD_2 / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((METHOD_2 / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        self.assertEqual((committed["schema_version"], committed["rule"]["version"], committed["rule"]["body_method"]),
                         (3, "w3-2026-11-11-m2", 2))
        for name, digest in committed["sources"].items():
            if name.startswith("jlc_previews/") or name.startswith("lcsc_backfill/"):
                self.assertEqual(wg._sha256(wg.DATA / name), digest, name)
        # It reads each record's method 2 readings, and none of the method 1 files.
        previews = sorted(name for name in committed["sources"] if name.startswith("jlc_previews/"))
        self.assertEqual(sum(name.endswith("/remeasure-2026-11-11-method-2.json") for name in previews), 3)
        self.assertFalse(any(name.endswith(("/measurements.json", "/remeasure-2026-11-10.json")) for name in previews))
        self.assertTrue(all(p["body_method"] == 2 for g in committed["groups"] for p in g["passed"]))
        self.assertIn("method 2", (METHOD_2 / "groups.md").read_text(encoding="utf-8"))
        # The earlier reports render as they did: they name no method.
        self.assertNotIn("method", (CURRENT / "groups.md").read_text(encoding="utf-8"))

    def test_the_2026_11_12_report_is_under_the_combined_rule(self):
        # Rule w3-2026-11-12: method 2 readings, with the two follow-up conditions. A dated
        # snapshot like the others, checked against what it records.
        folder = wg.DATA / "w3_sampling/2026-11-12"
        committed = json.loads((folder / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((folder / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        rule = committed["rule"]
        self.assertEqual((committed["schema_version"], rule["version"], rule["body_method"]), (4, "w3-2026-11-12", 2))
        self.assertEqual({k: v for k, v in rule.items() if k != "short_at_preview"}, wg.w3_rule.RULE)
        for name, digest in committed["sources"].items():
            if name.startswith("jlc_previews/") or name.startswith("lcsc_backfill/"):
                self.assertEqual(wg._sha256(wg.DATA / name), digest, name)
        previews = sorted(name for name in committed["sources"] if name.startswith("jlc_previews/"))
        self.assertEqual(sum(name.endswith("/remeasure-2026-11-11-method-2.json") for name in previews), 3)
        self.assertFalse(any(name.endswith(("/measurements.json", "/remeasure-2026-11-10.json")) for name in previews))
        # Five views, each median over method 2 readings; every passed sample names its
        # method and its view, and none is unsupported.
        self.assertEqual({view.split("/")[-2] + view.split("README.md")[1]: made["bodies"]
                          for view, made in committed["views"].items()},
                         {"2026-11-05-c2-board-3#bottom": 12, "2026-11-05-c2-board-3#top": 39,
                          "2026-11-07-w3-pilot-0402#top": 89, "2026-11-11-board-4#bottom": 84,
                          "2026-11-11-board-4#top": 120})
        passed = [p for g in committed["groups"] for p in g["passed"]]
        self.assertEqual(len(passed), 161)
        self.assertTrue(all(p["body_method"] == 2 and wg.w3_rule.sample_ok(p) and wg.w3_rule.bias_supported(p)
                            for p in passed))
        self.assertFalse(any(g["bias_unsupported"] for g in committed["groups"]))
        status = {(g["footprint"], g["mfr"], g["series"]): g["status"] for g in committed["groups"]}
        self.assertEqual(sorted(key for key, value in status.items() if value == "sampled"),
                         [("CHIP_0402", "Yageo", "CC"), ("CHIP_0402", "Yageo", "RC")])
        self.assertEqual(sorted(key for key, value in status.items() if value == "paused"),
                         [("CHIP_0201", "Yageo", "RC"), ("CHIP_0402", "Vishay", "CRCW"),
                          ("CHIP_0805", "Yageo", "CC"), ("CHIP_1210", "Yageo", "RC")])
        text = (folder / "groups.md").read_text(encoding="utf-8")
        self.assertIn("method 2", text)
        self.assertIn("not drawn: 10 of 58 judged (17.2%), above 10%", text)
        self.assertIn("not drawn: 15 of 44 judged (34.1%), above 10%", text)

    def test_the_2026_11_12b_report_is_the_combined_rule_at_passive_0_2_7(self):
        # The same rule and records as 2026-11-12, on the tree that binds the 0402 Yageo
        # RC group (passive 0.2.7 and its frame rows): no group's outcome differs.
        first = json.loads((wg.DATA / "w3_sampling/2026-11-12/groups.json").read_text(encoding="utf-8"))
        folder = wg.DATA / "w3_sampling/2026-11-12b"
        committed = json.loads((folder / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((folder / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        self.assertEqual((first["sources"]["lib/passive"], committed["sources"]["lib/passive"]), ("0.2.6", "0.2.7"))
        self.assertEqual(committed["rule"], first["rule"])
        self.assertEqual(committed["views"], first["views"])
        for name, digest in committed["sources"].items():
            if name.startswith("jlc_previews/") or name.startswith("lcsc_backfill/"):
                self.assertEqual(wg._sha256(wg.DATA / name), digest, name)
        outcome = lambda report: [(g["footprint"], g["type"], g["mfr"], g["series"], g["status"], g["passed"],
                                   g["failed"], g["not_drawn"], g["bias_unsupported"], g["not_drawn_share"])
                                  for g in report["groups"]]
        self.assertEqual(outcome(committed), outcome(first))
        self.assertEqual({g["package_version"] for g in committed["groups"]}, {"0.2.7"})

    def test_the_2026_11_13_report_is_the_tree_with_the_hongjiacheng_pages(self):
        # The same rule, records and library as 2026-11-12b, on the tree whose map cites
        # the 2026-11-11 Hongjiacheng backfill's confirmed pages (six more parts, none of
        # them a chip resistor or capacitor): no group, member or sample differs.
        before = json.loads((wg.DATA / "w3_sampling/2026-11-12b/groups.json").read_text(encoding="utf-8"))
        folder = wg.DATA / "w3_sampling/2026-11-13"
        committed = json.loads((folder / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((folder / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        changed = {"lcsc_map.json", "jlc_frames.json"}
        pages = lambda report: [name for name in report["sources"] if name.startswith("lcsc_backfill/")]
        self.assertEqual((pages(before), pages(committed)),
                         (["lcsc_backfill/2026-11-07-samsung-b1/confirmed.json"],
                          ["lcsc_backfill/2026-11-11-hongjiacheng/confirmed.json"]))
        rest = lambda report: {name: digest for name, digest in report["sources"].items()
                               if name not in changed and not name.startswith("lcsc_backfill/")}
        self.assertEqual(rest(committed), rest(before))
        self.assertEqual({key: value for key, value in committed.items() if key != "sources"},
                         {key: value for key, value in before.items() if key != "sources"})

    def test_the_2026_11_13b_report_is_the_tree_with_three_more_frame_rows(self):
        # The tree that binds C7519, C131941 and C189961 (esd 0.1.4, load-switch 0.2.1,
        # @contrib/audio-amp 0.2.1). None is a chip part, so only the frame table's
        # digest differs from 2026-11-13.
        before = json.loads((wg.DATA / "w3_sampling/2026-11-13/groups.json").read_text(encoding="utf-8"))
        folder = wg.DATA / "w3_sampling/2026-11-13b"
        committed = json.loads((folder / "groups.json").read_text(encoding="utf-8"))
        self.assertEqual((folder / "groups.md").read_text(encoding="utf-8"), wg.render(committed))
        self.assertNotEqual(committed["sources"]["jlc_frames.json"], before["sources"]["jlc_frames.json"])
        self.assertEqual(dict(committed, sources=dict(committed["sources"], **{"jlc_frames.json": None})),
                         dict(before, sources=dict(before["sources"], **{"jlc_frames.json": None})))

    def test_samples_follow_the_rule(self):
        for g in self.report["groups"]:
            rows = {row[0]: row for row in g["member_rows"]}
            direct = {d["pn"] for d in g["direct"]}
            passed = {p["pn"] for p in g["passed"]}
            failed = {f["pn"] for f in g["failed"]}
            not_drawn = {n["pn"] for n in g["not_drawn"]}
            unsupported = {u["pn"] for u in g["bias_unsupported"]}
            self.assertFalse(passed & failed or passed & not_drawn or failed & not_drawn)
            self.assertFalse(unsupported & (passed | failed | not_drawn))
            self.assertEqual(g["judged"], len(passed | failed | not_drawn | unsupported))
            # Not drawn: every copy either passed or not drawn, and at least one not drawn.
            for n in g["not_drawn"]:
                self.assertTrue(any(c["not_drawn"] for c in n["outcomes"]))
                self.assertTrue(all(c["ok"] or c["not_drawn"] for c in n["outcomes"]))
            self.assertEqual(g["required"], max(3, math.ceil(0.05 * g["members"])))
            self.assertEqual(g["more_needed"], max(0, g["required"] - len(passed)))
            self.assertLessEqual(len(g["sample"]), g["more_needed"])
            excess = len(not_drawn) > 0.1 * max(g["judged"], g["required"])
            self.assertEqual(g["not_drawn_excess"], excess)
            self.assertEqual(g["status"] == "paused", bool(g["failed"] or g["anomalies"] or excess))
            for p in g["passed"]:
                self.assertTrue(wg.w3_rule.sample_ok(p))
                # Its bias comes from a view the group names, with other groups' bodies in the majority.
                self.assertTrue(wg.w3_rule.bias_supported(p))
                self.assertEqual((p["bias_bodies"], p["bias_other_bodies"]),
                                 (g["views"][p["bias_view"]]["bodies"], g["views"][p["bias_view"]]["other_bodies"]))
                self.assertGreater(2 * p["bias_other_bodies"], p["bias_bodies"])
            self.assertTrue(wg.w3_rule.views_consistent(g["passed"]))
            tried = direct | passed | failed | not_drawn | unsupported
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
        # As method 1 read it, kept reproducible, with the rule's two conditions applied.
        rc = group(self.before_1, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("sampled", 49, 49))
        # Every RC sample is within the raw limit too, not only after the bias is taken off.
        self.assertTrue(all(p["raw_offset_mm"] <= p["offset_limit_mm"] for p in rc["passed"]))
        # The pilot view's median is taken over 89 bodies, 45 of them not RC: one more than RC's 44.
        pilot = [p for p in rc["passed"] if "w3-pilot-0402" in p["bias_view"]]
        self.assertEqual({(p["bias_bodies"], p["bias_other_bodies"]) for p in pilot}, {(89, 45)})
        self.assertEqual({(p["bias_bodies"], p["bias_other_bodies"]) for p in rc["passed"] if p not in pilot},
                         {(39, 34)})
        # No CRCW sample fails, but 9 of its 49 judged members are not drawn: above the
        # rule's 10%, so the group is paused (under w3-2026-11-10b it only needed 9 more).
        crcw = group(self.before_1, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["failed"]), ("paused", 40, []))
        self.assertEqual(len(crcw["not_drawn"]), 9)
        self.assertTrue(all(c["status"] == "unverified" for n in crcw["not_drawn"] for c in n["outcomes"]))
        self.assertEqual((crcw["judged"], crcw["not_drawn_share"], crcw["not_drawn_excess"]), (49, 0.1837, True))
        self.assertIn("not drawn: 9 of 49 judged (18.4%), above 10%", wg.render(self.before_1))
        # With the share at 20% the same readings leave it needing samples, as before.
        with mock.patch.dict(wg.w3_rule.RULE, not_drawn_max_share=0.2), method_1(), before_board_4():
            crcw = group(wg.build(), "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], crcw["more_needed"], crcw["not_drawn_excess"]), ("needs_samples", 9, False))
        # CC stays paused: C113781 is left for manual review (10.5 deg), whatever C576731 is.
        cc = group(self.before_1, "CHIP_0402", "MLCC", "Yageo", "CC")
        self.assertEqual(cc["status"], "paused")
        self.assertEqual([f["pn"] for f in cc["failed"]], ["C113781"])
        self.assertEqual([n["pn"] for n in cc["not_drawn"]], ["C576731"])
        self.assertEqual(max(p["raw_offset_mm"] for p in rc["passed"]), 0.121)
        self.assertEqual(max(abs(p["angle_deg"]) for p in rc["passed"]), 9.9)

    def test_the_pilot_and_board_3_as_method_2_reads_them(self):
        # The same two records with the model read alone (board 4 left out).
        rc = group(self.before, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("sampled", 49, 49))
        self.assertEqual({p["pn"] for p in rc["passed"]},
                         {p["pn"] for p in group(self.before_1, "CHIP_0402", "ChipResistor", "Yageo", "RC")["passed"]})
        # The margins: 0.050 of 0.128 mm raw (0.121 by method 1), 0.036 of 0.064 mm residual, 2.6 deg.
        self.assertEqual(max(p["raw_offset_mm"] for p in rc["passed"]), 0.05)
        self.assertEqual(max(p["residual_offset_mm"] for p in rc["passed"]), 0.036)
        self.assertEqual(max(abs(p["angle_deg"]) for p in rc["passed"]), 2.6)
        # The pilot's view median: (+0.005, -0.077) mm by method 1, the text above each part.
        pilot = str((PILOT / "README.md").relative_to(wg.ROOT))
        self.assertEqual({tuple(p["bias_xy_mm"]) for p in rc["passed"] if p["records"] == [pilot]}, {(0.011, -0.01)})
        # The two conditions count bodies and part numbers, not offsets, so they read the
        # same by either method: 45 of the pilot's 89 bodies are not RC, and CRCW's 9 of
        # 49 not drawn pause it.
        self.assertEqual({(p["bias_bodies"], p["bias_other_bodies"]) for p in rc["passed"] if p["records"] == [pilot]},
                         {(89, 45)})
        crcw = group(self.before, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["failed"], len(crcw["not_drawn"])),
                         ("paused", 40, [], 9))
        self.assertTrue(crcw["not_drawn_excess"])
        # C113781, left for manual review at 10.5 deg by method 1, reads within 2.6 deg: CC is not
        # paused (its 1 not drawn of 10 judged is exactly 10%, not above it).
        cc = group(self.before, "CHIP_0402", "MLCC", "Yageo", "CC")
        self.assertEqual((cc["status"], len(cc["passed"]), cc["failed"], cc["more_needed"]), ("needs_samples", 9, [], 1))
        self.assertIn("C113781", {p["pn"] for p in cc["passed"]})

    def test_method_2_still_fails_bodies_drawn_off_their_pads(self):
        # Board 3 draws its three 0805 Yageo CC models 0.11-0.15 mm toward one pad, leaving
        # the other bare, and its 0201 RC at -143 deg 0.10 mm along its pads. With the
        # model read alone these still fail: one that method 1 passed (C527206), and
        # C138161 now on the raw offset as well.
        board_3 = str((BOARD_3 / "README.md").relative_to(wg.ROOT))
        cc = group(self.report, "CHIP_0805", "MLCC", "Yageo", "CC")
        self.assertEqual((cc["status"], [f["pn"] for f in cc["failed"]]), ("paused", ["C527206", "C527232", "C576985"]))
        residuals = {f["pn"]: [c["reading"]["residual_offset_mm"] for c in f["outcomes"]] for f in cc["failed"]}
        self.assertEqual(residuals, {"C527206": [0.12], "C527232": [0.138], "C576985": [0.156]})
        self.assertTrue(all(c["record"] == board_3 and c["status"] == "body" and c["reading"]["offset_limit_mm"] == 0.228
                            for f in cc["failed"] for c in f["outcomes"]))
        self.assertEqual([f["pn"] for f in group(self.report_1, "CHIP_0805", "MLCC", "Yageo", "CC")["failed"]],
                         ["C527232", "C576985"])
        rc = group(self.report, "CHIP_0201", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], [f["pn"] for f in rc["failed"]]), ("paused", ["C138161"]))
        worst = max((c for f in rc["failed"] for c in f["outcomes"] if c["reading"]),
                    key=lambda c: c["reading"]["raw_offset_mm"])
        self.assertEqual((worst["record"], worst["ref"], worst["status"], worst["reading"]["raw_offset_mm"],
                          worst["reading"]["offset_limit_mm"]), (board_3, "R4", "manual review", 0.099, 0.08))
        # Every copy either method judges: one passed by method 1 fails by method 2, and no other.
        with method_1():
            one = wg.judged_members()
        two = wg.judged_members()
        self.assertEqual(set(one), set(two))
        newly_failing = [(key[1], b["ref"]) for key in sorted(two) for a, b in zip(one[key], two[key])
                         if a["ok"] and not b["ok"]]
        self.assertEqual(newly_failing, [("C527206", "C12")])
        self.assertEqual((sum(c["ok"] for v in one.values() for c in v), sum(c["ok"] for v in two.values() for c in v)),
                         (225, 340))

    def test_every_group_under_method_1_and_method_2(self):
        # What the reading changes, group by group, with the rule's two conditions applied
        # to each: (a) method 1 without board 4's record, (b) method 1 with it, (c) method 2
        # with it. (Without the conditions, as on main and under PR #111 alone, CRCW 0402 in
        # (a) and RC 1210 in (b) and (c) need samples instead: the not-drawn share pauses them.)
        def statuses(report):
            return {(g["footprint"], g["mfr"], g["series"]): (g["status"], len(g["passed"]), len(g["failed"]))
                    for g in report["groups"]}
        a, b, c = statuses(self.before_1), statuses(self.report_1), statuses(self.report)
        moved = {key: (a[key], b[key], c[key]) for key in a if len({a[key], b[key], c[key]}) > 1}
        self.assertEqual(moved, {
            ("CHIP_0402", "Vishay", "CRCW"): (("paused", 40, 0), ("paused", 45, 3), ("paused", 48, 0)),
            ("CHIP_0402", "Yageo", "RC"): (("sampled", 49, 0), ("paused", 46, 3), ("sampled", 49, 0)),
            ("CHIP_0402", "Samsung", "CL"): (("needs_samples", 0, 0), ("paused", 0, 1), ("needs_samples", 1, 0)),
            ("CHIP_0402", "Yageo", "CC"): (("paused", 8, 1), ("paused", 7, 5), ("sampled", 12, 0)),
            ("CHIP_0603", "Samsung", "CL"): (("needs_samples", 0, 0), ("paused", 1, 1), ("needs_samples", 2, 0)),
            ("CHIP_0603", "Yageo", "CC"): (("needs_samples", 0, 0), ("paused", 0, 1), ("needs_samples", 1, 0)),
            ("CHIP_0805", "Yageo", "CC"): (("paused", 2, 2), ("paused", 2, 2), ("paused", 1, 3)),
            ("CHIP_1210", "Yageo", "RC"): (("needs_samples", 1, 0), ("paused", 29, 0), ("paused", 29, 0)),
            ("CHIP_2010", "Yageo", "RC"): (("paused", 1, 1), ("paused", 1, 1), ("needs_samples", 2, 0)),
            ("CHIP_2512", "Yageo", "RC"): (("paused", 0, 1), ("paused", 0, 1), ("needs_samples", 1, 0)),
        })
        count = lambda report: tuple(sum(g["status"] == s for g in report["groups"])
                                     for s in ("sampled", "needs_samples", "paused"))
        self.assertEqual((count(self.before_1), count(self.report_1), count(self.report)),
                         ((1, 13, 6), (0, 9, 11), (2, 14, 4)))
        # Without board 4's record method 2 gives (1, 16, 3): Yageo CC 0402 is then one sample
        # short, and RC 1210 has only 1 not drawn against its 43 required.
        self.assertEqual(count(self.before), (1, 16, 3))
        # What pauses the four under method 2: a failed sample, or the not-drawn share.
        self.assertEqual({(g["footprint"], g["mfr"], g["series"]): (len(g["failed"]), g["not_drawn_excess"])
                          for g in self.report["groups"] if g["status"] == "paused"},
                         {("CHIP_0201", "Yageo", "RC"): (1, False), ("CHIP_0402", "Vishay", "CRCW"): (0, True),
                          ("CHIP_0805", "Yageo", "CC"): (3, False), ("CHIP_1210", "Yageo", "RC"): (0, True)})
        # No reading of any record is unsupported: every view's median holds enough other bodies.
        self.assertFalse(any(g["bias_unsupported"] for r in (self.before, self.report) for g in r["groups"]))

    def test_readings_by_another_method_are_not_read_as_the_rules(self):
        # A record with no readings by the rule's method is not judged ...
        real = wg.readings_file
        with mock.patch.object(wg, "readings_file", lambda record, meta, method=None:
                               [] if record == BOARD_4 else real(record, meta, method)):
            self.assertEqual(wg.build()["groups"], self.before["groups"])
        # ... and a file listed for the rule's method that holds another method's readings stops the build.
        with mock.patch.object(wg, "readings_file", lambda record, meta, method=None: [meta["measurements"]]):
            with self.assertRaisesRegex(SystemExit, "listed as method 2 readings"):
                wg.judged_members()
        for record, meta in wg.preview_inputs().items():
            self.assertEqual(wg.readings_file(record, meta, 1), [meta["measurements"]])
            self.assertEqual(wg.readings_file(record, meta, 2), ["w3-readings.json", "remeasure-2026-11-11-method-2.json"])
            self.assertEqual(wg.readings_file(record, meta, 3), [])

    def test_a_group_whose_only_fault_is_an_undrawn_sample_is_not_paused(self):
        # Board 3: C874860 is not drawn at any of its four placements; C309589 passes.
        r1210 = group(self.before, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], [n["pn"] for n in r1210["not_drawn"]]), ("needs_samples", ["C874860"]))
        self.assertNotIn("C874860", r1210["sample"])
        # One of its two judged members: half of them, but 1 is within 10% of the 43
        # the group requires, so it is not an excess while so few have been judged.
        self.assertEqual((r1210["judged"], r1210["required"], r1210["not_drawn_excess"]), (2, 43, False))
        # By method 1, 2512 also had C878096 off its residual limit (0.169 of 0.125 mm) and
        # was paused; with the model alone it reads 0.081 mm and passes.
        r2512 = group(self.before_1, "CHIP_2512", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r2512["status"], [f["pn"] for f in r2512["failed"]]), ("paused", ["C878096"]))
        r2512 = group(self.before, "CHIP_2512", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r2512["status"], [p["pn"] for p in r2512["passed"]], len(r2512["not_drawn"])),
                         ("needs_samples", ["C878096"], 1))

    def pilot_without(self, drop):
        """The pilot's readings (by the rule's method) without the bodies DROP(part
        number) selects: a board on which those models were never drawn. Board 4 is
        left out, as in the other tests about the pilot and board 3."""
        meta = json.loads((PILOT / "w3.json").read_text(encoding="utf-8"))
        name = wg.readings_file(PILOT, meta)[-1]
        readings = json.loads((PILOT / name).read_text(encoding="utf-8"))
        with (PILOT / meta["bom"]).open(encoding="utf-8-sig", newline="") as handle:
            pns = {ref.strip(): row["LCSC Part #"] for row in csv.DictReader(handle)
                   for ref in row["Designator"].split(",")}
        kept = [m for m in readings if not ("offset_xy_mm" in m.get("body", {}) and drop(pns[m["ref"]]))]
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == PILOT / name:
                return json.dumps(kept)
            return real(path, *args, **kwargs)
        with mock.patch.object(Path, "read_text", read_text), before_board_4():
            return wg.build(), len(readings) - len(kept)

    def test_a_median_of_the_groups_own_bodies_supports_no_sample(self):
        # Tony's condition (2026-11-11): on a board of one group the residual only
        # shows that the group agrees with itself. The pilot with nothing but RC drawn:
        rc = group(self.before, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        members = {row[0] for row in rc["member_rows"]}
        report, dropped = self.pilot_without(lambda pn: pn not in members)
        self.assertEqual(dropped, 45)
        alone = group(report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        # The 44 pilot samples, all within the limits against RC's own median, still do
        # not count; the 5 on board 3, whose view other groups fill, do. (By method 1
        # C137880's residual against RC's own median was over the limit and it failed
        # outright; by method 2 none does.)
        self.assertEqual((len(alone["passed"]), len(alone["bias_unsupported"]),
                          [f["pn"] for f in alone["failed"]]), (5, 44, []))
        self.assertEqual(alone["status"], "needs_samples")
        self.assertEqual({p["bias_view"] for p in alone["passed"]},
                         {"tools/sourcing_data/jlc_previews/2026-11-05-c2-board-3/README.md#top"})
        for u in alone["bias_unsupported"]:
            (c,) = u["outcomes"]
            self.assertTrue(c["ok"] and not c["supported"])
            self.assertEqual((c["reading"]["bias_bodies"], c["reading"]["bias_other_bodies"]), (44, 0))
        self.assertNotIn(alone["bias_unsupported"][0]["pn"], alone["sample"])

    def test_a_group_that_is_half_of_its_view_supports_no_sample(self):
        # The pilot has 44 RC bodies and 45 others. One other body fewer and RC is half
        # of the median, which may then sit on an RC body: more than half must be others.
        crcw = group(self.before, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        one = next(p["pn"] for p in crcw["passed"] if "w3-pilot-0402" in p["bias_view"])
        report, dropped = self.pilot_without(lambda pn: pn == one)
        self.assertEqual(dropped, 1)
        rc = group(report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), len(rc["bias_unsupported"])), ("needs_samples", 5, 44))
        self.assertEqual(rc["views"][str((PILOT / "README.md").relative_to(wg.ROOT)) + "#top"],
                         {"bodies": 88, "other_bodies": 44})

    def pilot_with_uniform_offset(self, offset_y):
        """The pilot's readings with every drawn body moved to (0, OFFSET_Y): the
        reviewer's counterexample, where the board median takes the whole offset off."""
        meta = json.loads((PILOT / "w3.json").read_text(encoding="utf-8"))
        name = wg.readings_file(PILOT, meta)[-1]
        readings = json.loads((PILOT / name).read_text(encoding="utf-8"))
        for m in readings:
            if "offset_xy_mm" in m.get("body", {}):
                m["body"]["offset_xy_mm"] = [0.0, offset_y]
                m["body"]["offset_limit_mm"] = 0.127
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == PILOT / name:
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
        name = wg.readings_file(PILOT, meta)[-1]
        readings = json.loads((PILOT / name).read_text(encoding="utf-8"))
        # R8 is C137920, the pilot's largest angle by method 2 (-2.6 deg).
        target = next(m for m in readings if m["ref"] == "R8")
        self.assertEqual((target["body"]["delta_mod_180"], target["body_ok"]), (-2.6, True))
        target["body"]["delta_mod_180"] = 10.5
        real = Path.read_text

        def read_text(path, *args, **kwargs):
            if path == PILOT / name:
                return json.dumps(readings)
            return real(path, *args, **kwargs)
        with mock.patch.object(Path, "read_text", read_text), before_board_4():
            report = wg.build()
        rc = group(report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual(rc["status"], "paused")
        self.assertEqual([f["pn"] for f in rc["failed"]], ["C137920"])

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

    def test_board_4_as_method_2_judges_its_record(self):
        record = str((BOARD_4 / "README.md").relative_to(wg.ROOT))

        def here(items):
            return sorted(x["pn"] for x in items if any(c["record"] == record for c in x["outcomes"]))
        # No chip body of this record fails: every one of its 204 drawn bodies reads along its pads, centred.
        self.assertEqual([c["ref"] for g in self.report["groups"] for f in g["failed"] for c in f["outcomes"]
                          if c["record"] == record and not c["ok"]], [])
        # (C138161, the 0201 calibration number, passes at all ten copies here and stays
        # failed by board 3's R4.)
        self.assertEqual([here(g["failed"]) for g in self.report["groups"] if here(g["failed"])], [["C138161"]])
        # D, Yageo RC 1210: as by method 1, 28 passed and 14 not drawn; the offsets are smaller.
        # No sample fails, but 15 of its 44 judged part numbers are not drawn (34.1%): paused.
        r1210 = group(self.report, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], len(r1210["passed"]), r1210["failed"], r1210["more_needed"]),
                         ("paused", 29, [], 14))
        self.assertEqual((r1210["judged"], r1210["not_drawn_share"], r1210["not_drawn_excess"]), (44, 0.3409, True))
        self.assertEqual(len(here(r1210["not_drawn"])), 14)
        self.assertEqual(max(p["raw_offset_mm"] for p in r1210["passed"]), 0.036)
        # D, Vishay CRCW 0402: 8 of the 9 samples pass (C482153 read 10.1 deg by method 1, -0.8 by
        # method 2); C2085363 is not drawn. With the two calibration numbers, 48 passed of 49,
        # and 10 of 58 judged not drawn (17.2%): paused, as it was before this record.
        crcw = group(self.report, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["failed"], crcw["more_needed"]),
                         ("paused", 48, [], 1))
        self.assertEqual((crcw["judged"], crcw["not_drawn_share"], crcw["not_drawn_excess"]), (58, 0.1724, True))
        self.assertEqual(here(crcw["not_drawn"]), ["C2085363"])
        self.assertEqual(len(crcw["not_drawn"]), 10)
        # C, Yageo RC 0402: the three calibration numbers pass at all ten copies each, so the
        # group stays as the pilot left it.
        rc = group(self.report, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["failed"]), ("sampled", 49, []))
        for pn in ("C138033", "C185445", "C60491"):
            self.assertIn(record, next(p["records"] for p in rc["passed"] if p["pn"] == pn))
        # A/C, Yageo CC 0402: three gap parts and the calibration numbers pass, which takes the
        # group from 9 passed to 12 of the 10 required. This record is what samples it.
        cc = group(self.report, "CHIP_0402", "MLCC", "Yageo", "CC")
        self.assertEqual((cc["status"], len(cc["passed"]), cc["required"], cc["failed"], len(cc["not_drawn"])),
                         ("sampled", 12, 10, [], 1))
        self.assertEqual((cc["judged"], cc["not_drawn_share"], cc["not_drawn_excess"]), (13, 0.0769, False))
        # Each of this record's views holds bodies of ten groups or more: for RC 0402, 15 of
        # its own among 120 on top and 84 on the bottom.
        self.assertEqual({view.rsplit("#")[1]: (made["bodies"], made["other_bodies"])
                          for view, made in rc["views"].items() if view.startswith(record)},
                         {"top": (120, 105), "bottom": (84, 69)})
        self.assertEqual(sorted(p["pn"] for p in cc["passed"] if p["records"] == [record]),
                         ["C106255", "C107006", "C113780"])
        self.assertEqual(group(self.before, "CHIP_0402", "MLCC", "Yageo", "CC")["status"], "needs_samples")

    def test_board_4_as_method_1_judged_its_record(self):
        # The 2026-11-11 preview of board 4, read by measure_preview.py method 1, as the rule
        # on main takes it: what PR #111 alone gives. (The record's README shows that these
        # readings include each part's silkscreen text.)
        record = str((BOARD_4 / "README.md").relative_to(wg.ROOT))

        def here(items):
            return sorted(x["pn"] for x in items if any(c["record"] == record for c in x["outcomes"]))
        # D, Yageo RC 1210: 42 samples, 28 passed, none failed, 14 not drawn. Under
        # w3-2026-11-10b that left it needing samples; the not-drawn share (15 of 44) pauses it.
        r1210 = group(self.report_1, "CHIP_1210", "ChipResistor", "Yageo", "RC")
        self.assertEqual((r1210["status"], len(r1210["passed"]), r1210["failed"], r1210["more_needed"]),
                         ("paused", 29, [], 14))
        self.assertEqual(sum(record in p["records"] for p in r1210["passed"]), 28)
        self.assertEqual(len(here(r1210["not_drawn"])), 14)
        self.assertEqual(len(r1210["not_drawn"]), 15)
        self.assertIn("C7276637", here(r1210["not_drawn"]))  # one of the two replacements
        # D, Vishay CRCW 0402: 9 samples, 7 passed, C482153 left for manual review, C2085363 not
        # drawn. The two calibration numbers fail at their turned copies. Paused.
        crcw = group(self.report_1, "CHIP_0402", "ChipResistor", "Vishay", "CRCW")
        self.assertEqual((crcw["status"], len(crcw["passed"]), crcw["more_needed"]), ("paused", 45, 4))
        self.assertEqual(here(crcw["failed"]), ["C481918", "C482097", "C482153"])
        self.assertEqual(here(crcw["not_drawn"]), ["C2085363"])
        self.assertEqual(len(crcw["not_drawn"]), 10)
        # C, Yageo RC 0402: the three calibration numbers the pilot passed fail here, so the
        # group the pilot sampled is paused while this record counts.
        rc = group(self.report_1, "CHIP_0402", "ChipResistor", "Yageo", "RC")
        self.assertEqual((rc["status"], len(rc["passed"]), rc["required"]), ("paused", 46, 49))
        self.assertEqual(here(rc["failed"]), ["C138033", "C185445", "C60491"])
        self.assertEqual(group(self.before_1, "CHIP_0402", "ChipResistor", "Yageo", "RC")["status"], "sampled")
        # Every group this record judges, and what it is afterwards.
        judged = {(g["footprint"], g["mfr"], g["series"]): g["status"] for g in self.report_1["groups"]
                  if here(g["failed"]) or here(g["not_drawn"]) or any(record in p["records"] for p in g["passed"])}
        self.assertEqual(judged, {
            ("CHIP_0201", "Yageo", "RC"): "paused", ("CHIP_0402", "Vishay", "CRCW"): "paused",
            ("CHIP_0402", "Yageo", "RC"): "paused", ("CHIP_0402", "Samsung", "CL"): "paused",
            ("CHIP_0402", "Yageo", "CC"): "paused", ("CHIP_0603", "Samsung", "CL"): "paused",
            ("CHIP_0603", "Yageo", "CC"): "paused", ("CHIP_1210", "Yageo", "RC"): "paused",
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
        return [wg.w3_rule.sample_reading(0.0, o, 0.127, bias, body_method=2) for o in offsets]

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
        reading = wg.w3_rule.sample_reading(0.0, (0.0, 0.06), 0.127, (0.0, 0.0), body_method=2)
        self.assertTrue(wg.w3_rule.sample_ok(reading))
        self.assertFalse(wg.w3_rule.sample_ok(dict(reading, residual_offset_mm=0.0)))
        self.assertFalse(wg.w3_rule.sample_ok(dict(reading, raw_offset_mm=0.0)))

    def test_a_reading_by_another_method_is_not_a_sample(self):
        # The limits are the rule's only for its own reading: a method 1 reading well
        # inside them, or one that names no method (which is method 1), does not pass.
        self.assertEqual(wg.w3_rule.RULE["body_method"], 2)
        good = wg.w3_rule.sample_reading(0.0, (0.0, 0.02), 0.127, (0.0, 0.0), body_method=2)
        self.assertTrue(wg.w3_rule.sample_ok(good))
        self.assertEqual(wg.w3_rule.sample_reading(0.0, (0.0, 0.02), 0.127, (0.0, 0.0))["body_method"], 1)
        self.assertFalse(wg.w3_rule.sample_ok(wg.w3_rule.sample_reading(0.0, (0.0, 0.02), 0.127, (0.0, 0.0))))
        self.assertFalse(wg.w3_rule.sample_ok({k: v for k, v in good.items() if k != "body_method"}))
        for other in (1, 3, "2", 2.5, True, None):
            self.assertFalse(wg.w3_rule.sample_ok(dict(good, body_method=other)), other)

    def test_the_limits_are_the_accepted_ones(self):
        # w3-2026-11-10b, as accepted on 2026-11-11, is unchanged: the reading method and
        # the two follow-up conditions are the only additions.
        rule = dict(wg.w3_rule.RULE)
        self.assertEqual((rule.pop("version"), rule.pop("body_method")), ("w3-2026-11-12", 2))
        self.assertEqual((rule.pop("bias_min_other_bodies"), rule.pop("bias_other_share_above"),
                          rule.pop("not_drawn_max_share"), rule.pop("not_drawn_excess")), (3, 0.5, 0.1, "pause"))
        self.assertEqual(rule, {
            "package": "passive", "types": ["ChipResistor", "MLCC"], "min_sample": 3, "fraction": 0.05,
            "min_stock": 30, "angle_limit_deg": 10.0, "offset_raw_fraction": 1.0,
            "offset_residual_fraction": 0.5, "offset_bias": "board_median",
            "infer_zero_stock": False, "infer_not_drawn": False})
        committed = json.loads((CURRENT / "groups.json").read_text(encoding="utf-8"))["rule"]
        self.assertEqual({k: v for k, v in committed.items() if k not in ("version", "short_at_preview")}, rule)


class W3FollowUpConditionTests(unittest.TestCase):
    """Tony's two conditions of 2026-11-11, on the rule itself."""

    def reading(self, bodies, other, view="record#top"):
        return wg.w3_rule.sample_reading(0.0, (0.0, 0.0), 0.127, (0.0, 0.0), view, bodies, other)

    def test_the_bias_needs_other_groups_in_the_majority(self):
        ok = wg.w3_rule.bias_supported
        self.assertTrue(ok(self.reading(89, 45)))       # the pilot, for Yageo RC
        self.assertFalse(ok(self.reading(88, 44)))      # half is not more than half
        self.assertFalse(ok(self.reading(44, 0)))       # a single-group board
        self.assertFalse(ok(self.reading(3, 2)))        # a majority, but only 2 bodies
        self.assertTrue(ok(self.reading(5, 3)))
        self.assertFalse(ok(self.reading(5, 6)))        # more others than bodies
        self.assertFalse(ok(self.reading(5, 3, view=" ")))
        self.assertFalse(ok(wg.w3_rule.sample_reading(0.0, (0.0, 0.0), 0.127, (0.0, 0.0))))
        self.assertFalse(ok(dict(self.reading(5, 3), bias_bodies=5.0)))

    def test_samples_of_one_view_must_describe_it_alike(self):
        same = [dict(self.reading(7, 4), pn=str(n)) for n in range(3)]
        self.assertTrue(wg.w3_rule.views_consistent(same))
        self.assertFalse(wg.w3_rule.views_consistent(same + [self.reading(8, 5)]))
        self.assertFalse(wg.w3_rule.views_consistent(same + [dict(self.reading(7, 4), bias_xy_mm=[0.0, 0.01])]))
        # Four samples on a view that holds three bodies of their group.
        self.assertFalse(wg.w3_rule.views_consistent(same + [self.reading(7, 4)]))
        self.assertTrue(wg.w3_rule.views_consistent(same + [self.reading(8, 5, view="record#bottom")]))

    def test_not_drawn_above_a_tenth_of_the_judged_members_is_an_excess(self):
        excess = wg.w3_rule.not_drawn_excess
        self.assertTrue(excess(9, 49, 968))     # Vishay CRCW 0402: 18.4%
        self.assertTrue(excess(5, 49, 968))     # 10.2%
        self.assertFalse(excess(4, 49, 968))    # 8.2%
        self.assertFalse(excess(1, 10, 199))    # Yageo CC 0402: exactly 10% is not above it
        self.assertTrue(excess(2, 11, 199))
        # While fewer are judged than required, the share is of the required sample.
        self.assertFalse(excess(1, 2, 846))     # Yageo RC 1210: 1 of the 43 required
        self.assertFalse(excess(4, 5, 846))
        self.assertTrue(excess(5, 5, 846))      # 5 of 43 is 11.6%


if __name__ == "__main__":
    unittest.main()
