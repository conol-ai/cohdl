# `do214`: DO-214 molded-body diode land patterns

Reusable DO-214 land patterns for new parts (library plan W4, item 3c,
waves 1 and 2). This package ships lands only: no device, no part and no
supplier binding. Bindings come after the JLCPCB frame preview boards. One
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
the two long body edges. No DO-214 part has been read on a preview board yet:
this orientation is the one the assembler reads as 0 degrees for SOD parts,
and the frame boards must confirm it for SMA, SMB and SMC before the first
publish.
Bidirectional TVS parts (`...CA`) carry no band but reuse this land.

## Lands

| Footprint | Outline | Lands (x by y) | Pad centres | Outer copper | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|---|
| `do214::DIOM5026X241N` | SMA / DO-214AC | 2.10 x 1.70 mm | +/-2.20 mm (4.40 mm) | 6.50 mm | 7.0 x 3.4 mm | 5.30 x 2.85 mm (G x C max) |
| `do214::DIOM5336X265N` | SMB / DO-214AA | 2.10 x 2.26 mm | +/-2.42 mm (4.84 mm) | 6.94 mm | 7.5 x 4.5 mm | 5.59 x 3.94 mm (G x C max) |
| `do214::DIOM7959X261N` | SMC / DO-214AB | 2.50 x 3.30 mm | +/-3.45 mm (6.90 mm) | 9.40 mm | 9.9 x 6.8 mm | 8.13 x 6.22 mm (G x C max) |

Wave 1 added SMA; wave 2 added SMB and SMC. The SMC land comes from the
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

## Existing lands for the same outlines

None in the library for SMA, SMB or SMC.

## Open questions (for the maintainers)

1. **SMAJ land width.** The SMAJ sheet asks for lands at least 2.26 mm wide;
   this land (from the designated primary, SS34) is 1.70 mm. SMAJ is 31 of
   the 109 SMA snapshot parts, so their binding-time check will fail on width.
   *Recommendation:* before the first publish, widen the land to 2.10 x
   2.26 mm with the 2.30 mm gap. That satisfies both Hongjiacheng sheets
   (SS34 M >= 1.70, K <= 2.30; SMAJ M >= 2.26, K <= 2.74) and still fits the
   7.0 x 3.4 mm courtyard. This change keeps the primary's land, as the
   maintainer's rule requires, and leaves the decision open.
   *Wave-2 evidence (land unchanged):* the SMBJ sheet's land rows are
   numerically identical to the SMAJ sheet's (M >= 2.26, J >= 2.10,
   K <= 2.74 mm). On SMB, M = 2.26 mm just covers the 2.21 mm maximum lead
   width; on SMA the leads are at most 1.65 mm wide, and SS34 and Diodes both
   use 1.70 mm. The SMAJ rows therefore look carried over from the SMB sheet
   rather than drawn for SMA. The SMCJ sheet has no land rows at all, so it
   gives no family convention. Reviewers may prefer to keep 1.70 mm and treat
   the SMAJ M row as a probable copy (asking Hongjiacheng to confirm), rather
   than widen the SMA land.
2. **Final name.** DIOM is outside RFC-021's closed set, so no check covers
   the name. It is IPC-7351B-style: DIOM + overall length + body width (tenths
   of a millimetre, midpoint of the primary tolerance) + X + maximum height
   (hundredths), giving `DIOM5026X241N` where JEDEC-fed calculators give
   about `DIOM5226X244N`. *Recommendation:* keep IPC-7351B-style names and fix
   them before the first publish (as for `sod`).
3. **Frame confirmation.** The -x cathode frame is not yet measured for any
   DO-214 part; the wave-1 and wave-2 frame boards must read SMA, SMB and SMC
   before the first publish (Tony's sign-off).
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
   `DIOM5436X240N` and `DIOM7959X250N`. Settle them with question 2.
