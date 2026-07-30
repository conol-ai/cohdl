# MOSFET source manifest

Retrieved and audited on 2026-07-30. The PDF is stored byte-for-byte as
downloaded; it was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `diodes-dmg1012t.pdf` | `DMG1012T-7`, `DMG1012T-13` | Diodes Incorporated DMG1012T datasheet, DS31783 Rev. 8-2, February 2022 | <https://www.diodes.com/datasheet/download/DMG1012T.pdf> | `2f226aaee67a900773753a79c93d565820d01a300a4b1b7409fd07072da8aa40` |

## Coverage and geometry

- Page 1 supplies the top-view terminal map and the two exact tape-and-reel
  ordering codes: pin 1 gate, pin 2 source, and pin 3 drain.
- Page 6 supplies the SOT523 outline and suggested pad layout. The footprint
  is a 90-degree rotation of that drawing and preserves the official
  0.40 x 0.51 mm lands, 1.29 mm row separation, and 1.00 mm lead pitch.
- The courtyard covers both the maximum package outline and every copper land;
  the footprint includes an explicit pin-1 mark.
- No separate manufacturer CAD file was used; the footprint is derived from
  the dimensioned manufacturer land-pattern drawing.

The PDF passed `pdfinfo`, Poppler text extraction, and full-page rendering.
Pages 1 and 6 were visually inspected for identity, exact ordering codes,
terminal map, package orientation, and land dimensions.
