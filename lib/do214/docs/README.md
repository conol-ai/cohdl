# `do214`: DO-214 molded-body diode land patterns

Reusable DO-214 land patterns, and the flat-lead SMAF outline, for new parts
(library plan W4, item 3c, waves 1 to 3). This package ships lands only: no device, no part and no
supplier binding. Parts live in the category packages; the
first three on these lands are listed under "Preview readings" below. One
land per standard outline is taken from one primary document and
cross-checked against one tier-1 manufacturer document; when the primary
gives no land, the land is derived from the cross-check document and says
so. Each part series' own datasheet is checked against the land when that
series is bound.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. **Pad 1 is the cathode (band end) at -x**; pad 2 is the anode at +x,
the same frame as `sod`. The silkscreen carries
`polarity_marker cathode_pin 1 shape band` (RFC-031) outboard of pad 1 and
the two long body edges. This orientation is the one the assembler reads as
0 degrees for some SOD parts; do214 0.1.0 was published (2026-11-09) with it
before any DO-214 part had been read. A change to pad numbering or rotation
would be a breaking change needing a new minor version. Bidirectional TVS
parts (`...CA`) carry no band but reuse this land.

### Preview readings on these lands (2026-11-08)

Three part numbers have been read on assembler previews of these lands. The
readings are per part number; they do not confirm the frame for the lands as
a whole (open question 3).

| Land | Part number | Reading | Library part and mapping |
|---|---|---|---|
| `DIOM5026X241N` (SMA) | C7420365 SS34 | Polarized. The assembler's model has its cathode at +x, a half turn from pad 1. With `rotate 180` applied, the model's minus sign and cathode band sat at the cathode silk bar on all four copies (read by eye; the measuring script gives no angle for a model with two marks). The preview labels the model SMAJ12A, so this shows the polarity of that part number in that preview, not the part's identity. | `diode::SCHOTTKY_SS34`; bound with `rotate 180` |
| `DIOM5336X265N` (SMB) | C19077561 SMBJ6.0CA | Bidirectional TVS, read by the body only on a board with no correction: long axis within 0.2 degrees of the pad axis, centre within 0.033 mm, on all four copies. | `esd::TVS_SMBJ6V0CA`; bound with a zero mapping |
| `DIOM7959X261N` (SMC) | C19077609 SMCJ36CA | Bidirectional TVS, read by the body only: long axis within 0.2 degrees, centre within 0.027 mm. Caveat: read with a window 0.8 mm wider than the stock body script, which clips this body's corners and gives no long axis for a body only 1.18 times as long as wide. | `esd::TVS_SMCJ36CA`; bound with a zero mapping |

A body reading shows alignment and centring; the direction is known mod 180
only, which is enough only because a bidirectional TVS is symmetric. So the
one polarized part read on these lands is a half turn from the land, and the
two TVS readings cannot show direction: the frame is not confirmed as the
assembler's zero for SMA, SMB or SMC. One part number is too few to move a
land, so the half turn is a per-part mapping. No SMAF part has been read. The
records are `tools/sourcing_data/jlc_previews/2026-11-08-reverify/` and
`tools/sourcing_data/jlc_previews/2026-11-08-probe/` in the repository.

All three parts are bound since 2026-11-11 (SS34, SMBJ6.0CA and SMCJ36CA; `diode` 0.1.2
and `esd` 0.1.3), with the mappings above: their part numbers reached
`tools/sourcing_data/lcsc_map.json` through the backfill run of that date, and
the archived rows were copied into `jlc_frames.json` keyed to this package's
version. The bindings are per part number and still do not confirm the frame.

## Lands

| Footprint | Outline | Lands (x by y) | Pad centres | Outer copper | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|---|
| `do214::DIOM5026X241N` | SMA / DO-214AC | 2.10 x 1.70 mm | +/-2.20 mm (4.40 mm) | 6.50 mm | 7.0 x 3.4 mm | 5.30 x 2.85 mm (G x C max) |
| `do214::DIOM5336X265N` | SMB / DO-214AA | 2.10 x 2.26 mm | +/-2.42 mm (4.84 mm) | 6.94 mm | 7.5 x 4.5 mm | 5.59 x 3.94 mm (G x C max) |
| `do214::DIOM7959X261N` | SMC / DO-214AB | 2.50 x 3.30 mm | +/-3.45 mm (6.90 mm) | 9.40 mm | 9.9 x 6.8 mm | 8.13 x 6.22 mm (G x C max) |
| `do214::FP_DO221AC_SMAF` | SMAF (flat-lead SMA) | 1.60 x 1.80 mm | +/-1.90 mm (3.80 mm) | 5.40 mm | 5.9 x 3.3 mm | 4.90 x 2.70 mm (D x C max) |

Wave 1 added SMA; wave 2 added SMB and SMC; wave 3 added SMAF. The SMC land comes from the
Diodes cross-check because the Hongjiacheng primary has no land (see below).
Each courtyard contains the copper and the maximum component envelope with at
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
| `hongjiacheng-smbj6v0ca.evidence.md` | SMB primary | Hongjiacheng SMBJ series Rev 1.1, p2 (LCSC C19077561) | `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39` |
| `diodes-smb.evidence.md` | SMB cross-check | Diodes SMB Package Information Rev. 2019-08-30, p1 | `10a17932644b9b3c6034ca13b0962ab239c1391460795e5c52b6e5541eec711d` |
| `hongjiacheng-smcj36ca.evidence.md` | SMC primary (outline only) | Hongjiacheng SMCJ series Rev 2.2, p2 (LCSC C19077609) | `5cbe7c34174b49058f9e13aa94ae9d17b521feea2d4537cc46876c764fc3cefe` |
| `diodes-smc.evidence.md` | SMC land source and cross-check | Diodes SMC Package Information Rev. 2017-03-14, p1 | `b4bd36debe2405dbc75c3d20799afd771c7008cac2ae48900fb1e2970abf4287` |
| `hongjiacheng-ss24f.evidence.md` | SMAF primary | Hongjiacheng SS22F-SS220F Rev 2.0, p3 (LCSC C18199202) | `21831b8b07a50f1201f8812f858b89348285f1d62444fcb5d7de4f2b331acf5b` |
| `diodes-smaf.evidence.md` | SMAF cross-check | Diodes SMAF Package Information Rev. 2019-06-27, p1 | `60be78ccf2d43e2704629ba08fd5d49d89b6d483a0f318139de445232367a73c` |

## Primary versus cross-check

**SMA.** SS34 gives one-sided limits (M >= 1.70, J >= 2.10, K <= 2.30 mm), used as the
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

**SMB.** The SMBJ sheet's land rows give one-sided limits (M >= 2.26,
J >= 2.10, K <= 2.74 mm), used as the values. Diodes gives X 2.50, Y 2.30,
C 4.30, G 1.80, X1 6.80 mm.

| | SMBJ (this land) | Diodes package file |
|---|---|---|
| Land length (x) | 2.10 | 2.50 |
| Land width (y) | 2.26 | 2.30 |
| Centre spacing | 4.84 | 4.30 |
| Gap | 2.74 | 1.80 |
| Outer span | 6.94 | 6.80 |

The Diodes lands are 0.04 mm wider and 0.40 mm longer; they reach 0.47 mm
further inward and stop 0.07 mm short of this land's outer edge. The outlines
agree on body width (3.30-3.94 mm in both) and overall length
(Hongjiacheng G 5.08-5.59 against Diodes E 5.00-5.59 mm). Hongjiacheng's body
is longer (B 4.25-4.85 against E1 4.06-4.57 mm) and taller (D 2.15-2.65
against A 2.00-2.50 mm); its leads are slightly narrower at the low end
(A 1.85-2.21 against b 1.96-2.21 mm). On the Hongjiacheng outline the lead
tips sit 0.68-0.93 mm inside this land's outer edge, but a lead foot's inner
end (G/2 - E) can be as close as 1.02 mm to the centre, so up to 0.35 mm of
a long foot's heel sits over the 2.74 mm gap; the Diodes land (inner edge
0.90 mm) covers it.

**SMC.** The Hongjiacheng SMCJ sheet gives the outline only, with no land on
any page, so the land is **derived from the Diodes SMC package file**
(X 2.50, Y 3.30, C 6.90, G 4.40, X1 9.40 mm) and the comparison is between
outlines:

| Outline (mm) | Hongjiacheng SMCJ (primary) | Diodes SMC |
|---|---|---|
| Overall length | 7.75-8.13 (G) | 7.75-8.13 (E) |
| Body length | 6.60-7.11 (B) | 6.60-7.11 (B) |
| Body width | 5.59-6.22 (C) | 5.59-6.22 (A) |
| Lead width | 2.85-3.27 (A) | 2.75-3.18 (C) |
| Lead foot | 0.76-1.52 (E) | 0.76-1.52 (H) |
| Height | 1.99-2.61 (D) | 2.00-2.50 (J) |

The outlines agree within 0.11 mm. On the Hongjiacheng outline the lead tips
sit 0.64-0.83 mm inside the land's outer edge, every lead foot's inner end
stays outboard of the 2.20 mm inner land edge, and the 3.27 mm maximum lead
width leaves only 0.015 mm per side on the 3.30 mm land (Diodes' own 3.18 mm
maximum leaves 0.06 mm).

**SMAF.** The SS22F-SS220F sheet gives one-sided limits (M >= 1.80,
J >= 1.60, K <= 2.2 mm), used as the values, as for SMA and SMB. Diodes
gives X 2.50, Y 1.70, C 4.00, G 1.50, X1 6.50 mm, identical to its SMA land.

| | SS24F (this land) | Diodes package file |
|---|---|---|
| Land length (x) | 1.60 | 2.50 |
| Land width (y) | 1.80 | 1.70 |
| Centre spacing | 3.80 | 4.00 |
| Gap | 2.20 | 1.50 |
| Outer span | 5.40 | 6.50 |

The two documents describe different-sized flat-lead bodies under the same
name:

| Outline (mm) | Hongjiacheng SS22F-SS220F (primary) | Diodes SMAF |
|---|---|---|
| Overall length | 4.40-4.90 (D) | 4.80-5.60 (HE) |
| Body length | 3.30-3.70 (B) | 3.95-4.60 (E) |
| Body width | 2.40-2.70 (C) | 2.25-2.95 (D) |
| Lead width | 1.30-1.60 (A) | 1.25-1.65 (b) |
| Lead foot | 0.80-1.20 (F) | 0.50-1.50 (L) |
| Height | 0.90-1.20 (E) | 0.90-1.10 (A) |

The Diodes SMAF is 0.4-0.7 mm longer, which is why its land is 1.10 mm wider
overall. On the Hongjiacheng outline this land puts the lead tips 0.25-0.50
mm inside its 2.70 mm outer edge; with the longest foot and shortest overall
length up to 0.10 mm of each heel sits over the 2.20 mm gap (as on SMB,
open question 4). A Diodes-length part (HE up to 5.60 mm, tips at 2.80 mm)
would overhang this land's outer edge by up to 0.10 mm, so a non-Hongjiacheng
SMAF series must be checked against this land when it is bound.

**Fit: Hongjiacheng-length bodies only.** `do214::FP_DO221AC_SMAF` fits
SMAF bodies of the Hongjiacheng length (overall length D 4.40-4.90 mm). A
Diodes "SMAF" (HE 4.80-5.60 mm) can overhang the 2.70 mm outer land edge by
up to 0.10 mm, so bind another vendor's SMAF series only after checking its
overall length against that edge.

**Name.** Neither document names a JEDEC outline: neither the Hongjiacheng
nor the Diodes source prints "DO-221AC". The 3c plan's `DO-221AC` label is
kept in the free name `FP_DO221AC_SMAF`, which was published in do214 0.1.0
and is now permanent; read it as a label for this SMAF land, not as a
JEDEC registration. See open question 7.

## Existing lands for the same outlines

None in the library for SMA, SMB, SMC or SMAF.

## Open questions (for the maintainers)

1. **SMAJ land width.** *Settled in the #101 review: keep 1.70 mm, a
   deliberate waiver of the SMAJ sheet's M >= 2.26 mm.* The SMAJ sheet asks
   for lands at least 2.26 mm wide; this land (from the designated primary,
   SS34) is 1.70 mm. The SMAJ land rows are table-only (no land drawing on
   any of its five pages) and numerically identical to the SMBJ sheet's SMB
   rows (M >= 2.26, J >= 2.10, K <= 2.74 mm), where M = 2.26 mm just covers
   SMB's 2.21 mm maximum lead width. SMA leads are at most 1.65 mm wide, and
   SS34 and the Diodes SMA file both use 1.70 mm, so the SMAJ rows are
   treated as carried over from SMB. The SMCJ sheet has no land rows at
   all, so it gives no family convention. SMAJ is 31 of the 109 SMA
   snapshot parts; their binding-time check records this waiver instead of
   failing on width. The alternative considered was 2.10 x 2.26 mm lands
   with the 2.30 mm gap, which meets both Hongjiacheng sheets and still fits
   the 7.0 x 3.4 mm courtyard; asking Hongjiacheng to confirm the SMAJ rows
   remains worthwhile.
2. **Final name.** DIOM is outside RFC-021's closed set, so no check covers
   the name. It is IPC-7351B-style: DIOM + overall length + body width (tenths
   of a millimetre, midpoint of the primary tolerance) + X + maximum height
   (hundredths), giving `DIOM5026X241N` where JEDEC-fed calculators give
   about `DIOM5226X244N`. *Settled by publication:* do214 0.1.0 was
   published with this name, so it is permanent.
3. **Frame confirmation.** do214 0.1.0 was published (2026-11-09) before
   any DO-214 part had been read. The first readings (2026-11-08, "Preview
   readings" above) do not confirm the -x cathode frame: the one polarized
   part (SS34 on SMA) read a half turn from the land, and the SMB and SMC
   readings are of bidirectional parts, which cannot show direction. So this
   stays an open item on a published package: more polarized part numbers
   must be read on SMA, SMB and SMC, and any on SMAF, before the frame is
   called confirmed or the lands are turned. A change to pad numbering or rotation would be a breaking change
   needing a new minor version.
4. **SMB heel coverage.** `DIOM5336X265N` takes the SMBJ gap at its 2.74 mm
   maximum, as the SMA land does with its K limit. With Hongjiacheng's
   longest feet and shortest overall length, up to 0.35 mm of each heel sits
   over the gap. *Recommendation:* keep the primary's values for now (the
   rule), and decide with question 1 whether one-sided limits should be
   applied at the conservative end. For SMB that would be J = 2.57 mm with
   the same 3.47 mm outer edge: 2.57 x 2.26 mm lands at +/-2.185 mm, a
   1.80 mm gap equal to Diodes', which still meets every SMBJ limit and
   covers every heel.
5. **SMC land source.** The Hongjiacheng primary has no land, so
   `DIOM7959X261N` uses the Diodes SMC land. Hongjiacheng's leads can be
   0.09 mm wider than Diodes' (3.27 against 3.18 mm), leaving 0.015 mm per
   side on the 3.30 mm land. *Recommendation:* accept it; check the Vishay
   SS36 snapshot part's mounting-pad layout (C35722) when that series is
   bound.
6. **Final names (wave 2).** `DIOM5336X265N` and `DIOM7959X261N` follow
   question 2's rule; the 3c plan's JEDEC-based placeholders were
   `DIOM5436X240N` and `DIOM7959X250N`. *Settled by publication* with
   question 2: both names are in do214 0.1.0 and are permanent.
7. **SMAF name and outline.** *Settled: the published name stays.*
   IPC-7351B has no family for a flat-lead molded diode, so the land took
   the 3c plan's free name `FP_DO221AC_SMAF`. Neither the Hongjiacheng nor
   the Diodes document prints "DO-221AC", and the two describe bodies
   0.4-0.7 mm apart in length. The #101 review recommended renaming the land
   after its source outline (for example `DIOM4726X120N`), but do214 0.1.0
   was published to registry.cohdl.org as `FP_DO221AC_SMAF`, and a published
   footprint path is permanent, so the name is fixed. What the name cannot
   say is recorded here instead: the land fits Hongjiacheng-length bodies
   (D 4.40-4.90 mm) only; a Diodes "SMAF" (HE 4.80-5.60 mm) can overhang the
   2.70 mm outer land edge by up to 0.10 mm, so bind another vendor's SMAF
   series only after checking its overall length against that edge.
8. **SMAF one-sided limits.** As for SMA and SMB, the SMAF land uses the
   limits themselves. The 2.2 mm gap leaves up to 0.10 mm of heel
   uncovered at the extreme of the tolerances. *Recommendation:* decide with
   questions 1 and 4; a conservative alternative is J = 1.70 mm at the same
   2.70 mm outer edge (2.00 mm gap), which still meets every SS24F limit.
