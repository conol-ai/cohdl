#!/usr/bin/env python3
"""Build, upload and verify the registry's LCSC supplier catalog (RFC-034 §4, plan D2).

``build -o DIR [--cache-dir CACHE | --dates REPORT]`` fetches nothing. It writes DIR/snapshot.json,
DIR/aliases.json and DIR/report.json (and report.md) from the repository's own data:

- the B1 snapshot ``tools/sourcing_data/jlc_catalog/2026-10-30/``, after checking
  its files against its ``provenance.json``;
- every row of ``lcsc_map.json``, read from the confirmed page of the
  ``lcsc_backfill.py`` run that mapped it. Each cited run is re-checked as
  ``promote_lcsc_map.py`` does, and the row's MPN, class and manufacturer (equal,
  or related through the reviewed ``mfr_aliases.json``) must be what that page read.

Rows are merged by part number. Two records for one part number must agree on
MPN, manufacturer, package and class, or the build fails. Each row's
``fetched_at`` is the time its page was fetched: from the run's request log,
or, for a page the run read from its cache, from that cache's metadata
(``--cache-dir``, the fetch cache outside the repository; its URL and body hash are
checked). The report lists those dates, so ``--dates`` with an earlier report
rebuilds the same body without the cache. A row with no such time fails the build. The body is then checked
as the registry's ``parseSnapshot`` checks it, and against its limits: at most
10,000 rows and 4 MiB, since one request replaces the whole catalog. Every
``supplier lcsc`` part number in ``lib/`` must be in it. The report records the
counts, sources, size and the body's SHA-256.

``upload DIR --expect-sha256 HEX`` sends DIR's aliases, then the snapshot, to
the registry's admin endpoints, only if snapshot.json still has the SHA-256
the report records and the person signed off (HEX). The endpoints take an
official account's web session, so the person passes theirs in the
``COHDL_ADMIN_SESSION`` environment variable; it is never printed or written.

``verify DIR`` asks the public ``/suppliers/lcsc/parts`` for every library part
number and compares each answer with the snapshot row. It is read-only.

``--registry`` (default https://registry.cohdl.org, or ``COHDL_REGISTRY``)
applies to ``upload`` and ``verify``. Exit 0 on success, 1 on a failed check.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sys
import unicodedata
import urllib.error
import urllib.parse
import urllib.request

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import confirmation_mpn, load_aliases, manufacturer_match
from promote_lcsc_map import run_rows

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "tools/sourcing_data"
SNAPSHOT = DATA / "jlc_catalog/2026-10-30"
MAP = DATA / "lcsc_map.json"
ALIASES = DATA / "mfr_aliases.json"
RUNS = DATA / "lcsc_backfill"
ORIGIN = "https://jlcpcb.com"

# The supplier row of the library plan (W5).
SUPPLIER = "lcsc"
DISPLAY_NAME = "LCSC"
PRODUCT_URL_TEMPLATE = "https://www.lcsc.com/product-detail/{pn}.html"
CLASSES = ["basic", "preferred", "extended"]
# registry/src/worker/suppliers.ts
MAX_ROWS = 10_000
BODY_LIMIT = 4 * 1024 * 1024
USER_AGENT = "cohdl-upload-supplier-snapshot/1 (+https://github.com/conol-ai/cohdl)"


class SnapshotError(ValueError):
    pass


def _json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise SnapshotError(f"{path}: {error}") from error


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def text_ok(value, max_bytes: int = 256) -> bool:
    """The registry's ``text()``: NFC, trimmed, non-empty, no control or format characters, bounded."""
    return (isinstance(value, str) and value and unicodedata.normalize("NFC", value) == value
            and value.strip() == value and len(value.encode("utf-8")) <= max_bytes
            and not any(unicodedata.category(ch) in ("Cc", "Cf") for ch in value))


def cache_date(cache_dir: Path, url: str) -> str | None:
    """When the fetch cache read URL, if it holds a verified copy (tools/sourcing/http.py)."""
    digest = hashlib.sha256(url.encode("utf-8")).hexdigest()
    try:
        meta = json.loads((cache_dir / f"{digest}.json").read_text(encoding="utf-8"))
        body = (cache_dir / f"{digest}.body").read_bytes()
    except (OSError, ValueError):
        return None
    if meta.get("url") != url or meta.get("sha256") != _sha256(body):
        return None
    return meta.get("snapshot_date")


def snapshot_rows(snapshot: Path, cache_dir: Path | None, known: dict | None = None) -> tuple[list[dict], dict]:
    """The B1 snapshot's pages with their fetched_at, and {part number: date} for those dated from the cache.

    KNOWN is an earlier report's ``fetched_at_from_cache``, used in place of the cache.
    """
    provenance = _json(snapshot / "provenance.json")
    for name, digest in sorted(provenance["sha256"].items()):
        if _sha256((snapshot / name).read_bytes()) != digest:
            raise SnapshotError(f"{snapshot / name} does not match the SHA-256 in provenance.json")
    dates = {}
    for record in _json(snapshot / "requests.json"):
        if "/partdetail/" in record["url"]:
            dates[record["url"]] = record["snapshot_date"]
    rows, from_cache = [], {}
    for page in _json(snapshot / "catalog.json"):
        url = ORIGIN + page["partdetail"]
        fetched = dates.get(url)
        if fetched is None:
            fetched = (known or {}).get(page["c"]) or (cache_dir and cache_date(cache_dir, url))
            if fetched:
                from_cache[page["c"]] = fetched
        if not fetched:
            raise SnapshotError(f"{page['c']}: {snapshot.name}'s request log has no fetch of {url}; "
                                "pass --cache-dir with the run's fetch cache, or --dates with its report")
        rows.append(dict(page, fetched_at=fetched, source=f"jlc_catalog/{snapshot.name}"))
    return rows, from_cache


def map_rows(map_path: Path, runs: Path, aliases_path: Path) -> list[dict]:
    """Each official map row's confirmed page, from the re-checked run that mapped it.

    Runs recomputed from one cache share a confirmed.json; the row's run is the one
    that passes ``promote_lcsc_map.py``'s checks and lists the row itself.
    """
    by_snapshot = {}
    for run in sorted(p for p in runs.iterdir() if (p / "provenance.json").is_file()):
        digest = _json(run / "provenance.json").get("sha256", {}).get("confirmed.json")
        if digest:
            by_snapshot.setdefault(digest, []).append(run)
    aliases = load_aliases(aliases_path)
    checked, rows = {}, []

    def accepted(run):
        if run not in checked:
            try:
                listed = {(r["mfr"], r["mpn"], r["pn"]) for r in run_rows(run, aliases_path)}
            except ValueError:
                listed = None
            pages = {}
            for page in _json(run / "confirmed.json"):
                pages.setdefault(page["c"], []).append(page)
            checked[run] = listed and (listed, pages)
        return checked[run]

    for row in _json(map_path)["rows"]:
        runs_for = [run for run in by_snapshot.get(row["snapshot_sha256"], [])
                    if accepted(run) and (row["mfr"], row["mpn"], row["pn"]) in accepted(run)[0]]
        if not runs_for:
            raise SnapshotError(f"{row['pn']}: no re-checked run under {runs.name}/ with confirmed.json "
                                f"{row['snapshot_sha256'][:12]}… lists it")
        run = runs_for[0]
        # As promote_lcsc_map.py matches a row to its page.
        matches = [page for page in accepted(run)[1].get(row["pn"], [])
                   if confirmation_mpn(page["mpn"]) == confirmation_mpn(row["mpn"])
                   and page["category"] == row["category"] and manufacturer_match(row["mfr"], page["mfr"], aliases)]
        if len(matches) != 1:
            raise SnapshotError(f"{row['pn']}: {len(matches)} confirmed pages in {run.name} match the map row")
        page = matches[0]
        rows.append(dict(page, fetched_at=page["fetched_date"], source=f"lcsc_backfill/{run.name}"))
    return rows


def catalog_row(page: dict) -> dict:
    return {"pn": page["c"], "mpn": page["mpn"], "manufacturer": page["mfr"], "package": page["package"],
            "class": page["category"], "stock": page["stock"],
            "links": [{"label": "JLCPCB", "url": ORIGIN + page["partdetail"]}], "fetched_at": page["fetched_at"]}


def merge(pages: list[dict]) -> list[dict]:
    """One catalog row per part number; disagreeing records are an error."""
    rows = {}
    for page in pages:
        row = catalog_row(page)
        old = rows.get(row["pn"])
        if old is None:
            rows[row["pn"]] = row
            continue
        keys = ("mpn", "manufacturer", "package", "class")
        if any(old[k] != row[k] for k in keys):
            raise SnapshotError(f"{row['pn']}: two different records: "
                                + "; ".join(f"{k} {old[k]!r} / {row[k]!r}" for k in keys if old[k] != row[k]))
        if row["fetched_at"] > old["fetched_at"]:  # keep the later page's stock and date
            rows[row["pn"]] = row
    return sorted(rows.values(), key=lambda row: int(row["pn"][1:]))


def check_rows(rows: list[dict]) -> list[str]:
    """Rows the registry's ``parseSnapshot`` would refuse."""
    problems = []
    stamp = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$")
    for row in rows:
        bad = [k for k in ("pn", "mpn", "manufacturer", "package") if not text_ok(row[k])]
        if row["class"] not in CLASSES:
            bad.append("class")
        if not isinstance(row["stock"], int) or row["stock"] < 0:
            bad.append("stock")
        if not stamp.match(row["fetched_at"]):
            bad.append("fetched_at")
        if any(not text_ok(link["url"], 2048) or not link["url"].startswith("https://") for link in row["links"]):
            bad.append("links")
        if bad:
            problems.append(f"{row['pn']}: invalid {', '.join(bad)}")
    return problems


def library_pns(lib: Path = ROOT / "lib") -> list[str]:
    found = set()
    for path in lib.rglob("*.cohdl"):
        found.update(re.findall(r'\bsupplier\s+lcsc\s+"([^"]+)"', path.read_text(encoding="utf-8")))
    return sorted(found, key=lambda pn: int(pn[1:]))


def encode(value) -> bytes:
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), sort_keys=True).encode("utf-8")


def build(out: Path, cache_dir: Path | None, dates: Path | None = None, snapshot: Path = SNAPSHOT,
          map_path: Path = MAP, runs: Path = RUNS, aliases_path: Path = ALIASES, lib: Path = ROOT / "lib") -> dict:
    b1, from_cache = snapshot_rows(snapshot, cache_dir, dates and _json(dates)["fetched_at_from_cache"])
    mapped = map_rows(map_path, runs, aliases_path)
    rows = merge(b1 + mapped)
    problems = check_rows(rows)
    if problems:
        raise SnapshotError("; ".join(problems))
    body = encode({"display_name": DISPLAY_NAME, "product_url_template": PRODUCT_URL_TEMPLATE,
                   "classes": CLASSES, "parts": rows})
    if len(rows) > MAX_ROWS or len(body) > BODY_LIMIT:
        raise SnapshotError(f"{len(rows)} rows, {len(body)} bytes: over the registry's "
                            f"{MAX_ROWS} rows or {BODY_LIMIT} bytes")
    pns = {row["pn"] for row in rows}
    bound = library_pns(lib)
    missing = [pn for pn in bound if pn not in pns]
    if missing:
        raise SnapshotError("library part numbers missing from the catalog: " + ", ".join(missing))
    load_aliases(aliases_path)  # validates; the registry normalizes the names itself
    aliases = encode(_json(aliases_path)["aliases"])
    by_class = {name: sum(row["class"] == name for row in rows) for name in CLASSES}
    report = {
        "schema_version": 1,
        "supplier": SUPPLIER,
        "display_name": DISPLAY_NAME,
        "product_url_template": PRODUCT_URL_TEMPLATE,
        "classes": CLASSES,
        "rows": len(rows),
        "rows_by_class": by_class,
        "sources": {
            f"jlc_catalog/{snapshot.name}": len(b1),
            "lcsc_map.json": len(mapped),
            "overlap": len(b1) + len(mapped) - len(rows),
        },
        "source_sha256": {name: _sha256(path.read_bytes()) for name, path in (
            (f"jlc_catalog/{snapshot.name}/catalog.json", snapshot / "catalog.json"),
            ("lcsc_map.json", map_path), ("mfr_aliases.json", aliases_path))},
        "fetched_at_from_cache": from_cache,
        "fetched_at_range": [min(r["fetched_at"] for r in rows), max(r["fetched_at"] for r in rows)],
        "library_part_numbers": bound,
        "limits": {"rows": MAX_ROWS, "bytes": BODY_LIMIT},
        "bytes": len(body),
        "snapshot_sha256": _sha256(body),
        "aliases": json.loads(aliases),
        "aliases_sha256": _sha256(aliases),
    }
    out.mkdir(parents=True, exist_ok=True)
    (out / "snapshot.json").write_bytes(body)
    (out / "aliases.json").write_bytes(aliases)
    (out / "report.json").write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (out / "report.md").write_text(render(report), encoding="utf-8")
    return report


def render(report: dict) -> str:
    src = report["sources"]
    lines = [
        f"# {report['display_name']} supplier catalog: pre-upload report", "",
        f"- Rows: {report['rows']} of at most {report['limits']['rows']} "
        f"({', '.join(f'{n} {k}' for k, n in report['rows_by_class'].items())}).",
        f"- Body: {report['bytes']:,} bytes of at most {report['limits']['bytes']:,}; "
        f"SHA-256 `{report['snapshot_sha256']}`.",
        "- Sources: " + "; ".join(f"`{k}` {n}" for k, n in src.items() if k != "overlap")
        + f"; {src['overlap']} part numbers in both, merged.",
        f"- Pages fetched {report['fetched_at_range'][0]} to {report['fetched_at_range'][1]}. "
        + ("Dated from the fetch cache, since the run read them from it: "
           + ", ".join(f"{pn} {date}" for pn, date in report["fetched_at_from_cache"].items()) + "."
           if report["fetched_at_from_cache"] else "Every date is from a request log."),
        f"- Library part numbers, all present: {', '.join(report['library_part_numbers'])}.",
        "- Aliases: " + "; ".join(f"{a} → {c}" for a, c in report["aliases"].items())
        + f" (SHA-256 `{report['aliases_sha256']}`).",
        "", "Source files (SHA-256):", "",
    ] + [f"- `{name}` `{digest}`" for name, digest in report["source_sha256"].items()] + [""]
    return "\n".join(lines)


def request(method: str, url: str, body: bytes | None = None, session: str | None = None, origin: str | None = None):
    headers = {"User-Agent": USER_AGENT, "Accept": "application/json"}
    if body is not None:
        headers["Content-Type"] = "application/json"
    if session is not None:
        headers["Cookie"] = f"__Host-session={session}"
        headers["Origin"] = origin
    req = urllib.request.Request(url, data=body, method=method, headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=60) as response:
            return response.status, json.loads(response.read() or b"null")
    except urllib.error.HTTPError as error:
        try:
            return error.code, json.loads(error.read() or b"null")
        except ValueError:
            return error.code, None


def upload(out: Path, expect: str, registry: str, session: str | None) -> list[str]:
    report = _json(out / "report.json")
    body, aliases = (out / "snapshot.json").read_bytes(), (out / "aliases.json").read_bytes()
    if _sha256(body) != report["snapshot_sha256"] or _sha256(aliases) != report["aliases_sha256"]:
        return [f"{out}'s snapshot.json or aliases.json no longer matches its report.json"]
    if expect != report["snapshot_sha256"]:
        return [f"--expect-sha256 {expect} is not the reported snapshot {report['snapshot_sha256']}"]
    if not session or not re.fullmatch(r"[a-f0-9]{64}", session):
        return ["COHDL_ADMIN_SESSION must hold an official account's 64-hex web session"]
    origin = urllib.parse.urlsplit(registry)._replace(path="", query="", fragment="").geturl()
    for alias, canonical in json.loads(aliases).items():
        status, answer = request("PUT", f"{origin}/api/admin/manufacturer-aliases/{urllib.parse.quote(alias, safe='')}",
                                 encode({"canonical": canonical}), session, origin)
        if status != 200:
            return [f"alias {alias!r}: HTTP {status} {answer}"]
        print(f"alias {alias} -> {canonical}: {answer.get('alias_key')}")
    status, answer = request("PUT", f"{origin}/api/admin/suppliers/{SUPPLIER}/snapshot", body, session, origin)
    if status != 200 or answer.get("rows") != report["rows"]:
        return [f"snapshot: HTTP {status} {answer}"]
    print(f"snapshot: {answer['rows']} rows replaced the {SUPPLIER} catalog")
    return []


def verify(out: Path, registry: str) -> list[str]:
    report = _json(out / "report.json")
    rows = {row["pn"]: row for row in json.loads((out / "snapshot.json").read_bytes())["parts"]}
    pns = report["library_part_numbers"]
    origin = urllib.parse.urlsplit(registry)._replace(path="", query="", fragment="").geturl()
    status, answer = request("GET", f"{origin}/suppliers/{SUPPLIER}/parts?pn={','.join(pns)}")
    if status != 200:
        return [f"/suppliers/{SUPPLIER}/parts: HTTP {status} {answer}"]
    problems = []
    for part in answer["parts"]:
        got, want = part["catalog"], rows[part["pn"]]
        if got == "unknown":
            problems.append(f"{part['pn']}: unknown to the registry")
            continue
        diff = [k for k in ("mpn", "manufacturer", "package", "class", "stock", "links", "fetched_at") if got[k] != want[k]]
        if diff:
            problems.append(f"{part['pn']}: the registry's {', '.join(diff)} differ from the snapshot")
        else:
            print(f"{part['pn']}: {got['mpn']} ({got['manufacturer']}, {got['class']}) known")
    print(f"snapshot date: {answer.get('snapshot_date')}")
    return problems


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    b = sub.add_parser("build")
    b.add_argument("-o", "--out", type=Path, required=True)
    dated = b.add_mutually_exclusive_group()
    dated.add_argument("--cache-dir", type=Path)
    dated.add_argument("--dates", type=Path, help="an earlier report.json, for its cache dates")
    u = sub.add_parser("upload")
    u.add_argument("out", type=Path)
    u.add_argument("--expect-sha256", required=True)
    v = sub.add_parser("verify")
    v.add_argument("out", type=Path)
    for p in (u, v):
        p.add_argument("--registry", default=os.environ.get("COHDL_REGISTRY", "https://registry.cohdl.org"))
    args = parser.parse_args(argv)
    try:
        if args.command == "build":
            report = build(args.out, args.cache_dir, args.dates)
            print(render(report))
            return 0
        if args.command == "upload":
            problems = upload(args.out, args.expect_sha256, args.registry, os.environ.get("COHDL_ADMIN_SESSION"))
        else:
            problems = verify(args.out, args.registry)
    except SnapshotError as error:
        problems = [str(error)]
    for problem in problems:
        print(f"upload_supplier_snapshot.py: {problem}", file=sys.stderr)
    return 1 if problems else 0


if __name__ == "__main__":
    raise SystemExit(main())
