# Hongjiacheng SS34 evidence record (part)

- Library coverage: `Schottky_SS34` and `SCHOTTKY_SS34`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "SS32 THRU SS320
  SMA Surface Mount Schottky Barrier Rectifier", Rev 2.0, 3 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420365.html>
  (`SS34`, SMA(DO-214AC)).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/9e763de50d66dc26804ef4c3650380df.pdf>, 259,506 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `f83a234fc3a645c2fe6d495d363a8b1bef79562769c813a72ca58e70f6b98927`.
- Same document as the land's record, `lib/do214/docs/hongjiacheng-ss34.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: page 1 lists SS32 to SS320, each marked with its own name; the
  ratings below are the SS34 column.
- Pins: page 1, "Polarity: Cathode line denotes the cathode end"; the function
  diagram draws the band at the cathode. The sheet does not number the
  terminals; the library puts the cathode on pad 1 (lib/README.md).
- Page 1, "Maximum Ratings" (Ta = 25 C): VRRM 40 V, VRMS 28 V, VDC 40 V,
  IF(AV) 3.0 A (60 Hz sine, resistive load, lead temperature per Fig. 1),
  IFSM 80 A (8.3 ms half sine), Tj and Tstg -55 to +150 C, RthJ-A 80 C/W and
  RthJ-L 20 C/W typical.
- Page 2, "Electrical Characteristics" (Ta = 25 C), the cell shared by SS32,
  SS33 and SS34 (read on the rendered page, since the extracted text does not
  keep the column spans): VF 0.55 V maximum at IF = 3.0 A; IR 0.5 mA maximum
  at VR = VDC and 25 C, 50 mA at 100 C; CJ 500 pF typical at 4.0 V DC, 1 MHz.
- Land fit: page 3 is the primary source of `do214::DIOM5026X241N` (see the
  land's record, and the 1.70 mm land-width waiver in lib/do214/docs/README.md).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
