# Board 15 SOT-23 bipolar transistor preview, 2026-11-23

JLCPCB detected a two-layer, 128 × 168 mm board. Its PCBA BOM step confirmed all 45 C numbers and matched each of the 90 top and 90 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-15 ZIP from commit `719c0a0`, SHA-256 `ccbf27fdc0501b804170c8964fd887bc29ffaa313b2ea1de870437e2c0171ac2`: the 45 bipolar transistors of `transistor` 0.2.2 (`tools/gen_sot23.py`), each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sot::SOT3P190X290X130N` (`sot` 0.1.2) with no mapping. The first run stopped before the PCBA order page and the second when its bottom export reached Downloads late (Arc held it for the multiple-downloads permission); this record is the third run of the same package.

## Readings

44 of the 45 part numbers read with the assembler's pin 1 mark at our pad 1 on all four copies, with no mapping: 176 of 180 copies, each within 15 degrees of pad 1's direction (limit 20). `readings.json` lists every part number with its designators and the four angles, counter-clockwise from our pad 1 seen from the top.

| Part number | Manufacturer | MPN | Copies | Mark vs pad 1 (deg) | Result |
|---|---|---|---|---|---|
| C2145 | Jiangsu Changjing Electronics Technology | MMBT5551(RANGE:200-300) | Q153–Q156 | 6.9, 7.7, -6.9, -6.6 | Pass |
| C2149 | Jiangsu Changjing Electronics Technology | S9015(RANGE:300-400) | Q169–Q172 | 6.5, 7.2, -6.8, -6.1 | Pass |
| C2150 | Jiangsu Changjing Electronics Technology | SS8050(RANGE:200-350) | Q173–Q176 | 7.1, 7.9, -7, -5.9 | Pass |
| C6749 | Jiangsu Changjing Electronics Technology | S9013 J3(RANGE:200-350) | Q165–Q168 | 6.3, 7.7, -7, -6.5 | Pass |
| C8326 | Jiangsu Changjing Electronics Technology | MMBT5401(RANGE:200-300) | Q149–Q152 | 6.6, 7.2, -6.6, -6.6 | Pass |
| C8512 | Jiangsu Changjing Electronics Technology | MMBT2222A 1P | Q141–Q144 | 6.4, 7.4, -7.3, -6.4 | Pass |
| C8542 | Jiangsu Changjing Electronics Technology | SS8550 Y2(RANGE:200-350) | Q177–Q180 | 6.5, 7.1, -6.6, -6.2 | Pass |
| C8543 | Jiangsu Changjing Electronics Technology | S9012 2T1(RANGE:200-350) | Q161–Q164 | 6.6, 7.4, -6.3, -5.9 | Pass |
| C20526 | Jiangsu Changjing Electronics Technology | MMBT3904(RANGE:100-300) | Q145–Q148 | 7, 7.7, -7, -6.1 | Pass |
| C105432 | Jiangsu Changjing Electronics Technology | S8550(RANGE:120-200) | Q157–Q160 | 14.2, 15, -14.6, -13.4 | Pass |
| C7420323 | Zhuhai Hongjiacheng Technology | 2SA812 | Q1–Q4 | 7, 7.9, -6.6, -6.5 | Pass |
| C7420324 | Zhuhai Hongjiacheng Technology | 2SC1623 | Q5–Q8 | 6.6, 7.5, -7.1, -6.4 | Pass |
| C7420337 | Zhuhai Hongjiacheng Technology | BC847C | Q41–Q44 | 6.6, 7.4, -7.2, -6.1 | Pass |
| C7420351 | Zhuhai Hongjiacheng Technology | MMBT2222A | Q57–Q60 | 6.9, 7.8, -6.6, -6 | Pass |
| C7420352 | Zhuhai Hongjiacheng Technology | MMBT2907A | Q61–Q64 | 6.9, 7.5, -6.7, -5.9 | Pass |
| C7420353 | Zhuhai Hongjiacheng Technology | MMBT3904 | Q65–Q68 | 6.8, 7.4, -6.6, -6.2 | Pass |
| C7420354 | Zhuhai Hongjiacheng Technology | MMBT3906 | Q69–Q72 | 7, 7.8, -7, -5.9 | Pass |
| C7420355 | Zhuhai Hongjiacheng Technology | MMBT4401 | Q73–Q76 | 7.1, 7.5, -6.7, -5.7 | Pass |
| C7420356 | Zhuhai Hongjiacheng Technology | MMBT5401 | Q81–Q84 | 6, 7.2, -6.7, -6.3 | Pass |
| C7420357 | Zhuhai Hongjiacheng Technology | MMBT5551 | Q85–Q88 | 7.3, 7.6, -6.5, -5.8 | Pass |
| C7420370 | Zhuhai Hongjiacheng Technology | SS8050 | Q133–Q136 | 6.1, 7.4, -7.1, -6.6 | Pass |
| C7420371 | Zhuhai Hongjiacheng Technology | SS8550 | Q137–Q140 | 6.1, 7.5, -6.6, -6.4 | Pass |
| C20069125 | Zhuhai Hongjiacheng Technology | S8050 | Q109–Q112 | 1.7, 3.6, -3.2, -2.4 | Pass |
| C20069126 | Zhuhai Hongjiacheng Technology | S8550 | Q113–Q116 | 6.7, 7.4, -7.4, -6.5 | Pass |
| C20069127 | Zhuhai Hongjiacheng Technology | S9013 | Q121–Q124 | 10.6, 11.8, -11.3, -10.4 | Pass |
| C20069129 | Zhuhai Hongjiacheng Technology | S9012 | Q117–Q120 | 6.8, 7.4, -7.1, -6.1 | Pass |
| C20069130 | Zhuhai Hongjiacheng Technology | S9014 | Q125–Q128 | 6.9, 8, -7.3, -6.1 | Pass |
| C20069131 | Zhuhai Hongjiacheng Technology | S9015 | Q129–Q132 | 11, 11.5, -10.4, -10.3 | Pass |
| C20069132 | Zhuhai Hongjiacheng Technology | BC817-16 | Q21–Q24 | 6.9, 7.8, -7.1, -6.4 | Pass |
| C20069134 | Zhuhai Hongjiacheng Technology | BC846B | Q33–Q36 | 7.1, 7.8, -7.5, -6.2 | Pass |
| C20069135 | Zhuhai Hongjiacheng Technology | BC847B | Q37–Q40 | 7.2, 7.9, -7.5, -5.9 | Pass |
| C20069136 | Zhuhai Hongjiacheng Technology | BC856B | Q45–Q48 | -13, -11.9, 13.3, 13.4 | Pass |
| C20069137 | Zhuhai Hongjiacheng Technology | BC857C | Q49–Q52 | 7.1, 7.5, -7.1, -5.6 | Pass |
| C20069138 | Zhuhai Hongjiacheng Technology | MMBTA42 | Q93–Q96 | 7.3, 7.8, -7.2, -6.1 | Pass |
| C20069139 | Zhuhai Hongjiacheng Technology | MMBTA06 | Q89–Q92 | 1.8, 1.7, -0.7, 0.4 | Pass |
| C20069140 | Zhuhai Hongjiacheng Technology | MMBT4403 | Q77–Q80 | 10, 11.3, -10.9, -10.3 | Pass |
| C20069141 | Zhuhai Hongjiacheng Technology | MMBTA44 | Q97–Q100 | 6, 7.9, -7.3, -6.5 | Pass |
| C20069142 | Zhuhai Hongjiacheng Technology | BC807-40 | Q17–Q20 | 10, 11.4, -10.8, -10.1 | Pass |
| C20069143 | Zhuhai Hongjiacheng Technology | 2SC1815 | Q9–Q12 | 6.2, 7.4, -7.3, -6.6 | Pass |
| C20069146 | Zhuhai Hongjiacheng Technology | MMBTA92 | Q105–Q108 | 6.6, 7.6, -6.9, -6.4 | Pass |
| C20069147 | Zhuhai Hongjiacheng Technology | MMBTA56 | Q101–Q104 | -81.5, -80.9, 80.5, 80.5 | Manual review |
| C20069154 | Zhuhai Hongjiacheng Technology | BC817-25 | Q25–Q28 | 10.6, 11.6, -10.6, -10.1 | Pass |
| C20069155 | Zhuhai Hongjiacheng Technology | BC817-40 | Q29–Q32 | 2.3, 3.4, -2.8, -1.8 | Pass |
| C28646266 | Zhuhai Hongjiacheng Technology | 2SC3356 | Q13–Q16 | 7.2, 7.3, -7.1, -5.7 | Pass |
| C28646275 | Zhuhai Hongjiacheng Technology | FMMT493 | Q53–Q56 | 6, 7.4, -7.1, -6.5 | Pass |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` `a2643d31…`, `bottom.png` `7974a88f…`, `top.pixels.png` `142ff12f…`, `bottom.pixels.png` `2ef7b62f…`). `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-15
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_15-lcsc-cpl.csv" > /tmp/board15-checks.json
cmp "$D/checks.json" /tmp/board15-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board15-measurements.json
cmp "$D/measurements.json" /tmp/board15-measurements.json
```

## Manual review and limits

C20069147 (Zhuhai Hongjiacheng MMBTA56, Q101–Q104) is drawn on every copy, but its model does not sit on our land the way the others do: the archived pixels show its leads turned a quarter turn against our pads and its body off our origin, with its pin 1 mark about 2 mm from our pad 1 (−81.5 and −80.9 degrees on the top, 80.5 on the bottom). It is not read as passed or as a mapping: it needs a mapping candidate of its own, checked on a later preview.

## Bindings

None yet. A polarized part is bound with its own reading and a backfill of its map row (the confirmed C number and the page's identity), as `diode` 0.1.3 did for board 10. That backfill fetches the 44 part numbers' pages and waits for approval; the bindings follow it, with no mapping.
