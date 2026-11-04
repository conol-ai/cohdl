# LCSC batch 1 acceptance board

The preview board for the first six library bindings (implementation plan D1). Unlike the frame boards, it declares no parts of its own: each row places the `@st/stm32` 0.3.0 part as the library ships it, so the preview checks the library's own `supplier lcsc` clause. Each part is placed four times: top 0°, top 37°, bottom 0° and bottom 37°. That makes 24 placements on a board of about 140 × 211 mm.

| Row | Library part | Entry | LCSC | Binding |
|---|---|---|---|---|
| LQFP208 | `st_stm32::MCU_STM32F767BIT6` | primary | C129276 | `rotate 270` |
| LQFP100_F4 | `st_stm32::MCU_STM32F427VGT6` | primary | C117815 | `rotate 270` |
| LQFP100_L4 | `st_stm32::MCU_STM32L496VGT6` | primary | C717701 | `rotate 270` |
| LQFP32_KICAD | `st_stm32::MCU_STM32L412K8T6` | primary | C529436 | default |
| TFBGA100 | `st_stm32::MCU_STM32H743VIH6` | alt `STM32H743VIH6TR` | C1857444 | default |
| UFBGA64 | `st_stm32::MCU_STM32G431RBI6` | primary | C730103 | default |

Pins, boxes and placements are frame board 2's (`../c2-board-2/`), whose preview of 2026-11-02 is these frames' evidence. That preview carried no maps, so it read LQFP100 and LQFP208 at rotate 270. With the library's bindings applied, every copy here should read rotate 0: pin 1 on our pin 1 and the body on our land, on both sides and at both angles. On a BGA, pin 1 is ball A1.

## Preview package

One command runs every step below and writes the upload package only when all of them pass:

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py lcsc-batch-1 -o lcsc-batch-1-preview.zip \
  --cohdl target/debug/cohdl --kicad-cli $K/MacOS/kicad-cli \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

The steps are: a clean tree; the committed manifest equals `lcsc_batch_1.py`'s output; the build with its binding check; `check_cpl.py`; and a cross-check of the BOM and CPLs. The ZIP holds `lcsc_batch_1-gerbers.zip` (copper, mask, paste, silkscreen, outline and drill), the supplier BOM and CPL, `CHECKLIST.md`, the manifest and lock files, `verification.json` and `SHA256SUMS`. KiCad stamps a creation date into each Gerber, so only the Gerber archive's hash changes between runs.

## Regenerate step by step

```sh
python3 -I tools/frame_boards/lcsc_batch_1.py tools/frame_boards/lcsc-batch-1/manifest.json
python3 -I tools/jlc_frame_board.py tools/frame_boards/lcsc-batch-1/manifest.json -o BOARD \
  --cohdl target/debug/cohdl --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
python3 -I tools/frame_boards/check_cpl.py BOARD \
  --kicad-python /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3 \
  --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
```

`jlc_frame_board.py` refuses the board unless the supplier BOM gives every copy its row's LCSC number, so a binding missing from the library cannot slip through as an unbound part. `tools/tests/test_jlc_frame_board.py` rebuilds the board with the compiler and checks each supplier CPL angle.

## Checks before upload (2026-11-04)

- With KiCad 10.0.7, `check_cpl.py` reports:
  - `smt_pos.py --neutral` equals the neutral CPL, byte for byte;
  - `smt_pos.py --map` equals the supplier CPL, byte for byte;
  - KiCad's own position export agrees in position, side and top-side angle: 24 of 24 rows. Bottom-side angles are covered only by the `smt_pos.py` comparison above, since KiCad writes them in its own encoding.
- The supplier BOM `dcfe8fae` lists the six LCSC numbers, with the H743 row on its bound `STM32H743VIH6TR` alternate. The supplier CPL is `7a363758`, the neutral CPL `89064e56` (SHA-256 prefixes).
- On the top side, the LQFP100 and LQFP208 copies are at their authoring angle plus 270; the others at the authoring angle. The bottom copies follow `--bottom-rotation lcsc=mirrored`.

Upload for preview only and record the result with the screenshots in `tools/sourcing_data/jlc_previews/`. Do not order. `tools/frame_boards/jlc_preview.py` runs the upload in Arc and stops at Component Placements; the `jlc-preview` skill (`.claude/skills/jlc-preview/`) describes the whole flow.

## Preview of 2026-11-04

Recorded in `tools/sourcing_data/jlc_previews/2026-11-04-lcsc-batch-1/`. All 24 placements were made. Every copy reads 0 with the bindings applied, including the three `rotate 270` LQFP bindings. The 16 QFP readings were taken by `measure_preview.py`, and the 8 BGA readings by hand from the mark nearest ball A1.
