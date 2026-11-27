# Board 10 Hongjiacheng SOD-123 Zener re-verify preview, 2026-11-21

JLCPCB detected a two-layer, 88 × 168 mm board. Its PCBA BOM step confirmed all 25 C numbers and matched each designator exactly once on its side (50 top and 50 bottom placements), all rows selected, with no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-10 ZIP from commit `562cd13`, SHA-256 `10ed36f93c1c89e900796d17fd2664e825ac590785c9da9c278b4f88495e9d6a`: the 25 part numbers that boards 8 and 9 (`../2026-11-21-board-8/`, `../2026-11-21-board-9/`) read reversed on all four copies, each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sod::SOD3716X135N` with the candidate `rotate 180` as the row's mapping. The supplier CPL therefore reads 180 and 217 on the top and, under the `lcsc=mirrored` bottom convention, 180 and 143 on the bottom. The first attempt stopped before the Gerber upload (the quote page did not load; exit 2); the second, with the same package, is this record.

## Readings

All 25 part numbers read with the cathode at our pad 1 on all four copies: the candidate is confirmed. `readings.json` lists each part number with its designators.

As on boards 8 and 9, the preview tool cannot judge these: each model carries two signs of its own, so `checks.json` sends every copy to manual review ("unverified: none": every model is drawn). Every copy was read by eye on the contact sheets: the model's cathode band and its "−" sign sit at our pad 1, beside our silkscreen bar, and the "+" at pad 2. A pixel check agrees: on every copy `measurements.json` finds exactly two marks of the part's own, and the smaller one (the "−", at most 0.66 of the "+" in pixels) is the one toward our pad 1.

| Part number | MPN | Copies | Result |
|---|---|---|---|
| C22395529 | MM1W18 | D1–D4 | Pass, read by hand |
| C22395530 | MM1W22 | D5–D8 | Pass, read by hand |
| C22395531 | MM1W24 | D9–D12 | Pass, read by hand |
| C22395532 | MM1W30 | D13–D16 | Pass, read by hand |
| C22395533 | MM1W82 | D21–D24 | Pass, read by hand |
| C22395536 | MM1Z5V6B | D29–D32 | Pass, read by hand |
| C22395538 | MM1Z5V1 | D25–D28 | Pass, read by hand |
| C22395539 | MM1Z5V6 | D33–D36 | Pass, read by hand |
| C22395540 | MM1Z62 | D37–D40 | Pass, read by hand |
| C22395541 | MM1W75 | D17–D20 | Pass, read by hand |
| C22395547 | MMSZ4688 | D41–D44 | Pass, read by hand |
| C22395553 | MMSZ5227B | D49–D52 | Pass, read by hand |
| C22395554 | MMSZ5230B | D61–D64 | Pass, read by hand |
| C22395555 | MMSZ5234B | D65–D68 | Pass, read by hand |
| C22395556 | MMSZ5236B | D69–D72 | Pass, read by hand |
| C22395558 | MMSZ5239B | D77–D80 | Pass, read by hand |
| C22395559 | MMSZ5241B | D81–D84 | Pass, read by hand |
| C22395560 | MMSZ5251B | D85–D88 | Pass, read by hand |
| C22395561 | MMSZ5254B | D89–D92 | Pass, read by hand |
| C22399894 | MMSZ5225B | D45–D48 | Pass, read by hand |
| C22399895 | MMSZ5228B | D53–D56 | Pass, read by hand |
| C22399896 | MMSZ5229B | D57–D60 | Pass, read by hand |
| C22399897 | MMSZ5238B | D73–D76 | Pass, read by hand |
| C22399899 | MMSZ5258B | D93–D96 | Pass, read by hand |
| C22399900 | MMSZ5259B | D97–D100 | Pass, read by hand |

**Body readings** (`body.json`, with `../2026-11-21-board-8/body.py`): on every copy the drawn body's long axis lies within 1.1 degrees of the pad 1 to pad 2 axis (mod 180) and its centre within 0.129 mm of the footprint origin (limit 0.25 mm). The body lengths vary between copies because the lighter cathode band is not counted as body; the axis and centre are what the rows use.

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.pixels.png` SHA-256 `c5a74e2d…`, `bottom.pixels.png` `d48b1a66…`); `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-21-board-10
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_10-lcsc-cpl.csv" > /tmp/board10-checks.json
cmp "$D/checks.json" /tmp/board10-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board10-measurements.json
cmp "$D/measurements.json" /tmp/board10-measurements.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py "$D" /tmp/board10-body.json
cmp "$D/body.json" /tmp/board10-body.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py "$D" /tmp/board10-sheet 52
```

## Bindings

`diode` 0.1.3 binds the 25 part numbers with `supplier lcsc "<part number>" rotate 180`, each through one row of `tools/sourcing_data/jlc_frames.json` (`zb10-sod3716x135n-<part number>`) keyed to `sod` 0.1.3, the land this board placed. Their map rows are from `../../lcsc_backfill/2026-11-21-hongjiacheng-zener/`.

## Manual review and limits

- Every copy is a manual-review reading by eye, as on boards 8 and 9; the measuring script gives no angle for a model that carries two marks. These are polarity readings with the mapping applied, not measured angles.
- The readings establish polarity on the assembler's model for each part number in this preview, not the part's identity. The BOM stock result describes this session.
- The half turn belongs to these part numbers' model, not to the series or the land: other part numbers of the same series, some on the same larger model, read correctly with no mapping on boards 8 and 9.
