# Hongjiacheng SMBJ6.0CA evidence record (part)

- Library coverage: `SMBJ6V0CA` and `TVS_SMBJ6V0CA`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMBJ SERIES
  600W Surface Mount Transient Voltage Suppressors", Rev 1.1, 5 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077561.html>
  (`SMBJ6.0CA`, DO-214AA(SMB)).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/29e006ceaae5808e6d17d021de7bda89.pdf>, 294,303 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39`.
- Same document as the land's record, `lib/do214/docs/hongjiacheng-smbj6v0ca.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 2 lists `SMBJ6.0CA` as the bidirectional form of SMBJ6.0A
  (marking AG); page 3 note (2): the suffix CA specifies a bidirectional
  device.
- Pins: a bidirectional ("CA") device has no polarity. Page 2 labels the outline's cathode band "for uni-directional products only". The
  library numbers the two interchangeable terminals IO1 (pad 1) and IO2
  (pad 2).
- Page 1, "Maximum Ratings" (Ta = 25 C): PPPM 600 W with a 10/1000 us
  waveform; PM(AV) 5.0 W on an infinite heat sink at TL = 75 C; Tj and Tstg
  -55 to +150 C; RthJ-L 20 C/W and RthJ-A 100 C/W typical. The IFSM and VF
  rows are forward ratings, stated for unidirectional parts; they are not
  used.
- Page 2, "Electrical Characteristics" (Ta = 25 C), SMBJ6.0A / SMBJ6.0CA row:
  VRWM 6.0 V; VBR 6.67 V minimum, 7.37 V maximum at IT = 10 mA; VC 10.3 V at
  IPP = 58.25 A; IR 800 uA at VRWM.
- Page 3, note (3): "For Bi-Directional devices having VR of 10 volts and
  under, the IR limit is double", so 1600 uA for this part.
- Land fit: page 2 is the primary source of `do214::DIOM5336X265N` (see the
  land's record).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
