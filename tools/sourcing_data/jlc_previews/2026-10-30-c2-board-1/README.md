# C2 frame board 1: assembler preview, 2026-10-30

The assembler's placement preview of C2 frame board 1 (`tools/frame_boards/c2-board-1/`, generated at commit `418972f`). It was uploaded for preview only, with no order. The rows of `jlc_frames.json` whose ids start with `c2b1-` cite this record.

## What was uploaded

The board holds 30 parts, each placed four times: top 0°, top 37°, bottom 0° and bottom 37°. The CPL was exported with `--bottom-rotation lcsc=mirrored` and carries no supplier mapping, so the preview shows each footprint's raw frame offset.

| File | sha256 |
|---|---|
| `c2_frame_board_1-gerbers.zip` | `c1c7c1bec0291b438f6dab9671340214bf10ca2cce6966f44401e5725e406277` |
| `c2_frame_board_1-lcsc-bom.csv` | `37443a78233157b98daec7cfadde1c291c771285f8a7eba62baf5b8d7e16447c` |
| `c2_frame_board_1-lcsc-cpl.csv` | `b0d79a552e11fbff7a995f1915ec0c38adc032faffccf3636755e69043e0f69b` |

The BOM step matched all 30 parts. The assembler would not select C32346 (XTAL3215, Y1–Y4): it showed a quantity of 0 and a warning. Those four were not placed, and this record has no row for that part.

## Screenshots

`top.jpg` is the top view and `bottom.jpg` the view from below, mirrored left to right. Both are the assembler's whole-board captures at 2805 × 4096 px, about 25.7 px/mm.

## Method

`tools/frame_boards/measure_preview.py` finds the preview's magenta pin-1 dots, maps them to board millimetres through the board outline, and compares each with the footprint's pad 1. The script's output is `measurements.json`. `delta_ccw_from_top` is how far the preview's pin 1 sits counter-clockwise from ours, seen from the top. The dot sits on the lead, not on the pad centre, so single readings scatter by up to about 20°. Every rotation below is the nearest multiple of 90° and is consistent across all four copies of the part.

## Results

The map's `rotate` is in the footprint's local frame, so one value serves both sides.

| Part | Footprint | Top 0° / 37° | Bottom 0° / 37° (seen from top) | Map `rotate` |
|---|---|---|---|---|
| C2146, C8545, C20526 | `transistor::FP_JSCJ_SOT23` | 280–291 | 70–81 | 90 |
| C8254 | `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` | 95, 96 | 267, 266 | 270 |
| C7532 | `qfp::QFP50P900X900X160_48N` | 92, 93 | 267, 267 | 270 |
| C8734 | `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` | 92, 92 | 266, 266 | 270 |
| C97521 | `soic::SOIC8P127X790X216N` | 93, 97 | 268, 270 | 270 |
| C7950 | `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` | 95, 97 | 261, 263 | 270 |
| C7433 | `soic::TI_D0008A_4214825C` | 94, 95 | 264, 265 | 270 |
| C7972 | `ldo::SOT5P95X290X145N` | 173, 177 | 180, 178 | 180 |
| C7722 | `logic::SOT5P95X300X145N` | 96, 94 | 258, 258 | 270 |
| C12044 | `load-switch::FP_TI_DBV0006A` | 98, 97 | 261, 259 | 270 |
| C85364 | `esd::FP_SOT_23_6` | 180, 181 | 178, 175 | 180 |
| C52717 | `soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM` | 95, 97 | 265, 265 | 270 |
| C7859 | `soic::TSSOP24P65_TI_PW0024A` | 93, 95 | 268, 267 | 270 |
| C13738 | `osc::FP_Crystal_SMD_3225_4Pin_3_2x2_5mm` | 1, 1 | 2, 359 | 0 |
| C9002 | `osc::FP_Crystal_3225` | 2, 2 | 2, 359 | 0 |

Read by eye from the screenshots:

- **Two-terminal polarised parts.** On C2286 (LED 0603) and C81598 (SOD-123), the preview's `−` sign sits on pad 1, on our cathode bar, on all four copies: rotate 0. C7420372 (DFN1006) is centred on its pads and the preview shows no polarity mark, so its polarity is not confirmed by this preview.
- **Resistors and capacitors.** C25744, C1525, C25804, C15849, C17414, C15850, C17927 and C13585 sit centred on their pads on all four copies: rotate 0.
- **USB-C receptacle, C165948.** This is the CPL origin control. The preview marks the model's two locating pegs, 5.80 mm apart (ours are 5.78 mm apart). In our footprint's local frame, on all four copies, they sit at y = −1.00 to −1.13 mm. Our peg holes are at y = −2.60 mm, so the supplier's origin lies at (0, −1.54) in our frame. Along x, the readings are −0.12 and −0.14 mm on the top copies and +0.05 and +0.12 mm on the bottom ones. The sign flips with the mirrored view, so this is screenshot calibration, not an offset. The map value is `at (0mm, -1.55mm)`. A preview with that map applied should confirm it before it is relied on.

## What it shows

- **C1 results confirmed.** C2146 needs `rotate 90` and C97521 needs `rotate 270`, as in C1. The two other SOT-23 models need the same `rotate 90`.
- **Bottom-side convention confirmed.** Each 37° copy reads the same as its 0° copy on both sides, and each bottom reading is the mirror image of the top one. That is what `--bottom-rotation lcsc=mirrored` requires at any angle.
- **Correction depends on the supplier model, not only on the package.** C7972 and C7722 sit on SOT-23-5 lands with the same pad numbering, but they need 180 and 270. C85364 and C12044, both SOT-23-6, need 180 and 270 as well. A per-part binding (RFC-034 §2) is therefore required.

## Footprints moved since (library plan W3)

transistor 0.2.0, soic 0.3.0 and qfp 0.3.0 move the nine footprints of the SOT-23, SOIC-8, TSSOP and LQFP rows onto the assembler's frame by exactly the `rotate` above. Their rows in `jlc_frames.json` now name the new versions with a zero mapping, and `measured` keeps the value found here. The SOT-23-5/6 rows keep their per-part mappings, and the USB-C row keeps its `at`.
