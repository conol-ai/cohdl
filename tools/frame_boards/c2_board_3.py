"""C2 board 3 manifest: the footprint families boards 1 and 2 left without evidence.

Usage: python3 -I tools/frame_boards/c2_board_3.py OUT.json

The 2026-10-06 list of 14 families was not kept, so this board takes what
boards 1 and 2 left open and the most-used footprints that still have no
frame row in jlc_frames.json:

- the four chip sizes no board has covered, with library parts the official
  map binds (tools/sourcing_data/lcsc_map.json);
- second and third models on the footprints boards 1 and 2 moved, so each
  group has the three direct checks the shared-evidence rule asks for;
- one more model on a SOT-23-5 and a SOT-23-6 land, where board 1 found the
  correction depends on the model;
- QFN, SOD-323, ESOP-8 and tactile-switch lands, each with a catalog part
  of the same package;
- the 3215 crystal, which board 1's preview did not place, and two BGAs that
  board 2's could not place for stock. Each BGA is placed once: the
  assembler builds five boards, so one copy needs five parts, not twenty.

Every part number is a confirmed page: the W1 snapshot of 2026-10-30 or the
B2 sample's confirmations (tools/sourcing_data/lcsc_backfill/2026-10-30-sample200).
The script indexes the library itself (inventory.py), so preview_package.py
can check that the committed manifest is still its output.
"""
import json, math, subprocess, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
with tempfile.TemporaryDirectory() as td:
    subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                   check=True, capture_output=True)
    inv = json.load(open(Path(td) / "inv.json"))
cat = {r["c"]: r for r in json.load(open(ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"))}
# Extended parts: the B2 sample's partdetail confirmations.
cat.update({r["c"]: r for r in json.load(open(ROOT / "tools/sourcing_data/lcsc_backfill/2026-10-30-sample200/confirmed.json"))})
mapped = {r["pn"]: r for r in json.load(open(ROOT / "tools/sourcing_data/lcsc_map.json"))["rows"]}

# (row name, footprint, part type or None for the footprint's first user, C-number, copies, note)
# copies: 4 is top 0/37 and bottom 0/37, as on boards 1 and 2; 1 is top 0 only.
SEL = [
    # Chip sizes with no frame row; each part is an official map row.
    ("C0201_680P", "passive::CHIP_0201", "MLCC<680pF, 50V, 10%>[C0201]", "C718246", 4, "map row"),
    ("C0201_330P", "passive::CHIP_0201", "MLCC<330pF, 16V, 10%>[C0201]", "C541229", 4, "map row"),
    ("R0201", "passive::CHIP_0201", "ChipResistor<221ohm, 1%>[R0201]", "C138161", 4, "map row"),
    ("R1210_510R", "passive::CHIP_1210", "ChipResistor<510ohm, 1%>[R1210]", "C309589", 4, "map row"),
    ("R1210_25R5", "passive::CHIP_1210", "ChipResistor<25.5ohm, 1%>[R1210]", "C874860", 4, "map row; 30 in stock"),
    ("R2010", "passive::CHIP_2010", "ChipResistor<200kohm, 1%>[R2010]", "C877226", 4, "map row"),
    ("R2512_4M7", "passive::CHIP_2512", "ChipResistor<4.7Mohm, 5%>[R2512]", "C878293", 4, "map row"),
    ("R2512_4R7", "passive::CHIP_2512", "ChipResistor<4.7ohm, 1%>[R2512]", "C878096", 4, "map row"),
    # More models on footprints boards 1 and 2 moved; expected map 0.
    ("LQFP48_F030", "qfp::KICAD_LQFP_48_7X7MM_P0_5MM", "STM32F030C8Tx", "C23922", 4, "second model"),
    ("LQFP48_L151", "qfp::KICAD_LQFP_48_7X7MM_P0_5MM", "STM32L151C8Tx", "C9861", 4, "third model"),
    ("TQFP32_M328", "qfp::KICAD_LQFP_32_7X7MM_P0_8MM", None, "C14877", 4, "third model; device supplies pins only"),
    ("SOIC8_LM2904", "soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM", None, "C18229", 4, "second model; device supplies pins only"),
    ("SOIC8_SP3485", "soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM", None, "C8963", 4, "third model; device supplies pins only"),
    # Board 1 read 180 for C7972 and C85364 on these lands; a second model each.
    ("SOT235_TP4054", "ldo::SOT5P95X290X145N", None, "C32574", 4, "second model; device supplies pins only"),
    ("SOT236_SRV05", "esd::FP_SOT_23_6", None, "C7420376", 4, "second model; device supplies pins only"),
    # Families with no frame row, on a catalog part of the same package.
    ("QFN24", "qfn::ESPRESSIF_QFN24_0P5_4_E2P8", None, "C24112", 4, "device supplies pins only"),
    ("SOD323_1N4148", "@contrib/esd::FP_SOD323_2P5X1P25mm", None, "C2128", 4, "polarity; device supplies pins only"),
    ("SOD323_1N5819", "@contrib/esd::FP_SOD323_2P5X1P25mm", None, "C191023", 4, "polarity; device supplies pins only"),
    ("ESOP8", "audio-amp::FP_NS4168_ESOP8", None, "C16581", 4, "exposed pad; device supplies pins only"),
    ("SW_TS1088", "switches::FP_XUNPU_TS_1088R", None, "C720477", 4, "device supplies pins only"),
    # Left unplaced by boards 1 and 2.
    ("XTAL3215", "osc::FP_Crystal_SMD_3215_2Pin_ABS07", None, "C32346", 4, "board 1 retry"),
    ("UFBGA100", "bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM", "STM32L152VCHx", "C1338995", 1, "board 2 retry; 10 in stock; ball A1"),
    ("TFBGA169_ST", "bga::KICAD_ST_TFBGA_169_7X7MM_LAYOUT13X13_P0_5MM", "STM32U595AJHx", "C24622621", 1, "board 2 retry; 10 in stock; ball A1"),
]
GAP, WIDTH = 2.0, 120.0
versions = {name: ver for name, ver in inv["pkgs"].values()}


def ident(package):
    """Source spelling of a package name: "@st/stm32" is st_stm32, "load-switch" load_switch."""
    return package.lstrip("@").replace("/", "_").replace("-", "_")

def reach(row):
    cx, cy, w, h = map(float, inv["footprints"][row[1]]["courtyard"])
    return math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)


rows, deps, x, y, shelf = [], {"std": "0.3.0"}, 0.0, 0.0, 0.0
for name, fp, ptype, c, copies, note in sorted(SEL, key=lambda r: (-r[4], -reach(r))):
    if note.startswith("map row") and c not in mapped:
        raise SystemExit(f"{name}: {c} is not in lcsc_map.json")
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
    side = 2 * reach((name, fp))
    step = side + GAP
    cell_w, cell_h = (2 * step, 2 * step) if copies == 4 else (step, step)
    if x + cell_w > WIDTH and x > 0:
        x, y, shelf = 0.0, y + shelf, 0.0
    ox, oy = x + side / 2 + GAP / 2, y + side / 2 + GAP / 2
    places = [{"at_mm": [round(ox, 3), round(oy, 3)], "rotate": 0, "side": "top"}]
    if copies == 4:
        places += [{"at_mm": [round(ox + step, 3), round(oy, 3)], "rotate": 37, "side": "top"},
                   {"at_mm": [round(ox, 3), round(oy + step, 3)], "rotate": 0, "side": "bottom"},
                   {"at_mm": [round(ox + step, 3), round(oy + step, 3)], "rotate": 37, "side": "bottom"}]
    x += cell_w
    shelf = max(shelf, cell_h)
    part = cat[c]
    fp_pkg, fp_name = fp.split("::", 1)
    deps[fp_pkg] = versions[fp_pkg]
    rows.append({"name": name, "footprint": ident(fp_pkg) + "::" + fp_name, "part": ident(pkg) + "::" + t, "pn": c,
                 "mfr": part["mfr"], "mpn": part["mpn"], "pins": pins,
                 "width_mm": round(side, 3), "height_mm": round(side, 3), "covers_rotation": True, "placements": places,
                 "note": note, "jlc_package": part["package"], "category": part["category"]})
json.dump({"name": "c2_frame_board_3", "dependencies": deps, "rows": rows}, open(sys.argv[1], "w"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements, board ~", WIDTH, "x", round(y + shelf, 1), "mm")
