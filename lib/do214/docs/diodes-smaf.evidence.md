# Diodes Incorporated SMAF package file evidence record (SMAF cross-check)

- Library coverage: independent cross-check of `do214::DIOM4726X120N`.
- Document identity: Diodes Incorporated, "SMAF Package Information",
  Rev. 2019-06-27 (June 2019), 5 pages.
- Official source: <https://www.diodes.com/assets/Package-Files/SMAF.pdf>,
  318,917 bytes. Not distributor-served.
- Retrieved: 2026-11-07.
- SHA-256: `60be78ccf2d43e2704629ba08fd5d49d89b6d483a0f318139de445232367a73c`.

## Facts used

- Page 1, "Mechanical Data": "Polarity: Cathode Bar". The document names no
  JEDEC outline.
- Page 1, "Package Outline Dimensions": A (height) 0.90-1.10, b (lead width)
  1.25-1.65, c (lead thickness) 0.10-0.40, D (body width) 2.25-2.95,
  E (body length) 3.95-4.60, HE (overall length) 4.80-5.60, L (lead foot)
  0.50-1.50 mm.
- Page 1, "Suggested Pad Layout": X = 2.50 (land length), Y = 1.70 (land
  width), C = 4.00 (centres), G = 1.50 (gap), X1 = 6.50 (outer span) mm.
  These are the same values as the Diodes SMA layout
  (`diodes-sma.evidence.md`).
- This Diodes outline is longer than the Hongjiacheng SMAF (HE 4.80-5.60
  against D 4.40-4.90 mm); see `README.md`.

The PDF is not redistributed by this repository. Download it from the
official source and verify the recorded SHA-256 before auditing the land.
