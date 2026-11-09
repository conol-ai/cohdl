# Cellular module source manifest

Entries ending in `.evidence.md` are repository-authored evidence records.
Their SHA-256 values identify the upstream manufacturer PDF, which is not
redistributed here.

| Local file | Coverage | Document | Source URL | Retrieval | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `air780e-hw.pdf` | `Air780E` | Luat (Hezhou) hardware design manual V1.0 (109-pin pinout, electrical specs) | https://docs.openluat.com/air780e/hardware/design/ | 2026-07-29 | `d792cb1b1e380a7cac2fd3e4d1d361b1b5b2fcd356d7ca33cb19188acb6f15e9` |
| `Air780E_GPIO_table.pdf` | `Air780E` GPIO function table | Luat (Hezhou) official GPIO table | https://docs.openluat.com/air780e/ | 2026-07-29 | `4ec4d57a2e8193fc74a02353f2b733c80622c2bbe3ba7921e58bc63e1078e328` |
| `Air780E_AT_PADS_Decal_V20241030.zip` | `Air780E` PADS decal (AT variant) | Luat (Hezhou) official PADS library, 109 pads | https://docs.openluat.com/air780e/ | 2026-07-29 | `14c6bcd01fa8aa51e78fd5d66e8d6637aa5cde3e82613807a211dcb4d6cb50dc` |
| `Air780E_LuaOS_PADS_Decal_V20241030.zip` | `Air780E` PADS decal (LuaOS variant) | Luat (Hezhou) official PADS library, 109 pads | https://docs.openluat.com/air780e/ | 2026-07-29 | `291a308036c1afc6de19d61ddd1e1006d6044f1537496005e65b416896fd6dc9` |
| `air780eg-hw.evidence.md` | `Air780EG` | Luat (Hezhou) hardware design manual V1.1.2, 109-pin pinout and recommended PCB land drawing | https://docs.openluat.com/air780eg/product/ | 2026-09-03 | `0af5b7b7c1066ae74cc8edfa85e06db7a6650497b8af3a2c1fe0aa71b60f41f9` |

FULL 109-pin device (`AIR780E`): all 109 pads of
`FP_Luat_AIR780E_LGA109_177x158mm` are generated from the official PADS decal
(`Air780E_AT.asc`, decal `AIR780E`; the LuatOS decal is identical) with y
negated into CoHDL's +y-down board frame, so pin 1 is top-left by the chamfer
as in the manual's pin view (pp. 13, 16) and recommended land (p. 20). Copper
sizes are the decal's top-copper (layer −2) stacks: 1.3 × 0.6 mm perimeter
pads, 1.0 × 0.7 mm inner pads (0.7 × 1.0 mm in the inner rows) and 1.3 mm
square centre ground pads. The decal's larger figures are not copper: its
solder-mask layer (121) is 0.05 mm per side larger on every pad (up to
1.4 mm), and its paste layer (123) opens the perimeter pads at
0.7 × 1.4 mm. Neither mask nor paste is modelled. The 109-pin pinout
authority table lives in
`/Users/adan/adan-ws/cohdl-doc/evidence/conol-watch/air780e-pinout.md`.

## Air780E footprint correction in 0.2.0 — 2026-11-07

`FP_Luat_AIR780E_LGA109_177x158mm`, used by `CELL_AIR780E` and
`CELL_AIR780EG`, was mirrored top to bottom in 0.1.x. It copied the PADS
decal's +y-up coordinates unchanged into CoHDL's +y-down frame, which put
pin 1 at the lower left, numbered the pins clockwise seen from the top, and
put pad 99 at the bottom of the left inner column. No rotation of that land
matches the module, so a board built on it cannot be assembled. Version 0.2.0
regenerates every pad from the decal with y negated, checked pad by pad
against the decal and against the manual's top view (pp. 13, 16, 20). It also
corrects pad shapes to the decal's copper stacks:

- pads 89–94 are 1.3 mm squares (decal shape `S`), not circles;
- pads 54–58, 68–72, 101, 102 and 105–109 are 0.7 × 1.0 mm (long axis along
  y), not 1.0 × 0.7 mm;
- the corner pads 95–98 are 0.6 × 1.3 mm like the rest of their rows, not
  1.0 × 0.7 mm.

Pad symbol `P_AIR780E_PAD_ROUND` is removed; `P_AIR780E_PAD_IN_H` and
`P_AIR780E_PAD_GND` are new. Pin numbers, the device pin maps and the
footprint name are unchanged, but every pad off the horizontal centre line
moves: a design that used 0.1.x must re-route the module after
`cohdl update`. `check` and `build` keep passing, so the change will not show
up as an error.

## T520 polymer tantalum (VBAT bulk)

| Local file | Coverage | Document | Source URL | Retrieval | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| (datasheet at KEMET) | `C_T520_100u_10V_D` = KEMET `T520D107M010ATE070` | KEMET T520 series datasheet, 100µF 10V D-case 7343-31, ESR 70mΩ | https://www.kemet.com/en/us/technical-resources/selector-guides/t520-t530-t550-t560-family.html | 2026-08-05 | vendor site (datasheet not bundled; MPN from KEMET T520 ordering scheme, ESR 070 = 70mΩ) |

VBAT 100µF polymer tantalum replaces the MLCC placeholder per design note
§13.2: pure MLCC on VBAT resonates and can drop the module on RF bursts.
