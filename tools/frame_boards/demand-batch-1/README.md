# Demand batch 1 preview board

**Built, not previewed, not evidence.** This directory holds the input
manifest for an assembler preview of ten part numbers whose library parts
are unbound. Nothing here is a binding, and no frame row is written from
it.

## Why it exists

The parts added on 2026-11-25 (`load-switch` 0.2.2, `diode` 0.1.3,
`transistor` 0.2.2) have candidate LCSC part numbers but no
`supplier lcsc` clause: a binding needs a frame row with evidence in
`tools/sourcing_data/jlc_frames.json`, and that evidence is an assembler
preview read by a person. This board is the input for that preview.

## Rows

Each row declares a local part with the candidate part number on the
library device and land, with a zero mapping, placed four times: top 0 and
37 degrees, bottom 0 and 37 degrees, in 14 mm boxes on a 16 mm pitch (the
preview board 5 layout).

| Library part (unbound) | Candidate | Land |
|---|---|---|
| `load_switch::LOADSW_TPS2553DBVR` | C55266 | `load_switch::FP_TI_DBV0006A` |
| `load_switch::LOADSW_TPS2553DBVR_1` | C111738 | `load_switch::FP_TI_DBV0006A` |
| `load_switch::LOADSW_TPS2552DBVR` | C46506 | `load_switch::FP_TI_DBV0006A` |
| `diode::TVS_VISHAY_SMBJ30A` | C1973126 | `do214::DIOM5336X265N` |
| `diode::TVS_MDD_SMBJ30A` | C113998 | `do214::DIOM5336X265N` |
| `diode::TVS_HONGJIACHENG_SMBJ30A` | C19077584 | `do214::DIOM5336X265N` |
| `diode::TVS_VISHAY_SMBJ30CA` | C553967 | `do214::DIOM5336X265N` |
| `diode::TVS_MDD_SMBJ30CA` | C113999 | `do214::DIOM5336X265N` |
| `diode::TVS_HONGJIACHENG_SMBJ30CA` | C19077585 | `do214::DIOM5336X265N` |
| `transistor::FET_BSS138` | C52895 | `sot::SOT3P190X290X130N` |

The manufacturer, MPN, package, class and stock in each row are what the
part's JLCPCB page read on 2026-11-25. None of the ten is in
`tools/sourcing_data/lcsc_map.json`.

## State on 2026-11-25

- `manifest.json` was written by hand in the input format of
  `tools/jlc_frame_board.py`; there is no generator script for it.
- **The board package is built** (library at commit `537c43d`, Python
  3.11.0rc1, `kicad-cli` 10.0.8):

  ```sh
  python3.11 -I tools/jlc_frame_board.py tools/frame_boards/demand-batch-1/manifest.json \
    -o BOARD --cohdl target/debug/cohdl
  ```

  It reported `generated 40 placements`. A 180 x 75 mm board.
- `package/` is a copy of that output: `fabrication/` (Gerbers, drill file
  and `SHA256SUMS`), `demand_batch_1.kicad_pcb`,
  `demand_batch_1-lcsc-bom.csv`, `demand_batch_1-lcsc-cpl.csv`,
  `demand_batch_1-cpl.csv` and the tool's `CHECKLIST.md`.
- `render/` holds `kicad-cli pcb render` images of that board file:
  `top.png` and `bottom.png` (each side seen from its own side), and one
  480 px crop per copy, named `<C-number>_<designator>_<side>_<angle>.png`
  (16 mm square about the placement). The renders show our copper and
  silkscreen only. They contain no supplier model and are not a preview.
- **Checked against the board file, not against any preview:** all ten part
  numbers are in the LCSC BOM with four designators each; every part
  number has a top 0, top 37, bottom 0 and bottom 37 copy at its manifest
  position; each copy is on the intended land with the intended pad
  numbers and sizes (six 1.1 x 0.6 mm pads on `FP_TI_DBV0006A`, two
  2.10 x 2.26 mm on `DIOM5336X265N`, three 0.8 x 0.9 mm on
  `SOT3P190X290X130N`).
- **The silkscreen carries reference designators.** Preview board 5 left
  them out of the Gerber silkscreen because text beside a part disturbed
  the measuring script's readings. Regenerate without them if the readings
  are to be measured by `measure_preview.py`.
- Nothing was uploaded. No preview was run, no reading exists, and no row
  of `tools/sourcing_data/jlc_frames.json` or `lcsc_map.json` was written.

## What a person still has to confirm

For each part number, on JLCPCB's assembly preview of the built board:

1. The model is drawn, on our pads, on all four copies.
2. Pin 1 (SOT-23-6, SOT-23) or the cathode band (unidirectional SMBJ) sits
   at our pad 1; if it reads a quarter or half turn, that reading is a
   candidate rotation, to be confirmed on a second board with the
   candidate applied, as preview board 5 did.
3. For the bidirectional SMBJ parts, the body's long axis lies along the
   pad axis and is centred (the body-only rule of
   `docs/library-supplier-plan.md`); orientation is known mod 180 only.

Only after that review does a part get a frame row with `evidence`, a map
row from a promoted `tools/lcsc_backfill.py` run, and its
`supplier lcsc` clause in a new package version.
