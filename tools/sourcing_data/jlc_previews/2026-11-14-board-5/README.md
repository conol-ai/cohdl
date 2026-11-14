# Preview board 5: assembler preview, 2026-11-14

This is the assembler's placement preview of preview board 5 (`tools/frame_boards/preview-board-5/`), the re-verify board for the three candidate rotations that preview board 4 read raw (`../2026-11-11-board-4/`, section A). Each part number is placed four times (top 0° and 37°, bottom 0° and 37°) **with its candidate applied** as the supplier mapping: C7519 `rotate 270`, C131941 `rotate 180`, C189961 `rotate 270`.

The package is `preview-board-5.zip`, SHA-256 `f6af93dcb7028bb34a60ba9c85ff9deed99e7fadecb08b67257ddf83fb25c5a6`, built by `tools/frame_boards/preview_package.py` at commit `e6a0f92` (the head of PR #122 then), with the reference designators left out of the Gerber silkscreen through `../2026-11-09-reverify-2/kicad-cli-no-refdes`. Alex approved this upload for the preview only. `jlc_preview.py` of the same commit ran it. The flow stopped at Component Placements: nothing was added to the cart and no order was placed.

**Result: all three candidates are confirmed.** On all 12 copies the model's one pin 1 mark is at our pad 1, within 11° of it, and by eye the model's leads lie on our pads. The bindings that follow are in "Bindings" at the end.

## Files

| File | What it is |
|---|---|
| `top.png`, `bottom.png` | The preview's own 2D whole-board exports, kept byte for byte (JPEG data under a `.png` name), 4096 × 1548 px, 36.1 px/mm. `bottom.png` is the view from below, mirrored left to right. |
| `checks.json` | `jlc_preview.py`'s per-instance check, as the run wrote it (attribution 2, body method 2: the defaults since 2026-11-12). `jlc_preview.py --recheck top.png bottom.png --board package/board.json --cpl package/preview_board_5-lcsc-cpl.csv` reproduces it byte for byte. |
| `measurements.json` | `measure_preview.py measure package/board.json top.bmp bottom.bmp measurements.json`, with the exports converted to BMP by `sips -s format bmp`, default method and attribution. It reproduces byte for byte; `tools/tests/test_jlc_preview.py` checks both files with `COHDL_REMEASURE_PREVIEWS=1`. |
| `crops/U1.jpg` … `crops/U12.jpg` | One 420 × 420 px crop per placement (11.6 mm square), centred on the footprint origin, cut from the BMPs of `top.png` (U1, U2, U5, U6, U9, U10) and `bottom.png` (U3, U4, U7, U8, U11, U12) and saved as JPEG for the manual review below. Bottom crops are as the export shows them, from below. |
| `package/` | Every file of the uploaded package, unpacked and unchanged: Gerber archive, LCSC BOM and CPL as uploaded, manifest, `board.json`, both lock files, `verification.json`, the checklist and `README.txt`. `shasum -a 256 -c SHA256SUMS` passes for all of them. `verification.json` says `supplier_preview: not uploaded` because it was written before the upload. |

The run wrote no lossless pixel copies; the exports above are the evidence, and the BMPs are rebuilt from them by `sips`. No file here names a local path, and none needed redacting: the package's files carry none. The run's log is not archived, because it names the directory the run wrote to; the lines quoted below are from it.

| File | SHA-256 |
|---|---|
| `top.png` | `e74ebd287d66df37c8d2ea84f0438ab2ceac90d481ceac987df9c98fe8735d70` |
| `bottom.png` | `f45364ab6959064f181e4e82a693611fb6bbdb6ac5071583758c96780b1d8dde` |
| `checks.json` | `077c8c914c0757920dd4ce6559b964476ae5eacce33dcb9ff9fcb887283283df` |
| `measurements.json` | `4906fe6f5cad093819e58d80c71c972fde140a5df7afa3282e7ac5ad1a618fec` |

**The live manifest has moved since.** `tools/frame_boards/preview-board-5/manifest.json` now pins the versions that carry the bindings below (`esd` 0.1.4, `load-switch` 0.2.1, `@contrib/audio-amp` 0.2.1), because it must equal its generator's output. Its rows, and so its BOM and CPL, are unchanged. `package/manifest.json` here is the one that was previewed, pinning `esd` 0.1.2, `load-switch` 0.2.0 and `@contrib/audio-amp` 0.2.0, and is kept as it was.

## What was uploaded

| Refs | Part | Library entry and land | Candidate | CPL angles (top 0°, top 37°, bottom 0°, bottom 37°) |
|---|---|---|---:|---|
| U1–U4 | NS4150B C189961 | `@contrib/audio-amp::AMP_NS4150B`, `SOP8P65X490X110N` | 270 | 270 / 307 / 270 / 233 |
| U5–U8 | USBLC6-2SC6 C7519 | `esd::ESD_USBLC6`, `esd::FP_SOT_23_6` | 270 | 270 / 307 / 270 / 233 |
| U9–U12 | TPS22918DBVR C131941 | `load-switch::LOADSW_TPS22918DBVR`, `load-switch::FP_TI_DBV0006A` | 180 | 180 / 217 / 180 / 143 |

The designators and part numbers are those of `package/preview_board_5-lcsc-bom.csv` and `package/board.json`. Each row is a local part with the map row's identity on the library entry's device and land; pads, footprints and parts are the library's, unchanged. Local checks before upload: KiCad 12/12; `smt_pos` neutral and mapped outputs equal the compiler's; BOM, CPL and netlist cross-check passed (`package/verification.json`).

## The run

- Detected board: 2 layers, 40 × 110 mm ("Detected 2 layer board of 40x110mm").
- BOM step: 3 part numbers detected and confirmed. Each of the 6 top and 6 bottom designators appeared once, on its side, with its part number, selected. No stock warning.
- `jlc_preview.py` exited 0: "Instances: 12 with their pin 1 mark at pad 1, 0 chip bodies along their pads; unverified: none; manual review: none".

## Results

Pin 1 mark (`measurements.json`): `delta_ccw_from_top` is the preview's pin 1 mark counter-clockwise from our pad 1 about the footprint origin, seen from the top. Each part has exactly one mark of its own.

| Part | Ref | Side, angle | Board 4, raw | Board 5, `delta_ccw_from_top` | Mark to pad 1 |
|---|---|---|---:|---:|---:|
| C189961 NS4150B (`rotate 270`) | U1 | top 0° | 94.8 | 4.8 | 0.66 mm |
| | U2 | top 37° | 95.2 | 5.0 | 0.65 mm |
| | U3 | bottom 0° | 264.9 | 354.8 | 0.70 mm |
| | U4 | bottom 37° | 265.3 | 354.5 | 0.70 mm |
| C7519 USBLC6-2SC6 (`rotate 270`) | U5 | top 0° | 100.1 | 10.6 | 0.56 mm |
| | U6 | top 37° | 100.8 | 10.0 | 0.53 mm |
| | U7 | bottom 0° | 259.7 | 349.4 | 0.53 mm |
| | U8 | bottom 37° | 259.9 | 349.2 | 0.54 mm |
| C131941 TPS22918DBVR (`rotate 180`) | U9 | top 0° | 183.0 | 1.8 | 0.11 mm |
| | U10 | top 37° | 183.0 | 1.2 | 0.10 mm |
| | U11 | bottom 0° | 177.4 | 358.1 | 0.10 mm |
| | U12 | bottom 37° | 178.2 | 357.7 | 0.11 mm |

The board-4 column is the same part number's copy at the same side and angle on preview board 4, placed with no mapping (`../2026-11-11-board-4/measurements.json`). Each board-5 reading is the board-4 reading turned by the candidate (added on top, taken away from below, where a copy reads the mirror) within 1.8°: the turn is the whole correction. Taking the bottom readings as mirrors, the four copies of each part agree within 1.1°.

## Manual review and limits

Each crop was looked at by eye (`crops/`). The white dot beside each part is our silkscreen pin 1 dot; the magenta dot is the assembler's pin 1 mark.

- **C189961 (U1–U4).** The model's body is drawn with eight gull-wing leads, four down each side, each on one of our pads; its pin 1 dimple is at the upper-left corner of the body, next to our pad 1, and the magenta mark sits just outside pad 1, beside our silk dot. U2 shows the same at 37°. From below (U3, U4) the leads show on the pads in the mirrored view, with the mark at pad 1.
- **C7519 (U5–U8).** The SOT-23-6 model shows six leads, three a side, each on one of our pads, with its pin 1 dimple at the pad 1 corner. The mark sits outboard of the pad 1 lead, in line with pad 1 across the row but farther from the origin, so its ray from the origin is 10° off pad 1's; that position, not a turned body, is the 10° (board 4 read the same 10° past the quarter turn). The leads and the dimple decide it.
- **C131941 (U9–U12).** The SOT-23-6 model shows six leads, three a side, on our pads, with its pin 1 dimple at the pad 1 corner and the mark on pad 1 itself (0.10 mm).
- **Reference text.** None is in the Gerber silkscreen, so nothing hides a lead.
- **Limits.** The preview's model is the assembler's drawing, not the part; this shows how the assembler will place it, which is what a mapping is for. A pin 1 mark angle alone does not show that the leads are on the pads; for each part that was read by eye on all four copies.

## Bindings

Each candidate passed on all four copies, so each part number is bound, with a frame row citing this README (mapping and measured both the candidate) and the board-4 readings in its exception:

| Part number | Binding | Frame row | Package |
|---|---|---|---|
| C7519 | `esd::ESD_USBLC6` primary, `supplier lcsc "C7519" rotate 270` | `pb5-fpsot236-c7519` | `esd` 0.1.4 (0.1.3 plus this binding) |
| C131941 | `load-switch::LOADSW_TPS22918DBVR` primary, `supplier lcsc "C131941" rotate 180` | `pb5-fptidbv0006a-c131941` | `load-switch` 0.2.1 |
| C189961 | `@contrib/audio-amp::AMP_NS4150B` primary, `supplier lcsc "C189961" rotate 270` | `pb5-sop8p65x490x110n-c189961` | `@contrib/audio-amp` 0.2.1 |

**Each mapping is the part number's, not the land's.** On C2 frame board 1 (`../2026-10-30-c2-board-1/`) other part numbers on SOT-23-6 lands read differently: C85364 on `esd::FP_SOT_23_6` needs 180, and C12044 on `load-switch::FP_TI_DBV0006A` needs 270. Neither those rows nor these say anything about another part number on the same land.
