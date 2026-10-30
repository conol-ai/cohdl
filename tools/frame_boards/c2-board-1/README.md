# C2 frame board 1

The first frame-alignment board of library plan phase C2. It holds 30 LCSC parts, each placed four times:

- top 0°;
- top 37°;
- bottom 0°;
- bottom 37°.

That makes 120 placements. Each part is a Basic or Preferred part from the W1 snapshot of 2026-10-30, placed on a library footprint of the same package. The one exception is the USB-C receptacle.

No row carries a supplier mapping. The preview therefore shows each footprint's raw frame offset against the assembler's model:

- C2146 and C97521 re-measure the C1 results (`rotate 90` and `rotate 270`).
- Two more SOT-23 models (C8545, C20526) check that one SOT-23 result holds across models.
- The LED, SOD-123 and DFN1006 rows check polarity.
- The USB-C receptacle (C165948, Extended, preview only) is the CPL origin control. Its courtyard centre is off the footprint origin (RFC-034 §3.1).

The LQFP48_IPC row places HT1621B (C7532) on `qfp::QFP50P900X900X160_48N`. That row reuses an STM32 device of the footprint only for its pins.

## Regenerate

```sh
python3 -I tools/frame_boards/inventory.py /tmp/inv.json
python3 -I tools/frame_boards/c2_board_1.py /tmp/inv.json tools/frame_boards/c2-board-1/manifest.json
python3 -I tools/jlc_frame_board.py tools/frame_boards/c2-board-1/manifest.json -o BOARD \
  --cohdl target/debug/cohdl --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
```

The supplier CPL is exported with `--bottom-rotation lcsc=mirrored`. The Gerbers, drill files and CPL all measure from the outline's lower-left corner.

## Checks before upload (2026-10-30)

- The compiler CPL equals `smt_pos.py --map` on the board: 120 of 120 rows match.
- With the drill/place origin, it also equals KiCad's own position export in position, side and top-side angle: 120 of 120 rows match.

Upload for preview only and record the result in `jlc_frames.json` with the screenshots. Do not order.

The manifest pins the library as previewed, before transistor 0.2.0, soic 0.3.0 and qfp 0.3.0 moved nine of its footprints onto the assembler's frame. The preview of 2026-10-30 is recorded in `tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`, and its rows in `jlc_frames.json` start with `c2b1-`.
