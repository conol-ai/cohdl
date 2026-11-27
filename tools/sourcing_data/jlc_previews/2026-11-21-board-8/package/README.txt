preview-board-8 preview package, commit 56da519

Upload preview_board_8-gerbers.zip as the Gerber, preview_board_8-lcsc-bom.csv as the BOM and
preview_board_8-lcsc-cpl.csv as the CPL. The Gerber archive holds copper, solder mask,
paste, silkscreen, outline and drill files.

The local checks in verification.json passed. The assembler preview has not
been run.

Expect 64 part numbers and 256 placements: 128 top, 128 bottom.
Check pin 1 (ball A1 on a BGA) on every copy, with no manual rotation
correction. Record a missing part, an ambiguous marker or a failed measurement
as unverified, not passed.

Preview only; do not place an order.

CHECKLIST.md lists each placement. SHA256SUMS covers every other file here.
