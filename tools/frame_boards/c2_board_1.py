"""C2 board 1 manifest: library footprints with catalog parts of the same package.

Usage: python3 -I tools/frame_boards/c2_board_1.py INVENTORY.json OUT.json
"""
import json, math, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
inv = json.load(open(sys.argv[1]))
cat = {r["c"]: r for r in json.load(open(ROOT / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json"))}

# (row name, footprint, part type or None for the footprint's first user, C-number, note)
SEL = [
    ("R0402", "passive::CHIP_0402", None, "C25744", ""),
    ("C0402", "passive::CHIP_0402", "MLCC<3.9pF, 100V, 5%>[C0402]", "C1525", ""),
    ("R0603", "passive::CHIP_0603", None, "C25804", ""),
    ("C0603", "passive::CHIP_0603", "MLCC<100nF, 16V, 10%>[C0603]", "C15849", ""),
    ("LED0603", "led::FP_LED_0603_1608Metric", None, "C2286", "polarity"),
    ("R0805", "passive::CHIP_0805", None, "C17414", ""),
    ("C0805", "passive::CHIP_0805", "MLCC<10pF, 100V, 5%>[C0805]", "C15850", ""),
    ("R1206", "passive::CHIP_1206", None, "C17927", ""),
    ("C1206", "passive::CHIP_1206", "MLCC<10pF, 100V, 5%>[C1206]", "C13585", ""),
    ("SOT23_S8050", "transistor::FP_JSCJ_SOT23", None, "C2146", "C1 re-measure: rotate 90"),
    ("SOT23_2N7002", "transistor::FP_JSCJ_SOT23", None, "C8545", "same package, other model"),
    ("SOT23_MMBT3904", "transistor::FP_JSCJ_SOT23", None, "C20526", "same package, other model"),
    ("SOD123", "diode::FP_D_SOD_123", None, "C81598", "polarity"),
    ("SOIC8_208", "soic::SOIC8P127X790X216N", None, "C97521", "C1 re-measure: rotate 270"),
    ("SOIC8_KICAD", "soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM", None, "C7950", ""),
    ("SOIC8_TI", "soic::TI_D0008A_4214825C", None, "C7433", ""),
    ("TSSOP20", "soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM", None, "C52717", ""),
    ("TSSOP24", "soic::TSSOP24P65_TI_PW0024A", None, "C7859", ""),
    # The library's own STM32F103C8T6 uses this footprint, so the row reuses its device (E802).
    ("LQFP48_KICAD", "qfp::KICAD_LQFP_48_7X7MM_P0_5MM", "STM32F103C8Tx", "C8734", ""),
    # An MPN the library does not have, so the footprint's STM32 device only supplies pins.
    ("LQFP48_IPC", "qfp::QFP50P900X900X160_48N", None, "C7532", "device supplies pins only"),
    ("LQFP32", "qfp::KICAD_LQFP_32_7X7MM_P0_8MM", None, "C8254", ""),
    ("SOT235_LDO", "ldo::SOT5P95X290X145N", None, "C7972", ""),
    ("SOT235_LOGIC", "logic::SOT5P95X300X145N", None, "C7722", ""),
    ("SOT236_ESD", "esd::FP_SOT_23_6", None, "C85364", ""),
    ("SOT236_DBV", "load-switch::FP_TI_DBV0006A", None, "C12044", ""),
    ("DFN1006", "esd::FP_Hongjiacheng_DFN1006_2L", None, "C7420372", "polarity"),
    ("XTAL3225_A", "osc::FP_Crystal_SMD_3225_4Pin_3_2x2_5mm", None, "C13738", ""),
    ("XTAL3225_B", "osc::FP_Crystal_3225", None, "C9002", ""),
    ("XTAL3215", "osc::FP_Crystal_SMD_3215_2Pin_ABS07", None, "C32346", ""),
    ("USBC", "usb::FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12", None, "C165948", "origin control: courtyard centre is off the origin"),
]
EXTRA = {"C165948": {"mfr": "Korean Hroparts Elec", "mpn": "TYPE-C-31-M-12", "package": "SMD", "category": "extended"}}
USB_PINS = ["GND", "VBUS", "CC1", "CC2", "DP", "DN", "SBU1", "SBU2", "SHIELD"]
GAP, WIDTH = 2.0, 100.0
MODULE = {"usb::FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12": "connectors::type_c::"}
versions = {name: ver for name, ver in inv["pkgs"].values()}


def ident(package):
    """Source spelling of a package name: "@st/stm32" is st_stm32, "load-switch" load_switch."""
    return package.lstrip("@").replace("/", "_").replace("-", "_")

rows, deps, x, y, shelf = [], {"std": "0.3.0"}, 0.0, 0.0, 0.0
for name, fp, ptype, c, note in SEL:
    users = inv["users"][fp]
    user = users[0]
    pkg, ver = user["pkg"], user["ver"]
    t = ptype or user["type"]
    base = t.split("<")[0].split("[")[0].strip()
    variant = t[t.index("["):] if "[" in t else ""
    devs = inv["devices"]
    pins = devs.get(pkg + "::" + base + variant, devs.get(pkg + "::" + base, ["A", "B"])) or ["A", "B"]
    if fp.startswith("usb::FP_USB_C"):
        pins = USB_PINS
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
    part = cat.get(c) or EXTRA[c]
    fp_pkg, fp_name = fp.split("::", 1)
    deps[fp_pkg] = versions[fp_pkg]
    module = MODULE.get(fp, "")
    rows.append({"name": name, "footprint": ident(fp_pkg) + "::" + module + fp_name, "part": ident(pkg) + "::" + module + t, "pn": c,
                 "mfr": part["mfr"], "mpn": part["mpn"], "pins": pins,
                 "width_mm": round(side, 3), "height_mm": round(side, 3), "placements": places,
                 "note": note, "jlc_package": part["package"], "category": part["category"]})
json.dump({"name": "c2_frame_board_1", "dependencies": deps, "rows": rows}, open(sys.argv[2], "w"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", 4 * len(rows), "placements, board ~", WIDTH, "x", round(y + shelf, 1), "mm")
