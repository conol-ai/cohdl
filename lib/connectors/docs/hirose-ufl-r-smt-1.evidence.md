# Hirose U.FL-R-SMT-1(10) evidence

- Manufacturer: Hirose Electric Co., Ltd.
- Exact order code: `U.FL-R-SMT-1(10)`.
- Manufacturer drawing: Hirose drawing `EDC3-302540-10`, revision 2,
  code `CL331-0472-2-10`, dated 2010-12-10 and marked "DRAWING FOR
  REFERENCE" on 2012-11-08. It is a one-page A3 drawing.
- Manufacturer-authored drawing mirror:
  <https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/8942/HIROS08829-1.pdf>.
- Drawing SHA-256:
  `27580d7e4b3c323ea916fa0421ac156e0be62e364cfd70498cf9dfb6ddb6a6c6`.
- Function: vertical 50 ohm U.FL board receptacle. The centre contact is pin 1;
  the two shell solder lands are the same electrical ground contact, pin 2.
- Drawing facts: the single page labels the centre and outer contacts, gives
  the recommended PC-board pattern, and specifies a 1.00 x 1.00 mm centre
  land, two 1.00 x 2.20 mm outer-contact lands, 1.90 mm centre-to-outer-land
  spacing, and 4.00 mm total land-pattern width (all nominal dimensions).
- Geometry cross-check: KiCad's manufacturer-specific
  `Connector_Coaxial.pretty/U.FL_Hirose_U.FL-R-SMT-1_Vertical.kicad_mod`,
  sha256 `ccd686e3e0c2f74cea7236adaad05eafe5da2de547e58b4658fc4f5a1083850c`.
  It records a 1.05 x 1.00 mm signal land at (-1.05, 0) and two
  2.20 x 1.05 mm ground lands at (0.475, +/-1.475), in millimetres. All three
  are 0.25 mm-radius round-rectangles, matching KiCad's locked ratios.
- Qualification boundary: the connector land does not define the surrounding
  transmission line. The consuming board must implement a stackup-specific
  50 ohm feed, ground-via fence and the manufacturer's RF keepout.

The manufacturer drawing is not redistributed by this repository.
