# ESD source manifest

Retrieved and revalidated on 2026-07-30; `h5vl10b.evidence.md` added 2026-09-04.

Entries ending in `.evidence.md` are repository-authored evidence records.
Their SHA-256 values identify upstream manufacturer PDFs that are not
redistributed here.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `st-usblc6-2-datasheet.pdf` | STMicroelectronics `USBLC6-2SC6` | `DS4260` Rev 7, 2021-12-24 | [STMicroelectronics](https://www.st.com/resource/en/datasheet/usblc6-2.pdf) | `bc30154f310cd631043214ed52571daefe04270587ec55072d49a23bd18b9068` |
| `tpd1e05u06.evidence.md` | Texas Instruments `TPD1E05U06DPYR` | `SLVSBO7O`, revised 2024-08 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/tpd1e05u06.pdf) | `c167cf1e72a5473a4d2c59b6a3c0251498701da05b7785919b9ceaae3b3e02c6` |
| `tpd1e1b04.evidence.md` | Texas Instruments `TPD1E1B04DPYR` | `SLVSDL0A`, revised 2016-07 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/tpd1e1b04.pdf) | `f32ea1cd8859cb177c75b6eed470012971a1abd6fbc93907fd12820012ec5d29` |
| `esd122.evidence.md` | Texas Instruments `ESD122DMYR` | `SLVSDP5A`, revised 2018-08 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/esd122.pdf) | `16ba54c64476aa5abbadc84ce72e60cee6baf710c958be0aad90cf363393f3ab` |
| `h5vl10b.evidence.md` | Zhuhai Hongjiacheng `H5VL10B` | `Rev 2.0`, 3 pages | Zhuhai Hongjiacheng Technology Co., Ltd (retrieved via LCSC C7420372) | `0d0aaef1b2af0c641cfc5b2135b7242b346aa0a792611d7dcd0fbefd8bf58048` |

## Footprint and pin audit

- The top-view pin map is 1/6 = I/O1, 3/4 = I/O2, 2 = GND, and 5 = VBUS;
  the device model matches it in both directions.
- `FP_SOT_23_6` is rotated 90 degrees from Figure 19 without reflection. It
  uses ST's 0.60 x 1.20 mm lands, 0.95 mm lead pitch, and 2.30 mm row-center
  spacing. Pin 1 remains the upper-left lead in top view.
- ST also publishes CAD models from the
  [USBLC6-2 product page](https://www.st.com/en/protections-and-emi-filters/usblc6-2.html);
  the datasheet's dimensioned Figure 19 is the geometry authority used here.
- `FP_X1SON_DPY_2` follows TI drawing DPY0002A: 0.30 x 0.50 mm lands on
  0.70 mm pitch. It is shared by the two exact DPYR parts above.
- `FP_X2SON_DMY_3` follows TI drawing DMY0003A: three 0.20 mm circular lands,
  with 0.25 mm horizontal and 1.00 mm vertical center spacing.
- The pin models preserve TI's top-view numbering: TPD1E05U06 pin 2 is GND;
  TPD1E1B04 has two equivalent I/O terminals; ESD122 pin 1 is GND and pins
  2/3 are IO1/IO2.
- `FP_Hongjiacheng_DFN1006_2L` follows the H5VL10B Rev 2.0 suggested pad
  layout (p3): lands J 0.50-0.60mm long x M 0.55-0.65mm wide with a K
  0.25-0.35mm gap, nominal 0.55 x 0.60mm lands centred 0.85mm apart, inside
  the DFN1006-2L body outline (A 0.95-1.05 x B 0.55-0.65mm). H5VL10B is a
  symmetric bidirectional protector, so pins 1/2 are interchangeable
  electrically; the datasheet function diagram (1 PB 2) fixes the numbering.

## H5VL10B retrieval notes

- The manufacturer document (footer: Zhuhai Hongjiacheng Technology co., Ltd)
  was retrieved from the LCSC listing C7420372, whose `productModel` is the
  exact string `H5VL10B` with package DFN1006-2L. LCSC shows the distributor
  brand string `R+O` on that listing; the datasheet footer is the
  manufacturer identity used for the part.
- TDSEMIC's `H5VL10B-TD` (LCSC C51901858) is a different order code from a
  different vendor and is NOT this part; the accepted exact identity is the
  exact string `H5VL10B`.
