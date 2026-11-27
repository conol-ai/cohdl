# Board 18 Hongjiacheng SOT-23 re-verify preview, 2026-11-23

JLCPCB detected a two-layer, 28 × 68 mm board. Its PCBA BOM step confirmed all 3 C numbers and matched each of the 6 top and 6 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-18 ZIP from commit `e2c9af3`, SHA-256 `d7c7c3b37d7888a29ddca51b41c4dc8a2df0214cc67d8572700d56de6dd7ac83`: the three Zhuhai Hongjiacheng part numbers boards 15 and 16 (`../2026-11-23-board-15/`, `../2026-11-23-board-16/`) left for manual review, each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sot::SOT3P190X290X130N` with the candidate `rotate 90` as the row's mapping. The supplier CPL therefore reads 90 and 127 on the top and, under the `lcsc=mirrored` bottom convention, 90 and 53 on the bottom. The first run stopped at the BOM step (the page did not show the match in time; exit 2); this record is the second, with the same package.

## Readings

All three part numbers read with the model's pin 1 mark at our pad 1 on all four copies: the candidate is confirmed. The marks read -10 to 10.5 degrees from pad 1's direction (limit 20), the same side of pad 1 as the part numbers boards 15 and 16 passed with no mapping. The archived pixels show each model's leads on our pads, its pin 1 lead on our pad 1. `readings.json` lists each part number with its designators, mapping and angles.

| Part number | MPN | Device | Copies | Mark vs pad 1 (deg) | Result |
|---|---|---|---|---|---|
| C20069147 | MMBTA56 | `transistor::PNP_SOT23` | Q9–Q12 | 10.5, 10.5, -9.9, -9.5 | Pass |
| C28646268 | HL2302 | `mosfet::NMOS_SOT23` | Q1–Q4 | 10, 10.5, -9.9, -9.8 | Pass |
| C28646269 | HL3407A | `mosfet::PMOS_SOT23` | Q5–Q8 | 10.3, 10.5, -10, -9.6 | Pass |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` `d27a791f…`, `bottom.png` `aa12aa66…`, `top.pixels.png` `2466a3fc…`, `bottom.pixels.png` `ce338015…`). `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-18
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_18-lcsc-cpl.csv" > /tmp/board18-checks.json
cmp "$D/checks.json" /tmp/board18-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board18-measurements.json
cmp "$D/measurements.json" /tmp/board18-measurements.json
```

## Bindings

None yet. These three are bound with `rotate 90` after the backfill of their map rows, with the 71 part numbers boards 15 and 16 passed with no mapping.
