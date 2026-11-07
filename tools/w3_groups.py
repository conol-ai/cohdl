#!/usr/bin/env python3
"""Group the mapped chip resistors and MLCCs for the W3 sampled-inference proposal.

``python3 -I tools/w3_groups.py -o DIR [--check]`` writes DIR/groups.json and
DIR/groups.md. A committed report is a dated snapshot whose ``sources`` record
what it was built from. Nothing is fetched, and no binding or frame row is written:
this is the grouping report the proposal
(``docs/library-supplier-w3-sampling.md``) asks for, not evidence.

Scope: the ``ChipResistor`` and ``MLCC`` parts of ``lib/passive`` whose
(manufacturer, MPN) ``lcsc_map.json`` maps. Every AVL entry is one member.

A group is (footprint package, its version, footprint, part type, manufacturer,
series), where the series is the MPN's leading letters (Yageo ``RC``/``CC``,
Vishay ``CRCW``, Samsung ``CL``). Different series never share a group, and
the page's package text (``0402``) is not part of the key.

For each group the report gives its members, the members' frame rows with
their own evidence (``jlc_frames.json``), the sample the draft rule asks for
(at least MIN_SAMPLE or FRACTION of the members, whichever is larger), how
many more direct checks that needs, and a deterministic choice of them among
members with at least MIN_STOCK on their confirmed page. A group with a direct
reading that is not (0, 0, 0) is marked ``paused``. Members with no stock can
belong to a group but are never chosen as samples.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import sys

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import _balanced_block, _string_fields, _strip_comments
from sourcing.bindings import package_metadata

ROOT = Path(__file__).resolve().parents[1]
DATA = ROOT / "tools/sourcing_data"
PASSIVE = ROOT / "lib/passive"
TYPES = ("ChipResistor", "MLCC")
# The draft sampling rule under review; see the proposal.
MIN_SAMPLE = 3
FRACTION = 0.05
# The assembler builds at least five boards and adds attrition. On 2026-11-07 the
# W3 pilot's BOM step left unconfirmed, with a shortage warning, exactly the 11
# samples whose confirmed page read fewer than 20 pieces (5 to 19), and confirmed
# every one with 21 or more. 30 leaves room for stock sold since the page was read.
MIN_STOCK = 30
# Samples the assembler's BOM step reported short at a preview, although their
# page read MIN_STOCK or more: the stock was sold after the page was read. Each
# is replaced by the nearest eligible member, so the other samples, which that
# preview confirmed in stock, stay as they are.
SHORT_AT_PREVIEW = {
    "C185414": "2026-11-07 w3-pilot-0402 (page read 39)",
    "C185430": "2026-11-07 w3-pilot-0402 (page read 100)",
}


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def passive_entries(lib: Path = PASSIVE) -> list[dict]:
    """Every AVL entry of a chip resistor or MLCC part, with its effective footprint."""
    entries = []
    for path in sorted((lib / "src").glob("*.cohdl")):
        text = _strip_comments(path.read_text(encoding="utf-8"))
        for match in re.finditer(r"\bpub\s+part\s+(\w+)\s*:\s*(\w+)[^{]*\{", text):
            if match.group(2) not in TYPES:
                continue
            block, _ = _balanced_block(text, match.end() - 1)
            footprint = None
            for kind in re.finditer(r"\b(primary|alt)\s*\{", block):
                fields, _ = _balanced_block(block, kind.end() - 1)
                found = re.search(r"\bfootprint\s*:\s*(\w+)", fields)
                if kind.group(1) == "primary":
                    footprint = found.group(1) if found else None
                values = dict(_string_fields(fields))
                entries.append({"part": match.group(1), "type": match.group(2), "entry": kind.group(1),
                                "mfr": values["mfr"], "mpn": values["mpn"],
                                "footprint": found.group(1) if found else footprint})
    return entries


def series(mpn: str) -> str:
    found = re.match(r"[A-Z]+", mpn)
    if not found:
        raise ValueError(f"{mpn}: no series prefix")
    return found.group(0)


def required(members: int) -> int:
    return max(MIN_SAMPLE, math.ceil(FRACTION * members))


def spread(items: list, count: int) -> list:
    """COUNT items evenly spaced through ITEMS, first and last included when possible."""
    if count >= len(items):
        return list(items)
    if count == 1:
        return [items[0]]
    return [items[round(i * (len(items) - 1) / (count - 1))] for i in range(count)]


def choose(items: list, count: int, short) -> list:
    """spread(ITEMS, COUNT), with each item in SHORT replaced by the nearest item, by
    position, that is neither chosen nor short (the lower one first on a tie)."""
    chosen = spread(items, count)
    taken = {id(item) for item in chosen}
    for n, item in enumerate(chosen):
        if not short(item):
            continue
        at = next(i for i, other in enumerate(items) if other is item)
        nearest = sorted(range(len(items)), key=lambda i: (abs(i - at), i))
        free = [i for i in nearest if id(items[i]) not in taken and not short(items[i])]
        if free:
            chosen[n] = items[free[0]]
            taken.add(id(chosen[n]))
    return [item for item in chosen if not short(item)]


def confirmed_pages(snapshot: str) -> tuple[Path, dict[str, dict]]:
    """The backfill run's confirmed pages whose SHA-256 the map rows cite."""
    for path in sorted((DATA / "lcsc_backfill").glob("*/confirmed.json")):
        if _sha256(path) == snapshot:
            return path, {page["c"]: page for page in json.loads(path.read_text(encoding="utf-8"))}
    raise SystemExit(f"w3_groups.py: no lcsc_backfill run has confirmed pages {snapshot}")


def build() -> dict:
    name, version = package_metadata(PASSIVE / "cohdl.toml")
    map_rows = json.loads((DATA / "lcsc_map.json").read_text(encoding="utf-8"))["rows"]
    by_identity = {(row["mfr"], row["mpn"]): row for row in map_rows}
    snapshots = {row["snapshot_sha256"] for row in map_rows}
    if len(snapshots) != 1:
        raise SystemExit("w3_groups.py: the map cites more than one confirmed snapshot")
    pages_path, pages = confirmed_pages(snapshots.pop())
    frames = json.loads((DATA / "jlc_frames.json").read_text(encoding="utf-8"))["rows"]
    direct = {(row["package"], row["package_version"], row["footprint"], row["pn"]): row
              for row in frames if row["supplier"] == "lcsc" and row["evidence"] is not None}

    groups: dict[tuple, dict] = {}
    unmapped = 0
    for entry in passive_entries():
        row = by_identity.get((entry["mfr"], entry["mpn"]))
        if row is None:
            unmapped += 1
            continue
        page = pages[row["pn"]]
        key = (name, version, entry["footprint"], entry["type"], entry["mfr"], series(entry["mpn"]))
        group = groups.setdefault(key, {"members": [], "direct": []})
        group["members"].append([row["pn"], entry["mpn"], entry["part"], entry["entry"],
                                 row["category"], page["stock"], page["package"]])
        frame = direct.get((name, version, entry["footprint"], row["pn"]))
        if frame is not None:
            group["direct"].append({"pn": row["pn"], "row": frame["id"], "mapping": frame["mapping"],
                                    "record": frame["evidence"]["record"]})

    report = []
    for key in sorted(groups):
        group = groups[key]
        members = sorted(group["members"], key=lambda m: m[1])
        checked = {d["pn"] for d in group["direct"]}
        anomalies = [d["pn"] for d in group["direct"] if any(group_value != 0 for group_value in d["mapping"].values())]
        need = max(0, required(len(members)) - len(checked))
        eligible = [m for m in members if m[0] not in checked and m[5] >= MIN_STOCK]
        report.append({
            "package": key[0], "package_version": key[1], "footprint": key[2], "type": key[3],
            "mfr": key[4], "series": key[5],
            "members": len(members),
            "zero_stock": sum(m[5] <= 0 for m in members),
            "below_min_stock": sum(0 < m[5] < MIN_STOCK for m in members),
            "page_package_text": sorted({m[6] for m in members}),
            "direct": sorted(group["direct"], key=lambda d: d["pn"]),
            "required": required(len(members)),
            "more_needed": need,
            "status": "paused" if anomalies else ("sampled" if need == 0 else "needs_samples"),
            "anomalies": anomalies,
            "sample": [m[0] for m in choose(eligible, need, lambda m: m[0] in SHORT_AT_PREVIEW)],
            "member_rows": [m[:6] for m in members],
        })
    return {
        "schema_version": 1,
        "rule": {"min_sample": MIN_SAMPLE, "fraction": FRACTION, "min_stock": MIN_STOCK,
                 "short_at_preview": SHORT_AT_PREVIEW},
        "sources": {
            "lib/passive": version,
            "lcsc_map.json": _sha256(DATA / "lcsc_map.json"),
            "jlc_frames.json": _sha256(DATA / "jlc_frames.json"),
            str(pages_path.relative_to(DATA)): _sha256(pages_path),
        },
        "unmapped_entries": unmapped,
        "member_columns": ["pn", "mpn", "part", "entry", "category", "stock"],
        "groups": report,
    }


def render(report: dict) -> str:
    groups = report["groups"]
    rule = report["rule"]
    total = sum(g["members"] for g in groups)
    lines = [
        "# W3 sampling groups: chip resistors and MLCCs",
        "",
        "Generated by `tools/w3_groups.py`; do not edit. This is a grouping report for the "
        "sampled-inference proposal, not evidence. No binding depends on it.",
        "",
        f"- `lib/passive` {report['sources']['lib/passive']}; {total:,} mapped entries in {len(groups)} groups "
        f"({report['unmapped_entries']:,} entries are not mapped).",
        f"- Draft rule under review: at least {rule['min_sample']} or {rule['fraction']:.0%} of a group's "
        f"members, whichever is larger, checked directly. Samples are chosen only among members "
        f"with at least {rule['min_stock']} in stock.",
        f"- Groups: {sum(g['status'] == 'sampled' for g in groups)} already sampled, "
        f"{sum(g['status'] == 'needs_samples' for g in groups)} need samples, "
        f"{sum(g['status'] == 'paused' for g in groups)} paused.",
        f"- Direct checks still needed: {sum(g['more_needed'] for g in groups)}. "
        f"Chosen: {sum(len(g['sample']) for g in groups)}.",
        f"- Members with no stock: {sum(g['zero_stock'] for g in groups):,}. They can be inferred only "
        "once the rule is accepted, and are never recorded as previewed.",
        "- A group whose members with stock cannot make up its sample (Chosen below More needed) "
        "cannot be inferred: its members need their own evidence.",
        "",
        "| Footprint | Type | Manufacturer | Series | Members | No stock | Direct | Required | More needed | Chosen | Status |",
        "|---|---|---|---|---:|---:|---:|---:|---:|---:|---|",
    ]
    for g in groups:
        lines.append(f"| `{g['footprint']}` | {g['type']} | {g['mfr']} | {g['series']} | {g['members']:,} | "
                     f"{g['zero_stock']:,} | {len(g['direct'])} | {g['required']} | {g['more_needed']} | "
                     f"{len(g['sample'])} | {g['status']} |")
    lines += ["", "## Samples chosen", ""]
    for g in groups:
        if g["sample"]:
            lines.append(f"- `{g['footprint']}` {g['type']} {g['mfr']} {g['series']}: {', '.join(g['sample'])}")
    return "\n".join(lines) + "\n"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("-o", "--out", type=Path, required=True)
    parser.add_argument("--check", action="store_true", help="fail if DIR differs from a fresh build")
    args = parser.parse_args(argv)
    report = build()
    files = {"groups.json": json.dumps(report, indent=1, ensure_ascii=False) + "\n", "groups.md": render(report)}
    if args.check:
        stale = [name for name, text in files.items()
                 if not (args.out / name).is_file() or (args.out / name).read_text(encoding="utf-8") != text]
        for name in stale:
            print(f"w3_groups.py: {args.out / name} is not what the repository builds", file=sys.stderr)
        return 1 if stale else 0
    args.out.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (args.out / name).write_text(text, encoding="utf-8")
    print(render(report).split("\n| Footprint")[0])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
