---
name: jlc-preview
description: Run the JLCPCB placement preview for a CoHDL frame or acceptance board and record its pin 1 readings. Use when a board under tools/frame_boards needs its assembler preview (after a supplier binding, a footprint move or a new frame row), or when asked to upload a preview package, take preview screenshots, or measure pin 1 on a preview.
---

# JLCPCB placement preview

The flow builds the preview package, works the assembler's web pages in the person's own Arc window, stops at Component Placements, measures pin 1 and writes the record. It never orders.

## Before you start

- **Authorization.** Uploading a package to the assembler is an external upload. Get the person's go-ahead for that specific package (name or SHA-256) first. Once given, reruns and retries of the same package need no new confirmation. A different package, or any step past Component Placements, needs a new one.
- **Never.** Never add to the cart, quote, order or pay. Never tick the assembly terms for the person. Never request, replay or read the site's `/api` traffic: the pages may make their own requests, but the script reads only the DOM (docs/library-supplier-plan.md, ground rules).
- **The person's part.** The person signs in to JLCPCB in Arc and handles any captcha. On first use, macOS asks whether the terminal may control Arc (allow it), and Arc asks whether the site may download multiple files (allow it). Do not read, record or type their credentials.
- **Screenshots.** Do not capture the screen; it shows the person's other windows. Use only the preview's own export, which the script does.

## Steps

1. **Build the package** from a clean tree (`tools/frame_boards/<board>/README.md` has the command):
   ```sh
   K=/Applications/KiCad/KiCad.app/Contents
   python3 -I tools/frame_boards/preview_package.py <board> -o <scratch>/<board>-preview-<commit>.zip \
     --cohdl target/debug/cohdl --kicad-cli $K/MacOS/kicad-cli \
     --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
   ```
   Give the person the ZIP's SHA-256 when asking for the go-ahead.
2. **Run the preview** (Arc open, the person signed in):
   ```sh
   python3 -I tools/frame_boards/jlc_preview.py <package>.zip
   ```
   The script:
   - checks the package's sums;
   - opens a quote tab of its own and works only in it, found by its window and tab ids. Other tabs, even other JLCPCB projects, are never read or navigated. If its tab is closed, moved or leaves `cart.jlcpcb.com`, the run stops;
   - uploads the Gerber, then switches on PCBA (Both Sides when there are bottom parts);
   - uploads the BOM and CPL;
   - reads the BOM step row by row. Each designator must appear exactly once, on its CPL side, in a selected row matched to its BOM part number. The counts of parts detected and confirmed must match, and no stock warning may show;
   - exports the 2D top and bottom views into `~/.cache/cohdl-preview/runs/<package>/preview/`. The exports are JPEG data under a `.png` name. Beside each, the script keeps the decoded pixels it read as a lossless PNG (`top.pixels.png`, `bottom.pixels.png`), and it writes `preview/pixels.json`: the decoder (`sips` and macOS versions) and the SHA-256 of each export and of its pixels.

   Each instance on the side (`board.json`) needs evidence of its own, or the export is taken again:
   - **Non-polar chip parts** (`measure_preview.is_chip`: the library's `passive` `CHIP_*` lands, two pads) get no pin 1 mark from the preview. Their drawn body must lie along the pads' axis, mod 180 (within 10°), and be centred on its pads (within the lesser of 0.25 mm and a quarter of the pad half-spacing).
   - **Every other part** needs a pin 1 mark. Each mark counts once, for the nearest footprint within its reach, so a BGA's second mark cannot cover for a part that was not drawn.

   A retake stops early when two exports in a row leave the same instances without evidence: a model that is not drawn will not be drawn by a third. A large board's export can take minutes to land in Downloads; the script waits up to 8 minutes and clicks the export once more after 4. After the last retake the run goes on to the other side. An instance whose model is not drawn (bare pads and the checkered centroid marker) is **unverified**. A drawn instance without its evidence (no mark, or a chip body not along or not centred on its pads) goes to **manual review**. Neither passes. Each instance's verdict is in `preview/checks.json`. A package from before `board.json` was added takes `--board`, the output of `measure_preview.py board` for the package's `.kicad_pcb`.
   - **Exit 2:** a person must take over (sign-in, captcha, a dialog, a page the script cannot place). Say what it printed, wait, then rerun.
   - **Exit 1:** a check failed. The preview is **unverified**; report it, and do not work around it.
   - **Exit 3:** both exports were taken, but some instances are unverified or for manual review. Measure and record the rest, and list those instances with their reasons.
3. **Measure:**
   ```sh
   R=~/.cache/cohdl-preview/runs/<package>
   python3 -I tools/frame_boards/measure_preview.py measure $R/upload/board.json $R/preview/top.pixels.png $R/preview/bottom.pixels.png $R/measurements.json
   ```
   `board.json` comes in the package. For an older package, use the file passed as `--board`. Measure the `.pixels.png` files, not a fresh decode: they are the pixels the run's checks were read from, and `measure_preview.py` refuses JPEG data. (For an export that has no pixels file, `sips -s format bmp top.png --out top.bmp`, then `measure_preview.py pixels top.bmp top.pixels.png`, and record `measure_preview.py decoder`.)
4. **Read the results.**
   - `delta_ccw_from_top` is the preview's pin 1, counter-clockwise from ours, seen from the top. The binding's `rotate` should leave it near 0; single readings scatter by up to about 10°.
   - A BGA shows two marks near A1, and the script leaves its angle empty. Read it by hand from the mark nearest our ball A1, and check that no mark sits near another corner. Record it as manual review, and say so in the results table.
   - A chip part's row carries `body`: `delta_mod_180` (the body's long axis against the pads', mod 180), `offset_mm` and `body_ok`. A non-polar part's orientation is only known mod 180, and its frame row says so.
   - A part not placed (stock), a model not drawn (`"drawn": false`), a missing or ambiguous mark, or a failed measurement is **unverified**, never passed. Opening the preview is not a pass.
5. **Record** in `tools/sourcing_data/jlc_previews/<date>-<board>/`. Follow `2026-11-04-lcsc-batch-1/`:
   - `README.md`: what was uploaded, with hashes; the settings; the BOM step; the method; a results table; what it shows; and a section headed "Manual review and limits";
   - `top.png` and `bottom.png`, the exports as downloaded, with their SHA-256 (a re-encoded JPEG shifts readings by about 0.1°);
   - `top.pixels.png`, `bottom.pixels.png` and `pixels.json` from the run's `preview/` directory, with the SHA-256 of each `.pixels.png`. The exports are JPEG, and two JPEG decoders can differ by a pixel at an edge; the lossless pixels are what `measurements.json` reproduces from, value for value, on any machine. In the README, name the decoder from `pixels.json`. Records from before 2026-11-13 keep only the JPEG and reproduce with `sips` (the 2026-11-08 probe record is the exception and the model);
   - `measurements.json`.

   Then update the frame rows or bindings the record supports, and say which.

## Files

- `tools/frame_boards/preview_package.py`: the package, built only after every local check passes.
- `tools/frame_boards/jlc_preview.py`: the browser flow. Its selectors were taken from the live pages on 2026-11-04, so when the site changes the script stops with exit 2. Then inspect the page through the same `execute javascript` channel, fix the selector, and add a test in `tools/tests/test_jlc_preview.py`.
- `tools/frame_boards/measure_preview.py`: the pin 1 measurement.
