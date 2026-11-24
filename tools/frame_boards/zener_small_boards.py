"""Preview board 20: the Hongjiacheng SOD-323 and SOD-523 Zeners of the B1 snapshot.

Usage: python3 -I tools/frame_boards/preview_board_20.py OUT.json

A Zener is polarized and outside W3 sampled inference, so each part number
needs its own reading: the assembler's model, placed with no mapping, must show
its cathode at our pad 1 on every copy. This board places every one of the 71
SOD-323 and SOD-523 Zeners the B1 snapshot (jlc_catalog/2026-10-30) lists from
Zhuhai Hongjiacheng (BZT52C..S, BZT52B..S, MM3Z, MMSZ52xxBS and BZX584C), four
times each (top 0 and 37 degrees, bottom 0 and 37 degrees).

Each row declares a local part with the snapshot page's identity on the
library's device type and the land its sheet names, taken from the library
part in `lib/diode/src/zener_sod323_sod523.cohdl` whose MPN the page names:
`diode::Zener<VZ, P>` on `sod::SOD2613X110N` (SOD-323) or `sod::SOD1608X70N`
(SOD-523). Nothing here is a binding or a frame row; the bindings follow from
the readings and from map rows of a reviewed backfill. A page must read at
least 40 in stock, what the BOM step asks for four placements on ten boards.

The copies stand 10 mm apart in 8 mm boxes, and the package is built with
reference designators left out of the Gerber silkscreen, as boards 5 and 8 to
19 were.
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
SOURCE = ROOT / "lib/diode/src/zener_sod323_sod523.cohdl"
LANDS = {"SOD-323": "sod::SOD2613X110N", "SOD-523": "sod::SOD1608X70N"}
SERIES = r"(BZT52[BC]\w+S|MM3Z\w+|MMSZ\d{4}BS|BZX584C\w+)"
FOUR = (("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37))
BOX, PITCH, MARGIN, BLOCKS_PER_ROW = 8.0, 10.0, 10.0, 8
NEED = 40
COUNTS = {"preview_board_20": 71}


def library_parts():
    """MPN -> (package, version, part type, footprint) for the parts of SOURCE; also the device's pins,
    the sod version and each land's courtyard."""
    with tempfile.TemporaryDirectory() as td:
        subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                       check=True, capture_output=True)
        inv = json.loads((Path(td) / "inv.json").read_text())
    users = {fp: {u["part"]: u for u in inv["users"][fp]} for fp in LANDS.values()}
    out = {}
    for name, mpn, fp in re.findall(r'^pub part (D_\w+): Zener<[^>]+> \{\n    primary \{ mfr: "[^"]+", mpn: "([^"]+)",'
                                    r'\s+footprint: (sod::\w+)', SOURCE.read_text(encoding="utf-8"), re.M):
        u = users[fp][name]
        out[mpn] = (u["pkg"], u["ver"], u["type"], fp)
    sod = next(ver for name, ver in inv["pkgs"].values() if name == "sod")
    courtyards = {fp: inv["footprints"][fp]["courtyard"] for fp in LANDS.values()}
    return out, inv["devices"]["diode::Zener"], sod, courtyards


def build(board):
    parts, pins, sod_version, courtyards = library_parts()
    for fp, courtyard in courtyards.items():
        cx, cy, w, h = map(float, courtyard)
        if 2 * max(abs(cx) + w / 2, abs(cy) + h / 2) * 2 ** 0.5 > BOX:
            raise SystemExit(f"{fp}: the land does not fit its {BOX} mm box at every angle")
    chosen = sorted((r for r in json.loads(CATALOG.read_text(encoding="utf-8"))
                     if r["mfr"] == "hongjiacheng" and r["package"] in LANDS and re.fullmatch(SERIES, r["mpn"])),
                    key=lambda r: (r["package"], r["mpn"][:5], r["c"]))
    if len(chosen) != COUNTS[board] or len({r["c"] for r in chosen}) != len(chosen):
        raise SystemExit(f"{board}: expected {COUNTS[board]} distinct part numbers, found {len(chosen)}")
    deps, rows = {"std": "0.3.0", "sod": sod_version}, []
    for i, page in enumerate(chosen):
        if page["mpn"] not in parts:
            raise SystemExit(f"{page['c']}: no library part for {page['mpn']}")
        if page["stock"] < NEED:
            raise SystemExit(f"{page['c']}: its page reads {page['stock']} in stock, the board needs {NEED}")
        package, version, part_type, footprint = parts[page["mpn"]]
        if footprint != LANDS[page["package"]]:
            raise SystemExit(f"{page['c']}: the page says {page['package']}, the library part sits on {footprint}")
        deps[package] = version
        bx, by = MARGIN + (i % BLOCKS_PER_ROW) * 2 * PITCH, MARGIN + (i // BLOCKS_PER_ROW) * 2 * PITCH
        places = [{"at_mm": [bx + (j % 2) * PITCH, by + (j // 2) * PITCH], "rotate": angle, "side": side}
                  for j, (side, angle) in enumerate(FOUR)]
        rows.append({"name": "Zener_%s_%s" % (page["mpn"], page["c"]), "footprint": footprint,
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
