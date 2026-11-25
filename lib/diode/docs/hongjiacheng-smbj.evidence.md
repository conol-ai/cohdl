# Hongjiacheng SMBJ series evidence record (30 V parts)

- Library coverage: `TVS_SMBJ_Uni` and `TVS_HONGJIACHENG_SMBJ30A`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMBJ SERIES
  600W Surface Mount Transient Voltage Suppressors", Rev 1.1, 5 pages.
- JLCPCB part page read on 2026-11-25: C19077584, manufacturer
  "hongjiacheng", MPN `SMBJ30A`, package DO-214AA(SMB), class Extended,
  stock 231,959.
- PDF as served by LCSC for C19077584:
  <https://datasheet.lcsc.com/datasheet/pdf/29e006ceaae5808e6d17d021de7bda89.pdf>,
  294,303 bytes.
- Retrieved: 2026-11-25.
- SHA-256: `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39`.
- Same document, by SHA-256, as the land's record
  `lib/do214/docs/hongjiacheng-smbj6v0ca.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 5 "Part Numbering" builds the type number as SMBJ,
  voltage, optional C (bidirectional), A (5% tolerance). Page 3 notes 1 and
  2 say the same.
- Pins: page 2 outline, "Cathode Band (For uni-directional products
  only)". The sheet does not number the terminals; the library puts the
  cathode on pad 1 (lib/README.md).
- Page 1, maximum ratings: PPPM 600 W at 10/1000 us.
- Page 3, the SMBJ30A / SMBJ30CA row (markings MK / CK): VRWM 30.0 V, VBR
  33.30 to 36.80 V at IT 1 mA, VC 48.4 V at IPP 12.40 A, IR 5 uA at VRWM.
- Land: page 2 is the primary source of `do214::DIOM5336X265N`, so this
  part sits on its own maker's land.

The PDF is not redistributed by this repository. Download it from the
source above and verify the recorded SHA-256 before auditing the model.
