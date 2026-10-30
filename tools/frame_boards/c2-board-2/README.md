# C2 frame board 2

The second frame-alignment board of library plan phase C2. It holds 16 LCSC parts, each placed four times: top 0°, top 37°, bottom 0° and bottom 37°. That makes 64 placements on a board of about 182 × 217 mm. No row carries a supplier mapping.

## What it checks

**The footprints board 1 moved.** Board 1 measured nine footprints off the assembler's frame, and transistor 0.2.0, soic 0.3.0 and qfp 0.3.0 moved them. The USB-C origin board confirmed the SOT-23 and the 208 mil SOIC-8. These seven rows confirm the rest; each should read rotate 0.

| Row | Part | Footprint |
|---|---|---|
| LQFP32_KICAD | C529436 STM32L412K8T6 | `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` |
| LQFP48_KICAD | C8734 STM32F103C8T6 | `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` |
| LQFP48_IPC | C7532 HT1621B | `qfp::QFP50P900X900X160_48N` |
| SOIC8_KICAD | C7950 LM358DR2G | `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` |
| SOIC8_TI | C7433 OP07CDR | `soic::TI_D0008A_4214825C` |
| TSSOP20 | C52717 STM8S003F3P6TR | `soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM` |
| TSSOP24 | C7859 SN74LVC4245APWR | `soic::TSSOP24P65_TI_PW0024A` |

**The STM32 package families.** The library's `@st/stm32` parts use these footprints hundreds of times. Each row is a library part whose MPN the B2 sample matched exactly to an LCSC part (`tools/sourcing_data/lcsc_backfill/2026-10-30-sample200/`), so a confirmed frame becomes a binding's evidence directly. All are Extended parts.

| Row | Part | Footprint | Stock on 2026-10-30 |
|---|---|---|---|
| LQFP64 | C1337443 STM32L476RGT3 | `qfp::KICAD_LQFP_64_10X10MM_P0_5MM` | 1 |
| LQFP100_F4 | C117815 STM32F427VGT6 | `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` | 128 |
| LQFP100_L4 | C717701 STM32L496VGT6 | same, other model | 31 |
| LQFP176 | C5269976 STM32F437IIT7 | `qfp::KICAD_LQFP_176_24X24MM_P0_5MM` | 3 |
| LQFP208 | C129276 STM32F767BIT6 | `qfp::KICAD_LQFP_208_28X28MM_P0_5MM` | 139 |
| UFBGA64 | C730103 STM32G431RBI6 | `bga::KICAD_UFBGA_64_5X5MM_LAYOUT8X8_P0_5MM` | 494 |
| UFBGA100 | C1338995 STM32L152VCH6 | `bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM` | 10 |
| TFBGA100 | C1857444 STM32H743VIH6TR | `bga::KICAD_TFBGA_100_8X8MM_LAYOUT10X10_P0_8MM` | 1013 |
| TFBGA169_ST | C24622621 STM32U595AJH6 | `bga::KICAD_ST_TFBGA_169_7X7MM_LAYOUT13X13_P0_5MM` | 10 |

The LQFP64 and LQFP176 parts were short of four in stock. The B2 sample had no better match for either footprint, and none for LQFP144. If the assembler will not select them, those rows are left out of the preview and the families go to a later board.

On a BGA, pin 1 is ball A1, and `measure_preview.py` reads it as pad 1.

## Regenerate

```sh
python3 -I tools/frame_boards/inventory.py /tmp/inv.json
python3 -I tools/frame_boards/c2_board_2.py /tmp/inv.json tools/frame_boards/c2-board-2/manifest.json
python3 -I tools/jlc_frame_board.py tools/frame_boards/c2-board-2/manifest.json -o BOARD \
  --cohdl target/debug/cohdl --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
```

The supplier CPL is exported with `--bottom-rotation lcsc=mirrored`. The Gerbers, drill files and CPL all measure from the outline's lower-left corner.

## Checks before upload (2026-10-31)

- The compiler CPL equals `smt_pos.py --map` on the board, byte for byte.
- It also equals KiCad's own position export with the drill/place origin in position, side and top-side angle: 64 of 64 rows match.

Upload for preview only and record the result in `jlc_frames.json` with the screenshots. Do not order.
