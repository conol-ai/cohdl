# Board 8 Hongjiacheng SOD-123 Zener preview, 2026-11-21

JLCPCB detected a two-layer, 168 × 168 mm board. Its PCBA BOM step confirmed all 64 C numbers and matched each designator exactly once on its side (256 placements), all rows selected, with no stock warning. The run ended at Component Placements. Its input was the preview-board-8 ZIP from commit `56da519`, SHA-256 `4d0140cb40943fd69fc2ec545ecebe748b4727227cd26f419037f7e440354c1e`, the series BZT52C, BZT52B and MM1W, each part placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sod::SOD3716X135N` with no mapping.

## Readings

58 part numbers read with the cathode at our pad 1 on all four copies; 6 read reversed on all four copies; none was left undrawn. `readings.json` lists each part number with its designators and reading.

The preview tool cannot judge these: a Zener is polarized, and each model carries two signs of its own ("2 marks: which is pin 1 is read by hand"), so `checks.json` sends every drawn copy to manual review. The reading is the model's cathode band and its "−" sign against our pad 1, the land's band end, beside our silkscreen bar. My pixel classifier (which magenta mark is the smaller '-', and which end of the body holds the white band) read 224 copies as cathode at pad 1 and none reversed; the other 32 (8 part numbers whose model draws its signs over the pads) were read by eye, and every copy was then read by eye on the sheets. Both readings agree.

- **Reversed on all four copies** (candidate `rotate 180`): C22395529 (MM1W18), C22395530 (MM1W22), C22395531 (MM1W24), C22395532 (MM1W30), C22395541 (MM1W75), C22395533 (MM1W82). Each uses the assembler's larger SOD-123 model, with "+" over our pad 1 and the band and "−" at the far end. The same larger model appears on some part numbers that read correctly, so the half turn belongs to the part number's model, not to the series. The reversed part numbers fall in consecutive C-number runs (C22395529–C22395561, C22399894–C22399900).
- **Not drawn:** none.

A reversed reading is a candidate, not a mapping: it becomes a binding only after a re-verify board places the part with `rotate 180` applied and shows the band at our pad 1, as board 5 did for three ICs.

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived; `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-21-board-8
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_8-lcsc-cpl.csv" > /tmp/board8-checks.json
cmp "$D/checks.json" /tmp/board8-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board8-measurements.json
cmp "$D/measurements.json" /tmp/board8-measurements.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py "$D" /tmp/board8-sheet 64
```

The last command writes the contact sheets the readings were made from (64 per sheet, four copies of a part number side by side).

## Limits

These readings establish polarity on the assembler's model for each part number in this preview, not the part's identity. The BOM stock result describes this session. No binding or frame row is written here; bindings need these readings and map rows from a reviewed backfill.
