"""Preview boards 13 and 14: W3 samples of five groups no earlier board covers.

Usage: python3 -I tools/frame_boards/preview_board_13.py OUT.json   (or preview_board_14.py)

The W3 report of 2026-11-23b (tools/sourcing_data/w3_sampling/2026-11-23b/),
built after the first packages' BOM steps, asks for these samples, each chosen by the
report among members with at least the rule's minimum stock on their
confirmed page:

- CHIP_0201 ChipResistor Vishay CRCW: 32 (623 members, none passed);
- CHIP_0201 MLCC Yageo CC: 4 (117 members, 2 passed);
- CHIP_1206 MLCC Yageo CC: 5 (97 members, none passed);
- CHIP_2010 ChipResistor Yageo RC: 42 (865 members, 2 passed);
- CHIP_2512 ChipResistor Yageo RC: 43 (863 members, 1 passed).

None of these groups is on boards 11 and 12, so boards 11 and 12's results
cannot change what they ask for. The paused groups (Yageo RC 0201 and 1210,
Vishay CRCW 0402, Yageo CC 0805) are left out: a sample cannot unpause them.

Each sample is placed four times (top and bottom, 0 and 37 degrees). The 126
samples alternate between the two boards in each group's report order, as on
boards 11 and 12, so no group fills half of a side's bodies
(``w3_rule.bias_supported``): the largest is about a third.

A 2512 land turned 37 degrees spans about 8.4 mm, so each copy has a 9 mm
box on the 10 mm grid; ``build`` checks every land against it. Vishay CRCW
members are the library's alternates: the row names the alternate's C
number and MPN on the primary part's device type.

The four controls are boards 11 and 12's, copied from board 7's archived
manifest; each belongs to a group not sampled here. The inputs pin the report
by hash and the released passive 0.2.8 lands, as on boards 7, 11 and 12.
"""
import hashlib
import json
import re
import subprocess
import sys
from itertools import zip_longest
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
GROUPS = (("0201", "ChipResistor", "Vishay", "CRCW", 32), ("0201", "MLCC", "Yageo", "CC", 4),
          ("1206", "MLCC", "Yageo", "CC", 5), ("2010", "ChipResistor", "Yageo", "RC", 42),
          ("2512", "ChipResistor", "Yageo", "RC", 43))
COLS, PITCH, MARGIN, BOX = 16, 10.0, 10.0, 9
BOARDS = {"preview_board_13": 0, "preview_board_14": 1}
CONTROLS = {"preview_board_13": ("C113780", "C11702", "C21189", "C17477"),
            "preview_board_14": ("C113780", "C11702", "C17888", "C19702")}
ENTRY = {"Yageo": "primary", "Vishay": "alt"}


def check_box(tree):
    """Every sampled land, turned to any angle, fits its BOX."""
    src = subprocess.check_output(["git", "show", f"{tree}:src/footprints.cohdl"], cwd=ROOT, text=True)
    for size in {g[0] for g in GROUPS}:
        block = src[src.index(f"pub footprint CHIP_{size} {{"):]
        w, h = map(float, re.search(r"courtyard \{[^}]*size: \(([\d.]+)mm, ([\d.]+)mm\)", block).groups())
        if (w * w + h * h) ** 0.5 > BOX:
            raise ValueError(f"CHIP_{size}: the land does not fit its {BOX} mm box at every angle")


def build(board):
    half = BOARDS[board]
    inputs = json.loads((HERE / board.replace("_", "-") / "inputs.json").read_text())
    source = (ROOT / inputs["report"]).read_bytes()
    if hashlib.sha256(source).hexdigest() != inputs["report_sha256"]:
        raise ValueError("W3 report differs from the selected snapshot")
    report = json.loads(source)
    pages_path, = (name for name in report["sources"] if name.endswith("/confirmed.json"))
    raw = (ROOT / "tools/sourcing_data" / pages_path).read_bytes()
    if hashlib.sha256(raw).hexdigest() != report["sources"][pages_path]:
        raise ValueError("stock snapshot differs from the W3 report")
    pages = {r["c"]: r for r in json.loads(raw)}
    tree = inputs["packages"]["passive"]["tree"]
    check_box(tree)
    batches = []
    for size, kind, mfr, series, count in GROUPS:
        group, = (g for g in report["groups"] if (g["footprint"], g["type"], g["mfr"], g["series"])
                  == ("CHIP_" + size, kind, mfr, series))
        if len(group["sample"]) != count or group["more_needed"] != count:
            raise ValueError(f"CHIP_{size} {kind} {mfr} {series}: the report asks for {group['more_needed']}")
        name = ("capacitors_" if kind == "MLCC" else "resistors_") + size
        src = subprocess.check_output(["git", "show", f"{tree}:src/{name}.cohdl"], cwd=ROOT, text=True)
        parts = dict(re.findall(r"^pub part (\w+): ((?:MLCC|ChipResistor)<[^\n]+?\]) \{", src, re.M))
        rows = {r[0]: r for r in group["member_rows"]}
        batch = []
        for pn in group["sample"][half::2]:
            _, mpn, part, entry, category, stock = rows[pn]
            p = pages[pn]
            if (p["mpn"], p["package"], p["stock"]) != (mpn, size, stock) or stock < report["rule"]["min_stock"]:
                raise ValueError(f"{pn}: incompatible identity, package or stock")
            if entry != ENTRY[mfr] or part not in parts:
                raise ValueError(f"{pn}: {part} is not a part of passive {inputs['packages']['passive']['version']} "
                                 f"with a {ENTRY[mfr]} {mfr} entry")
            batch.append(dict(pn=pn, mpn=mpn, mfr=mfr, part="passive::" + parts[part],
                              footprint="passive::CHIP_" + size, group=f"CHIP_{size} {kind} {mfr} {series}",
                              purpose="sample", jlc_package=size, stock=stock, page_fetched=p["fetched_date"],
                              category=category))
        batches.append(batch)
    archive = ROOT / "tools/sourcing_data/jlc_previews/2026-11-21-board-7/package/manifest.json"
    old = {r["pn"]: r for r in json.loads(archive.read_text())["rows"]}
    controls = []
    for pn in CONTROLS[board]:
        row = {k: v for k, v in old[pn].items() if k not in ("name", "placements")}
        controls.append(dict(row, purpose="control"))
    if any(c["group"] in {b[0]["group"] for b in batches} for c in controls):
        raise ValueError("a control belongs to a sampled group")
    batches.append(controls)
    rows = []
    for row in (r for group in zip_longest(*batches) for r in group if r):
        col, band = len(rows) % COLS, len(rows) // COLS
        places = [{"at_mm": [MARGIN + col * PITCH, MARGIN + (4 * band + 2 * j + i) * PITCH], "side": side,
                   "rotate": angle}
                  for i, side in enumerate(("top", "bottom")) for j, angle in enumerate((0, 37))]
        rows.append(dict(row, name=row["purpose"] + "_" + row["pn"], pins=["A", "B"], width_mm=BOX,
                         height_mm=BOX, covers_rotation=True, placements=places))
    if len({r["pn"] for r in rows}) != len(rows):
        raise ValueError("duplicate target or control")
    return {"name": board, "dependencies": {n: p["version"] for n, p in inputs["packages"].items()}, "rows": rows}


def main(board, argv):
    manifest = build(board)
    Path(argv[1]).write_text(json.dumps(manifest, ensure_ascii=False, indent=1) + "\n")
    print(len(manifest["rows"]), "rows,", sum(len(r["placements"]) for r in manifest["rows"]), "placements")
