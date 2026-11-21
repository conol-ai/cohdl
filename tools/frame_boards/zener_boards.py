"""Preview boards 8 and 9: the Hongjiacheng SOD-123 Zeners of the B1 snapshot.

Usage: python3 -I tools/frame_boards/preview_board_8.py OUT.json   (or preview_board_9.py)

A Zener is polarized and outside W3 sampled inference, so each part number
needs its own reading: the assembler's model, placed with no mapping, must show
its cathode at our pad 1 on every copy. These two boards place every one of
the 128 SOD-123 Zeners the B1 snapshot (jlc_catalog/2026-10-30) lists from
Zhuhai Hongjiacheng, four times each (top 0 and 37 degrees, bottom 0 and 37
degrees), split by series so each board stays near 64 part numbers:

- board 8: BZT52C, BZT52B and MM1W;
- board 9: MMSZ52xxB, MMSZ46xx, MM1Z..B and MM1Z.

Four part numbers whose snapshot page reads fewer than 30 in stock are left out
(LEFT_OUT); the rest, 124, are placed.

Each row declares a local part with the snapshot page's identity on the
library's device type and land: `diode::Zener<VZ, P>` on `sod::SOD3716X135N`,
taken from the library part whose MPN the page names. Nothing here is a
binding or a frame row, and no part number needs a map row to be previewed;
the bindings follow from the readings and from map rows of a reviewed backfill.
The page's identity is the snapshot's (each C number confirmed on its own
part page on 2026-10-30); the BOM step of the preview shows the stock of the
day. A page must read at least 30 in stock and five per placement.

The copies stand 10 mm apart, and the package is built with reference
designators left out of the Gerber silkscreen, as board 5 was.
"""
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
CATALOG = ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"
FOOTPRINT = "sod::SOD3716X135N"
FOUR = (("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37))
BOX, PITCH, MARGIN, BLOCKS_PER_ROW = 8.0, 10.0, 10.0, 8
SERIES = {
    "preview_board_8": (r"BZT52C\w+", r"BZT52B\w+", r"MM1W\w+"),
    "preview_board_9": (r"MMSZ\d{4}B", r"MMSZ4[67]\d\d", r"MM1Z\w+B", r"MM1Z\d+(?:V\d)?"),
}
# Part numbers of the series whose snapshot page reads too few in stock: left out, and named
# here so a changed snapshot cannot silently drop or add one.
LEFT_OUT = {"C19077397": 6, "C22395534": 9, "C22395535": 12, "C22466340": 9}
COUNTS = {"preview_board_8": 65, "preview_board_9": 63}


def library_parts():
    """MPN -> (package, version, part type) for the Zeners on the SOD-123 land."""
    with tempfile.TemporaryDirectory() as td:
        subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                       check=True, capture_output=True)
        inv = json.loads((Path(td) / "inv.json").read_text())
    users = {u["part"]: u for u in inv["users"][FOOTPRINT]}
    src = (ROOT / "lib/diode/src/zener.cohdl").read_text(encoding="utf-8")
    out = {}
    for name, mpn in re.findall(r'^pub part (D_\w+): Zener<[^>]+> \{\n    primary \{ mfr: "[^"]+", mpn: "([^"]+)"',
                                src, re.M):
        u = users[name]
        out[mpn] = (u["pkg"], u["ver"], u["type"])
    sod = next(ver for name, ver in inv["pkgs"].values() if name == "sod")
    return out, inv["devices"]["diode::Zener"], sod, inv["footprints"][FOOTPRINT]["courtyard"]


def build(board):
    parts, pins, sod_version, courtyard = library_parts()
    cx, cy, w, h = map(float, courtyard)
    if 2 * max(abs(cx) + w / 2, abs(cy) + h / 2) * 2 ** 0.5 > BOX:
        raise SystemExit(f"{FOOTPRINT}: the land does not fit its {BOX} mm box at every angle")
    pages = [r for r in json.loads(CATALOG.read_text(encoding="utf-8"))
             if r["package"] == "SOD-123" and r["mfr"] == "hongjiacheng"]
    chosen = []
    for rx in SERIES[board]:
        chosen += sorted((r for r in pages if re.fullmatch(rx, r["mpn"])), key=lambda r: r["c"])
    if len(chosen) != COUNTS[board] or len({r["c"] for r in chosen}) != len(chosen):
        raise SystemExit(f"{board}: expected {COUNTS[board]} distinct part numbers, found {len(chosen)}")
    need = max(30, 5 * len(FOUR))
    short = {r["c"]: r["stock"] for r in chosen if r["stock"] < need}
    if any(LEFT_OUT.get(c) != s for c, s in short.items()):
        raise SystemExit(f"{board}: part numbers short of {need} in stock differ from LEFT_OUT: {short}")
    chosen = [r for r in chosen if r["c"] not in short]
    deps, rows = {"std": "0.3.0", "sod": sod_version}, []
    for i, page in enumerate(chosen):
        if page["mpn"] not in parts:
            raise SystemExit(f"{page['c']}: no library part for {page['mpn']}")
        if page["stock"] < need:
            raise SystemExit(f"{page['c']}: its page reads {page['stock']} in stock, the board needs {need}")
        package, version, part_type = parts[page["mpn"]]
        deps[package] = version
        bx, by = MARGIN + (i % BLOCKS_PER_ROW) * 2 * PITCH, MARGIN + (i // BLOCKS_PER_ROW) * 2 * PITCH
        places = [{"at_mm": [bx + (j % 2) * PITCH, by + (j // 2) * PITCH], "rotate": angle, "side": side}
                  for j, (side, angle) in enumerate(FOUR)]
        rows.append({"name": "Zener_%s_%s" % (page["mpn"], page["c"]), "footprint": FOOTPRINT,
                     "part": package + "::" + part_type, "pn": page["c"], "mfr": "Zhuhai Hongjiacheng Technology",
                     "mpn": page["mpn"], "pins": pins, "width_mm": BOX, "height_mm": BOX, "covers_rotation": True,
                     "placements": places,
                     "note": "First reading of a polarized part, no mapping; not a binding. Identity from the "
                             "B1 snapshot page (manufacturer 'hongjiacheng', a reviewed alias)",
                     "jlc_package": page["package"], "category": page["category"], "stock": page["stock"],
                     "page_fetched": "2026-10-30"})
    return {"name": board, "dependencies": deps, "rows": rows}


def main(board, argv):
    manifest = build(board)
    Path(argv[1]).write_text(json.dumps(manifest, indent=1, ensure_ascii=False), encoding="utf-8")
    print(len(manifest["rows"]), "rows,", sum(len(r["placements"]) for r in manifest["rows"]), "placements")
