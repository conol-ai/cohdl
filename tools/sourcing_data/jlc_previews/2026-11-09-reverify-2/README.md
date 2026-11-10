# PR #78–#101 re-verify board 2: assembler preview, 2026-11-09

This is the assembler's placement preview of the second re-verify board. It carries two part numbers, each placed four times (top 0° and 37°, bottom 0° and 37°) with its candidate rotation: RS0104 again, with every edge of the model visible, and W25Q32, whose candidate came from a manual review of the probe preview (`probe-manual-review.md`).

The package is `pr78-101-reverify-2.zip`, SHA-256 `804899ed3fe0feb46dc0ea2482d8d503d7119553422c6ded1fa16aab86374644`. It was built by `tools/frame_boards/preview_package.py` from a local merge of #101 `59c646d`, #95 `bf3a5eb` and #93 `b2908da` (flash 0.3.0, the W25Q32 XSON land; package commit `38b8434`, a local merge that was never pushed: it is on no GitHub branch and not on `main`; the three PR commits are on `main`), through `build_reprev2.py` here. Alex approved this upload for the preview only. `jlc_preview.py` and `measure_preview.py` came from #79 (blob `dd56fb46b2`), run from a scratch copy. The flow stopped at Component Placements: nothing was added to the cart and no order was placed.

The one difference from a normal export: the Gerber silkscreen leaves out the reference designators (`kicad-cli pcb export gerbers --exclude-refdes`, through the `kicad-cli-no-refdes` wrapper here), so the reference text cannot hide the model's leads. Copper, mask, paste and outline are as exported normally (line for line, apart from timestamps). Pads, footprints and parts are the library's, unchanged.

## Files

| File | What it is |
|---|---|
| `top.png`, `bottom.png` | The preview's own 2D whole-board exports, kept byte for byte (JPEG data under a `.png` name), 3077 × 1688 px. `bottom.png` is the view from below, mirrored left to right. |
| `measurements.json` | `measure_preview.py measure package/board.json top.bmp bottom.bmp`, with the PNGs converted to BMP by `sips`. |
| `checks.json` | `jlc_preview.py`'s per-instance coverage check. |
| `crops/U1.png` … `crops/U8.png` | One 420 × 420 px crop per placement, cut from `top.png` (U1, U2, U5, U6) and `bottom.png` (U3, U4, U7, U8) for the manual review below. |
| `body.json`, `body_axes.py` | Body outline readings, added 2026-11-10: `python3 -I body_axes.py tools/frame_boards package/board.json top.bmp bottom.bmp U1 … U8`, on the same BMPs. The script's docstring defines each field. |
| `build_reprev2.py`, `kicad-cli-no-refdes` | The script that wrote the board manifest and built the package, and the `kicad-cli` wrapper it was given. |
| `probe-manual-review.md` | The 2026-11-09 manual review of the probe preview that set the W25Q32 candidate (and the 2N7002W and AMS1117 readings). The paths it names are local working directories; the probe's exports, package and measurements are archived in `../2026-11-08-probe/`. |
| `package/` | Every file of the uploaded package, unpacked: Gerber archive, LCSC BOM and CPL as uploaded, manifest, `board.json`, both lock files, `verification.json`, the checklist and `README.txt`. `SHA256SUMS` covers the others (`shasum -a 256 -c SHA256SUMS`). `README.txt` and `verification.json` were redacted after the upload: in each, one local directory prefix (the path of the scratch directory the package was built in) was replaced by `<scratch>`, and nothing else. So their two `SHA256SUMS` lines no longer match the files here, and the uploaded zip's SHA-256 above no longer reproduces from this tree; `shasum -a 256 -c SHA256SUMS` reports exactly those two as failed and every other file as OK. `SHA256SUMS` itself is unchanged and still holds the uploaded files' SHA-256: `README.txt` `36eadd17c44dee979f45a0e7014443720ad4700b7c5d2720fb5cf025066a2b69`, `verification.json` `4b58eabb478d9e947b06644534c1de39172a15092f00a2689fb3df6f5ae577f5`. `verification.json` says `supplier_preview: not uploaded` because it was written before the upload. |

| Export | SHA-256 |
|---|---|
| `top.png` | `faf94cdbe05f54e04c808b1c2e4717432b51148e31a05790c00a79ec46c370fa` |
| `bottom.png` | `8c5a629e8cf828109875cc49384422cebf4c31f5a0036034325acd080bf5bfde` |
| `measurements.json` | `1b400df208f3682b896463e36936f7218588992c98e01936f0afcb8838b39f41` |

## What was uploaded

| Refs | Part | Footprint | Candidate | CPL angles |
|---|---|---|---:|---|
| U1–U4 | RS0104YTQF14 C5372421 | `contrib_level_shifter::QFN14N50P350X350_1EP200X150` (`@contrib/level-shifter` 0.2.0) | 270 | 270 / 307 / 270 / 233 |
| U5–U8 | W25Q32RVXHJQ C32246052 | `flash::FP_W25Q32RV_XSON8_2X3` (`flash` 0.3.0 at `b2908da`) | 270 | 270 / 307 / 270 / 233 |

**Why W25Q32 270.** In the probe preview the assembler's model was 2.0 mm wide and 3.0 mm tall, with its pin 1 dimple and mark at the lower left, nearest our pad 4. The datasheet (Rev E, PDF page 6 Figure 1b and page 78) and our land are 3.0 mm wide and 2.0 mm tall, pin 1 top-left, pads 1–4 down the left and 5–8 up the right. A mirror keeps width and height; a quarter turn counter-clockwise swaps them and moves the top-left corner to the lower left. So the candidate was 270, with this pass condition: the body becomes 3.0 wide × 2.0 tall, and the pin 1 mark and dimple land on our pad 1. Length and width right but pin 1 at pad 4 or 5/8 would mean a mirror as well, and would not pass.

Local checks before upload: KiCad 8/8; `smt_pos` neutral and mapped outputs equal the compiler's; BOM, CPL and netlist cross-check passed (`package/verification.json`).

## BOM step

2 part numbers detected and confirmed. Each of the 4 top and 4 bottom designators appeared once, on its side, with its part number, selected. No stock warning. Detected board: 2 layers, 40 × 75 mm. The preview's self-check found a pin 1 mark on all 8 placements.

## Results

Pin 1 mark (`measurements.json`):

| Part | Ref | Side, angle | `delta_ccw_from_top` | Mark to nearest pads |
|---|---|---|---:|---|
| RS0104YTQF14 C5372421 | U1 | top 0° | 357.2 | pad 1 0.28 mm, pad 2 1.08 mm |
| | U2 | top 37° | 356.7 | pad 1 0.29, pad 2 1.10 |
| | U3 | bottom 0° | 3.2 | pad 1 0.27, pad 2 1.08 |
| | U4 | bottom 37° | 3.1 | pad 1 0.27, pad 2 1.08 |
| W25Q32RVXHJQ C32246052 | U5 | top 0° | 6.5 | pad 1 0.54, pad 2 0.74 |
| | U6 | top 37° | 6.2 | pad 1 0.52, pad 2 0.74 |
| | U7 | bottom 0° | 353.2 | pad 1 0.54, pad 2 0.74 |
| | U8 | bottom 37° | 353.0 | pad 1 0.53, pad 2 0.73 |

Body outline (`body.json`). The land axis is pad 1 to pad 8 for XSON-8 (across the pad rows, the 3.0 mm side) and pad 2 to pad 6 for QFN-14 (down the left row; the body is square, so read mod 90):

| Ref | Side, angle | Body (mm) | Long axis vs land (mod 180) | Edges vs land (mod 90) | Centre offset | Dimple: nearest pads |
|---|---|---|---:|---:|---:|---|
| U1 | top 0° | 3.47 × 3.43 | — | −0.6° | 0.030 mm | — |
| U2 | top 37° | 3.46 × 3.44 | — | 0.1° | 0.023 mm | — |
| U3 | bottom 0° | 3.42 × 3.42 | — | 0.6° | 0.009 mm | — |
| U4 | bottom 37° | 3.43 × 3.43 | — | 0.1° | 0.002 mm | — |
| U5 | top 0° | 3.00 × 1.96 | 0.1° | 0.0° | 0.042 mm | pad 1 0.28, pad 2 0.43 |
| U6 | top 37° | 2.99 × 1.96 | 0.2° | −0.2° | 0.042 mm | pad 1 0.28, pad 2 0.44 |
| U7 | bottom 0° | 2.97 × 1.98 | −0.1° | 0.8° | 0.042 mm | pad 1 0.29, pad 2 0.42 |
| U8 | bottom 37° | 2.97 × 1.96 | −0.4° | 0.0° | 0.039 mm | pad 1 0.29, pad 2 0.43 |

## Manual review and limits

- **RS0104.** With no reference text, U1's top view shows the model's leads: 5 on the left, 5 on the right, 2 on the bottom and, on the top edge, the lead right of centre. The top edge's other lead lies under JLC's pin 1 mark, which is drawn on that lead and sits on our pad 1. So 13 of the 14 leads are seen on our pads, and the 14th is the one JLC marks as pin 1, on our pad 1. The model's pin 1 dimple is in the same corner. U2 shows the same at 37°. From below, the body hides the leads; the bottom readings agree with the top. The first board (`../2026-11-08-reverify/`) and this one agree within 0.5°.
- **W25Q32.** With rotate 270 the model's body is 3.0 mm wide and 2.0 mm tall: the datasheet's top view and our land. The pads show on its left and right edges, 4 a side. The model's pin 1 dimple is at the top-left corner, and JLC's pin 1 mark sits just outside it, beside our pin 1 silk dot, nearest our pad 1. The same holds at 37° and from below (`crops/U5.png` … `U8.png`). XSON is leadless, so the model's terminals do not show; the reading rests on the body outline and the pin 1 position.
- **The W25Q32 mark angle is not the evidence.** The ray readings of 6–7° (353° from below) come from the mark sitting off the body corner, between pads 1 and 2 in distance (0.54 vs 0.74 mm), not from a turned body: the body's edges run along the pad rows within 0.8° and its long axis within 0.4°, on all four copies. The body outline and the dimple decide it.

## Since the preview (2026-11-10)

PRs #78–#101 merged on 2026-11-09. On `main`, `lib/@contrib/level-shifter` is identical to `bf3a5eb`, and `lib/flash/src/w25q32.cohdl` to `b2908da`. `lib/flash` differs from `b2908da` only in the W25Q128JVSIQ binding that #80 added and its README section; the W25Q32 land, part and device are unchanged.

- **RS0104 C5372421** is bound in `@contrib/level-shifter` 0.2.1: `supplier lcsc "C5372421" rotate 270`, frame row `rv2-qfn14n50p350x3501ep200x150-c5372421`.
- **W25Q32RVXHJQ C32246052** is bound in `flash` 0.3.1: `supplier lcsc "C32246052" rotate 270`, frame row `rv2-fpw25q32rvxson82x3-c32246052`.

Both rows cite this README; their evidence notes give the numbers above. Neither version changes a land.
