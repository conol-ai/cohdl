# LDO source manifest

Retrieved and audited on 2026-07-30. The PDF is stored byte-for-byte as
downloaded; no PDF was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `diodes-ap2112.pdf` | `AP2112K-3.3TRG1` | Diodes Incorporated AP2112 datasheet, document DS39724 Rev. 2-2, June 2017 | <https://www.diodes.com/datasheet/download/AP2112.pdf> | `ef8d376f2ec356e29172eb9e053819a0ebdcc576dba7fc9ab0505c568427920f` |
| `ti-lp5912.pdf` | `LP5912-3.3DRVR`, `LP5912-1.8DRVR`, `LP5912-5.0DRVR` | Texas Instruments LP5912, SNVSA77D, December 2015, revised November 2016, 38 pages | <https://www.ti.com/lit/ds/symlink/lp5912.pdf> (retrieved 2026-11-25) | `4266eef05203d0d90b7824cd7445df5a857bf95922df68274aef4a6f8c122d88` |
| `ti-tlv755p.pdf` | `TLV75533PDRVR`, `TLV75512PDRVR`, `TLV75525PDRVR` | Texas Instruments TLV755P, SBVS320D, November 2017, revised September 2024, 51 pages | <https://www.ti.com/lit/ds/symlink/tlv755p.pdf> (retrieved 2026-11-25) | `f9e676918d1af7144e55835616108def097b48fb5784bd0084ae77a418c64fbf` |

## Coverage and geometry

- The ordering table covers the exact fixed-3.3-V tape-and-reel MPN.
- Page 1 supplies the SOT25 top-view pin map: 1 VIN, 2 GND, 3 EN, 4 NC,
  and 5 VOUT.
- Page 14 supplies both the package outline and the suggested pad layout.
  `SOT5P95X290X160N` uses the specified 0.55 x 0.80 mm lands, 0.95 mm
  pitch, 2.40 mm row-centre separation, and an explicit pin-1 mark.
- No separate manufacturer CAD file was used; the footprint is derived from
  the dimensioned manufacturer land-pattern drawing.

The PDF passed `pdfinfo`, Poppler text extraction, and full-page rendering.
Pages 1 and 14 were visually inspected for identity, pin map, package
orientation, and land dimensions.

## RT9193 (Richtek, TSOT-23-5)

| Local file | Covered item | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `rt9193.pdf` | `LDO_RT9193_15PB` / `LDO_RT9193_28PB` | DS9193-18, June 2022 | [Richtek](https://www.richtek.com) | `bbab3a6dcb12085e8b2988d10d25f34a9fad4b3a5e1cdbcdb7b4314efc55f43f` |

BP pin requires 22nF (or larger) noise-bypass cap per DS9193 p2; OUT needs
1uF per p3 test conditions. Suffix PB = auto discharge + TSOT-23-5.

## Application contract — added 2026-10-16

[AP2112K-3.3TRG1 application contract](ap2112-application.md) summarizes the
pinned source's limits, capacitors, enable/discharge behavior and power-isolation
limitations. It is attached to the part and device for exact registry lookup.

## LP5912 family (ldo 0.1.3)

Added 2026-11-25. A family is modelled as one device per pin table and
one part per stocked member; a fixed output voltage is part of what the
OUT pin is, so each voltage is its own device.

| Family | Device | Output | Member included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|---|
| LP5912, WSON-6 (DRV) | `LP5912_3V3` | 3.3 V | `LDO_LP5912_3V3`: `LP5912-3.3DRVR`, C524780, 27,200 |
| LP5912, WSON-6 (DRV) | `LP5912_1V8` | 1.8 V | `LDO_LP5912_1V8`: `LP5912-1.8DRVR`, C2876234, 5,133 |
| LP5912, WSON-6 (DRV) | `LP5912_5V0` | 5.0 V | `LDO_LP5912_5V0`: `LP5912-5.0DRVR`, C2761351, 2,638 |

- **Pins read.** p3, "Pin Functions": OUT 1, NC 2, PG 3, EN 4, GND 5,
  IN 6, exposed thermal pad. NC may be left open or grounded; PG needs an
  external pull-up and is `optional`; EN has an internal 3 MOhm
  pull-down; the pad is to be grounded or left floating and is the
  `optional` pin `PAD` on pad 7. Checked from the table to the model and
  back.
- **Values used.** VIN 1.6 V to 6.5 V, IOUT up to 500 mA (p4); fixed
  outputs from 0.8 V to 5.5 V (section 5, p3).
- **Left out.** `LP5912-1.5DRVR`, `-1.2DRVR`, `-2.8DRVR`, `-3.0DRVR`,
  `-0.9DRVR`, `-1.0DRVR` and the `DRVT` reels: under 400 pieces each in
  the snapshot. Adding one is a new device with its `output_voltage` and
  one part; the pin table and land do not change.

## TLV755P family (ldo 0.1.3)

Added 2026-11-25. Table 4-1 (p3) gives each package its own pin numbers,
so each package is a pin table; only the DRV table is modelled.

| Family | Device | Output | Member included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|---|
| TLV755P, WSON-6 (DRV) | `TLV75533P_DRV` | 3.3 V | `LDO_TLV75533PDRVR`: `TLV75533PDRVR`, C2861750, 15,654 |
| TLV755P, WSON-6 (DRV) | `TLV75512P_DRV` | 1.2 V | `LDO_TLV75512PDRVR`: `TLV75512PDRVR`, C2866903, 1,528 |
| TLV755P, WSON-6 (DRV) | `TLV75525P_DRV` | 2.5 V | `LDO_TLV75525PDRVR`: `TLV75525PDRVR`, C544744, 1,202 |

- **Pins read.** p3, Figure 4-4 and the DRV column of Table 4-1: OUT 1,
  NC 2 and 5, GND 3, EN 4, IN 6; "The thermal pad is internally connected
  to GND", so pad 7 is listed with pin 3 under `GND`. Checked from the
  table to the model and back.
- **Values used.** Absolute maximum supply 6.0 V (p4); 1 uF or larger on
  IN and on OUT (p3).
- **Not the LP5912.** Both families use the same land, but pin 3 is GND
  here and PG there, and pin 5 is NC here and GND there. They are two
  families and never alternates.
- **Left out.** The DBV (SOT-23-5), DQN (X2SON-4) and DYD members are
  other pin tables on other lands. `TLV75533PDBVR` (C404027) has the
  deepest stock of the family, 116,994 in the snapshot; it needs a
  `TLV75533P_DBV` device and the DBV0005A land of pp44-46, which this
  package does not have. DRV members under 500 pieces (`TLV75530PDRVR`,
  `TLV75518PDRVR`, `TLV75515PDRVR`) are also left out.

## Land and LCSC part numbers for both families

All six parts use `son::TI_DRV0006A` (`son` 0.1.0, a new dependency),
TI drawing 4222173/D, printed on LP5912 pp35-37 and TLV755P pp48-50; see
`lib/son/docs/README.md` for the land, its coordinates and its paste
apertures.

No part here carries a `supplier lcsc` clause: a binding needs a row in
`tools/sourcing_data/lcsc_map.json` and a frame row with reviewed evidence
in `tools/sourcing_data/jlc_frames.json`, and neither exists. The candidate
numbers and stock are from the local jlcparts snapshot of 2026-11-25, which
has no manufacturer field; the part pages were not read. The preview board
input and build for these candidates is `tools/frame_boards/demand-batch-2/`;
it has not been previewed.
