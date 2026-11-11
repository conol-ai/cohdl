"""Second local re-verification board for the PR #78-#101 preview findings.

Candidate supplier rotations only (not bindings), both 270:

- RS0104 C5372421: the first re-verify board (2026-11-08) read pin 1 within
  3.3 degrees, but the reference text hid the top edge's two leads.  This board
  leaves the reference designators out of the Gerber silkscreen
  (kicad-cli --exclude-refdes, through KICAD_CLI_WRAPPER) so every edge shows.
- W25Q32 C32246052: in the 2026-11-08 probe preview JLC's model body is
  2.0 mm wide and 3.0 mm tall, where the datasheet's top view (Rev E, PDF
  pages 6 and 78) and our land are 3.0 x 2.0, and its pin 1 indent and mark sit
  at our pad 4's corner.  A mirror cannot swap width and height; a quarter
  turn counter-clockwise does both, so the candidate is 270.

Same four placements per part as before: top and bottom, 0 and 37 degrees.
Pads, footprints and parts are the library's, unchanged.

Usage: python3 -I build_reprev2.py WORKTREE PROBE_MANIFEST OUTDIR EXTRA_PAGES.json KICAD_CLI_WRAPPER
"""
import hashlib, json, sys, tomllib
from pathlib import Path

root, probe, out, extra, wrapper = (Path(a).resolve() for a in sys.argv[1:6])
sys.path.insert(0, str(root / "tools/frame_boards"))
import preview_package as pp

CANDIDATES = {"C5372421": 270, "C32246052": 270}
probe_rows = {r["pn"]: r for r in json.loads(probe.read_text())["rows"]}
pages = {p["c"]: p for p in json.loads(extra.read_text())}

rows, deps = [], {"std": "0.3.0"}
for i, (pn, rotate) in enumerate(CANDIDATES.items()):
    src = probe_rows[pn]
    assert pages[pn]["stock"] >= 30, (pn, pages[pn]["stock"])
    namespace = src["footprint"].split("::")[0]
    package = {"contrib_level_shifter": "@contrib/level-shifter"}.get(namespace, namespace)
    deps[package] = tomllib.loads((root / "lib" / package / "cohdl.toml").read_text())["package"]["version"]
    places = [{"at_mm": [10 + i * 35 + (j % 2) * 16, 10 + (j // 2) * 16], "rotate": angle, "side": side}
              for j, (side, angle) in enumerate([("top", 0), ("top", 37), ("bottom", 0), ("bottom", 37)])]
    rows.append({"name": "Cand_%d" % (i + 1), "footprint": src["footprint"], "part": src["part"], "pn": pn,
                 "mfr": src["mfr"], "mpn": src["mpn"], "pins": src["pins"], "width_mm": 14, "height_mm": 14,
                 "covers_rotation": True, "placements": places,
                 "mapping": {"dx_mm": 0, "dy_mm": 0, "rotate_deg": rotate},
                 "note": "Candidate supplier rotation %d for re-verification; not a binding." % rotate})

board = out / "pr78-101-reverify-2"
board.mkdir(parents=True, exist_ok=True)
(board / "manifest.json").write_text(json.dumps({"name": "pr78_101_reverify_2", "dependencies": deps, "rows": rows}, indent=2) + "\n")

work = out / "build"
work.mkdir(exist_ok=True)
k = Path("/Applications/KiCad/KiCad.app/Contents")
zip_path = out / "pr78-101-reverify-2.zip"
verification = pp.build(str(board), zip_path, str(root / "target/debug/cohdl"), str(wrapper),
                        str(k / "Frameworks/Python.framework/Versions/Current/bin/python3"), False, work)
(out / "verification.json").write_text(json.dumps(verification, indent=2) + "\n")
print(json.dumps({"zip": str(zip_path), "sha256": hashlib.sha256(zip_path.read_bytes()).hexdigest(),
                  "placements": verification["placements"], "checks": verification["checks"]}, indent=2))
