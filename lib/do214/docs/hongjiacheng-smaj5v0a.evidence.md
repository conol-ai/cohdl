# Hongjiacheng SMAJ5.0A evidence record (SMA / DO-214AC second view)

- Library coverage: second Hongjiacheng view of the outline used by
  `do214::DIOM5026X241N`; not the land's primary source.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMAJ SERIES
  400W Surface Mount Transient Voltage Suppressors", Rev 2.2, 5 pages (PDF
  created 2026-06-23, modified 2026-09-10).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077523.html>
  (`SMAJ5.0A`, SMA(DO-214AC)).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2312041002_hongjiacheng-SMAJ5-0A_C19077523.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/f3d2c9df3269749880b305befd579c39.pdf>, 500,192 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `3ff5a7b8618e9ea109acec3fdfefba71442e321ac3cd4461702f08964d205947`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). LCSC shows the brand
  string `R+O` on the listing; the document footer is the identity used.

## Facts used

- Page 2, "Package Outline Dimensions (SMA/DO-214AC)": A-H identical to the
  SS34 sheet (A 1.25-1.65, B 3.95-4.65, C 2.35-2.85, D 1.98-2.41, E 0.76-1.52,
  F 0.203 max, G 4.70-5.30, H 0.15-0.31 mm). Note (1): the cathode band is on
  unidirectional parts only; bidirectional (`CA`) parts are unmarked.
- Page 2, the same table also lists land rows without a drawing: M 2.26 min,
  J 2.10 min, K 2.74 max (mm). These differ from the SS34 sheet (M 1.70 min,
  K 2.30 max). `DIOM5026X241N` (2.10 x 1.70 mm lands, 2.30 mm gap) meets this
  sheet's J and K limits but not its M limit; see the open question in
  `README.md`.
- No separate "Suggested Pad Layout" figure appears on any of the five pages.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
