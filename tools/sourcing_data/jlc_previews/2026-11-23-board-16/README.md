# Board 16 SOT-23 MOSFET preview, 2026-11-23

JLCPCB detected a two-layer, 88 × 168 mm board. Its PCBA BOM step confirmed all 29 C numbers and matched each of the 58 top and 58 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-16 ZIP from commit `719c0a0`, SHA-256 `0363218b0f0a6e326b0ce6e6f944a3bfd15edffd640045886e36c7bc719a73d2`: the 29 MOSFETs of `mosfet` 0.1.1 (`tools/gen_sot23.py`), each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sot::SOT3P190X290X130N` (`sot` 0.1.2) with no mapping. The first run of this package timed out waiting for the Gerber upload; this record is the second.

## Readings

27 of the 29 part numbers read with the assembler's pin 1 mark at our pad 1 on all four copies, with no mapping: 108 of 116 copies, each within 16.8 degrees of pad 1's direction (limit 20). `readings.json` lists every part number with its designators and the four angles, counter-clockwise from our pad 1 seen from the top.

| Part number | Manufacturer | MPN | Copies | Mark vs pad 1 (deg) | Result |
|---|---|---|---|---|---|
| C8545 | Jiangsu Changjing Electronics Technology | 2N7002 | Q109–Q112 | 6.2, 7.3, -7.3, -6.9 | Pass |
| C10487 | Vishay | SI2301CDS-T1-GE3 | Q113–Q116 | 13.8, 14.3, -14.3, -14 | Pass |
| C7420321 | Zhuhai Hongjiacheng Technology | 2N7002 | Q5–Q8 | 7.1, 7.7, -7.4, -6.9 | Pass |
| C7420338 | Zhuhai Hongjiacheng Technology | BSS123 | Q13–Q16 | 6.9, 7.6, -7.3, -6.5 | Pass |
| C7420339 | Zhuhai Hongjiacheng Technology | BSS138 | Q17–Q20 | 7.1, 7.7, -7.1, -6.4 | Pass |
| C7420340 | Zhuhai Hongjiacheng Technology | BSS84 | Q21–Q24 | 6.1, 6.9, -6.4, -6.8 | Pass |
| C7420343 | Zhuhai Hongjiacheng Technology | HL2300 | Q29–Q32 | 7.1, 7.5, -6.8, -6.2 | Pass |
| C7420344 | Zhuhai Hongjiacheng Technology | HL2301A | Q33–Q36 | 6.5, 6.8, -6.6, -6.3 | Pass |
| C7420345 | Zhuhai Hongjiacheng Technology | HL2303 | Q41–Q44 | 6.5, 6.7, -6.6, -6.1 | Pass |
| C7420346 | Zhuhai Hongjiacheng Technology | HL2304 | Q45–Q48 | 7.4, 7.3, -6.8, -5.8 | Pass |
| C7420347 | Zhuhai Hongjiacheng Technology | HL2310A | Q65–Q68 | 7.6, 7.7, -6.8, -5.8 | Pass |
| C7420348 | Zhuhai Hongjiacheng Technology | HL3400A | Q73–Q76 | 7.6, 7.6, -6.3, -5.4 | Pass |
| C7420349 | Zhuhai Hongjiacheng Technology | HL3401A | Q81–Q84 | 6.5, 6.7, -6.1, -5.8 | Pass |
| C20069148 | Zhuhai Hongjiacheng Technology | 2SK3018 | Q9–Q12 | 10.8, 12.1, -11.7, -11.2 | Pass |
| C20069151 | Zhuhai Hongjiacheng Technology | FDV301N | Q25–Q28 | 16.2, 16.8, -16.5, -16 | Pass |
| C22379475 | Zhuhai Hongjiacheng Technology | 2N7002K | Q1–Q4 | 10.9, 12.2, -11.6, -10.5 | Pass |
| C22469448 | Zhuhai Hongjiacheng Technology | HL3400 | Q77–Q80 | 7.5, 8, -7.5, -6.5 | Pass |
| C22469449 | Zhuhai Hongjiacheng Technology | HL3401 | Q85–Q88 | 13.8, 14.4, -14, -13.6 | Pass |
| C28646261 | Zhuhai Hongjiacheng Technology | HL2305 | Q49–Q52 | 6.7, 7.3, -6.8, -6.3 | Pass |
| C28646262 | Zhuhai Hongjiacheng Technology | HL2306 | Q53–Q56 | 7.7, 8.2, -7, -6.3 | Pass |
| C28646263 | Zhuhai Hongjiacheng Technology | HL2309 | Q61–Q64 | 2.9, 3.3, -2.3, -1.3 | Pass |
| C28646264 | Zhuhai Hongjiacheng Technology | HL6042 | Q105–Q108 | 1.6, 1.4, -0.5, 0.3 | Pass |
| C28646268 | Zhuhai Hongjiacheng Technology | HL2302 | Q37–Q40 | -80.6, -79.2, 81.1, 81.4 | Manual review |
| C28646269 | Zhuhai Hongjiacheng Technology | HL3407A | Q93–Q96 | -79.7, -79, 80.5, 80.7 | Manual review |
| C28646270 | Zhuhai Hongjiacheng Technology | HL3415A | Q97–Q100 | 3.6, 3.2, -1.8, -0.8 | Pass |
| C28646271 | Zhuhai Hongjiacheng Technology | HL3402 | Q89–Q92 | 8, 8, -6.9, -5.9 | Pass |
| C28646272 | Zhuhai Hongjiacheng Technology | HL2312 | Q69–Q72 | 4.4, 4.3, -2.9, -0.8 | Pass |
| C28646273 | Zhuhai Hongjiacheng Technology | HL3416 | Q101–Q104 | -2.7, -1.7, 1.6, 2 | Pass |
| C28646274 | Zhuhai Hongjiacheng Technology | HL2307 | Q57–Q60 | 13.5, 14.2, -14, -13.9 | Pass |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` `b2efe1e3…`, `bottom.png` `5ee43012…`, `top.pixels.png` `43f23031…`, `bottom.pixels.png` `b61ee5b8…`). `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-16
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_16-lcsc-cpl.csv" > /tmp/board16-checks.json
cmp "$D/checks.json" /tmp/board16-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board16-measurements.json
cmp "$D/measurements.json" /tmp/board16-measurements.json
```

## Manual review and limits

C28646268 (Zhuhai Hongjiacheng HL2302, Q37–Q40) and C28646269 (Zhuhai Hongjiacheng HL3407A, Q93–Q96) are drawn on every copy, but their model does not sit on our land the way the others do. The archived pixels show the same model as board 15's C20069147, also Hongjiacheng: its body is centred on our origin but turned a quarter turn, its leads above and below the land, and its pin 1 mark is about 2 mm from our pad 1 (near −80 degrees on the top, 81 on the bottom). Neither is read as passed or as a mapping. Board 15's record gives the candidate for all three part numbers, `rotate 90`, to be checked on a later preview.

## Bindings

None yet. As on board 15, the 27 part numbers that passed are bound after a backfill of their map rows, which fetches their pages and waits for approval. They are bound with no mapping.
