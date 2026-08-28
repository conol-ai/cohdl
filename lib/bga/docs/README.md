# BGA land-pattern provenance

## Source document

Both footprints are transcribed from STMicroelectronics DS9826 Rev 6,
2019-09-03, for the STM32F072x8/STM32F072xB family.

- Official document URL:
  <https://www.st.com/resource/en/datasheet/stm32f072cb.pdf>
- Audited document SHA-256:
  `660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453`.
- Package-map cross-check: STMicroelectronics `STM32_open_pin_data`,
  commit `7d1f1514ed5583ec5007ad91236b4e1d377295b1`
  (STM32CubeMX-DB.6.0.180), files `STM32F072RBHx.xml` and
  `STM32F072V(8-B)Hx.xml`.
- Pinned source URL:
  <https://github.com/STMicroelectronics/STM32_open_pin_data/tree/7d1f1514ed5583ec5007ad91236b4e1d377295b1>

The document itself remains in the STM32 component package rather than being
duplicated here. This package records its immutable checksum so the geometry
can be audited independently.

## `BGA64C50P8X8_500X500X60N`

- Package: UFBGA64, 64 collapsible solder balls, 5 x 5 mm nominal body,
  0.50 mm pitch, and 0.60 mm maximum height.
- Geometry authority: Figure 41 and Table 74 (package outline and dimensions);
  Figure 42 and Table 75 (recommended footprint and PCB design rules).
- Ball map: full 8 x 8 array, rows A through H and columns 1 through 8.
- Recommended copper land: 0.280 mm diameter.
- Covered audited identity: `STM32F072RBH6`, including its checked
  `STM32F072RBH6TR` packing alternate.
- Courtyard: 5.65 x 5.65 mm, covering the 5.15 mm maximum body by 0.25 mm on
  every side.

## `BGA100C50P12X12_700X700X60N`

- Package: UFBGA100, 100 collapsible solder balls on a sparse 12 x 12 grid,
  7 x 7 mm nominal body, 0.50 mm pitch, and 0.60 mm maximum height.
- Geometry authority: Figure 35 and Table 71 (package outline, exact ball map,
  and dimensions); Figure 36 and Table 72 (recommended footprint and PCB
  design rules).
- Ball map: rows A, B, L, and M are full; rows C and K contain columns
  1-5 and 8-12; rows D, E, H, and J contain columns 1-3 and 10-12; rows F
  and G contain columns 1-2 and 11-12.
- Recommended copper land: 0.280 mm diameter.
- Covered audited identities: `STM32F072V8H6`, `STM32F072VBH6`, and
  `STM32F072VBH7`, including the checked `STM32F072VBH6TR` packing alternate.
- Courtyard: 7.65 x 7.65 mm, covering the 7.15 mm maximum body by 0.25 mm on
  every side.

The package drawings show bottom views with column 1 at the right and their
top-view A1 index areas at the upper-left. The PCB land patterns are top views:
A1 is therefore at the upper-left, columns increase to the right, and rows
increase downward in CoHDL/KiCad's +Y-down frame. The sparse UFBGA100 map was
also checked ball-for-ball against the pinned ST pin data above.

DS9826 recommends a 0.370 mm typical solder-mask opening, a 0.280 mm stencil
opening, and 0.100-0.125 mm stencil thickness for both packages; it additionally
recommends a 0.100 mm trace width for UFBGA64. CoHDL's current pad declaration
expresses the official copper land but cannot express independent solder-mask
or paste apertures, stencil thickness, or trace width. Those values remain
fabrication requirements for the board layout and stencil process.
