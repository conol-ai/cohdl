"""The lcsc-library skill names only files that exist, and its backfill wrapper fails closed."""

import hashlib
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


if __name__ == "__main__":
    unittest.main()
