"""Offline orchestration tests for the W1 JLC catalog tool."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))
sys.path.insert(0, str(Path(__file__).resolve().parent))

from jlc_catalog import (  # noqa: E402
    canonical_url, load_candidates, parse_partdetail, run_catalog, with_preferred)
from sourcing.http import RobotsError  # noqa: E402
from sourcing.sitemap import filter_partdetail_urls  # noqa: E402
from sourcing_test_support import LocalServer, install_socket_guard, make_client, page  # noqa: E402


class CatalogTests(unittest.TestCase):
    def setUp(self):
        install_socket_guard(self)
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def fixture(self, responses=None):
        paths = [
            "/partdetail/First-MPN/C1/C1",
            "/partdetail/Second-MPN/C2/C2",
        ]
        sitemap = filter_partdetail_urls("http://127.0.0.1" + path for path in paths)
        candidates = [{"c": "C1", "mpn": "First-MPN", "candidate_category": "basic"},
                      {"c": "C2", "mpn": "Second-MPN", "candidate_category": "preferred"},
                      {"c": "C3", "mpn": "Missing", "candidate_category": "basic"}]
        return candidates, sitemap

    def test_full_paths_snapshot_and_unresolved(self):
        candidates, sitemap = self.fixture()
        # C1 is a Basic candidate whose page now reads Extended; C2 is Preferred.
        responses = {"/partdetail/First-MPN/C1/C1": (200, page("C1", "First-MPN", category="Extended")),
                     "/partdetail/Second-MPN/C2/C2": (200, page("C2", "Second-MPN", category="Preferred"))}
        with LocalServer(responses) as server:
            client = make_client(self.root / "cache")
            result = run_catalog(candidates, sitemap, origin=server.url(), client=client,
                                 output_dir=self.root / "snapshot")
            self.assertEqual(server.requests.count("/partdetail/First-MPN/C1/C1"), 1)
            self.assertNotIn("/partdetail/C1", server.requests)
            rows = json.loads((self.root / "snapshot/catalog.json").read_text())
            self.assertEqual([row["c"] for row in rows], ["C1", "C2"])
            self.assertEqual(set(rows[0]), {"c", "mpn", "mfr", "package", "category", "stock",
                                            "datasheet", "partdetail"})
            self.assertEqual(json.loads((self.root / "snapshot/unresolved.json").read_text()), ["C3"])
            self.assertEqual(result["counts"]["catalog"], 2)
            self.assertEqual([row["category"] for row in rows], ["extended", "preferred"])
            self.assertEqual(result["category_changed"],
                             [{"c": "C1", "candidates": "basic", "page": "extended"}])

    def test_cache_resume_and_budget_do_not_repeat_requests(self):
        candidates, sitemap = self.fixture()
        responses = {"/partdetail/First-MPN/C1/C1": (200, page("C1", "First-MPN")),
                     "/partdetail/Second-MPN/C2/C2": (200, page("C2", "Second-MPN"))}
        with LocalServer(responses) as server:
            first = make_client(self.root / "cache", max_requests=2)
            result = run_catalog(candidates, sitemap, origin=server.url(), client=first,
                                 output_dir=self.root / "one")
            self.assertTrue(result["budget_exhausted"])
            self.assertEqual(first.request_count, 2)  # robots + one partdetail
            count = len(server.requests)
            second = make_client(self.root / "cache", max_requests=10)
            result = run_catalog(candidates, sitemap, origin=server.url(), client=second,
                                 output_dir=self.root / "two")
            self.assertEqual(second.request_count, 1)
            self.assertEqual(len(server.requests) - count, 1)
            self.assertFalse(result["budget_exhausted"])

    def test_dry_run_is_zero_request(self):
        candidates, sitemap = self.fixture()
        client = make_client(self.root / "cache")
        result = run_catalog(candidates, sitemap, origin="http://127.0.0.1:9", client=client,
                             output_dir=self.root / "unused", dry_run=True)
        self.assertEqual(client.request_count, 0)
        self.assertEqual(result["planned_partdetail_requests"], 2)
        self.assertEqual(result["estimated_seconds"], 4)
        self.assertEqual(result["unresolved"], ["C3"])

    def test_parser_and_candidate_validation(self):
        self.assertEqual(parse_partdetail(page("C1", "X"))["stock"], 10)
        self.assertIsNone(parse_partdetail("<html>unknown real page</html>"))
        path = self.root / "candidates.json"
        path.write_text(json.dumps({"Basic": [["C1", "X"]], "Preferred": []}))
        self.assertEqual(load_candidates(path)[0]["candidate_category"], "basic")
        with self.assertRaises(ValueError):
            canonical_url("http://127.0.0.1", "/partdetail/C1")

    def test_parser_reads_only_the_pages_own_part(self):
        parsed = parse_partdetail(page("C12", "S8050", mfr="Maker", stock=7, related=("C1", "C120")), "C12")
        self.assertEqual(parsed, {"c": "C12", "mpn": "S8050", "mfr": "Maker", "package": "0603",
                                  "category": "basic", "stock": 7,
                                  "datasheet": "https://example.invalid/datasheet.pdf"})
        self.assertIsNone(parse_partdetail(page("C12", "S8050"), "C1"))
        # The page shows a Preferred part as Extended; only the candidate list knows.
        extended = parse_partdetail(page("C3", "Y", category="Extended"))
        self.assertEqual(extended["category"], "extended")
        self.assertEqual(with_preferred(extended, {"C3"})["category"], "preferred")
        self.assertEqual(with_preferred(parsed, {"C12"})["category"], "basic")

    def test_parser_fails_closed(self):
        # One flight chunk, so the edits below see whole escaped objects.
        good = page("C5", "Z", stock=3, split=10**6)
        self.assertEqual(parse_partdetail(good)["stock"], 3)
        self.assertIsNone(parse_partdetail(page("C5", "Z", category="mystery")))
        # The JSON-LD mpn disagrees with the flight payload.
        self.assertIsNone(parse_partdetail(good.replace('"mpn": "Z"', '"mpn": "W"')))
        # Two objects for the part disagree on the stock.
        self.assertIsNone(parse_partdetail(good.replace(
            r'\"componentLibraryType\":\"base\"',
            r'\"componentLibraryType\":\"base\",\"overseasStockCount\":4')))
        # The part has no stock; the related part's stock must not stand in.
        self.assertIsNone(parse_partdetail(good.replace(
            r'\"overseasStockCount\":3', r'\"otherCount\":3')))
        self.assertIsNone(parse_partdetail(good.replace("self.__next_f", "self.other")))

    def test_isolated_help_and_repository_cache_rejection(self):
        help_run = subprocess.run([sys.executable, "-I", str(TOOLS / "jlc_catalog.py"), "--help"],
                                  capture_output=True, text=True)
        self.assertEqual(help_run.returncode, 0, help_run.stderr)
        rejected = subprocess.run([
            sys.executable, "-I", str(TOOLS / "jlc_catalog.py"),
            "--allowed-host", "127.0.0.1", "--cache-dir", str(TOOLS.parent / ".cache"),
            "--max-requests", "0", "--dry-run",
        ], capture_output=True, text=True)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn("outside repository", rejected.stderr)

    def test_robots_error_stops_and_separates_not_fetched(self):
        candidates, sitemap = self.fixture()
        with LocalServer({"/robots.txt": (503, "no")}) as server:
            client = make_client(self.root / "cache")
            result = run_catalog(candidates, sitemap, origin=server.url(), client=client,
                                 output_dir=self.root / "snapshot")
            self.assertTrue(result["run_stopped"])
            self.assertIn("RobotsError", result["run_stopped_reason"])
            self.assertEqual(result["not_fetched"], ["C1", "C2"])
            self.assertNotIn("/partdetail/Second-MPN/C2/C2", server.requests)
            self.assertEqual(json.loads((self.root / "snapshot/not_fetched.json").read_text()),
                             ["C1", "C2"])


if __name__ == "__main__":
    unittest.main()
