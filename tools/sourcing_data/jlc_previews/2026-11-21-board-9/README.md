# Board 9 Hongjiacheng SOD-123 Zener preview, 2026-11-21

JLCPCB detected a two-layer, 168 × 168 mm board. Its PCBA BOM step confirmed all 60 C numbers and matched each designator exactly once on its side (240 placements), all rows selected, with no stock warning. The run ended at Component Placements. Its input was the preview-board-9 ZIP from commit `56da519`, SHA-256 `585f04680f9097924a4cdec904f843ff040dc68c0056ae6139b6fdc74ec10652`, the series MMSZ52xxB, MMSZ46xx, MM1Z..B and MM1Z, each part placed four times (top 0 and 37 degrees, bottom 0 and 37 degrees) on `sod::SOD3716X135N` with no mapping.

## Readings

40 part numbers read with the cathode at our pad 1 on all four copies; 19 read reversed on all four copies; 1 was not drawn. `readings.json` lists each part number with its designators and reading.

The preview tool cannot judge these: a Zener is polarized, and each model carries two signs of its own ("2 marks: which is pin 1 is read by hand"), so `checks.json` sends every drawn copy to manual review. The reading is the model's cathode band and its "−" sign against our pad 1, the land's band end, beside our silkscreen bar. This export draws the reference designators in red over each model, which hides much of the magenta '-' and '+'; the white cathode band inside the body stays visible and was the cue on the small model. Every copy was read by eye.

- **Reversed on all four copies** (candidate `rotate 180`): C22395536 (MM1Z5V6B), C22395538 (MM1Z5V1), C22395539 (MM1Z5V6), C22395540 (MM1Z62), C22395547 (MMSZ4688), C22395553 (MMSZ5227B), C22395554 (MMSZ5230B), C22395555 (MMSZ5234B), C22395556 (MMSZ5236B), C22395558 (MMSZ5239B), C22395559 (MMSZ5241B), C22395560 (MMSZ5251B), C22395561 (MMSZ5254B), C22399894 (MMSZ5225B), C22399895 (MMSZ5228B), C22399896 (MMSZ5229B), C22399897 (MMSZ5238B), C22399899 (MMSZ5258B), C22399900 (MMSZ5259B). Each uses the assembler's larger SOD-123 model, with "+" over our pad 1 and the band and "−" at the far end. The same larger model appears on some part numbers that read correctly, so the half turn belongs to the part number's model, not to the series. The reversed part numbers fall in consecutive C-number runs (C22395529–C22395561, C22399894–C22399900).
- **Not drawn:** C22466344 (MM1Z6V2B, D93-D96): the assembler drew a placeholder, so nothing is read.

A reversed reading is a candidate, not a mapping: it becomes a binding only after a re-verify board places the part with `rotate 180` applied and shows the band at our pad 1, as board 5 did for three ICs.

## Reproduction

The exports, their lossless pixels and `pixels.json` are archived; `package/` holds the unpacked approved ZIP. From the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-21-board-9
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_9-lcsc-cpl.csv" > /tmp/board9-checks.json
cmp "$D/checks.json" /tmp/board9-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board9-measurements.json
cmp "$D/measurements.json" /tmp/board9-measurements.json
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/contact_sheet.py "$D" /tmp/board9-sheet 60
```

The last command writes the contact sheets the readings were made from (60 per sheet, four copies of a part number side by side).

## Limits

These readings establish polarity on the assembler's model for each part number in this preview, not the part's identity. The BOM stock result describes this session. No binding or frame row is written here; bindings need these readings and map rows from a reviewed backfill.

## Body readings and bindings (added 2026-11-21)

The bindings came after the map rows: the backfill run `../../lcsc_backfill/2026-11-21-hongjiacheng-zener/` confirmed each part number on its own page and was promoted into `lcsc_map.json`.

**Body readings.** `body.json` reads the 160 copies of the 40 part numbers read with the cathode at pad 1, with `../2026-11-21-board-8/body.py` (the stock `../2026-11-09-reverify-2/body_axes.py` with this land's axis added) on the archived lossless pixels. On every copy the drawn body's long axis lies within 2.4 degrees of the pad 1 to pad 2 axis (mod 180) and its centre within 0.176 mm of the footprint origin (limit 0.25 mm). The export's red reference text covers parts of some models; the body test counts only dark pixels, so the text leaves gaps in the body but does not move its axis. To reproduce, from the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-21-board-9
python3 -I tools/sourcing_data/jlc_previews/2026-11-21-board-8/body.py "$D" /tmp/board9-body.json
cmp "$D/body.json" /tmp/board9-body.json
```

**Bindings.** `diode` 0.1.3 binds those 40 part numbers with `supplier lcsc "<part number>"` and no mapping, each through one row of `tools/sourcing_data/jlc_frames.json` (`zb9-sod3716x135n-<part number>`) keyed to `sod` 0.1.3, the land this board placed. Each row's note names the copies, the body figures and what the reading is not. The reversed part numbers are not bound; preview board 10 re-verifies them with the candidate `rotate 180`.
