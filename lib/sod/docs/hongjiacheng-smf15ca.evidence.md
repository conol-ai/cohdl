# Hongjiacheng SMF15CA evidence record (SOD-123FL primary source)

- Library coverage: primary outline source of `sod::SODFL3818X120N`. The
  sheet gives no land; the land comes from the cross-check
  (`diodes-sod123f.evidence.md`). No part binds it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMF SERIES
  200W Surface Mount Transient Voltage Suppressors", Rev 2.2, 5 pages (PDF
  created 2026-07-02, modified 2026-09-10).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077510.html>
  (`SMF15CA`, SOD-123FL).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2312041000_hongjiacheng-SMF15CA_C19077510.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/fa41ee376507916bceb7512729759974.pdf>, 452,593 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `c8f137df222c1c36b0356096e10d601434b420c2f65407c222229dbd725df74a`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1: package box "SOD-123FL". Page 2 lists `SMF15CA` (bidirectional,
  marking IM).
- Page 2, "Package Outline Dimensions (SOD-123FL)", millimetre column:
  A (lead width) 0.90-1.10, B (body length) 2.55-2.85, C (body width)
  1.60-1.90, D (lead tip to lead tip) 3.60-3.90, E (height) 1.00-1.20,
  F (lead foot length) 0.40-0.90, G (lead thickness) 0.10-0.25. The drawing
  puts the cathode band on the left in top view. Notes: (1) cathode band on
  unidirectional parts only; (2) bidirectional parts are unmarked.
- The inch column is inconsistent with the millimetre column for A, D, E and
  F (for example A reads 0.035-0.430 in and D 0.031-0.043 in); only the
  millimetre column is used.
- No land rows and no suggested pad layout appear on any of the five pages
  (checked on the rendered pages and with `pdftotext`).
- Fit of the derived Diodes land (1.34 x 1.80 mm at +/-1.43 mm): the lead
  tips (D/2, 1.80-1.95 mm) sit 0.15-0.30 mm inside the 2.10 mm outer land
  edge; the inner end of a lead foot (D/2 - F) lies 0.90-1.55 mm from the
  centre, outboard of the 0.76 mm inner land edge.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
