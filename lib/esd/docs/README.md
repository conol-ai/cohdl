# ESD source manifest

Retrieved and revalidated on 2026-07-30.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `st-usblc6-2-datasheet.pdf` | STMicroelectronics `USBLC6-2SC6` | `DS4260` Rev 7, 2021-12-24 | [STMicroelectronics](https://www.st.com/resource/en/datasheet/usblc6-2.pdf) | `bc30154f310cd631043214ed52571daefe04270587ec55072d49a23bd18b9068` |

## Footprint and pin audit

- The top-view pin map is 1/6 = I/O1, 3/4 = I/O2, 2 = GND, and 5 = VBUS;
  the device model matches it in both directions.
- `FP_SOT_23_6` is rotated 90 degrees from Figure 19 without reflection. It
  uses ST's 0.60 x 1.20 mm lands, 0.95 mm lead pitch, and 2.30 mm row-center
  spacing. Pin 1 remains the upper-left lead in top view.
- ST also publishes CAD models from the
  [USBLC6-2 product page](https://www.st.com/en/protections-and-emi-filters/usblc6-2.html);
  the datasheet's dimensioned Figure 19 is the geometry authority used here.

