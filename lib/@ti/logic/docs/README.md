# Texas Instruments logic reference documents

Retrieved 2026-08-05 from Texas Instruments. The PDF is stored byte-for-byte
as downloaded.

## `sn74lvc8t245-datasheet.pdf`

- Covers: `SN74LVC8T245PWR` in the 24-pin PW TSSOP package.
- Document identity: SCES584D, June 2005 revised May 2026.
- Official URL:
  <https://www.ti.com/lit/ds/symlink/sn74lvc8t245.pdf>
- SHA-256:
  `d3de8e28727828613d1bd143d892b7189409167c66c513eb0ac2e0664de0c456`
- Pin authority: Table 4-1 on PDF page 5.
- Footprint authority: official PW0024A package outline and example board
  layout 4220208/A on PDF pages 30-31. The board layout specifies 24 lands,
  each 1.50 x 0.45 mm, at 0.65 mm pitch with 5.80 mm opposing row-centre
  spacing.

## Footprint frame change in 0.2.0

Version 0.2.0 follows soic 0.3.0, which turned `soic::TSSOP24P65_TI_PW0024A` by a quarter turn (270) onto the
assembler's frame (library plan W3). Pad numbers and lands are unchanged, but
a design that places a part on one of these lands sees its copper turn after
`cohdl update`, with `check` and `build` still passing. Migrate the design's
`place` statements first, as the land package's README describes, with
`tools/realign_place.py`.
