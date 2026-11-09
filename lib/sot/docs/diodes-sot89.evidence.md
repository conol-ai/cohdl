# Diodes Incorporated SOT89 package file evidence record (SOT-89-3 land source and cross-check)

- Library coverage: land source of `sot::SOT3P150X450X250N` (the
  Hongjiacheng primary's land does not fit its outline) and independent
  cross-check of its outline.
- Document identity: Diodes Incorporated, "SOT89 Package Information",
  Rev. 2026-01-16 (January 2026), 4 pages.
- Official source: <https://www.diodes.com/assets/Package-Files/SOT89.pdf>,
  421,941 bytes. Not distributor-served.
- Retrieved: 2026-11-07.
- SHA-256: `7e918198860b429583dadb78657bc0e721091e6835311523aeb69f045724440a`.

## Facts used

- Page 1, "Package Outline Dimensions" (top view: leads on the lower edge,
  tab on the upper edge; bottom view: the tab is one piece with the centre
  lead), millimetres: A (height) 1.40-1.60, B (centre lead width)
  0.50-0.62, B1 (outer lead width) 0.42-0.54, c 0.35-0.43, D (body length)
  4.40-4.60, D1 (tab width) 1.62-1.83, D2 1.61-1.81, E (body width)
  2.40-2.60, E2 2.05-2.35, e 1.50, H (tab tip to lead tip) 3.95-4.25, H1
  2.63-2.93, L (lead length) 0.90-1.20, L1 0.00-0.527, z 0.20-0.40.
- Page 1, "Suggested Pad Layout" (same orientation, origin at the land
  centre, Y5 = Y4/2): C = 1.500 (outer land to centre-line pitch),
  G = 0.244 (gap between an outer land and the tab land), X = 0.580 and
  Y = 1.630 (outer lands), X1 = 0.760 and Y2 = 1.500 (lead-2 land),
  X2 = 1.933 and Y1 = 3.030 (tab land, joined to the lead-2 land in one
  T-shaped copper area), Y3 = 0.635 (centre line to the outer lands' inner
  ends), Y4 = 4.530 (overall), Y5 = 2.265 mm.
- Reconstructed centres in that orientation (+y toward the leads): outer
  lands (-/+1.500, 1.450); lead-2 land (0, 1.515); tab land (0, -0.750).
  `SOT3P150X450X250N` declares the lead-2 land 1.63 mm long at (0, 1.450),
  which adds 0.13 mm inside the tab land and leaves the copper unchanged,
  and turns the drawing onto the SOT frame (datasheet (x, y) at (y, -x)).

The PDF is not redistributed by this repository. Download it from the
official source and verify the recorded SHA-256 before auditing the land.
