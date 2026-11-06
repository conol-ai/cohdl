"""LCSC batch 1 manifest: the six @st/stm32 parts bound in the library, as shipped.

Usage: python3 -I tools/frame_boards/lcsc_batch_1.py OUT.json

Each row places the library part itself (``library_part``), so the preview
checks the library's own `supplier lcsc` binding, including the three
`rotate 270` exceptions. Pins, boxes and the four placements per part (top
and bottom, 0 and 37 degrees) are copied from frame board 2, whose preview
of 2026-11-02 is the evidence for these frames. The board builds against the
library's current @st/stm32, which still ships these six bindings unchanged.
"""
import json, re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
board2 = json.load(open(ROOT / "tools/frame_boards/c2-board-2/manifest.json", encoding="utf-8"))
mapped = {(r["mfr"], r["mpn"]): r["pn"] for r in json.load(open(ROOT / "tools/sourcing_data/lcsc_map.json"))["rows"]}

# (board 2 row, library part, bound entry's MPN)
SEL = [
    ("LQFP208", "MCU_STM32F767BIT6", "STM32F767BIT6"),
    ("LQFP100_F4", "MCU_STM32F427VGT6", "STM32F427VGT6"),
    ("LQFP100_L4", "MCU_STM32L496VGT6", "STM32L496VGT6"),
    ("LQFP32_KICAD", "MCU_STM32L412K8T6", "STM32L412K8T6"),
    ("TFBGA100", "MCU_STM32H743VIH6", "STM32H743VIH6TR"),
    ("UFBGA64", "MCU_STM32G431RBI6", "STM32G431RBI6"),
]
rows = {r["name"]: r for r in board2["rows"]}
out = []
for name, part, mpn in SEL:
    row = dict(rows[name])
    if row["mpn"] != mpn or mapped.get((row["mfr"], mpn)) != row["pn"]:
        raise SystemExit(f"{name}: board 2 row and lcsc_map.json disagree on {mpn}")
    row["library_part"] = "st_stm32::" + part
    row["note"] = "library binding as shipped in @st/stm32 0.3.0"
    out.append(row)
deps = {k: v for k, v in board2["dependencies"].items() if k in ("std", "@st/stm32", "qfp", "bga")}
deps["@st/stm32"] = re.search(r'^version = "([^"]+)"', (ROOT / "lib/@st/stm32/cohdl.toml").read_text(), re.M).group(1)
json.dump({"name": "lcsc_batch_1", "dependencies": deps, "rows": out},
          open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(out), "rows,", sum(len(r["placements"]) for r in out), "placements")
