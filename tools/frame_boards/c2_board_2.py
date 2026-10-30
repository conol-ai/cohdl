"""C2 board 2 manifest: the footprints board 1 moved, at map 0, and the STM32 package families.

Usage: python3 -I tools/frame_boards/c2_board_2.py INVENTORY.json OUT.json

Board 1 measured nine footprints off the assembler's frame, and transistor
0.2.0, soic 0.3.0 and qfp 0.3.0 moved them. The usbc-origin board confirmed
two of them; this board confirms the other seven with no map. The STM32 rows
are library parts whose MPN the B2 sample matched exactly to an LCSC part
(tools/sourcing_data/lcsc_backfill/2026-10-30-sample200), one per package
family the library uses and board 1 did not cover.
"""
import json, math, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
inv = json.load(open(sys.argv[1]))
cat = {r["c"]: r for r in json.load(open(ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"))}
# Extended parts: the B2 sample's partdetail confirmations.
cat.update({r["c"]: r for r in json.load(open(ROOT / "tools/sourcing_data/lcsc_backfill/2026-10-30-sample200/confirmed.json"))})

# (row name, footprint, part type or None for the footprint's first user, C-number, note)
SEL = [
    # Moved by board 1; expected map 0.
    ("LQFP32_KICAD", "qfp::KICAD_LQFP_32_7X7MM_P0_8MM", "STM32L412K8Tx", "C529436", "moved rotate 270; B2 exact match"),
    ("LQFP48_KICAD", "qfp::KICAD_LQFP_48_7X7MM_P0_5MM", "STM32F103C8Tx", "C8734", "moved rotate 270"),
    # An MPN the library does not have, so the footprint's STM32 device only supplies pins.
    ("LQFP48_IPC", "qfp::QFP50P900X900X160_48N", None, "C7532", "moved rotate 270; device supplies pins only"),
    ("SOIC8_KICAD", "soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM", None, "C7950", "moved rotate 270"),
    ("SOIC8_TI", "soic::TI_D0008A_4214825C", None, "C7433", "moved rotate 270"),
    ("TSSOP20", "soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM", None, "C52717", "moved rotate 270"),
    ("TSSOP24", "soic::TSSOP24P65_TI_PW0024A", None, "C7859", "moved rotate 270"),
    # STM32 package families, each a B2 exact match.
    ("LQFP64", "qfp::KICAD_LQFP_64_10X10MM_P0_5MM", "STM32L476RGTx", "C1337443", "low stock"),
    ("LQFP100_F4", "qfp::KICAD_LQFP_100_14X14MM_P0_5MM", "STM32F427VGTx", "C117815", ""),
    ("LQFP100_L4", "qfp::KICAD_LQFP_100_14X14MM_P0_5MM", "STM32L496VGTx", "C717701", "same package, other model"),
    ("LQFP176", "qfp::KICAD_LQFP_176_24X24MM_P0_5MM", "STM32F437IITx", "C5269976", "low stock"),
    ("LQFP208", "qfp::KICAD_LQFP_208_28X28MM_P0_5MM", "STM32F767BITx", "C129276", ""),
    ("UFBGA64", "bga::KICAD_UFBGA_64_5X5MM_LAYOUT8X8_P0_5MM", "STM32G431RBIx", "C730103", "ball A1"),
    ("UFBGA100", "bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM", "STM32L152VCHx", "C1338995", "ball A1"),
    ("TFBGA100", "bga::KICAD_TFBGA_100_8X8MM_LAYOUT10X10_P0_8MM", "STM32H743VIHx", "C1857444", "ball A1"),
    ("TFBGA169_ST", "bga::KICAD_ST_TFBGA_169_7X7MM_LAYOUT13X13_P0_5MM", "STM32U595AJHx", "C24622621", "ball A1"),
]
GAP, WIDTH = 2.0, 175.0
versions = {name: ver for name, ver in inv["pkgs"].values()}


def ident(package):
    """Source spelling of a package name: "@st/stm32" is st_stm32, "load-switch" load_switch."""
    return package.lstrip("@").replace("/", "_").replace("-", "_")

def reach(row):
    cx, cy, w, h = map(float, inv["footprints"][row[1]]["courtyard"])
    return math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)


rows, deps, x, y, shelf = [], {"std": "0.3.0"}, 0.0, 0.0, 0.0
# Largest first packs the shelves tighter: about 175 x 209 mm instead of 160 x 293.
for name, fp, ptype, c, note in sorted(SEL, key=lambda r: -reach(r)):
    users = inv["users"][fp]
    # A named part type picks its own package's user, not the footprint's first.
    user = next((u for u in users if ptype and u["type"].split("[")[0] == ptype), users[0])
    pkg, ver = user["pkg"], user["ver"]
    t = ptype or user["type"]
    base = t.split("<")[0].split("[")[0].strip()
    variant = t[t.index("["):] if "[" in t else ""
    devs = inv["devices"]
    pins = devs.get(pkg + "::" + base + variant, devs.get(pkg + "::" + base, ["A", "B"])) or ["A", "B"]
    deps[pkg] = ver
    fpinfo = inv["footprints"][fp]
    cx, cy, w, h = map(float, fpinfo["courtyard"])
    # Box that holds the courtyard about the origin at any rotation used here.
    r = math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)
    side = 2 * r
    cell_w, cell_h = 2 * (side + GAP), 2 * (side + GAP)
    if x + cell_w > WIDTH and x > 0:
        x, y, shelf = 0.0, y + shelf, 0.0
    ox, oy = x + side / 2 + GAP / 2, y + side / 2 + GAP / 2
    step = side + GAP
    places = [{"at_mm": [round(ox, 3), round(oy, 3)], "rotate": 0, "side": "top"},
              {"at_mm": [round(ox + step, 3), round(oy, 3)], "rotate": 37, "side": "top"},
              {"at_mm": [round(ox, 3), round(oy + step, 3)], "rotate": 0, "side": "bottom"},
              {"at_mm": [round(ox + step, 3), round(oy + step, 3)], "rotate": 37, "side": "bottom"}]
    x += cell_w
    shelf = max(shelf, cell_h)
    part = cat[c]
    fp_pkg, fp_name = fp.split("::", 1)
    deps[fp_pkg] = versions[fp_pkg]
    module = ""
    rows.append({"name": name, "footprint": ident(fp_pkg) + "::" + module + fp_name, "part": ident(pkg) + "::" + module + t, "pn": c,
                 "mfr": part["mfr"], "mpn": part["mpn"], "pins": pins,
                 "width_mm": round(side, 3), "height_mm": round(side, 3), "placements": places,
                 "note": note, "jlc_package": part["package"], "category": part["category"]})
json.dump({"name": "c2_frame_board_2", "dependencies": deps, "rows": rows}, open(sys.argv[2], "w"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", 4 * len(rows), "placements, board ~", WIDTH, "x", round(y + shelf, 1), "mm")
