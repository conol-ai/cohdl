"""Preview board 5 manifest: a re-verify board for three candidate rotations.

Usage: python3 -I tools/frame_boards/preview_board_5.py OUT.json

Preview board 4 (tools/sourcing_data/jlc_previews/2026-11-11-board-4) placed
three ICs with no mapping, and each read the same turn on all four copies:
C7519 and C189961 a quarter turn, C131941 half a turn. A raw reading is a
candidate, not a mapping: it is confirmed only by a board that places the part
with the candidate applied and shows the model's leads on our pads with pin 1
at our pad 1, as the two re-verify boards of 2026-11-08 and 2026-11-09 did for
their parts. This is that board for these three.

Each row declares a local part with the map row's identity on the library
entry's device type and land, as on board 4, and carries the candidate as its
`mapping`, which `jlc_frame_board.py` writes into the supplier CPL. Each is
placed four times: top 0 and 37 degrees, bottom 0 and 37 degrees. Nothing here
is a binding, and no frame row is written from this file. The board was
previewed on 2026-11-14 (jlc_previews/2026-11-14-board-5) and confirmed all three
candidates, which are now bound; the generator accepts a bound or framed part
number only where its frame row cites that record with the candidate as mapping.

The board-4 record showed silkscreen reference text disturbing readings beside
a part. The copies therefore stand 16 mm apart, and the package is to be built
with the reference designators left out of the Gerber silkscreen, as re-verify
board 2 was (README.md).

Every part number is a confirmed page already in the repository (the B2 full
run, lcsc_backfill/2026-11-07-samsung-b1) and a row of lcsc_map.json. Nothing
is fetched. A page must read at least five pieces per placement (the assembler
builds five boards) and at least 30.
"""
import json, re, subprocess, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DATA = ROOT / "tools/sourcing_data"
RECORD = "tools/sourcing_data/jlc_previews/2026-11-11-board-4/README.md"
# The record of this board's own preview, which confirmed all three candidates (2026-11-14).
PREVIEWED = "tools/sourcing_data/jlc_previews/2026-11-14-board-5/README.md"
with tempfile.TemporaryDirectory() as td:
    subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                   check=True, capture_output=True)
    inv = json.load(open(Path(td) / "inv.json"))
pages = {r["c"]: r for r in json.load(open(DATA / "lcsc_backfill/2026-11-07-samsung-b1/confirmed.json"))}
mapped = {r["pn"]: r for r in json.load(open(DATA / "lcsc_map.json"))["rows"]}
framed = {r["pn"]: r for r in json.load(open(DATA / "jlc_frames.json"))["rows"]}

# (part number, library part as package::PART, candidate rotate, what board 4 read:
# its pin 1 counter-clockwise from ours on top 0, top 37, bottom 0, bottom 37, seen from the top)
CANDIDATES = [
    ("C7519", "esd::ESD_USBLC6", 270, (100.1, 100.8, 259.7, 259.9)),
    ("C131941", "load-switch::LOADSW_TPS22918DBVR", 180, (183.0, 183.0, 177.4, 178.2)),
    ("C189961", "@contrib/audio-amp::AMP_NS4150B", 270, (94.8, 95.2, 264.9, 265.3)),
]
FOUR = (("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37))
BOX, PITCH, MARGIN = 14.0, 16.0, 10.0
ENTRY = re.compile(r'(primary|alt) \{ mfr: "([^"]+)", mpn: "([^"]+)"')


def ident(package):
    """Source spelling of a package name: "@contrib/audio-amp" is contrib_audio_amp."""
    return package.lstrip("@").replace("/", "_").replace("-", "_")


parts = {}
for footprint, users in inv["users"].items():
    for user in users:
        parts[user["pkg"] + "::" + user["part"]] = dict(user, footprint=footprint)
for directory, (package, _) in sorted(inv["pkgs"].items()):
    for path in sorted((ROOT / "lib" / directory / "src").rglob("*.cohdl")):
        for m in re.finditer(r"\bpub part (\w+): [^{]+\{(.*?)\n\}", path.read_text(encoding="utf-8"), re.S):
            key = package + "::" + m.group(1)
            if key in parts:
                parts[key]["entries"] = ENTRY.findall(m.group(2))
                parts[key]["bound"] = {pn: int(turn or 0) for pn, turn in
                                       re.findall(r'supplier lcsc "(C\d+)"(?: rotate (\d+))?', m.group(2))}

deps, rows = {"std": "0.3.0"}, []
for i, (pn, key, rotate, read) in enumerate(CANDIDATES):
    part, page = parts[key], pages[pn]
    need = max(30, 5 * len(FOUR))
    if page["stock"] < need:
        raise SystemExit(f"{pn}: its page reads {page['stock']} in stock, the board needs {need}")
    entry = next((e for e in part["entries"] if (e[1], e[2]) == (mapped[pn]["mfr"], mapped[pn]["mpn"])), None)
    if entry is None or page["mpn"] != mapped[pn]["mpn"]:
        raise SystemExit(f"{pn}: {key} has no entry {mapped[pn]['mfr']} {mapped[pn]['mpn']}")
    # Since its preview a candidate may be bound and framed, but only as this board confirmed it.
    row = framed.get(pn)
    if row is not None and (row["evidence"]["record"] != PREVIEWED or row["mapping"] != {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate}):
        raise SystemExit(f"{pn}: has a frame row that is not this board's confirmation of rotate {rotate}")
    if pn in part["bound"] and (row is None or part["bound"][pn] != rotate):
        raise SystemExit(f"{pn}: bound otherwise than this board confirmed (rotate {rotate})")
    package, footprint = part["pkg"], part["footprint"]
    fp_package, fp_name = footprint.split("::", 1)
    cx, cy, w, h = map(float, inv["footprints"][footprint]["courtyard"])
    if 2 * max(abs(cx) + w / 2, abs(cy) + h / 2) * 2 ** 0.5 > BOX:
        raise SystemExit(f"{pn}: the land does not fit its {BOX} mm box at every angle")
    t = part["type"]
    base = t.split("<")[0].split("[")[0].strip()
    variant = t[t.index("["):] if "[" in t else ""
    devices = inv["devices"]
    pins = devices.get(package + "::" + base + variant, devices.get(package + "::" + base))
    deps[package] = part["ver"]
    deps[fp_package] = next(ver for name, ver in inv["pkgs"].values() if name == fp_package)
    places = [{"at_mm": [MARGIN + i * (2 * PITCH + 3) + (j % 2) * PITCH, MARGIN + (j // 2) * PITCH], "rotate": angle, "side": side}
              for j, (side, angle) in enumerate(FOUR)]
    rows.append({"name": "Cand_%s_%s" % (key.split("::", 1)[1], pn), "footprint": ident(fp_package) + "::" + fp_name,
                 "part": ident(package) + "::" + t, "pn": pn, "mfr": mapped[pn]["mfr"], "mpn": page["mpn"], "pins": pins,
                 "width_mm": BOX, "height_mm": BOX, "covers_rotation": True, "placements": places,
                 "mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate},
                 "candidate": {"rotate_deg": rotate, "read_raw_deg": dict(zip(("top_0", "top_37", "bottom_0", "bottom_37"), read)),
                               "record": RECORD},
                 "note": "Candidate supplier rotation %d for re-verification; not a binding. lib %s %s; lcsc_map rule %s"
                         % (rotate, key, entry[0], mapped[pn]["rule"]),
                 "jlc_package": page["package"], "category": page["category"], "stock": page["stock"],
                 "page_fetched": page["fetched_date"][:10]})
json.dump({"name": "preview_board_5", "dependencies": deps, "rows": rows},
          open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements")
