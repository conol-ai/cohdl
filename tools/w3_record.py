#!/usr/bin/env python3
"""Write a W3 group record and the frame rows inferred from it.

``python3 -I tools/w3_record.py RECORD --group FOOTPRINT,TYPE,MFR,SERIES [--check]``

The group must be ``sampled`` in what ``w3_groups.py`` builds from the current
tree, under the rule in ``sourcing/w3_rule.py``. The tool then:
- adopts each passed sample's own frame row into ``jlc_frames.json`` at the
  current ``passive`` version, if it is not there yet: the row from the archived
  ``frames.json`` of the preview record that judged it, with its id, mapping and
  evidence unchanged;
- writes RECORD, a group record (``docs/library-supplier-w3-sampling.md`` §3.3).
  Each sample carries the reading the rule judges it by: the method its body was
  read by, its angle, raw offset, the view's median and the residual, and what
  that median is made of (the view, its drawn bodies, and how many are not the
  group's). A sample placed more than once has one reading per copy: the first,
  on the earliest record, is the sample's own, and the rest are listed under
  ``other_copies``. Every copy must be within the rule, or the group would not be
  ``sampled``. The group also states how many members were judged and its
  not-drawn share;
- writes one ``sampled`` row for every other member with stock on its
  confirmed page, not listed as not drawn on a preview, and with no row of
  its own under the group's key.

With ``--check`` it changes nothing and fails if RECORD or ``jlc_frames.json``
differs from what it would write. Nothing is fetched.
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import sys

sys.path.insert(0, str(Path(__file__).resolve().parent))
import w3_groups  # noqa: E402
from sourcing import w3_rule  # noqa: E402

ROOT = w3_groups.ROOT
FRAMES = w3_groups.DATA / "jlc_frames.json"
LAND = ("lib/passive/src/footprints.cohdl", "lib/passive/src/pads.cohdl")
# A sample's reading as the group record carries it (``w3_rule.sample_reading``).
READING = ("body_method", "angle_deg", "offset_limit_mm", "raw_offset_xy_mm", "raw_offset_mm", "bias_xy_mm",
           "residual_offset_mm", "bias_view", "bias_bodies", "bias_other_bodies")


def group_id(g: dict) -> str:
    return "-".join([g["package"], g["package_version"], g["footprint"], g["type"], g["mfr"], g["series"]])


def archived_rows(record: str) -> dict[str, dict]:
    """The archived frames.json next to a preview record's README, by part number."""
    path = ROOT / Path(record).parent / "frames.json"
    if not path.is_file():
        return {}
    return {row["pn"]: row for row in json.loads(path.read_text(encoding="utf-8"))["rows"]}


def build(record_path: str, key: tuple[str, str, str, str]) -> tuple[dict, dict]:
    report = w3_groups.build()
    found = [g for g in report["groups"] if (g["footprint"], g["type"], g["mfr"], g["series"]) == key]
    if len(found) != 1:
        raise SystemExit(f"w3_record.py: no single group {key}")
    g = found[0]
    if g["status"] != "sampled":
        raise SystemExit(f"w3_record.py: group {key} is {g['status']}, not sampled")
    frames = json.loads(FRAMES.read_text(encoding="utf-8"))
    rows = frames["rows"]
    gid = group_id(g)
    # Drop this group's earlier output, so a rerun rewrites it rather than stacking.
    rows = [r for r in rows if (r.get("sampled") or {}).get("group") != gid]
    own = {(r["package"], r["package_version"], r["footprint"], r["pn"]): r for r in rows
           if r["supplier"] == "lcsc" and r.get("evidence") is not None}
    ids = {r["id"] for r in rows}

    # Every copy of every judged member, to carry the copies beyond a sample's first.
    judged = w3_groups.judged_members()
    samples = []
    for passed in g["passed"]:
        pn = passed["pn"]
        k = (g["package"], g["package_version"], g["footprint"], pn)
        row = own.get(k)
        if row is None:
            candidates = [a for record in passed["records"] for a in [archived_rows(record).get(pn)] if a]
            if len(candidates) != 1:
                raise SystemExit(f"w3_record.py: sample {pn} has no single own row to adopt")
            row = dict(candidates[0], package_version=g["package_version"])
            if row["id"] in ids or row["footprint"] != g["footprint"] or row["mapping"] != {
                    "dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}:
                raise SystemExit(f"w3_record.py: archived row for {pn} cannot be adopted")
            rows.append(row)
            own[k], ids = row, ids | {row["id"]}
        sample = {"pn": pn, "row": row["id"], **{field: passed[field] for field in READING}}
        copies = []
        for c in judged[(g["footprint"], pn)]:
            made = g["views"][c["view"]]
            copies.append({"ref": c["ref"], **dict(c["reading"] or {}, bias_view=c["view"], bias_bodies=made["bodies"],
                                                   bias_other_bodies=made["other_bodies"])})
        bad = [c["ref"] for c in copies
               if set(c) != {"ref", *READING} or not (w3_rule.sample_ok(c) and w3_rule.bias_supported(c))]
        if bad or not copies or {field: copies[0][field] for field in READING} != {
                field: sample[field] for field in READING}:
            raise SystemExit(f"w3_record.py: sample {pn} is not within the rule at every copy ({', '.join(bad)})")
        sample["copies"] = len(copies)
        if copies[1:]:
            sample["other_copies"] = copies[1:]
        samples.append(sample)

    members = sorted(m[0] for m in g["member_rows"])
    zero_stock = sorted(m[0] for m in g["member_rows"] if m[5] <= 0)
    not_drawn = sorted(n["pn"] for n in g["not_drawn"])
    if not w3_rule.views_consistent(samples) or g["not_drawn_excess"] or g["bias_unsupported"]:
        raise SystemExit(f"w3_record.py: group {key} does not hold the rule's conditions on its views")
    pages = next(name for name in report["sources"] if name.startswith("lcsc_backfill/"))
    record = {
        "schema_version": 1,
        "rule": w3_rule.RULE,
        "built_by": "tools/w3_record.py",
        "stock_source": {"path": f"tools/sourcing_data/{pages}", "sha256": report["sources"][pages]},
        "groups": [{
            "id": gid, "package": g["package"], "package_version": g["package_version"],
            "footprint": g["footprint"], "type": g["type"], "mfr": g["mfr"], "series": g["series"],
            "status": "active",
            "land": {path: w3_groups._sha256(ROOT / path) for path in LAND},
            "members": members, "zero_stock": zero_stock, "not_drawn": not_drawn,
            # Part numbers judged on a preview, and the share of them not drawn
            # (of max(judged, required), ``w3_rule.not_drawn_excess``).
            "judged": g["judged"], "not_drawn_share": g["not_drawn_share"],
            "samples": sorted(samples, key=lambda s: s["pn"]),
        }],
    }

    sample_pns = {s["pn"] for s in samples}
    stock = {m[0]: m[5] for m in g["member_rows"]}
    for pn in members:
        k = (g["package"], g["package_version"], g["footprint"], pn)
        if (pn in sample_pns or k in own or (stock[pn] <= 0 and not w3_rule.RULE["infer_zero_stock"])
                or (pn in not_drawn and not w3_rule.RULE["infer_not_drawn"])):
            continue
        rows.append({
            "id": f"w3-{g['footprint'].lower().replace('_', '')}-{g['series'].lower()}-{pn.lower()}",
            "package": g["package"], "package_version": g["package_version"], "footprint": g["footprint"],
            "supplier": "lcsc", "pn": pn,
            "mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0}, "measured": None, "exception": None,
            "evidence": None, "evidence_ref": None,
            "sampled": {"group": gid, "record": record_path,
                        "note": f"inferred under W3 rule {w3_rule.RULE['version']} from {len(samples)} samples "
                                f"of its group; not previewed itself"},
        })
    return record, dict(frames, rows=rows)


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("record", help="the group record's repository path")
    parser.add_argument("--group", required=True, help="FOOTPRINT,TYPE,MFR,SERIES")
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    record, frames = build(args.record, tuple(args.group.split(",")))
    files = {ROOT / args.record: json.dumps(record, indent=1, ensure_ascii=False) + "\n",
             FRAMES: json.dumps(frames, indent=2, ensure_ascii=False) + "\n"}
    if args.check:
        stale = [p for p, text in files.items() if not p.is_file() or p.read_text(encoding="utf-8") != text]
        for p in stale:
            print(f"w3_record.py: {p.relative_to(ROOT)} is not what the repository builds", file=sys.stderr)
        return 1 if stale else 0
    for p, text in files.items():
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text, encoding="utf-8")
    g = record["groups"][0]
    inferred = sum(1 for r in frames["rows"] if (r.get("sampled") or {}).get("group") == g["id"])
    print(f"{g['id']}: {len(g['members'])} members, {len(g['samples'])} samples, "
          f"{len(g['zero_stock'])} with no stock, {len(g['not_drawn'])} not drawn, {inferred} inferred rows")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
