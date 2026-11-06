"""LCSC batch 2 manifest: the passive part numbers lcsc_map.json maps, one copy each.

Usage: python3 -I tools/frame_boards/lcsc_batch_2_passive.py OUT.json

Every row of the B2 sample's map whose (mfr, mpn) is a lib/passive entry gets
a row: 79, the sample's 64 Yageo primaries and 15 Vishay alternates.
None of them can share another part's frame evidence yet. The part pages'
package field ("0603") is not a supplier footprint identifier: on frame board
1, C7972 and C7722 both read "SOT-23-5" and needed rotate 180 and 270. So each
part number needs its own evidence, and this board gives it in one preview.

Each row declares a local part with the map row's reviewed identity (the
library entry's mfr, which is the page's after alias review, and the page's
mpn), on the library entry's device type and footprint, so the preview shows
the raw frame with no map. A part number the library has since bound keeps
one identity across the package set (E813). Chip parts are not polarised, and the bottom-side convention is
settled (C1), so one top copy at 0 degrees is enough.

A part whose page read stock 0 is left off: the assembler shows it with a
stock warning and does not place it (C32346 on frame board 1), and
jlc_preview.py stops on any stock warning. Those part numbers are printed,
and stay without evidence.
"""
import json, math, re, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
with tempfile.TemporaryDirectory() as td:
    subprocess.run([sys.executable, "-I", str(ROOT / "tools/frame_boards/inventory.py"), td + "/inventory.json"],
                   check=True, capture_output=True)
    inv = json.load(open(td + "/inventory.json"))
# The batch is the sample run's map, the 108 rows lcsc_map.json held until the B2
# full run was promoted; the full run's rows are for later batches.
SAMPLE = ROOT / "tools/sourcing_data/lcsc_backfill/2026-10-30-sample200-aliases"
mapped = json.load(open(SAMPLE / "lcsc_map.json"))["rows"]
pages = {r["c"]: r for r in json.load(open(SAMPLE / "confirmed.json"))}

ENTRY = re.compile(r'(primary|alt) \{ mfr: "([^"]+)", mpn: "([^"]+)"')
entries = {}
for user in [u for users in inv["users"].values() for u in users if u["pkg"] == "passive"]:
    entries.setdefault(user["part"], user)
by_identity = {}
for path in sorted((ROOT / "lib/passive/src").glob("*.cohdl")):
    for m in re.finditer(r"\bpub part (\w+): [^{]+\{(.*?)\n\}", path.read_text(), re.S):
        footprint = re.search(r"footprint: (\w+)", m.group(2)).group(1)
        for kind, mfr, mpn in ENTRY.findall(m.group(2)):
            by_identity.setdefault((mfr, mpn), []).append((m.group(1), kind, footprint))

GAP, WIDTH = 2.0, 100.0
rows, x, y, shelf = [], 0.0, 0.0, 0.0
selected, unstocked = [], []
for row in mapped:
    hits = by_identity.get((row["mfr"], row["mpn"]), [])
    if not hits:
        continue
    if len(hits) != 1:
        raise SystemExit(f"{row['mpn']}: {len(hits)} lib/passive entries")
    if pages[row["pn"]]["stock"] == 0:
        unstocked.append((hits[0][2], row["pn"], row["mpn"]))
        continue
    selected.append((row, *hits[0]))
# Footprint by footprint, then part number, so the board reads in groups.
selected.sort(key=lambda s: (s[3], int(s[0]["pn"][1:])))
for row, part, kind, footprint in selected:
    page = pages[row["pn"]]
    if page["mpn"] != row["mpn"]:
        raise SystemExit(f"{row['pn']}: page reads {page['mpn']}, map {row['mpn']}")
    user = entries[part]
    t = user["type"]
    base, variant = t.split("<")[0].strip(), t[t.index("["):]
    pins = inv["devices"].get("passive::" + base + variant) or ["A", "B"]
    cx, cy, w, h = map(float, inv["footprints"]["passive::" + footprint]["courtyard"])
    side = 2 * math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)
    cell = side + GAP
    if x + cell > WIDTH and x > 0:
        x, y, shelf = 0.0, y + shelf, 0.0
    at = [round(x + side / 2 + GAP / 2, 3), round(y + side / 2 + GAP / 2, 3)]
    x += cell
    shelf = max(shelf, cell)
    rows.append({"name": part + ("_ALT" if kind == "alt" else ""), "footprint": "passive::" + footprint,
                 "part": "passive::" + t, "pn": row["pn"], "mfr": row["mfr"], "mpn": page["mpn"], "pins": pins,
                 "width_mm": round(side, 3), "height_mm": round(side, 3), "covers_rotation": True,
                 "placements": [{"at_mm": at, "rotate": 0, "side": "top"}],
                 "note": f"lib/passive {part} {kind}; lcsc_map rule {row['rule']}",
                 "jlc_package": page["package"], "category": page["category"]})
deps = {"std": "0.3.0", "passive": inv["pkgs"]["passive"][1]}
json.dump({"name": "lcsc_batch_2_passive", "dependencies": deps, "rows": rows},
          open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements")
print(len(unstocked), "left off, stock 0:", " ".join(f"{fp}/{pn}" for fp, pn, _ in sorted(unstocked)))
