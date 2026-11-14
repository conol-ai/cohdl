# Hongjiacheng SMF15CA evidence record (part)

- Library coverage: `SMF15CA` and `TVS_SMF15CA`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SMF SERIES
  200W Surface Mount Transient Voltage Suppressors", Rev 2.2, 5 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C19077510.html>
  (`SMF15CA`, SOD-123FL).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/fa41ee376507916bceb7512729759974.pdf>, 452,593 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `c8f137df222c1c36b0356096e10d601434b420c2f65407c222229dbd725df74a`.
- Same document as the land's record, `lib/sod/docs/hongjiacheng-smf15ca.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 2 lists `SMF15CA` as the bidirectional form of SMF15A
  (marking IM); page 3 note (2): the suffix CA specifies a bidirectional
  device.
- Pins: a bidirectional ("CA") device has no polarity. Page 2, outline notes: the cathode band is on unidirectional parts only and bidirectional devices are unmarked. The
  library numbers the two interchangeable terminals IO1 (pad 1) and IO2
  (pad 2).
- Page 1, "Maximum Ratings" (Ta = 25 C): PPPM 200 W with a 10/1000 us
  waveform; PM(AV) 0.4 W on an infinite heat sink at TL = 75 C; Tj and Tstg
  -55 to +150 C; RthJ-L 100 C/W and RthJ-A 220 C/W typical. The IFSM and VF
  rows are forward ratings, stated for unidirectional parts; they are not
  used.
- Page 2, "Electrical Characteristics" (Ta = 25 C), SMF15A / SMF15CA row:
  VRWM 15 V; VBR 16.7 V minimum, 18.5 V maximum at IT = 1 mA; VC 24.4 V at
  IPP = 8.2 A; IR 1 uA at VRWM. Note (3)'s doubled IR limit applies only at
  10 V and under, so not to this part.
- Land fit: the sheet gives the outline only (page 2); `sod::SODFL3818X120N`
  takes its land from the Diodes SOD123F package file (see the land's record).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
