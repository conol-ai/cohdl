#!/usr/bin/env python3
"""Promote a reviewed ``lcsc_backfill.py`` run into the official LCSC map.

``python3 -I tools/promote_lcsc_map.py RUN_DIR [--check]`` merges RUN_DIR's
``lcsc_map.json`` into ``tools/sourcing_data/lcsc_map.json``.  Nothing is
fetched.  Before any row is written, the run is checked again:

- every file listed in its ``provenance.json`` still has the recorded SHA-256;
- the run's alias table is the reviewed ``mfr_aliases.json`` as it stands now,
  so no alias that was proposed but not accepted reaches the official map;
- each row cites the run's confirmed snapshot, and its part number, MPN and
  class are what that snapshot read on the part's page, with the manufacturer
  equal or related through the reviewed alias table as the row's rule says.

The merged map must still name one part number per (manufacturer, MPN) and
one (manufacturer, MPN) per part number.  A row already present with the same
identity and part number is replaced by the run's, so a later run refreshes
its date and snapshot.  Any conflict fails the whole promotion and writes
nothing.  ``--check`` reports what would change and writes nothing.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import sys

# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from lcsc_backfill import confirmation_mpn, load_aliases, manufacturer_match

ROOT = Path(__file__).resolve().parents[1]
OFFICIAL = ROOT / "tools/sourcing_data/lcsc_map.json"
ALIASES = ROOT / "tools/sourcing_data/mfr_aliases.json"
FIELDS = ("category", "date", "mfr", "mpn", "pn", "rule", "snapshot_sha256")


class PromotionError(ValueError):
    pass


def _json(path: Path):
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, ValueError) as error:
        raise PromotionError(f"{path}: {error}") from error


def _sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_rows(run: Path, aliases_path: Path = ALIASES) -> list[dict]:
    """The run's map rows, after checking them against its own evidence."""
    provenance = _json(run / "provenance.json")
    for name, digest in sorted(provenance.get("sha256", {}).items()):
        if not (run / name).is_file() or _sha256(run / name) != digest:
            raise PromotionError(f"{run / name} does not match the SHA-256 in provenance.json")
    if "lcsc_map.json" not in provenance.get("sha256", {}) or "confirmed.json" not in provenance["sha256"]:
        raise PromotionError(f"{run}/provenance.json does not cover lcsc_map.json and confirmed.json")
    sources = provenance.get("sources", {})
    if sources.get("aliases_sha256") != _sha256(aliases_path):
        raise PromotionError(f"{run} used a different alias table than the reviewed {aliases_path.name}; "
                             "recompute the run from its cache first")
    aliases = load_aliases(aliases_path)
    snapshot = provenance["sha256"]["confirmed.json"]
    pages = {}
    for page in _json(run / "confirmed.json"):
        pages.setdefault(page["c"], []).append(page)
    data = _json(run / "lcsc_map.json")
    if data.get("schema_version") != 1 or data.get("supplier") != "lcsc" or not isinstance(data.get("rows"), list):
        raise PromotionError(f"{run}/lcsc_map.json is not a schema 1 lcsc map")
    rows = []
    for row in data["rows"]:
        label = f"{row.get('mfr')} {row.get('mpn')} -> {row.get('pn')}"
        if not isinstance(row, dict) or set(row) != set(FIELDS):
            raise PromotionError(f"{label}: row fields must be exactly {', '.join(FIELDS)}")
        if row["snapshot_sha256"] != snapshot or row["date"] != provenance.get("date"):
            raise PromotionError(f"{label}: row does not cite the run's snapshot and date")
        if row["rule"] not in ("exact", "alias"):
            raise PromotionError(f"{label}: a run row's rule must be exact or alias")
        matches = [page for page in pages.get(row["pn"], [])
                   if confirmation_mpn(page["mpn"]) == confirmation_mpn(row["mpn"])
                   and manufacturer_match(row["mfr"], page["mfr"], aliases) == row["rule"]]
        if not matches:
            raise PromotionError(f"{label}: no confirmed page reads this part number, MPN and manufacturer "
                                 f"under rule {row['rule']}")
        if row["category"] not in {page["category"] for page in matches} | (
                {"preferred"} if any(page["category"] == "extended" for page in matches) else set()):
            raise PromotionError(f"{label}: class {row['category']} is not what the page read")
        rows.append({field: row[field] for field in FIELDS})
    return rows


def merge(official: list[dict], promoted: list[dict]) -> tuple[list[dict], list[str]]:
    rows = {(row["mfr"], row["mpn"], row["pn"]): row for row in official}
    changes = []
    for row in promoted:
        key = (row["mfr"], row["mpn"], row["pn"])
        if key not in rows:
            changes.append("add %s %s -> %s (%s)" % (*key, row["rule"]))
        elif rows[key] != row:
            changes.append("refresh %s %s -> %s" % key)
        rows[key] = row
    by_identity, by_pn, errors = {}, {}, []
    for mfr, mpn, pn in sorted(rows):
        if by_identity.setdefault((mfr, mpn), pn) != pn:
            errors.append(f"{mfr} {mpn} would name both {by_identity[(mfr, mpn)]} and {pn}")
        if by_pn.setdefault(pn, (mfr, mpn)) != (mfr, mpn):
            errors.append(f"{pn} would name both {' '.join(by_pn[pn])} and {mfr} {mpn}")
    if errors:
        raise PromotionError("; ".join(errors))
    return [rows[key] for key in sorted(rows)], changes


def render(rows: list[dict]) -> str:
    return json.dumps({"schema_version": 1, "supplier": "lcsc", "rows": rows},
                      ensure_ascii=False, indent=2) + "\n"


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("run", type=Path)
    parser.add_argument("--map", type=Path, default=OFFICIAL)
    parser.add_argument("--aliases", type=Path, default=ALIASES)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    try:
        official = _json(args.map)
        if official.get("schema_version") != 1 or official.get("supplier") != "lcsc":
            raise PromotionError(f"{args.map} is not a schema 1 lcsc map")
        rows, changes = merge(official["rows"], run_rows(args.run, args.aliases))
    except PromotionError as error:
        print("promote_lcsc_map.py: " + str(error), file=sys.stderr)
        return 1
    for change in changes:
        print(change)
    print("%d rows, %d changed" % (len(rows), len(changes)))
    if not args.check:
        args.map.write_text(render(rows), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
