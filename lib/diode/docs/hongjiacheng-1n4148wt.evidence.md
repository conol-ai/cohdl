# Hongjiacheng 1N4148WT evidence record (part)

- Library coverage: `Diode_1N4148WT` and `D_1N4148WT`.
- Document identity: Zhuhai Hongjiacheng Technology Co., Ltd, "1N4148WT Fast
  Switching Diodes" (the sheet spells it "SWITHING"), Rev 2.0, 3 pages.
- Authorized-distributor product page: <https://www.lcsc.com/product-detail/C7420320.html>
  (`1N4148WT`, SOD-523).
- PDF as served by LCSC: <https://datasheet.lcsc.com/datasheet/pdf/0c7f487bdf1474cd2924fe5595db8ec0.pdf>, 182,398 bytes.
- Retrieved: 2026-11-07. Nothing was fetched for this record: the values below
  were read on 2026-11-11 from the copy retrieved that day, after checking its
  SHA-256.
- SHA-256: `71312a1ff0a0911bd02cb41941bbc00a9abdea0146b9843db0cd9ad03fe8365a`.
- Same document as the land's record, `lib/sod/docs/hongjiacheng-1n4148wt.evidence.md`, which holds the
  manufacturer-site note, the package outline and the land dimensions.

## Facts used

- Identity: the sheet covers 1N4148WT only (marking T4).
- Pins: page 1, "Polarity: Cathode line denotes the cathode end". The sheet
  does not number the terminals; the library puts the cathode on pad 1
  (lib/README.md).
- Page 1, "Maximum Ratings" (Ta = 25 C): VRRM 100 V, VRMS 75 V, V(BR)R 100 V
  at IR = 100 uA, IF(AV) 150 mA, IFRM 300 mA, IFSM 2.0 A (tp = 1.0 us) and
  1.0 A (tp = 1.0 s), Pd 150 mW, Tj and Tstg -55 to +150 C, RthJ-A 833 C/W
  typical.
- Page 2, "Electrical Characteristics" (Ta = 25 C), maxima only: VF 0.715 V
  at 1.0 mA, 0.855 V at 10 mA, 1.0 V at 50 mA, 1.25 V at 150 mA; IR 25 nA at
  VR = 20 V and 1.0 uA at VR = 75 V; CT 2.0 pF at VR = 0 V, f = 1 MHz; trr
  4.0 ns (IF = IR = 10 mA, Irr = 0.1 x IR, RL = 100 ohm).
- Land fit: page 3 is the primary source of `sod::SOD1608X70N` (see the land's
  record).

The PDF is not redistributed by this repository. Download it from the source
above and verify the recorded SHA-256 before auditing the library model.
