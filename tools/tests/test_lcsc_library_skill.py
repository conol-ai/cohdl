"""The lcsc-library skill names only files that exist, and its backfill wrapper fails closed."""

from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]
ROOT = TOOLS.parent
sys.path.insert(0, str(TOOLS))

import lcsc_backfill_approved as approved  # noqa: E402
from sourcing.http import PolicyError  # noqa: E402

SKILL = ROOT / ".claude/skills/lcsc-library/SKILL.md"
PATH = re.compile(r"`((?:\.claude|docs|lib|tools)/[^`\s]+)`")


class SkillTests(unittest.TestCase):
    def test_every_concrete_path_the_skill_names_exists(self):
        text = SKILL.read_text()
        paths = [p for p in PATH.findall(text) if not re.search(r"[<>*…]", p)]
        self.assertGreater(len(paths), 15)
        missing = [p for p in paths if not (ROOT / p.rstrip("/")).exists()]
        self.assertEqual(missing, [])

    def test_the_skill_defers_to_the_library_authoring_rules(self):
        # Device/part shape, rating semantics and pad conventions live in LIBRARY.md.
        self.assertTrue((ROOT / "LIBRARY.md").is_file())
        self.assertIn("`LIBRARY.md`", SKILL.read_text().split("## Modes")[0])

    def test_steps_are_numbered_in_order_and_every_step_reference_exists(self):
        text = SKILL.read_text()
        numbers = [int(n) for n in re.findall(r"^### (\d+)\. ", text, re.M)]
        self.assertEqual(numbers, list(range(1, len(numbers) + 1)))
        refs = {int(n) for n in re.findall(r"\bsteps? (\d+)", text, re.I)}
        self.assertTrue(refs)
        self.assertLessEqual(refs, set(numbers))

    def test_the_skill_has_its_front_matter(self):
        head = SKILL.read_text().split("---")[1]
        self.assertIn("name: lcsc-library", head)
        self.assertIn("description:", head)

    def test_the_skill_grants_no_standing_approval_of_its_own(self):
        # An approval belongs to the run that has it, named in its report; the skill is shared.
        text = SKILL.read_text()
        for phrase in ("Alex", "pre-approved", "preapproved"):
            self.assertNotIn(phrase, text)

    def test_both_skills_build_previews_without_reference_designators(self):
        wrapper = "tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes"
        self.assertTrue((ROOT / wrapper).is_file())
        for skill in (SKILL, ROOT / ".claude/skills/jlc-preview/SKILL.md"):
            clis = re.findall(r"--kicad-cli\s+([^\s`]+)", skill.read_text())
            self.assertTrue(clis, skill)
            self.assertEqual(set(clis), {wrapper}, skill)

    def test_kicad_per_user_files_are_ignored_and_none_is_tracked(self):
        self.assertIn("*.kicad_prl", (ROOT / ".gitignore").read_text().splitlines())
        if not (ROOT / ".git").exists():
            self.skipTest("not a git checkout")
        tracked = subprocess.run(["git", "ls-files", "*.kicad_prl"], cwd=ROOT, capture_output=True, text=True, check=True)
        self.assertEqual(tracked.stdout, "")

    def test_extracted_scripts_carry_no_local_paths(self):
        for name in ("regen_locks.py", "examples_golden.sh", "lcsc_backfill_approved.py"):
            self.assertNotIn("/Users/", (TOOLS / name).read_text(), name)


class FakeClient:
    def __init__(self, cached):
        self.cached, self.sent = set(cached), []

    def is_cached(self, url):
        return url in self.cached

    def get(self, url):
        self.sent.append(url)
        return url


class GuardTests(unittest.TestCase):
    def test_an_uncached_page_off_the_list_is_refused_before_any_request(self):
        undo = approved.guard(FakeClient, frozenset({"https://jlcpcb.com/partdetail/a"}))
        try:
            client = FakeClient({"https://jlcpcb.com/partdetail/cached"})
            with self.assertRaises(PolicyError):
                client.get("https://jlcpcb.com/partdetail/b")
            self.assertEqual(client.sent, [])
            client.get("https://jlcpcb.com/partdetail/a")
            client.get("https://jlcpcb.com/partdetail/cached")
            self.assertEqual(client.sent, ["https://jlcpcb.com/partdetail/a", "https://jlcpcb.com/partdetail/cached"])
        finally:
            undo()
        FakeClient(set()).get("https://jlcpcb.com/partdetail/b")  # undone: no guard left behind

    def test_a_list_whose_hash_is_not_the_approved_one_is_refused(self):
        with tempfile.TemporaryDirectory() as d:
            path = Path(d) / "urls.json"
            data = approved.list_bytes(["https://jlcpcb.com/partdetail/a"])
            path.write_bytes(data)
            self.assertEqual(approved.check_approved(path, hashlib.sha256(data).hexdigest()),
                             ["https://jlcpcb.com/partdetail/a"])
            with self.assertRaises(SystemExit):
                approved.check_approved(path, "0" * 64)
            path.write_text(json.dumps(["b", "a"]))
            with self.assertRaises(SystemExit):
                approved.check_approved(path, hashlib.sha256(path.read_bytes()).hexdigest())


class RunTests(unittest.TestCase):
    """cmd_run with the offline scan and the backfill itself replaced by fakes."""

    A, B, C = (f"https://jlcpcb.com/partdetail/{x}" for x in "abc")

    def run_with(self, approved_urls, left, review_rows=(), batch=None):
        calls = []

        def fake_main(argv):
            calls.append(argv)
            out = Path(argv[argv.index("--output-dir") + 1])
            # The shape lcsc_backfill.py writes: one row per identity for review, with its reason.
            rows = [dict(row, candidates=[], reason="fetch_failed") for row in review_rows]
            rows.append({"candidates": [], "mfr": "M", "mpn": "other", "reason": "no_candidate"})
            (out / "review.json").write_text(json.dumps({"fetch_failures": 0, "rows": rows, "schema_version": 1}))
            return 0

        saved = approved.uncached_candidates, approved.batch_identities, approved.lb.main
        approved.uncached_candidates = lambda subset, cache: (2, left)
        approved.batch_identities = lambda path: batch if batch and path.name == "batch" else {("M", "in")}
        approved.lb.main = fake_main
        try:
            with tempfile.TemporaryDirectory() as d:
                path = Path(d) / "urls.json"
                data = approved.list_bytes(approved_urls)
                path.write_bytes(data)
                args = approved.build_parser().parse_args([
                    "run", "--approved", str(path), "--sha256", hashlib.sha256(data).hexdigest(),
                    "--subset", d, "--cache-dir", d, "--output-dir", str(Path(d) / "out"), "--date", "2026-11-25",
                    "--map", str(Path(d) / "no-map.json")] + (["--batch", str(Path(d) / "batch")] if batch else []))
                with redirect_stdout(io.StringIO()) as out:
                    status = approved.cmd_run(args)
                return status, calls, out.getvalue()
        finally:
            approved.uncached_candidates, approved.batch_identities, approved.lb.main = saved

    def test_a_stopped_run_resumes_over_the_rest_of_the_approved_list(self):
        status, calls, _ = self.run_with([self.A, self.B], [self.B])
        self.assertEqual(status, 0)
        argv = calls[0]
        self.assertEqual(argv[argv.index("--max-requests") + 1], "3")

    def test_an_uncached_page_off_the_approved_list_stops_the_run_before_the_backfill(self):
        with self.assertRaises(SystemExit):
            self.run_with([self.A], [self.A, self.C])

    def test_failed_identities_are_counted_in_and_outside_the_batch(self):
        rows = [{"mfr": "M", "mpn": "in"}, {"mfr": "M", "mpn": "out"}, {"mfr": "N", "mpn": "in"}]
        _, _, printed = self.run_with([self.A], [self.A], rows)
        self.assertIn("fetch_failed: 1 in the batch, 2 outside it", printed)

    def test_with_the_whole_batch_named_the_count_says_how_many_were_not_planned_for(self):
        # SUBSET is the snapshot-listed part ("M", "in"); the batch also holds ("M", "out").
        rows = [{"mfr": "M", "mpn": "in"}, {"mfr": "M", "mpn": "out"}, {"mfr": "N", "mpn": "in"}]
        _, _, printed = self.run_with([self.A], [self.A], rows, batch={("M", "in"), ("M", "out")})
        self.assertIn("fetch_failed: 2 in the batch (1 of them planned for, 1 not planned for), 1 outside it", printed)


class CarriedTests(unittest.TestCase):
    """An identity the map holds and the library lacks is read too, so the map keeps one snapshot."""

    def test_a_map_identity_missing_from_the_library_is_added_to_a_copy_of_it(self):
        with tempfile.TemporaryDirectory() as d:
            lib = Path(d) / "lib"
            (lib / "p/src").mkdir(parents=True)
            (lib / "p/src/a.cohdl").write_text('pub part A: D { primary { mfr: "M", mpn: "held" } }\n')
            (lib / "p/docs").mkdir()
            (lib / "p/docs/sheet.pdf").write_bytes(b"%PDF")
            path = Path(d) / "map.json"
            path.write_text(json.dumps({"rows": [{"mfr": "M", "mpn": "held"}, {"mfr": "UNI-ROYAL", "mpn": "0402X"}]}))
            carried = approved.carried_identities(lib, path)
            self.assertEqual(carried, [("UNI-ROYAL", "0402X")])
            copy = approved.lib_with_carried(lib, carried, Path(d) / "tmp")
            read = {(r.mfr, r.mpn) for r in approved.lb.extract_avl_rows(copy)}
            self.assertEqual(read, {("M", "held"), ("UNI-ROYAL", "0402X")})
            self.assertFalse(list(copy.rglob("*.pdf")))  # sources only
            self.assertEqual(approved.carried_identities(copy, path), [])

    def test_the_official_map_has_an_identity_the_library_lacks(self):
        # The case the rule exists for: without it a plain run leaves this row on an older snapshot.
        carried = approved.carried_identities(ROOT / "lib", ROOT / "tools/sourcing_data/lcsc_map.json")
        self.assertIn(("UNI-ROYAL", "0402WGF1803TCE"), carried)


if __name__ == "__main__":
    unittest.main()
