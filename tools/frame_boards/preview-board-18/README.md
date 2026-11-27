# Preview board 18 — Hongjiacheng SOT-23 parts with the candidate rotate 90

The board places the three part numbers boards 15 and 16 left for manual review, four times each (top 0 and 37 degrees, bottom 0 and 37 degrees), 12 placements:

- C20069147, Zhuhai Hongjiacheng MMBTA56 (`transistor::PNP_SOT23`), from `tools/sourcing_data/jlc_previews/2026-11-23-board-15/`;
- C28646268, HL2302 (`mosfet::NMOS_SOT23`), and C28646269, HL3407A (`mosfet::PMOS_SOT23`), from `tools/sourcing_data/jlc_previews/2026-11-23-board-16/`.

On those boards the assembler drew the three with one model, centred on the land but a quarter turn against it: its pin 1 lead at the lower left, its pin 3 lead at the top, the leads above and below our pads. Turned 90 degrees counter-clockwise (seen from the top) about the origin, each lead falls on our pad of the same number. Each row here carries that candidate, `rotate 90` with no offset, as its `mapping`, which `jlc_frame_board.py` writes into the supplier CPL. The candidate passes when all four copies of each part number show the model's pin 1 mark at our pad 1, its leads on our pads. Nothing here is a binding.

`sot23_boards.py` writes the manifest (`REVERIFY`): the part numbers come from the two records' `readings.json`, every one they left for manual review.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-18 -o preview-board-18.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

## Stock

The stock is the snapshot's (2026-10-30); every page here reads at least 40. The preview's BOM step shows the stock of the day.
