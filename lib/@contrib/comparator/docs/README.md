# Comparator source documents

This package provides Texas Instruments `TLV9022DR` and `TLV9032DR` with
the shared `soic::TI_D0008A_4214825C` land from soic 0.3.0.

- [Application contract](tlv90x2-application.md): pin assignments, output
  differences, supporting circuitry, operating limits, and source provenance.
- [Retained TI datasheet](ti-tlv90x2-snosda3h.pdf): SNOSDA3H, November 2025;
  pin table on PDF page 4 and D0008A land drawing on pages 68–69.

The application contract records the original source URL and SHA-256.
The unmodified manufacturer PDF retains Texas Instruments copyright;
the package's MIT license applies to the authored declarations and notes.

## Footprint frame change in 0.2.0

Version 0.2.0 follows soic 0.3.0, which turned `soic::TI_D0008A_4214825C` by a quarter turn (270) onto the
assembler's frame (library plan W3). Pad numbers and lands are unchanged, but
a design that places a part on one of these lands sees its copper turn after
`cohdl update`, with `check` and `build` still passing. Migrate the design's
`place` statements first, as the land package's README describes, with
`tools/realign_place.py`.
