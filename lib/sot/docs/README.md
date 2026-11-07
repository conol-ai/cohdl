# `sot`: SOT land patterns

Reusable SOT land patterns for new parts (library plan W4, item 3c, wave 1).
This package ships lands only: no device, no part and no supplier binding.
Bindings come after the wave-1 JLCPCB frame preview board. One land per
standard outline is taken from one primary document and cross-checked against
one tier-1 manufacturer document; each part series' own datasheet (and its
pin map: BEC, GSD, diode pairs ...) is checked when that series is bound.

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

## Lands

| Footprint | Outline | Lands | Pad centres | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|
| `sot::SOT3P190X290X130N` | SOT-23-3 (TO-236AB) | 0.80 x 0.90 mm declared, placed `rotate 270` (0.90 in x, 0.80 in y) | 1 (1.00, 0.95), 2 (1.00, -0.95), 3 (-1.00, 0) mm | 3.4 x 3.6 mm | 2.55 x 3.00 mm (E1 x D max, in this frame) |

The name follows RFC-021's checked template `SOT{pins}P{pitch}X{bodyX}X{bodyY}N`.
E809 reads the pitch as the closest spacing between distinct pad numbers,
pins 1-2 at 1.90 mm, so the name reads `P190`. Body digits are the primary
outline's nominal D 2.90 and E 1.30 mm.

## Sources

The PDFs are not redistributed; each `.evidence.md` file records the document
identity, both URLs, the 2026-11-07 retrieval, the SHA-256 and the exact
page and dimensions used. The Hongjiacheng document is a manufacturer PDF
served by LCSC; the maintainer approved that fallback for these lands.

| Record | Role | Document | SHA-256 |
|---|---|---|---|
| `hongjiacheng-mmbt5551.evidence.md` | SOT-23-3 primary | Hongjiacheng MMBT5551 Rev 2.0, p3 (LCSC C7420357) | `7723e0fc4bf81c91d1d965a3fa04f287d74654013bfdc0efa3cd5c35a8c317f9` |
| `diodes-sot23.evidence.md` | SOT-23-3 cross-check | Diodes SOT23 Package Information Rev. 2017-01-30, p1 | `d4843d22b6c5502465b699ec9a5e42a83dbf54dd286b0eaa9e630956841b7608` |

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

## Existing lands for the same outline

These lands are not moved or changed:

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
   measured frame, not measured on this footprint. The wave-1 frame board
   must read it before the first publish (Tony's sign-off).
