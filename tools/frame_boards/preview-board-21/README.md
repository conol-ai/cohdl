# Preview board 21 — re-verify the 13 reversed SOD-523 Zeners

Board 20 (record `jlc_previews/2026-11-24-board-20`) read 13 Hongjiacheng BZX584C Zeners reversed on all four copies: the assembler's "SOD-523" model put its "+" at our pad 1 and its band and "−" at pad 2. A reading is a candidate, not a mapping. This board places each of the 13 again, four times (top 0 and 37 degrees, bottom 0 and 37 degrees), on `sod::SOD1608X70N` with the candidate `rotate 180` as the row's `mapping`, which `jlc_frame_board.py` writes into the supplier CPL. A part number is confirmed when every copy shows the band and "−" at our pad 1; only then may it be bound with `rotate 180`, and only once it also has a map row (10 of the 13 have one; C19077464, C22395578 and C22395579 wait on the backfill).

`zener_small_boards.py` writes the manifest from board 20's `readings.json`; it refuses a count other than 13.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-21 -o preview-board-21.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

The stock is the B1 snapshot's (2026-10-30); every one of the 13 passed the BOM step of board 20 on 2026-11-24.
