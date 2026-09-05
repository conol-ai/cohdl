# Microphone source manifest

Retrieved and audited on 2026-07-30. The PDF is stored byte-for-byte as
downloaded; it was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `tdk-invensense-ics-43434.pdf` | `ICS-43434` | InvenSense DS-000069 Revision 1.2, released 2016-08-29 | <https://invensense.tdk.com/wp-content/uploads/2016/02/DS-000069-ICS-43434-v1.2.pdf> | `5f940de70ac517f541528d1ad5bb9a5cb4326fe6dfc1228e000cd1b3aed9d927` |
| `ics-40300.evidence.md` | `ICS-40300` | InvenSense DS-ICS-40300-00 Revision 1.3, released 2018-07-17 | <https://admin-uat.invensense.com/sites/default/files/2026-01/DS-ICS-40300-00-v1.3.pdf> | `65073ca752d946f2bdd13566578846d9bc58a0998dfb1d4c6ae161b05f6519c4` |

## Retrieval and lifecycle notes

TDK's current product page,
<https://www.invensense.tdk.com/en-us/products/microphone/ics-43434>, lists
DS-000069 Version 1.2 and marks the product EOL. The historical official PDF
endpoint redirected to protected HTML during automated retrieval. The local
manufacturer-authored Version 1.2 PDF was retrieved from authorized
distributor Mouser:
<https://www.mouser.cn/datasheet/2/400/ds_000069_ics_43434_v1_2-2581173.pdf>.
Both URLs are recorded for reproducibility.

## Coverage and geometry

- Page 10 supplies the complete terminal-side-down top-view map: 1 WS, 2 LR,
  3 GND, 4 SCK, 5 VDD, and 6 SD.
- Page 17 supplies the 1:1 PCB land pattern: five 0.600 x 0.522 mm lands,
  0.900 mm column pitch, 0.822 mm row pitch, and a ground annulus with
  1.625 mm outer and 1.025 mm inner diameters.
- Page 17 recommends a PCB sound hole of at least 0.5 mm. The footprint uses
  the annulus's 1.025 mm inner diameter as a coincident non-plated aperture,
  which exceeds that minimum and is representable in CoHDL.
- CoHDL cannot currently express Figure 14's separately segmented solder-paste
  stencil for the ground annulus. The emitted footprint therefore needs a
  fabrication-level stencil review to keep paste out of the acoustic path.
- Page 19 supplies the 3.50 x 2.65 mm package outline and pin-1 corner.
- No separate manufacturer CAD file was used; the footprint is derived from
  the dimensioned manufacturer land-pattern and package drawings.

The PDF passed `pdfinfo`, Poppler text extraction, and full-page rendering.
Pages 10, 17, and 19 were visually inspected for identity, pin map, package
orientation, sound aperture, and land dimensions.
