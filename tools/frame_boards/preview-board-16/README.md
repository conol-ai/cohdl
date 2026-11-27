# Preview board 16 — SOT-23 MOSFETs

The board places 29 part numbers, 116 placements: the 29 MOSFETs `tools/gen_sot23.py` added to the library (Zhuhai Hongjiacheng, JSCJ, Vishay), four times each (top 0 and 37 degrees, bottom 0 and 37 degrees). `sot23_boards.py` writes the manifest; boards 15 and 16 together cover all 74.

Each row is a local part with the library part's identity on its `mosfet::NMOS_SOT23` and `PMOS_SOT23` device and the `sot::SOT3P190X290X130N` land, placed with no mapping, under the C number the B1 snapshot page gives for that manufacturer and MPN. A transistor is polarized, so this is a first reading per part number: the assembler's model must put its pin 1 mark at our pad 1 (pin 1 (the gate), lower right in the land's frame) on all four copies. Nothing here is a binding; bindings need these readings and map rows from a reviewed backfill.

## Reference text kept away from the parts

The copies stand 10 mm apart, each in an 8 mm box (the land's courtyard is 3.4 x 3.6 mm). Build the package with the reference designators left out of the Gerber silkscreen, as boards 5 and 8 to 14 were. `CHECKLIST.md` in the package lists each designator with its position, side and angle.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-16 -o preview-board-16.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

## Stock

The stock is the snapshot's (2026-10-30); every page here reads at least 40, what the BOM step asks for four placements on ten boards. The preview's BOM step shows the stock of the day; a part number short there is left out of a rebuilt package, which needs its own approval.
