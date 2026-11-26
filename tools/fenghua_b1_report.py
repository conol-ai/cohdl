#!/usr/bin/env python3
"""Write tools/passive_data/fenghua_b1_dry_run.json: Fenghua MLCCs in the B1 snapshot
against the library's capacitors.

``python3 -I tools/fenghua_b1_report.py [--check]``. Nothing is fetched.

DRY RUN. gen_passive.py does not read the file this writes, and this report
writes no part, alternate, map row or binding. It answers one question: which
Fenghua MLCC part numbers that JLCPCB stocks as Basic or Preferred are the
same capacitor as a part the library already has? Those part numbers could
become alternates once two things are settled:

- the library's name for the manufacturer, which is a maintainer decision;
- a reviewed alias from the snapshot's spelling to that name, cited by an
  issue.

The B1 snapshot (tools/sourcing_data/jlc_catalog/2026-10-30/) spells the
manufacturer "FH (Guangdong Fenghua Advanced Tech)". Its 57 MLCC part numbers
are decoded with the "How To Order" rule of Fenghua's General Series MLCC
specification, 1005~1812 TYPE (27 pages). The rule is transcribed, with the
document's URL and SHA-256, in
tools/passive_data/fenghua_general_mlcc.evidence.md. The code reads, in
order: size code, dielectric code, capacitance code, tolerance code,
rated-voltage code, termination (N) and packing (T or D). The capacitance
and voltage codes give two significant digits and a count of zeros, with R
marking the decimal point; capacitance is in pF and voltage in V.

A part number matches when it decodes to a combination the library offers
(size, rated volts, dielectric, farads), with the tolerance the library pairs
with that dielectric (C0G J = +/-5%, X7R and X5R K = +/-10%). The same rule is
used for Samsung in samsung_b1_alternates.py, which gives the library's
combinations. Every other part number is listed with one reason, checked in
this order:
- "not decoded";
- the wrong tolerance for its dielectric;
- a combination whose Yageo primary Yageo will not confirm;
- a combination the library does not offer.
Part numbers that are not MLCCs (CMI inductors) are listed apart.
"""

from __future__ import annotations

import argparse
from decimal import Decimal
import json
from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import samsung_b1_alternates  # noqa: E402

CATALOG = ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"
OUT = ROOT / "tools/passive_data/fenghua_b1_dry_run.json"
CATALOG_MFR = "FH (Guangdong Fenghua Advanced Tech)"
SIZES = ("01005", "0201", "0402", "0603", "0805", "1206", "1210", "1808", "1812")
# Specification p. 2, dielectric code. Only the three the library has are decoded to it.
DIELECTRIC = {"CG": "C0G", "X": "X5R", "B": "X7R"}
OTHER_DIELECTRIC = ("BS", "BT", "DS", "DT")
TOLERANCE = samsung_b1_alternates.TOLERANCE
MPN = re.compile(r"^(1005|0201|0402|0603|0805|1206|1210|1808|1812)(CG|BS|BT|DS|DT|X|B)"
                 r"(\d{3}|\dR\d|R\d{2})([ABCDFGJKM])(\d{3}|\dR\d)N([TD])$")


def _code(code: str) -> Decimal:
    """Two significant digits and a count of zeros; R is the decimal point."""
    if "R" in code:
        return Decimal(code.replace("R", "."))
    return Decimal(code[:2]) * Decimal(10) ** int(code[2])


def decode(mpn: str) -> tuple[str, float, str, Decimal, str] | None:
    """(EIA size, volts, dielectric, farads, tolerance code), or None."""
    found = MPN.match(mpn)
    if not found or found.group(2) not in DIELECTRIC:
        return None
    size, diel, cap, tol, volt, _ = found.groups()
    size = "01005" if size == "1005" else size
    return size, float(_code(volt)), DIELECTRIC[diel], _code(cap) * Decimal("1e-12"), tol


def build() -> dict:
    offered, withdrawn = samsung_b1_alternates.library_points()
    matched, unmatched, other = {}, [], []
    for page in sorted(json.loads(CATALOG.read_text(encoding="utf-8")), key=lambda p: int(p["c"][1:])):
        if page["mfr"] != CATALOG_MFR:
            continue
        if not MPN.match(page["mpn"]) and not re.match(r"^\d{4}(CG|BS|BT|DS|DT|X|B)", page["mpn"]):
            other.append({"pn": page["c"], "mpn": page["mpn"], "package": page["package"],
                          "reason": "not an MLCC part number"})
            continue
        decoded = decode(page["mpn"])
        if decoded is None:
            unmatched.append({"pn": page["c"], "mpn": page["mpn"], "reason": "not decoded"})
            continue
        size, volts, diel, farads, tol = decoded
        if size != page["package"]:
            raise SystemExit(f"{page['c']}: decodes to {size}, the page reads {page['package']}")
        point = (size, volts, diel, f"{float(farads):.6g}")
        key = offered.get(point)
        if key is None or tol != TOLERANCE[diel]:
            reason = (f"tolerance {tol}, the library pairs {TOLERANCE[diel]}" if tol != TOLERANCE[diel]
                      else "no library part: its Yageo primary is unconfirmed" if point in withdrawn
                      else "no library part")
            unmatched.append({"pn": page["c"], "mpn": page["mpn"], "reason": reason})
            continue
        if key in matched:
            raise SystemExit(f"{key}: both {matched[key]['mpn']} and {page['mpn']}")
        matched[key] = {"mpn": page["mpn"], "pn": page["c"], "category": page["category"]}
    return {
        "_provenance": {
            "status": "dry run: no part, alternate, map row or binding is written from this file",
            "source": f"B1 snapshot {CATALOG.relative_to(ROOT)} (confirmed JLCPCB pages), rows whose "
                      f"manufacturer reads '{CATALOG_MFR}'",
            "rules": "tools/passive_data/fenghua_general_mlcc.evidence.md (How To Order, specification p. 2)",
            "meaning": "key is '<EIA size>|<rated volts>|<dielectric>|<farads>', as in samsung_b1.json; value is "
                       "a Fenghua part number JLCPCB stocks as Basic or Preferred for that exact combination, with "
                       "the tolerance the library pairs with the dielectric",
            "blocked_on": "the library's manufacturer name (maintainer) and a reviewed alias with its issue",
            "generator": "tools/fenghua_b1_report.py",
        },
        "matched": dict(sorted(matched.items())),
        "unmatched": unmatched,
        "not_mlcc": other,
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    text = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    if args.check:
        if not OUT.is_file() or OUT.read_text(encoding="utf-8") != text:
            print(f"fenghua_b1_report.py: {OUT} is stale", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text, encoding="utf-8")
    report = json.loads(text)
    print(f"{len(report['matched'])} matched, {len(report['unmatched'])} unmatched, "
          f"{len(report['not_mlcc'])} not MLCC")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
