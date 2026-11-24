# Preview board 20 — SOD-323 and SOD-523 Zeners

The board places 71 part numbers, 284 placements: every Zhuhai Hongjiacheng SOD-323 and SOD-523 Zener of the B1 snapshot (jlc_catalog/2026-10-30), four times each (top 0 and 37 degrees, bottom 0 and 37 degrees). By series: BZT52C..S 29, BZT52B..S 6, MM3Z 15 and MMSZ52xxBS 6 on `sod::SOD2613X110N` (SOD-323), and BZX584C 15 on `sod::SOD1608X70N` (SOD-523). `zener_small_boards.py` writes the manifest and refuses a count other than 71.

Each row is a local part with the snapshot page's identity on `diode::Zener<VZ, P>` and the land the part's own sheet names, the library part of `lib/diode/src/zener_sod323_sod523.cohdl` (`diode` 0.1.5), placed with no mapping. A Zener is polarized, so this is a first reading per part number: the assembler's model must put its cathode band at our pad 1 on all four copies. Nothing here is a binding; bindings need these readings and map rows from a reviewed backfill. The B5819WS (SOD-323) and 1N4148WT (SOD-523) records found Hongjiacheng's models a half turn from these lands, so a part number read reversed on all four copies goes to a re-verify board with the candidate `rotate 180`, as boards 8 to 10 did.

## Reference text kept away from the parts

The copies stand 10 mm apart, each in an 8 mm box (the SOD-323 courtyard is 3.4 x 2.0 mm, the SOD-523 one 2.6 x 1.5 mm). Build the package with the reference designators left out of the Gerber silkscreen, as boards 5 and 8 to 19 were. `CHECKLIST.md` in the package lists each designator with its position, side and angle.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-20 -o preview-board-20.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

## Stock

The stock is the snapshot's (2026-10-30); every page here reads at least 40, what the BOM step asks for four placements on ten boards. The preview's BOM step shows the stock of the day; a part number short there is left out of a rebuilt package, which needs its own approval.
