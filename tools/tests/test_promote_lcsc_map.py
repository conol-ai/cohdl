import contextlib
import hashlib
import io
import json
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import promote_lcsc_map as pm

ROOT = Path(__file__).resolve().parents[2]
RUN = ROOT / "tools/sourcing_data/lcsc_backfill/2026-10-30-sample200-aliases"
EMPTY = {"schema_version": 1, "supplier": "lcsc", "rows": []}


class PromoteTests(unittest.TestCase):
    def setUp(self):
        self.tempdir = tempfile.TemporaryDirectory()
        self.addCleanup(self.tempdir.cleanup)
        self.root = Path(self.tempdir.name)
        self.run_dir = self.root / "run"
        shutil.copytree(RUN, self.run_dir)
        self.map = self.root / "lcsc_map.json"
        self.map.write_text(json.dumps(EMPTY), encoding="utf-8")

    def promote(self, *extra):
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = pm.main([str(self.run_dir), "--map", str(self.map), *extra])
        return code, out.getvalue(), err.getvalue()

    def edit_run(self, name, change):
        """Edit one run file and re-record its hash, as a careless edit would."""
        path = self.run_dir / name
        data = json.loads(path.read_text(encoding="utf-8"))
        change(data)
        path.write_text(json.dumps(data), encoding="utf-8")
        provenance = json.loads((self.run_dir / "provenance.json").read_text(encoding="utf-8"))
        provenance["sha256"][name] = hashlib.sha256(path.read_bytes()).hexdigest()
        (self.run_dir / "provenance.json").write_text(json.dumps(provenance), encoding="utf-8")

    def test_the_committed_map_is_the_promoted_sample(self):
        code, out, _ = self.promote()
        self.assertEqual(code, 0)
        self.assertIn("108 rows, 108 changed", out)
        self.assertEqual(self.map.read_bytes(), (ROOT / "tools/sourcing_data/lcsc_map.json").read_bytes())
        # Promoting the same run again changes nothing.
        code, out, _ = self.promote()
        self.assertIn("108 rows, 0 changed", out)

    def test_check_writes_nothing(self):
        before = self.map.read_bytes()
        code, out, _ = self.promote("--check")
        self.assertEqual(code, 0)
        self.assertIn("108 rows, 108 changed", out)
        self.assertEqual(self.map.read_bytes(), before)

    def test_a_changed_run_file_is_refused(self):
        with (self.run_dir / "confirmed.json").open("a", encoding="utf-8") as handle:
            handle.write(" ")
        code, _, err = self.promote()
        self.assertEqual(code, 1)
        self.assertIn("confirmed.json does not match the SHA-256", err)
        self.assertEqual(json.loads(self.map.read_text()), EMPTY)

    def test_an_alias_table_other_than_the_reviewed_one_is_refused(self):
        aliases = self.root / "aliases.json"
        aliases.write_text('{"schema_version": 1, "aliases": {"YAGEO": "Yageo"}}', encoding="utf-8")
        code, _, err = self.promote("--aliases", str(aliases))
        self.assertEqual(code, 1)
        self.assertIn("different alias table", err)

    def test_a_row_its_confirmed_page_does_not_support_is_refused(self):
        def wrong_mpn(data):
            data["rows"][0]["mpn"] += "X"
        self.edit_run("lcsc_map.json", wrong_mpn)
        code, _, err = self.promote()
        self.assertEqual(code, 1)
        self.assertIn("no confirmed page reads", err)

        shutil.rmtree(self.run_dir)
        shutil.copytree(RUN, self.run_dir)

        def exact_for_alias(data):
            row = next(row for row in data["rows"] if row["rule"] == "alias")
            row["rule"] = "exact"
        self.edit_run("lcsc_map.json", exact_for_alias)
        code, _, err = self.promote()
        self.assertEqual(code, 1)
        self.assertIn("under rule exact", err)

    def test_a_conflict_with_the_official_map_writes_nothing(self):
        first = json.loads((RUN / "lcsc_map.json").read_text(encoding="utf-8"))["rows"][0]
        for clash in (dict(first, pn="C999999999"), dict(first, mpn=first["mpn"] + "-OTHER")):
            with self.subTest(clash=clash):
                self.map.write_text(json.dumps(dict(EMPTY, rows=[clash])), encoding="utf-8")
                before = self.map.read_bytes()
                code, _, err = self.promote()
                self.assertEqual(code, 1)
                self.assertIn("would name both", err)
                self.assertEqual(self.map.read_bytes(), before)


if __name__ == "__main__":
    unittest.main()
