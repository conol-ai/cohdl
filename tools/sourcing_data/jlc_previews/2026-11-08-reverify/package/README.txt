<scratch>/reprev/out/pr78-101-reverify preview package, commit 6a68c88

Upload pr78_101_reverify-gerbers.zip as the Gerber, pr78_101_reverify-lcsc-bom.csv as the BOM and
pr78_101_reverify-lcsc-cpl.csv as the CPL. The Gerber archive holds copper, solder mask,
paste, silkscreen, outline and drill files.

The local checks in verification.json passed. The assembler preview has not
been run.

Expect 4 part numbers and 16 placements: 8 top, 8 bottom.
Check pin 1 (ball A1 on a BGA) on every copy, with no manual rotation
correction. Record a missing part, an ambiguous marker or a failed measurement
as unverified, not passed.

Preview only; do not place an order.

CHECKLIST.md lists each placement. SHA256SUMS covers every other file here.
