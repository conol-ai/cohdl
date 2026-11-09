# `do214`: DO-214 molded-body diode land patterns

Reusable DO-214 land patterns for new parts (library plan W4, item 3c,
wave 1). This package ships lands only: no device, no part and no supplier
binding. Bindings come after the wave-1 JLCPCB frame preview board. One land
per standard outline is taken from one primary document and cross-checked
against one tier-1 manufacturer document; each part series' own datasheet is
checked against the land when that series is bound.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. **Pad 1 is the cathode (band end) at -x**; pad 2 is the anode at +x,
the same frame as `sod`. The silkscreen carries
`polarity_marker cathode_pin 1 shape band` (RFC-031) outboard of pad 1 and
the two long body edges. No DO-214 part has been read on a preview board yet:
this orientation is the one the assembler reads as 0 degrees for SOD parts,
and the wave-1 frame board must confirm it before the first publish.
Bidirectional TVS parts (`...CA`) carry no band but reuse this land.

## Lands

| Footprint | Outline | Lands (x by y) | Pad centres | Outer copper | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|---|
| `do214::DIOM5026X241N` | SMA / DO-214AC | 2.10 x 1.70 mm | +/-2.20 mm (4.40 mm) | 6.50 mm | 7.0 x 3.4 mm | 5.30 x 2.85 mm (G x C max) |

The courtyard contains the copper and the maximum component envelope with at
least 0.25 mm on every side (checked by `tests/library/geometry.rs`).

## Sources

The PDFs are not redistributed; each `.evidence.md` file records the document
identity, both URLs, the 2026-11-07 retrieval, the SHA-256 and the exact
page and dimensions used. The Hongjiacheng documents are manufacturer PDFs
served by LCSC; the maintainer approved that fallback for these lands.

| Record | Role | Document | SHA-256 |
|---|---|---|---|
| `hongjiacheng-ss34.evidence.md` | SMA primary | Hongjiacheng SS32-SS320 Rev 2.0, p3 (LCSC C7420365) | `f83a234fc3a645c2fe6d495d363a8b1bef79562769c813a72ca58e70f6b98927` |
| `hongjiacheng-smaj5v0a.evidence.md` | SMA second Hongjiacheng view | Hongjiacheng SMAJ series Rev 2.2, p2 (LCSC C19077523) | `3ff5a7b8618e9ea109acec3fdfefba71442e321ac3cd4461702f08964d205947` |
| `diodes-sma.evidence.md` | SMA cross-check | Diodes SMA Package Information Rev. 2017-03-14, p1 | `97001a21cf9bd5cf2deaccbd92c3ae16fe56a678be4229fc240657f927f4abfc` |

## Primary versus cross-check

SS34 gives one-sided limits (M >= 1.70, J >= 2.10, K <= 2.30 mm), used as the
values.

| | SS34 (this land) | SMAJ series (limits) | Diodes package file |
|---|---|---|---|
| Land length (x) | 2.10 | >= 2.10 | 2.50 |
| Land width (y) | 1.70 | >= 2.26 | 1.70 |
| Centre spacing | 4.40 | - | 4.00 |
| Gap | 2.30 | <= 2.74 | 1.50 |
| Outer span | 6.50 | - | 6.50 |

Diodes has the same width and outer span; its lands reach 0.40 mm further
inward. The outlines agree: Hongjiacheng G 4.70-5.30 / C 2.35-2.85 / D
1.98-2.41 mm against Diodes E 4.80-5.59 / A 2.29-2.92 / J 1.96-2.40 mm. The
two Hongjiacheng sheets have identical outline dimensions but different land
limits.

## Existing lands for the same outline

None in the library.

## Open questions (for the maintainers)

1. **SMAJ land width.** The SMAJ sheet asks for lands at least 2.26 mm wide;
   this land (from the designated primary, SS34) is 1.70 mm. SMAJ is 31 of
   the 109 SMA snapshot parts, so their binding-time check will fail on width.
   *Recommendation:* before the first publish, widen the land to 2.10 x
   2.26 mm with the 2.30 mm gap. That satisfies both Hongjiacheng sheets
   (SS34 M >= 1.70, K <= 2.30; SMAJ M >= 2.26, K <= 2.74) and still fits the
   7.0 x 3.4 mm courtyard. This change keeps the primary's land, as the
   maintainer's rule requires, and leaves the decision open.
2. **Final name.** DIOM is outside RFC-021's closed set, so no check covers
   the name. It is IPC-7351B-style: DIOM + overall length + body width (tenths
   of a millimetre, midpoint of the primary tolerance) + X + maximum height
   (hundredths), giving `DIOM5026X241N` where JEDEC-fed calculators give
   about `DIOM5226X244N`. *Recommendation:* keep IPC-7351B-style names and fix
   them before the first publish (as for `sod`).
3. **Frame confirmation.** The -x cathode frame is not yet measured for any
   DO-214 part; the wave-1 frame board must read it before the first publish
   (Tony's sign-off).
