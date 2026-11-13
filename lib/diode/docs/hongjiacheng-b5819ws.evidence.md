# Hongjiacheng B5819WS evidence record (part)

- Library coverage: `Schottky_B5819WS` and `SCHOTTKY_B5819WS`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "B5817WS THRU
  B5819WS SOD-323 Surface Mount Schottky Barrier Rectifier", Rev 2.0, 3 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420331.html>
  (`B5819WS`, SOD-323).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/971b15e3a97f69d3242d0be09da57662.pdf>, 231,564 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `d0140bf852f26e98f404651f9055c2b99fef6bea17326ee6be37854c714ede94`.
- Same document as the land's record, `lib/sod/docs/hongjiacheng-b5819ws.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 1 lists B5817WS, B5818WS and B5819WS (marking SL); the
  ratings below are the B5819WS column.
- Pins: page 1, "Polarity: Cathode line denotes the cathode end"; the function
  diagram draws the cathode bar at the band. The sheet does not number the
  terminals; the library puts the cathode on pad 1 (lib/README.md).
- Page 1, "Maximum Ratings" (Ta = 25 C): VRRM 40 V, VRMS 28 V, VDC 40 V,
  IF(AV) 1.0 A, IFSM 10 A (8.3 ms half sine), Pd 250 mW, Tj and Tstg -55 to
  +150 C, RthJ-A 400 C/W typical.
- Page 2, "Electrical Characteristics" (Ta = 25 C): VF 0.6 V maximum at
  IF = 1.0 A and 0.9 V maximum at IF = 3.0 A; IR 1.0 mA maximum at VR = VDC;
  CJ 120 pF typical at VR = 4.0 V, f = 1 MHz.
- Land fit: page 3 is the primary source of `sod::SOD2613X110N` (see the
  land's record).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
