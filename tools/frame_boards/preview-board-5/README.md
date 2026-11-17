# Preview board 5: re-verify three candidate rotations

Preview board 4 placed three ICs with no mapping, and each read the same turn on all four copies (`tools/sourcing_data/jlc_previews/2026-11-11-board-4/`, section A). A raw reading is a candidate, not a mapping. This board places the same three part numbers **with the candidate applied**, as the re-verify boards of 2026-11-08 and 2026-11-09 did for theirs. If the preview then shows each model's leads on our pads with pin 1 at our pad 1, on all four copies, the candidate is confirmed and the part number can be bound.

**Previewed on 2026-11-14; all three candidates confirmed.** The package built at `e6a0f92` (SHA-256 `f6af93dcb7028bb34a60ba9c85ff9deed99e7fadecb08b67257ddf83fb25c5a6`) went through the assembler's preview with Alex's approval and stopped at Component Placements: nothing was added to the cart. On all 12 copies the model's one pin 1 mark is at our pad 1 and its leads are on our pads. The record is `tools/sourcing_data/jlc_previews/2026-11-14-board-5/`, and the bindings it supports are `esd` 0.1.4, `load-switch` 0.2.1 and `@contrib/audio-amp` 0.2.1 ("After the preview" below).

**The manifest has moved since.** It now pins those three versions, because `preview_board_5.py` reads the library's versions and `preview_package.py` requires its output. Its rows are unchanged: each still declares a local part carrying the candidate as its `mapping`, so the board's BOM and CPL are the same. The generator now accepts a part number that is bound or framed only where its frame row cites the board-5 record with the candidate as mapping. The package that was previewed, with its `manifest.json` pinning `esd` 0.1.2, `load-switch` 0.2.0 and `@contrib/audio-amp` 0.2.0, is kept unchanged in the record's `package/`.

## What it carries

Three part numbers, 12 placements (6 top, 6 bottom). Each row is placed four times: top 0° and 37°, bottom 0° and 37°.

| Part | Library entry | Land | Board 4 read, raw (top 0°, top 37°, bottom 0°, bottom 37°) | Candidate |
|---|---|---|---|---|
| C7519 USBLC6-2SC6 | `esd::ESD_USBLC6` primary | `esd::FP_SOT_23_6` | 100.1°, 100.8°, 259.7°, 259.9° | `rotate 270` |
| C131941 TPS22918DBVR | `load-switch::LOADSW_TPS22918DBVR` primary | `load-switch::FP_TI_DBV0006A` | 183.0°, 183.0°, 177.4°, 178.2° | `rotate 180` |
| C189961 NS4150B | `@contrib/audio-amp::AMP_NS4150B` primary | `@contrib/audio-amp` `SOP8P65X490X110N` | 94.8°, 95.2°, 264.9°, 265.3° | `rotate 270` |

- The readings are the preview's pin 1 mark counter-clockwise from our pad 1, seen from the top (`measurements.json` of the board-4 record). A bottom copy reads the mirror of a top one, so a quarter turn reads about 90° on top and about 270° on the bottom.
- Each row declares a local part with the map row's identity on the library entry's device type and land, as on board 4, and carries the candidate as its `mapping`. `jlc_frame_board.py` writes it into the supplier CPL: a top copy authored at 0° goes out at 270° (or 180°), one at 37° at 307° (or 217°). The manifest's `candidate` repeats the raw readings and names the record.
- Pads, footprints and parts are the library's, unchanged. The manifest pins the versions in the tree (`esd`, `load-switch`, `@contrib/audio-amp`); `preview_package.py` refuses a manifest that is not `preview_board_5.py`'s output, so a package is always of the tree it is built from.

**Not on this board.** Board 3 read a quarter turn for two more part numbers placed raw, the SOT-23-5 C32574 and the SOT-23-6 C7420376, and did not record them (`jlc_previews/2026-11-05-c2-board-3/`, "Manual review and limits"). The library has no part for either: each borrowed another part's device for its pins. A re-verify would give evidence for a pair (land, part number) that nothing can bind, so they are left out until the library has the parts.

## Reference text kept away from the parts

The board-4 record found that silkscreen reference text beside a part disturbs a reading, and re-verify board 1 had the text hide two leads. Two things keep it away here:

- The copies stand 16 mm apart, each in a 14 mm box of its own.
- The package is built with the reference designators left out of the Gerber silkscreen, through the wrapper re-verify board 2 used (`tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes`, passed as `--kicad-cli`). `CHECKLIST.md` in the package lists each designator with its position, side and angle.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-5 -o preview-board-5.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

The supplier CPL is exported with `--bottom-rotation lcsc=mirrored`. The Gerbers, drill files and CPL all measure from the outline's lower-left corner. KiCad stamps a creation date into each Gerber, so the Gerber archive and the outer ZIP differ between runs while the BOM and CPL do not; an approval names one built package.

## Stock

From pages already in the repository (`lcsc_backfill/2026-11-07-samsung-b1/confirmed.json`); nothing is fetched. Each row needs 20 pieces (four placements on five boards), and the generator asks for at least 30.

| Part | Page read on | Stock on the page | Category |
|---|---|---:|---|
| C7519 | 2026-11-05 | 30,009 | extended |
| C131941 | 2026-11-05 | 71,324 | extended |
| C189961 | 2026-11-05 | 176,882 | extended |

All three were selected and drawn on board 4's preview of 2026-11-11. Stock has moved before between a page and a preview: `jlc_preview.py` stops on any stock warning, and the preview is then unverified.

## What to expect, and what passes

- On every copy the model's leads lie on our pads, and its one pin 1 mark is at our pad 1: `jlc_preview.py` reports "mark" (attribution 2: exactly one mark of the part's own, within 20° of pad 1), and `measure_preview.py measure` gives `delta_ccw_from_top` near 0 with `mark_ok`. C7519's mark sits outside the body beside a lead, and read 10° past the quarter turn on board 4; expect about 10° here too, inside the limit.
- Each copy is also read by eye on the export: leads on pads, pin 1 corner. A mark alone does not show that the leads are on the pads.
- A copy that is not selected, not drawn, or reads otherwise is unverified, not a pass. A candidate is confirmed only if all four of its copies pass.

## After the preview

The record goes to `tools/sourcing_data/jlc_previews/<date>-board-5/` with the exports, the package, `checks.json` and `measurements.json`. For each confirmed candidate, in the same pull request:

- a frame row citing that record, with the candidate as both `mapping` and `measured`;
- the `supplier lcsc` clause on the library entry and a new patch version of its package: `esd` (0.1.4 if #110's 0.1.3 has merged, since a published version never changes content), `load-switch` 0.2.1, `@contrib/audio-amp` 0.2.1;
- the pins, manifests and locks that follow.

A candidate that does not pass stays unbound, with what was read.

All three passed (2026-11-14). The frame rows are `pb5-fpsot236-c7519`, `pb5-fptidbv0006a-c131941` and `pb5-sop8p65x490x110n-c189961`; the bindings are in `esd` 0.1.4 (#110's 0.1.3 plus this binding), `load-switch` 0.2.1 and `@contrib/audio-amp` 0.2.1.
