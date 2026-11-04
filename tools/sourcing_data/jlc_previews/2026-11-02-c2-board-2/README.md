# C2 frame board 2: assembler preview, 2026-11-02

The assembler's placement preview of C2 frame board 2 (`tools/frame_boards/c2-board-2/`, generated at commit `335fe9c`). It was uploaded for preview only, with no order. The rows of `jlc_frames.json` whose ids start with `c2b2-` cite this record.

## What was uploaded

The board holds 16 parts, each placed four times: top 0°, top 37°, bottom 0° and bottom 37°. The CPL was exported with `--bottom-rotation lcsc=mirrored` and carries no supplier mapping.

| File | sha256 |
|---|---|
| `c2_frame_board_2-gerbers.zip` | `834923101d4244e3db30fc7bb29f984eb60e7f359213b49e49e8c37a4feb3a63` |
| `c2_frame_board_2-lcsc-bom.csv` | `313cff600bb858acc2d31f6c289882768ad1933bf804973ac8c967ebfefbf564` |
| `c2_frame_board_2-lcsc-cpl.csv` | `aee0bd48b99a98ac2565662efcd1cb83566d6087930a9b96dba5bbfcf2338c55` |

The BOM step matched all 16 parts. The assembler fixed the quantity at five boards, 20 of each part, and would not select four of them for stock:

| Row | Part | Stock |
|---|---|---|
| LQFP176 (U9–U12) | C5269976 | 3 |
| LQFP64 (U29–U32) | C1337443 | 1 |
| TFBGA169_ST (U45–U48) | C24622621 | 10 |
| UFBGA100 (U57–U60) | C1338995 | 10 |

Those 16 placements were not made, and this record has no row for those parts. The preview shows the other 12 parts.

## Screenshots

`top.jpg` is the top view and `bottom.jpg` the view from below, mirrored left to right. Both are the assembler's whole-board captures at 3439 × 4096 px.

## Method

`measurements.json` is the output of `tools/frame_boards/measure_preview.py measure`, as for board 1. `delta_ccw_from_top` is how far the preview's pin 1 sits counter-clockwise from ours, seen from the top. Single readings scatter by up to about 10° because the dot sits on the lead.

On the BGAs the script finds two magenta marks near the A1 corner, so it leaves `delta_ccw_from_top` empty. Their angles, taken by hand from the same file, are in the table. One mark sits within 0.2 mm of our ball A1 and the other 0.4–0.9 mm outside the corner along the diagonal. A part turned by a quarter turn would put both at another corner, 5–6 mm away.

## Results

| Part | Footprint | Top 0° / 37° | Bottom 0° / 37° (seen from top) | Map `rotate` |
|---|---|---|---|---|
| C529436 | `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` (0.3.0) | 2.5, 1.8 | 352.8, 353.5 | 0 |
| C7532 | `qfp::QFP50P900X900X160_48N` (0.3.0) | 2.9, 2.5 | 356.4, 356.1 | 0 |
| C8734 | `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` (0.3.0) | 2.7, 2.4 | 355.2, 355.1 | 0 |
| C7950 | `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` (0.3.0) | 10.4, 9.8 | 352.0, 350.3 | 0 |
| C7433 | `soic::TI_D0008A_4214825C` (0.3.0) | 7.6, 6.6 | 353.9, 352.5 | 0 |
| C52717 | `soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM` (0.3.0) | 7.1, 6.0 | 351.5, 351.0 | 0 |
| C7859 | `soic::TSSOP24P65_TI_PW0024A` (0.3.0) | 4.3, 3.0 | 353.6, 353.6 | 0 |
| C117815 | `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` (0.3.0) | 91.6, 91.8 | 266.7, 267.1 | 270 |
| C717701 | `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` (0.3.0) | 92.1, 92.5 | 267.2, 267.3 | 270 |
| C129276 | `qfp::KICAD_LQFP_208_28X28MM_P0_5MM` (0.3.0) | 90.2, 90.9 | 268.6, 269.1 | 270 |
| C1857444 | `bga::KICAD_TFBGA_100_8X8MM_LAYOUT10X10_P0_8MM` (0.2.0) | 0.5, 359.8 | 0.8, 0.5 | 0 |
| C730103 | `bga::KICAD_UFBGA_64_5X5MM_LAYOUT8X8_P0_5MM` (0.2.0) | 356.7, 355.9 | 3.6, 2.7 | 0 |

On the BGAs the angle is that of the mark nearest our ball A1.

## What it shows

- **The W3 move holds.** The seven footprints that transistor 0.2.0, soic 0.3.0 and qfp 0.3.0 moved, and that the USB-C origin board did not already confirm, read rotate 0 on all four copies with no map. Their frame rows now cite this record with a measured value of zero.
- **The larger KiCad LQFPs need the same quarter turn.** LQFP100 on two models and LQFP208 read 270, as LQFP32 and LQFP48 did before the move. Their rows carry `rotate 270` until those footprints are moved too.
- **The two BGAs are on the assembler's frame.** TFBGA100 and UFBGA64 read rotate 0 with A1 on A1.
- **Bottom side.** As before, each bottom reading is the mirror image of its top reading, and each 37° copy reads like its 0° copy.

## Not measured

LQFP64, LQFP176, TFBGA169 and UFBGA100 were not placed, as above. The B2 sample had no part for LQFP144. Those footprints have no frame evidence yet.
