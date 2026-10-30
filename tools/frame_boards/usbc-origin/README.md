# USB-C origin check board

C2 frame board 1 measured the supplier origin of the USB-C receptacle C165948 at (0, −1.54) in our footprint's local frame (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`). This board applies the map `at (0mm, -1.55mm)` and asks the assembler's preview to confirm it before the map is relied on.

It holds three parts, each placed four times: top 0°, top 37°, bottom 0° and bottom 37°.

| Row | Part | Footprint | Map | Checks |
|---|---|---|---|---|
| USBC | C165948 | `usb::connectors::type_c::FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12` (usb 0.1.5) | `at (0mm, -1.55mm)` | The supplier origin. The preview's locating pegs should sit on our peg holes. |
| SOT23_S8050 | C2146 | `transistor::FP_JSCJ_SOT23` (transistor 0.2.0) | none | The footprint moved by `rotate 90`; pin 1 should sit on our pin 1. |
| SOIC8_208 | C97521 | `soic::SOIC8P127X790X216N` (soic 0.3.0) | none | The footprint moved by `rotate 270`; pin 1 should sit on our pin 1. |

The two controls check the library-plan W3 move end to end, on the moved footprints with no map.

## Regenerate

```sh
python3 -I tools/jlc_frame_board.py tools/frame_boards/usbc-origin/manifest.json -o BOARD \
  --cohdl target/debug/cohdl --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli
```

The manifest's rows copy board 1's, with their own placements and the USB-C map.

## Checks before upload (2026-10-30)

- The compiler's supplier CPL and `smt_pos.py --map` on the board are byte-identical.
- The USB-C rows of the supplier CPL sit 1.55 mm from the neutral CPL rows, towards the footprint's local −y, at 0° and at 37° on both sides.

Upload for preview only and record the result with the screenshots. Do not order.
