# Preview board 4: assembler preview, 2026-11-11

This is the assembler's placement preview of preview board 4 (`tools/frame_boards/preview-board-4/`): the OpenWalkieTalkie gap parts, C97521 as the library ships it, ten order numbers with no map row, the calibration set and two W3 sample sets. 84 part numbers in 243 placements, 147 on top and 96 on the bottom.

The package is `preview-board-4.zip`, SHA-256 `21e573823a1430c1aa394e3f7d69b51e5037669f932b042aadba14f055156aa2`, built by `tools/frame_boards/preview_package.py` at commit `cf8e3cd` (the head of PR #111 then). Alex approved this upload for the preview only. `jlc_preview.py` of the same commit ran it. The flow stopped at Component Placements: nothing was added to the cart and no order was placed.

It is the board's second package. The first (SHA-256 `1cd1883d3d51001af779ffc51cd685f393daa21cf7284e42ec341ba84ae30cd2`, commit `f62f3be`) was uploaded the same day and stopped at the BOM step: "84 parts detected, 82 Parts confirmed" with a shortage warning, C470029 and C875102 not selected. No placement preview was taken from it. This package replaces the two with C7276637 and C728392. That dated fact is this record's `w3.json` `short_at_preview`; it was kept in the board's own `w3.json` until this record existed.

**No binding, map row or frame row is written from this record.** The readings below are evidence for Tony to weigh. Nothing here changes the W3 rule or a W3 report.

## Files

| File | What it is |
|---|---|
| `top.png`, `bottom.png` | The preview's own 2D whole-board exports, kept byte for byte (JPEG data under a `.png` name), 4096 × 3863 px, 31.18 px/mm. `bottom.png` is the view from below, mirrored left to right. |
| `checks.json` | `jlc_preview.py`'s per-instance check, as the run wrote it. |
| `measurements.json` | `measure_preview.py measure package/board.json top.bmp bottom.bmp`, with the exports converted to BMP by `sips -s format bmp`. This is what `tools/w3_groups.py` reads. |
| `body_model.py`, `body-model.json` | A record-local body tool and its output for the 204 drawn chip bodies and 8 inductors: the drawn model alone, without the silkscreen text that `measure_preview.py`'s window also takes in (see "Why 75 instances were left for manual review"). `python3 -I body_model.py tools/frame_boards package/board.json top.bmp bottom.bmp body-model.json`, on the same BMPs. Its docstring defines each field. |
| `fit.json`, `fit-model.json` | `tools/frame_boards/calibration_fit.py package/board.json READINGS package/preview_board_4-lcsc-bom.csv OUT`, with `measurements.json` and with `body-model.json`. |
| `w3.json` | What `tools/w3_groups.py` reads from this record, and the first package's `short_at_preview`. |
| `remeasure-2026-11-11-method-2.json`, `w3-readings.json` | Added after the record was made (see "Addendum" at the end): `measure_preview.py measure --method 2` on the same BMPs, and the index that names it for `tools/w3_groups.py`. `measurements.json` is unchanged. |
| `crops/` | 420 × 420 px JPEG crops cut from the exports for the manual review: `U1`–`U16`, `L1`–`L8`, and five chip examples (`C17`, `R17`, `R68`, `R32` with their silkscreen text; `R88` not drawn; `R93` a drawn 1210). |
| `package/` | Every file of the uploaded package, unpacked and unchanged: Gerber archive, LCSC BOM and CPL as uploaded, manifest, `board.json`, both lock files, `verification.json`, the checklist and `README.txt`. `shasum -a 256 -c SHA256SUMS` passes for all of them. `verification.json` says `supplier_preview: not uploaded` because it was written before the upload. |

No file here names a local path, and none needed redacting: the package's files carry none. The run's log is not archived, because it names the directory the run wrote to; where this page quotes it, that directory is written `<scratch>/`.

| File | SHA-256 |
|---|---|
| `top.png` | `ed53bd7e27f2acb03142c34f4af4e62f3f3e4f683ad8ca504970de88f73f28d9` |
| `bottom.png` | `8f8f6cf3bc8746a2f5e66dd65e623b65f9fd842a238b74f173cfec726c7e9140` |
| `checks.json` | `0f09f3f358e18f086b2aad93a5fc0012060ecf7952d64ce73810a772cad302f8` |
| `measurements.json` | `0b3b34f4066ad41171e39751c2a4f3fe9bd0c0e44fb3dbbfb61c59c851d1c71c` |
| `body-model.json` | `58f43f9340775caf41a82dfa38deb237adae414ea6427213538ce44afa27345d` |

## The run

- Detected board: 2 layers, 120.37 × 127.78 mm.
- BOM step: 84 part numbers detected and confirmed. Each of the 147 top and 96 bottom designators appeared once, on its side, with its part number, selected. No stock warning.
- Both exports were taken, each twice, with the same instances lacking evidence both times.
- `jlc_preview.py` exited 3: "Instances: 16 with a pin 1 mark, 137 chip bodies along their pads; unverified: R88, R91, R92, R94, R95, R96, R102, R107, R110, R111, R112, R116, R118, R127, R128; manual review: C4, C7, C9, … R163" (75 instances; the full list is `checks.json`, which the log names as `<scratch>/run2/preview/checks.json`).

| `checks.json` status | Top | Bottom | All |
|---|---:|---:|---:|
| `mark` (one pin 1 mark of its own) | 8 | 8 | 16 |
| `body` (chip body along its pads) | 102 | 35 | 137 |
| `manual review` | 22 | 53 | 75 |
| `unverified` (no model drawn) | 15 | 0 | 15 |

The 16 `mark` instances are the four IC part numbers; `jlc_preview.py` checks only that each has one mark of its own, not where it is. Where each mark is, is in A and B below.

## Why 75 instances were left for manual review

Nothing is wrong with the placements. The 75 are of two kinds.

**8 are the inductors** (L1–L8). They are drawn, have no pin 1 mark (they are not polarised) and are not `passive::CHIP_*` lands, so `jlc_preview.py` has no check for them: "drawn, with no pin 1 mark of its own". They are read with `body_model.py` below.

**67 are chip bodies whose reading failed `measure_preview.body_ok`**: 58 by angle (over 10° from the pads' axis), 27 by elongation (under 1.1), 14 by offset (over the per-part limit), some by more than one. 57 are on `CHIP_0402`, 7 on `CHIP_0201`, 2 on `CHIP_0603` and 1 on `CHIP_2010`. They are not the 37° or 90° copies in particular: every angle has them, and the bottom has more (49 of 84 drawn chip bodies) than the top (18 of 120).

The cause is in the measurement, not in the bodies. `measure_preview.body` takes every pixel within 1.5 pad half-spacings of the origin that is not board green, bare copper or marker black. On this board that window also holds part of each part's own white silkscreen reference text (`crops/C17.jpg`, `R17.jpg`, `R68.jpg`, `R32.jpg`). The text is not the model, it sits to one side of the part, and it turns with the part. At 31 px/mm an 0402 model is about 31 × 16 px, so a few hundred white pixels of text beside it turn its principal axis and pull its centroid.

`body_model.py` reads the same exports with the same calibration and window, but keeps only the pixels connected to the part's origin: the drawn model, which board green and bare copper separate from the text. The text is a median 11% of the window's pixels.

| All 204 drawn chip bodies | `measurements.json` (model and text) | `body-model.json` (model alone) |
|---|---:|---:|
| Over 10° from the pads' axis | 58 | 0 |
| Largest angle | 78.5° | 5.2° |
| Largest offset, as a share of the per-part limit | 1.54 (R68) | 0.61 (R68) |
| Largest offset on `CHIP_0402` (limit 0.127 mm) | 0.142 mm | 0.053 mm |
| Median offset on `CHIP_0402` | 0.079 mm | 0.020 mm |
| Smallest elongation | 1.01 | 1.23 |

The 67 chip bodies left for manual review read, as the model alone: within 5.0° of their pads' axis, within 0.049 mm of the origin (at most 0.61 of the limit), elongation at least 1.62. I looked at crops of each class (0402 on top and bottom, 0201, 2010 turned) and they show the same: the body lies along its pads, centred, with the reference text beside it.

Limits of `body_model.py`: it is a record-local tool, tested on a synthetic image only (`tools/tests/test_calibration_fit.py`); on the two smallest lands the text can touch the model, or the model can break in two at this resolution (R68: 96 of 164 window pixels, 0.42 mm long where its siblings are 0.55–0.74); and the red designator the preview prints over each body is part of what it reads. **The W3 rule reads `measurements.json`**, and under it these 67 are "manual review", which pauses a group (D below). Whether `measure_preview.py` should read the model alone is a change to the rule's reading and to what earlier records reproduce, so it is not made here.

## A: OpenWalkieTalkie gap parts

Pin 1 marks (`measurements.json`; one mark at each copy; a bottom copy's `delta_ccw_from_top` is the mirror image of the top's, so it is given as seen like a top part):

| Part | Land | Top 0° | Top 37° | Bottom 0° | Bottom 37° | Mark nearest our pads | Implied `rotate` |
|---|---|---:|---:|---:|---:|---|---:|
| C7519 USBLC6-2SC6 | `esd::FP_SOT_23_6` | 100.1 | 100.8 | 100.3 | 100.1 | pad 3 0.73–0.75 mm, pad 2 1.66–1.68 | 270 |
| C131941 TPS22918DBVR | `load-switch::FP_TI_DBV0006A` | 183.0 | 183.0 | 182.6 | 181.8 | pad 4 0.07–0.12, pad 5 0.92–0.94 | 180 |
| C189961 NS4150B | `@contrib/audio-amp` `SOP8P65X490X110N` | 94.8 | 95.2 | 95.1 | 94.7 | pad 4 2.19–2.20, pad 3 2.76–2.77 | 270 |

- All three are drawn, and each reads the same quarter turn on all four copies (top and bottom, 0° and 37°), within 1.2° of each other.
- **C7519** (`crops/U5.jpg`–`U8.jpg`): the model lies a quarter turn from our land, its leads above and below where our pads are left and right, with its pin 1 dimple and the mark at the lower left; our pad 1 is at the upper left. The mark is 10° off the quarter because it sits outside the body beside the lead, not because the body is turned. Board 1 read `rotate 180` for C85364 on this same land, so the two part numbers on `esd::FP_SOT_23_6` need different rotations.
- **C131941** (`crops/U9.jpg`–`U12.jpg`): the model's leads are on our pads, with pin 1 at the lower right, on our pad 4: half a turn. Board 1 read `rotate 270` for C12044 on this same land. Again the correction is the part number's, not the land's.
- **C189961** (`crops/U1.jpg`–`U4.jpg`): the model lies a quarter turn from our land (leads above and below, our pads left and right), pin 1 at the lower left.
- These are raw readings, placed with no mapping. A candidate rotation is confirmed only by a board that places the part with it and shows the model's leads on our pads, as the re-verify boards did. None is recorded as a frame row here, candidate or otherwise.

Chip parts and inductors. "Tool" is `checks.json` (`body` / `manual review` of the four copies); the two readings are the ranges over the four copies:

| Part | Land | Tool | `measurements.json`: angle; largest offset | `body-model.json`: angle; largest offset (limit) | Model size along × across |
|---|---|---|---|---|---|
| C113780 CC0402JRNPO9BN102 | `CHIP_0402` | 3 / 1 | −15.8…+5.2°; 0.073 mm | −0.5…+1.4°; 0.022 mm (0.127) | 0.99 × 0.61 mm |
| C107006 CC0402JRNPO9BN331 | `CHIP_0402` | 0 / 4 | −35.2…+13.9°; 0.140 | −1.7…+0.8°; 0.024 (0.127) | 0.99 × 0.51 |
| C106255 CC0402KRX5R6BB474 | `CHIP_0402` | 1 / 3 | −37.6…+11.5°; 0.138 | −1.8…+0.3°; 0.019 (0.127) | 0.99 × 0.61 |
| C52923 CL05A105KA5NQNC | `CHIP_0402` | 1 / 3 | +1.5…+74.6°; 0.138 | −0.4…+1.3°; 0.037 (0.127) | 0.99 × 0.51 |
| C107039 CC0603JRNPO9BN152 | `CHIP_0603` | 3 / 1 | −10.8…+1.0°; 0.124 | −0.3…+0.1°; 0.030 (0.206) | 2.15 × 0.87 |
| C19666 CL10A475KO8NNNC | `CHIP_0603` | 3 / 1 | −10.6…+3.9°; 0.122 | −0.2…+0.0°; 0.028 (0.206) | 2.08 × 0.87 |
| C23630 CL10A225KO8NNNC | `CHIP_0603` | 4 / 0 | −1.4…+4.1°; 0.110 | −0.2…+0.3°; 0.029 (0.206) | 2.12 × 0.87 |
| C337893 DFE201612E-2R2M=P2 | `FP_L_0806_2016Metric` | 0 / 4 | not read (not a chip land) | −0.9…+3.0°; 0.035 | 1.96 × 1.73 |
| C97018 WPN201610U4R7MT | `FP_L_WPN201610U_2016Metric` | 0 / 4 | not read | −0.3…+0.5°; 0.014 | 2.02 × 1.60 |

- All nine are drawn at all four copies. As the model alone, each lies along its pads within 3.0° and is centred within 0.037 mm, the same on top and bottom and at 0° and 37°. A non-polar two-terminal part read mod 180 implies rotation 0 and no offset.
- Under the tool's own check only C23630 passes at all four copies. The other six chip parts have at least one copy left for manual review, for the reason above.
- The two inductors (`crops/L1.jpg`–`L8.jpg`): C337893 is drawn as a dark body with a terminal on each pad; C97018 as one light block over both pads, 2.0 × 1.6 mm, so its reading shows the block's long side along the pads and no more.

## B: C97521 W25Q128JVSIQ as the library ships it

`flash` 0.3.1 on `soic` 0.3.0 (`SOIC8P127X790X216N`), `supplier lcsc "C97521"` with mapping 0, as `package/cohdl.lock` pins them.

| Ref | Side, angle | `delta_ccw_from_top` | Mark to nearest pads |
|---|---|---:|---|
| U13 | top 0° | 3.8 | pad 1 0.71 mm, pad 2 1.48 mm |
| U14 | top 37° | 4.2 | pad 1 0.72, pad 2 1.45 |
| U15 | bottom 0° | 355.8 | pad 1 0.73, pad 2 1.46 |
| U16 | bottom 37° | 355.9 | pad 1 0.73, pad 2 1.48 |

Pin 1 reads 0 on all four copies: the mark is at our pad 1, beside our silkscreen pin 1 dot, and the model's leads are on our pads, four a side (`crops/U13.jpg`–`U16.jpg`). The 4° is the mark sitting just outside the end of the pad 1 lead. This is the confirming board the mapping of 0 at `soic` 0.3.0 did not have.

## E: order numbers with a page but no map row

All on `CHIP_0402`, four copies each, all drawn. Same columns as in A:

| Part | Tool | `measurements.json`: angle; largest offset | `body-model.json`: angle; largest offset (limit 0.127–0.128) | Model size |
|---|---|---|---|---|
| C25741 0402WGF1003TCE | 4 / 0 | −1.2…+3.0°; 0.097 mm | −0.3…+0.5°; 0.026 mm | 0.99 × 0.51 mm |
| C11702 0402WGF1001TCE | 1 / 3 | −14.1…+16.4°; 0.063 | −1.6…−0.2°; 0.020 | 0.99 × 0.51 |
| C26083 0402WGF1004TCE | 0 / 4 | −16.4…+13.6°; 0.062 | −1.6…+0.3°; 0.023 | 0.96 × 0.58 |
| C17168 0402WGF0000TCE | 4 / 0 | −1.5…+4.5°; 0.100 | −1.3…+1.9°; 0.030 | 0.99 × 0.51 |
| C25768 0402WGF2202TCE | 3 / 1 | −9.7…+12.4°; 0.089 | −1.6…+0.1°; 0.020 | 0.99 × 0.58 |
| C25905 0402WGF5101TCE | 3 / 1 | −12.3…+9.1°; 0.072 | −1.1…+2.5°; 0.038 | 0.99 × 0.58 |
| C25092 0402WGF220JTCE | 4 / 0 | −3.2…+8.2°; 0.084 | −0.7…+2.0°; 0.034 | 0.99 × 0.48 |
| C25890 0402WGF3301TCE | 3 / 1 | −15.4…+2.1°; 0.092 | −2.6…+3.0°; 0.024 | 0.99 × 0.55 |
| C23733 CL05A475MP5NRNC | 2 / 2 | −6.9…+2.9°; 0.108 | −2.0…+0.3°; 0.035 | 1.03 × 0.48 |
| C1548 0402CG150J500NT | 2 / 2 | −26.9…+1.0°; 0.113 | −3.9…+3.1°; 0.040 | 1.22 × 0.58 |

Three (C25741, C17168, C25092) pass the tool's check at all four copies. All ten read, as the model alone, along their pads within 3.9° and centred within 0.040 mm at every copy. These part numbers have no map row and no library entry, so a reading here is evidence for (land, part number) only.

## C: calibration

Ten part numbers, each at 0°, 90°, 180°, 270° and 37° on top and again on the bottom: 100 bodies, all drawn. `calibration_fit.py` fits each view's readings as an export shift (fixed in the board frame), a model offset (fixed in the part's frame, across and along its pads) or both. Misfits are RMS over the readings' residual vectors, in mm.

**With the tool's reading (`fit.json`), the offset turns with the part.**

| Pooling | View | Shift only: RMS | Offset only: RMS | Both: shift (x, y); RMS |
|---|---|---:|---:|---|
| whole view, 10 part numbers | top | 0.101 | 0.025 | (+0.004, −0.012); 0.022 |
| | bottom | 0.119 | 0.031 | (−0.009, −0.005); 0.029 |
| `CHIP_0402`, 7 part numbers, one shared offset | top | 0.076 | 0.039 | (+0.002, −0.015); 0.036 |
| | bottom | 0.093 | 0.040 | (−0.009, −0.005); 0.039 |

Per part number, the offset of the `both` fit is across the pads, toward the land's −y at authoring angle 0, with almost nothing along them (at most 0.017 mm):

| Part | Land | Top: across; shift size; RMS | Bottom: across; shift size; RMS |
|---|---|---|---|
| C60491 RC0402FR-07100KL | `CHIP_0402` | −0.074; 0.022; 0.023 | −0.099; 0.032; 0.024 |
| C138033 RC0402FR-071ML | `CHIP_0402` | −0.095; 0.020; 0.019 | −0.117; 0.011; 0.010 |
| C185445 RC0402FR-0720K5L | `CHIP_0402` | −0.015; 0.017; 0.019 | −0.048; 0.024; 0.027 |
| C113781 CC0402JRNPO9BN681 | `CHIP_0402` | −0.053; 0.020; 0.026 | −0.042; 0.002; 0.014 |
| C325464 CC0402KRX5R7BB105 | `CHIP_0402` | −0.104; 0.022; 0.019 | −0.110; 0.005; 0.016 |
| C481918 CRCW0402100KFKED | `CHIP_0402` | −0.064; 0.009; 0.009 | −0.088; 0.008; 0.044 |
| C482097 CRCW040220R0FKED | `CHIP_0402` | −0.076; 0.017; 0.020 | −0.095; 0.009; 0.012 |
| C138161 RC0201FR-07221RL | `CHIP_0201` | −0.062; 0.023; 0.018 | −0.078; 0.025; 0.032 |
| C877226 RC2010FK-07200KL | `CHIP_2010` | −0.200; 0.009; 0.010 | −0.220; 0.023; 0.019 |
| C878096 RC2512FK-074R7L | `CHIP_2512` | −0.136; 0.006; 0.009 | −0.152; 0.027; 0.024 |

So under this reading the data favour "model offset" strongly: a shift alone misfits about four times worse than an offset alone in both views, the offset differs by part number (0.015 to 0.220 mm; the seven 0402 part numbers do not share one, 0.015 to 0.117), and the shift left over is 0.012 mm on top and 0.010 mm on the bottom, within the scatter.

**But the "offset" is the silkscreen text, not the model.** The land's −y at authoring angle 0 is where the generator puts each part's reference text, and the text turns with the part. With the model alone (`fit-model.json`):

| Pooling | View | Shift only: shift (x, y); RMS | Offset only: RMS | Both: shift; largest offset of any part number; RMS |
|---|---|---|---:|---|
| whole view, 10 part numbers | top | (+0.002, −0.016); 0.022 | 0.026 | (+0.002, −0.016); 0.021; 0.020 |
| | bottom | (−0.011, −0.004); 0.019 | 0.020 | (−0.010, −0.005); 0.014; 0.017 |
| `CHIP_0402`, 7 part numbers | top | (+0.001, −0.019); 0.022 | 0.029 | (+0.001, −0.020); 0.004; 0.022 |
| | bottom | (−0.013, −0.006); 0.016 | 0.020 | (−0.012, −0.006); 0.004; 0.015 |

Here one shift per view fits as well as shift and offsets together (0.022 against 0.020 on top, 0.019 against 0.017 below), and no part number's own offset exceeds 0.021 mm. The shift is 0.016 mm on top and 0.012 mm on the bottom, half a pixel (one pixel is 0.032 mm). C877226's 0.20 mm becomes less than 0.005 mm. So with the text out, neither hypothesis has anything left to explain: the drawn models sit on their lands within the measurement's scatter, at every angle, on both sides.

**The view medians** that the W3 rule takes off (`w3_rule.board_bias`, over every drawn chip body of a view, from `measurements.json`):

| View | Drawn chip bodies | Median (x, y), `measurements.json` | Median, `body-model.json` |
|---|---:|---|---|
| top | 120 | (−0.005, −0.064) mm | (+0.007, −0.014) mm |
| bottom | 84 | (−0.021, −0.061) mm | (−0.010, −0.009) mm |

Their composition, drawn bodies by W3 group:

| Group | Top | Bottom |
|---|---:|---:|
| `CHIP_1210` Yageo `RC` | 28 | 0 |
| `CHIP_0402` Vishay `CRCW` | 18 | 10 |
| `CHIP_0402` Yageo `CC` | 16 | 16 |
| `CHIP_0402` Yageo `RC` | 15 | 15 |
| `CHIP_0201`, `CHIP_2010`, `CHIP_2512` Yageo `RC` | 5 each | 5 each |
| `CHIP_0603` Samsung `CL` | 4 | 4 |
| `CHIP_0603` Yageo `CC`, `CHIP_0402` Samsung `CL` | 2 each | 2 each |
| in no group (E) | 20 | 20 |

On top 63 of the 120 are at 0°, 27 at 37° and 10 each at 90°, 180° and 270°; on the bottom 27 at 0° (KiCad 180°), 27 at 37° and 10 each at the others. The top median of −0.064 mm in y is what the text does to a body at 0° (the text is above it); the turned bodies have it along x or with the other sign.

**What this says about the earlier records** (not archived there; `body_model.py` run on their exports on 2026-11-11):

- The W3 pilot (`../2026-11-07-w3-pilot-0402/`, 89 drawn `CHIP_0402` bodies, all at 0°): median (+0.005, −0.077) mm as recorded, (+0.008, −0.009) mm as the model alone; largest offset 0.121 mm as recorded, 0.046 mm as the model alone. The pilot's 0.077 mm "board bias" along −y is the same reference text above each part.
- Board 3 (`../2026-11-05-c2-board-3/`): C877226's "about 0.18 mm that turns with the part": its three top copies read 0.134–0.189 mm as recorded and 0.016–0.076 mm as the model alone. Other lands there keep model-alone offsets of 0.04–0.08 mm in different directions, which this board cannot explain.

**For the 0402 Yageo `RC` group (PR #107), as facts:**

- Its three calibration part numbers (C60491, C138033, C185445; 30 bodies, all drawn) are judged by the rule on main from `measurements.json`. Each fails: 12 of the 30 copies are "manual review", and the copies with status `body` have residuals up to 0.130 mm against the 0.064 mm the rule allows (C138033 R23). Raw offsets reach 0.142 mm (R17, R20) against the 0.128 mm limit.
- The same 30 bodies as the model alone: within 4.0° and 0.053 mm (0.42 of the limit); median 0.023 mm.
- The pilot's passed samples of this group had raw offsets up to 0.121 mm (C185445, the largest of its group) against a limit of 0.128 mm, which is the margin #107 rests on. As the model alone the pilot's largest offset over all 89 bodies is 0.046 mm. So the small margin was the text's, and the pilot's samples were at one angle with the text on one side, which the median then took off.
- A fresh `w3_groups.py` report on a tree with this record marks the group `paused` (3 members failed). The dated reports are unchanged. Which reading the rule should use, and whether these three are failures, is Tony's decision.

## D: W3 samples

15 instances were not drawn: bare pads and the centroid marker (`crops/R88.jpg`), in both exports of the top view. All are D samples, at 0° on top:

| Group | Not drawn |
|---|---|
| `CHIP_1210` Yageo `RC` (14 of 42 samples) | R88 C875203 RC1210JR-0715KL, R91 C875206 RC1210JR-07180KL, R92 C874782 RC1210FR-071K47L, R95 C7276637 RC1210FR-071M6L, R96 C875217 RC1210JR-071R5L, R102 C875225 RC1210JR-0727KL, R107 C875232 RC1210JR-0730KL, R110 C875234 RC1210JR-0736KL, R111 C874966 RC1210FR-073K01L, R112 C875236 RC1210JR-073KL, R116 C874999 RC1210FR-0743RL, R118 C875026 RC1210FR-074K02L, R127 C875251 RC1210JR-076K2L, R128 C875123 RC1210FR-0771R5L |
| `CHIP_0402` Vishay `CRCW` (1 of 9 samples) | R94 C2085363 CRCW04021M30FKED |

C7276637 is one of the two replacements for the samples the first package found short. Nine of the 14 are `JR` (5%) parts and five `FR` (1%).

Under the rule on main (`w3-2026-11-10b`), from `measurements.json` and `checks.json`:

| Group | Samples here | Passed | Failed | Not drawn | Group after this record |
|---|---:|---:|---:|---:|---|
| `CHIP_1210` Yageo `RC` (846 members, 43 required) | 42 | 28 | 0 | 14 | 29 passed, 15 not drawn in all, 14 more needed: `needs_samples` |
| `CHIP_0402` Vishay `CRCW` (968 members, 49 required) | 9 | 7 | 1 | 1 | 45 passed, 10 not drawn in all, 4 more needed: `paused` |

- The 28 passed 1210 samples: largest raw offset 0.087 of 0.250 mm, largest residual 0.045 of 0.125 mm, angle within 2.9°. As the model alone: within 0.8° and 0.036 mm.
- The 7 passed `CRCW` samples: largest raw 0.074 of 0.128 mm, largest residual 0.034 of 0.064 mm.
- The failed `CRCW` sample is C482153 (R108): "manual review" at 10.1° against the 10° limit, raw 0.044 mm, residual 0.021 mm. As the model alone it reads 5.2° and 0.016 mm. The group is also paused by its two calibration part numbers on this board (C481918, C482097), whose turned copies fail as in C.
- The share not drawn: 14 of 42 (33%) of the 1210 samples on this board, 1 of 9 of the `CRCW` samples. Over everything judged so far: Yageo `RC` 1210, 15 of 44 judged (34.1%); Vishay `CRCW` 0402, 10 of 58 (17.2%).

Under PR #112's proposed conditions (`w3-2026-11-11`, branch `tools/w3-followup-conditions` at `ee8fe3e`, run on a scratch copy of that branch with this record added):

- **Other groups in the median:** met for both. The top view's median is over 120 drawn bodies; 92 are not Yageo `RC` 1210 (76.7%) and 102 are not Vishay `CRCW` 0402 (85.0%). No sample is `bias_unsupported`. Passed and failed counts are as on main.
- **Not-drawn share:** both groups exceed the proposed 10%. Yageo `RC` 1210 at 34.1% is `paused` for that alone (on main it is `needs_samples`). Vishay `CRCW` 0402 at 17.2% is over it too, besides its failures.

So on this board the 1210 group gets 28 passed samples toward 43, and a third of its part numbers sampled have no model. On main that leaves it needing 14 more samples from part numbers that are drawn; under #112 it pauses.

## What remains unverified

- The 15 part numbers not drawn: no reading.
- C7519, C131941 and C189961: the rotations are read from raw placements, not confirmed with the mapping applied.
- The two inductors and every chip part left for manual review: read here by a record-local tool and by eye, not passed by `jlc_preview.py`.
- The E part numbers have no map row or library entry.
- `body_model.py` itself: one board, one resolution, a synthetic test.

## Addendum, 2026-11-11: read again with `measure_preview.py` method 2

Everything above is the record as it was made, and stays as written: where it says "the tool" or "`measure_preview.py`" it means the reading the tool had then, now called method 1. This section was added with the change that gives `measure_preview.py` a second method, the drawn model alone (`body_model`), which is what `body_model.py` here did by hand. That change is a proposal for Tony (`docs/library-supplier-w3-sampling.md` §6); nothing is bound from it.

- `remeasure-2026-11-11-method-2.json` (SHA-256 `963ac558c9e2589efa5fe0a17e9ed71df7b1147c1c5b1133abac7405e0a509fc`) is `measure_preview.py measure --method 2 package/board.json top.bmp bottom.bmp`, on the same BMPs. Its rows are those of `measurements.json` apart from each chip body's `body` and `body_ok`. `measurements.json` still reproduces with `measure --method 1`.
- All 204 drawn chip bodies pass the body check: largest angle 6.3° (C32), largest offset 0.053 mm (R46, 0.41 of its limit), smallest elongation 1.23. View medians: top (+0.004, −0.011) mm, bottom (−0.010, −0.009) mm.
- Method 2 is not `body_model.py` unchanged. It also keeps thin runs of copper colour inside a model (the export's blend where the red designator meets a white terminal, which had cut R68's terminals off: 0.42 mm long here, 0.55 mm by method 2, offset 0.049 → 0.017 mm), leaves out a pad's bare copper only where it is at least 3 px across, and joins the model through paths at least 3 px wide, so text that grazes a model is cut off. Its readings differ from `body-model.json` by up to 0.045 mm and 6.9°; `body-model.json`, `fit-model.json` and `body_model.py` stay as the record's own, and the tests still pin them.
- `jlc_preview.py`'s check, run on these exports as it is now: 204 `body`, 16 `mark`, 15 `unverified`, 8 `manual review` (L1–L8, which have no check). All 67 chip instances left for manual review above pass.
- Under the proposed rule (`w3-2026-11-11-m2`, the accepted limits with the reading method named), this record fails no chip sample and pauses no group: `tools/sourcing_data/w3_sampling/2026-11-11-method-2/`. Section D's table above is the rule on main.
