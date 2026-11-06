#!/usr/bin/env python3
"""Index datasheet links for every library part identity and every B1 snapshot part.

``python3 -I tools/datasheet_links.py -o DIR [--check]`` writes
DIR/datasheet_links.json and DIR/datasheet_links.md. Committed indexes are
dated snapshots (``tools/sourcing_data/datasheet_links/YYYY-MM-DD/``) whose
``sources`` record what they were built from; a later library change does not
make them wrong, only older. Nothing is fetched or downloaded:
the index only collects links the repository already records. It is the input
for the agent-facing part notes (plan 5b/5c).

An identity is one ``primary`` or ``alt`` entry of a ``pub part`` in ``lib/``.
Its documents are the ``#[doc]`` files on the part and on the device it
instantiates (same package). For each document, official links come from:

- the document itself, when it is a text record: the lines that name the
  identity's MPN; or, for a record with at most MAX_RECORD_URLS links, all of
  them (an evidence record covers one part family);
- the package's ``docs/README.md``: the lines that name the document's file,
  with the SHA-256 recorded on them.

A document-level link counts for an identity only if its line names the MPN
or the identity's manufacturer, so a Vishay alternate does not inherit the
Yageo primary's datasheet. Links to LCSC or JLCPCB are distributor links, not
official ones. The LCSC datasheet link comes from the confirmed part page
behind the identity's ``lcsc_map.json`` row.

Each identity is classed ``official_mpn`` (an official link on a line naming
its MPN), ``official_document`` (only document-level links for its
manufacturer, such as a series datasheet), ``lcsc_only``, ``local_only`` (a
document in the repository but no recorded link) or ``none``. Links and
documents are stored once and referenced by index; LCSC datasheet links are
keyed by part number.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import _balanced_block, _string_fields, _strip_comments
from sourcing.bindings import package_metadata

ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT / "lib"
DATA = ROOT / "tools/sourcing_data"
MAX_RECORD_URLS = 20
URL = re.compile(r"https?://[^\s<>()\[\]`|\"']+")
SHA256 = re.compile(r"\b[0-9a-f]{64}\b")
DISTRIBUTORS = ("lcsc.com", "jlcpcb.com")
DOC = re.compile(r'#\[doc\("([^"]+)"\)\]')
ITEM = re.compile(r"\b(?:pub\s+)?(part|device)\s+(\w+)")
CLASSES = ("official_mpn", "official_document", "lcsc_only", "local_only", "none")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _official(url: str) -> bool:
    host = url.split("/")[2].lower()
    return not any(host == d or host.endswith("." + d) for d in DISTRIBUTORS)


def _mfr_token(mfr: str) -> str:
    return re.split(r"[\s(,]", mfr.strip())[0].lower()


def packages() -> list[tuple[str, str, Path]]:
    found = []
    for manifest in sorted([*LIB.glob("*/cohdl.toml"), *LIB.glob("@*/*/cohdl.toml")]):
        metadata = package_metadata(manifest)
        if metadata:
            found.append((metadata[0], metadata[1], manifest.parent))
    return found


def items(text: str):
    """(kind, name, docs, block) for each part and device, docs being the #[doc] files just above it."""
    pending: list[str] = []
    pos = 0
    pattern = re.compile(r'#\[doc\("([^"]+)"\)\]|\b(?:pub\s+)?(part|device)\s+(\w+)[^{;]*\{')
    while True:
        match = pattern.search(text, pos)
        if not match:
            return
        if match.group(1):
            pending.append(match.group(1))
            pos = match.end()
            continue
        block, end = _balanced_block(text, match.end() - 1)
        header = text[match.start():match.end()]
        yield match.group(2), match.group(3), pending, header, block
        pending = []
        pos = end


def library_identities() -> tuple[list[dict], dict[tuple[str, str], dict]]:
    identities, documents = [], {}
    for name, version, root in packages():
        readme = root / "docs" / "README.md"
        readme_lines = readme.read_text(encoding="utf-8").splitlines() if readme.is_file() else []
        device_docs: dict[str, list[str]] = {}
        parts = []
        for path in sorted((root / "src").rglob("*.cohdl")):
            text = _strip_comments(path.read_text(encoding="utf-8"))
            for kind, item, docs, header, block in items(text):
                if kind == "device":
                    device_docs.setdefault(item, []).extend(docs)
                else:
                    device = re.match(r"part\s+\w+\s*:\s*(\w+)", re.sub(r"^pub\s+", "", header.strip()))
                    parts.append((item, device.group(1) if device else None, docs, block))
        for part, device, docs, block in parts:
            files = list(dict.fromkeys(docs + device_docs.get(device, [])))
            for entry in re.finditer(r"\b(primary|alt)\s*\{", block):
                fields = dict(_string_fields(_balanced_block(block, entry.end() - 1)[0]))
                if "mfr" not in fields or "mpn" not in fields:
                    continue
                identities.append({"package": name, "version": version, "part": part, "entry": entry.group(1),
                                   "mfr": fields["mfr"], "mpn": fields["mpn"], "docs": files})
            for doc in files:
                key = (name, doc)
                if key not in documents:
                    documents[key] = document(root, doc, readme_lines)
    return identities, documents


def document(root: Path, doc: str, readme_lines: list[str]) -> dict:
    path = root / doc
    record = {"local": path.is_file(), "sha256": None, "readme_lines": [], "record_lines": []}
    base = Path(doc).name
    for line in readme_lines:
        if base in line and URL.search(line):
            record["readme_lines"].append(line)
            sha = SHA256.search(line)
            record["sha256"] = record["sha256"] or (sha.group(0) if sha else None)
    if record["local"] and path.suffix in (".md", ".txt"):
        record["record_lines"] = [line for line in path.read_text(encoding="utf-8").splitlines() if URL.search(line)]
    elif record["local"] and record["sha256"] is None:
        record["sha256"] = _sha256(path)
    return record


def mpn_forms(mpn: str) -> list[str]:
    """The MPN, and the MPN without a tape-and-reel suffix, which catalogs often leave off."""
    bare = re.sub(r"[-/]?(TR|T&R|RL)$", "", mpn)
    return [mpn] + ([bare] if bare != mpn and len(bare) >= 6 else [])


def links_for(identity: dict, documents: dict) -> list[dict]:
    forms, token = mpn_forms(identity["mpn"]), _mfr_token(identity["mfr"])
    found: dict[str, dict] = {}
    for doc in identity["docs"]:
        record = documents[(identity["package"], doc)]
        lines = [(line, "record") for line in record["record_lines"]]
        small = sum(len(URL.findall(line)) for line, _ in lines) <= MAX_RECORD_URLS
        lines += [(line, "readme") for line in record["readme_lines"]]
        for line, source in lines:
            names_mpn = any(form in line for form in forms)
            if not (names_mpn or (source == "record" and small) or token in line.lower()):
                continue
            if source == "readme" and not (names_mpn or token in line.lower()):
                continue
            for url in URL.findall(line):
                url = url.rstrip(".,;")
                if _official(url) and url not in found:
                    found[url] = {"url": url, "doc": doc, "match": "mpn" if names_mpn else "document",
                                  "sha256": record["sha256"] if source == "readme" else None}
    return list(found.values())


def build() -> dict:
    identities, documents = library_identities()
    map_rows = json.loads((DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]
    by_identity = {(row["mfr"], row["mpn"]): row["pn"] for row in map_rows}
    snapshot = {row["snapshot_sha256"] for row in map_rows}
    pages_path = next(p for p in sorted((DATA / "lcsc_backfill").glob("*/confirmed.json"))
                      if _sha256(p) in snapshot)
    pages = {page["c"]: page for page in json.loads(pages_path.read_text(encoding="utf-8"))}
    catalog_path = DATA / "jlc_catalog/2026-10-30/catalog.json"
    catalog = json.loads(catalog_path.read_text(encoding="utf-8"))
    lcsc = {page["c"]: page.get("datasheet") for page in catalog}
    lcsc.update({pn: page.get("datasheet") for pn, page in pages.items()})

    rows, counts, link_index, doc_index, datasheets = [], {}, {}, {}, {}
    for identity in identities:
        links = links_for(identity, documents)
        pn = by_identity.get((identity["mfr"], identity["mpn"]))
        datasheet = lcsc.get(pn) if pn else None
        local = any(documents[(identity["package"], d)]["local"] and not d.endswith((".md", ".txt"))
                    for d in identity["docs"])
        status = "official" if links else "lcsc_only" if datasheet else "local_only" if local else "none"
        if status == "official" and any(l["match"] == "mpn" for l in links):
            status = "official_mpn"
        elif status == "official":
            status = "official_document"
        counts.setdefault(identity["package"], dict.fromkeys(CLASSES, 0))[status] += 1
        if datasheet:
            datasheets[pn] = datasheet
        rows.append([identity["package"], identity["part"], identity["entry"], identity["mfr"], identity["mpn"], status,
                     [[link_index.setdefault((l["url"], l["sha256"]), len(link_index)), l["match"]] for l in links],
                     [doc_index.setdefault((identity["package"], d), len(doc_index)) for d in identity["docs"]], pn])
    library_pns = {pn for pn in by_identity.values()}
    for page in catalog:
        if page.get("datasheet"):
            datasheets[page["c"]] = page["datasheet"]
    snapshot_rows = [[page["c"], page["mfr"], page["mpn"], page["category"], page["c"] in library_pns]
                     for page in sorted(catalog, key=lambda p: int(p["c"][1:]))]
    return {
        "schema_version": 1,
        "sources": {"lcsc_map.json": _sha256(DATA / "lcsc_map.json"),
                    str(pages_path.relative_to(DATA)): _sha256(pages_path),
                    str(catalog_path.relative_to(DATA)): _sha256(catalog_path)},
        "links": [[url, sha] for (url, sha) in link_index],
        "docs": [list(key) for key in doc_index],
        "identity_columns": ["package", "part", "entry", "mfr", "mpn", "status",
                             "official [index into links, match]", "docs (index into docs)", "lcsc_pn"],
        "identities": rows,
        "snapshot_columns": ["pn", "mfr", "mpn", "category", "mapped_by_library"],
        "snapshot": snapshot_rows,
        "lcsc_datasheets": dict(sorted(datasheets.items(), key=lambda item: int(item[0][1:]))),
        "counts": dict(sorted(counts.items())),
    }


def render(report: dict) -> str:
    counts = report["counts"]
    total = {c: sum(v[c] for v in counts.values()) for c in CLASSES}
    snap = report["snapshot"]
    lines = [
        "# Datasheet links",
        "",
        "Generated by `tools/datasheet_links.py`; do not edit. Links only: nothing was downloaded.",
        "",
        f"- Library identities: {sum(total.values()):,}. Official link on a line naming the MPN: "
        f"{total['official_mpn']:,}; official document link for the manufacturer: {total['official_document']:,}; "
        f"LCSC datasheet only: "
        f"{total['lcsc_only']:,}; a document in the repository but no recorded link: {total['local_only']:,}; "
        f"nothing: {total['none']:,}.",
        f"- B1 snapshot parts: {len(snap):,}, of which {sum(1 for r in snap if r[0] in report['lcsc_datasheets']):,} "
        f"have an LCSC datasheet link and {sum(1 for r in snap if r[4]):,} are mapped by a library entry.",
        "",
        "| Package | Official (MPN) | Official (document) | LCSC only | Local only | None |",
        "|---|---:|---:|---:|---:|---:|",
    ]
    for package, c in counts.items():
        lines.append(f"| `{package}` | {c['official_mpn']:,} | {c['official_document']:,} | {c['lcsc_only']:,} | "
                     f"{c['local_only']:,} | {c['none']:,} |")
    gaps = [r for r in report["identities"] if r[5] == "none"]
    lines += ["", f"## No link at all ({len(gaps):,})", ""]
    lines += [f"- `{r[0]}` `{r[1]}` ({r[2]}): {r[3]} `{r[4]}`" for r in gaps]
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-o", "--out", type=Path, required=True)
    parser.add_argument("--check", action="store_true", help="fail if DIR differs from a fresh build")
    args = parser.parse_args(argv)
    report = build()
    files = {"datasheet_links.json": json.dumps(report, ensure_ascii=False, separators=(",", ":")) + "\n",
             "datasheet_links.md": render(report)}
    if args.check:
        stale = [n for n, t in files.items()
                 if not (args.out / n).is_file() or (args.out / n).read_text(encoding="utf-8") != t]
        for name in stale:
            print(f"datasheet_links.py: {args.out / name} is not what the repository builds", file=sys.stderr)
        return 1 if stale else 0
    args.out.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (args.out / name).write_text(text, encoding="utf-8")
    print(files["datasheet_links.md"].split("\n## No link")[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
