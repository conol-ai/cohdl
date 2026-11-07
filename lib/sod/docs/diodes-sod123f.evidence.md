# Diodes Incorporated SOD123F package file evidence record (SOD-123FL land source and cross-check)

- Library coverage: land source of `sod::SODFL3818X120N` (the Hongjiacheng
  primary gives no land) and independent cross-check of its outline.
- Document identity: Diodes Incorporated, "SOD123F Package Information",
  Rev. 2026-04-08 (April 2026), 4 pages.
- Official source: <https://www.diodes.com/assets/Package-Files/SOD123F.pdf>,
  311,064 bytes. Not distributor-served.
- Retrieved: 2026-11-07.
- SHA-256: `d7104f55a097b2b40c3a1353911176d871040959c9ecfc8a999835cf087aca0d`.

## Facts used

- Page 1, "Mechanical Data": "Polarity: Cathode Bar".
- Page 1, "Package Outline Dimensions": A (height) 0.81-1.15, b (lead width)
  0.80-1.05, c (lead thickness) 0.05-0.30, D (body width) 1.70-1.90 (typ
  1.80), E (body length) 2.60-2.80 (typ 2.70), He (lead span) 3.30-3.70 (typ
  3.50), L2 (lead foot) 0.35-0.85 mm.
- Page 1, "Suggested Pad Layout": X = 1.34 (land length), Y = 1.80 (land
  width), C = 2.86 (centres), G = 1.52 (gap), X1 = 4.20 (outer span) mm.
  `SODFL3818X120N` uses these values: 1.34 x 1.80 mm lands at +/-1.43 mm.
- Diodes names the outline SOD123F; Hongjiacheng and LCSC call the same
  two-flat-lead outline SOD-123FL. Hongjiacheng's lead span (3.60-3.90 mm)
  is longer than Diodes' He (3.30-3.70 mm); see `README.md`.

The PDF is not redistributed by this repository. Download it from the
official source and verify the recorded SHA-256 before auditing the land.
