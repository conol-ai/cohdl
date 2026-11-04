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
   - uploads the Gerber, then switches on PCBA (Both Sides when there are bottom parts);
   - uploads the BOM and CPL;
   - requires the package's part numbers, every row selected, no stock warning, and every top and bottom designator;
   - exports the 2D top and bottom views into `~/.cache/cohdl-preview/runs/<package>/preview/`.

   Each export must show a pin 1 mark for every part on its side, or it is taken again.
   - **Exit 2:** a person must take over (sign-in, captcha, a dialog, a page the script cannot place). Say what it printed, wait, then rerun.
   - **Exit 1:** a check failed. The preview is **unverified**; report it, and do not work around it.
3. **Measure:**
   ```sh
   R=~/.cache/cohdl-preview/runs/<package>
   $K/Frameworks/Python.framework/Versions/Current/bin/python3 -I tools/frame_boards/measure_preview.py board <board>/out/<name>.kicad_pcb $R/board.json
   (cd $R/preview && sips -s format bmp top.png --out top.bmp && sips -s format bmp bottom.png --out bottom.bmp)
   python3 -I tools/frame_boards/measure_preview.py measure $R/board.json $R/preview/top.bmp $R/preview/bottom.bmp $R/measurements.json
   ```
   The `.kicad_pcb` comes from `jlc_frame_board.py <manifest> -o DIR --skip-kicad` at the package's commit. Its supplier CPL must hash as the package's.
4. **Read the results.**
   - `delta_ccw_from_top` is the preview's pin 1, counter-clockwise from ours, seen from the top. The binding's `rotate` should leave it near 0; single readings scatter by up to about 10°.
   - A BGA shows two marks near A1, and the script leaves its angle empty. Read it by hand from the mark nearest our ball A1, and check that no mark sits near another corner. Say in the record that it was read by hand.
   - A part not placed (stock), a missing or ambiguous mark, or a failed measurement is **unverified**, never passed. Opening the preview is not a pass.
5. **Record** in `tools/sourcing_data/jlc_previews/<date>-<board>/`. Follow `2026-11-04-lcsc-batch-1/`:
   - `README.md`: what was uploaded, with hashes; the settings; the BOM step; the method; a results table; what it shows; and what is not verified;
   - `top.jpg` and `bottom.jpg`, re-encoded from the exports;
   - `measurements.json`.

   Then update the frame rows or bindings the record supports, and say which.

## Files

- `tools/frame_boards/preview_package.py`: the package, built only after every local check passes.
- `tools/frame_boards/jlc_preview.py`: the browser flow. Its selectors were taken from the live pages on 2026-11-04, so when the site changes the script stops with exit 2. Then inspect the page through the same `execute javascript` channel, fix the selector, and add a test in `tools/tests/test_jlc_preview.py`.
- `tools/frame_boards/measure_preview.py`: the pin 1 measurement.
