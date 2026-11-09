# Hongjiacheng SMCJ36CA evidence record (SMC / DO-214AB primary source)

- Library coverage: primary outline source of `do214::DIOM7959X261N`. The
  sheet gives no land; the land comes from the cross-check
  (`diodes-smc.evidence.md`). No part binds it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMCJ SERIES
  1500W Surface Mount Transient Voltage Suppressors", Rev 2.2, 5 pages (PDF
  created 2026-06-23).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077609.html>
  (`SMCJ36CA`, SMC(DO-214AB)).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2312221742_hongjiacheng-SMCJ36CA_C19077609.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/63434cf44d11833c6d7a145dda2ad39c.pdf>, 320,276 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `5cbe7c34174b49058f9e13aa94ae9d17b521feea2d4537cc46876c764fc3cefe`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1: package box "DO-214AB(SMC)". Page 3 lists `SMCJ36CA`
  (bidirectional, marking BFP).
- Page 2, "Package Outline Dimensions (SMC/DO-214AB)", millimetres: A (lead
  width) 2.85-3.27, B (body length) 6.60-7.11, C (body width) 5.59-6.22,
  D (height) 1.99-2.61, E (lead foot length) 0.76-1.52, F 0.20 max,
  G (overall length) 7.75-8.13, H (lead thickness) 0.15-0.31. The drawing
  puts the cathode band on the left in top view. Notes: (1) cathode band on
  unidirectional parts only; (2) bidirectional parts are unmarked.
- No land rows and no suggested pad layout appear on any of the five pages
  (checked on the rendered pages and with `pdftotext`).
- Fit of the derived Diodes land (2.50 x 3.30 mm at +/-3.45 mm): the lead
  tips sit 0.64-0.83 mm inside the 4.70 mm outer land edge; the inner end of
  a lead foot (G/2 - E) lies 2.36-3.31 mm from the centre, outboard of the
  2.20 mm inner land edge; the 3.27 mm maximum lead width leaves 0.015 mm per
  side on the 3.30 mm land.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
