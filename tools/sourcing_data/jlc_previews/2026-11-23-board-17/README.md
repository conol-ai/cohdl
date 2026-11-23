# Board 17 SOT-23 diode and TVS array preview, 2026-11-23

JLCPCB detected a two-layer, 148 × 168 mm board. Its PCBA BOM step confirmed all 51 C numbers and matched each of the 102 top and 102 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-17 ZIP from commit `ea2b2cb`, SHA-256 `184ccbf6b02bf92742875946091542d0a5848e2644637b6ce558cff7a8dc4a94`: the 38 diodes of `diode` 0.1.4 (`tools/gen_sot23_diodes.py`, all but 1SS226, which had none in stock) and the 13 two-line TVS arrays of `esd` 0.1.5 (`tools/gen_sot23_tvs.py`), each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sot::SOT3P190X290X130N` (`sot` 0.1.2) with no mapping.

## Readings

33 of the 51 part numbers read with the assembler's pin 1 mark at our pad 1 on all four copies, with no mapping: 132 of 204 copies, each within 15.3 degrees of pad 1's direction (limit 20). They include every TVS array. `readings.json` lists every part number with its designators and the four angles, counter-clockwise from our pad 1 seen from the top.

| Part number | Manufacturer | MPN | Device | Copies | Mark vs pad 1 (deg) | Result |
|---|---|---|---|---|---|---|
| C2500 | Nexperia | BAV99,215 | `DiodePairSeries_SOT23` | D149–D152 | 14.6, 15.3, -14.3, -13.7 | Pass |
| C32677 | ProTek | PSM712-LF-T7 | `TVS2Asym_SOT23` | D201–D204 | 13.9, 14.8, -13.9, -13.8 | Pass |
| C37704 | Nexperia | BAT54C,215 | `SchottkyPairCommonCathode_SOT23` | D145–D148 | 14.8, 15.3, -14.8, -13.7 | Pass |
| C68978 | Jiangsu Changjing Electronics Technology | BAV70 | `DiodePairCommonCathode_SOT23` | D141–D144 | 15, 15.2, -14.5, -13.5 | Pass |
| C7420333 | Zhuhai Hongjiacheng Technology | BAT54S | `SchottkyPairSeries_SOT23` | D45–D48 | 7.1, 7.9, -7.3, -5.9 | Pass |
| C7420335 | Zhuhai Hongjiacheng Technology | BAV70 | `DiodePairCommonCathode_SOT23` | D57–D60 | 6.3, 7.8, -7.1, -6.5 | Pass |
| C7420336 | Zhuhai Hongjiacheng Technology | BAV99 | `DiodePairSeries_SOT23` | D61–D64 | 6.3, 7.7, -7, -6.7 | Pass |
| C7502685 | Zhuhai Hongjiacheng Technology | BAS40 | `Schottky_SOT23` | D21–D24 | -81.1, -80.1, 81.6, 81.5 | Manual review |
| C7502686 | Zhuhai Hongjiacheng Technology | BAS70 | `Schottky_SOT23` | D33–D36 | -81, -79.8, 81.4, 81.4 | Manual review |
| C7502687 | Zhuhai Hongjiacheng Technology | BAS70-04 | `SchottkyPairSeries_SOT23` | D25–D28 | -82.6, -81.7, 81, 81 | Manual review |
| C7502688 | Zhuhai Hongjiacheng Technology | BAS40-04 | `SchottkyPairSeries_SOT23` | D13–D16 | -81.5, -81.2, 80.4, 81.7 | Manual review |
| C7502689 | Zhuhai Hongjiacheng Technology | BAS40-05 | `SchottkyPairCommonCathode_SOT23` | D17–D20 | -81.7, -80.6, 80.3, 81.6 | Manual review |
| C7502690 | Zhuhai Hongjiacheng Technology | BAT54A | `SchottkyPairCommonAnode_SOT23` | D37–D40 | -81.9, -80.9, 81.3, 81.1 | Manual review |
| C7502723 | Zhuhai Hongjiacheng Technology | MMBD4148 | `Diode_SOT23` | D133–D136 | -81.8, -80.9, 80.5, 81.2 | Manual review |
| C7502725 | Zhuhai Hongjiacheng Technology | BAS21 | `Diode_SOT23` | D9–D12 | -81.1, -80.8, 80.7, 81.2 | Manual review |
| C7502726 | Zhuhai Hongjiacheng Technology | BAS21S | `DiodePairSeries_SOT23` | D5–D8 | -81, -80.4, 80.9, 80.9 | Manual review |
| C7502729 | Zhuhai Hongjiacheng Technology | BAW56 | `DiodePairCommonAnode_SOT23` | D65–D68 | -81.1, -79.9, 80.8, 81.1 | Manual review |
| C7502730 | Zhuhai Hongjiacheng Technology | 1SS181 | `DiodePairCommonAnode_SOT23` | D1–D4 | -80.9, -80.1, 80.6, 81.2 | Manual review |
| C17179589 | Zhuhai Hongjiacheng Technology | MMBD4148CC | `DiodePairCommonCathode_SOT23` | D125–D128 | 11, 11.5, -11, -10.2 | Pass |
| C17179590 | Zhuhai Hongjiacheng Technology | MMBD4148SE | `DiodePairSeries_SOT23` | D129–D132 | 11.2, 11.5, -11.2, -10.2 | Pass |
| C19077467 | Zhuhai Hongjiacheng Technology | BZX84C3V3 | `Zener_SOT23` | D97–D100 | -163.5, -163.8, 163.6, 162.9 | Manual review |
| C19077468 | Zhuhai Hongjiacheng Technology | BZX84C5V1 | `Zener_SOT23` | D105–D108 | -163.4, -163.8, 163.8, 163 | Manual review |
| C19077469 | Zhuhai Hongjiacheng Technology | BZX84C5V6 | `Zener_SOT23` | D109–D112 | -163.6, -164.2, 163.3, 162.9 | Manual review |
| C19077470 | Zhuhai Hongjiacheng Technology | BZX84C10 | `Zener_SOT23` | D69–D72 | -163.8, -163.8, 163.3, 162.7 | Manual review |
| C19077471 | Zhuhai Hongjiacheng Technology | BZX84C12 | `Zener_SOT23` | D73–D76 | -162.6, -163.7, 163.6, 163.4 | Manual review |
| C19077472 | Zhuhai Hongjiacheng Technology | BZX84C15 | `Zener_SOT23` | D77–D80 | -163.2, -164, 163.7, 163.2 | Manual review |
| C19077473 | Zhuhai Hongjiacheng Technology | BZX84C18 | `Zener_SOT23` | D85–D88 | -163.1, -163.9, 163.5, 162.8 | Manual review |
| C20615823 | Zhuhai Hongjiacheng Technology | H5VHT2U | `TVS2Uni_SOT23` | D185–D188 | 5.8, 6, -5.4, -4.2 | Pass |
| C20615824 | Zhuhai Hongjiacheng Technology | H5VUT2U | `TVS2Steering_SOT23` | D193–D196 | 4, 5.3, -5.3, -5 | Pass |
| C20615825 | Zhuhai Hongjiacheng Technology | H8VNT2B | `TVS2Bi_SOT23` | D197–D200 | 4, 5.2, -5.2, -4.8 | Pass |
| C20615826 | Zhuhai Hongjiacheng Technology | H12VNT2U | `TVS2Uni_SOT23` | D157–D160 | 5.2, 6.1, -4.9, -4.3 | Pass |
| C20615827 | Zhuhai Hongjiacheng Technology | H24VNT2B | `TVS2Bi_SOT23` | D165–D168 | 4.8, 5.9, -5, -4.3 | Pass |
| C20615828 | Zhuhai Hongjiacheng Technology | H36VLT2B | `TVS2Bi_SOT23` | D169–D172 | 4.7, 6.2, -4.9, -4.5 | Pass |
| C20617923 | Zhuhai Hongjiacheng Technology | H3V3HT2B | `TVS2Bi_SOT23` | D173–D176 | 6.5, 7.2, -6.8, -6.2 | Pass |
| C20617924 | Zhuhai Hongjiacheng Technology | H3V3HT2U | `TVS2Uni_SOT23` | D177–D180 | 7.2, 7.3, -6.8, -5.9 | Pass |
| C20617925 | Zhuhai Hongjiacheng Technology | H5VNT2B | `TVS2Bi_SOT23` | D189–D192 | 6.9, 7.3, -6.8, -5.6 | Pass |
| C20617926 | Zhuhai Hongjiacheng Technology | H5VHT2B | `TVS2Bi_SOT23` | D181–D184 | 6.2, 7.7, -7.3, -6.6 | Pass |
| C20617927 | Zhuhai Hongjiacheng Technology | H12VNT2B | `TVS2Bi_SOT23` | D153–D156 | 6.5, 7.5, -7.1, -6.4 | Pass |
| C20617928 | Zhuhai Hongjiacheng Technology | H15VNT2B | `TVS2Bi_SOT23` | D161–D164 | 10.3, 11.2, -10.7, -10.2 | Pass |
| C22379470 | Zhuhai Hongjiacheng Technology | BZX84C7V5 | `Zener_SOT23` | D117–D120 | 10.3, 11.3, -10.7, -10.1 | Pass |
| C22379471 | Zhuhai Hongjiacheng Technology | BZX84C16 | `Zener_SOT23` | D81–D84 | 10.5, 10.9, -10.3, -9.8 | Pass |
| C22379472 | Zhuhai Hongjiacheng Technology | BZX84C30 | `Zener_SOT23` | D89–D92 | 14, 14.4, -13.4, -13.5 | Pass |
| C22379473 | Zhuhai Hongjiacheng Technology | BZX84C6V8 | `Zener_SOT23` | D113–D116 | 13.8, 14.5, -13.6, -12.9 | Pass |
| C22379474 | Zhuhai Hongjiacheng Technology | BZX84C33 | `Zener_SOT23` | D93–D96 | 13.8, 13.9, -13.9, -12.9 | Pass |
| C22395521 | Zhuhai Hongjiacheng Technology | BZX84C4V7 | `Zener_SOT23` | D101–D104 | 13.4, 14.1, -14.1, -13.3 | Pass |
| C22395522 | Zhuhai Hongjiacheng Technology | BZX84C8V2 | `Zener_SOT23` | D121–D124 | 13.1, 14.3, -13.9, -13.7 | Pass |
| C22395543 | Zhuhai Hongjiacheng Technology | MMBZ5226B | `Zener_SOT23` | D137–D140 | 13.4, 14.1, -14.2, -13.4 | Pass |
| C22466350 | Zhuhai Hongjiacheng Technology | BAT54C | `SchottkyPairCommonCathode_SOT23` | D41–D44 | -13.6, -11.8, 12.2, 13.3 | Pass |
| C22466364 | Zhuhai Hongjiacheng Technology | BAS70-06 | `SchottkyPairCommonAnode_SOT23` | D29–D32 | 0.5, 2.2, -1.8, -0.4 | Pass |
| C22466368 | Zhuhai Hongjiacheng Technology | BAT54 | `Schottky_SOT23` | D49–D52 | 0.8, 2.4, -1.6, 0 | Pass |
| C22466371 | Zhuhai Hongjiacheng Technology | BAT720 | `Schottky_SOT23` | D53–D56 | 1.1, 2.5, -1.9, -0.2 | Pass |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` `21e863da…`, `bottom.png` `5adf027c…`, `top.pixels.png` `5e0a3c79…`, `bottom.pixels.png` `1351de64…`). `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-17
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_17-lcsc-cpl.csv" > /tmp/board17-checks.json
cmp "$D/checks.json" /tmp/board17-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board17-measurements.json
cmp "$D/measurements.json" /tmp/board17-measurements.json
```

## Manual review and limits

All 18 part numbers left for manual review are Zhuhai Hongjiacheng parts, every copy drawn, in two groups:

- 11 drawn with the model of boards 15 and 16 (`../2026-11-23-board-15/`): body centred on the land but a quarter turn against it, pin 1 mark about 81 degrees clockwise of our pad 1 seen from the top on the top side, and the mirror of that on the bottom. Board 15's candidate applies: `rotate 90`. C7502685 (BAS40), C7502686 (BAS70), C7502687 (BAS70-04), C7502688 (BAS40-04), C7502689 (BAS40-05), C7502690 (BAT54A), C7502723 (MMBD4148), C7502725 (BAS21), C7502726 (BAS21S), C7502729 (BAW56), C7502730 (1SS181).
- 7 BZX84C Zeners drawn with a SOT-23 model upright but a half turn against the land: its two leads on our pad 3 side, its pin 1 mark at the upper left, about 164 degrees from our pad 1. Turned 180 degrees about the origin, each lead falls on our pad of the same number; the candidate is `rotate 180`. C19077467 (BZX84C3V3), C19077468 (BZX84C5V1), C19077469 (BZX84C5V6), C19077470 (BZX84C10), C19077471 (BZX84C12), C19077472 (BZX84C15), C19077473 (BZX84C18).

Neither candidate is a reading: each is checked on a later preview before any binding.

## Bindings

None yet. The 33 part numbers that passed are bound after a backfill of their map rows (a fetch that needs its own approval), with no mapping.
