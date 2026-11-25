# Demand batch 2 preview board

**Built, not previewed, not evidence.** This directory holds the input
manifest, the built board package and renders for an assembler preview of
23 candidate part numbers whose library parts are unbound. Nothing here is
a binding, and no frame row is written from it.

## Why it exists

The parts added on 2026-11-25 (`son` 0.1.0, `load-switch` 0.2.2, `ldo`
0.1.3, `dcdc` 0.1.2, `@ti/power-switch` 0.1.1, `fuse` 0.1.0) have
candidate LCSC part numbers but no `supplier lcsc` clause: a binding needs
a frame row with evidence in `tools/sourcing_data/jlc_frames.json`, and
that evidence is an assembler preview read by a person. This board is the
input for that preview.

## Rows

Each row declares a local part with the candidate part number on the
library device and land, with a zero mapping, placed four times: top 0 and
37 degrees, bottom 0 and 37 degrees, in 14 mm boxes on a 16 mm pitch.

| Library part (unbound) | Candidate | Land |
|---|---|---|
| `load_switch::LOADSW_TPS2553DRVR` | C138719 | `son::TI_DRV0006A` |
| `load_switch::LOADSW_TPS2553DRVR_1` | C411878 | `son::TI_DRV0006A` |
| `load_switch::LOADSW_TPS2552DRVR` | C473916 | `son::TI_DRV0006A` |
| `ldo::LDO_LP5912_3V3` | C524780 | `son::TI_DRV0006A` |
| `ldo::LDO_LP5912_1V8` | C2876234 | `son::TI_DRV0006A` |
| `ldo::LDO_LP5912_5V0` | C2761351 | `son::TI_DRV0006A` |
| `ldo::LDO_TLV75533PDRVR` | C2861750 | `son::TI_DRV0006A` |
| `ldo::LDO_TLV75512PDRVR` | C2866903 | `son::TI_DRV0006A` |
| `ldo::LDO_TLV75525PDRVR` | C544744 | `son::TI_DRV0006A` |
| `dcdc::DCDC_LMR36520ADDAR` | C2879422 | `dcdc::TI_DDA0008B` |
| `dcdc::DCDC_LMR36520FADDAR` | C1858395 | `dcdc::TI_DDA0008B` |
| `ti_power_switch::EFUSE_TPS26630RGER` | C1850276 | `ti_power_switch::TI_RGE0024H` |
| `ti_power_switch::EFUSE_TPS26631RGER` | C1850273 | `ti_power_switch::TI_RGE0024H` |
| `ti_power_switch::EFUSE_TPS26632RGER` | C1850278 | `ti_power_switch::TI_RGE0024H` |
| `ti_power_switch::EFUSE_TPS26620DRCR` | C1848361 | `ti_power_switch::TI_DRC0010J` |
| `ti_power_switch::EFUSE_TPS26621DRCR` | C1848341 | `ti_power_switch::TI_DRC0010J` |
| `ti_power_switch::EFUSE_TPS259827ONRGET` | C2155765 | `ti_power_switch::TI_RGE0024M` |
| `ti_power_switch::EFUSE_TPS259827ONRGER` | C2155761 | `ti_power_switch::TI_RGE0024M` |
| `fuse::FUSE_451_0A5` | C142836 | `fuse::FP_LITTELFUSE_NANO2` |
| `fuse::FUSE_451_1A` | C3099 | `fuse::FP_LITTELFUSE_NANO2` |
| `fuse::FUSE_451_2A` | C99547 | `fuse::FP_LITTELFUSE_NANO2` |
| `fuse::FUSE_451_3A` | C13982 | `fuse::FP_LITTELFUSE_NANO2` |
| `fuse::FUSE_451_4A` | C27515 | `fuse::FP_LITTELFUSE_NANO2` |

The manufacturer and MPN in each row are the library part's. The `stock`
field is the local jlcparts snapshot of 2026-11-25; the part pages were
not read (C3099 excepted), so no row carries a page date, package or
class. None of the 23 is in `tools/sourcing_data/lcsc_map.json`. Several
are too thinly stocked to place 20 pieces: C1850276 has 48, C1848361 has
196.

In each row the first two pins (a supply input and a ground, or the two
fuse terminals) form the board's two nets; the tool marks the remaining
listed pins unconnected.

## State on 2026-11-25

- `manifest.json` was written by hand in the input format of
  `tools/jlc_frame_board.py`; there is no generator script for it.
- **The board package is built** (Python 3.11.0rc1, `kicad-cli` 10.0.8):

  ```sh
  python3.11 -I tools/jlc_frame_board.py tools/frame_boards/demand-batch-2/manifest.json \
    -o BOARD --cohdl target/debug/cohdl
  ```

  It reported `generated 92 placements`. A 215 x 145 mm board.
- `package/` is a copy of that output: `fabrication/` (Gerbers, drill file
  and `SHA256SUMS`), `demand_batch_2.kicad_pcb`,
  `demand_batch_2-lcsc-bom.csv`, `demand_batch_2-lcsc-cpl.csv`,
  `demand_batch_2-cpl.csv` and the tool's `CHECKLIST.md`.
- `render/` holds 94 `kicad-cli pcb render` images of that board file:
  `top.png` and `bottom.png` (each side seen from its own side), and one
  480 px crop per copy, 92 in all, named
  `<C-number>_<designator>_<side>_<angle>.png` (16 mm square about the
  placement). The renders show our copper, paste and silkscreen only.
  They contain no supplier model and are not a preview.
- **Checked against the board file, not against any preview:** all 23
  part numbers are in the LCSC BOM with four designators each; every part
  number has a top 0, top 37, bottom 0 and bottom 37 copy at its manifest
  position; each copy is on the intended land with the intended number of
  distinct pad numbers (7 on `TI_DRV0006A`, 9 on `TI_DDA0008B`, 25 on
  `TI_RGE0024H`, 26 on `TI_RGE0024M`, 11 on `TI_DRC0010J`, 2 on
  `FP_LITTELFUSE_NANO2`).
- **The silkscreen carries reference designators.** Preview board 5 left
  them out of the Gerber silkscreen because text beside a part disturbed
  the measuring script's readings. Regenerate without them if the readings
  are to be measured by `measure_preview.py`.
- Nothing was uploaded. No preview was run, no reading exists, and no row
  of `tools/sourcing_data/jlc_frames.json` or `lcsc_map.json` was written.

## What a person still has to confirm

For each part number, on JLCPCB's assembly preview of the built board:

1. The model is drawn, on our pads, on all four copies, and it is the
   package the land is for (a 2 x 2 mm WSON-6, an 8-pin PowerPAD SOIC, a
   4 x 4 mm VQFN-24, a 3 x 3 mm VSON-10, a 2410 fuse).
2. For the ICs, the model's pin 1 mark sits at our pad 1 (the silkscreen
   dot). A quarter or half turn is a candidate rotation, to be confirmed
   on a second board with the candidate applied, as preview board 5 did.
3. For the fuses, which have no pin 1 and no polarity, the body's long
   axis lies along the pad axis and is centred; orientation is known mod
   180 only.
4. For the three WSON-6 families sharing `son::TI_DRV0006A`, each part
   number is read on its own: a rotation found for one says nothing about
   another.

Only after that review does a part get a frame row with `evidence`, a map
row from a promoted `tools/lcsc_backfill.py` run, and its
`supplier lcsc` clause in a new package version.
