#!/usr/bin/env python3
"""Confirm W2 LCSC mappings for every primary and alternate official AVL row.

The offline index combines normalized MPNs from sitemap slugs and jlcparts.
Candidates are confirmed through the shared HTTP client.  AVL extraction uses a
small Python lexer for ``primary``/``alt`` blocks rather than requiring a built
``target/debug/cohdl``; it reads strings and balanced blocks, not regex-only rows.
Partdetail pages are parsed by ``jlc_catalog.parse_partdetail``.
"""

from __future__ import annotations

import argparse
from dataclasses import dataclass
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path
import random
import re
import sys
import time
import urllib.parse

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from jlc_catalog import (
    DEFAULT_CANDIDATES,
    OfflineClient,
    canonical_url,
    load_candidates,
    parse_partdetail,
    with_preferred,
    validate_cache_dir,
    write_json,
)
from sourcing.http import FetchClient, RequestLimitError, RobotsError, RunStoppedError
from sourcing.sitemap import SitemapResult, entries_from_mapping, filter_partdetail_urls, load_sitemaps


ROOT = Path(__file__).resolve().parents[1]
CATEGORY = {"basic": 0, "preferred": 1, "extended": 2}


@dataclass(frozen=True)
class AVLRow:
    source: str
    kind: str
    mfr: str
    mpn: str


@dataclass(frozen=True)
class Candidate:
    c: str
    path: str
    sources: tuple[str, ...]


def normalize_mpn(value: str) -> str:
    return "".join(char for char in value.upper() if char.isascii() and char.isalnum())


def confirmation_mpn(value: str) -> str:
    """W2 confirmation ignores case and whitespace, never punctuation."""
    return "".join(value.split()).upper()


def load_aliases(path: str | Path) -> dict[str, str]:
    raw = json.loads(Path(path).read_text(encoding="utf-8"))
    if raw.get("schema_version") != 1 or not isinstance(raw.get("aliases"), dict):
        raise ValueError("mfr_aliases must have schema_version 1 and an aliases object")
    aliases = {}
    for supplier, official in raw["aliases"].items():
        if not isinstance(supplier, str) or not isinstance(official, str) or not supplier or not official:
            raise ValueError("aliases must map non-empty supplier strings to official strings")
        key = " ".join(supplier.casefold().split())
        if key in aliases and aliases[key] != official:
            raise ValueError(f"conflicting manufacturer alias: {supplier}")
        aliases[key] = official
    return aliases


def manufacturer_match(official: str, supplier: str, aliases: dict[str, str]) -> str | None:
    if official == supplier:
        return "exact"
    if aliases.get(" ".join(supplier.casefold().split())) == official:
        return "alias"
    return None


def extract_avl_rows(lib_dir: str | Path) -> list[AVLRow]:
    rows = []
    for path in sorted(Path(lib_dir).rglob("*.cohdl")):
        text = _strip_comments(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\b(primary|alt)\s*\{", text):
            block, _ = _balanced_block(text, match.end() - 1)
            fields = dict(_string_fields(block))
            if "mfr" not in fields or "mpn" not in fields:
                raise ValueError(f"AVL row lacks mfr/mpn: {path}:{text.count(chr(10), 0, match.start()) + 1}")
            rows.append(AVLRow(str(path.relative_to(lib_dir)), match.group(1),
                               fields["mfr"], fields["mpn"]))
    return rows


def build_index(sitemap: SitemapResult, candidates: list[dict[str, str]]) -> dict[str, list[Candidate]]:
    grouped: dict[str, dict[str, Candidate]] = {}
    for entry in sitemap.entries.values():
        # Live slugs (sitemap of 2026-10-30) are "<brand or LCSC id>-<MPN>",
        # with every other "-" and punctuation in the MPN written as "_", so
        # the first "-" separates the prefix.  The whole slug stays a key for
        # the few without one; the partdetail page remains authoritative.
        keys = {normalize_mpn(entry.slug)}
        if "-" in entry.slug:
            keys.add(normalize_mpn(entry.slug.split("-", 1)[1]))
        for key in keys:
            if key:
                grouped.setdefault(key, {})[entry.pn] = Candidate(entry.pn, entry.path, ("sitemap",))
    for row in candidates:
        entry = sitemap.entries.get(row["c"])
        if entry is None:
            continue
        key = normalize_mpn(row["mpn"])
        old = grouped.setdefault(key, {}).get(row["c"])
        sources = tuple(sorted(set((old.sources if old else ()) + ("jlcparts",))))
        grouped[key][row["c"]] = Candidate(row["c"], entry.path, sources)
    return {key: [rows[key] for key in sorted(rows, key=lambda pn: int(pn[1:]))]
            for key, rows in grouped.items()}


def choose_candidate(confirmed: list[dict]) -> tuple[dict | None, list[dict]]:
    if not confirmed:
        return None, []
    ordered = sorted(confirmed, key=lambda row: (
        CATEGORY[row["category"]], row["stock"] <= 0, -row["stock"], int(row["c"][1:])))
    best = ordered[0]
    tied = [row for row in ordered[1:]
            if (CATEGORY[row["category"]], row["stock"] <= 0, -row["stock"]) ==
               (CATEGORY[best["category"]], best["stock"] <= 0, -best["stock"])]
    return (None if tied else best), tied


def run_backfill(
    avl_rows: list[AVLRow],
    index: dict[str, list[Candidate]],
    *,
    origin: str,
    client: FetchClient,
    output_dir: Path,
    aliases: dict[str, str],
    snapshot_date: str,
    dry_run: bool = False,
    source_metadata: dict | None = None,
    preferred: frozenset[str] = frozenset(),
) -> dict:
    unique = sorted({(row.mfr, row.mpn) for row in avl_rows if row.mpn and normalize_mpn(row.mpn)})
    urls = sorted({canonical_url(origin, candidate.path)
                   for mfr, mpn in unique for candidate in index.get(normalize_mpn(mpn), [])})
    if dry_run:
        planned_requests = (sum(not client.is_cached(url) for url in urls)
                            if client is not None else len(urls))
        return {"avl_entries": len(avl_rows), "unique_identities": len(unique),
                "planned_partdetail_requests": planned_requests,
                "estimated_seconds": planned_requests * 2}

    output_dir.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    confirmed: dict[str, dict] = {}
    fetched: set[str] = set()
    failures = []
    exhausted = False
    run_stopped_reason = None
    for url in urls:
        try:
            response = client.get(url)
        except RequestLimitError:
            exhausted = True
            break
        except (RunStoppedError, RobotsError) as error:
            run_stopped_reason = f"{type(error).__name__}: {error}"
            break
        except Exception as error:
            failures.append({"url": url, "error": type(error).__name__})
            fetched.add(url)
            continue
        fetched.add(url)
        try:
            parsed = parse_partdetail(response.body.decode("utf-8"),
                                      urllib.parse.urlsplit(url).path.rstrip("/").rsplit("/", 1)[-1])
        except (UnicodeDecodeError, ValueError):
            parsed = None
        if parsed is not None:
            parsed = with_preferred(parsed, preferred)
            parsed["partdetail"] = urllib.parse.urlsplit(url).path
            parsed["fetched_date"] = response.snapshot_date
            confirmed[url] = parsed

    confirmed_rows = sorted(confirmed.values(), key=lambda row: int(row["c"][1:]))
    confirmed_hash = write_json(output_dir / "confirmed.json", confirmed_rows)

    rows, review, proposals = [], [], {}
    for mfr, mpn in unique:
        candidates = index.get(normalize_mpn(mpn), [])
        matches, other_mfr = [], []
        candidate_urls = [canonical_url(origin, candidate.path) for candidate in candidates]
        has_not_fetched = any(url not in fetched for url in candidate_urls)
        for candidate in candidates:
            page = confirmed.get(canonical_url(origin, candidate.path))
            if page is None or confirmation_mpn(page["mpn"]) != confirmation_mpn(mpn):
                continue
            rule = manufacturer_match(mfr, page["mfr"], aliases)
            if rule is not None:
                matches.append({**page, "partdetail": candidate.path, "rule": rule})
            else:
                other_mfr.append({**page, "partdetail": candidate.path})
        best, tied = choose_candidate(matches)
        identity = {"mfr": mfr, "mpn": mpn}
        if best is None:
            if tied:
                reason = "tied_after_priority"
            elif not candidates:
                reason = "no_candidate"
            elif has_not_fetched:
                reason = "not_fetched"
            elif other_mfr:
                # The MPN is confirmed but the manufacturer name differs: an
                # alias for a human to review, never applied automatically.
                reason = "manufacturer_not_matched"
                for row in other_mfr:
                    proposal = proposals.setdefault((row["mfr"], mfr), {
                        "supplier": row["mfr"], "official": mfr, "identities": 0, "examples": []})
                    proposal["identities"] += 1
                    if len(proposal["examples"]) < 3:
                        proposal["examples"].append({"mpn": mpn, "c": row["c"]})
            else:
                reason = "no_confirmed_match"
            review.append({**identity, "reason": reason,
                           "candidates": [{"c": row["c"], "category": row["category"],
                                           "stock": row["stock"], "partdetail": row["partdetail"],
                                           **({"mfr": row["mfr"]} if reason == "manufacturer_not_matched" else {})}
                                          for row in (matches or other_mfr)]})
            continue
        rows.append({"mfr": mfr, "mpn": mpn, "pn": best["c"], "rule": best["rule"],
                     "category": best["category"], "snapshot_sha256": "", "date": snapshot_date})

    rows.sort(key=lambda row: (row["mfr"], row["mpn"], row["pn"]))
    for row in rows:
        row["snapshot_sha256"] = confirmed_hash
    review.sort(key=lambda row: (row["mfr"], row["mpn"]))
    map_hash = write_json(output_dir / "lcsc_map.json",
                          {"schema_version": 1, "supplier": "lcsc", "rows": rows})
    review_hash = write_json(output_dir / "review.json",
                             {"schema_version": 1, "rows": review, "fetch_failures": failures})
    proposals_hash = write_json(output_dir / "alias_proposals.json", {
        "schema_version": 1,
        "note": "Proposals only; an alias takes effect after review in mfr_aliases.json.",
        "rows": sorted(proposals.values(), key=lambda row: (-row["identities"], row["supplier"]))})
    requests_hash = write_json(output_dir / "requests.json", [
        {"url": record.url, "user_agent": record.user_agent, "snapshot_date": record.snapshot_date}
        for record in client.request_log])
    result = {
        "schema_version": 1, "date": snapshot_date, "snapshot_sha256": confirmed_hash,
        "sources": source_metadata or {"origin": origin},
        "counts": {"avl_entries": len(avl_rows), "unique_identities": len(unique),
                   "mapped_identities": len(rows), "review_identities": len(review),
                   "partdetail_urls": len(urls), "confirmed_pages": len(confirmed),
                   "requests": client.request_count},
        "request_count": client.request_count, "elapsed_seconds": round(time.monotonic() - started, 6),
        "budget_exhausted": exhausted,
        "run_stopped": run_stopped_reason is not None,
        "run_stopped_reason": run_stopped_reason,
        "not_fetched": [row for row in review if row["reason"] == "not_fetched"],
        "match_rate": {"method": "mapped_identities / unique_identities; unique key is exact (mfr, mpn) after selecting confirmed candidates",
                       "numerator": len(rows), "denominator": len(unique),
                       "value": len(rows) / len(unique) if unique else 0.0},
        "sha256": {"confirmed.json": confirmed_hash, "lcsc_map.json": map_hash,
                   "review.json": review_hash, "requests.json": requests_hash,
                   "alias_proposals.json": proposals_hash},
        "parser_status": "real_sample_2026-10-30",
    }
    write_json(output_dir / "provenance.json", result)
    return result


def reproducible_sample(rows: list[AVLRow], size: int, seed: int) -> list[dict]:
    identities = sorted({(row.mfr, row.mpn) for row in rows if row.mpn})
    if size < 0 or size > len(identities):
        raise ValueError("sample size must be between zero and the identity count")
    selected = sorted(random.Random(seed).sample(identities, size))
    return [{"mfr": mfr, "mpn": mpn} for mfr, mpn in selected]


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lib-dir", type=Path, default=ROOT / "lib")
    parser.add_argument("--candidates", type=Path, default=DEFAULT_CANDIDATES)
    parser.add_argument("--aliases", type=Path, default=ROOT / "tools/sourcing_data/mfr_aliases.json")
    parser.add_argument("--sitemap-index", default="https://jlcpcb.com/sitemap-index.xml")
    parser.add_argument("--sitemap", action="append")
    parser.add_argument("--sitemap-json", type=Path)
    parser.add_argument("--origin", default="https://jlcpcb.com")
    parser.add_argument("--allowed-host", action="append", required=True)
    parser.add_argument("--cache-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--date", default=datetime.now(timezone.utc).date().isoformat())
    parser.add_argument("--sample", type=int)
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--max-requests", type=int, required=True)
    parser.add_argument("--dry-run", action="store_true")
    return parser


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    if args.max_requests < 0:
        raise SystemExit("--max-requests must be non-negative")
    try:
        validate_cache_dir(args.cache_dir)
    except ValueError as error:
        raise SystemExit(str(error)) from error
    avl_rows = extract_avl_rows(args.lib_dir)
    if args.sample is not None:
        sample = reproducible_sample(avl_rows, args.sample, args.seed)
        selected = {(row["mfr"], row["mpn"]) for row in sample}
        avl_rows = [row for row in avl_rows if (row.mfr, row.mpn) in selected]
        if not args.dry_run:
            args.output_dir.mkdir(parents=True, exist_ok=True)
            write_json(args.output_dir / "sample.json",
                       {"schema_version": 1, "seed": args.seed, "requested_size": args.sample,
                        "population": len({(row.mfr, row.mpn) for row in extract_avl_rows(args.lib_dir) if row.mpn}),
                        "method": "random.Random(seed).sample over sorted unique nonempty (mfr, mpn), then sorted",
                        "rows": sample})
    candidates = load_candidates(args.candidates)
    if args.dry_run and args.sitemap_json:
        entries = entries_from_mapping(json.loads(args.sitemap_json.read_text(encoding="utf-8")))
        sitemap = SitemapResult(entries, {}, filter_partdetail_urls(
            (canonical_url(args.origin, entry.path) for entry in entries.values()),
            allowed_hosts=args.allowed_host))
        result = run_backfill(avl_rows, build_index(sitemap, candidates), origin=args.origin,
                              client=None, output_dir=args.output_dir, aliases={},
                              snapshot_date=args.date, dry_run=True)
        print(json.dumps(result, indent=2, sort_keys=True))
        return 0
    client = FetchClient(args.cache_dir, allowed_hosts=args.allowed_host, max_requests=args.max_requests)
    if args.sitemap_json:
        entries = entries_from_mapping(json.loads(args.sitemap_json.read_text(encoding="utf-8")))
        sitemap = SitemapResult(entries, {}, filter_partdetail_urls(
            (canonical_url(args.origin, entry.path) for entry in entries.values()),
            allowed_hosts=args.allowed_host))
    elif args.dry_run:
        sitemap, sitemap_meta = load_sitemaps(OfflineClient(client), args.sitemap_index,
                                              sitemap_urls=args.sitemap,
                                              allowed_hosts=args.allowed_host)
    else:
        sitemap, sitemap_meta = load_sitemaps(client, args.sitemap_index, sitemap_urls=args.sitemap,
                                              allowed_hosts=args.allowed_host)
    source_metadata = {
        "origin": args.origin, "lib_dir": str(args.lib_dir),
        "candidates": str(args.candidates),
        "candidates_sha256": hashlib.sha256(args.candidates.read_bytes()).hexdigest(),
        "aliases": str(args.aliases),
        "aliases_sha256": hashlib.sha256(args.aliases.read_bytes()).hexdigest(),
    }
    if args.sitemap_json:
        source_metadata["sitemap_json"] = str(args.sitemap_json)
        source_metadata["sitemap_json_sha256"] = hashlib.sha256(args.sitemap_json.read_bytes()).hexdigest()
    result = run_backfill(avl_rows, build_index(sitemap, candidates), origin=args.origin,
                          client=client, output_dir=args.output_dir,
                          aliases=load_aliases(args.aliases), snapshot_date=args.date,
                          dry_run=args.dry_run, source_metadata=source_metadata,
                          preferred=frozenset(row["c"] for row in candidates
                                              if row["candidate_category"] == "preferred"))
    if args.dry_run:
        # A dry run reads sitemaps only from the cache, so they cost no request.
        sitemap_requests = 0
        result["planned_total_requests_including_sitemap"] = sitemap_requests + result["planned_partdetail_requests"]
        result["estimated_seconds"] = result["planned_total_requests_including_sitemap"] * 2
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


def _strip_comments(text: str) -> str:
    out = list(text)
    i, string, escaped = 0, False, False
    while i < len(text):
        if string:
            if escaped:
                escaped = False
            elif text[i] == "\\":
                escaped = True
            elif text[i] == '"':
                string = False
            i += 1
        elif text.startswith("//", i):
            end = text.find("\n", i)
            end = len(text) if end < 0 else end
            out[i:end] = " " * (end - i)
            i = end
        elif text.startswith("/*", i):
            end = text.find("*/", i + 2)
            if end < 0:
                raise ValueError("unterminated block comment")
            end += 2
            out[i:end] = ["\n" if char == "\n" else " " for char in text[i:end]]
            i = end
        else:
            string = text[i] == '"'
            i += 1
    return "".join(out)


def _balanced_block(text: str, opening: int) -> tuple[str, int]:
    depth, i, string, escaped = 0, opening, False, False
    while i < len(text):
        char = text[i]
        if string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                string = False
        elif char == '"':
            string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[opening + 1:i], i + 1
        i += 1
    raise ValueError("unterminated AVL block")


def _string_fields(block: str):
    pattern = re.compile(r"\b([A-Za-z_][A-Za-z0-9_]*)\s*:\s*\"((?:\\.|[^\"\\])*)\"")
    for match in pattern.finditer(block):
        try:
            yield match.group(1), json.loads('"' + match.group(2) + '"')
        except json.JSONDecodeError as error:
            raise ValueError(f"invalid AVL string: {match.group(0)}") from error


if __name__ == "__main__":
    raise SystemExit(main())
