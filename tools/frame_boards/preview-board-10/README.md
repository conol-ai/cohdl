# Preview board 10 — re-verify the 25 reversed Zeners

Boards 8 and 9 (records `jlc_previews/2026-11-21-board-8` and `-board-9`) read 25 Hongjiacheng SOD-123 Zeners reversed on all four copies: the assembler's model put its cathode band and "−" at our pad 2. A reading is a candidate, not a mapping. This board places each of the 25 again, four times (top 0 and 37 degrees, bottom 0 and 37 degrees), with the candidate `rotate 180` as the row's `mapping`, which `jlc_frame_board.py` writes into the supplier CPL. A part number is confirmed when every copy shows the band and "−" at our pad 1; only then may it be bound with `rotate 180`.

`zener_boards.py` writes the manifest from the two records' `readings.json`; it refuses a count other than 25.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-10 -o preview-board-10.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

The stock is the B1 snapshot's (2026-10-30); every one of the 25 passed the BOM step of boards 8 and 9 on 2026-11-21.
