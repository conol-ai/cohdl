# OpenWalkieTalkie Rev C: assembler preview, 2026-11-12 (three ICs, seven chip parts, two inductors)

This is an extract of the assembler's placement preview of a real board, OpenWalkieTalkie Rev C, taken on 2026-11-12. It holds what twelve frame rows in `tools/sourcing_data/jlc_frames.json` rest on, and nothing else is claimed from it. Three are ICs placed with a rotation (below); nine are `passive` parts placed with none ([Chip parts and inductors](#chip-parts-and-inductors)):

| Row | LCSC | Part | Land | Mapping applied | Placements |
|---|---|---|---|---|---|
| `owtc-fpsot236-c7519` | C7519 USBLC6-2SC6 | `esd::ESD_USBLC6` | `esd::FP_SOT_23_6` | `rotate 270` | U20 (top) |
| `owtc-fptidbv0006a-c131941` | C131941 TPS22918DBVR | `load_switch::LOADSW_TPS22918DBVR` | `load_switch::FP_TI_DBV0006A` | `rotate 180` | U2, U10 (bottom) |
| `owtc-sop8p65x490x110n-c189961` | C189961 NS4150B | `contrib_audio_amp::AMP_NS4150B` | `contrib_audio_amp::SOP8P65X490X110N` | `rotate 270` | U1 (top) |
| `owtc-chip0603-c107039` | C107039 CC0603JRNPO9BN152 | `passive::C_1n5_50V_C0G_0603` | `passive::CHIP_0603` | none | C2 (bottom) |
| `owtc-chip0402-c113780` | C113780 CC0402JRNPO9BN102 | `passive::C_1n_50V_C0G_0402` | `passive::CHIP_0402` | none | C32, C33, C34, C35, C36, C37, C38, C39 (top); C6, C50 (bottom) |
| `owtc-chip0402-c107006` | C107006 CC0402JRNPO9BN331 | `passive::C_330p_50V_C0G_0402` | `passive::CHIP_0402` | none | C8, C9, C10 (top) |
| `owtc-chip0603-c19666` | C19666 CL10A475KO8NNNC | `passive::C_4u7_16V_X5R_0603` | `passive::CHIP_0603` | none | C40, C41, C48 (top); C18 (bottom) |
| `owtc-chip0402-c106255` | C106255 CC0402KRX5R6BB474 | `passive::C_470n_10V_X5R_0402` | `passive::CHIP_0402` | none | C25 (bottom) |
| `owtc-chip0402-c477807` | C477807 RC0402FR-0786K6L | `passive::R_86K6_F_0402` | `passive::CHIP_0402` | none | R31 (bottom) |
| `owtc-chip0402-c137990` | C137990 RC0402FR-073K6L | `passive::R_3K6_F_0402` | `passive::CHIP_0402` | none | R37 (top) |
| `owtc-fplwpn201610u2016metric-c97018` | C97018 WPN201610U4R7MT | `passive::IND_4U7_WPN201610U` | `passive::FP_L_WPN201610U_2016Metric` | none | L3 (bottom) |
| `owtc-fpl08062016metric-c337893` | C337893 DFE201612E-2R2M=P2 | `passive::IND_2U2` | `passive::FP_L_0806_2016Metric` | none | L4, L5 (bottom) |

For the three ICs it is the second of two readings. The first is preview board 4 (`../2026-11-11-board-4/`, section A), which placed four copies of each part with **no** mapping and read where the assembler's pin 1 lay. This preview placed the same part numbers **with** the rotation that reading implied, and reads pin 1 on our pad 1.

## Where it comes from

The board is a customer board, not a CoHDL frame board. Its full record is in the board's own repository: `conol-ai/openwalkietalkie`, branch `hardware/rev-c`, commit `560f8b4b7056e9fba917a968c60aefcddcbea807` (draft PR conol-ai/openwalkietalkie#5), directory `hardware/rev-c/preview/2026-11-12/`. That record has all 118 previewed placements, its README, the run log and the uploaded package's files. The same board was previewed a second time that day (`hardware/rev-c/preview/2026-11-12-iteration-2/`, commit `733535aafc2639094c44e7c4071ae9f88ed3bef9`); that record is cited below by hash and not copied. The run had the board owner's approval, for the preview only. `tools/frame_boards/jlc_preview.py` at commit `1fe7273` ran it, on the Standard PCBA type, and stopped at Component Placements: nothing was added to the cart and no order was placed.

The uploaded package is `owt-revc-preview.zip`, SHA-256 `8254b63a3369b7d8adab818bc40975ccdda4b08b445f1494b4ef4f5f5d5d5847`, built at that repository's commit `9b3e7ddd5685f71c1156068bc348344bec3ad999`. The package is not copied here. `source-package-SHA256SUMS` lists its members and `source-SHA256SUMS` every file of the source record, both as that record has them.

The board pinned the same library package versions as preview board 4, with the same content hashes in both lock files: `esd` 0.1.2 (`sha256:6195444e…95f6`), `load-switch` 0.2.0 (`sha256:2306e411…bc96`) and `@contrib/audio-amp` 0.2.0 (`sha256:a1ac6087…b2af`). The three lands are unchanged in the versions that carry the bindings (`esd` 0.1.4, `load-switch` 0.2.1, `@contrib/audio-amp` 0.2.1). Both boards also pinned `passive` 0.2.6 (`sha256:d1f973fb…832e`). Its chip lands (`pads.cohdl`, `footprints.cohdl`) and `inductors.cohdl` are byte for byte the same in `passive` 0.2.8, which carries the nine bindings.

## Files

| File | What it is |
|---|---|
| `exports/top.png`, `exports/bottom.png` | The preview's own 2D whole-board exports, byte for byte (JPEG data under a `.png` name), 2271 × 3819 px, 39.4 px/mm. `bottom.png` is the view from below, mirrored left to right. They are in `exports/`, as in the source record, and not beside this page: `tools/tests/test_measure_preview.py` compares every export it finds there with the green extent of its middle row and column, which describes a bare frame board and not this one, whose middle row and column cross parts and silkscreen. `tools/tests/test_owt_rev_c_record.py` checks these two against the board's bounding box instead. |
| `board-extract.json` | The board outline's bounding box and the 28 footprints claimed here (reference, footprint, side, position, angle, pads), cut unchanged and in order from the package's `board.json`. |
| `measurements.json` | `measure_preview.py measure --attribution 1 board-extract.json top.bmp bottom.bmp`, with the exports converted by `sips -s format bmp`. Its 28 rows equal the same 28 rows of the source record's `measurements.json`, which was measured with the whole `board.json`. A chip part's row has a `body` read by method 2; an inductor's row has none, because the tool reads a body only on a `passive` `CHIP_*` land. |
| `inductor-bodies.json` | The same method 2 body reading (`measure_preview.body`) run on the three inductor placements, with `ok` its `body_ok`. It equals the `bodies` entries for L3, L4 and L5 of the source record's `features.json`. |
| `placements.csv`, `part-numbers.csv` | The 28 placement rows and 12 part-number rows of the source record's tables, with their header lines. |
| `lcsc-bom-rows.csv`, `lcsc-cpl-rows.csv` | The 12 BOM rows and 28 CPL rows of the package as uploaded, with their header lines. The CPL rotation already includes the mapping. |
| `crops/` | The source record's crops of the four IC and three inductor placements (it has none of chip parts): the export's pixels doubled, a cyan ring on our pad 1, a yellow square on each other pad, a white cross on the footprint origin. Bottom crops are the view from below. |
| `source-SHA256SUMS`, `source-package-SHA256SUMS` | The source record's and the package's own hash lists. |

No file here names a local path.

| File | SHA-256 |
|---|---|
| `exports/top.png` | `e66cfd404f5dbf3eca799423d844adf9cce89c91cff57db088b680d9ee622464` |
| `exports/bottom.png` | `1ca0b8ee6926c7e62fc4566e176e2b74a9cb79c2131020aca2eb03de785640a0` |
| `board-extract.json` | `9554e5644ed9d3b394eb5ba6f2dfddbc898316428b3e087ff713c357797cb173` |
| `measurements.json` | `7be7b7269675471a32135f33f15d44e5980000e4c1b0abe8150e9235402fd462` |
| `inductor-bodies.json` | `e250d8357abe762d32ad68a92dfd58bb08a60e5a323b09f8bdc6ce91fb30a397` |
| `placements.csv` | `27e5b846972f5d8e7bddfa774db67a2c5e69faf7f49320df3fc17b149f0cecb8` |
| `part-numbers.csv` | `ad79b2869fcbc5409fe8001debd64dce73e13041d7593b29a47897de11bae7f2` |
| `lcsc-bom-rows.csv` | `21d40a82089c3dad8cdd690393128d92c782363d943353c0b3471f7dceb5a08a` |
| `lcsc-cpl-rows.csv` | `8ffedf1ed913d83b36061c965f5f47d05253b45c058f8f5d9741492f0f695803` |
| `source-SHA256SUMS` | `b18b8d6399ceace2ed924f151becb868dfcc0658cc3f627bd9c0f99d2a97ce5a` |
| `source-package-SHA256SUMS` | `4fa078ca1399ac54d91a13d22ec40c548729481228e244e68506ff8d62573625` |
| `crops/L3-bottom.png` | `de2d2041fc1e3c9c0d609139eb6c63b817ab0e9056f141625b0b2a7c1bfbf656` |
| `crops/L4-bottom.png` | `896d761107a89f3158e3c2b32e128960a1e8f46a3b19b4f9b2f4aad55f2603eb` |
| `crops/L5-bottom.png` | `293a1b72826002a48b8c7cb9c49876052473242ded5119d076f7e19251bd9bec` |
| `crops/U1-top.png` | `79a1606822e6ee6088fb54849c526668a7294b245527101a720f274c1bc34e35` |
| `crops/U10-bottom.png` | `78395ee4295dab1faf5670368a149851ed26ebeac0e0d3e84a6680bc825c69d0` |
| `crops/U2-bottom.png` | `647ea500687cfe03ceeff236f63bd7106604e42148f282190d961509ec7211b2` |
| `crops/U20-top.png` | `a2b7554b07fbee8907725f360b74c760dbc47cc249cd2e86f4a397f8500913f1` |

The two exports and the seven crops have the hashes `source-SHA256SUMS` gives for them.

## Readings: the three ICs

`delta_ccw_from_top` is the angle from our pad 1 to the assembler's pin 1 mark, about the footprint origin; 0° means the mark is on pad 1's side. A bottom placement is read as the tool reads it, from below.

| Placement | LCSC | Side | Board angle | Mapping applied | CPL rotation sent | Pad 1 (mm) | Pin 1 mark (mm) | Reading |
|---|---|---|---:|---|---:|---|---|---:|
| U20 | C7519 | top | −90° | `rotate 270` | 180 | (+0.95, −1.15) | (+0.90, −1.69) | 11.7° |
| U2 | C131941 | bottom | −90° | `rotate 180` | 90 | (−0.95, −1.30) | (−0.95, −1.43) | 357.5° |
| U10 | C131941 | bottom | +90° | `rotate 180` | 270 | (+0.95, +1.30) | (+0.97, +1.38) | 358.9° |
| U1 | C189961 | top | −90° | `rotate 270` | 180 | (+0.975, −2.175) | (+0.94, −2.87) | 5.9° |

- **U20, C7519** (`crops/U20-top.png`): the model's six leads sit on our six pads and its pin 1 dimple is at our pad 1's corner. The mark lies beside pad 1's lead, outside the body, which is why it reads 11.7° and not 0°; board 4's raw reading was 10° past the quarter for the same reason.
- **U2 and U10, C131941** (`crops/U2-bottom.png`, `crops/U10-bottom.png`): leads on pads, pin 1 dimple at pad 1's corner, the mark 0.13 mm and 0.08 mm from pad 1's centre. The two copies are half a turn apart on the board.
- **U1, C189961** (`crops/U1-top.png`): the model's eight leads sit on our eight pads and the mark lies on pad 1's lead, 0.70 mm past the pad's centre.

`jlc_preview.py` listed U1 and U2 for manual review: it gives each mark to the nearest footprint, and a chip capacitor is nearer to each of those two marks than the IC is (the source record's README has the distances). `measure_preview.py measure` reads all four.

Against preview board 4, which applied no mapping (its `measurements.json`, top copies at 0° and 37°, then bottom copies as seen like a top part):

| LCSC | Board 4, no mapping | Implied | This board, mapping applied |
|---|---|---:|---|
| C7519 | 100.1°, 100.8°, 100.3°, 100.1° | `rotate 270` | 11.7° |
| C131941 | 183.0°, 183.0°, 182.6°, 181.8° | `rotate 180` | 357.5°, 358.9° |
| C189961 | 94.8°, 95.2°, 95.1°, 94.7° | `rotate 270` | 5.9° |

## Chip parts and inductors

Nine `passive` part numbers were placed with no mapping (rotation 0, no offset) and read by the drawn body, method 2: the angle of the body's long axis from the pad axis, mod 180, and the distance of its centre from the footprint origin, against the land's limit (the lesser of 0.25 mm and a quarter of the pad half-spacing). None has a pin 1 mark or a polarity.

| LCSC | Manufacturer part | Library entry | Land | Copies | Axis | Largest offset (limit) | Board 4, four copies: axis; largest offset |
|---|---|---|---|---:|---|---|---|
| C107039 | Yageo CC0603JRNPO9BN152 | `C_1n5_50V_C0G_0603` primary | `CHIP_0603` | 1 | +0.1° | 0.013 mm (0.206) | −0.7…+0.1°; 0.030 mm |
| C113780 | Yageo CC0402JRNPO9BN102 | `C_1n_50V_C0G_0402` primary | `CHIP_0402` | 10 | −0.2…+0.4° | 0.024 mm (0.127) | −1.1…+1.6°; 0.026 mm |
| C107006 | Yageo CC0402JRNPO9BN331 | `C_330p_50V_C0G_0402` primary | `CHIP_0402` | 3 | −0.3…+0.1° | 0.022 mm (0.127) | −1.9…+0.8°; 0.026 mm |
| C19666 | Samsung CL10A475KO8NNNC | `C_4u7_16V_X5R_0603` alt | `CHIP_0603` | 4 | −0.2…0.0° | 0.014 mm (0.206) | −0.5…+0.4°; 0.024 mm |
| C106255 | Yageo CC0402KRX5R6BB474 | `C_470n_10V_X5R_0402` primary | `CHIP_0402` | 1 | 0.0° | 0.014 mm (0.127) | −2.9…+0.4°; 0.019 mm |
| C477807 | Yageo RC0402FR-0786K6L | `R_86K6_F_0402` primary | `CHIP_0402` | 1 | −0.1° | 0.003 mm (0.127) | not placed |
| C137990 | Yageo RC0402FR-073K6L | `R_3K6_F_0402` primary | `CHIP_0402` | 1 | 0.0° | 0.013 mm (0.128) | not placed |
| C97018 | Sunlord WPN201610U4R7MT | `IND_4U7_WPN201610U` primary | `FP_L_WPN201610U_2016Metric` | 1 | 0.0° | 0.012 mm (0.175) | −0.3…+0.5°; 0.014 mm (`body-model.json`) |
| C337893 | Murata DFE201612E-2R2M=P2 | `IND_2U2` primary | `FP_L_0806_2016Metric` | 2 | 0.0° | 0.013 mm (0.237) | −0.9…+3.0°; 0.035 mm (`body-model.json`) |

- **Which entry.** Each part number is bound on the entry `tools/sourcing_data/lcsc_map.json` gives it by manufacturer and exact part number, not on the part the board happened to use it for. C19666 is the 16 V Samsung `CL10A475KO8NNNC`; the board placed it for a 10 V part, and the library lists it as an alternate of `C_4u7_16V_X5R_0603`.
- **All 24 bodies pass the tool's check** (`body_ok`): drawn, along the pads within 0.4°, centred within 0.024 mm, on both sides and at board angles of 0°, ±90° and 180°.
- **The inductors** (`crops/L3-bottom.png`, `crops/L4-bottom.png`, `crops/L5-bottom.png`; `inductor-bodies.json`). C337893 is drawn as a dark body with a terminal on each pad. C97018 is drawn as one light block over both pads, 2.01 × 1.60 mm, so its reading shows the block's long side along the pads and its centre on them, and no more.
- **Board 4** (`../2026-11-11-board-4/`, section A and the method 2 addendum) placed four copies of seven of the nine, top and bottom, at 0° and 37°, with no mapping. Chip figures are from its `remeasure-2026-11-11-method-2.json`, the inductors' from its `body-model.json`.
- **C477807 and C137990** were not on board 4. They are members of the W3 group `CHIP_0402` / Yageo `RC`, bound since `passive` 0.2.7 by inference from that group's samples. Their rows here are their own evidence and replace the inferred rows. They are not W3 samples: this record has no `w3.json` (it is a customer board with no view median over a frame board's chips), so `tools/w3_groups.py` lists them as direct rows it does not judge.
- **The second preview** of the same board that day placed these 24 parts with the same CPL rows and read them again: the largest axis 0.4° and the largest offset 0.024 mm, every body passing. Its files are not copied here. In the source repository at commit `733535aafc2639094c44e7c4071ae9f88ed3bef9`, `hardware/rev-c/preview/2026-11-12-iteration-2/`: `measurements.json` `53226fd350623d66dfb6cd64bfc6513a15d279a187807ca38689bef663538aa8`, `features.json` (inductor bodies) `57d7aae9b3ff05ef17c99ebe0a2f55063072d687cb4101d23e3b79ddd0af38e9`, `exports/top.png` `cfce07fe671314aa24543d142407065c5f8b56a5a3572d8a4b464f64f33f96f5`, `exports/bottom.png` `9c9226078416dba012366327682a37dc633b979f112cdeea370c07135dc11ff0`.

Each placement, from `measurements.json` (chip parts) and `inductor-bodies.json` (L3–L5):

| Placement | LCSC | Side | Board angle | Axis | Offset (x, y) mm | Offset mm | Limit mm | Model along × across mm | Elongation |
|---|---|---|---:|---:|---|---:|---:|---|---:|
| C2 | C107039 | bottom | 90° | +0.1° | (−0.007, −0.011) | 0.013 | 0.206 | 2.41 × 0.86 | 2.94 |
| C6 | C113780 | bottom | −90° | 0.0° | (−0.019, −0.007) | 0.020 | 0.127 | 1.47 × 0.58 | 2.71 |
| C32 | C113780 | top | −90° | +0.4° | (−0.024, +0.000) | 0.024 | 0.127 | 1.50 × 0.63 | 2.66 |
| C33 | C113780 | top | 90° | −0.2° | (+0.009, −0.003) | 0.009 | 0.127 | 1.47 × 0.58 | 2.74 |
| C34 | C113780 | top | 90° | +0.2° | (+0.009, +0.002) | 0.010 | 0.127 | 1.47 × 0.58 | 2.75 |
| C35 | C113780 | top | 90° | −0.1° | (+0.010, +0.003) | 0.010 | 0.128 | 1.50 × 0.58 | 2.77 |
| C36 | C113780 | top | −90° | +0.1° | (−0.012, +0.000) | 0.012 | 0.127 | 1.50 × 0.58 | 2.68 |
| C37 | C113780 | top | −90° | −0.1° | (−0.006, −0.004) | 0.007 | 0.127 | 1.50 × 0.56 | 2.89 |
| C38 | C113780 | top | 0° | 0.0° | (−0.007, +0.002) | 0.007 | 0.127 | 1.47 × 0.58 | 2.69 |
| C39 | C113780 | top | 180° | −0.1° | (+0.006, +0.003) | 0.007 | 0.127 | 1.50 × 0.58 | 2.75 |
| C50 | C113780 | bottom | 180° | 0.0° | (+0.003, −0.010) | 0.010 | 0.127 | 1.47 × 0.58 | 2.67 |
| C8 | C107006 | top | 0° | −0.3° | (+0.001, −0.021) | 0.021 | 0.128 | 1.50 × 0.58 | 2.75 |
| C9 | C107006 | top | 180° | −0.1° | (+0.001, −0.022) | 0.022 | 0.128 | 1.47 × 0.58 | 2.70 |
| C10 | C107006 | top | 90° | +0.1° | (−0.015, −0.006) | 0.016 | 0.127 | 1.47 × 0.58 | 2.70 |
| C18 | C19666 | bottom | 0° | −0.2° | (−0.006, −0.011) | 0.012 | 0.206 | 2.36 × 0.89 | 2.81 |
| C40 | C19666 | top | 180° | 0.0° | (+0.007, −0.007) | 0.010 | 0.206 | 2.36 × 0.89 | 2.80 |
| C41 | C19666 | top | 0° | 0.0° | (−0.007, −0.005) | 0.009 | 0.206 | 2.36 × 0.89 | 2.77 |
| C48 | C19666 | top | −90° | 0.0° | (−0.006, −0.012) | 0.014 | 0.206 | 2.36 × 0.89 | 2.78 |
| C25 | C106255 | bottom | 180° | 0.0° | (−0.002, −0.014) | 0.014 | 0.127 | 1.50 × 0.58 | 2.74 |
| R31 | C477807 | bottom | 180° | −0.1° | (+0.001, −0.003) | 0.003 | 0.127 | 1.47 × 0.58 | 2.72 |
| R37 | C137990 | top | 90° | 0.0° | (−0.013, −0.003) | 0.013 | 0.128 | 1.50 × 0.58 | 2.78 |
| L3 | C97018 | bottom | 180° | 0.0° | (+0.001, −0.012) | 0.012 | 0.175 | 2.01 × 1.60 | 1.25 |
| L4 | C337893 | bottom | 0° | 0.0° | (−0.002, −0.013) | 0.013 | 0.237 | 2.77 × 1.65 | 1.72 |
| L5 | C337893 | bottom | 0° | 0.0° | (+0.000, −0.004) | 0.004 | 0.237 | 2.77 × 1.70 | 1.70 |

## Limits

- One placement of C7519 and of C189961, two of C131941, all at ±90° on the board. Board 4 covers 0° and 37°, top and bottom, without the mapping.
- A chip or inductor body reading is blind to a half turn. That does not matter for a part with no polarity, and each row's note says the orientation is known mod 180 only.
- C106255, C107039, C477807, C137990 and C97018 have one placement each on this board. C106255, C107039 and C97018 also have four copies each on board 4. C477807 and C137990 were read on this board only: once each here, and again in its second preview.
- The inductors are judged by the drawn body alone, as chip parts are. `measure_preview.py` does not read a body off a `CHIP_*` land by itself, and no recorded rule names inductors the way `docs/library-supplier-plan.md` §W3 names chip parts and bidirectional TVS parts. C97018's model is one block, so its reading shows the block's axis and centre, not terminals on pads.
- The outline is drawn with a 0.05 mm stroke where the tool assumes 0.1 mm, so a position reads up to 0.025 mm toward the board centre (the source record's README). The largest offset claimed here is 0.024 mm against a limit of 0.127 mm.
- These mappings belong to the part number, not to the land. On `esd::FP_SOT_23_6` C85364 read `rotate 180` and on `load_switch::FP_TI_DBV0006A` C12044 read `rotate 270` (C2 frame board 1, `../2026-10-30-c2-board-1/`).
- Nothing else on this board is claimed here: not its other chip parts, its ferrite beads, connectors or switches, and not the parts its own record marks as needing a mapping. In particular the ten UNI-ROYAL resistor numbers it read on `CHIP_0402` (C11702, C25741, C25768, C25092, C26083, C25905, C25744, C25890, C17168, C25760) have no library entry yet, and C15849 and C97946 have no identity row in `lcsc_map.json`: `lib/passive/docs/README.md` lists them.

## Reproducing

```sh
sips -s format bmp exports/top.png --out top.bmp
sips -s format bmp exports/bottom.png --out bottom.bmp
python3 -I -B tools/frame_boards/measure_preview.py measure --attribution 1 board-extract.json top.bmp bottom.bmp out.json
```

`out.json` equals `measurements.json` byte for byte. `inductor-bodies.json` is `measure_preview.body(view, footprint)` for L3, L4 and L5 on the same two BMPs, calibrated on the same `board-extract.json`, each with `"ok": body_ok(reading)`, written with `json.dumps(…, indent=1)`. `tools/tests/test_owt_rev_c_record.py` runs both on macOS.

`measurements.json` was made before the tool numbered how a mark is given to a part, so it is an attribution 1 file (`measure_preview.ATTRIBUTION`) and reproduces with `--attribution 1`. Under attribution 2, the current one, each of the four rows has the same one mark and the same angle, and passes as a mark at pad 1 (`mark_ok`); the same test checks that.

## Licence

The exports and crops are the assembler's renderings of a board published under the MIT licence, kept as the evidence the readings rest on.
