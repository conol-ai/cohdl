#!/usr/bin/env python3
"""Write the SOT-23 frame rows that the preview readings of boards 15 to 19 support.

``python3 -I tools/frame_boards/sot23_frames.py [--check]`` replaces, in
``tools/sourcing_data/jlc_frames.json``, every row whose id starts with one of
``RECORDS``' prefixes, then writes one row per part number that a record read
with its pin 1 mark at our pad 1 on all four copies, provided that
``lcsc_map.json`` maps the part's manufacturer and MPN to that part number. A
part number with no map row yet gets no frame row, so the bindings
``tools/gen_sot23*.py`` write follow the map as it grows.

A part read with no mapping gets the mapping (0, 0, 0). A part re-verified with
a candidate gets the candidate as both mapping and measurement, with the
exception naming the record that left it for manual review. With ``--check``
nothing is written, and the tool fails if the committed table differs.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
FRAMES = ROOT / "tools/sourcing_data/jlc_frames.json"
MAP = ROOT / "tools/sourcing_data/lcsc_map.json"
PREVIEWS = ROOT / "tools/sourcing_data/jlc_previews"
LAND = ("sot", "0.1.2", "SOT3P190X290X130N")
# record -> frame row id prefix; the reading each record's readings.json must give a part
RECORDS = {"2026-11-23-board-15": ("sb15", "pin 1 at pad 1"),
           "2026-11-23-board-16": ("sb16", "pin 1 at pad 1"),
           "2026-11-23-board-17": ("sb17", "pin 1 at pad 1"),
           "2026-11-23-board-18": ("sb18", "pin 1 at pad 1 with the candidate"),
           "2026-11-23-board-19": ("sb19", "pin 1 at pad 1 with the candidate")}


def note(part: dict, mapped: bool) -> str:
    refs = part["designators"]
    angles = ", ".join(f"{d:g}" for d in part["deltas_ccw_from_top"])
    how = (f"with rotate {part['mapping']['rotate_deg']} applied" if mapped else "with no mapping")
    return (f"{part['mpn']}, the preview model's own pin 1 mark at our pad 1 {how} on all four copies "
            f"({refs[0]}-{refs[-1]}: top and bottom, 0 and 37 deg), {angles} deg counter-clockwise from pad 1's "
            f"direction seen from the top (measurements.json, limit 20). What this is not: a check of the part's "
            f"identity, which is the map row's.")


def build() -> list[dict]:
    mapped = {}
    for row in json.loads(MAP.read_text(encoding="utf-8"))["rows"]:
        mapped.setdefault((row["mfr"], row["mpn"]), set()).add(row["pn"])
    out = []
    for record, (prefix, wanted) in RECORDS.items():
        folder = PREVIEWS / record
        for part in json.loads((folder / "readings.json").read_text(encoding="utf-8"))["parts"]:
            if part["reading"] != wanted or part["pn"] not in mapped.get((part["mfr"], part["mpn"]), set()):
                continue
            mapping = part.get("mapping", {"dx_mm": 0, "dy_mm": 0, "rotate_deg": 0})
            exception = None
            if mapping["rotate_deg"]:
                exception = (f"Per-part mapping: the supplier's model for {part['pn']} is drawn a "
                             f"{'half' if mapping['rotate_deg'] == 180 else 'quarter'} turn against this land, "
                             f"its pin 1 mark away from pad 1 with no mapping on all four copies "
                             f"({part['candidate_record']})")
            out.append({"id": f"{prefix}-{LAND[2].lower()}-{part['pn'].lower()}", "package": LAND[0],
                        "package_version": LAND[1], "footprint": LAND[2], "supplier": "lcsc", "pn": part["pn"],
                        "mapping": dict(mapping), "measured": dict(mapping), "exception": exception,
                        "evidence": {"method": "preview", "date": "2026-11-23",
                                     "record": str((folder / "README.md").relative_to(ROOT)),
                                     "note": note(part, bool(mapping["rotate_deg"]))},
                        "evidence_ref": None})
    return out


def table() -> str:
    frames = json.loads(FRAMES.read_text(encoding="utf-8"))
    prefixes = tuple(prefix + "-" for prefix, _ in RECORDS.values())
    # The block stays where it is (w3_record.py moves its own rows to the end), or goes last when new.
    ours = [i for i, row in enumerate(frames["rows"]) if row["id"].startswith(prefixes)]
    rest = [row for row in frames["rows"] if not row["id"].startswith(prefixes)]
    at = ours[0] if ours else len(rest)
    rows = rest[:at] + build() + rest[at:]
    return json.dumps(dict(frames, rows=rows), indent=2, ensure_ascii=False) + "\n"


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true")
    opts = ap.parse_args(argv)
    text = table()
    if opts.check:
        if FRAMES.read_text(encoding="utf-8") != text:
            print("sot23_frames.py: jlc_frames.json is not what the readings build", file=sys.stderr)
            return 1
        return 0
    FRAMES.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
