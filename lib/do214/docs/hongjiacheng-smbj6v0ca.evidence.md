# Hongjiacheng SMBJ6.0CA evidence record (SMB / DO-214AA primary source)

- Library coverage: primary source of `do214::DIOM5336X265N`. No part binds
  it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMBJ SERIES
  600W Surface Mount Transient Voltage Suppressors", Rev 1.1, 5 pages (PDF
  created 2024-12-26).
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077561.html>
  (`SMBJ6.0CA`, DO-214AA(SMB)).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2312041004_hongjiacheng-SMBJ6-0CA_C19077561.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/29e006ceaae5808e6d17d021de7bda89.pdf>, 294,303 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1: package box "DO-214AA(SMB)"; the function diagram shows the
  bidirectional symbol. Page 2 lists `SMBJ6.0CA` (bidirectional, marking AG).
- Page 2, "Package Outline Dimensions (SMB/DO-214AA)", millimetres: A (lead
  width) 1.85-2.21, B (body length) 4.25-4.85, C (body width) 3.30-3.94,
  D (height) 2.15-2.65, E (lead foot length) 0.75-1.52, F 0.203 max,
  G (overall length) 5.08-5.59, H (lead thickness) 0.15-0.31. The drawing
  puts the cathode band on the left in top view, labelled "for
  uni-directional products only".
- Page 2, the same table's land rows, drawn under the outline as two hatched
  lands J-K-J long by M wide: M (land width) 2.26 min, J (land length) 2.10
  min, K (gap between lands) 2.74 max. One-sided limits only, so the land
  uses the limits themselves (as `do214::DIOM5026X241N` does): 2.10 x 2.26 mm
  lands, centres +/-2.42 mm, outer copper span 6.94 mm.
- These three land limits are numerically identical to the SMAJ sheet's
  (`hongjiacheng-smaj5v0a.evidence.md`); see the SMA open question in
  `README.md`.
- Fit: the lead tips (G/2, 2.54-2.80 mm) sit 0.68-0.93 mm inside the land's
  3.47 mm outer edge. The inner end of a lead foot (G/2 - E) lies
  1.02-2.05 mm from the centre, so with the 1.37 mm inner land edge up to
  0.35 mm of a long foot's heel can sit over the gap.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
