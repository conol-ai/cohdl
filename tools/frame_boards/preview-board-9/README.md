# Preview board 9 — Hongjiacheng SOD-123 Zeners (MMSZ52xxB, MMSZ46xx, MM1Z..B and MM1Z)

The board places 60 part numbers, 240 placements: every Zhuhai Hongjiacheng SOD-123 Zener of these series in the B1 snapshot whose page reads at least 30 in stock, four times each (top 0 and 37 degrees, bottom 0 and 37 degrees). `zener_boards.py` writes the manifest; boards 8 and 9 together cover the snapshot's 128 such part numbers except the four it leaves out for stock (C19077397, C22395534, C22395535, C22466340).

Each row is a local part with the snapshot page's identity on the library's `diode::Zener<VZ, P>` device and the `sod::SOD3716X135N` land, placed with no mapping. A Zener is polarized, so this is a first reading per part number: the assembler's model must put its cathode band at our pad 1 (the land's band end, at -x) on all four copies. Nothing here is a binding; bindings need these readings and map rows from a reviewed backfill.

## Reference text kept away from the parts

The copies stand 10 mm apart, each in an 8 mm box (the land spans at most 5.2 mm at 37 degrees). Build the package with the reference designators left out of the Gerber silkscreen, as board 5 was. `CHECKLIST.md` in the package lists each designator with its position, side and angle.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-9 -o preview-board-9.zip \
  --cohdl target/debug/cohdl \
  --kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

## Stock

The stock is the snapshot's (2026-10-30). The preview's BOM step shows the stock of the day; a part number short there is left out of a rebuilt package, which needs its own approval.
