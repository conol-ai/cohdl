# MDD SMBJ series evidence record

- Library coverage: `TVS_SMBJ_Uni` and `TVS_MDD_SMBJ30A`.
- Document identity: Microdiode Semiconductor (MDD), "SMBJ5.0(C)A THRU
  SMBJ440(C)A", stand-off voltage 5.0 to 440 V, peak pulse power 600 W,
  Rev:2025A7, 5 pages.
- JLCPCB part page read on 2026-11-25: C113998, manufacturer
  "MDD(Microdiode Semiconductor)", MPN `SMBJ30A`, package DO-214AA(SMB),
  class Extended, stock 136,164.
- PDF as served by LCSC for C113998:
  <https://datasheet.lcsc.com/datasheet/pdf/2fdb55289569ebfc0bc21eab3853afe6.pdf>,
  1,215,362 bytes. The manufacturer site <http://www.microdiode.com> (printed
  in the page footer) did not complete a TLS connection on 2026-11-25, so
  the LCSC copy is the fallback.
- Retrieved: 2026-11-25.
- SHA-256: `95385273b5009dc9ec2f2a432e07e215f71626b825bd53a9f5525f4186566d3b`.

## Facts used

- Identity: page 1 "Part Number Code" builds the type number as SMBJ,
  voltage, optional C (bidirectional), A (5% VBR tolerance). The sheet
  prints no packing suffix, so the MPN is the bare type number.
- Pins: page 1, "Cathode Band (for Uni-directional products only)" and
  "Polarity: Polarity symbol marking on body". The sheet does not number
  the terminals; the library puts the cathode on pad 1 (lib/README.md).
- Page 1, maximum ratings (25 C): PPPM 600 W minimum at 10/1000 us; IFSM
  100 A (8.3 ms half sine); TJ and TSTG -65 to +150 C.
- Page 2, the SMBJ30A / SMBJ30CA row (markings MK / CK): VRWM 30.0 V, VBR
  33.30 to 36.80 V at IT 1 mA, VC 48.4 V at IPP 12.4 A, IR 1 uA at VRWM.
- Page 1 outline, DO-214AA/SMB, millimetres: overall length 4.90 to 5.60,
  body 4.05 to 4.70 long by 3.30 to 3.95 wide, height 2.00 to 2.45, lead
  width 1.80 to 2.20, lead foot 0.75 to 1.55.
- Page 5, "Suggested Pad Layout": A 2.8 (land width), B 2.4 (land length),
  C 4.6 (centres), D 2.2 (gap), E 7.0 (outer span).
- Land fit: see "SMBJ family", "Land", in README.md.

The PDF is not redistributed by this repository. Download it from the
source above and verify the recorded SHA-256 before auditing the model.
