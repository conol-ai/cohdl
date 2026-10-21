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
  a separate, concentric 0.5 mm non-plated aperture, leaving 0.2625 mm between
  the hole edge and the copper annulus's inner edge.
- Figure 14 supplies the solder-paste apertures: five 0.500 x 0.422 mm
  rectangles and a four-segment annulus with 1.625 mm outer / 1.125 mm inner
  diameters and 0.100 mm gaps. The footprint encodes both independently of
  copper using `paste:` overrides.
- Page 19 supplies the 3.50 x 2.65 mm package outline and pin-1 corner.
- No separate manufacturer CAD file was used; the footprint is derived from
  the dimensioned manufacturer land-pattern and package drawings.

The PDF passed `pdfinfo`, Poppler text extraction, and full-page rendering.
Pages 10, 17, and 19 were visually inspected for identity, pin map, package
orientation, sound aperture, and land dimensions.

## ICS-43434 footprint correction in 0.1.2

Re-audited page 17 on 2026-10-21 following a reported PCB DRC blocker.
Earlier versions modeled pin 3 as a solid 1.625 mm SMD disk over a 1.025 mm
NPTH. A coincident drill does not subtract copper from the SMD pad in CAD:
KiCad 10.0.4 reports zero hole-to-copper clearance and a solder-mask bridge.
The 1.025 mm dimension is the copper opening; the datasheet specifies the
PCB sound-hole minimum separately. The corrected footprint uses CoHDL's
annulus and segmented-paste support, with the 0.5 mm sound hole above.
The device pin map, part identity, footprint name, and pad positions are
unchanged.

KiCad export also requires the compiler's accompanying custom-pad anchor
fix: earlier emitters placed zero-sized anchors at the acoustic centre,
which KiCad clamps into disconnected copper/paste dots. The corrected
emitters put nonzero anchors inside the ring and each paste sector while
preserving the authored geometry. Updating only the package does not fix
an older KiCad emitter. IPC-2581 already represents the ring with a cutout.
