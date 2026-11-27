preview-board-15 preview package, commit 719c0a0

Upload preview_board_15-gerbers.zip as the Gerber, preview_board_15-lcsc-bom.csv as the BOM and
preview_board_15-lcsc-cpl.csv as the CPL. The Gerber archive holds copper, solder mask,
paste, silkscreen, outline and drill files.

The local checks in verification.json passed. The assembler preview has not
been run.

Expect 45 part numbers and 180 placements: 90 top, 90 bottom.
Check pin 1 (ball A1 on a BGA) on every copy, with no manual rotation
correction. Record a missing part, an ambiguous marker or a failed measurement
as unverified, not passed.

Preview only; do not place an order.

CHECKLIST.md lists each placement. SHA256SUMS covers every other file here.
