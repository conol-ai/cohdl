# C2 frame board 3: assembler preview, 2026-11-05

This is the assembler's placement preview of C2 frame board 3 (`tools/frame_boards/c2-board-3/`), which also carries the batch 2 passives (`tools/frame_boards/lcsc-batch-2-passive/`). The board's package was built by `tools/frame_boards/preview_package.py` at commit `ad83a40`; the package's SHA-256 is `b332a01f608ac168e60f8992337e427d1ae27e1ed70b4d2f105761a987f54a96`. It was uploaded for the preview only, and the flow was stopped at Component Placements. No order was placed and nothing was added to the cart.

## What was uploaded

49 part numbers and 109 placements, with no supplier mapping: each CPL row is the raw footprint frame. Most C2 parts are placed four times (top 0°, top 37°, bottom 0°, bottom 37°); the UFBGA100 and TFBGA169 once (top 0°); each batch 2 passive once (top 0°). The CPL was exported with `--bottom-rotation lcsc=mirrored`.

| File | sha256 |
|---|---|
| `c2_frame_board_3-gerbers.zip` | `5e56d30036421f105b8199170e1f8f3f9522fc41e3c30689530d8452622448be` |
| `c2_frame_board_3-lcsc-bom.csv` | `78b06d7499acc994a069eb4df64cf15e9ee575f0d6f682411cc2c7fc9e2a996d` |
| `c2_frame_board_3-lcsc-cpl.csv` | `26fab365b4b4229a5bf73de93a7430f75d7b905ec90d2392c89f28837d5872e8` |

The assembler read the board as 2 layers, 114.5 × 125.78 mm. The settings were the assembler's defaults apart from both sides. The BOM step detected and confirmed all 49 part numbers; each of the 69 top and 40 bottom designators appeared once, on its CPL side, in a selected row with its part number, and no stock warning showed.

An earlier package of the same board, with 62 part numbers (`4201c4f3…`, commit `1de3766`), stopped at the BOM step: twelve passives showed a stock shortage, and the crystal C32346 matched no part. Those 13 were left off; `tools/frame_boards/c2-board-3/README.md` lists them, and they stay unverified.

## Screenshots

`top.png` is the top view, and `bottom.png` the view from below, mirrored left to right. Both are the preview's own 2D whole-board exports, 4096 × 3728 px, kept byte for byte as downloaded (the preview writes JPEG data under a `.png` name): SHA-256 `1fd4170a5f93c1ca5b9af41fc950fce767ddf55ad624545e9c5ca44ecc5e2d1d` (top) and `ae456ae1e5674a3b762ea7d7c20f557e562faf9f9501b3c328aa29e293b4ad37` (bottom).

## Method

`jlc_preview.py` (commit `8f00824` and after) checked each instance on its own evidence and wrote `checks.json`:
- a non-polar chip part (`measure_preview.is_chip`: a `passive` `CHIP_*` land with two pads) gets no pin 1 mark; its drawn body must lie along the pads' axis, mod 180, within 10°, and be centred within the lesser of 0.25 mm and a quarter of the pad half-spacing;
- every other part needs a pin 1 mark of its own, each mark counted once for the nearest footprint within reach;
- a model not drawn (bare pads and the checkered centroid marker) is unverified, and a drawn part without its evidence goes to manual review.

`measurements.json` is the output of `tools/frame_boards/measure_preview.py measure` on the two exports converted to BMP with `sips`, with the package's `board.json`; it reproduces byte for byte from them. `delta_ccw_from_top` is how far the preview's pin 1 sits counter-clockwise from ours, seen from the top; the map that cancels it is `(360 − delta) mod 360`, to the nearest quarter turn. A chip row carries `body`: `delta_mod_180`, `offset_mm` and `body_ok`.

A reading is recorded in `jlc_frames.json` only when every copy of the part number agrees within 10° of one quarter turn (the scatter between single readings), or, for the parts noted, when a person read it from the exports.

## Results

| Part | Footprint | Top 0° / 37° | Bottom 0° / 37° (seen from top) | Read by | Map |
|---|---|---|---|---|---|
| C14877 ATMEGA328P-AU | `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` | 0.9, 2.5 | 355.6, 357.2 | script | 0 |
| C23922 STM32F030C8T6 | `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` | 1.1, 3.7 | 356.7, 358.3 | script | 0 |
| C9861 STM32L151C8T6 | same | 2.7, 5.6 | 358.3, 358.9 | script | 0 |
| C18229 LM2904DR2G | `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` | 6.3, 7.4 | 350.7, 351.9 | script | 0 |
| C8963 SP3485EN-L/TR | same | 7.3, 8.5 | 351.7, 352.3 | script | 0 |
| C16581 TP4056-42-ESOP8 | `audio-amp::FP_NS4168_ESOP8` | 85.8, 87.0 | 265.3, 268.3 | script | `rotate 270` |
| C24112 MPU-6050 | `qfn::ESPRESSIF_QFN24_0P5_4_E2P8` | 90.7, 90.9 | 266.4, 266.9 | script | `rotate 270` |
| C2128 1N4148WS | `@contrib/esd::FP_SOD323_2P5X1P25mm` | − on pad 1 | − on pad 1 | by eye | 0 |
| C191023 1N5819WS | same | − on pad 1 | − on pad 1 | by eye | 0 |
| C1338995 STM32L152VCH6 | `bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM` | 91 (one copy) | — | manual review, two marks | `rotate 270` |
| C32574 TP4054-42-SOT25R | `ldo::SOT5P95X290X145N` | 101.4, 102.3 | 264.9, 262.9 | script | not recorded |
| C7420376 SRV05-4 | `esd::FP_SOT_23_6` | 108.2, 109.1 | 262.0, 259.1 | script | not recorded |
| C720477 TS-1088-AR02016 | `switches::FP_XUNPU_TS_1088R` | no mark | no mark | — | not recorded |
| C874860 RC1210FR-0725R5L | `passive::CHIP_1210` | not drawn | not drawn | — | not recorded |
| C878293 RC2512JK-074M7L | `passive::CHIP_2512` | not drawn | not drawn | — | not recorded |
| C24622621 STM32U595AJH6 | `bga::KICAD_ST_TFBGA_169_7X7MM_LAYOUT13X13_P0_5MM` | not drawn (one copy) | — | — | not recorded |

**Passives that read 0 (mod 180).** 33 part numbers: every drawn chip body lies along its pads, within 7.9° mod 180, and is centred on them (`checks.json`, `measurements.json`). Each gets its own frame row with `rotate 0`, and its evidence note says the orientation is known mod 180 only.
- `CHIP_0201` (four copies each): C138161, C541229, C718246
- `CHIP_0402`: C93943, C106254, C107011, C114763, C137837, C137927, C477801, C482239, C3009257
- `CHIP_0603`: C105575, C163421, C163429, C482876
- `CHIP_0805`: C107108, C137422, C141703, C482966, C483001, C483188, C527206, C527232, C576985
- `CHIP_1206`: C137203, C274047, C488876, C488985
- `CHIP_1210` (four copies): C309589
- `CHIP_2010`: C137046, C877226 (four copies)
- `CHIP_2512` (four copies): C878096

## What it shows

- **The moved QFP and SOIC lands hold for more models.** Two more LQFP-48 models, a third LQFP-32 and two more SOIC-8 read 0 on all four copies, so each of these groups now has three direct checks.
- **ESOP-8 and QFN-24 need a quarter turn for these part numbers.** C16581 and C24112 read about 90 on top and 270 on the bottom, on all four copies. Each is recorded as a per-part exception with `rotate 270`; the lands are not moved on the evidence of one part number each.
- **SOD-323 polarity matches.** On both diodes the preview's minus sign sits on pad 1, beside our pin 1 dot, on all four copies, with the cathode band at that end.
- **Chip passives sit along their lands.** 33 part numbers across eight chip sizes, including the 0201, 1210, 2010 and 2512 sizes that had no frame row.
- **Both sides and both angles agree** wherever a reading was taken: each bottom reading is the mirror image of the top one, and each 37° copy reads like its 0° copy.

## Manual review and limits

- **Unverified: models not drawn.** C874860 (1210, R5–R8), C878293 (2512, R17–R20) and C24622621 (TFBGA169, U41) show bare pads and the checkered centroid marker on every export. The assembler matched and selected them, but has no model to show, so the preview says nothing about their frame.
- **Manual review: the tactile switch C720477** (SW1–SW4) is drawn but carries no pin 1 mark, and its land is two symmetric pads, so the preview cannot show its orientation. Not recorded.
- **Manual review: the SOT-23-5 C32574 and the SOT-23-6 C7420376.** The models' leads run across our lands, and pin 1 sits a quarter turn counter-clockwise from ours on all four copies, as board 1 found for C7722 and C12044. But the readings (101–102° and 108–109° on top) are 11–19° from the quarter turn, outside the 10° scatter, so they are not recorded here. Likely map: `rotate 270` for each. Board 1 read 180 for C7972 and C85364 on these same lands, which confirms that the correction depends on the part number.
- **Manual review: the UFBGA100 C1338995** (U46, one copy). Both of the preview's marks sit at the corner a quarter turn counter-clockwise from our ball A1 (0.17 and 0.95 mm from that corner ball), and none sits near A1. That reads as `rotate 270`, by hand, on a single copy. Its frame row is held back for now (below).
- **SOD-323 polarity was read by eye**, on the assumption that pad 1 is the cathode: the footprint carries only a pin 1 dot.
- **A non-polar part's frame is known mod 180.** A chip body cannot show 0 from 180; for these parts that makes no difference to the copper.
- **The preview is the assembler's render, not a placed board.** It shows where the assembler's models land, not paste, reflow or a physical part.
- **Runs.** The same package was run five times on 2026-11-05 while `jlc_preview.py` was fixed: the first stopped on the 2D switch, the second on the old one-mark-per-part rule, and the third and fourth on exports that took about three minutes to arrive. Each run makes a new project. This record keeps the fifth run's exports and readings.

## Since the preview

**Commits.** This record was written on a branch that was then replayed onto `main`, so the commits it cites have new hashes there, with the same changes: `ad83a40` is `5e920d4`, `1de3766` is `58aa803`, and `8f00824` is `3c74504`. The package rebuilt at `5e920d4` differs from `b332a01f…` only in the commit it names (`README.txt`, `verification.json`) and in the creation dates of the Gerber headers. The supplier BOM and CPL, `board.json`, the manifest, the lock files and the Gerber and drill bodies are byte for byte the same.

**Bindings, `passive` 0.2.4.** The 33 passives above are bound in `passive` 0.2.4, and every `passive` frame row now names 0.2.4. The preview was made with 0.2.3. 0.2.4 changes no land: `lib/passive/src/pads.cohdl` and `footprints.cohdl` are byte for byte those of the published 0.2.3 (`c6ac912`, registry hash `sha256:0f0fe09d…`), and the only other change is the 33 `supplier lcsc` clauses. So each reading holds for 0.2.4 as it did for 0.2.3.

**Board manifests.** With those bindings in the library, a board part that names the same part number must carry the same identity (E813). The board 3 and batch 2 manifests now give mapped parts the map row's reviewed manufacturer, `Yageo` or `Vishay`, in place of the page's `YAGEO` or `Vishay Intertech`, and name `passive` 0.2.4. Built that way, the board's supplier BOM and CPL are byte for byte those uploaded.

**C1338995 held back.** Its `rotate 270` row would bind an `@st/stm32` part. That would change `@st/stm32` 0.3.0, which is in the current publish round. So the row is left out of `jlc_frames.json`, and the reading stays here. It is to be added with the next `@st/stm32` version.

**Added in `@st/stm32` 0.3.1 (2026-11-06).** The C1338995 row is now in `jlc_frames.json` (`c2b3-kicadufbga1007x7mmlayout12x12p05mm-c1338995`, `bga` 0.2.0, `rotate 270`), and `@st/stm32` 0.3.1 binds `MCU_STM32L152VCH6` with it. The reading is still the single manual one above, so it is reviewed on its own, apart from the other bindings of that version. The same version binds the LQFP48 parts C9861 and C23922 read here, once the B2 full run put them in `lcsc_map.json`. Every `passive` frame row now names 0.2.5, which changes no land from 0.2.4.

## Added 2026-11-10: inputs for the W3 rule

The W3 rule (`docs/library-supplier-w3-sampling.md`) counts this board's chip rows as samples once it can judge them, and it judges offsets as vectors. Nothing above is changed. The uploaded package was not kept, so on 2026-11-10 `preview_package.py` was run again at `ad83a40`. Its BOM and CPL equal the uploaded files byte for byte (`78b06d74…`, `26fab365…` above); only the Gerber archive differs, by its timestamps. Added:
- `board.json` and `c2_frame_board_3-lcsc-bom.csv`: from that rebuild.
- `remeasure-2026-11-10.json`: `measure_preview.py measure` run again on `top.png` and `bottom.png` with them. It equals `measurements.json` row for row, apart from each body's added `offset_xy_mm`.
- `w3.json`: what `tools/w3_groups.py` reads from this record. It holds the files above, `checks.json`, and the SHA-256 of the land files. The land files are byte for byte those at `ad83a40`.

## Addendum, 2026-11-11: read again with `measure_preview.py` method 2

Everything above is the record as made; its chip body readings are by what is now called method 1, which read each part's white reference text together with its model (`../2026-11-11-board-4/README.md`). `remeasure-2026-11-11-method-2.json` (SHA-256 `feb3a0e23542dadeaa5fbbcfdf91eb2d6482b36bd3dadf55194348a4fed9110e`) is `measure_preview.py measure --method 2 board.json top.bmp bottom.bmp` on the same exports: the drawn model alone. `w3-readings.json` names it for `tools/w3_groups.py`. `measurements.json` and `remeasure-2026-11-10.json` are unchanged and still reproduce with `measure --method 1`.

By method 2, 50 of the 51 drawn chip bodies pass the body check. R4 (C138161, `CHIP_0201`, bottom, −143°) does not: 0.099 mm against 0.080, where method 1 read 0.070 mm; its model is drawn off centre along its pads. C877226's four `CHIP_2010` copies read 0.031–0.068 mm where method 1 read 0.171–0.206. The three 0805 Yageo `CC` models (C11, C12, C13) read 0.114–0.148 mm toward one pad, as drawn. View medians: top (−0.021, +0.026) mm, bottom (−0.041, +0.013) mm. Whether W3 reads bodies this way is a proposal for Tony (`docs/library-supplier-w3-sampling.md` §6).
