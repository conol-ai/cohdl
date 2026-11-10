# PR #78–#101 re-verify board 1: assembler preview, 2026-11-08

This is the assembler's placement preview of the first board built to re-verify the candidate supplier rotations found on the 2026-11-08 probe preview of PRs #78–#101. It carries four part numbers, each placed four times: top 0° and 37°, bottom 0° and 37°.

The package is `pr78-101-reverify.zip`, SHA-256 `24cafbedab411621ebd8038ba47e7ca5108ea6ce526560e9f1e74f7a2d5ecaac`. It was built by `tools/frame_boards/preview_package.py` from a local merge of #101 `59c646d` and #95 `bf3a5eb` (package commit `6a68c88`), through `build_reprev.py` here. Alex approved this upload for the preview only. `jlc_preview.py` and `measure_preview.py` came from #79 (blob `dd56fb46b2`), run from a scratch copy. The flow stopped at Component Placements: nothing was added to the cart and no order was placed.

## Files

| File | What it is |
|---|---|
| `top.png`, `bottom.png` | The preview's own 2D whole-board exports, kept byte for byte (JPEG data under a `.png` name). `bottom.png` is the view from below, mirrored left to right. |
| `measurements.json` | `measure_preview.py measure package/board.json top.bmp bottom.bmp`, with the PNGs converted to BMP by `sips`. |
| `checks.json` | `jlc_preview.py`'s per-instance coverage check. |
| `body.json` | Body outline readings, added 2026-11-10 with `../2026-11-09-reverify-2/body_axes.py` on the same BMPs and `package/board.json`. |
| `build_reprev.py` | The script that wrote the board manifest and built the package. |
| `package/` | Every file of the uploaded package, unpacked: Gerber archive, LCSC BOM and CPL as uploaded, manifest, `board.json`, both lock files, `verification.json`, the checklist and `README.txt`. `SHA256SUMS` covers the others (`shasum -a 256 -c SHA256SUMS`). `README.txt` and `verification.json` name the scratch directory the package was built in; they are kept unchanged so the sums hold. `verification.json` says `supplier_preview: not uploaded` because it was written when the package was made, before the upload. |

| Export | SHA-256 |
|---|---|
| `top.png` | `a61241e22fe9e36ca2d5a758670dd62ea5e14c0c3797bf16105509525d0a8b12` |
| `bottom.png` | `2e7302111e060cecd5d06c71acbf289e26532225e64d916e7cac57f05c2ede2c` |
| `measurements.json` | `196745011949aedcb560e6a8a710b663a023e857a5c8de24d4fc2451438686c6` |

## What was uploaded

Each CPL row carries the candidate rotation (the CPL angles are 270/307/270/233 for RS0104 and 180/217/180/143 for the diodes). The diodes have no library part, so the board declares a two-pin `ReviewDiode` device on the library's land (`package/manifest.json`).

| Refs | Part | Footprint | Candidate |
|---|---|---|---:|
| U1–U4 | C5372421 RS0104YTQF14 | `contrib_level_shifter::QFN14N50P350X350_1EP200X150` (`@contrib/level-shifter` 0.2.0) | 270 |
| U5–U8 | C7420331 B5819WS | `sod::SOD2613X110N` (`sod` 0.1.0) | 180 |
| U9–U12 | C7420365 SS34 | `do214::DIOM5026X241N` (`do214` 0.1.0) | 180 |
| U13–U16 | C7420320 1N4148WT | `sod::SOD1608X70N` (`sod` 0.1.0) | 180 |

The candidates came from the probe: RS0104 read 86.1–86.2° on top and 273.5–273.8° from below, so `(360 − delta) mod 360` to the nearest quarter turn is 270; the three diodes showed JLC's "−" and cathode band at +x, against our cathode (pad 1) at −x.

Local checks before upload: KiCad 16/16; `smt_pos` neutral and mapped outputs equal the compiler's; BOM, CPL and netlist cross-check passed (`package/verification.json`).

## BOM step

4 part numbers detected and confirmed. Each of the 8 top and 8 bottom designators appeared once, on its side, with its part number, selected. No stock warning. Detected board: 2 layers, 40 × 145 mm. The preview's self-check found a pin 1 mark on all 16 placements.

## Results

| Part | Candidate | Placements | Reading | Result |
|---|---:|---|---|---|
| RS0104YTQF14 C5372421 | 270 | U1 top 0°, U2 top 37°, U3 bottom 0°, U4 bottom 37° | `delta_ccw_from_top` 357.5, 356.7, 2.7, 2.6 | Pin 1 direction within 3.3° on all four. Full pin correspondence not shown on this board (see below) |
| B5819WS C7420331 | 180 | U5–U8 | "−" mark and cathode band on our cathode silk bar side on all four | Pass, read by hand |
| SS34 C7420365 | 180 | U9–U12 | Same | Pass, read by hand |
| 1N4148WT C7420320 | 180 | U13–U16 | Same | Pass, read by hand |

Body readings (`body.json`), against each land's axis:

| Part | Refs | Long axis vs pad axis (mod 180) | Centre offset | Offset limit |
|---|---|---|---|---|
| RS0104 (square) | U1–U4 | edges within 0.3° (mod 90) | ≤ 0.040 mm | — |
| B5819WS | U5–U8 | 3.3°, −2.7°, 3.0°, 0.0° | ≤ 0.059 mm | 0.25 mm |
| SS34 | U9–U12 | 0.3°, 0.3°, 0.0°, −0.6° | ≤ 0.078 mm | 0.25 mm |
| 1N4148WT | U13–U16 | 1.2°, 3.8°, −2.4°, 1.9° | ≤ 0.060 mm | 0.18 mm |

The body lengths in `body.json` vary between copies because the reference text and the lighter cathode band are not counted as body; the axis and centre are what the rows use.

## Manual review and limits

- **RS0104 pin distribution.** On the top view of U1 the model shows 5 leads on the left edge, 5 on the right and 2 on the bottom. The top edge is covered by the reference text and the pin 1 dot, which sits on our pad 1 (top edge, left of centre). The model's own pin 1 dimple is in the same corner. From below, the body hides the leads. This fits 5/5/2/2 but does not show the top edge, so this board supports the pin 1 direction only. The second board (`../2026-11-09-reverify-2/`) leaves the reference text off and shows the top edge.
- **Diodes.** Each diode model shows a "+" and a "−", so `measure_preview.py` gives no angle for them; that is correct and is not to be turned into an automatic pass. They were read by hand from crops of `top.png` and `bottom.png`: on every placement the "−" and the lighter cathode band are at the end next to our cathode silk bar, and the "+" at the other end.
- **SS34's model** is labelled "SMAJ12A" in the preview, probably a reused SMA body model. This record shows only the polarity of the chosen C number in this preview. It does not check the real part's identity from the model's text.
- Left out of this board: W25Q32 (its model's pin numbering was checked first; it is on the second board), 2N7002W and AMS1117 (manual review), and the bidirectional TVS parts (their release rule was Tony's to make).

## Since the preview (2026-11-10)

PRs #78–#101 merged on 2026-11-09. On `main`, `lib/sod`, `lib/do214` and `lib/@contrib/level-shifter` are identical to the sources this package was built from (`git diff 59c646d main -- lib/sod lib/do214` and `git diff bf3a5eb main -- lib/@contrib/level-shifter` are empty).

- **RS0104 C5372421** is bound in `@contrib/level-shifter` 0.2.1 with `rotate 270`; its frame row cites the second board, which also shows the full lead correspondence. This board's readings agree with it within 0.5°.
- **B5819WS, SS34 and 1N4148WT** have frame rows in `jlc_frames.json` with `rotate 180`, citing this record: `rv1-sod2613x110n-c7420331`, `rv1-diom5026x241n-c7420365` and `rv1-sod1608x70n-c7420320`, on `sod` and `do214` 0.1.0. They bind nothing yet. `sod` and `do214` ship lands only, and no library part lists these part numbers, so there is no AVL entry to carry `supplier lcsc`. When a part is added for one of them on the same land and version, its binding takes the row's `rotate 180`.
