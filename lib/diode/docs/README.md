# Diode source manifest

Retrieved and revalidated on 2026-07-30.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `nexperia-pmeg6010elr-datasheet.pdf` | Nexperia `PMEG6010ELR` | Version 5, 2023-01-01 | [Nexperia](https://assets.nexperia.com/documents/data-sheet/PMEG6010ELR.pdf) | `c0eb3503d558ac894117e0dc520c33b0fe4e832f98602b6f56dfc4fc9538f112` |
| `diodes-inc-bav16w-1n4148w-datasheet.pdf` | Diodes Incorporated `1N4148W-7-F` | `DS30086` Rev 31-2, 2024-09-02 | [Diodes Incorporated](https://www.diodes.com/assets/Datasheets/BAV16W_1N4148W.pdf) | `39c16a6888bdab22418e93e17182174aad763a66957a4632e70c944194e3fc08` |

## Footprint and pin audit

- The Nexperia pin table assigns pin 1 to cathode and pin 2 to anode. The
  previous model was reversed and is corrected. `FP_D_SOD_123W` uses Figure
  17's 1.20 x 1.20 mm lands on 2.80 mm center spacing and encloses the
  published 4.4 x 2.1 mm occupied area.
- Diodes Incorporated identifies the cathode by the package band; the model
  keeps cathode on pin 1. `FP_D_SOD_123` now follows the official suggested
  layout exactly: X = 0.90 mm, Y = 0.95 mm, X1 = 4.05 mm, giving 3.15 mm
  center spacing.
- Product-status references are the official
  [PMEG6010ELR page](https://www.nexperia.com/product/PMEG6010ELR) and
  [1N4148W page](https://www.diodes.com/part/view/1N4148W).

