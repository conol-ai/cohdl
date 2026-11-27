# Board 21 Hongjiacheng SOD-523 Zener re-verify preview, 2026-11-25

JLCPCB detected a two-layer, 48 × 168 mm board. Its PCBA BOM step confirmed all 13 C numbers and matched each designator exactly once on its side (26 top and 26 bottom placements), all rows selected, with no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-21 ZIP from commit `48575b1`, SHA-256 `351c1cdf6a1e4fd0b460754eb1ce9ae5ee1695cc0afa37d4a7769b3aafaa70fa`: the 13 BZX584C part numbers that board 20 (`../2026-11-24-board-20/`) read reversed on all four copies, each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sod::SOD1608X70N` with the candidate `rotate 180` as the row's mapping. The supplier CPL therefore reads 180 and 217 on the top and, under the `lcsc=mirrored` bottom convention, 180 and 143 on the bottom.

## Readings

All 13 part numbers read with the cathode at our pad 1 on all four copies: the candidate is confirmed. `readings.json` lists each part number with its designators.

As on board 20, each model carries two signs, so `checks.json` sends every copy to manual review ("unverified: none": every model is drawn). Every copy was read by eye on contact sheets of the lossless pixels: the model's band and "−" sit at our pad 1, beside our silkscreen bar, the "+" at pad 2, and the model's "SOD-523" text reads upside down, as a half turn leaves it. A pixel check agrees on all 52 copies: the magenta at the pad 2 end (the "+") outweighs that at the pad 1 end by more than 1.3 times.

| Part number | MPN | Copies | Result |
|---|---|---|---|
| C22395577 | BZX584C10 | D1–D4 | Pass, read by hand |
| C22395578 | BZX584C11 | D5–D8 | Pass, read by hand |
| C22395579 | BZX584C12 | D9–D12 | Pass, read by hand |
| C22395580 | BZX584C13 | D13–D16 | Pass, read by hand |
| C19077466 | BZX584C15 | D17–D20 | Pass, read by hand |
| C22395581 | BZX584C18 | D21–D24 | Pass, read by hand |
| C22395582 | BZX584C39 | D25–D28 | Pass, read by hand |
| C19077462 | BZX584C3V3 | D29–D32 | Pass, read by hand |
| C22395575 | BZX584C3V9 | D33–D36 | Pass, read by hand |
| C22395576 | BZX584C4V3 | D37–D40 | Pass, read by hand |
| C19077463 | BZX584C5V1 | D41–D44 | Pass, read by hand |
| C19077464 | BZX584C5V6 | D45–D48 | Pass, read by hand |
| C19077465 | BZX584C9V1 | D49–D52 | Pass, read by hand |

**Body readings** (`body.json`, with `../2026-11-21-board-8/body.py`): on every copy the drawn body's rectangle edges lie within 4.1 degrees of the pad 1 to pad 2 axis (mod 90), its long axis within 2.6 degrees (mod 180), and its centre within 0.095 mm of the footprint origin (limit 0.25 mm).

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` SHA-256 `126f85be…`, `bottom.png` `0f606689…`, `top.pixels.png` `7f80c94e…`, `bottom.pixels.png` `5936a057…`); `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-25-board-21
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_21-lcsc-cpl.csv" > /tmp/board21-checks.json
cmp "$D/checks.json" /tmp/board21-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board21-measurements.json
cmp "$D/measurements.json" /tmp/board21-measurements.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py "$D" /tmp/board21-body.json
cmp "$D/body.json" /tmp/board21-body.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py "$D" /tmp/board21-sheet 52
```

## Bindings

`diode` 0.1.5 binds the 10 of the 13 that have a map row (from `../../lcsc_backfill/2026-11-24-zener2/`) with `supplier lcsc "<part number>" rotate 180`, each through one row of `tools/sourcing_data/jlc_frames.json` (`zb21-sod1608x70n-<part number>`) keyed to `sod` 0.1.3, the land this board placed. C19077464 (BZX584C5V6), C22395578 (BZX584C11) and C22395579 (BZX584C12) are confirmed here but have no map row yet: their backfill was left `page_unparsed`.

## Manual review and limits

- Every copy is a manual-review reading by eye; the measuring script gives no angle for a model that carries two signs. These are polarity readings with the mapping applied, not measured angles.
- The readings establish polarity on the assembler's model for each part number in this preview, not the part's identity. The BOM stock result describes this session.
- The half turn belongs to these part numbers' model, not to the land: BZX584C2V4 and BZX584C3V6 on the same land, drawn with another model, read correctly with no mapping on board 20.
