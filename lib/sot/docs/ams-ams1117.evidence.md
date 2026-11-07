# Advanced Monolithic Systems AMS1117 evidence record (SOT-223 primary source)

- Library coverage: primary outline source of `sot::SOT4P230X651X351N`. The
  sheet gives no land; the land comes from the cross-check
  (`diodes-sot223.evidence.md`). No part binds it in this change; AMS1117
  is bound in the ICs batch.
- Document identity: Advanced Monolithic Systems, Inc., "AMS1117 1A Low
  Dropout Voltage Regulator", 8 pages, no revision printed (PDF title
  "Microsoft Word - DS1117", created 2009-08-27). Package drawings
  "(SOT-223) AMS DRW# 042292" on page 7.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C6186.html>
  (`AMS1117-3.3`, SOT-223, Basic).
- Datasheet link recorded in the B1 catalog snapshot, which serves the PDF:
  <https://www.lcsc.com/datasheet/lcsc_datasheet_2410121508_Advanced-Monolithic-Systems-AMS1117-3-3_C6186.pdf>.
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/e6935943fc6b1bbf350a1a0f3e90dc4a.pdf>, 95,151 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `189a2651878a87d590b768eaa9b44217a3fdf460352ce6ecaff127221282a3f0`.
- Manufacturer site: the document footer names Advanced Monolithic Systems,
  Inc. and `www.advanced-monolithic.com`. No manufacturer-hosted copy was
  retrieved; the maintainer designated this LCSC-served manufacturer PDF as
  the SOT-223 primary (LIBRARY.md section 2 fallback).

## Facts used

- Page 1, "Pin Connections", "SOT-223 Top View": leads on the lower edge,
  1 Ground/Adjust left, 2 VOUT centre, 3 VIN right; the tab on the upper
  edge carries no number. "TAB IS OUTPUT" is printed beside the drawings.
  The pin map belongs to the part; the land uses only the geometry.
- Page 7, "3 LEAD SOT-223 PLASTIC PACKAGE", inches (millimetres): body
  length 0.248-0.264 (6.30-6.71), tab width 0.116-0.124 (2.95-3.15), tab
  tip to lead tip 0.264-0.287 (6.71-7.29), body width 0.130-0.146
  (3.30-3.71), lead pitch 0.090 (2.29) NOM, pin 1 to pin 3 0.181 (4.60)
  NOM, height 0.071 (1.80) MAX, lead width and lead foot 0.025-0.033
  (0.64-0.84), standoff 0.012 (0.31) MIN, lead thickness 0.010-0.014
  (0.25-0.36). A 0.033-0.041 (0.84-1.04) dimension is drawn at lead 3 in
  the top view without a label; it is not used.
- No land rows and no suggested pad layout appear on any of the eight
  pages (checked on the rendered pages and with `pdftotext`).
- Fit of the derived Diodes land (1.20 x 1.60 mm lead lands at 2.30 mm,
  3.30 x 1.60 mm tab land, 6.40 mm between row centres): the lead tips
  (3.355-3.645 mm from the centre) sit 0.36-0.65 mm inside the 4.00 mm outer
  land edge; the inner end of a lead foot (tip minus 0.64-0.84 mm) lies at
  least 2.51 mm from the centre, outboard of the 2.40 mm inner land edge;
  the 3.15 mm maximum tab width leaves 0.075 mm per side on the 3.30 mm tab
  land; the 2.29 mm nominal pitch is the land's 2.30 mm.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
