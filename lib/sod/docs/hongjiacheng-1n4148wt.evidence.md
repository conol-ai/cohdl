# Hongjiacheng 1N4148WT evidence record (SOD-523 primary source)

- Library coverage: primary source of `sod::SOD1608X70N`. No part binds it in
  this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "1N4148WT Fast
  Switching Diodes" (the sheet spells it "SWITHING"), Rev 2.0, 3 pages (PDF
  created 2025-05-21).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420320.html>
  (`1N4148WT`, SOD-523).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2311171118_hongjiacheng-1N4148WT_C7420320.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/0c7f487bdf1474cd2924fe5595db8ec0.pdf>, 182,398 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `71312a1ff0a0911bd02cb41941bbc00a9abdea0146b9843db0cd9ad03fe8365a`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1: "Case: SOD-523"; "Polarity: Cathode line denotes the cathode end".
- Page 3, "Package Outline Dimensions (SOD-523)", millimetres: A (body
  length) 1.10-1.30, B (lead width) 0.25-0.35, C (lead tip to lead tip)
  1.50-1.70, D (height) 0.50-0.70, E (body width) 0.70-0.90, F (lead
  thickness) 0.05-0.20. The drawing puts the cathode band on the left in top
  view.
- Page 3, "Suggested Pad Layout": X (land length, along the lead axis)
  0.55-0.65, Y (land width) 0.65-0.75, Z (land centre to land centre, drawn
  between the two land centres) 1.37-1.47 mm. The land uses the midpoints:
  0.60 x 0.70 mm lands on 1.42 mm centres (+/-0.71 mm), outer copper span
  2.02 mm, gap 0.82 mm.
- Fit: the lead tips (C/2, 0.75-0.85 mm) sit 0.16-0.26 mm inside the
  1.01 mm outer land edge.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
