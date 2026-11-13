# Hongjiacheng SMCJ36CA evidence record (part)

- Library coverage: `SMCJ36CA` and `TVS_SMCJ36CA`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMCJ SERIES
  1500W Surface Mount Transient Voltage Suppressors", Rev 2.2, 5 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077609.html>
  (`SMCJ36CA`, SMC(DO-214AB)).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/63434cf44d11833c6d7a145dda2ad39c.pdf>, 320,276 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `5cbe7c34174b49058f9e13aa94ae9d17b521feea2d4537cc46876c764fc3cefe`.
- Same document as the land's record, `lib/do214/docs/hongjiacheng-smcj36ca.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 3 lists `SMCJ36CA` as the bidirectional form of SMCJ36A
  (marking BFP); page 3 note (2): the suffix CA specifies a bidirectional
  device.
- Pins: a bidirectional ("CA") device has no polarity. Page 2, outline notes: the cathode band is on unidirectional parts only and bidirectional devices are unmarked. The
  library numbers the two interchangeable terminals IO1 (pad 1) and IO2
  (pad 2).
- Page 1, "Maximum Ratings" (Ta = 25 C): PPPM 1500 W with a 10/1000 us
  waveform; PM(AV) 6.5 W on an infinite heat sink at TL = 75 C; Tj and Tstg
  -55 to +150 C; RthJ-L 15 C/W and RthJ-A 75 C/W typical. The IFSM and VF
  rows are forward ratings, stated for unidirectional parts; they are not
  used.
- Page 3, "Electrical Characteristics" (Ta = 25 C), SMCJ36A / SMCJ36CA row:
  VRWM 36 V; VBR 40 V minimum, 44.2 V maximum at IT = 1 mA; VC 58.1 V at
  IPP = 25.9 A; IR 1 uA at VRWM. Note (3)'s doubled IR limit applies only at
  10 V and under, so not to this part.
- Land fit: the sheet gives the outline only (page 2); `do214::DIOM7959X261N`
  takes its land from the Diodes SMC package file (see the land's record).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
