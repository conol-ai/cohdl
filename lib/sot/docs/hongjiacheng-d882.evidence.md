# Hongjiacheng D882 evidence record (SOT-89-3 primary source)

- Library coverage: primary outline source of `sot::SOT3P150X450X250N`. The
  sheet's suggested pad layout does not fit its own outline (below), so the
  land comes from the cross-check (`diodes-sot89.evidence.md`). No part
  binds it in this change.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "D882 NPN
  Transistors, SOT-89 Plastic-Encapsulate Transistors", Rev 2.0, 3 pages
  (PDF created 2025-01-02). The text layer also carries an older `Rev:1.1`
  string under the rendered `Rev:2.0` footer.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C20069128.html>
  (`D882`, SOT-89).
- Datasheet link recorded in the B1 catalog snapshot: <https://wmsc.lcsc.com/wmsc/upload/file/pdf/v2/lcsc/2312291746_hongjiacheng-D882_C20069128.pdf>
  (returns an HTML viewer, not the PDF).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/de241213863e8351260fb7abe3fab88f.pdf>, 264,842 bytes.
- Retrieved: 2026-11-07.
- SHA-256: `fcd194b90c4682646af42a0999038147208230a342d1a2d5379b64910a937c35`.
- Manufacturer site: the document footer and header name Zhuhai Hongjiacheng
  Technology Co., Ltd and `www.ichjc.com`. No manufacturer-hosted copy was
  located; the maintainer approved the LCSC-served manufacturer PDF as the
  source for this land (LIBRARY.md section 2 fallback). The B1 snapshot lists
  the maker as `hongjiacheng`; the document footer is the identity used.

## Facts used

- Page 1, "Function Diagram" (top view): leads on the lower edge, pin 1 B
  left, pin 2 C centre, pin 3 E right; the tab on the upper edge carries no
  number. The pin map belongs to the series; the land uses only the
  numbering geometry.
- Page 3, "Package Outline Dimensions (SOT-89)", millimetres: A (body
  length) 4.4-4.6, B (tab width) 1.6-1.8, C (tab tip to lead tip) 3.9-4.1,
  D (body width) 2.4-2.6, E (pin 1 to pin 3 centres) 2.9-3.1, a (lead width,
  pin 1) 0.41-0.43 (the inch column reads 0.0164-0.018 in, 0.42-0.46 mm),
  b (lead width, pin 2) 0.35-0.45, L (lead length) 0.95-1.05, G 0.3-0.5,
  H (height) 1.4-1.5.
- Page 3, "Suggested Pad Layout": c (lead land width) 0.80-1.00, d (lead
  land length) 1.30-1.50, e 0.70-0.90, K (lead land pitch) 1.40-1.60, J (gap
  between the lead lands and the tab land) 1.80-2.00; an octagonal tab land
  X 2.50-2.70 wide by Y 4.30-4.50 tall with X1 1.30-1.50, Y1 3.10-3.30 and
  45-degree corners (theta).
- Why the land is not used: the lead lands, the gap and the tab land stack
  to d + J + Y = 7.40-8.00 mm against the 3.90-4.10 mm package (C), and to
  5.60-6.20 mm even with X and Y exchanged. With the lead lands on the lead
  feet, the tab land starts at least 1.80 mm above them, leaves most of the
  exposed tab and the lead-2 shank without copper, and extends well past the
  tab tip. No copper joins lead 2 to the tab. The drawing is not usable as a
  land for this outline, so `SOT3P150X450X250N` takes the Diodes land.

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the land.
