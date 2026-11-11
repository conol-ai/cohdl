# Preview board 4: OpenWalkieTalkie gaps, calibration, W3 samples

This frame board is driven by a real order. The inventory of the OpenWalkieTalkie Rev B and r6 assembly packages (plan item 5a, 2026-11-11) found that the library binds about 5% of that order's placements, although it has most of the parts. The board carries the part numbers that inventory names, as far as they can be placed from data already in the repository, plus the calibration set and two W3 sample sets.

It holds 84 part numbers in 243 placements (147 top, 96 bottom) on a board of 127.78 × 120.37 mm (x × y). The assembler reads the same outline and prints it as "120.37x127.78mm". The generator's 120 × 112 mm is the area the part boxes are laid out in, not the board: `jlc_frame_board.py` draws the outline 5 mm outside the outermost boxes, and an earlier version of this page gave that inner area as the board's size. No row carries a supplier mapping; the one bound part is placed as the library ships it.

**Not previewed yet.** The package builds and passes the local checks. Uploading it to the assembler's preview needs Alex's approval, like every preview, and each new package needs its own. A preview run stops at Component Placements: no cart, no quote, no order, no terms ticked.

- **First run (2026-11-11), unverified.** The first package (SHA-256 `1cd1883d3d51001af779ffc51cd685f393daa21cf7284e42ec341ba84ae30cd2`, commit `f62f3be`) was uploaded with Alex's approval. The assembler read a 2 layer board of 120.37 × 127.78 mm. Its BOM step read "84 parts detected, 82 Parts confirmed" with a shortage warning and left two rows unselected: C470029 (`RC1210FR-071ML`, its page read 1,014) and C875102 (`RC1210FR-0768KL`, 1,200), both W3 samples of Yageo `RC` 1210. `jlc_preview.py` stopped there. No placement preview was taken, and nothing was measured or recorded from that run.
- **This package.** The two are replaced by their group's nearest eligible members, C7276637 (`RC1210FR-071M6L`, page 4,560) and C728392 (`RC1210FR-07681RL`, page 5,038), at the same two positions. The other 82 rows and all 243 positions are as the first run's BOM step confirmed them (see D and Stock below).

## What it carries

Each row's `purpose` in the manifest is one of these.

| Purpose | Rows | Copies | What a reading gives |
|---|---:|---|---|
| A | 12 | top 0° and 37°, bottom 0° and 37° | An order number whose library entry exists and which `lcsc_map.json` maps: the reading is all its binding still lacks |
| B | 1 | same | C97521 as the library binds it (`library_part`), on the land soic 0.3.0 moved |
| E | 10 | same | An order number with a confirmed page in the 2026-10-30 catalog snapshot, but no map row and no library entry: evidence for (land, part number) only |
| C | 10 | 0°, 90°, 180°, 270° and 37°, top and bottom | The calibration set of `docs/library-supplier-w3-sampling.md` §5 |
| D | 51 | top 0° | The W3 report's samples for two groups that need samples, read from the report |

**A: OpenWalkieTalkie numbers already in the map.**

| Part | Library entry | Land | On the order |
|---|---|---|---|
| C113780 CC0402JRNPO9BN102 | `passive::C_1n_50V_C0G_0402` primary | `CHIP_0402` | 10 placements |
| C19666 CL10A475KO8NNNC | `passive::C_4u7_16V_X5R_0603` alt | `CHIP_0603` | 4 (r6: 9) |
| C23630 CL10A225KO8NNNC | `passive::C_2u2_16V_X5R_0603` alt | `CHIP_0603` | r6 only: 6 |
| C107006 CC0402JRNPO9BN331 | `passive::C_330p_50V_C0G_0402` primary | `CHIP_0402` | 3 |
| C52923 CL05A105KA5NQNC | `passive::C_1u_25V_X5R_0402` alt | `CHIP_0402` | 2 |
| C106255 CC0402KRX5R6BB474 | `passive::C_470n_10V_X5R_0402` primary | `CHIP_0402` | 1 |
| C107039 CC0603JRNPO9BN152 | `passive::C_1n5_50V_C0G_0603` primary | `CHIP_0603` | 1 |
| C337893 DFE201612E-2R2M=P2 | `passive::IND_2U2` primary | `FP_L_0806_2016Metric` | 2 |
| C97018 WPN201610U4R7MT | `passive::IND_4U7_WPN201610U` primary | `FP_L_WPN201610U_2016Metric` | 1 |
| C7519 USBLC6-2SC6 | `esd::ESD_USBLC6` primary | `esd::FP_SOT_23_6` | U20 |
| C131941 TPS22918DBVR | `load-switch::LOADSW_TPS22918DBVR` primary | `load-switch::FP_TI_DBV0006A` | U2, U10 |
| C189961 NS4150B | `@contrib/audio-amp::AMP_NS4150B` primary | `SOP8P65X490X110N` | U1 |

- **The two SOT-23-6 parts are expected to read a rotation.** Board 1 read 180° for C85364 on `esd::FP_SOT_23_6` and 270° for C12044 on `FP_TI_DBV0006A`, and found the correction depends on the model. So C7519 and C131941 are placed raw, and whatever they read is theirs.
- **C19666 and C23630 are entries of the 16 V parts.** The OpenWalkieTalkie design instantiates the 10 V parts (`C_4u7_10V_X5R_0603`, `C_2u2_10V_X5R_0603`), whose entries are other part numbers. A binding from this board therefore lands on the 16 V parts. The order's placements become bound only if the design uses those, or the 10 V parts gain these entries (a 16 V part does satisfy a 10 V requirement, but that is a library decision).
- Each row declares a local part with the map row's identity on the library entry's device type and land, as on `lcsc-batch-2-passive`.

**B: C97521 W25Q128JVSIQ** on `soic::SOIC8P127X790X216N`. Its frame row `c2b1-soic8_208-c97521` was measured on board 1 (rotate 270) before soic 0.3.0 turned the land; the mapping of 0 at 0.3.0 has had no confirming board. The order's CPL angle for U4 also disagrees with the library's reading. The row places `flash::FLASH_W25Q128JVSIQ` itself, as the library ships it now: flash 0.3.1 on soic 0.3.0, with `supplier lcsc "C97521"` and a zero map (flash 0.3.1 changed the W25Q32 land only; this part and its land are as in 0.3.0). The build refuses the board unless the supplier BOM carries C97521 for it, and the generator refuses to place it if soic is no longer at the version the frame row names. Expected: rotate 0.

**E: order numbers with a page but no map row.** UNI-ROYAL 0402WGF resistors C25741 (100 k, 14 placements on the order), C11702 (1 k, 12), C26083 (1 M, 5), C17168 (0 Ω, 3), C25768 (22 k, 2), C25905 (5.1 k, 2), C25092 (22 Ω) and C25890 (3.3 k); Samsung C23733 CL05A475MP5NRNC (4.7 µF, 5); Fenghua C1548 0402CG150J500NT (15 pF). All are JLC Basic parts on `passive::CHIP_0402`. Each row reuses the library device of its value for the pins only, as board 1 did for C25744. A reading here is evidence for the pair (land, part number). A binding also needs a library `alt` entry and a map row, which are separate work.

**C: calibration.** An export's shift stays fixed in the board frame whatever the part's angle; a model's own offset turns with the part. Each of these ten is placed at 0°, 90°, 180°, 270° and 37° on top, and again on the bottom, so each view has five readings of one part number to fit "export shift + the part's offset turned by its angle".

| Part | Land | Earlier reading |
|---|---|---|
| C60491 RC0402FR-07100KL | `CHIP_0402` | pilot −3.0°, raw 0.070 mm |
| C138033 RC0402FR-071ML | `CHIP_0402` | pilot +9.9°, raw 0.117 mm |
| C185445 RC0402FR-0720K5L | `CHIP_0402` | pilot +8.1°, raw 0.121 mm, the largest of its group |
| C113781 CC0402JRNPO9BN681 | `CHIP_0402` | pilot +10.5°, left for manual review; pauses Yageo `CC` 0402 |
| C325464 CC0402KRX5R7BB105 | `CHIP_0402` | pilot −0.3°, raw 0.082 mm |
| C481918 CRCW0402100KFKED | `CHIP_0402` | pilot −7.3°, raw 0.068 mm |
| C482097 CRCW040220R0FKED | `CHIP_0402` | pilot +7.5°, raw 0.100 mm |
| C138161 RC0201FR-07221RL | `CHIP_0201` | board 3 residual 0.046–0.064 mm; pauses Yageo `RC` 0201 |
| C877226 RC2010FK-07200KL | `CHIP_2010` | board 3: about 0.18 mm that turns with the part |
| C878096 RC2512FK-074R7L | `CHIP_2512` | board 3: mixed |

The first four 0402 parts are in the board's first band and the rest in its last, so each view has calibration bodies at both ends of the board. Five angles of one part sit side by side, top copies above bottom copies.

**What each view's median is made of.** The W3 rule judges a sample's residual against the median of every drawn chip body in its view. Tony's condition on that (#104): the median must include bodies of at least one other group, or it shows only that a group agrees with itself. The rows' `group` is their group in the W3 report, the manifest's `view_groups` counts the chip bodies placed in each view, and the generator refuses a layout in which a view has fewer than three groups or one group holds half of it.

| Group (W3 report) | Top bodies | Bottom bodies | From |
|---|---:|---:|---|
| `CHIP_1210` Yageo `RC` | 42 | 0 | D |
| `CHIP_0402` Vishay `CRCW` | 19 | 10 | D 9, C 10 a view |
| `CHIP_0402` Yageo `CC` | 16 | 16 | C 10, A 6 |
| `CHIP_0402` Yageo `RC` | 15 | 15 | C |
| `CHIP_0201` Yageo `RC` | 5 | 5 | C |
| `CHIP_2010` Yageo `RC` | 5 | 5 | C |
| `CHIP_2512` Yageo `RC` | 5 | 5 | C |
| `CHIP_0603` Samsung `CL` | 4 | 4 | A |
| `CHIP_0402` Samsung `CL` | 2 | 2 | A |
| `CHIP_0603` Yageo `CC` | 2 | 2 | A |
| in no group (UNI-ROYAL, Samsung C23733, Fenghua; all `CHIP_0402`) | 20 | 20 | E |
| **All chip bodies** | **135** | **84** | |

So the top view has ten groups and the bottom nine. The two sampled groups are judged in the top view: Yageo `RC` 1210 is 42 of its 135 bodies, with 93 from other groups; Vishay `CRCW` 0402 is 19, with 116 from others. No group reaches a third of a view. Three limits on what this gives:

- These are placed bodies. The median is over drawn ones, so the composition has to be counted again from the measurement; the pilot left 9 of 49 `CRCW` samples undrawn.
- Other groups are not other lands. `CHIP_0402` holds 72 of the top's 135 chip bodies and 63 of the bottom's 84, so each view's median is close to the 0402 median, and the 1210 samples are judged against it. That is what the rule's board median means; whether it should be per land is the question the fit below is for.
- The bodies are not all at one angle. On top 78 are at 0°, 27 at 37° and 10 each at 90°, 180° and 270°; on the bottom 27 at 0°, 27 at 37° and 10 each at the others. If an offset turns with the part, the turned bodies pull the median away from what a 0° body sees, more on the bottom than on top.

**D: W3 samples.** The report of 2026-11-10 (`tools/sourcing_data/w3_sampling/2026-11-10/groups.json`) lists the `sample` it chose for each group that needs samples. This board takes two groups whole: Vishay `CRCW` 0402 (9 samples, to replace the nine the pilot did not draw) and Yageo `RC` 1210 (42). The generator reads the part numbers from the report, in its order, after checking the report's SHA-256 (`5b0f77f2be21eb8cde5b15171d382338ef23f9e26cf6e7cdb35abc5f31929a2c`), and still checks each against the map, the pages and the library. A regenerated report therefore stops the generator until this board is looked at again.

*Short at the BOM step.* The repository's convention (the pilot, `tools/w3_groups.py`) is that a sample the assembler finds short, although its page read 30 or more, is recorded as a dated fact and replaced by the nearest eligible member of its group, so the samples the same run confirmed stay. The pilot's two are in its preview record's `w3.json`. This board has no preview record, because the run that found C470029 and C875102 short took no preview, and a record directory without a preview would be an invented one. So the fact is in this board's own `w3.json` (`short_at_preview`, with the date, the package and what each page read), next to the manifest it changes. `w3_groups.py` reads it together with the records' lists, so its next report passes over both numbers too; when this board's preview is recorded, the list moves into that record's `w3.json`.

The generator does not pick the replacements. It rebuilds the group's eligible members from the report (at least 30 in stock on the cached page, not yet tried, in MPN order), checks that `w3_groups.choose` gives the report's own sample from them, and takes `choose`'s answer with the board's list added. Nothing is fetched. The two new rows carry `replaces` and the reason in their note; the other 40 samples of the group and the 9 `CRCW` samples are unchanged, and the group still gets 42 samples from this board (it needs 43 passed in all and has 1).

## Left off

The manifest lists these under `left_off`.

- **C2799946** (C&K PTS636, SW3 and SW4; in `lcsc_map.json`) and **C2917717** (Johanson 2450AT18B100E, AE1): each page read stock 0 on 2026-11-06. The assembler does not place such a part.
- **C25744** (UNI-ROYAL 10 k) and **C15849** (Samsung 1 µF 0603): board 1 already gave each a frame row.

Order numbers with no page in the repository are not on the board. They need a fetch first: C25760, C107369, C342827, C97946, C147972, C16902 and the connector, switch and IC numbers the design defines locally.

## Stock

Every page read at least five pieces per placement, and at least 30. Pages are from 2026-10-30 to 2026-11-07, and stock has moved before between a page and a preview. `jlc_preview.py` stops on any stock warning, and the preview is then unverified. Changing a row makes a new package, which needs its own approval.

A page's number did not predict the first run. The two rows it found short had read 1,014 and 1,200. The rows with the least on their pages were all among the 82 the BOM step confirmed on 2026-11-11, so they stay: the 1210 samples C728395 and C723577 (each read exactly 30), C275640 (35), C875123 (35), C728375 (39), C875234 (44), C875093 (49) and C728367 (55), each needing 5; and the calibration parts C138161 (read 75, needs 50) and C878096 (252, needs 50). That they passed is a fact of that day only; the next run can find any row short.

## What to expect

- Chip parts: centred on the land, along its long axis, at every angle.
- C97521: pin 1 on our pad 1, at all four copies.
- C7519, C131941, C189961: read, not assumed. A rotation that all four copies agree on is a candidate mapping for that part number.
- A part the assembler does not select or draw is unverified, not a pass.

## After the preview: the calibration fit

`calibration_fit.py` is ready for the readings, so the upload does not have to wait on analysis. It takes the package's `board.json` and supplier BOM and the output of `measure_preview.py measure`, whose `body.offset_xy_mm` is each chip body's offset as a board-frame vector:

```sh
python3 -I tools/frame_boards/measure_preview.py measure board.json top.bmp bottom.bmp measurements.json
python3 -I tools/frame_boards/calibration_fit.py board.json measurements.json preview_board_4-lcsc-bom.csv fit.json
```

For every part number read at three or more orientations in a view (here the ten calibration parts, five readings a view), it fits three models by least squares: `export_only` (one shift, fixed in the board frame), `model_only` (one offset in the part's own frame, turning with it) and `both`. It reports the shift as board x, y and the offset along the pads (pad 1 to pad 2) and across them, with each fit's misfit (`rms_mm`, `max_mm`, and `sigma_mm` per coordinate after the fitted parameters). It does this three ways in each view:

- `parts`: each part number alone;
- `lands`: the part numbers of one land with one shared offset, which is what an offset between the assembler's land and ours would look like (`CHIP_0402` has seven part numbers of three groups; the other three lands have one each);
- `views`: one shift for the whole view with an offset for each part number, which is the export hypothesis in full: an export has one shift.

The part's frame comes from `board.json`'s pads. A bottom part's frame is the top one mirrored, which assumes the assembler mirrors its model with the part. Bodies the preview does not draw, and part numbers read at fewer than three orientations (the A and E rows, at 0° and 37°), are listed under `not_fitted`.

**No real data exists yet.** The tool is tested on synthetic readings only: a pure shift, a pure offset, a mixture, and noise of 0.015 mm (`tools/tests/test_calibration_fit.py`). It gives numbers, not a verdict: a hypothesis stands when its misfit is no worse than the scatter of repeated readings, which the earlier boards put at 0.01–0.02 mm. With five readings, each component of `both` is good to roughly that scatter over √5.

What the board can settle: for these ten part numbers, in these two exports, how much of the offset stays with the board and how much turns with the part; whether the seven 0402 part numbers share one offset; and whether one shift fits a whole view. What it cannot: that another export has the same shift (the pilot and board 3 differed by about 0.03 mm), that part numbers not on the board behave like these, or anything about a body that is not drawn. The W3 rule itself is Tony's to change.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-4 -o preview-board-4.zip \
  --cohdl target/debug/cohdl --kicad-cli $K/MacOS/kicad-cli \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

`preview_package.py` regenerates `manifest.json` with `preview_board_4.py` and refuses a manifest that differs. The supplier CPL is exported with `--bottom-rotation lcsc=mirrored`. The Gerbers, drill files and CPL all measure from the outline's lower-left corner.
