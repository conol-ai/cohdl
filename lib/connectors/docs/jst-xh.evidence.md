# JST S2B-XH-A(LF)(SN) evidence

- Manufacturer: J.S.T. Mfg. Co., Ltd. (JST).
- Exact order code: `S2B-XH-A(LF)(SN)` (LCSC C157931), the 2-circuit
  side-entry XH header with C = 9.2 mm. `(LF)(SN)` is a label marking, not
  a dimensional variant (p. 5, note 1).
- Library coverage: `BAT_CONN_2P`, `CONN_BAT_2P`, `FP_BAT_2P_2P5` and
  `P_BAT_2P_PAD` in `src/fpc/fpc.cohdl`.
- Manufacturer document: local `jst-xh.pdf`, the JST "XH CONNECTOR"
  catalogue, 9 pages, A4, created 2025-07-30 (PDF metadata). Vendored
  unmodified in 82ada62 (2026-08-06) and unchanged since.
- Source: <https://www.jst-mfg.com/product/pdf/eng/eXH.pdf>.
- Original PDF SHA-256:
  `9426b136902f11900825077535e5c65032b7fbc31ffb59c5e9e1f463bb20fb90`.
- Page numbers below are the PDF's own printed page numbers, which equal
  the PDF page indices.

## Reading the local PDF

The PDF is AES-256 encrypted with an owner password only (open, print and
copy allowed) and embeds three fonts: `FuturaPT-Bold` (subset) and two CID
Type 0C gothic fonts. Some viewers show the drawings and tables on pp. 2-6
blank. Poppler renders and extracts all of them, provided its data files
are installed (see the prerequisite below). Every value in this record was
read from this file with poppler 26.06.0, in two independent ways:

```sh
pdftotext -layout -f 2 -l 2 jst-xh.pdf -    # the same for pp. 1, 5 and 6
pdftoppm -r 300 -f 2 -l 2 -png jst-xh.pdf p # also pdftocairo -png -r 300
```

### Prerequisite: poppler with `poppler-data`

Poppler alone is not enough. All the dimension text is set in the two CID
gothic fonts, and poppler can only use them with the Adobe-Japan1 CMaps
from the separate `poppler-data` project (the `cMap/Adobe-Japan1`
directory under poppler's data directory, usually `share/poppler`).

Without `poppler-data`, the commands above still exit 0, but:

- `pdftotext` and `pdftoppm` print
  `Syntax Error: Missing language pack for 'Adobe-Japan1' mapping`, twice
  for p. 2 alone and 18 times for the whole file, among `Unknown font tag
  'G1'` / `'G2'` and `No font in show` errors;
- the text of p. 2 is only the heading `XH CONNECTOR` (14 bytes, against
  7,448 with the data installed);
- the rendering shows the outlines, holes and dimension lines with every
  number and label missing. Only the `XH CONNECTOR` heading and the JST
  logo remain.

That is the "blank dimensions" result of the #105 review. It was reproduced
for this record on 2026-11-11 by running the same poppler 26.06.0 build
with its data directory pointed at a path that does not exist.

Whether `poppler-data` is present depends on how poppler was installed.
The Homebrew `poppler` formula, used for this record, bundles it (there is
no separate formula). On Debian and Ubuntu it is the separate
`poppler-data` package beside `poppler-utils`; that was not run for this
record. To check an installation before reading any value:

```sh
pdftotext -layout jst-xh.pdf /dev/null       # must print nothing
pdftotext -layout -f 5 -l 5 jst-xh.pdf - | grep 'S2B-XH-A '
pdftotext -layout -f 2 -l 2 jst-xh.pdf - | grep -e '(14.3)' -e '2.5 ±0.05'
```

The first command must print no `Missing language pack` line (it printed
nothing here). The second must print the 2-circuit row of the p. 5
side-entry table, which reads, with the column spacing reduced:

```text
2    S2B-XH-A    S2B-XH-A-1    2.5    7.4    1,000
```

The third must print the p. 2 lines that carry `(14.3)` and `2.5 ±0.05`.
If any of the three differs, install `poppler-data` and run them again;
the values in the table below cannot be read until they pass.

### What was read

The text layer and the renderings agree on every value. Pages 1, 2, 5 and 6
were rendered whole at 100 dpi, and the p. 2 side-entry layout and assembly
drawing and the p. 5 side-entry drawing and table again at 300 dpi. No
rendering is committed: the vendored PDF itself is the legible copy, and
LIBRARY.md asks that rendered PDF pages are not left in the tree.

## Values used and where they are

All dimensions are in millimetres.

| Value | Where in `jst-xh.pdf` | Used for |
| --- | --- | --- |
| Hole 0.9 +0.1/-0, reference value | p. 2, "PC board layout and Assembly layout", Side entry type | drill 1.0 (top of range) |
| Hole pitch 2.5 +/-0.05, not accumulating | p. 2, Side entry type; note 2 | pads at x = -1.25 and +1.25 |
| Larger hole on fiberglass-cloth board | p. 2, note 4 | drill at the top of the range |
| Layout viewed from the connector mounting surface | p. 2, note 1 | orientation (below) |
| No. 1 circuit is the right-hand hole; header outline above the hole row | p. 2, Side entry type layout | orientation (below) |
| (C), hole row to the far side edge of the header | p. 2, Side entry type layout; note 3 | body extent, mating side |
| Mated length (14.3) over 4.5 + 7 | p. 2, Side entry assembly drawing | mated XHP keep-clear: 14.3 - 11.5 = 2.8 |
| Post section 0.64 square | p. 5, Header, Side entry type <2 circuits> | drill clears the 0.905 post diagonal |
| S2B-XH-A: A = 2.5, B = 7.4 | p. 5, Side entry type table, 2 circuits | body width, silkscreen, courtyard |
| C = 9.2 for S2B-XH-A (7.6 is S2B-XH-A-1) | p. 5, Side entry type table header; drawing "C: 9.2/7.6" | hole row to mating edge |
| Body depth 7 + 4.5 = 11.5 | p. 5, Side entry type <2 circuits> side view | body behind the hole row: 11.5 - 9.2 = 2.3 |
| Body height (6.1), post tail (3.4) | p. 5, side view; p. 2, assembly drawing | not modelled (3D only) |
| No 2-circuit SMT XH header (SMT: 3, 4 and 6 circuits only) | p. 6, Header/ SMT type table | why the land is through-hole |
| 3 A AC/DC (AWG #22), 250 V AC/DC, -25 to +85 C including temperature rise | p. 1, Specifications | `BAT_CONN_2P` device comment |
| Applicable PC board thickness 1.6 | p. 1, Specifications | consuming board (not modelled) |

Note 3 on p. 2 refers to the "Header/ Side entry type" section "on page 4";
that table is on printed p. 5 of this edition. The values above are from
p. 5.

The top-entry layouts on p. 2 give 2-circuit headers a 1 +/-0.05 hole.
That does not apply here: the side-entry layout gives 0.9 +0.1/-0 for
every circuit count.

## Footprint implementation

The p. 2 side-entry layout is viewed from the mounting surface with the
header outline above the hole row and No. 1 circuit on the right. It is
rotated 180 degrees into footprint coordinates without reflection, so pad
1 is at -x and the body extends towards +y. JST does not dimension the
body's lateral position; the p. 2 outline and the p. 5 front view are
drawn symmetric about the holes, so B is centred on them.

| Feature | JST value | Implementation | Deviation |
| --- | --- | --- | --- |
| Pads 1, 2 | 2 holes at 2.5 pitch | plated through-hole at x = -1.25, +1.25, y = 0 | 0.00 |
| Drill | 0.9 +0.1/-0 | 1.0 | at the upper limit, by choice (note 4) |
| Copper land | not given | 1.7 round: 0.35 annulus, 0.8 gap between lands | library choice |
| Body | B 7.4 wide; C 9.2 to the mating edge; 11.5 deep | x -3.7 to +3.7, y -2.3 to +9.2 | 0.00 |
| Silkscreen | - | body outline 0.11 outside: x +/-3.81, y -2.41 to +9.31 | library choice |
| Courtyard | - | body plus 0.5: 8.4 x 12.5 centred at (0, 3.45) | library choice |

The courtyard does not include the mated XHP housing, which reaches about
2.8 past the mating edge (+y), or the wires that leave that way. Keep that
side clear.

## Qualification boundary

JST calls the hole sizes reference values and asks to be contacted for the
target board and drilling process (p. 2, note 4). The copper land, the
mask and paste rules, the 1.6 mm board thickness (p. 1), the clearance for
the mated housing and harness, and the battery polarity must be checked on
the assembled board before production release. The polarity is this
library's assignment: JST assigns no function to the circuits.
