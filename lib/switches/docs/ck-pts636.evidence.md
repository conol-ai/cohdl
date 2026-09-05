# C&K PTS636SM25SMTRLFS evidence

- Manufacturer: C&K (Littelfuse).
- Exact order code: `PTS636SM25SMTRLFS`.
- Manufacturer datasheet: "PTS636 Series — 6.0 x 3.5 mm Top Actuated
  Switches", revision `VL 03/14/25`, 7 pages. Official source:
  <https://www.ckswitches.com/media/2779/pts636.pdf>; archived byte-preserving
  retrieval used for audit:
  <https://web.archive.org/web/20260206175834id_/https://www.ckswitches.com/media/2779/pts636.pdf>.
- Datasheet SHA-256:
  `3de81710d500f0d55da6fee16d49893d8fe163d644e4d4779ec565066737df54`.
- Function: surface-mount, straight-actuated, momentary SPST normally-open
  tactile switch; 6.0 x 3.5 mm body and 2.5 mm actuator height.
- Datasheet facts: page 1 states momentary `SPST, N.O.`, 12 VDC / 50 mA,
  180 gf nominal force and 70,000 operations for the 2.5 mm M-force variant;
  page 3 gives the 2.5 mm SMT gull-wing package and PCB land dimensions;
  page 7 defines the ordering fields `S` straight, `M` 180 gf, `25` 2.5 mm,
  `SMTR` surface-mount tape-and-reel, and `LFS` RoHS/silver-plated.
- Geometry cross-check: KiCad's manufacturer-specific
  `SW_Tactile_SPST_NO_Straight_CK_PTS636Sx25SMTRLFS.kicad_mod`, sha256
  `0d5c20cbfee0a31e23daaff0386231bb258bc0a7c3969ca9e32020c1c6dcd923`.
  It records two 1.25 x 1.00 mm lands centred 7.75 mm apart.
- Qualification boundary: this is a momentary pushbutton, not a maintained
  slide switch. It is intended to drive a debounced controller such as the
  MAX16054; it cannot directly replace a maintained-power switch function.

The manufacturer datasheet is not redistributed by this repository.
