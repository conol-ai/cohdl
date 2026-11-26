#!/usr/bin/env python3
"""Run ``lcsc_backfill.py`` so that it fetches only a list of pages a person approved.

Two steps, with a person's approval between them::

    python3 -I tools/lcsc_backfill_approved.py plan --subset SUBSET_LIB --cache-dir CACHE -o URLS.json
    python3 -I tools/lcsc_backfill_approved.py run --approved URLS.json --sha256 SHA \\
        --subset SUBSET_LIB --cache-dir CACHE --output-dir OUT --date YYYY-MM-DD

``plan`` sends no request. It builds the backfill's offline index from the cached sitemaps,
takes the identities (manufacturer, MPN) of SUBSET_LIB, a library directory holding only the
batch's new parts, and writes every candidate ``partdetail`` page of those MPNs that is not
cached yet. It prints the list's length and SHA-256: that hash is what the person approves.

``run`` refuses to start unless the list still has the approved SHA-256 and every uncached
candidate page of SUBSET_LIB is still on it. A run that stopped part way (two 403/429, an
interruption) therefore resumes with no new approval: the pages it fetched are cached now,
and only the rest of the approved list is left. It runs ``lcsc_backfill.main`` over the whole
``lib/``, as every backfill run has, so the run also re-reads the earlier identities from the
cache. The request cap is the pages left plus two (``robots.txt``), behind a guard: any page
that is neither cached nor on the list is refused with ``PolicyError`` before a request is
sent, and its identity is reported ``fetch_failed``. ``run`` then prints how many
``fetch_failed`` identities are in the batch and how many are outside it (refused, off the
list), for the run's README. Pass ``--batch`` the directory of all the batch's new parts when
SUBSET_LIB is only some of them (those the catalog snapshot lists): the count then also says
how many of the batch were not planned for.

The official map must cite one confirmed snapshot (``tools/w3_groups.py``), so a run has to
re-read every identity the map already holds. An identity an earlier run read that is not in
the library (an order line of a real board, for one) would otherwise keep its old snapshot.
``run`` therefore reads, besides the library, every (manufacturer, MPN) of ``--map`` that the
library lacks: it copies the library's ``.cohdl`` files to a temporary directory and adds one
AVL block per such identity. It prints how many it carried. Every other rule (user agent, 2 s per host, ``robots.txt``, no
``/api``, stop after two 403/429) stays the shared client's (``tools/sourcing/http.py``).
"""
from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import tempfile
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
REPO = TOOLS.parent
sys.path.insert(0, str(TOOLS))

import lcsc_backfill as lb  # noqa: E402
from sourcing import http  # noqa: E402
from sourcing.sitemap import load_sitemaps  # noqa: E402

HOST = "jlcpcb.com"
ORIGIN = "https://jlcpcb.com"
SITEMAP_INDEX = "https://jlcpcb.com/sitemap-index.xml"


def batch_identities(subset: Path) -> set[tuple[str, str]]:
    """The (manufacturer, MPN) identities of SUBSET that the backfill looks up."""
    return {(r.mfr, r.mpn) for r in lb.extract_avl_rows(subset) if r.mpn and lb.normalize_mpn(r.mpn)}


def carried_identities(lib_dir: Path, map_path: Path) -> list[tuple[str, str]]:
    """The map's (manufacturer, MPN) identities that no AVL entry of LIB_DIR names, sorted."""
    held = {(r.mfr, r.mpn) for r in lb.extract_avl_rows(lib_dir)}
    rows = json.loads(map_path.read_text())["rows"]
    return sorted({(row["mfr"], row["mpn"]) for row in rows} - held)


def lib_with_carried(lib_dir: Path, carried: list[tuple[str, str]], tmp: Path) -> Path:
    """A copy of LIB_DIR's sources under TMP, plus one AVL block per carried identity."""
    out = tmp / "lib"
    for path in sorted(lib_dir.rglob("*.cohdl")):
        target = out / path.relative_to(lib_dir)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
    blocks = [f"primary {{ mfr: {json.dumps(mfr, ensure_ascii=False)}, mpn: {json.dumps(mpn, ensure_ascii=False)} }}"
              for mfr, mpn in carried]
    extra = out / "_carried_from_map" / "identities.cohdl"
    extra.parent.mkdir(parents=True)
    extra.write_text("\n".join(blocks) + "\n", encoding="utf-8")
    if sorted({(r.mfr, r.mpn) for r in lb.extract_avl_rows(extra.parent)}) != carried:
        raise SystemExit("a carried identity does not survive being written as an AVL block")
    return out


def uncached_candidates(subset: Path, cache: Path) -> tuple[int, list[str]]:
    """(identities in SUBSET, sorted uncached candidate pages of their MPNs); no request is sent."""
    probe = http.FetchClient(cache, allowed_hosts=[HOST], max_requests=0)
    sitemap, _ = load_sitemaps(lb.OfflineClient(probe), SITEMAP_INDEX, sitemap_urls=None, allowed_hosts=[HOST])
    index = lb.build_index(sitemap, lb.load_candidates(lb.DEFAULT_CANDIDATES))
    unique = sorted(batch_identities(subset))
    urls = {lb.canonical_url(ORIGIN, c.path) for _, mpn in unique for c in index.get(lb.normalize_mpn(mpn), [])}
    return len(unique), sorted(u for u in urls if not probe.is_cached(u))


def list_bytes(urls: list[str]) -> bytes:
    return (json.dumps(urls, indent=0) + "\n").encode()


def guard(client_cls, approved: frozenset[str]):
    """Patch CLIENT_CLS.get to refuse an uncached URL off the approved list; returns an undo function."""
    real_get = client_cls.get

    def guarded_get(self, url):
        if not self.is_cached(url) and url not in approved:
            raise http.PolicyError(f"outside the approved list: {url}")
        return real_get(self, url)

    client_cls.get = guarded_get
    return lambda: setattr(client_cls, "get", real_get)


def check_approved(path: Path, sha256: str) -> list[str]:
    data = path.read_bytes()
    if hashlib.sha256(data).hexdigest() != sha256:
        raise SystemExit(f"{path}: SHA-256 is not the approved {sha256}; get the new list approved first")
    urls = json.loads(data)
    if not isinstance(urls, list) or urls != sorted(set(urls)) or not all(isinstance(u, str) for u in urls):
        raise SystemExit(f"{path}: not a sorted list of distinct URLs")
    return urls


def cmd_plan(args) -> int:
    count, urls = uncached_candidates(args.subset, args.cache_dir)
    data = list_bytes(urls)
    args.output.write_bytes(data)
    print(f"{count} identities, {len(urls)} uncached candidate pages")
    print(f"{args.output}: SHA-256 {hashlib.sha256(data).hexdigest()}")
    return 0


def fetch_failed_split(review: Path, batch: set[tuple[str, str]]) -> tuple[int, int]:
    """(fetch_failed identities in the batch, outside it) in a run's review.json."""
    rows = [row for row in json.loads(review.read_text())["rows"] if row["reason"] == "fetch_failed"]
    inside = sum((row["mfr"], row["mpn"]) in batch for row in rows)
    return inside, len(rows) - inside


def fetch_failed_line(review: Path, planned: set[tuple[str, str]], batch: set[tuple[str, str]] | None) -> str:
    if batch is None:
        inside, outside = fetch_failed_split(review, planned)
        return f"fetch_failed: {inside} in the batch, {outside} outside it (refused: off the approved list)"
    if not planned <= batch:
        raise SystemExit("--subset names an identity that --batch does not hold")
    inside, outside = fetch_failed_split(review, batch)
    among_planned, _ = fetch_failed_split(review, planned)
    return (f"fetch_failed: {inside} in the batch ({among_planned} of them planned for, "
            f"{inside - among_planned} not planned for), {outside} outside it (refused: off the approved list)")


def cmd_run(args) -> int:
    approved = check_approved(args.approved, args.sha256)
    count, left = uncached_candidates(args.subset, args.cache_dir)
    off_list = sorted(set(left) - set(approved))
    if off_list:
        raise SystemExit(f"{len(off_list)} uncached candidate pages are not on the approved list "
                         f"(first: {off_list[0]}); plan again")
    print(f"{count} identities; {len(left)} of the {len(approved)} approved pages still to fetch")
    carried = carried_identities(args.lib_dir, args.map) if args.map.is_file() else []
    print(f"{len(carried)} identities of the map are not in the library and are read as well"
          + (f" (first: {carried[0][0]} {carried[0][1]})" if carried else ""))
    undo = guard(http.FetchClient, frozenset(left))
    try:
        with tempfile.TemporaryDirectory() as tmp:
            lib_dir = lib_with_carried(args.lib_dir, carried, Path(tmp)) if carried else args.lib_dir
            args.output_dir.mkdir(parents=True, exist_ok=True)
            status = lb.main(["--allowed-host", HOST, "--cache-dir", str(args.cache_dir),
                              "--output-dir", str(args.output_dir), "--date", args.date,
                              "--max-requests", str(len(left) + 2), "--lib-dir", str(lib_dir)])
    finally:
        undo()
    review = args.output_dir / "review.json"
    if review.is_file():
        batch = batch_identities(args.batch) if args.batch else None
        print(fetch_failed_line(review, batch_identities(args.subset), batch))
    return status


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    plan = sub.add_parser("plan", help="list the uncached candidate pages (no request)")
    plan.add_argument("--subset", type=Path, required=True)
    plan.add_argument("--cache-dir", type=Path, required=True)
    plan.add_argument("-o", "--output", type=Path, required=True)
    run = sub.add_parser("run", help="run the backfill, fetching only the approved pages")
    run.add_argument("--approved", type=Path, required=True)
    run.add_argument("--sha256", required=True)
    run.add_argument("--subset", type=Path, required=True)
    run.add_argument("--cache-dir", type=Path, required=True)
    run.add_argument("--output-dir", type=Path, required=True)
    run.add_argument("--date", required=True)
    run.add_argument("--lib-dir", type=Path, default=REPO / "lib")
    run.add_argument("--batch", type=Path, help="all the batch's new parts, when --subset is only some of them")
    run.add_argument("--map", type=Path, default=REPO / "tools/sourcing_data/lcsc_map.json",
                     help="the official map; its identities missing from the library are read as well")
    return p


def main(argv=None) -> int:
    args = build_parser().parse_args(argv)
    return cmd_plan(args) if args.cmd == "plan" else cmd_run(args)


if __name__ == "__main__":
    raise SystemExit(main())
