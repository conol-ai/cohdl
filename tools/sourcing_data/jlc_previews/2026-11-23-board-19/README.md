# Board 19 Hongjiacheng SOT-23 diode re-verify preview, 2026-11-23

JLCPCB detected a two-layer, 68 × 168 mm board. Its PCBA BOM step confirmed all 18 C numbers and matched each of the 36 top and 36 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-19 ZIP from commit `c6651a5`, SHA-256 `86a460e14857ba98d2e9b6f11ef51bb25ada939425a9e4eb309e2069a21b46b7`: the 18 Zhuhai Hongjiacheng part numbers board 17 (`../2026-11-23-board-17/`) left for manual review, each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sot::SOT3P190X290X130N` with its candidate as the row's mapping: `rotate 90` for the 11 drawn a quarter turn, `rotate 180` for the 7 BZX84C Zeners drawn a half turn.

## Readings

All 18 part numbers read with the model's pin 1 mark at our pad 1 on all four copies: both candidates are confirmed. The marks read -17.3 to 17.7 degrees from pad 1's direction (limit 20). The archived pixels show each model's leads on our pads, its pin 1 lead on our pad 1. `readings.json` lists each part number with its designators, mapping and angles.

| Part number | MPN | Device | Rotate | Copies | Mark vs pad 1 (deg) | Result |
|---|---|---|---:|---|---|---|
| C7502685 | BAS40 | `Schottky_SOT23` | 90 | D21–D24 | 10.6, 11.5, -11.1, -10.8 | Pass |
| C7502686 | BAS70 | `Schottky_SOT23` | 90 | D29–D32 | 10.4, 11.3, -11, -10.7 | Pass |
| C7502687 | BAS70-04 | `SchottkyPairSeries_SOT23` | 90 | D25–D28 | 10.7, 11.4, -10.9, -10.2 | Pass |
| C7502688 | BAS40-04 | `SchottkyPairSeries_SOT23` | 90 | D13–D16 | 10.8, 11.7, -10.6, -10 | Pass |
| C7502689 | BAS40-05 | `SchottkyPairCommonCathode_SOT23` | 90 | D17–D20 | 11, 11.2, -10.5, -9.7 | Pass |
| C7502690 | BAT54A | `SchottkyPairCommonAnode_SOT23` | 90 | D33–D36 | 11.1, 11.1, -10.3, -9.5 | Pass |
| C7502723 | MMBD4148 | `Diode_SOT23` | 90 | D69–D72 | 11.3, 11.5, -10.3, -9.3 | Pass |
| C7502725 | BAS21 | `Diode_SOT23` | 90 | D9–D12 | 11.3, 11.4, -10.1, -9.1 | Pass |
| C7502726 | BAS21S | `DiodePairSeries_SOT23` | 90 | D5–D8 | 9.9, 10.7, -10.4, -10.5 | Pass |
| C7502729 | BAW56 | `DiodePairCommonAnode_SOT23` | 90 | D37–D40 | 9.8, 10.8, -10.4, -10.2 | Pass |
| C7502730 | 1SS181 | `DiodePairCommonAnode_SOT23` | 90 | D1–D4 | 10.2, 10.6, -10.2, -9.9 | Pass |
| C19077467 | BZX84C3V3 | `Zener_SOT23` | 180 | D57–D60 | 17.4, 17.5, -17.1, -16.8 | Pass |
| C19077468 | BZX84C5V1 | `Zener_SOT23` | 180 | D61–D64 | 17.4, 17.5, -17, -16.8 | Pass |
| C19077469 | BZX84C5V6 | `Zener_SOT23` | 180 | D65–D68 | 17.5, 17.5, -16.7, -16.3 | Pass |
| C19077470 | BZX84C10 | `Zener_SOT23` | 180 | D41–D44 | 17.7, 17.3, -16.9, -16.1 | Pass |
| C19077471 | BZX84C12 | `Zener_SOT23` | 180 | D45–D48 | 17.7, 17.5, -16.5, -16 | Pass |
| C19077472 | BZX84C15 | `Zener_SOT23` | 180 | D49–D52 | 16.4, 17, -16.8, -17.3 | Pass |
| C19077473 | BZX84C18 | `Zener_SOT23` | 180 | D53–D56 | 16.3, 17, -16.6, -16.9 | Pass |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` `4e2f9e98…`, `bottom.png` `c0ebb50b…`, `top.pixels.png` `01c66a46…`, `bottom.pixels.png` `63515407…`). `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-19
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_19-lcsc-cpl.csv" > /tmp/board19-checks.json
cmp "$D/checks.json" /tmp/board19-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board19-measurements.json
cmp "$D/measurements.json" /tmp/board19-measurements.json
```

## Bindings

None yet. These 18 are bound with their rotations after a backfill of their map rows.
