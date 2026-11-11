#!/usr/bin/env python3
"""Index datasheet links for every library part identity and every B1 snapshot part.

``python3 -I tools/datasheet_links.py -o DIR [--check]`` writes
DIR/datasheet_links.json and DIR/datasheet_links.md. The committed index is
one dated directory (``tools/sourcing_data/datasheet_links/YYYY-MM-DD/``, the
day it was built) whose ``sources`` record what it was built from. The tool
tests require it to equal a fresh build, so a change to the library's
identities or documents, to ``lcsc_map.json`` or to the catalog rebuilds it;
the rebuild goes in a directory named for its own day, replacing the older
one, which stays in git history. Nothing is fetched or downloaded:
the index only collects links the repository already records. It is the input
for the agent-facing part notes (plan 5b/5c).

An identity is one ``primary`` or ``alt`` entry of a ``pub part`` in ``lib/``.
Its documents are the ``#[doc]`` files on the part and on the device it
instantiates (same package). The candidate links of a document are the URLs on
the lines of the document itself, when it is a text record, and on the lines
of the package's ``docs/README.md`` that name the document's file (with the
SHA-256 recorded there).

A candidate link is attached to an identity only when its identity justifies
it, and each attached link records why (its ``match``):

- ``mpn``: the link's line names the identity's MPN as a whole part number,
  not as the prefix of a longer one (ESP32-C3 is not named by
  ESP32-C3-MINI-1-H4, nor STM32C071C8T6 by STM32C071C8T6N), and no longer
  library MPN covers it. A packaging suffix may be missing only where
  ``PACKAGING_SUFFIXES`` lists it for the manufacturer (ST's ``TR``, as in
  ``STM32C011D6Y6TR``). An empty MPN never matches.
- ``manufacturer``: the URL's host belongs to the identity's manufacturer
  (``MANUFACTURER_DOMAINS``), and the line is not a record line naming
  another library part's MPN (a catalog row for a sibling part).

Nothing else attaches a link. A shared record (a package README listing JST,
Molex and Samtec drawings) gives a JST part only its JST links and the lines
naming its MPN, and a Vishay alternate does not inherit the Yageo primary's
datasheet.

Each attached link is also labelled by its host (its ``label``): ``official``
only on a domain of the identity's own manufacturer; ``mirror`` on a known
distributor, aggregator, archive or code host (``MIRRORS``: LCSC, JLCPCB,
DigiKey, Mouser, GitHub and the like); ``unverified`` on any other host,
including another manufacturer's. A link on the manufacturer's own domain
with no path (its home page, such as ``https://www.run-ic.com``) is labelled
``homepage``, not ``official``: it names the vendor but is not a document.
The manufacturer→domain map is explicit
and deliberately small: a domain not in it is never official. The LCSC
datasheet link comes from the confirmed part page behind the identity's
``lcsc_map.json`` row.

Each identity is classed ``official_mpn`` (an official link on a line naming
its MPN), ``official_document`` (only official links that do not name its MPN,
such as a series datasheet), ``unverified`` (links naming its MPN, none on its
manufacturer's domain), ``homepage`` (no document link: the manufacturer's
home page only), ``lcsc_only``, ``local_only`` (a document in the
repository but no recorded link) or ``none``. Links and documents are stored
once and referenced by index; LCSC datasheet links are keyed by part number.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
from urllib.parse import urlsplit

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import _balanced_block, _string_fields, _strip_comments
from sourcing.bindings import package_metadata

ROOT = Path(__file__).resolve().parents[1]
LIB = ROOT / "lib"
DATA = ROOT / "tools/sourcing_data"
URL = re.compile(r"https?://[^\s<>()\[\]`|\"']+")
SHA256 = re.compile(r"\b[0-9a-f]{64}\b")
# Hosts that carry other companies' documents: distributors, aggregators,
# archives and code hosts. A link here is never official, even when it names the MPN.
MIRRORS = ("lcsc.com", "szlcsc.com", "jlcpcb.com", "digikey.com", "digikey.cn", "mouser.com", "mouser.cn",
           "arrow.com", "farnell.com", "element14.com", "octopart.com", "alldatasheet.com", "alldatasheet.net",
           "datasheetspdf.com", "datasheet4u.com", "datasheetarchive.com", "datasheets.com", "archive.org",
           "github.com", "githubusercontent.com", "github.io", "gitlab.com")
# The domains each library manufacturer (as spelled in lib/) publishes its own documents on.
# Explicit and small on purpose: a host not listed here is never official.
MANUFACTURER_DOMAINS = {
    "Abracon": ("abracon.com",),
    "Analog Devices": ("analog.com",),
    "Awinic": ("awinic.com",),
    "Bosch Sensortec": ("bosch-sensortec.com",),
    "C&K": ("ckswitches.com",),
    "Coilcraft": ("coilcraft.com",),
    "Diodes Inc": ("diodes.com",),
    "Diodes Incorporated": ("diodes.com",),
    "Dongguan OPSCO Optoelectronics": ("opscoled.com",),
    "Espressif": ("espressif.com",),
    "GCT": ("gct.co",),
    "GigaDevice": ("gigadevice.com", "gd32mcu.com"),
    "HRO": ("krhro.com",),
    "Hirose Electric": ("hirose.com",),
    "Hosonic": ("hosonic.com",),
    "Infineon": ("infineon.com",),
    "JST": ("jst-mfg.com",),
    "Johanson Technology": ("johansontechnology.com",),
    "KEMET": ("kemet.com",),
    "Lite-On": ("liteon.com",),
    "Luat (Hezhou)": ("openluat.com",),
    "MEMSIC": ("memsic.com",),
    "Molex": ("molex.com",),
    "Murata": ("murata.com",),
    "NSIWAY": ("nsiway.com.cn",),
    "NXP": ("nxp.com",),
    "Nexperia": ("nexperia.com",),
    "Panasonic": ("panasonic.com",),
    "Quectel": ("quectel.com",),
    "Raspberry Pi": ("raspberrypi.com",),
    "Richtek": ("richtek.com",),
    "Run-IC": ("run-ic.com",),
    "SG Micro": ("sg-micro.com",),
    "SGMICRO": ("sg-micro.com",),
    "STMicroelectronics": ("st.com",),
    "Samsung": ("samsungsem.com",),
    "Samtec": ("samtec.com",),
    "Semtech": ("semtech.com",),
    "SiFli Technologies": ("sifli.com",),
    "Sunlord": ("sunlordinc.com",),
    "TDK InvenSense": ("invensense.com", "invensense.tdk.com"),
    "TDK/InvenSense": ("invensense.com", "invensense.tdk.com"),
    "Texas Instruments": ("ti.com",),
    "Torex Semiconductor": ("torexsemi.com",),
    "Vishay": ("vishay.com",),
    "WCH": ("wch.cn", "wch-ic.com"),
    "Winbond": ("winbond.com",),
    "Wurth Elektronik": ("we-online.com",),
    "Xi'an Aerosemi Tech": ("aerosemi.com",),
    "Xinluda": ("xinluda.com",),
    "Yageo": ("yageogroup.com", "yageo.com"),
    "onsemi": ("onsemi.com",),
    "u-blox": ("u-blox.com",),
}
DOC = re.compile(r'#\[doc\("([^"]+)"\)\]')
ITEM = re.compile(r"\b(?:pub\s+)?(part|device)\s+(\w+)")
CLASSES = ("official_mpn", "official_document", "unverified", "homepage", "lcsc_only", "local_only", "none")


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _host(url: str) -> str:
    return url.split("/")[2].split("@")[-1].split(":")[0].lower()


def _on(host: str, domains) -> bool:
    return any(host == d or host.endswith("." + d) for d in domains)


def _is_homepage(url: str) -> bool:
    """No path beyond the root, no query and no fragment: a site's home page, not a document."""
    parts = urlsplit(url)
    return parts.path in ("", "/") and not parts.query and not parts.fragment


def label(url: str, mfr: str) -> str:
    """``official`` on a domain of ``mfr`` itself (``homepage`` for its bare root), ``mirror`` on a known
    mirror, else ``unverified``."""
    host = _host(url)
    if _on(host, MIRRORS):
        return "mirror"
    if not _on(host, MANUFACTURER_DOMAINS.get(mfr, ())):
        return "unverified"
    return "homepage" if _is_homepage(url) else "official"


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


# Packaging suffixes a manufacturer appends to an orderable MPN, which its own catalogs leave off
# (ST's tape-and-reel "TR": STM32C011D6Y6TR is listed as STM32C011D6Y6). Explicit per manufacturer:
# a library MPN is never matched by an open-ended prefix of itself.
PACKAGING_SUFFIXES = {"STMicroelectronics": ("TR",)}
# A part number continues through letters and digits, an adjacent "(" (an option code, as in
# U.FL-R-SMT-1(10)), and "-", "/", ".", "+" or "#" followed by a letter or digit ("-", ".", "+" or "#"
# behind one). A "/" before it is a path or list separator, and a "." that starts a document extension
# ends it, so ".../datasheets/AOTA-B201610S3R3-101-T.pdf" names AOTA-B201610S3R3-101-T.
_BEFORE = r"(?<![A-Za-z0-9])(?<![A-Za-z0-9][-.+#])"
_AFTER = r"(?![A-Za-z0-9(])(?![-/+#][A-Za-z0-9])(?!\.(?!(?:pdf|html?|zip|md|txt)\b)[A-Za-z0-9])"


def mpn_forms(mpn: str, mfr: str = "") -> list[str]:
    """The MPN, and the MPN without one of its manufacturer's packaging suffixes (PACKAGING_SUFFIXES).

    An empty MPN has no forms: it names no part, so it can never match a line."""
    mpn = (mpn or "").strip()
    if not mpn:
        return []
    bare = [mpn[:-len(suffix)] for suffix in PACKAGING_SUFFIXES.get(mfr, ()) if mpn.endswith(suffix)]
    return [mpn] + [form for form in bare if len(form) >= 6]


def part_namer(identities: list[dict]):
    """``named(line)``: the MPN forms of these identities that the line names as whole part numbers.

    A form must stand alone as a part number (no letter, digit or part-number punctuation continuing it
    on either side), and where forms overlap the longest wins: a line naming ESP32-C3-MINI-1-H4 names
    that module, not the ESP32-C3 chip. Cached per line."""
    forms = sorted({f for i in identities for f in mpn_forms(i["mpn"], i["mfr"])}, key=lambda f: (-len(f), f))
    pattern = re.compile(_BEFORE + "(?:" + "|".join(map(re.escape, forms)) + ")" + _AFTER) if forms else None
    cache: dict[str, frozenset[str]] = {}

    def named(line: str) -> frozenset[str]:
        if line not in cache:
            cache[line] = frozenset(m.group(0) for m in pattern.finditer(line)) if pattern else frozenset()
        return cache[line]
    return named


def links_for(identity: dict, documents: dict, named=None) -> list[dict]:
    """The links its identity justifies: on a line naming the MPN, or on the manufacturer's own domain.

    ``named`` is the package's ``part_namer``; without it, only this identity's MPN is looked for. A line
    names the MPN only when the MPN stands alone there as a whole part number and no longer part number
    covers it. A manufacturer-domain link on a record line naming another part's MPN (a catalog row)
    belongs to that part, not to this identity. A README line naming the cited document's file is about
    that document, whatever else it names."""
    forms, mfr = set(mpn_forms(identity["mpn"], identity["mfr"])), identity["mfr"]
    named = named or part_namer([identity])
    found: dict[str, dict] = {}
    for doc in identity["docs"]:
        record = documents[(identity["package"], doc)]
        # Record lines can be about anything the record covers; a README line names the cited file itself.
        lines = [(line, None, True) for line in record["record_lines"]]
        lines += [(line, record["sha256"], False) for line in record["readme_lines"]]
        for line, sha256, in_record in lines:
            parts = named(line)
            names_mpn = bool(parts & forms)
            names_other = in_record and not names_mpn and bool(parts - forms)
            for url in URL.findall(line):
                url = url.rstrip(".,;")
                kind = label(url, mfr)
                on_own_domain = kind in ("official", "homepage")
                match = "mpn" if names_mpn else "manufacturer" if on_own_domain and not names_other else None
                if match is None:
                    continue
                link = {"url": url, "doc": doc, "match": match, "label": kind, "sha256": sha256}
                if url not in found or _rank(link) < _rank(found[url]):
                    found[url] = link
    return list(found.values())


def _rank(link: dict) -> tuple[int, int]:
    return (link["match"] != "mpn", link["sha256"] is None)


def classify(links: list[dict], datasheet: str | None, local: bool) -> str:
    """The identity's class (CLASSES) from its attached links, its LCSC datasheet and its local documents."""
    official = [link for link in links if link["label"] == "official"]
    if any(link["match"] == "mpn" for link in official):
        return "official_mpn"
    if official:
        return "official_document"
    if any(link["label"] != "homepage" for link in links):
        return "unverified"
    return "homepage" if links else "lcsc_only" if datasheet else "local_only" if local else "none"


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
    namers = {package: part_namer([i for i in identities if i["package"] == package])
              for package in {i["package"] for i in identities}}
    for identity in identities:
        links = links_for(identity, documents, namers[identity["package"]])
        pn = by_identity.get((identity["mfr"], identity["mpn"]))
        datasheet = lcsc.get(pn) if pn else None
        local = any(documents[(identity["package"], d)]["local"] and not d.endswith((".md", ".txt"))
                    for d in identity["docs"])
        status = classify(links, datasheet, local)
        counts.setdefault(identity["package"], dict.fromkeys(CLASSES, 0))[status] += 1
        if datasheet:
            datasheets[pn] = datasheet
        rows.append([identity["package"], identity["part"], identity["entry"], identity["mfr"], identity["mpn"], status,
                     [[link_index.setdefault((l["url"], l["sha256"]), len(link_index)), l["match"], l["label"],
                       doc_index.setdefault((identity["package"], l["doc"]), len(doc_index))] for l in links],
                     [doc_index.setdefault((identity["package"], d), len(doc_index)) for d in identity["docs"]], pn])
    library_pns = {pn for pn in by_identity.values()}
    for page in catalog:
        if page.get("datasheet"):
            datasheets[page["c"]] = page["datasheet"]
    snapshot_rows = [[page["c"], page["mfr"], page["mpn"], page["category"], page["c"] in library_pns]
                     for page in sorted(catalog, key=lambda p: int(p["c"][1:]))]
    return {
        "schema_version": 3,
        "sources": {"lcsc_map.json": _sha256(DATA / "lcsc_map.json"),
                    str(pages_path.relative_to(DATA)): _sha256(pages_path),
                    str(catalog_path.relative_to(DATA)): _sha256(catalog_path)},
        "links": [[url, sha] for (url, sha) in link_index],
        "docs": [list(key) for key in doc_index],
        "identity_columns": ["package", "part", "entry", "mfr", "mpn", "status",
                             "links [index into links, match (mpn|manufacturer), label (official|homepage|mirror|unverified), "
                             "index into docs of the document it came from]", "docs (index into docs)", "lcsc_pn"],
        "manufacturer_domains": MANUFACTURER_DOMAINS,
        "mirrors": list(MIRRORS),
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
        "A link is attached to an identity only when its line names the identity's MPN or its host is a domain "
        "of the identity's manufacturer (and the line is not a catalog row for another part). It is official "
        "only on the manufacturer's own domain, and only with a path: the domain's bare home page is labelled "
        "homepage, since it is not a document. Links on "
        "distributors, aggregators, archives or code hosts are mirrors, and any other host is unverified.",
        "",
        f"- Library identities: {sum(total.values()):,}. Official link on a line naming the MPN: "
        f"{total['official_mpn']:,}; official link on the manufacturer's domain, not naming the MPN: "
        f"{total['official_document']:,}; links naming the MPN on a mirror or unverified host only: "
        f"{total['unverified']:,}; the manufacturer's home page only, no document link: {total['homepage']:,}; "
        f"LCSC datasheet only: "
        f"{total['lcsc_only']:,}; a document in the repository but no recorded link: {total['local_only']:,}; "
        f"nothing: {total['none']:,}.",
        f"- B1 snapshot parts: {len(snap):,}, of which {sum(1 for r in snap if r[0] in report['lcsc_datasheets']):,} "
        f"have an LCSC datasheet link and {sum(1 for r in snap if r[4]):,} are mapped by a library entry.",
        "",
        "| Package | Official (MPN) | Official (document) | Unverified | Homepage only | LCSC only | Local only | None |",
        "|---|---:|---:|---:|---:|---:|---:|---:|",
    ]
    for package, c in counts.items():
        lines.append(f"| `{package}` | {c['official_mpn']:,} | {c['official_document']:,} | {c['unverified']:,} | "
                     f"{c['homepage']:,} | {c['lcsc_only']:,} | {c['local_only']:,} | {c['none']:,} |")
    unverified = [r for r in report["identities"] if r[5] == "unverified"]
    lines += ["", f"## Links naming the MPN, none official ({len(unverified):,})", ""]
    lines += [f"- `{r[0]}` `{r[1]}` ({r[2]}): {r[3]} `{r[4]}`: "
              + ", ".join(f"{report['links'][i][0]} ({kind})" for i, _, kind, _ in r[6]) for r in unverified]
    homepages = [r for r in report["identities"] if r[5] == "homepage"]
    lines += ["", f"## The manufacturer's home page only ({len(homepages):,})", ""]
    lines += [f"- `{r[0]}` `{r[1]}` ({r[2]}): {r[3]} `{r[4]}`: "
              + ", ".join(report["links"][i][0] for i, *_ in r[6])
              + (f"; LCSC datasheet for {r[8]}" if r[8] in report["lcsc_datasheets"] else "") for r in homepages]
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
