"""Read JLCPCB sitemaps and retain only canonical partdetail paths.

The sitemap itself is only a discovery source.  A retained URL maps one C-number
to one full ``partdetail/<slug>/<C>/<C>`` path; identity is confirmed separately.
Ten-digit ``C99...`` identifiers are counted and returned separately, never put
in the main set.  All network reads are performed by :class:`sourcing.http.FetchClient`.
"""

from __future__ import annotations

from collections.abc import Iterable, Mapping
from dataclasses import asdict, dataclass
import gzip
from pathlib import Path
import re
from typing import Protocol
import urllib.parse
import xml.etree.ElementTree as ET


_C = re.compile(r"C[0-9]+")
_PATH = re.compile(r"^/partdetail/([^/]*)/(C[0-9]+)/(C[0-9]+)/?$")


class Reader(Protocol):
    def get(self, url: str): ...


@dataclass(frozen=True)
class SitemapEntry:
    pn: str
    path: str
    slug: str


@dataclass(frozen=True)
class FilterReport:
    total: int = 0
    non_detail: int = 0
    placeholder: int = 0
    ten_digit: int = 0
    duplicate: int = 0
    retained: int = 0
    conflicts: int = 0

    def as_dict(self) -> dict[str, int]:
        return asdict(self)


@dataclass(frozen=True)
class SitemapResult:
    entries: dict[str, SitemapEntry]
    ten_digit_entries: dict[str, SitemapEntry]
    report: FilterReport


@dataclass(frozen=True)
class _Counts:
    total: int = 0
    non_detail: int = 0
    placeholder: int = 0
    ten_digit: int = 0
    duplicate: int = 0
    conflicts: int = 0


def parse_sitemap_xml(data: bytes | str) -> tuple[str, list[str]]:
    """Return ``(root_kind, locations)`` from a standard sitemap document.

    Compressed input is accepted as gzip bytes.  Namespace prefixes do not
    matter; sitemapindex/urlset structure and their direct loc children do.
    """
    if isinstance(data, str):
        data = data.encode("utf-8")
    if data.startswith(b"\x1f\x8b"):
        data = gzip.decompress(data)
    root = ET.fromstring(data)
    kind = _local(root.tag)
    if kind not in ("sitemapindex", "urlset"):
        raise ValueError(f"unsupported sitemap root: {kind}")
    child = "sitemap" if kind == "sitemapindex" else "url"
    locations = []
    for node in root:
        if _local(node.tag) != child:
            continue
        loc = next((item for item in node if _local(item.tag) == "loc"), None)
        if loc is None or loc.text is None or not loc.text.strip():
            raise ValueError(f"sitemap {child} has no non-empty loc")
        locations.append(loc.text.strip())
    return kind, locations


def filter_partdetail_urls(urls: Iterable[str]) -> SitemapResult:
    """Apply W1 path, identifier and duplicate rules to absolute or path URLs."""
    entries: dict[str, SitemapEntry] = {}
    ten_digit: dict[str, SitemapEntry] = {}
    counts = {"total": 0, "non_detail": 0, "placeholder": 0, "ten_digit": 0,
              "duplicate": 0, "conflicts": 0}
    for url in urls:
        counts["total"] += 1
        parsed = urllib.parse.urlsplit(url)
        match = _PATH.fullmatch(parsed.path)
        if match is None or parsed.query or parsed.fragment:
            counts["non_detail"] += 1
            continue
        slug, first, second = match.groups()
        if first != second or not _C.fullmatch(first):
            counts["non_detail"] += 1
            continue
        if int(first[1:]) == 0:
            counts["placeholder"] += 1
            continue
        entry = SitemapEntry(first, parsed.path, urllib.parse.unquote(slug))
        target = ten_digit if len(first) == 11 and first.startswith("C99") else entries
        if len(first) == 11 and first.startswith("C99"):
            counts["ten_digit"] += 1
        previous = target.get(first)
        if previous is not None:
            counts["duplicate"] += 1
            if previous.path != entry.path:
                counts["conflicts"] += 1
            continue
        target[first] = entry
    report = FilterReport(**counts, retained=len(entries))
    return SitemapResult(entries, ten_digit, report)


def load_sitemaps(
    client: Reader,
    index_url: str,
    *,
    sitemap_urls: Iterable[str] | None = None,
) -> tuple[SitemapResult, dict]:
    """Fetch an index and its parts sitemaps through the shared HTTP client.

    ``sitemap_urls`` is useful for a bounded offline run or test.  Otherwise
    every child declared by the index is read.  The returned metadata records
    source URLs, per-child filtering reports and aggregate counts, not content.
    """
    kind, children = parse_sitemap_xml(client.get(index_url).body)
    if kind != "sitemapindex":
        raise ValueError("sitemap index must have a sitemapindex root")
    selected = list(children if sitemap_urls is None else sitemap_urls)
    unknown = set(selected) - set(children)
    if unknown:
        raise ValueError(f"parts sitemaps absent from index: {sorted(unknown)}")
    reports = []
    all_urls = []
    for url in selected:
        child_kind, locations = parse_sitemap_xml(client.get(url).body)
        if child_kind != "urlset":
            raise ValueError(f"parts sitemap must have a urlset root: {url}")
        all_urls.extend(locations)
        reports.append({"url": url, **filter_partdetail_urls(locations).report.as_dict()})
    result = filter_partdetail_urls(all_urls)
    metadata = {
        "index_url": index_url,
        "sitemaps": reports,
        "counts": result.report.as_dict(),
    }
    return result, metadata


def entries_from_mapping(rows: Mapping[str, Mapping[str, str] | str]) -> dict[str, SitemapEntry]:
    """Validate and load a previously saved factual sitemap result."""
    result = {}
    for pn, row in rows.items():
        path = row if isinstance(row, str) else row.get("path", "")
        parsed = urllib.parse.urlsplit(path)
        match = _PATH.fullmatch(parsed.path)
        if not match or match.group(2) != pn or match.group(3) != pn or int(pn[1:]) == 0:
            raise ValueError(f"invalid canonical partdetail path for {pn}: {path}")
        result[pn] = SitemapEntry(pn, parsed.path, urllib.parse.unquote(match.group(1)))
    return result


def _local(tag: str) -> str:
    return tag.rsplit("}", 1)[-1]
