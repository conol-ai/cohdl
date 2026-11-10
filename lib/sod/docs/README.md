# `sod`: SOD two-terminal diode land patterns

Reusable SOD land patterns for new parts (library plan W4, item 3c, waves 1
and 2). This package ships lands only: no device, no part and no supplier
binding. Bindings come after the JLCPCB frame preview boards. One land per
standard outline is taken from one primary document and cross-checked against
one tier-1 manufacturer document; when the primary gives no land, the land is
derived from the cross-check document and says so. Each part series' own
datasheet is checked against the land when that series is bound.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. **Pad 1 is the cathode (band end) at -x**; pad 2 is the anode at +x.
The silkscreen carries `polarity_marker cathode_pin 1 shape band` (RFC-031)
outboard of pad 1 and the two long body edges. The assembler has read this
orientation as 0 degrees for SOD-123 (C81598) and SOD-323 (C2128, C191023)
parts on earlier preview boards, so the lands are authored on it; the frame
boards still have to confirm it for these footprints before publishing. No
SOD-123FL or SOD-523 part has been read yet. Bidirectional TVS parts
(`...CA`, for example the SMF series) carry no band but reuse the land.

## Lands

| Footprint | Outline | Lands (x by y) | Pad centres | Outer copper | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|---|
| `sod::SOD3716X135N` | SOD-123 | 0.91 x 1.22 mm | +/-1.635 mm (3.27 mm) | 4.18 mm | 4.7 x 2.4 mm | 3.85 x 1.80 mm (A x C max) |
| `sod::SOD2613X110N` | SOD-323 | 0.70 x 0.70 mm | +/-1.075 mm (2.15 mm) | 2.85 mm | 3.4 x 2.0 mm | 2.80 x 1.40 mm (C x E max) |
| `sod::SODFL3818X120N` | SOD-123FL / SOD-123F | 1.34 x 1.80 mm | +/-1.43 mm (2.86 mm) | 4.20 mm | 4.7 x 2.5 mm | 3.90 x 1.90 mm (D x C max) |
| `sod::SOD1608X70N` | SOD-523 | 0.60 x 0.70 mm | +/-0.71 mm (1.42 mm) | 2.02 mm | 2.6 x 1.5 mm | 1.70 x 0.90 mm (C x E max) |

Wave 1 added SOD-123 and SOD-323; wave 2 added SOD-123FL and SOD-523. The
SOD-123FL land comes from the Diodes cross-check because the Hongjiacheng
primary has no land (see below).

Each courtyard contains the copper and the maximum component envelope with at
least 0.25 mm on every side (checked by `tests/library/geometry.rs`).

## Sources

The PDFs are not redistributed; each `.evidence.md` file records the document
identity, both URLs, the 2026-11-07 retrieval, the SHA-256 and the exact
page and dimensions used. The Hongjiacheng documents are manufacturer PDFs
served by LCSC; the maintainer approved that fallback for these lands.

| Record | Role | Document | SHA-256 |
|---|---|---|---|
| `hongjiacheng-bzt52c5v6.evidence.md` | SOD-123 primary | Hongjiacheng BZT52C2V0-BZT52C75 Rev 2.1, p4 (LCSC C19077402) | `6c66d1a4354d67a86200f0764e0b71641731b67985e0618cf7cd05147392799a` |
| `diodes-sod123.evidence.md` | SOD-123 cross-check | Diodes SOD123 Package Information Rev. 2017-03-16, p1 | `ceda14ae0447b97457c9df48192839f955832f1db6c1bb48d132da39cef5d714` |
| `hongjiacheng-b5819ws.evidence.md` | SOD-323 primary | Hongjiacheng B5817WS-B5819WS Rev 2.0, p3 (LCSC C7420331) | `d0140bf852f26e98f404651f9055c2b99fef6bea17326ee6be37854c714ede94` |
| `diodes-sod323.evidence.md` | SOD-323 cross-check | Diodes SOD323 Package Information Rev. 2017-03-16, p1 | `40af15e4e467bfc8022cc30ba8988a8211e22e468c7b37a683f259474e36639a` |
| `hongjiacheng-smf15ca.evidence.md` | SOD-123FL primary (outline only) | Hongjiacheng SMF series Rev 2.2, p2 (LCSC C19077510) | `c8f137df222c1c36b0356096e10d601434b420c2f65407c222229dbd725df74a` |
| `diodes-sod123f.evidence.md` | SOD-123FL land source and cross-check | Diodes SOD123F Package Information Rev. 2026-04-08, p1 | `d7104f55a097b2b40c3a1353911176d871040959c9ecfc8a999835cf087aca0d` |
| `hongjiacheng-1n4148wt.evidence.md` | SOD-523 primary | Hongjiacheng 1N4148WT Rev 2.0, p3 (LCSC C7420320) | `71312a1ff0a0911bd02cb41941bbc00a9abdea0146b9843db0cd9ad03fe8365a` |
| `diodes-sod523.evidence.md` | SOD-523 cross-check | Diodes SOD523 outline and suggested pad layout, 2016-04-11, p1 | `34b24ac0134963812b5c1412cd20c26c78eb24143d91ac4930c4112ec9558e47` |

## Primary versus cross-check

**SOD-123.** Hongjiacheng gives one-sided limits (J >= 0.91, K <= 2.36,
M >= 1.22 mm), used as the values. Diodes gives X 0.90, Y 0.95, X1 4.05 mm.

| | Hongjiacheng (this land) | Diodes package file | `diode::FP_D_SOD_123` |
|---|---|---|---|
| Land length (x) | 0.91 | 0.90 | 0.90 |
| Land width (y) | 1.22 | 0.95 | 0.95 |
| Centre spacing | 3.27 | 3.15 | 3.15 |
| Gap | 2.36 | 2.25 | 2.25 |
| Outer span | 4.18 | 4.05 | 4.05 |

Hongjiacheng's land is 0.27 mm wider and 0.13 mm longer overall. The two
outlines agree (lead span 3.55-3.85 mm in both; body width 1.40-1.80 versus
Diodes 1.40-1.70 mm). The Diodes package file's layout is identical to the
DS30086 Rev 31-2 layout already in `lib/diode/docs`.

**SOD-323.** Hongjiacheng gives ranges (X 0.65-0.75, Y 0.65-0.75, Z 2.10-2.20
mm), and the land uses the midpoints. Diodes gives X 0.59, Y 0.45, X1 2.70 mm.

| | Hongjiacheng (this land) | Diodes package file | `contrib_esd::FP_SOD323_2P5X1P25mm` |
|---|---|---|---|
| Land length (x) | 0.70 | 0.59 | 0.80 |
| Land width (y) | 0.70 | 0.45 | 0.80 |
| Centre spacing | 2.15 | 2.11 | 2.20 |
| Gap | 1.45 | 1.52 | 1.40 |
| Outer span | 2.85 | 2.70 | 3.00 |

The Diodes land is markedly narrower (0.45 versus 0.70 mm). Hongjiacheng's
outline is also looser (lead span up to 2.80 mm versus Diodes 2.70 mm). The
@contrib GBLC05C land follows ProTek's ranges (lands 0.66-0.91 mm, overall
span 2.87-3.12 mm): Hongjiacheng's 0.70 mm lands fall inside ProTek's land
range, but its 2.85 mm span is 0.02 mm below ProTek's minimum.

**SOD-123FL / SOD-123F.** The Hongjiacheng SMF sheet gives the outline only,
with no land on any page, so the land is **derived from the Diodes SOD123F
package file** (X 1.34, Y 1.80, C 2.86, G 1.52, X1 4.20 mm). The two
outlines differ mainly in lead span:

| Outline (mm) | Hongjiacheng SMF (primary) | Diodes SOD123F | Nexperia CFP3 (SOD123W), `diode::FP_D_SOD_123W` |
|---|---|---|---|
| Lead tip to lead tip | 3.60-3.90 (D) | 3.30-3.70 (He) | 3.3-3.7 |
| Body length | 2.55-2.85 (B) | 2.60-2.80 (E) | 2.4-2.8 |
| Body width | 1.60-1.90 (C) | 1.70-1.90 (D) | 1.5-1.9 |
| Lead width | 0.90-1.10 (A) | 0.80-1.05 (b) | 0.75-1.05 |
| Lead foot | 0.40-0.90 (F) | 0.35-0.85 (L2) | 0.3-0.6 |
| Height | 1.00-1.20 (E) | 0.81-1.15 (A) | 0.9-1.1 |

| Land (mm) | This land (Diodes values) | `diode::FP_D_SOD_123W` |
|---|---|---|
| Land length (x) | 1.34 | 1.20 |
| Land width (y) | 1.80 | 1.20 |
| Centre spacing | 2.86 | 2.80 |
| Gap | 1.52 | 1.60 |
| Outer span | 4.20 | 4.00 |

On the Hongjiacheng outline the lead tips sit 0.15-0.30 mm inside the land's
outer edge (0.25-0.45 mm on Diodes' own outline), and every lead foot's inner
end stays outboard of the 0.76 mm inner land edge. `diode::FP_D_SOD_123W` is
not this land: it is Nexperia's own CFP3 (SOD123W) outline and its own reflow
footprint (Figure 17 of the PMEG6010ELR datasheet), a different body with a
different land; on its 4.00 mm copper a 3.90 mm Hongjiacheng part would keep
only 0.05 mm of toe.

**SOD-523.** Hongjiacheng gives ranges (X 0.55-0.65, Y 0.65-0.75, Z 1.37-1.47
mm; Z is drawn between the two land centres), and the land uses the
midpoints. Diodes gives X 0.60, Y 0.70, G 0.80, X1 2.00 mm.

| | Hongjiacheng (this land) | Diodes package file |
|---|---|---|
| Land length (x) | 0.60 | 0.60 |
| Land width (y) | 0.70 | 0.70 |
| Centre spacing | 1.42 | 1.40 |
| Gap | 0.82 | 0.80 |
| Outer span | 2.02 | 2.00 |

The lands are the same size; Hongjiacheng's centres are 0.02 mm further
apart. The outlines agree within 0.05 mm: Hongjiacheng C 1.50-1.70 / E
0.70-0.90 / D 0.50-0.70 mm against Diodes He 1.55-1.65 / D 0.75-0.85 / A
0.55-0.65 mm. `mosfet::SOT3P100X160X80N` is the three-lead SOT-523, not this
two-lead outline.

## Existing lands for the same outlines

These lands are not moved or changed; both owners now exist side by side:

- `diode::FP_D_SOD_123` (Diodes DS30086 land, bound by `diode::D_1N4148W`)
  remains for existing users. `sod::SOD3716X135N` is the land for new parts.
  Both are kept (settled in the #101 review), and each source file names the
  other with the copper difference: 0.90 x 0.95 mm at +/-1.575 mm against
  0.91 x 1.22 mm at +/-1.635 mm (diode 0.1.1 adds the note on its side).
- `contrib_esd::FP_SOD323_2P5X1P25mm` stays with its @contrib package.
  Official packages do not depend on @contrib, so `sod::SOD2613X110N` is the
  official owner of SOD-323 for new parts.
- No existing land covers SOD-123FL or SOD-523. `diode::FP_D_SOD_123W`
  (Nexperia CFP3 / SOD123W) is a different body and land, compared above.

## Open questions (for the maintainers)

1. **Final names.** SOD is outside RFC-021's closed set, so no check covers
   these names. They are IPC-7351B-style: SOD + lead span + body width (both
   in tenths of a millimetre, nominal = midpoint of the primary tolerance,
   rounded half up) + X + maximum height (hundredths). SOD-323 therefore reads
   `SOD2613X110N` from Hongjiacheng's 2.30-2.80 mm span and 1.10 mm height,
   where an IPC calculator fed JEDEC values gives `SOD2513X100N`.
   *Recommendation:* keep IPC-7351B-style names, and fix them before the
   first publish (a rename after it is a breaking path change). Either accept
   the primary-source digits, or decide that SOD names use JEDEC nominal
   values so they do not depend on one vendor's tolerances. A follow-up RFC
   could add SOD/DIOM to the checked set. *Settled by publication:* sod 0.1.0
   was published with the primary-source digits, so these names are
   permanent.
2. **Duplicates.** SOD-123 now has two official owners whose copper differs
   (see above). *Recommendation:* keep both (option (a) in the 3c plan) until
   CoHDL has re-exports; then point `diode::FP_D_SOD_123` users at the `sod`
   land or keep it as the Diodes-specific variant. When `@contrib/esd` is
   promoted, bind GBLC05C to `sod::SOD2613X110N` only if ProTek's span range
   is met (it is not today by 0.02 mm), otherwise keep its own land.
3. **Frame confirmation.** The -x cathode frame is inferred from other SOD
   footprints' preview readings, not measured on these. The wave-1 and wave-2
   frame boards must read them before the first publish (Tony's sign-off).
4. **SOD-123FL land source and lead span.** The Hongjiacheng primary has no
   land, so `SODFL3818X120N` uses the Diodes SOD123F land, which Diodes drew
   for a 3.30-3.70 mm lead span; Hongjiacheng's SOD-123FL reaches 3.90 mm,
   leaving 0.15 mm of toe at the maximum. *Recommendation:* accept it (the
   toe is positive and the heel is covered), and check a second SOD-123FL
   land (onsemi case 498 or the MDD SM4007PL datasheet, the Basic part on
   the frame-board list) before the first publish; widen the outer edge only
   if that document asks for more toe.
5. **SOD-123FL name.** IPC-7351B uses the `SODFL` prefix for flat-lead SOD
   outlines; the digits follow question 1's rule (3.75 and 1.75 mm nominal
   rounded half up), giving `SODFL3818X120N` where the 3c plan's JEDEC-based
   placeholder was `SODFL3717X110N`. SOD-523's `SOD1608X70N` matches the
   plan's placeholder. *Settled by publication* with question 1: both
   names are in sod 0.1.0 and are permanent.
