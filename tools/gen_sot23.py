#!/usr/bin/env python3
"""Generate the SOT-23 bipolar transistors and MOSFETs of ``lib/transistor`` and ``lib/mosfet``.

``python3 -I tools/gen_sot23.py [--check]`` writes ``lib/transistor/src/sot23.cohdl``,
``lib/mosfet/src/sot23.cohdl`` and their evidence records
``lib/{transistor,mosfet}/docs/sot23.evidence.md`` from
``tools/discrete_data/sot23_bjt_mos.json``, a transcription of the manufacturer
datasheets of the SOT-23 transistors and MOSFETs in the B1 snapshot. With
``--check`` it writes nothing and fails if a committed file differs.

Every part sits on ``sot::SOT3P190X290X130N``, the audited SOT-23-3 land authored
on the assembler's frame. Each pin assignment is the one the part's own sheet
prints on its numbered SOT-23 diagram; the tool does not assume one. It fails
closed when a part's pins are not one of the arrangements a device here
declares, when its kind and pins disagree, when an MPN or part name repeats,
or when a rating is missing or not positive. Nothing is fetched, and no
supplier binding is written: a polarized part's binding arrives with its own
preview reading.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
from sourcing.bindings import Bindings, avl_entry  # noqa: E402

DATA = ROOT / "tools" / "discrete_data" / "sot23_bjt_mos.json"
LAND = "sot::SOT3P190X290X130N"
WIDTH = 100  # the line width `cohdl fmt` wraps at

# kind -> (package, device, the device's pins as {pad: (name, role)}, the sheet's letters per pad)
DEVICES = {
    "npn": ("transistor", "NPN_SOT23", {"1": ("B", "input"), "2": ("E", "passive"), "3": ("C", "passive")},
            {"1": "B", "2": "E", "3": "C"}),
    "pnp": ("transistor", "PNP_SOT23", {"1": ("B", "input"), "2": ("E", "passive"), "3": ("C", "passive")},
            {"1": "B", "2": "E", "3": "C"}),
    "nmos": ("mosfet", "NMOS_SOT23", {"1": ("Gate", "input"), "2": ("Source", "passive"), "3": ("Drain", "passive")},
             {"1": "G", "2": "S", "3": "D"}),
    "pmos": ("mosfet", "PMOS_SOT23", {"1": ("Gate", "input"), "2": ("Source", "passive"), "3": ("Drain", "passive")},
             {"1": "G", "2": "S", "3": "D"}),
}
GENERICS = {"transistor": "<VCEO: Voltage, IC: Current, PD: Power>", "mosfet": "<VDS: Voltage, ID: Current>"}
TRAIT = {"transistor": "BipolarTransistor", "mosfet": "Transistor"}
MFR_SHORT = {"Zhuhai Hongjiacheng Technology": "HJC", "Jiangsu Changjing Electronics Technology": "JSCJ",
             "Vishay": "VISHAY"}
TITLE = {"transistor": "bipolar transistors", "mosfet": "MOSFETs"}
DESCRIBE = {
    "NPN_SOT23": "NPN bipolar transistor, SOT-23: pin 1 base, 2 emitter, 3 collector. VCEO, IC and PD are the "
                 "sheet's absolute maximum ratings",
    "PNP_SOT23": "PNP bipolar transistor, SOT-23: pin 1 base, 2 emitter, 3 collector. VCEO, IC and PD are the "
                 "magnitudes of the sheet's absolute maximum ratings",
    "NMOS_SOT23": "N-channel MOSFET, SOT-23: pin 1 gate, 2 source, 3 drain. VDS and ID are the sheet's absolute "
                  "maximum ratings",
    "PMOS_SOT23": "P-channel MOSFET, SOT-23: pin 1 gate, 2 source, 3 drain. VDS and ID are the magnitudes of the "
                  "sheet's absolute maximum ratings",
}


class DataError(SystemExit):
    pass


def literal(value, unit: str) -> str:
    """A CoHDL literal: 0.115 A -> '115mA', 4.2 A -> '4.2A', 300 mW -> '300mW'."""
    v = abs(Decimal(str(value)))
    if v <= 0:
        raise DataError(f"gen_sot23.py: non-positive rating {value!r}")
    if unit == "A" and v < 1:
        v, unit = v * 1000, "mA"
    if unit == "mA" and v >= 1000 and v % 1000 == 0:
        v, unit = v / 1000, "A"
    if unit == "mW" and v >= 1000 and v % 1000 == 0:
        v, unit = v / 1000, "W"
    return format(v.normalize(), "f") + unit


def part_name(row: dict) -> str:
    mpn = re.sub(r"\(RANGE:[^)]*\)", "", row["mpn"])
    return "Q_%s_%s" % (MFR_SHORT[row["mfr"]], re.sub(r"[^A-Za-z0-9]+", "_", mpn).strip("_"))


def args(row: dict) -> str:
    r = row["ratings"]
    if row["kind"] in ("npn", "pnp"):
        return f"{literal(r['vceo_v'], 'V')}, {literal(r['ic_ma'], 'mA')}, {literal(r['pd_mw'], 'mW')}"
    return f"{literal(r['vds_v'], 'V')}, {literal(r['id_a'], 'A')}"


def comment(row: dict) -> str:
    r = row["ratings"]
    if row["kind"] in ("npn", "pnp"):
        text = f"VCEO {r['vceo_v']}V, IC {r['ic_ma']}mA, PD {r['pd_mw']}mW"
    else:
        rds = "; ".join(f"{x['max_ohm']}ohm max at VGS {x['vgs_v']}V" for x in r["rds_on"])
        text = f"VDS {r['vds_v']}V, VGS {r['vgs_max_v']}V max, ID {r['id_a']}A; RDS(on) {rds}"
        if row["gate_esd"]:
            text += "; gate protection diode"
    if row.get("marking"):
        text += f"; marking {row['marking']}"
    return f"// {row['mpn']} ({row['c']}): {text}; pinout {row['datasheet']['pinout']}"


_BINDINGS: list[Bindings] = []


def primary(row: dict, package: str) -> list[str]:
    """The primary entry as `cohdl fmt` writes it, with the part number's supplier clause when
    tools/sourcing_data/lcsc_map.json maps it and jlc_frames.json holds its row on this land."""
    if not _BINDINGS:
        _BINDINGS.append(Bindings(ROOT))
    manifest = ROOT / "lib" / package / "cohdl.toml"
    return avl_entry(_BINDINGS[0], manifest, "primary", row["mfr"], row["mpn"], LAND).rstrip("\n").split("\n")


def check(rows: list[dict]) -> None:
    seen, names = set(), set()
    for row in rows:
        if row["kind"] not in DEVICES:
            raise DataError(f"gen_sot23.py: {row['mpn']}: unknown kind {row['kind']!r}")
        letters = DEVICES[row["kind"]][3]
        if row["pins"] != letters:
            raise DataError(f"gen_sot23.py: {row['mpn']}: pins {row['pins']} are not {letters}")
        key = (row["mfr"], row["mpn"])
        if key in seen:
            raise DataError(f"gen_sot23.py: {row['mfr']} {row['mpn']} appears twice")
        seen.add(key)
        name = part_name(row)
        if name in names:
            raise DataError(f"gen_sot23.py: part name {name} repeats")
        names.add(name)
        if not re.fullmatch(r"[0-9a-f]{64}", row["datasheet"]["sha256"]):
            raise DataError(f"gen_sot23.py: {row['mpn']}: no datasheet checksum")
        args(row)


def source(package: str, rows: list[dict]) -> str:
    out = [f"// SOT-23 {TITLE[package]} of the B1 snapshot (library plan W4, item 3d).",
           "//",
           "// GENERATED by tools/gen_sot23.py from tools/discrete_data/sot23_bjt_mos.json",
           "// — do not hand-edit. Each pin assignment is the one the part's own sheet",
           "// prints on its numbered SOT-23 diagram; the sheets and their checksums are",
           "// in docs/sot23.evidence.md. Every part sits on sot::SOT3P190X290X130N, the",
           "// audited SOT-23-3 land on the assembler's frame (pin 1 lower right, pins 1",
           "// and 2 the +x column, pin 3 alone at -x). A polarized part carries a",
           "// supplier binding only with its own preview reading: the clause comes from",
           "// tools/sourcing_data/lcsc_map.json and the part number's row in",
           "// tools/sourcing_data/jlc_frames.json, as in tools/gen_zener.py.",
           ""]
    for kind in [k for k in DEVICES if DEVICES[k][0] == package]:
        _, device, pins, _ = DEVICES[kind]
        out += [f"// {DESCRIBE[device]}.", '#[doc("docs/sot23.evidence.md")]',
                f"pub device {device}{GENERICS[package]} {{", "    pins {"]
        out += [f"        required {name}: {pad} [{role}]" for pad, (name, role) in pins.items()]
        out += ["    }", "}", "", f"impl {TRAIT[package]} for {device} {{}}", ""]
    for kind in [k for k in DEVICES if DEVICES[k][0] == package]:
        device = DEVICES[kind][1]
        members = [r for r in rows if r["kind"] == kind]
        if not members:
            continue
        out += [f"// ---- {device}: {len(members)} parts ----", ""]
        for row in members:
            out += [comment(row), '#[doc("docs/sot23.evidence.md")]',
                    f"pub part {part_name(row)}: {device}<{args(row)}> {{",
                    *primary(row, package), "}", ""]
    return "\n".join(out)


def evidence(package: str, rows: list[dict]) -> str:
    out = [f"# SOT-23 {TITLE[package]}: datasheet evidence", "",
           "Generated by `tools/gen_sot23.py` from `tools/discrete_data/sot23_bjt_mos.json`; do not edit.", "",
           "Each part's sheet was fetched on 2026-11-23 through the LCSC datasheet link its B1 snapshot page "
           "(2026-10-30) gives, and its pin assignment read from the sheet's numbered SOT-23 diagram. The PDFs "
           "are not committed; the SHA-256 below identifies each one. Every part here uses the same arrangement "
           "per device; the tool refuses a part whose sheet shows another.", "",
           "| Part | MPN | Manufacturer | Snapshot C number | Pinout read from | PDF SHA-256 | Bytes |",
           "|---|---|---|---|---|---|---:|"]
    for row in rows:
        d = row["datasheet"]
        out.append(f"| `{part_name(row)}` | {row['mpn']} | {row['mfr']} | {row['c']} | {d['pinout']} | "
                   f"`{d['sha256']}` | {d['bytes']} |")
    notes = [r for r in rows if r.get("notes")]
    if notes:
        out += ["", "## Notes from the sheets", ""]
        out += [f"- {r['mpn']} ({r['c']}): {r['notes']}" for r in notes]
    out += ["", "## Sources", ""]
    out += [f"- {r['mpn']}: <{r['datasheet']['url']}> (fetched {r['datasheet']['fetched']})" for r in rows]
    return "\n".join(out) + "\n"


def outputs() -> dict[Path, str]:
    data = json.loads(DATA.read_text(encoding="utf-8"))
    rows = data["parts"]
    check(rows)
    files = {}
    for package in ("transistor", "mosfet"):
        members = [r for r in rows if DEVICES[r["kind"]][0] == package]
        files[ROOT / "lib" / package / "src" / "sot23.cohdl"] = source(package, members)
        files[ROOT / "lib" / package / "docs" / "sot23.evidence.md"] = evidence(package, members)
    return files


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true")
    opts = ap.parse_args(argv)
    stale = []
    for path, text in outputs().items():
        if opts.check:
            if not path.exists() or path.read_text(encoding="utf-8") != text:
                stale.append(path.relative_to(ROOT))
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text, encoding="utf-8")
    if stale:
        print("gen_sot23.py: not what the data builds: " + ", ".join(map(str, stale)), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
