# Hongjiacheng 2N7002W evidence record (SOT-323 primary source)

- Library coverage: primary source of `sot::SOT3P130X200X125N`. No part
  binds it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "2N7002W
  N-Channel MOSFET, SOT-323 Plastic-Encapsulate MOSFETS", Rev 2.0, 5 pages
  (PDF created 2024-12-30). The text layer also carries an older `Rev:1.0`
  string under the rendered `Rev:2.0` footer.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420322.html>
  (`2N7002W`, SOT-323).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2407101112_hongjiacheng-2N7002W_C7420322.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/731527b245a035a3d12b2715a61f4f86.pdf>, 293,292 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `def7318e3220b19ed9e8061b02294c21b8d17f2f88ebfaa91de71cb06c111451`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1, "Reference News" (top view): pin 1 G lower left, pin 2 S lower
  right, pin 3 D on the upper edge, the same numbering geometry as SOT-23.
  The pin map belongs to the series; the land uses only the geometry.
- Page 5, "Package Outline Dimensions (SOT-323)", millimetres: A (height)
  0.90-1.10, A1 0-0.10, A2 0.90-1.00, b (lead width) 0.15-0.40, c
  0.10-0.25, D (body length) 1.80-2.20, E (body width) 1.15-1.35, E1 (lead
  tip to lead tip) 2.15-2.45, e 0.650 typ, e1 (pin 1 to pin 2) 1.20-1.40,
  L 0.525 ref, L1 0.26-0.46.
- Page 5, "Suggested Pad Layout": J (land width along the row) 0.65-0.75,
  K (land length across the row) 0.85-0.95, M (row centre spacing)
  1.85-1.95, N (pin 1 to pin 2 centre spacing) 1.25-1.35. The land uses
  the midpoints: 0.70 x 0.90 mm lands, 1.30 mm pin 1-2 spacing, 1.90 mm row
  spacing.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
