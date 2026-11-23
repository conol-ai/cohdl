"""Preview boards 15 to 17: the SOT-23 transistors, MOSFETs and diodes of transistor 0.2.2, mosfet 0.1.1
and diode 0.1.4.

Usage: python3 -I tools/frame_boards/preview_board_15.py OUT.json   (or preview_board_16.py)

A transistor or MOSFET is polarized and outside W3 sampled inference, so each
part number needs its own reading: the assembler's model, placed with no
mapping, must show its pin 1 mark at our pad 1 on every copy. These three
boards place every SOT-23 part `tools/gen_sot23.py` and
`tools/gen_sot23_diodes.py` added, four times each (top 0 and 37 degrees,
bottom 0 and 37 degrees):

- board 15: the 45 bipolar transistors (transistor::NPN_SOT23, PNP_SOT23);
- board 16: the 29 MOSFETs (mosfet::NMOS_SOT23, PMOS_SOT23);
- board 17: the 39 diodes (diode::*_SOT23: single, paired and Zener) but
  1SS226 (C7502731), whose page reads none in stock (LEFT_OUT), and the 13
  two-line TVS arrays (esd::TVS2*_SOT23).

Each row declares a local part with the library part's identity, device type
and land (`sot::SOT3P190X290X130N`), and the C number the B1 snapshot page
(jlc_catalog/2026-10-30) gives for that manufacturer and MPN. Nothing here is
a binding or a frame row; the bindings follow from the readings. The snapshot
page must read at least 40 in stock, what the assembler's BOM step asks for
four placements on each of ten boards; the BOM step of the preview shows the
stock of the day.

The copies stand 10 mm apart in 8 mm boxes (the land's courtyard is 3.4 x
3.6 mm), and the package is built with reference designators left out of the
Gerber silkscreen, as boards 5 and 8 to 14 were.
"""
import json
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import gen_sot23  # noqa: E402
import gen_sot23_diodes  # noqa: E402
import gen_sot23_tvs  # noqa: E402

CATALOG = ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"
ALIASES = ROOT / "tools/sourcing_data/mfr_aliases.json"
FOOTPRINT = "sot::SOT3P190X290X130N"
FOUR = (("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37))
BOX, PITCH, MARGIN, BLOCKS_PER_ROW = 8.0, 10.0, 10.0, 8
NEED = 40
BOARDS = {"preview_board_15": ("npn", "pnp"), "preview_board_16": ("nmos", "pmos"),
          "preview_board_17": ("switching", "schottky", "zener", "tvs")}
COUNTS = {"preview_board_15": 45, "preview_board_16": 29, "preview_board_17": 51}
# Library parts whose snapshot page reads fewer than NEED in stock: left out, and named here so a
# changed snapshot cannot silently drop or add one.
LEFT_OUT = {"C7502731": 0}


def inventory():
    with tempfile.TemporaryDirectory() as td:
        subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                       check=True, capture_output=True)
        return json.loads((Path(td) / "inv.json").read_text())


def build(board):
    inv = inventory()
    users = {u["part"]: u for u in inv["users"][FOOTPRINT]}
    cx, cy, w, h = map(float, inv["footprints"][FOOTPRINT]["courtyard"])
    if 2 * max(abs(cx) + w / 2, abs(cy) + h / 2) * 2 ** 0.5 > BOX:
        raise SystemExit(f"{FOOTPRINT}: the land does not fit its {BOX} mm box at every angle")
    aliases = json.loads(ALIASES.read_text(encoding="utf-8"))["aliases"]
    pages = {r["c"]: r for r in json.loads(CATALOG.read_text(encoding="utf-8"))}
    diodes = board == "preview_board_17"
    gens = [gen_sot23_diodes, gen_sot23_tvs] if diodes else [gen_sot23]
    data = []
    for gen in gens:
        for r in json.loads(gen.DATA.read_text(encoding="utf-8"))["parts"]:
            data.append(dict(r, kind=r.get("kind", "tvs"), gen=gen))
    chosen = sorted((r for r in data if r["kind"] in BOARDS[board]), key=lambda r: (r["kind"], r["c"]))
    short = {r["c"]: pages[r["c"]]["stock"] for r in chosen if pages[r["c"]]["stock"] < NEED}
    if any(LEFT_OUT.get(c) != s for c, s in short.items()):
        raise SystemExit(f"{board}: part numbers short of {NEED} in stock differ from LEFT_OUT: {short}")
    chosen = [r for r in chosen if r["c"] not in short]
    if len(chosen) != COUNTS[board]:
        raise SystemExit(f"{board}: expected {COUNTS[board]} parts, found {len(chosen)}")
    deps, rows = {"std": "0.3.0"}, []
    for pkg, ver in inv["pkgs"].values():
        if pkg == "sot":
            deps["sot"] = ver
    for i, part in enumerate(chosen):
        page = pages[part["c"]]
        if (aliases.get(page["mfr"], page["mfr"]), page["mpn"], page["package"]) != (part["mfr"], part["mpn"], "SOT-23"):
            raise SystemExit(f"{part['c']}: the snapshot page does not name {part['mfr']} {part['mpn']} in SOT-23")
        if page["stock"] < NEED:
            raise SystemExit(f"{part['c']}: its page reads {page['stock']} in stock, the board needs {NEED}")
        gen = part["gen"]
        name = gen.part_name(part)
        user = users[name]
        deps[user["pkg"]] = user["ver"]
        device = (gen_sot23_tvs.DEVICES[part["config"]][0] if gen is gen_sot23_tvs
                  else gen.device(part) if gen is gen_sot23_diodes else gen_sot23.DEVICES[part["kind"]][1])
        bx, by = MARGIN + (i % BLOCKS_PER_ROW) * 2 * PITCH, MARGIN + (i // BLOCKS_PER_ROW) * 2 * PITCH
        places = [{"at_mm": [bx + (j % 2) * PITCH, by + (j // 2) * PITCH], "rotate": angle, "side": side}
                  for j, (side, angle) in enumerate(FOUR)]
        rows.append({"name": "%s_%s" % (name, part["c"]), "footprint": FOOTPRINT,
                     "part": user["pkg"] + "::" + user["type"], "pn": part["c"], "mfr": part["mfr"],
                     "mpn": part["mpn"], "pins": inv["devices"][user["pkg"] + "::" + device],
                     "width_mm": BOX, "height_mm": BOX, "covers_rotation": True, "placements": places,
                     "note": "First reading of a polarized part, no mapping; not a binding. Identity from the "
                             "B1 snapshot page (manufacturer %r, a reviewed alias)" % page["mfr"],
                     "jlc_package": page["package"], "category": page["category"], "stock": page["stock"],
                     "page_fetched": "2026-10-30"})
    return {"name": board, "dependencies": dict(sorted(deps.items())), "rows": rows}


def main(board, argv):
    manifest = build(board)
    Path(argv[1]).write_text(json.dumps(manifest, indent=1, ensure_ascii=False), encoding="utf-8")
    print(len(manifest["rows"]), "rows,", sum(len(r["placements"]) for r in manifest["rows"]), "placements")
