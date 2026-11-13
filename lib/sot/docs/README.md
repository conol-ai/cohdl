# `sot`: SOT land patterns

Reusable SOT land patterns for new parts (library plan W4, item 3c, waves 1
and 3). This package ships lands only: no device, no part and no supplier
binding. Bindings come after the JLCPCB frame preview boards. One land per
standard outline is taken from one primary document and cross-checked against
one tier-1 manufacturer document; when the primary gives no usable land, the
land is derived from the cross-check document and says so. Each part series'
own datasheet (and its pin map: BEC, GSD, diode pairs, regulator pins ...)
is checked when that series is bound.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. SOT-23-3 uses the frame of `transistor::FP_JSCJ_SOT23` 0.2.0, which
the assembler measured on the 2026-10-30 preview board
(`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`): the datasheet
top view turned 90 degrees so that **pins 1 and 2 form the +x column, pin 1
at +y (lower right), pin 2 at -y, and pin 3 alone at -x**. Numbering stays
counter-clockwise 1-2-3 seen from the top. The mirrored arrangement (pin 1 at
-y) swaps pins 1 and 2 on the board and is rejected by
`tests/library/package_families.rs`. The silkscreen carries
`pin_1_marker near pad 1 shape dot`, which lands beyond pad 1 on the +y side.

**Wave 3 uses the same turn for every SOT outline**: a point (x, y) of the
datasheet top view (leads on the lower edge, pin 1 at the lower left) is
placed at (y, -x). The edge that carries pin 1 faces +x with pin 1 at +y; the
opposite edge faces -x.

| Outline | +x column (from +y to -y) | -x side |
|---|---|---|
| SOT-23-3, SOT-323 | pin 1, pin 2 | pin 3 |
| SOT-89-3 | pin 1, pin 2, pin 3 (lead row) | tab (pad 2) |
| SOT-223 | pin 1, pin 2, pin 3 (lead row) | tab (pad 4) |

The same turn of KiCad's default SOT-89 and SOT-223 footprints (lead column
at -x, pin 1 at -y) is a 180-degree rotation, which is also what turns
KiCad's default SOT-23 onto the measured `FP_JSCJ_SOT23` frame. The SOT-89,
SOT-223 and SOT-323 frames are therefore inferred, not measured; the wave-3
frame board must read them (open question 3). A mirrored land (pin 1 at -y)
is rejected by `tests/library/package_families.rs` for every SOT land here.

## Tab numbering

Rule: **a tab that is one piece of metal with a lead takes that lead's
number; a tab that is a separate terminal takes the next number.** The rule
was agreed in the #101 review and is written in `LIBRARY.md` section 4, so
later tabbed lands (DPAK, SOT-89-5) follow it.

- **SOT-89-3: the tab is a second placement of pad 2.** The Diodes bottom
  view and the Hongjiacheng outline both show the tab and the centre lead as
  one lead frame, and the Diodes land joins them in one T-shaped copper area.
  The library's only other SOT-89 land, `contrib_ldo::FP_ETA_SOT89_5`,
  likewise numbers its merged centre land `2`. A device maps the collector
  (or drain, or the regulator's centre pin) as pin 2 alone; it cannot give
  the tab a different net. Three distinct pad numbers, so the name reads
  `SOT3`.
- **SOT-223: the tab is pad 4.** It is a separate terminal on the opposite
  edge (JEDEC TO-261AA is a four-terminal outline), drawn as a separate land
  by Diodes. AMS labels it "TAB IS OUTPUT" without a number; other vendors
  call it pin 4 or a second pin 2. Making it pad 4 forces every SOT-223
  device to state the tab's connection (E807 rejects a device that omits
  pad 4): AMS1117 binds as `VOUT: 2, 4`. Four distinct pad numbers, so the
  name reads `SOT4`.

`tests/library/package_families.rs` checks that renumbering either tab
against this rule fails RFC-021's pin-count check (E809), and the wave-3
fixture consumer binds a regulator with `VOUT: 2, 4`. That guard works
through the footprint *name*: E809 compares the name's pin count (`SOT3`,
`SOT4`) with the distinct pad numbers, so renaming a footprint (or changing
its pin count together with the tab) removes it. See open question 4.

## Lands

| Footprint | Outline | Lands | Pad centres | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|
| `sot::SOT3P190X290X130N` | SOT-23-3 (TO-236AB) | 0.80 x 0.90 mm declared, placed `rotate 270` (0.90 in x, 0.80 in y) | 1 (1.00, 0.95), 2 (1.00, -0.95), 3 (-1.00, 0) mm | 3.4 x 3.6 mm | 2.55 x 3.00 mm (E1 x D max, in this frame) |
| `sot::SOT3P150X450X250N` | SOT-89-3 (TO-243AA) | leads 0.58 x 1.63 mm; lead-2 stem 0.76 x 1.63 mm; tab 1.933 x 3.03 mm; all declared in datasheet orientation and placed `rotate 270` | 1 (1.45, 1.50), 2 stem (1.45, 0), 2 tab (-0.75, 0), 3 (1.45, -1.50) mm | 5.1 x 5.2 mm | 4.10 x 4.60 mm (C x A max, in this frame) |
| `sot::SOT3P130X200X125N` | SOT-323 / SC-70-3 | 0.70 x 0.90 mm declared, placed `rotate 270` | 1 (0.95, 0.65), 2 (0.95, -0.65), 3 (-0.95, 0) mm | 3.3 x 2.8 mm | 2.45 x 2.20 mm (E1 x D max, in this frame) |
| `sot::SOT4P230X651X351N` | SOT-223 (TO-261AA) | leads 1.20 x 1.60 mm; tab 3.30 x 1.60 mm; declared in datasheet orientation, placed `rotate 270` | 1 (3.20, 2.30), 2 (3.20, 0), 3 (3.20, -2.30), 4 tab (-3.20, 0) mm | 8.5 x 7.3 mm | 7.29 x 6.71 mm (tab tip to lead tip x body length max, in this frame) |

Wave 1 added SOT-23-3; wave 3 added SOT-89-3, SOT-323 and SOT-223. The SOT-89
and SOT-223 lands come from the Diodes cross-checks (see below). Each
courtyard contains the copper and the maximum component envelope with at
least 0.25 mm on every side (checked by `tests/library/geometry.rs`); the
SOT-89 and SOT-223 envelopes are centred on the land, whose toe and tab
extensions are symmetric.

The names follow RFC-021's checked template `SOT{pins}P{pitch}X{bodyX}X{bodyY}N`.
E809 reads the pins as the distinct pad numbers and the pitch as the closest
centre spacing between distinct pad numbers:

- SOT-23-3: pins 1-2 at 1.90 mm, so `P190`. Body digits are the primary
  outline's nominal D 2.90 and E 1.30 mm.
- SOT-89-3: pins 1-2 and 2-3 at 1.50 mm, so `P150`. The lead-2 stem is
  declared 1.63 mm long (the outer lands' length, 0.13 mm more than Diodes'
  Y2) so that its centre lines up with pads 1 and 3; the extra copper lies
  inside the tab land, so the copper is exactly Diodes' T, and the stem-to-
  pad-1 spacing is exactly 1.50 mm (with Diodes' 1.50 mm stem it would be
  1.501 mm and E809 would fire). Body digits: Hongjiacheng A 4.40-4.60 and
  D 2.40-2.60, nominal 4.50 and 2.50.
- SOT-323: pins 1-2 at 1.30 mm, so `P130`. Body digits: Hongjiacheng D
  1.80-2.20 and E 1.15-1.35, nominal 2.00 and 1.25.
- SOT-223: pins 1-2 at 2.30 mm, so `P230`; four pad numbers, so `SOT4`. Body
  digits: AMS body length 6.30-6.71 and width 3.30-3.71 mm, nominal 6.505
  and 3.505, rounded half up to 6.51 and 3.51 (open question 5). **Where
  651/351 come from:** AMS draws the outline in inches (0.248-0.264 x
  0.130-0.146 in) and prints millimetre conversions, each rounded to
  0.01 mm, beside them. The name takes the midpoint of that rounded
  millimetre column: (6.30 + 6.71) / 2 = 6.505, which rounds half up to
  6.51. The inch midpoints themselves give 6.50 and 3.51 (0.256 in =
  6.5024 mm, 0.138 in = 3.5052 mm), so the 651 is an artefact of taking
  the midpoint of AMS's rounded millimetre values, not a different body.
  The JEDEC / typical nominal, Diodes' typical D 6.50 x E 3.50 mm, would
  give `650X350`. The name was published as `sot::SOT4P230X651X351N` in
  sot 0.1.0 and is now permanent; the geometry does not depend on it.

## Sources

The PDFs are not redistributed; each `.evidence.md` file records the document
identity, both URLs, the 2026-11-07 retrieval, the SHA-256 and the exact
page and dimensions used. The Hongjiacheng and AMS documents are
manufacturer PDFs served by LCSC; the maintainer approved that fallback for
these lands.

| Record | Role | Document | SHA-256 |
|---|---|---|---|
| `hongjiacheng-mmbt5551.evidence.md` | SOT-23-3 primary | Hongjiacheng MMBT5551 Rev 2.0, p3 (LCSC C7420357) | `7723e0fc4bf81c91d1d965a3fa04f287d74654013bfdc0efa3cd5c35a8c317f9` |
| `diodes-sot23.evidence.md` | SOT-23-3 cross-check | Diodes SOT23 Package Information Rev. 2017-01-30, p1 | `d4843d22b6c5502465b699ec9a5e42a83dbf54dd286b0eaa9e630956841b7608` |
| `hongjiacheng-d882.evidence.md` | SOT-89-3 primary (outline; its land does not fit) | Hongjiacheng D882 Rev 2.0, p3 (LCSC C20069128) | `fcd194b90c4682646af42a0999038147208230a342d1a2d5379b64910a937c35` |
| `diodes-sot89.evidence.md` | SOT-89-3 land source and cross-check | Diodes SOT89 Package Information Rev. 2026-01-16, p1 | `7e918198860b429583dadb78657bc0e721091e6835311523aeb69f045724440a` |
| `hongjiacheng-2n7002w.evidence.md` | SOT-323 primary | Hongjiacheng 2N7002W Rev 2.0, p5 (LCSC C7420322) | `def7318e3220b19ed9e8061b02294c21b8d17f2f88ebfaa91de71cb06c111451` |
| `diodes-sot323.evidence.md` | SOT-323 cross-check | Diodes SOT323 Package Information Rev. 2017-03-20, p1 | `4ef5f66cccb307fedd8b60dac6202b906a2291f84836dac592e71680d71ac6ee` |
| `ams-ams1117.evidence.md` | SOT-223 primary (outline only) | AMS AMS1117 datasheet (no revision printed, PDF of 2009-08-27), p1 and p7 (LCSC C6186) | `189a2651878a87d590b768eaa9b44217a3fdf460352ce6ecaff127221282a3f0` |
| `diodes-sot223.evidence.md` | SOT-223 land source and cross-check | Diodes SOT223 Package Information Rev. 2017-03-06, p1 | `c8d9447a1925136b5d045767c8f0177ea9144c3c8bac5acb23ea3f9343589d19` |

## Primary versus cross-check

Hongjiacheng gives ranges (J 0.75-0.85, K 0.85-0.95, M 1.95-2.05, N 1.85-1.95
mm); the land uses the midpoints.

| | Hongjiacheng (this land) | Diodes package file | `transistor::FP_JSCJ_SOT23` (JSCJ S8050) |
|---|---|---|---|
| Land along the row | 0.80 | 0.80 (X) | 0.60 |
| Land across the row | 0.90 | 0.90 (Y) | 0.80 |
| Pin 1-2 spacing | 1.90 | 1.90 (2 x (X1 - X/2)) | 1.90 |
| Row spacing | 2.00 | 2.00 (C) | 2.02 |

The primary midpoints and the Diodes layout are identical. The outlines agree
(body 2.80-3.00 x 1.20-1.40 mm in both; lead span 2.25-2.55 versus Diodes
2.30-2.50 mm). The JSCJ land is smaller (0.6 x 0.8 mm) with 0.02 mm wider rows.

**SOT-89-3.** The Hongjiacheng D882 sheet does draw a "Suggested Pad
Layout", but it cannot be placed on its own outline: lead lands d
(1.30-1.50 mm long), a gap J (1.80-2.00 mm) and an octagonal tab land Y
(4.30-4.50 mm) stack to 7.40-8.00 mm against the 3.90-4.10 mm package (C),
or 5.60-6.20 mm with X and Y exchanged, and no copper joins lead 2 to the
tab. The land is therefore **derived from the Diodes SOT89 package file**.
Measured in the datasheet orientation (leads down):

| Land (mm) | This land (Diodes values) | Hongjiacheng D882 (not used) |
|---|---|---|
| Outer lead lands | 0.58 x 1.63 | c 0.80-1.00 x d 1.30-1.50 (e 0.70-0.90) |
| Lead pitch | 1.50 | K 1.40-1.60 |
| Lead-2 land | 0.76 wide, joined to the tab land | separate lead land, 1.80-2.00 from the tab land |
| Tab land | 1.933 x 3.03 rectangle | X 2.50-2.70 x Y 4.30-4.50 octagon |
| Overall copper height | 4.53 | 7.40-8.00 |

| Outline (mm) | Hongjiacheng D882 (primary) | Diodes SOT89 |
|---|---|---|
| Body length | 4.4-4.6 (A) | 4.40-4.60 (D) |
| Body width | 2.4-2.6 (D) | 2.40-2.60 (E) |
| Tab tip to lead tip | 3.9-4.1 (C) | 3.95-4.25 (H) |
| Tab width | 1.6-1.8 (B) | 1.62-1.83 (D1) |
| Lead width | 0.41-0.43 (a, pin 1), 0.35-0.45 (b, pin 2) | 0.42-0.54 (B1), 0.50-0.62 (B) |
| Lead length | 0.95-1.05 (L) | 0.90-1.20 (L) |
| Height | 1.4-1.5 (H) | 1.40-1.60 (A) |

The outlines agree within 0.15 mm; Diodes' centre lead is wider and its
overall length up to 0.15 mm longer. On the Hongjiacheng outline (centred on
the land) the lead tips sit 0.22-0.32 mm inside the 2.265 mm outer land
edge, the tab tip sits as far inside the tab land's outer edge, each lead
(L 0.95-1.05 mm, from 0.90-1.10 mm out to the tip) lies wholly on its land
(inner end 0.635 mm from the centre), and the 1.8 mm maximum tab and 0.46 mm
maximum leads fit the 1.933 mm and 0.58 mm lands.

**SOT-323.** Hongjiacheng gives ranges (J 0.65-0.75, K 0.85-0.95, M
1.85-1.95, N 1.25-1.35 mm); the land uses the midpoints. Diodes gives X 0.47,
Y 0.60, C 0.65, G 1.30, Y1 2.50 mm.

| | Hongjiacheng (this land) | Diodes package file |
|---|---|---|
| Land along the row | 0.70 | 0.47 (X) |
| Land across the row | 0.90 | 0.60 (Y) |
| Pin 1-2 spacing | 1.30 | 1.30 (2 x C) |
| Row spacing | 1.90 | 1.90 (G + Y) |
| Overall copper across the rows | 2.80 | 2.50 (Y1) |

The centres are identical; the Diodes lands are markedly smaller (0.23 mm
narrower, 0.30 mm shorter), as Diodes' SOD-323 land was in wave 1. The
outlines agree on body (D 1.80-2.20 and 1.15-1.35 mm in both) and pitch;
Hongjiacheng's lead span is longer (E1 2.15-2.45 against Diodes E 2.00-2.20
mm) and its leads may be narrower (b 0.15-0.40 against 0.25-0.40 mm).

**SOT-223.** The AMS1117 datasheet gives the outline only, with no land on
any page, so the land is **derived from the Diodes SOT223 package file**
(X 1.20, Y 1.60, C 2.30, X1 3.30, Y1 1.60, C1 6.40, Y2 8.00 mm), and the
comparison is between outlines:

| Outline (mm) | AMS1117 (primary) | Diodes SOT223 |
|---|---|---|
| Body length | 6.30-6.71 | 6.45-6.55 (D) |
| Body width | 3.30-3.71 | 3.45-3.55 (E) |
| Tab tip to lead tip | 6.71-7.29 | 6.90-7.10 (E1) |
| Tab width | 2.95-3.15 | 2.90-3.10 (b1) |
| Lead pitch | 2.29 nom (0.090 in) | 2.30 (e1) |
| Pin 1 to pin 3 | 4.60 nom | 4.60 (e) |
| Lead width | 0.64-0.84 | 0.60-0.80 (b) |
| Lead foot | 0.64-0.84 | 0.85-1.05 (L) |
| Height | 1.80 max | 1.55-1.65 (A) + 0.15 max (A1) |

AMS's tolerances are wider (it is a 2009 inch-dimensioned drawing). On the
AMS outline the lead tips sit 0.36-0.65 mm inside the 4.00 mm outer land edge,
every lead foot's inner end stays outboard of the 2.40 mm inner land edge, and
the 3.15 mm maximum tab leaves 0.075 mm per side on the 3.30 mm tab land.

## Existing lands for the same outline

None in the library for SOT-89-3, SOT-323 (three-lead) or SOT-223.
`contrib_ldo::FP_ETA_SOT89_5` is the five-lead SOT89-5, a different outline
(it is the precedent for numbering the SOT-89 centre land `2`), and
`contrib_analog_switch::SOT6P65X210X125N` is the six-lead SC70-6 / SOT-363.
For SOT-23-3 these lands are not moved or changed:

- `transistor::FP_JSCJ_SOT23` (JSCJ S8050 land, on the same frame, with three
  direct frame readings) remains for `transistor::BJT_JSCJ_S8050_J3Y`.
- `contrib_ldo::SOT3P190X290X160N` (Torex-envelope derivation, @contrib) stays
  with its package. Its name differs in the body-width field (1.60 versus 1.30
  mm), so the two are not a name collision.

## Open questions (for the maintainers)

1. **Body field of SOT names.** Existing SOT names mix body width and height
   in the last field (`SOT5P95X290X160N` versus `SOT5P95X290X145N`). This land
   uses body width (1.30 mm). *Recommendation:* body width, as RFC-021's
   template (`{bX}X{bY}`) says; record that in RFC-021's notes.
2. **Duplicates.** SOT-23-3 now has three lands (`sot`, `transistor`,
   `@contrib/ldo`). *Recommendation:* keep them (3c plan option (a)) until
   re-exports exist; bind new SOT-23-3 parts to `sot::SOT3P190X290X130N`.
3. **Frame confirmation.** The frame is copied from `FP_JSCJ_SOT23`'s
   measured frame, not measured on this footprint. sot 0.1.0 was published
   (2026-11-09) without that reading, so this is an open item on a published
   package: the frame remains unconfirmed until the wave-1 frame board reads
   it. The SOT-89,
   SOT-323 and SOT-223 frames apply the same turn by analogy (lead row at
   +x, tab at -x); the 3c plan suggested authoring SOT-89 and SOT-223 in the
   datasheet's own top view instead. *Recommendation:* keep one rule for the
   whole family, and let the wave-3 frame board (C9634 JSCJ D882, C20069128,
   C7420322, C6186/C6187 AMS1117) read it; rotate a land only if every
   model reads the same non-zero angle. Those frames are unconfirmed too,
   and a change to pad numbering or rotation would be a breaking change
   needing a new minor version.
4. **Tab numbering.** *Settled in the #101 review: the rule is adopted and
   written into `LIBRARY.md` section 4.* SOT-89 tab = second placement of
   pad 2 (verified on the Diodes bottom view); SOT-223 tab = pad 4 (rule
   above). This fixes the pin maps the discretes and ICs
   batches write: D882 and HT75xx/78L05 bind their centre pin as `2` only,
   AMS1117 binds `VOUT: 2, 4`. The
   alternative for SOT-223 (tab as a second pad 2, KiCad's `TabPin2` style)
   would make the land `SOT3` and silently tie the tab to pin 2 for every
   vendor, including any whose tab is not pin 2; numbering it 4 keeps that
   decision in each device's pin map, where E807 enforces it.
5. **SOT-223 name digits.** *Settled: the published name stays.* The
   wave-1/2 rule (primary midpoint, rounded half up) gives
   `SOT4P230X651X351N` from AMS's millimetre column 6.30-6.71 x
   3.30-3.71 mm, the rounded conversions of an inch drawing (0.248-0.264 x
   0.130-0.146 in, whose own midpoints give 6.50 x 3.51 mm). The 651 is an
   artefact of that rounded millimetre column, not a different body; the
   JEDEC / typical nominal (Diodes' D 6.50 x E 3.50 mm, the midpoints of
   6.45-6.55 and 3.45-3.55 mm, and the 3c plan's placeholder) would give
   `SOT4P230X650X350N`. The #101 review recommended that name, but sot
   0.1.0 was published to registry.cohdl.org as `SOT4P230X651X351N`, and a
   published footprint path is permanent, so the name is fixed. Read the
   `651X351` digits as the AMS-derived name of a standard SOT-223 land.
6. **SOT-89 primary land.** The Hongjiacheng D882 land cannot be placed on
   its outline, so the Diodes land is used. *Recommendation:* accept the
   Diodes land; when D882 and the other SOT-89 series are bound, check the
   JSCJ D882 sheet (C9634, the Basic part on the frame-board list) and ask
   Hongjiacheng whether the drawing is an error.
7. **SOT-323 land size.** The Hongjiacheng land (0.70 x 0.90 mm) is much
   larger than Diodes' (0.47 x 0.60 mm) on the same centres; the gap between
   pads 1 and 2 is 0.60 mm and 0.83 mm respectively. *Recommendation:* keep
   the primary's land (the rule), as was done for SOD-323.
