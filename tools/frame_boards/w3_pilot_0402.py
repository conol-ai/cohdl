"""W3 sampling pilot manifest: the samples the grouping report chose for three 0402 groups.

Usage: python3 -I tools/frame_boards/w3_pilot_0402.py OUT.json

The proposal (docs/library-supplier-w3-sampling.md, section 4) pilots its draft
rule on one board of 0402 parts, which tests both part types and two
manufacturers on one land:

- CHIP_0402 ChipResistor Yageo RC;
- CHIP_0402 ChipResistor Vishay CRCW;
- CHIP_0402 MLCC Yageo CC.

The part numbers are the report's `sample` for each group
(tools/sourcing_data/w3_sampling/2026-11-07/groups.json), chosen there among
members with at least five pieces in stock. Each row declares a local part
with the map row's identity on the library entry's device type and footprint,
so the preview shows the raw frame with no map, as on lcsc_batch_2_passive.py.
Chip parts are not polarised, and the bottom-side convention is settled (C1),
so each part gets one top copy at 0 degrees.

The board is evidence for the pilot only. A sample that reads (0, 0, 0) gets
its own frame row; the group's other members get nothing from it until the
proposal is accepted.
"""
import json, math, re, subprocess, sys, tempfile
from pathlib import Path
ROOT = Path(__file__).resolve().parents[2]
DATA = ROOT / "tools/sourcing_data"
REPORT = DATA / "w3_sampling/2026-11-07/groups.json"
GROUPS = [("CHIP_0402", "ChipResistor", "Yageo", "RC"),
          ("CHIP_0402", "ChipResistor", "Vishay", "CRCW"),
          ("CHIP_0402", "MLCC", "Yageo", "CC")]

with tempfile.TemporaryDirectory() as td:
    subprocess.run([sys.executable, "-I", str(ROOT / "tools/frame_boards/inventory.py"), td + "/inventory.json"],
                   check=True, capture_output=True)
    inv = json.load(open(td + "/inventory.json"))
report = json.load(open(REPORT))
mapped = {r["pn"]: r for r in json.load(open(DATA / "lcsc_map.json"))["rows"]}
pages_path = next(p for p in sorted((DATA / "lcsc_backfill").glob("*/confirmed.json"))
                  if p.parent.name in report["sources"] or str(p.relative_to(DATA)) in report["sources"])
pages = {r["c"]: r for r in json.load(open(pages_path))}

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

selected = []
for key in GROUPS:
    group = next(g for g in report["groups"] if (g["footprint"], g["type"], g["mfr"], g["series"]) == key)
    if group["package_version"] != inv["pkgs"]["passive"][1]:
        raise SystemExit(f"{key}: the report is for passive {group['package_version']}, "
                         f"the library is {inv['pkgs']['passive'][1]}")
    for pn in group["sample"]:
        row = mapped[pn]
        hits = by_identity.get((row["mfr"], row["mpn"]), [])
        if len(hits) != 1:
            raise SystemExit(f"{pn}: {len(hits)} lib/passive entries")
        if pages[pn]["stock"] < 5:
            raise SystemExit(f"{pn}: stock {pages[pn]['stock']} on its confirmed page")
        selected.append((key, row, *hits[0]))

GAP, WIDTH = 2.0, 100.0
rows, x, y, shelf = [], 0.0, 0.0, 0.0
for key, row, part, kind, footprint in selected:
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
                 "note": f"W3 pilot sample, group {' '.join(key)}; lib/passive {part} {kind}",
                 "jlc_package": page["package"], "category": page["category"]})
deps = {"std": "0.3.0", "passive": inv["pkgs"]["passive"][1]}
json.dump({"name": "w3_pilot_0402", "dependencies": deps, "rows": rows},
          open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements, board ~",
      WIDTH, "x", round(y + shelf, 1), "mm")
