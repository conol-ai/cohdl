# W3 sampling pilot board: assembler preview, 2026-11-07

This is the assembler's placement preview of the W3 pilot board (`tools/frame_boards/w3-pilot-0402/`). It carries the samples the grouping report (`tools/sourcing_data/w3_sampling/2026-11-07/`) chose for three `CHIP_0402` groups of `passive` 0.2.5. The package was built by `tools/frame_boards/preview_package.py` at commit `e6ba33d`. Its SHA-256 is `7f49d085527cef2524fbb6a55f1ed500ff6d57a71379c04281eb2c6788eafbfb`. Alex approved the upload for the preview only. The flow stopped at Component Placements: no order was placed and nothing was added to the cart.

This is evidence for the pilot only. No member of a group is bound by inference from it: the proposal (`docs/library-supplier-w3-sampling.md`) is a draft until Tony accepts it.

## What was uploaded

99 part numbers, one top copy each at 0°, with no supplier mapping: each CPL row is the raw footprint frame.

| File | sha256 |
|---|---|
| `w3_pilot_0402-gerbers.zip` | `98504c0ed909281aea2ec713b78d241f085d99166a32c20d986bd7e0f2132678` |
| `w3_pilot_0402-lcsc-bom.csv` | `d2d9cd6550b761bc0c6ddc93590ba92ebccf66f2c0090405ece3208a94d7c0c9` |
| `w3_pilot_0402-lcsc-cpl.csv` | `a28b52ed80040f581d3a71bab0576ee5c1a060cb0cf26cf9924c26868836e9c1` |

The assembler read the board as 2 layers, 28.42 × 106.02 mm, with the default settings. The BOM step detected and confirmed all 99 part numbers. Each of the 99 top designators appeared once, in a selected row with its part number, and no stock warning showed.

Two earlier packages stopped at the BOM step, and nothing was measured from them (`tools/frame_boards/w3-pilot-0402/README.md`):
- `8f056fde…`: 11 samples short, all with fewer than 20 pieces on their page. The minimum stock is now 30.
- `85e55a26…`: 2 samples sold out since their page was read (C185430, C185414). They were replaced by their groups' nearest eligible members.

## Screenshot

`top.png` is the preview's own 2D whole-board export of the top, 4096 × 1168 px, kept byte for byte as downloaded (JPEG data under a `.png` name). Its SHA-256 is `ca593ed3c704e58dce3b9257fe5f1dcaefa00bcca10d0c4132c70086084f0c48`. The board has no bottom parts, so no bottom view was exported.

## Method

As in `../2026-11-05-c2-board-3/README.md`. `jlc_preview.py` checked each instance on its own evidence and wrote `checks.json`. A chip part has no pin 1 mark, so it passes when:
- its drawn body lies along the pads' axis, mod 180, within 10°;
- it is centred within the lesser of 0.25 mm and a quarter of the pad half-spacing.

The export was taken twice, because the first left instances without evidence; the second left the same ones.

`measurements.json` is the output of `measure_preview.py measure board.json top.bmp - measurements.json`, with `top.png` converted to BMP by `sips` and the package's `board.json`. `-` stands for the bottom view, which a top-only board does not have.

## Results

| Group | Samples | Body along and centred | Worst angle (mod 180) | Not drawn | Manual review |
|---|---:|---:|---:|---|---|
| `CHIP_0402` ChipResistor Yageo `RC` | 44 | 44 | 9.9° | — | — |
| `CHIP_0402` ChipResistor Vishay `CRCW` | 47 | 38 | 7.5° | 9: C2085730 (R23), C2080002 (R29), C2085463 (R31), C2091853 (R35), C2085710 (R44), C2076988 (R54), C4100928 (R81), C2076789 (R82), C2091938 (R89) | — |
| `CHIP_0402` MLCC Yageo `CC` | 8 | 6 | 4.8° | 1: C576731 (C4) | 1: C113781 (C7) |

Over the 88 bodies that pass, the angle runs from −7.3° to +9.9° mod 180 (mean −0.2°, standard deviation 3.4°). The largest centre offset is 0.121 mm, against limits of about 0.127 mm.

Each of the 88 has its own frame row with `rotate 0`, in this record's `frames.json` (same row format as `jlc_frames.json`). Its evidence note says the orientation is known mod 180 only. These rows are each part's own evidence, as on the C2 boards. They are not inference.

## Status: archived evidence, not bindings

The 88 rows are archived here, not in `tools/sourcing_data/jlc_frames.json`. No generator reads `frames.json`: `tools/sourcing/bindings.py` reads only `jlc_frames.json`. They were first added to `jlc_frames.json` (commit `21c4ca5`), but every row names `passive` 0.2.5, so the generator bound them at once: `gen_passive.py --check` reported `lib/passive/src/resistors_0402.cohdl` and `capacitors_0402.cohdl` stale, and regenerating would have rewritten sources published as `passive` 0.2.5, which must not change. So the rows were moved here, and this PR stays a proposal.

`tools/tests/test_generator_bindings.py` checks that the committed sources of `gen_passive.py`, `gen_esp32.py` and `gen_stm32.py` match the real tables, and that no row of an archived `frames.json` is in `jlc_frames.json`.

Adopting the rows as bindings needs a new `passive` version: the rows' `package_version` moves to it, they go into `jlc_frames.json`, and the passive sources are regenerated. That has to be coordinated with PR #83, and it is Tony's decision.

## Manual review and limits

- **C113781 (C7)** reads 10.5° mod 180, just over the 10° limit. It is centred (0.053 mm) and its body is drawn. In the export the body looks square to its pads, and the red designator printed on it may have skewed the reading. It needs a person's reading. Until then it has no row in `frames.json`.
- **Not drawn.** Ten models were not drawn: bare pads and the checkered centroid marker, on both exports. They are unverified, not failed. Nine of the 47 Vishay samples are among them, so the preview may simply have no model for part of that series.
- **The pilot rule.** `tools/frame_boards/w3-pilot-0402/README.md` says that a sample outside the limits, or not drawn, pauses its group. By that rule:
  - Yageo `RC` 0402 passed: 44 of 44.
  - Vishay `CRCW` 0402 is paused: 9 not drawn.
  - Yageo `CC` 0402 is paused: 1 not drawn, 1 for manual review.

  No sample read outside the limits. Whether a sample that is not drawn should pause its group, or only fail to count towards its sample, is a question for the proposal's review.
- **The 9.9° reading** on a Yageo RC sample is within the limit. It is the edge of the scatter between single readings on these boards, which runs to about 10°.
