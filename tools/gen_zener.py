#!/usr/bin/env python3
"""Generate the Hongjiacheng Zener diodes of ``lib/diode``.

``python3 -I tools/gen_zener.py [--check]`` writes two part files and their
evidence records:

- ``lib/diode/src/zener.cohdl`` and ``lib/diode/docs/hongjiacheng-zener.evidence.md``
  from ``tools/diode_data/hongjiacheng_zener.json``, a transcription of seven
  Hongjiacheng SOD-123 datasheets (BZT52C, BZT52B, MMSZ52xxB, MMSZ46xx, MM1Z..B,
  MM1Z and MM1W), on their common land ``sod::SOD3716X135N``; this file also
  declares the ``Zener`` device;
- ``lib/diode/src/zener_sod323_sod523.cohdl`` and
  ``lib/diode/docs/hongjiacheng-zener-sod323-sod523.evidence.md`` from
  ``tools/diode_data/hongjiacheng_zener_sod323.json``, five sheets (BZT52C..S,
  BZT52B..S, MM3Z and MMSZ52xxBS on ``sod::SOD2613X110N``, BZX584C on
  ``sod::SOD1608X70N``), where each series names its land.

With ``--check`` it writes nothing and fails if a committed file differs.

Every part is one row of a sheet's electrical-characteristics table, pad 1 the
cathode. The
device states the nominal Zener voltage and the sheet's rated power; each
part's comment carries the row as printed (VZ min/nom/max at IZT, ZZT where
the sheet gives it, IR at VR, marking, page). A row the data file lists under a
series' ``omitted`` (a sheet that prints one MPN twice with different values)
is not a part; the evidence record names it and why. Nothing is fetched. The tool fails closed when a row does
not parse, an MPN repeats, VZ is not min < nom < max, or an MPN that encodes
its voltage (BZT52C5V6, MM1Z5V6B, MM1W330) disagrees with the row's nominal.
No supplier binding is written: bindings arrive with preview evidence.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from decimal import Decimal
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
# `python3 -I` does not put the script's directory on sys.path.
sys.path.insert(0, str(ROOT / "tools"))
from sourcing.bindings import Bindings, avl_entry  # noqa: E402

MANIFEST = ROOT / "lib" / "diode" / "cohdl.toml"
DATA = ROOT / "tools" / "diode_data" / "hongjiacheng_zener.json"
OUT = ROOT / "lib" / "diode" / "src" / "zener.cohdl"
EVIDENCE = ROOT / "lib" / "diode" / "docs" / "hongjiacheng-zener.evidence.md"
DOC = "docs/hongjiacheng-zener.evidence.md"
DATA_SMALL = ROOT / "tools" / "diode_data" / "hongjiacheng_zener_sod323.json"
OUT_SMALL = ROOT / "lib" / "diode" / "src" / "zener_sod323_sod523.cohdl"
EVIDENCE_SMALL = ROOT / "lib" / "diode" / "docs" / "hongjiacheng-zener-sod323-sod523.evidence.md"
DOC_SMALL = "docs/hongjiacheng-zener-sod323-sod523.evidence.md"

# The MPN's own voltage code: BZT52C5V6 -> 5.6, BZT52C10 -> 10, MM1Z5V6B -> 5.6.
ENCODED = {
    "bzt52c": re.compile(r"BZT52C(\d+)(?:V(\d))?"),
    "bzt52b": re.compile(r"BZT52B(\d+)(?:V(\d))?"),
    "mm1z": re.compile(r"MM1Z(\d+)(?:V(\d))?B"),
    "mm1z_plain": re.compile(r"MM1Z(\d+)(?:V(\d))?"),
    "mm1w": re.compile(r"MM1W(\d+)(?:V(\d))?"),
    "mmsz": None,  # JEDEC 1N52xx numbering: the voltage is not in the name.
    "mmsz46": None,  # JEDEC 1N46xx numbering, likewise.
    "bzt52c_s": re.compile(r"BZT52C(\d+)(?:V(\d))?S"),
    "bzt52b_s": re.compile(r"BZT52B(\d+)(?:V(\d))?S"),
    "mm3z": re.compile(r"MM3Z(\d+)(?:V(\d))?"),
    "mmsz_s": None,  # JEDEC 1N52xx numbering with the SOD-323 "S" suffix.
    "bzx584c": re.compile(r"BZX584C(\d+)(?:V(\d))?"),
}


class DataError(SystemExit):
    pass


def literal(text: str, unit: str) -> str:
    """A CoHDL literal for a decimal from the sheet: '5.60' -> '5.6V', '10' -> '10V'."""
    value = Decimal(text).normalize()
    if value <= 0:
        raise DataError(f"gen_zener.py: non-positive value {text!r}")
    s = format(value, "f")
    return f"{s}{unit}"


def power(mw: int) -> str:
    return f"{mw // 1000}W" if mw % 1000 == 0 else f"{mw}mW"


def check_row(key: str, row: dict, seen: set[str]) -> None:
    mpn = row["mpn"]
    if mpn in seen:
        raise DataError(f"gen_zener.py: {mpn} appears twice")
    seen.add(mpn)
    lo, nom, hi = (Decimal(row[k]) for k in ("vz_min_v", "vz_nom_v", "vz_max_v"))
    if not lo < nom < hi:
        raise DataError(f"gen_zener.py: {mpn}: VZ {lo} / {nom} / {hi} is not min < nom < max")
    rx = ENCODED[key]
    if rx is not None:
        m = rx.fullmatch(mpn)
        if not m:
            raise DataError(f"gen_zener.py: {mpn} does not follow the {key} numbering")
        coded = Decimal(m[1] + ("." + m[2] if m[2] else ""))
        if coded != nom:
            raise DataError(f"gen_zener.py: {mpn} encodes {coded} V but its row says {nom} V")


def comment(row: dict) -> str:
    izt = f"{row['izt_ma']}mA" if "izt_ma" in row else f"{row['izt_ua']}uA"
    parts = [f"VZ {row['vz_min_v']}-{row['vz_max_v']}V (nom {row['vz_nom_v']}V) at IZT {izt}"]
    if "zzt_ohm" in row:
        parts.append(f"ZZT {row['zzt_ohm']}ohm")
    parts += [f"IR {row['ir_ua']}uA at {row['vr_v']}V", f"marking {row['marking']}", f"page {row['page']}"]
    return "// " + "; ".join(parts)


def render(data: dict, bindings: Bindings | None = None) -> str:
    bindings = Bindings(ROOT) if bindings is None else bindings
    out = [
        "// Zhuhai Hongjiacheng SOD-123 Zener diodes (library plan W4, item 3d).",
        "//",
        "// GENERATED by tools/gen_zener.py from tools/diode_data/hongjiacheng_zener.json",
        "// — do not hand-edit. Every part is one row of a manufacturer sheet's",
        "// electrical-characteristics table; the sheets and their checksums are in",
        f"// {DOC}.",
        "//",
        "// All seven sheets print the same SOD-123 outline and suggested pad layout,",
        "// the table `sod::SOD3716X135N` was drawn from. Pad 1 is the cathode (the",
        "// band end, at -x), pad 2 the anode. `Zener<VZ, P>` states the nominal Zener",
        "// voltage and the sheet's rated power; the tolerance, test current and",
        "// impedance differ between series, so each part's comment keeps its row.",
        "// A part carries a supplier binding only with its own preview reading: a",
        "// polarized part is outside W3 sampled inference. The clause comes from",
        "// tools/sourcing_data/lcsc_map.json and the part number's row in",
        "// tools/sourcing_data/jlc_frames.json, as in tools/gen_passive.py.",
        "",
        f'#[doc("{DOC}")]',
        "pub device Zener<VZ: Voltage, P: Power> {",
        "    pins {",
        "        required Cathode: 1 [passive]",
        "        required Anode: 2 [passive]",
        "    }",
        "}",
        "",
        "impl Polarized for Zener {",
        "    pins {",
        "        Anode: Anode",
        "        Cathode: Cathode",
        "    }",
        "}",
        "",
        "impl Diode for Zener {}",
    ]
    for series in data["series"]:
        out += series_parts(series, data["manufacturer"], data["package"]["footprint"], DOC, bindings)
    return "\n".join(out) + "\n"


def series_parts(series: dict, mfr: str, footprint: str, doc: str, bindings: Bindings) -> list[str]:
    """One series' parts, after its heading comment."""
    key = series["key"]
    seen: set[str] = set()
    if not series["parts"]:
        raise DataError(f"gen_zener.py: series {key} has no parts")
    out = ["", f"// ---- {series['title']}, {series['revision']}: PD {power(series['power_mw'])} ----"]
    for row in series["parts"]:
        check_row(key, row, seen)
        if row["mpn"] in {o["mpn"] for o in series.get("omitted", [])}:
            raise DataError(f"gen_zener.py: {row['mpn']} is both a part and omitted")
        args = f"{literal(row['vz_nom_v'], 'V')}, {power(series['power_mw'])}"
        # Laid out as `cohdl fmt` does, with the supplier clause when the part number has one.
        entry = avl_entry(bindings, MANIFEST, "primary", mfr, row["mpn"], footprint).rstrip("\n").split("\n")
        out += [
            "",
            comment(row),
            f'#[doc("{doc}")]',
            f"pub part D_{row['mpn']}: Zener<{args}> {{",
            *entry,
            "}",
        ]
    return out


LANDS = {"sod::SOD2613X110N": "SOD-323", "sod::SOD1608X70N": "SOD-523"}


def render_small(data: dict, bindings: Bindings | None = None) -> str:
    """The SOD-323 and SOD-523 parts: the `Zener` device of zener.cohdl, each series on the land it names."""
    bindings = Bindings(ROOT) if bindings is None else bindings
    out = [
        "// Zhuhai Hongjiacheng SOD-323 and SOD-523 Zener diodes (library plan, round 3).",
        "//",
        "// GENERATED by tools/gen_zener.py from tools/diode_data/hongjiacheng_zener_sod323.json",
        "// — do not hand-edit. Every part is one row of a manufacturer sheet's",
        "// electrical-characteristics table; the sheets and their checksums are in",
        f"// {DOC_SMALL}.",
        "//",
        "// The four SOD-323 sheets print the outline and suggested pad layout",
        "// `sod::SOD2613X110N` was drawn from, the SOD-523 sheet the one of",
        "// `sod::SOD1608X70N`. Pad 1 is the cathode (the band end, at -x). The device",
        "// is `Zener<VZ, P>` of zener.cohdl; each part's comment keeps its row. A part",
        "// carries a supplier binding only with its own preview reading, as in",
        "// zener.cohdl.",
    ]
    for series in data["series"]:
        footprint = series["package"]["footprint"]
        if footprint not in LANDS:
            raise DataError(f"gen_zener.py: series {series['key']} names an unknown land {footprint}")
        out += series_parts(series, data["manufacturer"], footprint, DOC_SMALL, bindings)
    return "\n".join(out) + "\n"


EVIDENCE_HEAD = """\
# Hongjiacheng SOD-123 Zener evidence record (parts)

- Library coverage: `Zener` and the 285 `D_<MPN>` parts in `src/zener.cohdl`,
  generated by `tools/gen_zener.py` from `tools/diode_data/hongjiacheng_zener.json`.
- Manufacturer: Zhuhai Hongjiacheng Technology Co., Ltd (`www.ichjc.com` in every
  sheet's header). No manufacturer-hosted copy was located, so these are the
  manufacturer's PDFs as served by LCSC, the fallback the maintainer approved for
  the BZT52C5V6 sheet `sod::SOD3716X135N` was drawn from
  (`lib/sod/docs/hongjiacheng-bzt52c5v6.evidence.md`). The PDFs are not
  redistributed.
- Retrieved: 2026-11-21, one viewer page and one PDF per sheet through
  `tools/sourcing/http.py` (2 s spacing, robots.txt honoured), in two runs
  (five sheets, then the MM1Z and MMSZ46xx sheets). The BZT52C copy has the
  same SHA-256 as the copy retrieved on 2026-11-07.

"""

EVIDENCE_TAIL = """\
## Facts used

- **Polarity (page 1 of every sheet):** "Polarity: Cathode line denotes the cathode
  end". The outline drawing puts the cathode band on the left in top view. The
  device puts the cathode on pad 1, at -x on the land, the `sod` convention.
- **Package (last page of every sheet):** the "Package Outline Dimensions" and
  "Suggested Pad Layout" tables are identical across the seven sheets and to the
  BZT52C sheet the land was drawn from: A 3.55-3.85, B 2.55-2.85, C 1.40-1.80,
  D 0.95-1.35 mm; J 0.91 min, K 2.36 max, M 1.22 min. Both MM1Z sheets head that
  page "(SOT-23)", but their ordering tables and page 1 say SOD-123 and the
  drawing is the SOD-123 drawing; the page title is taken as a typo.
- **Power (page 1, Maximum Ratings):** BZT52C 350 mW; BZT52B, MMSZ52xxB,
  MMSZ46xx, MM1Z..B and MM1Z 500 mW; MM1W 1000 mW at Tc = 75 C. This is the `P`
  of `Zener<VZ, P>`.
- **Electrical characteristics (pages 2-3):** every row of each table, as printed:
  VZ min/nom/max at IZT, ZZT, IR at VR and the marking code (BZT52 rows also ZZK,
  IZK and the temperature coefficient; the MMSZ46xx table gives IZT in uA and no
  ZZT). `VZ` of `Zener<VZ, P>` is the nominal
  column. pdftotext splits some numbers ("3 . 10"); the transcription joins the
  digits around a decimal point and nothing else, and the generator checks
  min < nom < max for every row and, where the MPN encodes the voltage
  (BZT52C5V6, MM1Z5V6B, MM1W330), that it equals the nominal.

## Not covered

- Tolerance is not a device parameter: the sheets give min and max per row, not a
  tolerance, and the spread is not uniform within a series (MM1W3V3 3.10-3.50 V).
  Each part's comment keeps its row.
- The B5819WS, SS34 and 1N4148WT records found the assembler's models a half
  turn from their lands. No Zener here has been previewed; none carries a
  supplier binding.
- A row a sheet prints twice with different values is not a part (below)."""


EVIDENCE_SMALL_HEAD = """\
# Hongjiacheng SOD-323 and SOD-523 Zener evidence record (parts)

- Library coverage: the 173 `D_<MPN>` parts in `src/zener_sod323_sod523.cohdl`,
  generated by `tools/gen_zener.py` from
  `tools/diode_data/hongjiacheng_zener_sod323.json`, on the `Zener` device of
  `src/zener.cohdl`.
- Manufacturer: Zhuhai Hongjiacheng Technology Co., Ltd (`www.ichjc.com` in every
  sheet's header). As for the SOD-123 sheets, no manufacturer-hosted copy was
  located, so these are the manufacturer's PDFs as served by LCSC. The PDFs are
  not redistributed.
- Retrieved: 2026-11-24, one viewer page and one PDF per sheet through
  `tools/sourcing/http.py` (2 s spacing, robots.txt honoured), 12 requests in all.

"""

EVIDENCE_SMALL_TAIL = """\
## Facts used

- **Polarity (page 1 of every sheet):** "Polarity: Cathode line denotes the cathode
  end". The outline drawing puts the cathode band on the left in top view. The
  device puts the cathode on pad 1, at -x on the land, the `sod` convention.
- **Package (last page of every sheet):** the four SOD-323 sheets print the same
  "Package Outline Dimensions (SOD-323)" and "Suggested Pad Layout" tables:
  A 1.60-1.80, C 2.30-2.80, E 1.20-1.40 mm; X 0.65-0.75, Y 0.65-0.75,
  Z 2.10-2.20 mm. `sod::SOD2613X110N` has 0.70 x 0.70 mm lands at 2.15 mm
  centres, inside every range. The BZX584C sheet prints the SOD-523 tables:
  C 1.50-1.70, E 0.70-0.90 mm; X 0.55-0.65, Y 0.65-0.75, Z 1.37-1.47 mm, and
  `sod::SOD1608X70N` has 0.60 x 0.70 mm lands at 1.42 mm centres.
- **Power (page 1, Maximum Ratings):** PD 200 mW for the four SOD-323 series,
  150 mW for BZX584C. This is the `P` of `Zener<VZ, P>`.
- **Electrical characteristics (pages 2-3):** every row of each table, as printed:
  VZ min/nom/max at IZT, ZZT, ZZK at IZK, IR at VR and the marking code; the
  BZT52, MM3Z and BZX584C tables also give the temperature coefficient and the
  MM3Z table its test current. The MM3Z table orders its columns ZZT before IZT;
  the data file records each table's column order. `VZ` of `Zener<VZ, P>` is the
  nominal column, and the generator checks min < nom < max for every row and,
  where the MPN encodes the voltage, that it equals the nominal.
- **Transcription corrections:** the BZT52B..S text layer prints `BZT52B3VOS`
  (a letter O); the row is BZT52B3V0S by its marking 2W2, its place between
  2V7 and 3V3 and the series' naming, which is how the B1 snapshot lists it.
  The MM3Z table prints `63..8` as MM3Z56's maximum temperature coefficient; it
  is recorded as 63.8. Each row keeps what was printed.

## Not covered

- Tolerance is not a device parameter, as in the SOD-123 record.
- The B5819WS (SOD-323) and 1N4148WT (SOD-523) records found the assembler's
  models a half turn from these lands. No Zener here has been previewed yet;
  a part carries a supplier binding only with its own reading."""


def render_evidence(data: dict, head: str = EVIDENCE_HEAD, tail: str = EVIDENCE_TAIL) -> str:
    """The evidence record: the sheets, the facts used, and every MPN's sheet."""
    out = [head.rstrip("\n"), "",
           "| Sheet | Revision, pages, PDF created | Listing used | PDF as served by LCSC | Bytes | SHA-256 |",
           "|---|---|---|---|---:|---|"]
    for s in data["series"]:
        out.append(f"| {s['title']} | {s['revision']}, {s['pages']} pages, {s['pdf_created']} | "
                   f"[LCSC {s['listing_part']}]({s['lcsc_listing']}) | <{s['pdf_url']}> | "
                   f"{s['bytes']:,} | `{s['sha256']}` |")
    out += ["", tail]
    omitted = [(s, o) for s in data["series"] for o in s.get("omitted", [])]
    if omitted:
        out += ["", "## Rows not made parts", ""]
        out += [f"- `{o['mpn']}` ({s['title'].split(' Surface')[0]}): {o['reason']}." for s, o in omitted]
    out += ["", "## Parts and their sheets", "",
            "Each MPN, the sheet that lists it (page) and that sheet as served by LCSC.", "",
            "| MPN | VZ min-max (V) | Sheet, page | Document |", "|---|---|---|---|"]
    for s in data["series"]:
        for row in s["parts"]:
            out.append(f"| `{row['mpn']}` | {row['vz_min_v']}-{row['vz_max_v']} | "
                       f"{s['title'].split(' Surface')[0]}, p{row['page']} | <{s['pdf_url']}> |")
    return "\n".join(out) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--check", action="store_true")
    args = ap.parse_args(argv)
    data = json.loads(DATA.read_text(encoding="utf-8"))
    small = json.loads(DATA_SMALL.read_text(encoding="utf-8"))
    mpns = [r["mpn"] for d in (data, small) for s in d["series"] for r in s["parts"]]
    if len(mpns) != len(set(mpns)):
        raise DataError("gen_zener.py: an MPN appears in both data files")
    outputs = {OUT: render(data), EVIDENCE: render_evidence(data),
               OUT_SMALL: render_small(small),
               EVIDENCE_SMALL: render_evidence(small, EVIDENCE_SMALL_HEAD, EVIDENCE_SMALL_TAIL)}
    if args.check:
        stale = [p for p, text in outputs.items() if not p.exists() or p.read_text(encoding="utf-8") != text]
        for p in stale:
            print(f"generated Zener file is stale: {p}", file=sys.stderr)
        return 1 if stale else 0
    for p, text in outputs.items():
        p.write_text(text, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
