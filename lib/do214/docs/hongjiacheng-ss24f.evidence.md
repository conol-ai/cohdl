# Hongjiacheng SS24F evidence record (SMAF primary source)

- Library coverage: primary source of `do214::DIOM4726X120N`. No part
  binds it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SS22F THRU
  SS220F Surface Mount Schottky Barrier Rectifier", Rev 2.0, 3 pages (PDF
  created 2025-01-14). The text layer also carries an older `Rev:1.0` string
  under the rendered `Rev:2.0` footer.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C18199202.html>
  (`SS24F`, SMAF).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2403020740_hongjiacheng-SS24F_C18199202.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/9cf52fccd8072ebd42ffcf4e002e33ad.pdf>, 230,713 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `21831b8b07a50f1201f8812f858b89348285f1d62444fcb5d7de4f2b331acf5b`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1: "Case: SMAF"; "Polarity: Cathode line denotes the cathode end";
  the function diagram draws the band on the left. Page 1 lists `SS24F`
  (marking SS24F). The sheet names no JEDEC outline (no "DO-221AC").
- Page 3, "Package Outline Dimensions (SMAF)", millimetres: A (lead width)
  1.30-1.60, B (body length) 3.30-3.70, C (body width) 2.40-2.70,
  D (overall length) 4.40-4.90, E (height) 0.90-1.20, F (lead foot length)
  0.80-1.20, G (lead thickness) 0.12-0.20. The drawing puts the cathode band
  on the left in top view.
- Page 3, "Suggested Pad Layout": M (land width) 1.80 min, J (land length)
  1.60 min, K (gap between lands) 2.2 max. One-sided limits only, so the
  land uses the limits themselves: 1.60 x 1.80 mm lands, centres
  +/-1.90 mm, outer copper span 5.40 mm.
- Fit on this outline: the lead tips (D/2, 2.20-2.45 mm) sit 0.25-0.50 mm
  inside the 2.70 mm outer land edge; the inner end of a lead foot
  (D/2 - F) lies 1.00-1.65 mm from the centre, so with the longest foot and
  shortest overall length up to 0.10 mm of heel sits over the 2.20 mm gap
  (inner land edge 1.10 mm); the 1.60 mm maximum lead width fits the
  1.80 mm land width.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
