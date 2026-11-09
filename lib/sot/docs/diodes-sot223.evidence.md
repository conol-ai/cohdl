# Diodes Incorporated SOT223 package file evidence record (SOT-223 land source and cross-check)

- Library coverage: land source of `sot::SOT4P230X651X351N` (the AMS primary
  gives no land) and independent cross-check of its outline.
- Document identity: Diodes Incorporated, "SOT223 Package Information",
  Rev. 2017-03-06 (March 2017), 5 pages.
- Official source: <https://www.diodes.com/assets/Package-Files/SOT223.pdf>,
  545,850 bytes. Not distributor-served.
- Retrieved: 2026-11-07.
- SHA-256: `c8d9447a1925136b5d045767c8f0177ea9144c3c8bac5acb23ea3f9343589d19`.

## Facts used

- Page 1, "Package Outline Dimensions" (top view: three leads on the lower
  edge, the tab on the upper edge, no pin numbers): A (height) 1.55-1.65,
  A1 0.010-0.15, b (lead width) 0.60-0.80, b1 (tab width) 2.90-3.10,
  C 0.20-0.30, D (body length) 6.45-6.55, E (body width) 3.45-3.55,
  E1 (tab tip to lead tip) 6.90-7.10, e 4.60, e1 2.30, L (lead foot)
  0.85-1.05, Q 0.84-0.94 mm.
- Page 1, "Suggested Pad Layout": C = 2.30 (lead pitch), C1 = 6.40 (lead-row
  centre to tab centre), X = 1.20 and Y = 1.60 (lead lands), X1 = 3.30 and
  Y1 = 1.60 (tab land), Y2 = 8.00 (overall) mm. Centred on the overall
  span: lead lands at (-2.30, 3.20), (0, 3.20), (2.30, 3.20) and the tab
  land at (0, -3.20) in that orientation; `SOT4P230X651X351N` turns them
  onto the SOT frame (datasheet (x, y) at (y, -x)).

The PDF is not redistributed by this repository. Download it from the
official source and verify the recorded SHA-256 before auditing the land.
