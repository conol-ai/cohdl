# Diodes Incorporated SOD123 package file evidence record (SOD-123 cross-check)

- Library coverage: independent cross-check of `sod::SOD3716X135N`.
- Document identity: Diodes Incorporated, "SOD123 Package Information",
  Rev. 2017-03-16 (March 2017), 5 pages.
- Official source: <https://www.diodes.com/assets/Package-Files/SOD123.pdf>,
  546,069 bytes. Not distributor-served.
- Retrieved: 2026-11-07.
- SHA-256: `ceda14ae0447b97457c9df48192839f955832f1db6c1bb48d132da39cef5d714`.

## Facts used

- Page 1, "Package Outline Dimensions": He (lead span) 3.55-3.85, E (body
  length) 2.55-2.85, D (body width) 1.40-1.70, A (height) 1.00-1.35, b (lead
  width) 0.52-0.62 mm. Polarity: cathode bar.
- Page 1, "Suggested Pad Layout": X = 0.900, Y = 0.950, X1 = 4.050 mm, i.e.
  3.15 mm centres and a 2.25 mm gap.
- This is the same suggested layout as Diodes DS30086 Rev 31-2 (already in
  the repository as `lib/diode/docs/diodes-inc-bav16w-1n4148w-datasheet.pdf`,
  page 4) and therefore the land of `diode::FP_D_SOD_123`.

The PDF is not redistributed by this repository. Download it from the
official source and verify the recorded SHA-256 before auditing the land.
