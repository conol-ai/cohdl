#!/usr/bin/env python3
"""Write tools/passive_data/samsung_b1.json: Samsung MLCCs in the B1 snapshot that fit a library part.

``python3 -I tools/samsung_b1_alternates.py [--check]``. Nothing is fetched.

The B1 snapshot (tools/sourcing_data/jlc_catalog/2026-10-30/) holds the JLCPCB
Basic and Preferred parts. Its Samsung CL part numbers are decoded with the
part-numbering rules of Samsung's own catalog, which the package ships
(lib/passive/docs/samsung_mlcc_catalog_2025-12.pdf, pp. 6-7): size code,
dielectric code, capacitance code (two significant digits and a count of
zeros, in pF; R marks the decimal point), tolerance code, rated-voltage code.

A part number is kept only when it decodes to a combination the library
already offers (the same key as samsung_cl.json: size, rated volts,
dielectric, farads) with the tolerance the library pairs with that dielectric
(C0G J = +/-5%, X7R and X5R K = +/-10%). Each one is a real, stocked part:
its JLCPCB page was confirmed in the B1 run. gen_passive.py adds it as an
alternate; a part whose combination the library does not offer is listed
under "unmatched" and left for new parts (plan W4).
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
import gen_passive  # noqa: E402

CATALOG = ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"
OUT = ROOT / "tools/passive_data/samsung_b1.json"
SIZE = {"03": "0201", "05": "0402", "10": "0603", "21": "0805", "31": "1206", "32": "1210"}
DIELECTRIC = {"C": "C0G", "A": "X5R", "B": "X7R"}
# Catalog p. 6. Codes the library has no voltage for (such as G) leave a part undecoded.
VOLTS = {"S": 2.5, "R": 4, "Q": 6.3, "P": 10, "O": 16, "A": 25, "L": 35, "B": 50, "C": 100, "D": 200, "E": 250}
TOLERANCE = {"C0G": "J", "X5R": "K", "X7R": "K"}
MPN = re.compile(r"^CL(03|05|10|21|31|32)([A-Z])(\d{3}|\dR\d|R\d\d)([A-Z])([A-Z])")


def decode(mpn: str) -> tuple[str, float, str, Decimal, str] | None:
    """(size, volts, dielectric, farads, tolerance code), or None."""
    found = MPN.match(mpn)
    if not found or found.group(2) not in DIELECTRIC or found.group(5) not in VOLTS:
        return None
    size, diel, cap, tol, volt = found.groups()
    pf = Decimal(cap.replace("R", ".")) if "R" in cap else Decimal(cap[:2]) * Decimal(10) ** int(cap[2])
    return SIZE[size], VOLTS[volt], DIELECTRIC[diel], pf * Decimal("1e-12"), tol


def build() -> dict:
    available, verified = gen_passive.load_mlcc(), gen_passive.load_verified()
    offered, withdrawn = {}, set()
    for key, values in available.items():
        size, volts, diel = key.split("|")
        name, process, tol_letter, _ = gen_passive.C_DIELECTRIC[diel]
        for farads in values:
            point = (size, float(volts), diel, f"{farads:.6g}")
            # gen_passive.py emits no part whose Yageo primary Yageo will not confirm.
            yageo = (f"CC{size}{tol_letter}R{name}{gen_passive.C_VOLT_CODE[float(volts)]}{process}"
                     f"{gen_passive.eia_cap_code(farads)}")
            if verified.get(yageo) is False:
                withdrawn.add(point)
                continue
            offered[point] = f"{size}|{float(volts):g}|{diel}|{farads:.6g}"
    parts, unmatched = {}, []
    for page in sorted(json.loads(CATALOG.read_text(encoding="utf-8")), key=lambda p: int(p["c"][1:])):
        if not page["mfr"].startswith("Samsung") or not page["mpn"].startswith("CL"):
            continue
        decoded = decode(page["mpn"])
        if decoded is None:
            unmatched.append({"pn": page["c"], "mpn": page["mpn"], "reason": "not decoded"})
            continue
        size, volts, diel, farads, tol = decoded
        point = (size, float(volts), diel, f"{float(farads):.6g}")
        key = offered.get(point)
        if key is None or tol != TOLERANCE[diel]:
            reason = ("no library part: its Yageo primary is unconfirmed" if point in withdrawn
                      else "no library part" if key is None else f"tolerance {tol}, the library pairs {TOLERANCE[diel]}")
            unmatched.append({"pn": page["c"], "mpn": page["mpn"], "reason": reason})
            continue
        if key in parts:
            raise SystemExit(f"{key}: both {parts[key]['mpn']} and {page['mpn']}")
        parts[key] = {"mpn": page["mpn"], "pn": page["c"], "category": page["category"]}
    return {
        "_provenance": {
            "source": "B1 snapshot tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json (confirmed JLCPCB pages)",
            "rules": "lib/passive/docs/samsung_mlcc_catalog_2025-12.pdf pp. 6-7 (part numbering)",
            "meaning": "key is '<EIA size>|<rated volts>|<dielectric>|<farads>', as in samsung_cl.json; value is a "
                       "Samsung part number JLCPCB stocks as Basic or Preferred for that exact combination, with the "
                       "tolerance the library pairs with the dielectric",
            "generator": "tools/samsung_b1_alternates.py",
        },
        "parts": dict(sorted(parts.items())),
        "unmatched": unmatched,
    }


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args(argv)
    text = json.dumps(build(), indent=1, ensure_ascii=False) + "\n"
    if args.check:
        if not OUT.is_file() or OUT.read_text(encoding="utf-8") != text:
            print(f"samsung_b1_alternates.py: {OUT} is stale", file=sys.stderr)
            return 1
        return 0
    OUT.write_text(text, encoding="utf-8")
    report = json.loads(text)
    print(f"{len(report['parts'])} alternates, {len(report['unmatched'])} unmatched")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
