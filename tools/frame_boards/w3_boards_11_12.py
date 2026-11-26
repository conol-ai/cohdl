"""Preview boards 11 and 12: the next W3 samples of four Yageo groups.

Usage: python3 -I tools/frame_boards/preview_board_11.py OUT.json   (or preview_board_12.py)

The W3 report of 2026-11-22 (tools/sourcing_data/w3_sampling/2026-11-22/)
asks for these samples, each chosen by the report among members with at least
the rule's minimum stock on their confirmed page:

- CHIP_0603 MLCC Yageo CC: 13 (273 members, 1 passed);
- CHIP_0603 ChipResistor Yageo RC: 45 (969 members, 4 passed);
- CHIP_0805 ChipResistor Yageo RC: 44 (969 members, 5 passed);
- CHIP_1206 ChipResistor Yageo RC: 45 (967 members, 4 passed).

Each sample is placed four times (top and bottom, 0 and 37 degrees), as on
boards 6 and 7. The 147 samples are split between the two boards: each group's
list, in the report's order, alternates between them. The split is required by
the rule, not only by board size. A sample's residual counts only while its
group fills less than half of the drawn bodies on its view
(``w3_rule.bias_supported``), so no group may dominate a side. On either board
the largest group is about 31% of a side's bodies.

Four controls go on each board. All of them passed on board 6 or board 7, and
each belongs to a group not sampled here. Their rows are copied from board 7's
archived manifest.

The inputs pin the report by hash and the released passive 0.2.8 lands, as
board 7 did. The parts are looked up in that package version's tree. No land
has changed since 0.2.6.
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
GROUPS = (("0603", "MLCC", "CC", 13), ("0603", "ChipResistor", "RC", 45),
          ("0805", "ChipResistor", "RC", 44), ("1206", "ChipResistor", "RC", 45))
COLS, PITCH, MARGIN, BOX = 16, 10.0, 10.0, 6
BOARDS = {"preview_board_11": 0, "preview_board_12": 1}
CONTROLS = {"preview_board_11": ("C113780", "C11702", "C21189", "C17477"),
            "preview_board_12": ("C113780", "C11702", "C17888", "C19702")}


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
    batches = []
    for size, kind, series, count in GROUPS:
        group, = (g for g in report["groups"] if (g["footprint"], g["type"], g["mfr"], g["series"])
                  == ("CHIP_" + size, kind, "Yageo", series))
        if len(group["sample"]) != count or group["more_needed"] != count:
            raise ValueError(f"CHIP_{size} {kind} Yageo {series}: the report asks for {group['more_needed']}")
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
            if entry != "primary" or part not in parts:
                raise ValueError(f"{pn}: {part} is not a primary part of passive {inputs['packages']['passive']['version']}")
            batch.append(dict(pn=pn, mpn=mpn, mfr="Yageo", part="passive::" + parts[part],
                              footprint="passive::CHIP_" + size, group=f"CHIP_{size} {kind} Yageo {series}",
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
