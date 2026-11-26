"""The lcsc-library skill names only files that exist, and its backfill wrapper fails closed."""

from contextlib import redirect_stdout
import hashlib
import io
import json
from pathlib import Path
import re
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

    def run_with(self, approved_urls, left, review_rows=()):
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
        approved.batch_identities = lambda subset: {("M", "in")}
        approved.lb.main = fake_main
        try:
            with tempfile.TemporaryDirectory() as d:
                path = Path(d) / "urls.json"
                data = approved.list_bytes(approved_urls)
                path.write_bytes(data)
                args = approved.build_parser().parse_args([
                    "run", "--approved", str(path), "--sha256", hashlib.sha256(data).hexdigest(),
                    "--subset", d, "--cache-dir", d, "--output-dir", str(Path(d) / "out"), "--date", "2026-11-25"])
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


if __name__ == "__main__":
    unittest.main()
