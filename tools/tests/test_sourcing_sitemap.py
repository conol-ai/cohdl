"""Offline tests for canonical JLCPCB sitemap discovery."""

import gzip
from pathlib import Path
import socket
import sys
import unittest
from unittest import mock

TOOLS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(TOOLS))

from sourcing.sitemap import filter_partdetail_urls, parse_sitemap_xml  # noqa: E402


class SitemapTests(unittest.TestCase):
    def setUp(self):
        def refuse(*args):
            raise AssertionError("sitemap unit tests must not use the network")
        self.guard = mock.patch.object(socket.socket, "connect", new=refuse)
        self.guard.start()
        self.addCleanup(self.guard.stop)

    def test_gzip_index_and_urlset(self):
        index = b'<?xml version="1.0"?><sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"><sitemap><loc>https://jlcpcb.com/parts.xml.gz</loc></sitemap></sitemapindex>'
        kind, urls = parse_sitemap_xml(gzip.compress(index))
        self.assertEqual(kind, "sitemapindex")
        self.assertEqual(urls, ["https://jlcpcb.com/parts.xml.gz"])
        kind, urls = parse_sitemap_xml('<urlset><url><loc>https://jlcpcb.com/partdetail/X/C1/C1</loc></url></urlset>')
        self.assertEqual((kind, urls[0]), ("urlset", "https://jlcpcb.com/partdetail/X/C1/C1"))

    def test_strict_filter_and_counts(self):
        urls = [
            "https://jlcpcb.com/partdetail/Good-MPN/C123/C123",
            "https://jlcpcb.com/partdetail/Duplicate/C123/C123",
            "https://jlcpcb.com/partdetail/Placeholder/C0/C0",
            "https://jlcpcb.com/partdetail/Unknown/C9900000001/C9900000001",
            "https://jlcpcb.com/category/resistors",
            "https://jlcpcb.com/partdetail/C456",
            "https://jlcpcb.com/partdetail/Mismatch/C456/C457",
            "https://jlcpcb.com/partdetail/Extra/C456/C456/more",
            "https://jlcpcb.com/partdetail/Zero/C000/C000",
            "https://jlcpcb.com/partdetail/Good/C789/C789?x=1",
            "/partdetail/Path-Only/C456/C456",
        ]
        result = filter_partdetail_urls(urls)
        self.assertEqual(set(result.entries), {"C123", "C456"})
        self.assertEqual(result.entries["C123"].path, "/partdetail/Good-MPN/C123/C123")
        self.assertEqual(set(result.ten_digit_entries), {"C9900000001"})
        self.assertEqual(result.report.as_dict(), {
            "total": 11, "non_detail": 5, "placeholder": 2, "ten_digit": 1,
            "duplicate": 1, "retained": 2, "conflicts": 1,
        })

    def test_empty_slug_is_allowed_but_short_and_encoded_paths_are_not(self):
        result = filter_partdetail_urls([
            "https://jlcpcb.com//partdetail/X/C1/C1",
            "https://jlcpcb.com/partdetail//C1/C1",
            "https://jlcpcb.com/partdetail/%58/C2/C2",
        ])
        self.assertEqual(set(result.entries), {"C1", "C2"})
        self.assertEqual(result.report.non_detail, 1)


if __name__ == "__main__":
    unittest.main()
