"""Offline W2 AVL indexing, matching, sampling and schema tests."""

from datetime import date
import json
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from lcsc_backfill import (  # noqa: E402
    AVLRow, build_index, choose_candidate, extract_avl_rows, load_aliases,
    confirmation_mpn, manufacturer_match, normalize_mpn, reproducible_sample, run_backfill,
)
from sourcing.http import RunStoppedError  # noqa: E402
from sourcing.sitemap import filter_partdetail_urls  # noqa: E402
from sourcing_test_support import LocalServer, install_socket_guard, make_client, page  # noqa: E402


def validate_schema(document):
    assert set(document) == {"schema_version", "supplier", "rows"}
    assert document["schema_version"] == 1 and document["supplier"] == "lcsc"
    fields = {"mfr", "mpn", "pn", "rule", "category", "snapshot_sha256", "date"}
    for row in document["rows"]:
        assert set(row) == fields
        assert all(isinstance(row[key], str) and row[key] for key in ("mfr", "mpn", "pn"))
        assert row["rule"] in ("exact", "alias", "manual")
        assert row["category"] in ("basic", "preferred", "extended")
        assert re.fullmatch(r"[0-9a-f]{64}", row["snapshot_sha256"])
        date.fromisoformat(row["date"])


class BackfillTests(unittest.TestCase):
    def setUp(self):
        install_socket_guard(self)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_primary_alt_alias_priority_and_schema(self):
        lib = self.root / "lib/pkg"
        lib.mkdir(parents=True)
        (lib / "parts.cohdl").write_text('''
            pub part X: Y {
              primary { mfr: "Acme", mpn: "A-1", footprint: F }
              alt { mfr: "Other", mpn: "B 2" }
              alt { mfr: "Empty", mpn: "" }
            }
        ''')
        avl = extract_avl_rows(self.root / "lib")
        self.assertEqual([(row.kind, row.mpn) for row in avl],
                         [("primary", "A-1"), ("alt", "B 2"), ("alt", "")])
        sitemap = filter_partdetail_urls([
            "http://127.0.0.1/partdetail/Acme-A_1/C1/C1",
            "http://127.0.0.1/partdetail/Other-B_2/C2/C2",
            "http://127.0.0.1/partdetail/Acme-A_1/C3/C3",
        ])
        index = build_index(sitemap, [])
        self.assertEqual(normalize_mpn("a- 1"), "A1")
        self.assertEqual([c.c for c in index["A1"]], ["C1", "C3"])
        responses = {
            "/partdetail/Acme-A_1/C1/C1": (200, page("C1", "A-1", category="Extended", stock=99)),
            "/partdetail/Other-B_2/C2/C2": (200, page("C2", "B 2", mfr="Other Local")),
            "/partdetail/Acme-A_1/C3/C3": (200, page("C3", "A-1", category="Basic", stock=0)),
        }
        aliases_path = self.root / "aliases.json"
        aliases_path.write_text(json.dumps({"schema_version": 1,
                                            "aliases": {"Other Local": "Other"}}))
        with LocalServer(responses) as server:
            client = make_client(self.root / "cache")
            result = run_backfill(avl, index, origin=server.url(), client=client,
                                  output_dir=self.root / "out", aliases=load_aliases(aliases_path),
                                  snapshot_date="2026-10-30")
        output = json.loads((self.root / "out/lcsc_map.json").read_text())
        validate_schema(output)
        confirmed_bytes = (self.root / "out/confirmed.json").read_bytes()
        import hashlib
        confirmed_hash = hashlib.sha256(confirmed_bytes).hexdigest()
        self.assertTrue(all(row["snapshot_sha256"] == confirmed_hash for row in output["rows"]))
        self.assertEqual(result["snapshot_sha256"], confirmed_hash)
        self.assertEqual(result["sha256"]["confirmed.json"], confirmed_hash)
        confirmed = json.loads(confirmed_bytes)
        self.assertEqual(set(confirmed[0]), {"c", "mpn", "mfr", "package", "category", "stock",
                                             "datasheet", "partdetail", "fetched_date"})
        self.assertEqual([(row["mpn"], row["pn"], row["rule"]) for row in output["rows"]],
                         [("A-1", "C3", "exact"), ("B 2", "C2", "alias")])
        self.assertEqual(result["counts"]["mapped_identities"], 2)

    def test_tie_goes_to_review_and_stock_breaks_tie(self):
        base = {"mfr": "Acme", "mpn": "X", "partdetail": "/partdetail/X/C1/C1"}
        rows = [{**base, "c": "C1", "category": "basic", "stock": 5, "rule": "exact"},
                {**base, "c": "C2", "category": "basic", "stock": 5, "rule": "exact"}]
        best, tied = choose_candidate(rows)
        self.assertIsNone(best)
        self.assertEqual(len(tied), 1)
        rows[1]["stock"] = 6
        best, tied = choose_candidate(rows)
        self.assertEqual(best["c"], "C2")
        self.assertEqual(tied, [])

    def test_tie_review_dry_run_budget_and_resume(self):
        avl = [AVLRow("x.cohdl", "primary", "Acme", "X")]
        sitemap = filter_partdetail_urls([
            "http://127.0.0.1/partdetail/X/C1/C1",
            "http://127.0.0.1/partdetail/X/C2/C2",
        ])
        index = build_index(sitemap, [])
        responses = {"/partdetail/X/C1/C1": (200, page("C1", "X", stock=5)),
                     "/partdetail/X/C2/C2": (200, page("C2", "X", stock=5))}
        dry_client = make_client(self.root / "dry")
        dry = run_backfill(avl, index, origin="http://127.0.0.1:9", client=dry_client,
                           output_dir=self.root / "dry-out", aliases={}, snapshot_date="2026-10-30",
                           dry_run=True)
        self.assertEqual((dry_client.request_count, dry["planned_partdetail_requests"]), (0, 2))
        with LocalServer(responses) as server:
            limited = make_client(self.root / "cache", max_requests=2)
            first = run_backfill(avl, index, origin=server.url(), client=limited,
                                 output_dir=self.root / "one", aliases={}, snapshot_date="2026-10-30")
            self.assertTrue(first["budget_exhausted"])
            before = len(server.requests)
            resumed = make_client(self.root / "cache", max_requests=10)
            result = run_backfill(avl, index, origin=server.url(), client=resumed,
                                  output_dir=self.root / "two", aliases={}, snapshot_date="2026-10-30")
            self.assertEqual(resumed.request_count, 1)
            self.assertEqual(len(server.requests) - before, 1)
            review = json.loads((self.root / "two/review.json").read_text())
            self.assertEqual(review["rows"][0]["reason"], "tied_after_priority")
            self.assertEqual(len(review["rows"][0]["candidates"]), 2)

    def test_reproducible_sample(self):
        rows = [AVLRow("x", "primary", "M", f"PN-{i}") for i in range(20)]
        rows.append(AVLRow("x", "alt", "M", "PN-1"))
        one = reproducible_sample(rows, 5, 42)
        self.assertEqual(one, reproducible_sample(rows, 5, 42))
        self.assertNotEqual(one, reproducible_sample(rows, 5, 43))
        self.assertEqual(len(one), 5)

    def test_empty_alias_file_format(self):
        path = self.root / "aliases.json"
        path.write_text('{"schema_version": 1, "aliases": {}}')
        self.assertEqual(load_aliases(path), {})
        self.assertIsNone(manufacturer_match("Acme", "Acme Local", {}))
        self.assertEqual(manufacturer_match("Acme", "Acme", {}), "exact")

    def test_confirmation_does_not_erase_punctuation(self):
        self.assertEqual(normalize_mpn("MP1482DS-LF-Z"), normalize_mpn("MP1482DSLFZ"))
        self.assertNotEqual(confirmation_mpn("MP1482DS-LF-Z"), confirmation_mpn("MP1482DSLFZ"))
        self.assertEqual(confirmation_mpn(" mp 1482 ds-lf-z "), confirmation_mpn("MP1482DS-LF-Z"))
        avl = [AVLRow("x", "primary", "Acme", "MP1482DS-LF-Z")]
        sitemap = filter_partdetail_urls(["http://127.0.0.1/partdetail/Acme-MP1482DS_LF_Z/C1/C1"])
        with LocalServer({"/partdetail/Acme-MP1482DS_LF_Z/C1/C1":
                          (200, page("C1", "MP1482DSLFZ"))}) as server:
            result = run_backfill(avl, build_index(sitemap, []), origin=server.url(),
                                  client=make_client(self.root / "cache"), output_dir=self.root / "out",
                                  aliases={}, snapshot_date="2026-10-30")
        review = json.loads((self.root / "out/review.json").read_text())
        self.assertEqual(review["rows"][0]["reason"], "no_confirmed_match")
        self.assertEqual(json.loads((self.root / "out/lcsc_map.json").read_text())["rows"], [])

    def test_preferred_comes_from_the_candidate_list(self):
        avl = [AVLRow("x", "primary", "Acme", "X")]
        sitemap = filter_partdetail_urls(["http://127.0.0.1/partdetail/X/C1/C1"])
        with LocalServer({"/partdetail/X/C1/C1": (200, page("C1", "X", category="Preferred"))}) as server:
            run_backfill(avl, build_index(sitemap, []), origin=server.url(),
                         client=make_client(self.root / "cache"), output_dir=self.root / "out",
                         aliases={}, snapshot_date="2026-10-30", preferred=frozenset({"C1"}))
        rows = json.loads((self.root / "out/lcsc_map.json").read_text())["rows"]
        self.assertEqual([(row["pn"], row["category"]) for row in rows], [("C1", "preferred")])

    def test_manufacturer_mismatch_is_proposed_not_applied(self):
        avl = [AVLRow("x", "primary", "Yageo", "RC0402"), AVLRow("y", "primary", "Yageo", "RC0603")]
        sitemap = filter_partdetail_urls(["http://127.0.0.1/partdetail/YAGEO-RC0402/C1/C1",
                                          "http://127.0.0.1/partdetail/YAGEO-RC0603/C2/C2"])
        with LocalServer({"/partdetail/YAGEO-RC0402/C1/C1": (200, page("C1", "RC0402", mfr="YAGEO")),
                          "/partdetail/YAGEO-RC0603/C2/C2": (200, page("C2", "RC0603", mfr="YAGEO"))}) as server:
            run_backfill(avl, build_index(sitemap, []), origin=server.url(),
                         client=make_client(self.root / "cache"), output_dir=self.root / "out",
                         aliases={}, snapshot_date="2026-10-30")
        out = self.root / "out"
        self.assertEqual(json.loads((out / "lcsc_map.json").read_text())["rows"], [])
        review = json.loads((out / "review.json").read_text())["rows"]
        self.assertEqual({row["reason"] for row in review}, {"manufacturer_not_matched"})
        self.assertEqual(review[0]["candidates"][0]["mfr"], "YAGEO")
        proposals = json.loads((out / "alias_proposals.json").read_text())["rows"]
        self.assertEqual([(row["supplier"], row["official"], row["identities"]) for row in proposals],
                         [("YAGEO", "Yageo", 2)])

    def test_page_for_another_part_is_not_confirmed(self):
        avl = [AVLRow("x", "primary", "Acme", "X")]
        sitemap = filter_partdetail_urls(["http://127.0.0.1/partdetail/X/C1/C1"])
        with LocalServer({"/partdetail/X/C1/C1": (200, page("C2", "X"))}) as server:
            run_backfill(avl, build_index(sitemap, []), origin=server.url(),
                         client=make_client(self.root / "cache"), output_dir=self.root / "out",
                         aliases={}, snapshot_date="2026-10-30")
        self.assertEqual(json.loads((self.root / "out/confirmed.json").read_text()), [])
        self.assertEqual(json.loads((self.root / "out/lcsc_map.json").read_text())["rows"], [])

    def test_run_stopped_is_not_swallowed(self):
        avl = [AVLRow("x", "primary", "Acme", "X")]
        sitemap = filter_partdetail_urls(["http://127.0.0.1/partdetail/X/C1/C1",
                                          "http://127.0.0.1/partdetail/X/C2/C2"])
        with LocalServer({"/partdetail/X/C1/C1": (403, "no"),
                          "/partdetail/X/C2/C2": (403, "no"),
                          "/robots.txt": (200, "User-agent: *\nAllow: /\n")}) as server:
            # First refusal retries and stops on the second consecutive 403.
            result = run_backfill(avl, build_index(sitemap, []), origin=server.url(),
                                  client=make_client(self.root / "cache"), output_dir=self.root / "out",
                                  aliases={}, snapshot_date="2026-10-30")
        self.assertTrue(result["run_stopped"])
        self.assertIn("RunStoppedError", result["run_stopped_reason"])
        self.assertEqual(result["not_fetched"][0]["reason"], "not_fetched")

    def test_isolated_help(self):
        result = subprocess.run([sys.executable, "-I", str(TOOLS / "lcsc_backfill.py"), "--help"],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)


if __name__ == "__main__":
    unittest.main()
