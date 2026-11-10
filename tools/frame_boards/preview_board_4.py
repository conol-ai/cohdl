"""Preview board 4 manifest: OpenWalkieTalkie's unbound parts, a calibration set and W3 samples.

Usage: python3 -I tools/frame_boards/preview_board_4.py OUT.json

The board takes its part numbers from an inventory of a real order, the
OpenWalkieTalkie Rev B and r6 assembly packages (plan item 5a, 2026-11-11):
the library binds about 5% of that order's placements. It carries four sets,
marked `purpose` on each row:

A  order numbers whose library part exists and which lcsc_map.json already
   maps, so a preview reading is all a binding still lacks. This includes
   the order's two SOT-23-6 parts (USBLC6-2SC6, TPS22918DBVR), on lands where
   board 1 read a per-model correction.
B  the bound flash C97521 as the library ships it (`library_part`): its frame
   row was measured before soic 0.3.0 moved the land, and the order's CPL
   angle disagrees with it.
E  order numbers with a confirmed page in the local catalog snapshot but no
   map row and no library entry yet (UNI-ROYAL 0402WGF resistors, Samsung and
   Fenghua MLCCs). Each reuses the library device of its value for the pins,
   as board 1 did for C25744. The reading is evidence for (land, part
   number); the binding still needs the entry and the map row.
C  the calibration set proposed in docs/library-supplier-w3-sampling.md
   section 5 (#104): ten part numbers, each at 0, 90, 180 and 270 degrees and
   at 37 degrees, on both sides. An export shift stays fixed in the board
   frame; a model's own offset turns with the part.
D  the samples the W3 report of 2026-11-10 chose for two groups that need
   samples: Vishay CRCW 0402 (9) and Yageo RC 1210 (42), one top copy at 0
   degrees each, as on the W3 pilot.

A, B and E rows are placed four times: top 0 and 37, bottom 0 and 37.

Every part number is a confirmed page already in the repository: the B2 full
run (lcsc_backfill/2026-11-07-samsung-b1) or the W1 snapshot of 2026-10-30.
Nothing is fetched. A page must read at least five pieces per placement (the
assembler builds five boards) and at least 30. LEFT_OFF lists the order
numbers that qualify otherwise but are not placed, with the reason.

The W3 samples and calibration choices are written out below, because the
report they come from (tools/sourcing_data/w3_sampling/2026-11-10/groups.json,
SHA-256 5b0f77f2...) is on #104's branch and not in this tree. Each is still
checked here against the map, the pages and the library.
"""
import json, math, re, subprocess, sys, tempfile
from pathlib import Path
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
DATA = ROOT / "tools/sourcing_data"
with tempfile.TemporaryDirectory() as td:
    subprocess.run([sys.executable, "-I", str(HERE / "inventory.py"), str(Path(td) / "inv.json")],
                   check=True, capture_output=True)
    inv = json.load(open(Path(td) / "inv.json"))
pages = {r["c"]: r for r in json.load(open(DATA / "jlc_catalog/2026-10-30/catalog.json"))}
pages.update({r["c"]: r for r in json.load(open(DATA / "lcsc_backfill/2026-11-07-samsung-b1/confirmed.json"))})
mapped = {r["pn"]: r for r in json.load(open(DATA / "lcsc_map.json"))["rows"]}
framed = {r["pn"] for r in json.load(open(DATA / "jlc_frames.json"))["rows"]}

# (part number, library part as package::PART, purpose, note). The library part
# is the entry's own part for A, B, C and D; for E it only supplies the pins.
SEL = [
    ("C113780", "passive::C_1n_50V_C0G_0402", "A", "OWT C6, C32-C39, C50 (10)"),
    ("C19666", "passive::C_4u7_16V_X5R_0603", "A", "OWT C18, C40, C41, C48 (4; r6 9); the design instantiates C_4u7_10V_X5R_0603, the entry is on the 16 V part"),
    ("C107006", "passive::C_330p_50V_C0G_0402", "A", "OWT C8-C10 (3)"),
    ("C23630", "passive::C_2u2_16V_X5R_0603", "A", "OWT r6 only, C14, C16, C23, C24, C27, C31 (6); the design instantiates C_2u2_10V_X5R_0603, the entry is on the 16 V part"),
    ("C52923", "passive::C_1u_25V_X5R_0402", "A", "OWT C15, C26 (2); the design declares the part locally"),
    ("C106255", "passive::C_470n_10V_X5R_0402", "A", "OWT C25"),
    ("C107039", "passive::C_1n5_50V_C0G_0603", "A", "OWT C2"),
    ("C337893", "passive::IND_2U2", "A", "OWT L4, L5 (2)"),
    ("C97018", "passive::IND_4U7_WPN201610U", "A", "OWT L3"),
    ("C7519", "esd::ESD_USBLC6", "A", "OWT U20, SOT-23-6; board 1 read 180 for C85364 on this land"),
    ("C131941", "load-switch::LOADSW_TPS22918DBVR", "A", "OWT U2, U10 (2), SOT-23-6; board 1 read 270 for C12044 on this land"),
    ("C189961", "@contrib/audio-amp::AMP_NS4150B", "A", "OWT U1"),
    ("C97521", "flash::FLASH_W25Q128JVSIQ", "B", "OWT U4; library binding as shipped in flash 0.3.0, land moved by soic 0.3.0"),
    ("C25741", "passive::R_100K_F_0402", "E", "OWT 100k (14); UNI-ROYAL 0402WGF1003TCE"),
    ("C11702", "passive::R_1K_F_0402", "E", "OWT 1k (12); UNI-ROYAL 0402WGF1001TCE"),
    ("C26083", "passive::R_1M_F_0402", "E", "OWT 1M (5); UNI-ROYAL 0402WGF1004TCE"),
    ("C17168", "passive::R_0R_J_0402", "E", "OWT 0R (3); UNI-ROYAL 0402WGF0000TCE"),
    ("C25768", "passive::R_22K_F_0402", "E", "OWT 22k (2); UNI-ROYAL 0402WGF2202TCE"),
    ("C25905", "passive::R_5K1_F_0402", "E", "OWT 5.1k (2); UNI-ROYAL 0402WGF5101TCE"),
    ("C25092", "passive::R_22R_F_0402", "E", "OWT 22R; UNI-ROYAL 0402WGF220JTCE"),
    ("C25890", "passive::R_3K3_F_0402", "E", "OWT 3.3k; UNI-ROYAL 0402WGF3301TCE"),
    ("C23733", "passive::C_4u7_6V3_X5R_0402", "E", "OWT 4.7uF 0402 (5); Samsung CL05A475MP5NRNC, a 20% part"),
    ("C1548", "passive::C_15p_50V_C0G_0402", "E", "OWT C57; Fenghua 0402CG150J500NT"),
]
# Calibration (C): parts earlier previews drew, with what they read there.
CAL = [
    ("C60491", "Yageo RC 0402; pilot -3.0 deg, raw 0.070 mm"),
    ("C113781", "Yageo CC 0402; pilot +10.5 deg, left for manual review"),
    ("C481918", "Vishay CRCW 0402; pilot -7.3 deg, raw 0.068 mm"),
    ("C138033", "Yageo RC 0402; pilot +9.9 deg, raw 0.117 mm"),
    ("C185445", "Yageo RC 0402; pilot +8.1 deg, raw 0.121 mm, the largest"),
    ("C325464", "Yageo CC 0402; pilot -0.3 deg, raw 0.082 mm"),
    ("C482097", "Vishay CRCW 0402; pilot +7.5 deg, raw 0.100 mm"),
    ("C138161", "Yageo RC 0201; board 3 residual 0.046-0.064 mm pauses its group"),
    ("C877226", "Yageo RC 2010; board 3 offset about 0.18 mm that turns with the part"),
    ("C878096", "Yageo RC 2512; board 3 mixed"),
]
CAL_ANGLES = (0, 90, 180, 270, 37)
# W3 report 2026-11-10, `sample` of each group, in the report's order.
W3 = [
    ("CHIP_0402 ChipResistor Vishay CRCW", ["C482245", "C2076669", "C2085363", "C482122", "C482153", "C3151635",
                                             "C4112969", "C482229", "C3008837"]),
    ("CHIP_1210 ChipResistor Yageo RC", ["C375531", "C728359", "C723673", "C137103", "C275640", "C874782", "C470029",
                                          "C728367", "C874830", "C723647", "C874855", "C137098", "C137097", "C728373",
                                          "C728375", "C874966", "C723648", "C874999", "C728383", "C875026", "C728387",
                                          "C137092", "C875093", "C875102", "C875123", "C728395", "C137089", "C728398",
                                          "C875203", "C875206", "C137083", "C875217", "C326901", "C875225", "C875232",
                                          "C875234", "C875236", "C440313", "C728409", "C326893", "C875251", "C723577"]),
]
# Order numbers that would qualify but are not placed.
LEFT_OFF = {
    "C2799946": "OWT SW3, SW4 (C&K PTS636, in lcsc_map): its page read stock 0 on 2026-11-06, and the assembler does not place such a part",
    "C2917717": "OWT AE1 (Johanson 2450AT18B100E): its page read stock 0 on 2026-11-06; not in lcsc_map",
    "C25744": "OWT 10k (UNI-ROYAL 0402WGF1002TCE): already has frame row c2b1-r0402-c25744",
    "C15849": "OWT 1uF 0603 (Samsung CL10A105KB8NNNC): already has frame row c2b1-c0603-c15849",
}
GAP, WIDTH = 2.0, 120.0
ENTRY = re.compile(r'(primary|alt) \{ mfr: "([^"]+)", mpn: "([^"]+)"')


def ident(package):
    """Source spelling of a package name: "@contrib/audio-amp" is contrib_audio_amp."""
    return package.lstrip("@").replace("/", "_").replace("-", "_")


# Every library part: its user row (device type, footprint) and its AVL entries.
parts = {}
for footprint, users in inv["users"].items():
    for user in users:
        parts[user["pkg"] + "::" + user["part"]] = dict(user, footprint=footprint)
by_identity = {}
for directory, (package, _) in sorted(inv["pkgs"].items()):
    for path in sorted((ROOT / "lib" / directory / "src").rglob("*.cohdl")):
        for m in re.finditer(r"\bpub part (\w+): [^{]+\{(.*?)\n\}", path.read_text(encoding="utf-8"), re.S):
            key = package + "::" + m.group(1)
            if key in parts:
                parts[key]["entries"] = ENTRY.findall(m.group(2))
                parts[key]["bound"] = set(re.findall(r'supplier lcsc "(C\d+)"', m.group(2)))
                for kind, mfr, mpn in parts[key]["entries"]:
                    by_identity.setdefault((mfr, mpn), []).append(key)


def reach(footprint):
    cx, cy, w, h = map(float, inv["footprints"][footprint]["courtyard"])
    return math.hypot(abs(cx) + w / 2, abs(cy) + h / 2)


deps = {"std": "0.3.0"}


def row(pn, key, purpose, note, placements):
    """A manifest row without coordinates; PLACEMENTS is its (side, angle) list."""
    part, page = parts[key], pages[pn]
    need = max(30, 5 * len(placements))
    if page["stock"] < need:
        raise SystemExit(f"{pn}: its page reads {page['stock']} in stock, the board needs {need}")
    entry = None
    if pn in mapped:
        entry = next((e for e in part["entries"] if (e[1], e[2]) == (mapped[pn]["mfr"], mapped[pn]["mpn"])), None)
        if entry is None or page["mpn"] != mapped[pn]["mpn"]:
            raise SystemExit(f"{pn}: {key} has no entry {mapped[pn]['mfr']} {mapped[pn]['mpn']}")
    elif purpose != "E":
        raise SystemExit(f"{pn}: not in lcsc_map.json")
    if purpose == "E" and (pn in framed or (page["mfr"], page["mpn"]) in by_identity):
        raise SystemExit(f"{pn}: already has a frame row or a library entry; it is not an E row")
    package, footprint = part["pkg"], part["footprint"]
    fp_package, fp_name = footprint.split("::", 1)
    t = part["type"]
    base = t.split("<")[0].split("[")[0].strip()
    variant = t[t.index("["):] if "[" in t else ""
    devices = inv["devices"]
    pins = devices.get(package + "::" + base + variant, devices.get(package + "::" + base, ["A", "B"])) or ["A", "B"]
    deps[package] = part["ver"]
    deps[fp_package] = next(ver for name, ver in inv["pkgs"].values() if name == fp_package)
    side = round(2 * reach(footprint), 3)
    out = {"name": "%s_%s_%s" % (purpose, key.split("::", 1)[1], pn), "footprint": ident(fp_package) + "::" + fp_name,
           "part": ident(package) + "::" + t, "pn": pn,
           "mfr": mapped[pn]["mfr"] if pn in mapped else page["mfr"], "mpn": page["mpn"], "pins": pins,
           "width_mm": side, "height_mm": side, "covers_rotation": True, "placements": placements,
           "purpose": purpose, "note": note, "jlc_package": page["package"], "category": page["category"],
           "stock": page["stock"]}
    if purpose == "B":
        if pn not in part["bound"]:
            raise SystemExit(f"{pn}: {key} does not bind it")
        out["library_part"] = ident(package) + "::" + key.split("::", 1)[1]
    elif purpose == "E":
        out["note"] += "; device supplies pins only; no map row, no library entry"
    else:
        out["note"] += "; lib %s %s; lcsc_map rule %s" % (key, entry[0], mapped[pn]["rule"])
    return out


def from_map(pn):
    """The one library part whose entry is PN's map row."""
    hits = by_identity.get((mapped[pn]["mfr"], mapped[pn]["mpn"]), [])
    if len(hits) != 1:
        raise SystemExit(f"{pn}: {len(hits)} library entries for its map row")
    return hits[0]


FOUR = (("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37))
blocks = []  # (row, columns, rows): a block of cells, each the row's box plus GAP
for pn, key, purpose, note in SEL:
    blocks.append((row(pn, key, purpose, note, list(FOUR)), 2, 2))
blocks.sort(key=lambda b: -b[0]["width_mm"])  # stable: equal sizes keep SEL's order
calibration = [(row(pn, from_map(pn), "C", note, [(side, a) for side in ("top", "bottom") for a in CAL_ANGLES]),
                len(CAL_ANGLES), 2) for pn, note in CAL]
samples = [(row(pn, from_map(pn), "D", "W3 sample 2026-11-10, group " + group, [("top", 0)]), 1, 1)
           for group, pns in W3 for pn in pns]
for group, pns in W3:
    for pn in pns:
        if not from_map(pn).split("::")[1].endswith("_" + group.split()[0].split("_")[1]):
            raise SystemExit(f"{pn}: not a {group} part")

# Bands down the board. The calibration set is split between the first and the
# last band, so each view has calibration bodies at both ends of the board.
rows, x, y, shelf = [], 0.0, 0.0, 0.0
for band in (calibration[:4], blocks, samples, calibration[4:]):
    x, y, shelf = 0.0, y + shelf, 0.0
    for item, columns, lines in band:
        step = item["width_mm"] + GAP
        if x + columns * step > WIDTH and x > 0:
            x, y, shelf = 0.0, y + shelf, 0.0
        places = []
        for i, (side, angle) in enumerate(item["placements"]):
            places.append({"at_mm": [round(x + (i % columns + 0.5) * step, 3), round(y + (i // columns + 0.5) * step, 3)],
                           "rotate": angle, "side": side})
        item["placements"] = places
        x += columns * step
        shelf = max(shelf, lines * step)
        rows.append(item)
if len({r["pn"] for r in rows}) != len(rows) or set(LEFT_OFF) & {r["pn"] for r in rows}:
    raise SystemExit("a part number is on the board twice, or placed and left off")
json.dump({"name": "preview_board_4", "dependencies": deps, "rows": rows, "left_off": LEFT_OFF},
          open(sys.argv[1], "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(rows), "rows,", sum(len(r["placements"]) for r in rows), "placements, board ~", WIDTH, "x", round(y + shelf, 1), "mm")
