# Preview board 19 — Hongjiacheng SOT-23 diodes with their candidate rotations

The board places the 18 part numbers board 17 (`tools/sourcing_data/jlc_previews/2026-11-23-board-17/`) left for manual review, all Zhuhai Hongjiacheng, four times each (top 0 and 37 degrees, bottom 0 and 37 degrees), 72 placements:

- 11 diodes drawn with the model boards 15 and 16 found: centred on the land but a quarter turn against it. Their candidate is `rotate 90`, the one board 18 checks for three transistors and MOSFETs.
- 7 BZX84C Zeners drawn with a SOT-23 model a half turn against the land: its two leads on our pad 3 side, its pin 1 mark at the upper left. Turned 180 degrees about the origin, each lead falls on our pad of the same number. Their candidate is `rotate 180`.

Each row carries its candidate, with no offset, as its `mapping`, which `jlc_frame_board.py` writes into the supplier CPL. `sot23_boards.py` takes the angle from the record's own reading (`candidate_angle`: the quarter or half turn its top copy at 0 degrees read, turned back) and stops if any of the four copies read another. A candidate passes when all four copies of each part number show the model's pin 1 mark at our pad 1. Nothing here is a binding.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-19 -o preview-board-19.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

## Stock

The stock is the snapshot's (2026-10-30); every page here reads at least 40. The preview's BOM step shows the stock of the day.
