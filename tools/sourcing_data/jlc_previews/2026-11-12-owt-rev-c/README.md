# OpenWalkieTalkie Rev C: assembler preview, 2026-11-12 (three library parts)

This is an extract of the assembler's placement preview of a real board, OpenWalkieTalkie Rev C, taken on 2026-11-12. It holds what three frame rows in `tools/sourcing_data/jlc_frames.json` rest on, and nothing else is claimed from it:

| Row | LCSC | Part | Land | Mapping applied | Placements |
|---|---|---|---|---|---|
| `owtc-fpsot236-c7519` | C7519 USBLC6-2SC6 | `esd::ESD_USBLC6` | `esd::FP_SOT_23_6` | `rotate 270` | U20 (top) |
| `owtc-fptidbv0006a-c131941` | C131941 TPS22918DBVR | `load_switch::LOADSW_TPS22918DBVR` | `load_switch::FP_TI_DBV0006A` | `rotate 180` | U2, U10 (bottom) |
| `owtc-sop8p65x490x110n-c189961` | C189961 NS4150B | `contrib_audio_amp::AMP_NS4150B` | `contrib_audio_amp::SOP8P65X490X110N` | `rotate 270` | U1 (top) |

It is the second of two readings. The first is preview board 4 (`../2026-11-11-board-4/`, section A), which placed four copies of each part with **no** mapping and read where the assembler's pin 1 lay. This preview placed the same part numbers **with** the rotation that reading implied, and reads pin 1 on our pad 1.

## Where it comes from

The board is a customer board, not a CoHDL frame board. Its full record is in the board's own repository: `conol-ai/openwalkietalkie`, branch `hardware/rev-c`, commit `560f8b4b7056e9fba917a968c60aefcddcbea807` (draft PR conol-ai/openwalkietalkie#5), directory `hardware/rev-c/preview/2026-11-12/`. That record has all 118 previewed placements, its README, the run log and the uploaded package's files. The run had the board owner's approval, for the preview only. `tools/frame_boards/jlc_preview.py` at commit `1fe7273` ran it, on the Standard PCBA type, and stopped at Component Placements: nothing was added to the cart and no order was placed.

The uploaded package is `owt-revc-preview.zip`, SHA-256 `8254b63a3369b7d8adab818bc40975ccdda4b08b445f1494b4ef4f5f5d5d5847`, built at that repository's commit `9b3e7ddd5685f71c1156068bc348344bec3ad999`. The package is not copied here. `source-package-SHA256SUMS` lists its members and `source-SHA256SUMS` every file of the source record, both as that record has them.

The board pinned the same library package versions as preview board 4, with the same content hashes in both lock files: `esd` 0.1.2 (`sha256:6195444e…95f6`), `load-switch` 0.2.0 (`sha256:2306e411…bc96`) and `@contrib/audio-amp` 0.2.0 (`sha256:a1ac6087…b2af`). The three lands are unchanged in the versions that carry the bindings (`esd` 0.1.4, `load-switch` 0.2.1, `@contrib/audio-amp` 0.2.1).

## Files

| File | What it is |
|---|---|
| `exports/top.png`, `exports/bottom.png` | The preview's own 2D whole-board exports, byte for byte (JPEG data under a `.png` name), 2271 × 3819 px, 39.4 px/mm. `bottom.png` is the view from below, mirrored left to right. They are in `exports/`, as in the source record, and not beside this page: `tools/tests/test_measure_preview.py` compares every export it finds there with the green extent of its middle row and column, which describes a bare frame board and not this one, whose middle row and column cross parts and silkscreen. `tools/tests/test_owt_rev_c_record.py` checks these two against the board's bounding box instead. |
| `board-extract.json` | The board outline's bounding box and the four footprints (reference, footprint, side, position, angle, pads), cut unchanged from the package's `board.json`. |
| `measurements.json` | `measure_preview.py measure board-extract.json top.bmp bottom.bmp`, with the exports converted by `sips -s format bmp`. Its four rows equal the same four rows of the source record's `measurements.json`, which was measured with the whole `board.json`. |
| `placements.csv`, `part-numbers.csv` | The four placement rows and three part-number rows of the source record's tables, with their header lines. |
| `lcsc-bom-rows.csv`, `lcsc-cpl-rows.csv` | The three BOM rows and four CPL rows of the package as uploaded, with their header lines. The CPL rotation already includes the mapping. |
| `crops/` | The source record's crops of the four placements: the export's pixels doubled, a cyan ring on our pad 1, a yellow square on each other pad, a white cross on the footprint origin. Bottom crops are the view from below. |
| `source-SHA256SUMS`, `source-package-SHA256SUMS` | The source record's and the package's own hash lists. |

No file here names a local path.

| File | SHA-256 |
|---|---|
| `exports/top.png` | `e66cfd404f5dbf3eca799423d844adf9cce89c91cff57db088b680d9ee622464` |
| `exports/bottom.png` | `1ca0b8ee6926c7e62fc4566e176e2b74a9cb79c2131020aca2eb03de785640a0` |
| `board-extract.json` | `4768cc745fba80e512ef78e2ebed488691a36ce607aa1aeaa8df1c23013ef539` |
| `measurements.json` | `460948c853a196ca1deb86cd5f4f531d3ff70e35840ec8891e653db7f639f117` |
| `placements.csv` | `e471c6477d9c8fb3a95edd2b6b87e96646787a5c973c5975970ac819dc81e61d` |
| `part-numbers.csv` | `1d68be60af284edc9f36fd09b933f2eecf81c48159bd5ed624f4d59ee7987b74` |
| `lcsc-bom-rows.csv` | `c72ab909e458cd0aebc71ca5303ab3eb876e51988b66765aa17d2a13b34de3c4` |
| `lcsc-cpl-rows.csv` | `f6557515a0128450f6bd0d3bc0f44c39e36829ebc85b962ad3b0ad1dea57ef11` |
| `source-SHA256SUMS` | `b18b8d6399ceace2ed924f151becb868dfcc0658cc3f627bd9c0f99d2a97ce5a` |
| `source-package-SHA256SUMS` | `4fa078ca1399ac54d91a13d22ec40c548729481228e244e68506ff8d62573625` |
| `crops/U1-top.png` | `79a1606822e6ee6088fb54849c526668a7294b245527101a720f274c1bc34e35` |
| `crops/U10-bottom.png` | `78395ee4295dab1faf5670368a149851ed26ebeac0e0d3e84a6680bc825c69d0` |
| `crops/U2-bottom.png` | `647ea500687cfe03ceeff236f63bd7106604e42148f282190d961509ec7211b2` |
| `crops/U20-top.png` | `a2b7554b07fbee8907725f360b74c760dbc47cc249cd2e86f4a397f8500913f1` |

The two exports and the four crops have the hashes `source-SHA256SUMS` gives for them.

## Readings

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

## Limits

- One placement of C7519 and of C189961, two of C131941, all at ±90° on the board. Board 4 covers 0° and 37°, top and bottom, without the mapping.
- These mappings belong to the part number, not to the land. On `esd::FP_SOT_23_6` C85364 read `rotate 180` and on `load_switch::FP_TI_DBV0006A` C12044 read `rotate 270` (C2 frame board 1, `../2026-10-30-c2-board-1/`).
- Nothing else on this board is claimed here: not its chip parts, inductors, connectors or switches, and not the parts its own record marks as needing a mapping.

## Reproducing

```sh
sips -s format bmp exports/top.png --out top.bmp
sips -s format bmp exports/bottom.png --out bottom.bmp
python3 -I -B tools/frame_boards/measure_preview.py measure board-extract.json top.bmp bottom.bmp out.json
```

`out.json` equals `measurements.json` byte for byte; `tools/tests/test_owt_rev_c_record.py` runs this on macOS.

## Licence

The exports and crops are the assembler's renderings of a board published under the MIT licence, kept as the evidence the readings rest on.
