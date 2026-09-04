# Cellular module source manifest

| Local file | Coverage | Document | Source URL | Retrieval | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| `air780e-hw.pdf` | `Air780E` | Luat (Hezhou) hardware design manual V1.0 (109-pin pinout, electrical specs) | https://docs.openluat.com/air780e/hardware/design/ | 2026-07-29 | `d792cb1b1e380a7cac2fd3e4d1d361b1b5b2fcd356d7ca33cb19188acb6f15e9` |
| `Air780E_GPIO_table.pdf` | `Air780E` GPIO function table | Luat (Hezhou) official GPIO table | https://docs.openluat.com/air780e/ | 2026-07-29 | `4ec4d57a2e8193fc74a02353f2b733c80622c2bbe3ba7921e58bc63e1078e328` |
| `Air780E_AT_PADS_Decal_V20241030.zip` | `Air780E` PADS decal (AT variant) | Luat (Hezhou) official PADS library, 109 pads | https://docs.openluat.com/air780e/ | 2026-07-29 | `14c6bcd01fa8aa51e78fd5d66e8d6637aa5cde3e82613807a211dcb4d6cb50dc` |
| `Air780E_LuaOS_PADS_Decal_V20241030.zip` | `Air780E` PADS decal (LuaOS variant) | Luat (Hezhou) official PADS library, 109 pads | https://docs.openluat.com/air780e/ | 2026-07-29 | `291a308036c1afc6de19d61ddd1e1006d6044f1537496005e65b416896fd6dc9` |
| `air780eg-hw.pdf` | `Air780EG` | Luat (Hezhou) hardware design manual V1.1.2, 109-pin pinout and recommended PCB land drawing | https://docs.openluat.com/air780eg/product/ | 2026-09-03 | `0af5b7b7c1066ae74cc8edfa85e06db7a6650497b8af3a2c1fe0aa71b60f41f9` |

FULL 109-pin device (`Air780E_SOC`): all 109 pads placed from the official
PADS decal pad coordinates. Known approximation: padstack sizes are unified
to 1.2×0.6 / 0.6×1.2 mm; the official PADS decal uses per-pin padstack
lengths of ~1.1-2.1 mm. **Re-verify each padstack against the official decal
before board fabrication.** The 109-pin pinout authority table lives in
`/Users/adan/adan-ws/cohdl-doc/evidence/conol-watch/air780e-pinout.md`.

## T520 polymer tantalum (VBAT bulk)

| Local file | Coverage | Document | Source URL | Retrieval | SHA-256 |
| --- | --- | --- | --- | --- | --- |
| (datasheet at KEMET) | `C_T520_100u_10V_D` = KEMET `T520D107M010ATE070` | KEMET T520 series datasheet, 100µF 10V D-case 7343-31, ESR 70mΩ | https://www.kemet.com/en/us/technical-resources/selector-guides/t520-t530-t550-t560-family.html | 2026-08-05 | vendor site (datasheet not bundled; MPN from KEMET T520 ordering scheme, ESR 070 = 70mΩ) |

VBAT 100µF polymer tantalum replaces the MLCC placeholder per design note
§13.2: pure MLCC on VBAT resonates and can drop the module on RF bursts.
