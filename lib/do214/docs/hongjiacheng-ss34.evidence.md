# Hongjiacheng SS34 evidence record (SMA / DO-214AC primary source)

- Library coverage: primary source of `do214::DIOM5026X241N`. No part binds it
  in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SS32 THRU SS320
  SMA Surface Mount Schottky Barrier Rectifier", Rev 2.0, 3 pages (PDF created
  2025-01-13).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420365.html>
  (`SS34`, SMA(DO-214AC)).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2402010525_hongjiacheng-SS34_C7420365.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/9e763de50d66dc26804ef4c3650380df.pdf>, 259,506 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `f83a234fc3a645c2fe6d495d363a8b1bef79562769c813a72ca58e70f6b98927`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). LCSC shows the brand
  string `R+O` on the listing; the document footer is the identity used.

## Facts used

- Page 1: "Case: DO-214AC(SMA)"; "Polarity: Cathode line denotes the cathode
  end"; the function diagram draws the band on the left.
- Page 3, "Package Outline Dimensions (SMA/DO-214AC)", millimetres: A (lead
  width) 1.25-1.65, B (body length) 3.95-4.65, C (body width) 2.35-2.85,
  D (height) 1.98-2.41, E (lead foot length) 0.76-1.52, G (overall length)
  4.70-5.30.
- Page 3, "Suggested Pad Layout": M (land width) 1.70 min, J (land length)
  2.10 min, K (gap between lands) 2.30 max. One-sided limits only, so the land
  uses the limits themselves: 2.10 x 1.70 mm lands, centres +/-2.20 mm, outer
  copper span 6.50 mm.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
