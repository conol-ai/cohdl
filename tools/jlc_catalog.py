#!/usr/bin/env python3
"""Build a dated W1 catalog from jlcparts candidates and canonical sitemap paths.

``parse_partdetail`` reads the page's JSON-LD and Next.js flight payload, with
rules taken from the approved real sample of 2026-10-30; tests use small
self-written pages of the same shape.  No raw HTML is written to catalog snapshots.  Every network read goes through
``sourcing.http.FetchClient``; resume uses that client's URL cache.
"""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import re
import sys
import time

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from sourcing.http import FetchClient, RequestLimitError, RobotsError, RunStoppedError
from sourcing.sitemap import SitemapResult, entries_from_mapping, filter_partdetail_urls, load_sitemaps


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_CANDIDATES = ROOT / "tools/sourcing_data/jlcparts/2026-10-27/basic_preferred.json"
DEFAULT_OUTPUT = ROOT / "tools/sourcing_data/jlc_catalog"
FIELDS = ("c", "mpn", "mfr", "package", "category", "stock", "datasheet", "partdetail")
_C = re.compile(r"C[1-9][0-9]*")


_LD_JSON = re.compile(r'<script type="application/ld\+json">(.*?)</script>', re.DOTALL)
_FLIGHT = re.compile(r'self\.__next_f\.push\(\[1,("(?:[^"\\]|\\.)*")\]\)')
# The flight payload writes an absent value as this string instead of null.
_UNDEFINED = "$undefined"


def parse_partdetail(html: str, expected_c: str | None = None) -> dict | None:
    """Extract factual fields from a partdetail page, or None when unsure.

    Rules from the approved sample of 2026-10-30 (C2146, C97521, C1525):

    - The page's own part is the schema.org ``Product`` JSON-LD ``sku``.
    - The fields come from the Next.js flight payload (the concatenated
      ``self.__next_f.push([1, "..."])`` strings).  Several objects there carry
      ``componentCode``; only those equal to the ``sku`` are read, because the
      page also lists related parts with their own codes and stock.
    - ``componentModelEn`` is the MPN and must equal the JSON-LD ``mpn``;
      ``componentBrandEn`` the manufacturer; ``componentSpecificationEn`` the
      package; ``overseasStockCount`` the stock; ``dataManualUrl`` the datasheet.
    - ``componentLibraryType`` ``"base"`` is Basic and ``"expand"`` Extended;
      any other value is unknown.  The page does not mark Preferred parts (two
      jlcparts Preferred parts, C25725 and C32375, read "Extended" with
      ``preferredComponentFlag`` undefined), so ``with_preferred`` adds that
      from the candidate list.

    Any missing or conflicting identity field returns None.
    """
    product = None
    for match in _LD_JSON.finditer(html):
        try:
            data = json.loads(match.group(1))
        except ValueError:
            continue
        if isinstance(data, dict) and data.get("@type") == "Product":
            if product is not None:
                return None
            product = data
    if product is None:
        return None
    c = product.get("sku")
    if not isinstance(c, str) or not _C.fullmatch(c) or (expected_c and c != expected_c):
        return None
    try:
        flight = "".join(json.loads(match.group(1)) for match in _FLIGHT.finditer(html))
    except ValueError:
        return None
    fields: dict = {}
    decoder = json.JSONDecoder()
    for match in re.finditer(r'\{\s*"componentCode"\s*:\s*"' + c + r'"', flight):
        try:
            obj, _ = decoder.raw_decode(flight, match.start())
        except ValueError:
            continue
        for key, value in obj.items():
            if value is None or value == _UNDEFINED or isinstance(value, (dict, list)):
                continue
            if key in fields and fields[key] != value:
                return None
            fields[key] = value
    mpn, mfr = fields.get("componentModelEn"), fields.get("componentBrandEn")
    if not isinstance(mpn, str) or not mpn or mpn != product.get("mpn"):
        return None
    if not isinstance(mfr, str) or not mfr:
        return None
    category = {"base": "basic", "expand": "extended"}.get(fields.get("componentLibraryType"))
    if category is None:
        return None
    stock = fields.get("overseasStockCount")
    if not isinstance(stock, int) or isinstance(stock, bool) or stock < 0:
        return None
    package = fields.get("componentSpecificationEn", "")
    datasheet = fields.get("dataManualUrl", "")
    return {
        "c": c, "mpn": mpn, "mfr": mfr,
        "package": package if isinstance(package, str) else "",
        "category": category, "stock": stock,
        "datasheet": datasheet if isinstance(datasheet, str) else "",
    }


def with_preferred(parsed: dict, preferred: set[str] | frozenset[str]) -> dict:
    """Mark an Extended part as Preferred when the candidate list says so.

    The page decides Basic against Extended; a Basic page stays Basic whatever
    the list says.
    """
    if parsed["category"] == "extended" and parsed["c"] in preferred:
        return {**parsed, "category": "preferred"}
    return parsed


class OfflineClient:
    """Read-only cache view used by dry-run; missing resources fail closed."""

    def __init__(self, client: FetchClient):
        self.client = client

    def get(self, url: str):
        if not self.client.is_cached(url):
            raise ValueError(f"dry-run requires cached sitemap: {url}")
        return self.client.get(url)


def load_candidates(path: str | Path) -> list[dict[str, str]]:
    raw = json.loads(Path(path).read_text(encoding="utf-8"))
    rows = []
    seen = set()
    for category in ("Basic", "Preferred"):
        for item in raw.get(category, []):
            if not isinstance(item, (list, tuple)) or len(item) != 2:
                raise ValueError(f"invalid {category} candidate: {item!r}")
            pn, mpn = item
            if pn in seen:
                raise ValueError(f"duplicate candidate: {pn}")
            if not _C.fullmatch(pn) or not mpn:
                raise ValueError(f"invalid candidate identity: {item!r}")
            seen.add(pn)
            rows.append({"c": pn, "mpn": mpn, "candidate_category": category.lower()})
    return rows


def canonical_url(origin: str, path: str) -> str:
    if not path.startswith("/partdetail/") or path.rstrip("/").count("/") != 4:
        raise ValueError(f"not a full partdetail path: {path}")
    return origin.rstrip("/") + path


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write_json(path: Path, value) -> str:
    payload = (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode("utf-8")
    path.write_bytes(payload)
    return hashlib.sha256(payload).hexdigest()


def run_catalog(
    candidates: list[dict[str, str]],
    sitemap: SitemapResult,
    *,
    origin: str,
    client: FetchClient,
    output_dir: Path,
    dry_run: bool = False,
    snapshot_date: str | None = None,
    source_metadata: dict | None = None,
) -> dict:
    """Resolve, confirm and snapshot candidates; return provenance metadata."""
    unresolved = [row["c"] for row in candidates if row["c"] not in sitemap.entries]
    planned = [row for row in candidates if row["c"] in sitemap.entries]
    urls = [canonical_url(origin, sitemap.entries[row["c"]].path) for row in planned]
    estimated_requests = (sum(not client.is_cached(url) for url in urls)
                          if client is not None else len(urls))
    if dry_run:
        return {"planned_partdetail_requests": estimated_requests,
                "estimated_seconds": estimated_requests * 2,
                "unresolved": unresolved, "urls": urls}

    output_dir.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    rows, parse_unresolved, failures, budget_exhausted = [], [], [], False
    category_changed = []
    run_stopped_reason = None
    processed = set()
    for candidate, url in zip(planned, urls):
        try:
            response = client.get(url)
        except RequestLimitError:
            budget_exhausted = True
            break
        except (RunStoppedError, RobotsError) as error:
            run_stopped_reason = f"{type(error).__name__}: {error}"
            break
        except Exception as error:  # retain unresolved evidence, continue bounded run
            failures.append({"c": candidate["c"], "path": sitemap.entries[candidate["c"]].path,
                             "error": type(error).__name__})
            processed.add(candidate["c"])
            continue
        processed.add(candidate["c"])
        try:
            html = response.body.decode("utf-8")
        except UnicodeDecodeError:
            parse_unresolved.append(candidate["c"])
            continue
        parsed = parse_partdetail(html, candidate["c"])
        if parsed is None or parsed["c"] != candidate["c"]:
            parse_unresolved.append(candidate["c"])
            continue
        if candidate["candidate_category"] == "preferred":
            parsed = with_preferred(parsed, {candidate["c"]})
        if parsed["category"] != candidate["candidate_category"]:
            category_changed.append({"c": candidate["c"], "candidates": candidate["candidate_category"],
                                     "page": parsed["category"]})
        parsed["partdetail"] = sitemap.entries[candidate["c"]].path
        rows.append({key: parsed[key] for key in FIELDS})
    rows.sort(key=lambda row: int(row["c"][1:]))
    unresolved.extend(parse_unresolved)
    unresolved.extend(item["c"] for item in failures)
    not_fetched = [row["c"] for row in planned if row["c"] not in processed]
    unresolved.extend(not_fetched)
    catalog_hash = write_json(output_dir / "catalog.json", rows)
    unresolved_hash = write_json(output_dir / "unresolved.json", sorted(set(unresolved)))
    request_log = [{"url": record.url, "user_agent": record.user_agent,
                    "snapshot_date": record.snapshot_date} for record in client.request_log]
    log_hash = write_json(output_dir / "requests.json", request_log)
    not_fetched_hash = write_json(output_dir / "not_fetched.json", not_fetched)
    provenance = {
        "schema_version": 1,
        "date": snapshot_date or datetime.now(timezone.utc).date().isoformat(),
        "sources": source_metadata or {"origin": origin, "partdetail_paths": "filtered_jlcpcb_sitemap"},
        "counts": {"candidates": len(candidates), "catalog": len(rows),
                   "unresolved": len(set(unresolved)), "fetch_failures": len(failures),
                   "requests": client.request_count, "partdetail_urls": len(urls)},
        "request_count": client.request_count,
        "elapsed_seconds": round(time.monotonic() - started, 6),
        "budget_exhausted": budget_exhausted,
        "run_stopped": run_stopped_reason is not None,
        "run_stopped_reason": run_stopped_reason,
        "not_fetched": not_fetched,
        "failures": failures,
        "category_changed": category_changed,
        "sha256": {"catalog.json": catalog_hash, "unresolved.json": unresolved_hash,
                   "requests.json": log_hash, "not_fetched.json": not_fetched_hash},
        "sitemap_filter": sitemap.report.as_dict(),
        "parser_status": "real_sample_2026-10-30",
    }
    write_json(output_dir / "provenance.json", provenance)
    return provenance


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--candidates", type=Path, default=DEFAULT_CANDIDATES)
    parser.add_argument("--sitemap-index", default="https://jlcpcb.com/sitemap-index.xml")
    parser.add_argument("--sitemap", action="append", help="child sitemap URL; repeatable")
    parser.add_argument("--sitemap-json", type=Path, help="offline filtered entries mapping")
    parser.add_argument("--origin", default="https://jlcpcb.com")
    parser.add_argument("--allowed-host", action="append", required=True)
    parser.add_argument("--cache-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path)
    parser.add_argument("--date", default=datetime.now(timezone.utc).date().isoformat())
    parser.add_argument("--max-requests", type=int, required=True)
    parser.add_argument("--dry-run", action="store_true")
    return parser


def validate_cache_dir(path: Path) -> None:
    resolved = path.expanduser().resolve()
    if resolved == ROOT or ROOT in resolved.parents:
        raise ValueError(f"--cache-dir must be outside repository root: {resolved}")


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if args.max_requests < 0:
        raise SystemExit("--max-requests must be non-negative")
    try:
        validate_cache_dir(args.cache_dir)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    candidates = load_candidates(args.candidates)
    if args.dry_run and args.sitemap_json:
        entries = entries_from_mapping(json.loads(args.sitemap_json.read_text(encoding="utf-8")))
        sitemap = SitemapResult(entries, {}, filter_partdetail_urls(
            (canonical_url(args.origin, entry.path) for entry in entries.values()),
            allowed_hosts=args.allowed_host))
        result = run_catalog(candidates, sitemap, origin=args.origin, client=None,
                             output_dir=args.output_dir or DEFAULT_OUTPUT / args.date, dry_run=True)
        result["planned_total_requests_including_sitemap"] = result["planned_partdetail_requests"]
        result.pop("urls")
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    client = FetchClient(args.cache_dir, allowed_hosts=args.allowed_host,
                         max_requests=args.max_requests)
    if args.sitemap_json:
        entries = entries_from_mapping(json.loads(args.sitemap_json.read_text(encoding="utf-8")))
        sitemap = SitemapResult(entries, {}, filter_partdetail_urls(
            (canonical_url(args.origin, entry.path) for entry in entries.values()),
            allowed_hosts=args.allowed_host))
        sitemap_meta = {"sitemap_json": str(args.sitemap_json)}
    elif args.dry_run:
        sitemap, sitemap_meta = load_sitemaps(OfflineClient(client), args.sitemap_index,
                                              sitemap_urls=args.sitemap,
                                              allowed_hosts=args.allowed_host)
    else:
        sitemap, sitemap_meta = load_sitemaps(client, args.sitemap_index, sitemap_urls=args.sitemap,
                                              allowed_hosts=args.allowed_host)
    output = args.output_dir or DEFAULT_OUTPUT / args.date
    source_metadata = {
        "origin": args.origin,
        "candidates": str(args.candidates),
        "candidates_sha256": sha256_file(args.candidates),
        "partdetail_paths": "filtered_jlcpcb_sitemap",
    }
    if args.sitemap_json:
        source_metadata["sitemap_json"] = str(args.sitemap_json)
        source_metadata["sitemap_json_sha256"] = sha256_file(args.sitemap_json)
    else:
        source_metadata["sitemap"] = sitemap_meta
    result = run_catalog(candidates, sitemap, origin=args.origin, client=client,
                         output_dir=output, dry_run=args.dry_run, snapshot_date=args.date,
                         source_metadata=source_metadata)
    if args.dry_run:
        # A dry run reads sitemaps only from the cache, so they cost no request.
        sitemap_requests = 0
        result["planned_total_requests_including_sitemap"] = sitemap_requests + result["planned_partdetail_requests"]
        result["estimated_seconds"] = result["planned_total_requests_including_sitemap"] * 2
        result.pop("urls")
        print(json.dumps(result, indent=2, sort_keys=True))
    else:
        print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
