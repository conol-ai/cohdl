# USB-C origin check: assembler preview, 2026-10-30

The assembler's placement preview of the USB-C origin check board (`tools/frame_boards/usbc-origin/`), uploaded for preview only, with no order. Row `usbc-origin-c165948` of `jlc_frames.json` cites this record.

## What was uploaded

The board holds three parts, each placed four times: top 0°, top 37°, bottom 0° and bottom 37°. The CPL was exported with `--bottom-rotation lcsc=mirrored`.

| Part | Footprint | Map |
|---|---|---|
| C165948 (J1–J4) | `usb::connectors::type_c::FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12`, usb 0.1.5 | `at (0mm, -1.55mm)`, from C2 frame board 1 |
| C2146 (Q1–Q4) | `transistor::FP_JSCJ_SOT23`, transistor 0.2.0 | none |
| C97521 (U1–U4) | `soic::SOIC8P127X790X216N`, soic 0.3.0 | none |

| File | sha256 |
|---|---|
| `usbc_origin_check-gerbers.zip` | `abbf97f2f9568f711ad38d8c470403fbfc3c9e4ea4d8d111dc1dcade375807dc` |
| `usbc_origin_check-lcsc-bom.csv` | `278d10f56316d8e226f756b8d52ec9dd9a8363984284b727d9cced1b54b844f8` |
| `usbc_origin_check-lcsc-cpl.csv` | `ee35b63bbfc510fff2925d37a452311757fc3a027439de9075e9cae045aa2c5a` |

## Screenshots

`top.png` is the top view and `bottom.png` the view from below, mirrored left to right. Both are the assembler's whole-board captures at 3068 × 1801 px, about 39.5 px/mm.

## Method

`measurements.json` is the output of `tools/frame_boards/measure_preview.py measure`, as for board 1.

For the receptacle, `measure_preview.py pegs` compares the preview's two magenta peg dots with our non-plated peg holes, which the other view shows through the board. Both go through the same outline mapping, so the outline calibration cancels.
- `pegs.jsonl` holds this board's readings.
- `pegs-board-1.jsonl` holds the same comparison run on board 1's screenshots, which had no map.

The table gives each dot's offset from the nearest hole along the footprint's local y axis, the mating axis. "Needed" is the map `dy` that would put the dot on the hole.

| Copy | This board (map −1.55) | Needed | Board 1 (no map) | Needed |
|---|---|---|---|---|
| top 0° | −0.20, −0.18 | −1.35, −1.37 | +1.51, +1.53 | −1.51, −1.53 |
| top 37° | −0.12, −0.11 | −1.43, −1.44 | +1.42, +1.45 | −1.42, −1.45 |
| bottom 0° | −0.09, −0.12 | −1.46, −1.43 | +1.56, +1.58 | −1.56, −1.58 |
| bottom 37° | −0.13, −0.12 | −1.42, −1.43 | +1.53, +1.55 | −1.53, −1.55 |
| mean | −0.13 | −1.42 | +1.52 | −1.52 |

Along x, every reading is within 0.2 mm of the hole, and the sign follows the mirrored view.

## Results

- **USB-C, C165948.** With `at (0mm, -1.55mm)` applied, every peg dot on all four copies sits within 0.2 mm of our hole, against 1.4–1.6 mm with no map. The two previews put the needed offset between −1.35 and −1.58 mm, with means of −1.42 and −1.52. The 0.1 mm between those means is the method's resolution, so the map stays at −1.55 mm. Row `usbc-origin-c165948` records −1.42, this preview's reading.
- **C2146 on transistor 0.2.0, no map.** Pin 1 reads 9° and 14° on the top and 345° on the bottom: rotate 0 within the dot scatter. The footprint's `rotate 90` move holds.
- **C97521 on soic 0.3.0, no map.** Pin 1 reads 1° and 2° on the top and 355° on the bottom: rotate 0. The footprint's `rotate 270` move holds.
- **Bottom side.** As on board 1, each bottom reading is the mirror image of its top reading, and each 37° copy reads like its 0° copy.
