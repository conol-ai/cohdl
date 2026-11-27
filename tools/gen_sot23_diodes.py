#!/usr/bin/env python3
"""Generate the SOT-23 switching, Schottky and Zener diodes of ``lib/diode``.

``python3 -I tools/gen_sot23_diodes.py [--check]`` writes ``lib/diode/src/sot23.cohdl``
and its evidence record ``lib/diode/docs/sot23.evidence.md`` from
``tools/discrete_data/sot23_diodes.json``, a transcription of the manufacturer
datasheets of the SOT-23 diodes in the B1 snapshot. With ``--check`` it writes
nothing and fails if a committed file differs.

Every part sits on ``sot::SOT3P190X290X130N``. Its device is chosen by the
diodes its own sheet draws between the numbered SOT-23 pins: one diode
(anode 1, cathode 3, pin 2 not connected), two in series (1 to 3, 3 to 2),
two with a common cathode at 3, or two with a common anode at 3. The tool
fails closed when a part's diodes are not the arrangement its configuration
names, when an MPN or part name repeats, or when a rating is missing or not
positive. Nothing is fetched, and no supplier binding is written: a polarized
part's binding arrives with its own preview reading.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "tools"))
from gen_sot23 import LAND, MFR_SHORT, DataError, literal, primary  # noqa: E402

DATA = ROOT / "tools" / "discrete_data" / "sot23_diodes.json"
OUT = ROOT / "lib" / "diode" / "src" / "sot23.cohdl"
EVIDENCE = ROOT / "lib" / "diode" / "docs" / "sot23.evidence.md"
SHORT = dict(MFR_SHORT, Nexperia="NEXPERIA")

# config -> (the diodes as [anode pin, cathode pin], pins not connected, device pins)
CONFIGS = {
    "single": ([["1", "3"]], ["2"], [("Anode", "1"), ("Cathode", "3")]),
    "series": ([["1", "3"], ["3", "2"]], [], [("A1", "1"), ("K2", "2"), ("K1A2", "3")]),
    "common_cathode": ([["1", "3"], ["2", "3"]], [], [("A1", "1"), ("A2", "2"), ("K", "3")]),
    "common_anode": ([["3", "1"], ["3", "2"]], [], [("K1", "1"), ("K2", "2"), ("A", "3")]),
}
KIND = {"switching": "Diode", "schottky": "Schottky", "zener": "Zener"}
SUFFIX = {"single": "", "series": "PairSeries", "common_cathode": "PairCommonCathode",
          "common_anode": "PairCommonAnode"}
WHAT = {
    "single": "one diode: pin 1 anode, pin 3 cathode, pin 2 not connected",
    "series": "two diodes in series: pin 1 anode of the first, pin 3 its cathode and the second's anode, "
              "pin 2 the second's cathode",
    "common_cathode": "two diodes with a common cathode: pins 1 and 2 the anodes, pin 3 the cathodes",
    "common_anode": "two diodes with a common anode: pins 1 and 2 the cathodes, pin 3 the anodes",
}
KIND_TEXT = {"switching": "switching diode", "schottky": "Schottky diode", "zener": "Zener diode"}


def device(row: dict) -> str:
    return KIND[row["kind"]] + SUFFIX[row["config"]] + "_SOT23"


def generics(kind: str) -> str:
    return "<VZ: Voltage, P: Power>" if kind == "zener" else "<VR: Voltage, IF: Current>"


def args(row: dict) -> str:
    r = row["ratings"]
    if row["kind"] == "zener":
        return f"{literal(r['vz_nom_v'], 'V')}, {literal(r['pd_mw'], 'mW')}"
    return f"{literal(r['vr_v'], 'V')}, {literal(r['if_ma'], 'mA')}"


def part_name(row: dict) -> str:
    return "D_%s_%s" % (SHORT[row["mfr"]], re.sub(r"[^A-Za-z0-9]+", "_", row["mpn"]).strip("_"))


def comment(row: dict) -> str:
    r = row["ratings"]
    if row["kind"] == "zener":
        text = (f"VZ {r['vz_min_v']}-{r['vz_max_v']}V (nom {r['vz_nom_v']}V) at IZT {r['izt_ma']}mA; "
                f"ZZT {r['zzt_ohm']}ohm; PD {r['pd_mw']}mW")
    else:
        text = f"VR {r['vr_v']}V, IF {r['if_ma']}mA, VF {r['vf_v']}V max at {r['vf_at_ma']}mA"
    if row.get("marking"):
        text += f"; marking {row['marking']}"
    return f"// {row['mpn']} ({row['c']}): {text}"


def check(rows: list[dict]) -> None:
    seen, names = set(), set()
    for row in rows:
        if row["kind"] not in KIND or row["config"] not in CONFIGS:
            raise DataError(f"gen_sot23_diodes.py: {row['mpn']}: unknown kind or configuration")
        if row["kind"] == "zener" and row["config"] != "single":
            raise DataError(f"gen_sot23_diodes.py: {row['mpn']}: a Zener here is a single diode")
        diodes, nc, _ = CONFIGS[row["config"]]
        if row["diodes"] != diodes or sorted(row["nc"]) != nc:
            raise DataError(f"gen_sot23_diodes.py: {row['mpn']}: diodes {row['diodes']} (nc {row['nc']}) are not "
                            f"the {row['config']} arrangement {diodes} (nc {nc})")
        key = (row["mfr"], row["mpn"])
        if key in seen:
            raise DataError(f"gen_sot23_diodes.py: {row['mfr']} {row['mpn']} appears twice")
        seen.add(key)
        if part_name(row) in names:
            raise DataError(f"gen_sot23_diodes.py: part name {part_name(row)} repeats")
        names.add(part_name(row))
        if not re.fullmatch(r"[0-9a-f]{64}", row["datasheet"]["sha256"]):
            raise DataError(f"gen_sot23_diodes.py: {row['mpn']}: no datasheet checksum")
        args(row)




def source(rows: list[dict]) -> str:
    out = ["// SOT-23 switching, Schottky and Zener diodes of the B1 snapshot (library plan",
           "// W4, item 3d).",
           "//",
           "// GENERATED by tools/gen_sot23_diodes.py from tools/discrete_data/sot23_diodes.json",
           "// — do not hand-edit. Each device is the arrangement of diodes the part's own",
           "// sheet draws between the numbered SOT-23 pins; the sheets and their",
           "// checksums are in docs/sot23.evidence.md. Every part sits on",
           "// sot::SOT3P190X290X130N, the audited SOT-23-3 land on the assembler's frame",
           "// (pin 1 lower right, pins 1 and 2 the +x column, pin 3 alone at -x). A",
           "// polarized part carries a supplier binding only with its own preview",
           "// reading: the clause comes from tools/sourcing_data/lcsc_map.json and the",
           "// part number's row in tools/sourcing_data/jlc_frames.json.",
           "",
           "// Two diodes in one package: neither a single anode nor a single cathode, so",
           "// not `Polarized`; the trait carries the designator prefix.",
           "pub trait DiodePair {",
           '    designator_prefix: "D"',
           "}",
           ""]
    used = []
    for row in rows:
        if device(row) not in [d for d, _ in used]:
            used.append((device(row), row))
    for name, row in used:
        _, nc, pins = CONFIGS[row["config"]]
        what = KIND_TEXT[row["kind"]].capitalize() + ("" if row["config"] == "single" else " pair")
        out += [f"// {what}, SOT-23, {WHAT[row['config']]}.",
                '#[doc("docs/sot23.evidence.md")]', f"pub device {name}{generics(row['kind'])} {{", "    pins {"]
        out += [f"        required {pin}: {pad} [passive]" for pin, pad in pins]
        out += [f"        optional NC: {pad} [passive]" for pad in nc]
        out += ["    }", "}", ""]
        if row["config"] == "single":
            out += [f"impl Polarized for {name} {{", "    pins {", "        Anode: Anode",
                    "        Cathode: Cathode", "    }", "}", "", f"impl Diode for {name} {{}}", ""]
        else:
            out += [f"impl DiodePair for {name} {{}}", ""]
    for name, _ in used:
        members = [r for r in rows if device(r) == name]
        out += [f"// ---- {name}: {len(members)} parts ----", ""]
        for row in members:
            out += [comment(row), '#[doc("docs/sot23.evidence.md")]',
                    f"pub part {part_name(row)}: {name}<{args(row)}> {{", *primary(row, "diode"), "}", ""]
    return "\n".join(out)


def evidence(rows: list[dict]) -> str:
    out = ["# SOT-23 switching, Schottky and Zener diodes: datasheet evidence", "",
           "Generated by `tools/gen_sot23_diodes.py` from `tools/discrete_data/sot23_diodes.json`; do not edit.", "",
           "Each part's sheet was fetched on 2026-11-23 through the LCSC datasheet link its B1 snapshot page "
           "(2026-10-30) gives, and the diodes between its pins read from the sheet's numbered SOT-23 diagram; "
           "a Zener's voltage is its row of the family sheet's electrical characteristics. The PDFs are not "
           "committed; the SHA-256 below identifies each one.", "",
           "| Part | MPN | Manufacturer | Snapshot C number | Device | Read from | PDF SHA-256 | Bytes |",
           "|---|---|---|---|---|---|---|---:|"]
    for row in rows:
        d = row["datasheet"]
        out.append(f"| `{part_name(row)}` | {row['mpn']} | {row['mfr']} | {row['c']} | `{device(row)}` | "
                   f"{d['pinout']} | `{d['sha256']}` | {d['bytes']} |")
    notes = [r for r in rows if r.get("notes")]
    if notes:
        out += ["", "## Notes from the sheets", ""]
        out += [f"- {r['mpn']} ({r['c']}): {r['notes']}" for r in notes]
    out += ["", "## Sources", ""]
    out += [f"- {r['mpn']}: <{r['datasheet']['url']}> (fetched {r['datasheet']['fetched']})" for r in rows]
    return "\n".join(out) + "\n"


def outputs() -> dict[Path, str]:
    rows = json.loads(DATA.read_text(encoding="utf-8"))["parts"]
    check(rows)
    return {OUT: source(rows), EVIDENCE: evidence(rows)}


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
        print("gen_sot23_diodes.py: not what the data builds: " + ", ".join(map(str, stale)), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
