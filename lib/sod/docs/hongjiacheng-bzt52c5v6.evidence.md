# Hongjiacheng BZT52C5V6 evidence record (SOD-123 primary source)

- Library coverage: primary source of `sod::SOD3716X135N`. No part binds it in
  this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "BZT52C2V0 THRU
  BZT52C75 Surface Mount Zener Diodes", Rev 2.1, 4 pages (PDF created
  2025-10-17).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077402.html>
  (`BZT52C5V6`, SOD-123).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2403231215_hongjiacheng-BZT52C5V6_C19077402.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/02a7f2c734d5d8407bcaa05ee2a504ab.pdf>, 232,345 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `6c66d1a4354d67a86200f0764e0b71641731b67985e0618cf7cd05147392799a`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). LCSC shows the brand
  string `R+O` on the listing; the document footer is the identity used.

## Facts used

- Page 1: "Case: SOD-123"; "Polarity: Cathode line denotes the cathode end".
- Page 4, "Package Outline Dimensions (SOD-123)", millimetre column: A (lead
  tip to lead tip) 3.55-3.85, B (body length) 2.55-2.85, C (body width)
  1.40-1.80, D (height) 0.95-1.35. The drawing puts the cathode band on the
  left in top view.
- Page 4, "Suggested Pad Layout": J (land length) 0.91 min, K (gap between
  lands) 2.36 max, M (land width) 1.22 min. The table gives one-sided limits
  only, so the land uses the limits themselves: 0.91 x 1.22 mm lands, centres
  +/-1.635 mm, outer copper span 4.18 mm.
- The inch column of the outline table is inconsistent with the millimetre
  column for D, E and G (for example D reads 0.140-0.152 in, a copy of A);
  only the millimetre column is used.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
