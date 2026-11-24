# Board 20 Hongjiacheng SOD-323 and SOD-523 Zener preview, 2026-11-24

JLCPCB detected a two-layer, 188 × 168 mm board. Its PCBA BOM step confirmed all 71 C numbers and matched each designator exactly once on its side (142 top and 142 bottom placements), all rows selected, with no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-20 ZIP from commit `a9d79c4`, SHA-256 `d674d8d1f9e78ccdee482b4a7c877c156d14e6a45f7bf7df53a40e2a8ff774c5`: the 71 B1 Hongjiacheng Zeners of `diode` 0.1.5 (BZT52B…S, BZT52C…S, MM3Z and MMSZ52xxBS on `sod::SOD2613X110N`, SOD-323; BZX584C on `sod::SOD1608X70N`, SOD-523), each placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) with no mapping.

The designators follow the BOM, not the manifest's row order: the board is laid out by land, then series, then C number (`tools/frame_boards/zener_small_boards.py`), so D1–D140 are the BZT52 series, D141–D200 the BZX584C, D201–D260 the MM3Z and D261–D284 the MMSZ. `readings.json` gives each part number's four designators.

## Readings

58 part numbers read with the cathode at our pad 1 on all four copies; 13 read reversed on all four copies; none was left undrawn.

- **Cathode at pad 1** (58): all 35 BZT52B…S and BZT52C…S, all 15 MM3Z, all 6 MMSZ52xxBS (SOD-323), and two BZX584C: C22395573 (BZX584C2V4) and C22395574 (BZX584C3V6), which the assembler draws with a different model from the rest of the series.
- **Reversed on all four copies** (candidate `rotate 180`): the other 13 BZX584C, C22395577 (BZX584C10), C22395578 (C11), C22395579 (C12), C22395580 (C13), C19077466 (C15), C22395581 (C18), C22395582 (C39), C19077462 (C3V3), C22395575 (C3V9), C22395576 (C4V3), C19077463 (C5V1), C19077464 (C5V6) and C19077465 (C9V1). Each is drawn with the assembler's model labelled "SOD-523", with its "+" over our pad 1 and its band and "−" at pad 2. The same half-turned SOD-523 model was seen on Hongjiacheng's 1N4148WT.
- **Not drawn:** none.

A reversed reading is a candidate, not a mapping: it becomes a binding only after a re-verify board places the part with `rotate 180` applied and shows the band at our pad 1, as board 10 did for the SOD-123 Zeners.

**Method.** Each model carries two signs of its own, a "+" and a "−" (some also a magenta arrow on the body), so the preview tool cannot judge these copies. Every copy was read by eye on contact sheets of the lossless pixels (`../2026-11-21-board-8/contact_sheet.py`, and the same crops at 3.6 mm and 2.5 mm squares for the SOD-523 copies): the model's cathode band and its "−" against our pad 1, the land's band end, beside our silkscreen bar. A pixel check agrees on 282 of the 284 copies: the magenta at the pad 1 end outweighs that at the pad 2 end by at least 1.3 times on every reversed copy (the "+" is the heavier sign) and the reverse on every copy read correctly. The two it leaves undecided, D67 and D68 (BZT52C27S, bottom), carry a large magenta arrow on the body; by eye, the arrow points to pad 1 and the "−" and band sit there.

**The run's own verdicts and the tool fix.** The run judged five copies, D145, D149, D161, D169 and D185 (BZX584C11, C12, C18, C39 and C4V3, all reversed), as "mark": on each, the preview tool found one magenta sign of the part's own and it lay at pad 1, and the tool took it for a pin 1 mark. It was the "+". `checks-at-run.json` is the run's `checks.json` as written. Commit `9eca2f0`, just before this record, changes `jlc_preview.verdicts`: a two-terminal part with one mark at pad 1 now goes to manual review, as the jlc-preview skill already said; `checks.json` is the recheck under the fixed code, and sends all 284 copies to manual review. No archived record is changed by the fix: none had a two-terminal part judged "mark".

**Body readings** (`body.json`, with `../2026-11-21-board-8/body.py`, for the 232 copies read with the cathode at pad 1): on every copy the drawn body's rectangle edges lie within 3.3 degrees of the pad 1 to pad 2 axis (mod 90) and its centre within 0.146 mm of the footprint origin (limit 0.25 mm). The principal axis wanders more (up to 6.7 degrees, on the MMSZ bodies, which are nearly square at about 1.6 × 1.2 mm), so the rectangle edges are the figure the rows use.

| Series | Land | Part numbers | Cathode at pad 1 | Reversed |
|---|---|---|---|---|
| BZT52B…S, BZT52C…S | `sod::SOD2613X110N` | 35 | 35 | 0 |
| BZX584C | `sod::SOD1608X70N` | 15 | 2 | 13 |
| MM3Z | `sod::SOD2613X110N` | 15 | 15 | 0 |
| MMSZ52xxBS | `sod::SOD2613X110N` | 6 | 6 | 0 |

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived (decoder `sips -s format bmp`, sips-316, macOS 27.0.1; `top.png` SHA-256 `11765dd8…`, `bottom.png` `306827a6…`, `top.pixels.png` `e7d13d57…`, `bottom.pixels.png` `15cfad56…`); `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-24-board-20
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_20-lcsc-cpl.csv" > /tmp/board20-checks.json
cmp "$D/checks.json" /tmp/board20-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board20-measurements.json
cmp "$D/measurements.json" /tmp/board20-measurements.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py "$D" /tmp/board20-body.json
cmp "$D/body.json" /tmp/board20-body.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py "$D" /tmp/board20-sheet 64
```

The last command writes the contact sheets (64 per sheet, a part number's four copies side by side).

## Manual review and limits

- Every copy is a manual-review reading by eye; the measuring script gives no angle for a model that carries two signs. These are polarity readings, not measured angles.
- The readings establish polarity on the assembler's model for each part number in this preview, not the part's identity. The BOM stock result describes this session.
- The half turn belongs to the BZX584C part numbers' model, not to the land: two BZX584C on the same land, drawn with another model, read correctly.
- No binding or frame row is written by this record alone. A part number is bound only where it also has a map row from a reviewed backfill; 35 of the 71 have one (`../../lcsc_backfill/2026-11-24-zener2/`).
