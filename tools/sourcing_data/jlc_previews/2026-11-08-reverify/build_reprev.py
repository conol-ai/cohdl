"""Local re-verification board for the 2026-11-08 PR #78-#101 probe preview.

Candidate supplier rotations only (not bindings): RS0104 270 (delta 86-ish,
(360 - delta) mod 360 rounded to 90), and 180 for the three diodes whose JLC
cathode read on +x.  Same four placements per part as the probe: top and
bottom, 0 and 37 degrees.  W25Q32 is left out until its model's pin
numbering is checked.

Usage: python3 -I build_reprev.py WORKTREE PROBE_MANIFEST OUTDIR EXTRA_PAGES.json

EXTRA_PAGES is #83's 2026-11-07 confirmed pages (2f9e279), which hold RS0104's stock.
"""
import hashlib, json, sys, tomllib
from pathlib import Path

root, probe, out, extra = (Path(a).resolve() for a in sys.argv[1:5])
sys.path.insert(0, str(root / "tools/frame_boards"))
import preview_package as pp

CANDIDATES = {"C5372421": 270, "C7420331": 180, "C7420365": 180, "C7420320": 180}
probe_rows = {r["pn"]: r for r in json.loads(probe.read_text())["rows"]}
pages = {p["c"]: p for p in json.loads((root / "tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json").read_text())}
pages.update({p["c"]: p for p in json.loads(extra.read_text())})

rows, deps = [], {"std": "0.3.0"}
for i, (pn, rotate) in enumerate(CANDIDATES.items()):
    src = probe_rows[pn]
    assert pages[pn]["stock"] >= 30, (pn, pages[pn]["stock"])
    namespace = src["footprint"].split("::")[0]
    package = {"contrib_level_shifter": "@contrib/level-shifter"}.get(namespace, namespace)
    deps[package] = tomllib.loads((root / "lib" / package / "cohdl.toml").read_text())["package"]["version"]
    part, pins = (src["part"], src["pins"]) if pn == "C5372421" else ("ReviewDiode", ["P1", "P2"])
    places = [{"at_mm": [10 + i * 35 + (j % 2) * 16, 10 + (j // 2) * 16], "rotate": angle, "side": side}
              for j, (side, angle) in enumerate([("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37)])]
    rows.append({"name": "Cand_%d" % (i + 1), "footprint": src["footprint"], "part": part, "pn": pn,
                 "mfr": src["mfr"], "mpn": src["mpn"], "pins": pins, "width_mm": 14, "height_mm": 14,
                 "covers_rotation": True, "placements": places,
                 "mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate},
                 "note": "Candidate supplier rotation %d for re-verification; not a binding." % rotate})

board = out / "pr78-101-reverify"
board.mkdir(parents=True, exist_ok=True)
(board / "manifest.json").write_text(json.dumps({"name": "pr78_101_reverify", "dependencies": deps, "rows": rows}, indent=2) + "\n")

DEVICE = "pub device ReviewDiode { pins {\nrequired P1: 1 [passive]\nrequired P2: 2 [passive]\n} }\nimpl IC for ReviewDiode {}\n"
original = pp.jlc_frame_board.emit_project
def emit(manifest, project, spacing, supplier="lcsc"):
    result = original(manifest, project, spacing, supplier)
    (project / "src/review_devices.cohdl").write_text(DEVICE)
    return result
pp.jlc_frame_board.emit_project = emit

work = out / "build"
work.mkdir(exist_ok=True)
k = Path("/Applications/KiCad/KiCad.app/Contents")
zip_path = out / "pr78-101-reverify.zip"
verification = pp.build(str(board), zip_path, str(root / "target/debug/cohdl"), str(k / "MacOS/kicad-cli"),
                        str(k / "Frameworks/Python.framework/Versions/Current/bin/python3"), False, work)
(out / "verification.json").write_text(json.dumps(verification, indent=2) + "\n")
print(json.dumps({"zip": str(zip_path), "sha256": hashlib.sha256(zip_path.read_bytes()).hexdigest(),
                  "placements": verification["placements"], "checks": verification["checks"]}, indent=2))
