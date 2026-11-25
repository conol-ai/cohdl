# Demand batch 1 preview board (input only)

**Not built, not previewed, not evidence.** This directory holds the input
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
  `tools/jlc_frame_board.py`; there is no generator script for it. The
  tool's `load_manifest` and `expand` accept it.
- **The board package was not generated.** `tools/jlc_frame_board.py`
  needs Python 3.11 (`tomllib`) and `kicad-cli`; the machine this was
  written on had Python 3.10 and no `kicad-cli`. To build it:

  ```sh
  python3 -I tools/jlc_frame_board.py tools/frame_boards/demand-batch-1/manifest.json \
    -o BOARD --cohdl target/debug/cohdl
  ```

- No BOM, CPL, Gerber or checklist exists yet, and nothing was uploaded.

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
