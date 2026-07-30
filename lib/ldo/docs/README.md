# LDO source manifest

Retrieved and audited on 2026-07-30. The PDF is stored byte-for-byte as
downloaded; no PDF was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `diodes-ap2112.pdf` | `AP2112K-3.3TRG1` | Diodes Incorporated AP2112 datasheet, document DS39724 Rev. 2-2, June 2017 | <https://www.diodes.com/datasheet/download/AP2112.pdf> | `ef8d376f2ec356e29172eb9e053819a0ebdcc576dba7fc9ab0505c568427920f` |

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
