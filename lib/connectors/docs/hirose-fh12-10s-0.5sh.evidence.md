# Hirose FH12-10S-0.5SH(55) evidence

- Manufacturer: Hirose Electric Co., Ltd.
- Exact order code: `FH12-10S-0.5SH(55)`, CL `586-0522-3-55`.
- Manufacturer document: local `fh12-series.pdf` (FH12 catalogue, Nov. 2016),
  sha256 `9f056f58ff000739633cc3cb7697ba185967083dcf1429b7ab10783b3f55bac5`.
  Page 4 is the 0.5 mm bottom-contact product drawing and ordering table.
  Page 11, "Land / Metal Mask dimensions", 0.5 mm pitch bottom contact type,
  is the recommended land pattern; for 10 contacts its table gives
  B = 10.1 mm (outer hold-down span) and C = 6.5 mm (inner hold-down span).
- Geometry cross-check: KiCad's manufacturer-specific
  `Connector_FFC-FPC.pretty/Hirose_FH12-10S-0.5SH_1x10-1MP_P0.50mm_Horizontal.kicad_mod`
  (KiCad 10.0.7 library), sha256 `d5c17a1d8aba3c2acff865e9f652384131323b4f032b38cbdaba2d7f8a21d2bb`.
- Footprint origin: the midpoint between the two hold-down centres, as in
  KiCad. Contacts lie towards -y.

## Recommended lands and implementation

All dimensions and coordinates are in millimetres. Contact x follows
`0.5 * (n - 5.5)` for contacts 1 through 10. Page 11 gives the
contact land 0.3 x 1.3, the hold-down land 2.2 high and (B - C) / 2 = 1.8
wide, and 1.5 between the hold-down and contact lands, so the two land rows
are 0.65 + 1.5 + 1.1 = 3.25 apart.

| Pad(s) | Page 11 / KiCad | Implementation | Maximum deviation |
| --- | --- | --- | --- |
| 1-10 | 0.30 x 1.30, y = -1.85, 0.50 pitch | 0.30 x 1.30, y = -1.85, 0.50 pitch | 0.00 |
| MP left | 1.80 x 2.20 at (-(B + C) / 4, 1.40) = (-4.15, 1.40) | 1.80 x 2.20 at (-4.15, 1.40) | 0.00 |
| MP right | 1.80 x 2.20 at (4.15, 1.40) | 1.80 x 2.20 at (4.15, 1.40) | 0.00 |

The two mechanical hold-down lands share the passive `MP` device pin. The
courtyard is KiCad's, x +/-5.56 and y -3.00 to 4.90, which contains the
connector body and actuator; it is 11.12 x 7.9 centred at (0, 0.95).
