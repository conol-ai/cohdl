# QFP land-pattern provenance

## `QFP50P900X900X160_48N`

- Package: LQFP-48, 7 x 7 mm body, 0.50 mm pitch, 1.60 mm maximum
  height, 9 x 9 mm nominal lead span.
- Geometry authority: STMicroelectronics DS9826 Rev 6, Figure 50,
  "Recommended footprint for LQFP48 package."
- Covered audited identities: `STM32F072C8T6`, `STM32F072C8T7`,
  `STM32F072CBT6`, and `STM32F072CBT7`, including their checked packing
  alternates where published.
- Official document URL:
  <https://www.st.com/resource/en/datasheet/stm32f072cb.pdf>
- Document revision and date: DS9826 Rev 6, 2019-09-03.
- Audited document SHA-256:
  `660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453`.
- Geometry transcribed from the source: 1.20 x 0.30 mm lands, 0.50 mm
  pitch, 9.70 mm outside land span, and 5.80 mm tangential land-row span.

The 10.20 x 10.20 mm courtyard extends 0.25 mm beyond the outer copper
envelope on every side.

## `QFP50P1200X1200X160_64N`

- Package: LQFP-64, 10 x 10 mm body, 0.50 mm pitch, 1.60 mm maximum
  height, 12 x 12 mm nominal lead span.
- Geometry authority: STMicroelectronics DS9826 Rev 6, Figure 45,
  "Recommended footprint for LQFP64 package."
- Covered audited identities: `STM32F072R8T6`, `STM32F072R8T7`,
  `STM32F072RBT6`, and `STM32F072RBT7`, including their checked packing
  alternates where published.
- Official document URL:
  <https://www.st.com/resource/en/datasheet/stm32f072cb.pdf>
- Document revision and date: DS9826 Rev 6, 2019-09-03.
- Audited document SHA-256:
  `660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453`.
- Geometry transcribed from the source: 1.20 x 0.30 mm lands, 0.50 mm
  pitch, 12.70 mm outside land span, and 7.80 mm tangential land-row span.

The 13.20 x 13.20 mm courtyard extends 0.25 mm beyond the outer copper
envelope on every side.

## `QFP50P1600X1600X160_100N`

- Package: LQFP-100, 14 x 14 mm body, 0.50 mm pitch, 1.60 mm maximum
  height, 16 x 16 mm nominal lead span.
- Geometry authority: STMicroelectronics DS9826 Rev 6, Figure 39,
  "Recommended footprint for LQFP100 package."
- Covered audited identities: `STM32F072V8T6` and `STM32F072VBT6`, including
  the checked `STM32F072VBT6TR` packing alternate.
- Official document URL:
  <https://www.st.com/resource/en/datasheet/stm32f072cb.pdf>
- Document revision and date: DS9826 Rev 6, 2019-09-03.
- Audited document SHA-256:
  `660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453`.
- Geometry transcribed from the source: 1.20 x 0.30 mm lands, 0.50 mm
  pitch, 16.70 mm outside land span, and 12.30 mm tangential land-row span.

The 17.20 x 17.20 mm courtyard extends 0.25 mm beyond the outer copper
envelope on every side.

All three source figures are package top views. The declarations rotate each
view as a whole into `qfp`'s pin-1-at-upper-left convention; they never mirror
it, so numbering chirality is preserved in CoHDL/KiCad's +Y-down frame.

The footprint is reusable package geometry and therefore lives in `qfp`, not
in the STM32 component package. Device and exact-part datasheets remain with
their component libraries because `#[doc(...)]` paths are package-relative.
