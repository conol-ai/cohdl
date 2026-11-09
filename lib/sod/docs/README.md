# `sod`: SOD two-terminal diode land patterns

Reusable SOD land patterns for new parts (library plan W4, item 3c, wave 1).
This package ships lands only: no device, no part and no supplier binding.
Bindings come after the wave-1 JLCPCB frame preview board. One land per
standard outline is taken from one primary document and cross-checked against
one tier-1 manufacturer document; each part series' own datasheet is checked
against the land when that series is bound.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. **Pad 1 is the cathode (band end) at -x**; pad 2 is the anode at +x.
The silkscreen carries `polarity_marker cathode_pin 1 shape band` (RFC-031)
outboard of pad 1 and the two long body edges. The assembler has read this
orientation as 0 degrees for SOD-123 (C81598) and SOD-323 (C2128, C191023)
parts on earlier preview boards, so the lands are authored on it; the wave-1
frame board still has to confirm it for these footprints before publishing.

## Lands

| Footprint | Outline | Lands (x by y) | Pad centres | Outer copper | Courtyard | Body used for the courtyard |
|---|---|---|---|---|---|---|
| `sod::SOD3716X135N` | SOD-123 | 0.91 x 1.22 mm | +/-1.635 mm (3.27 mm) | 4.18 mm | 4.7 x 2.4 mm | 3.85 x 1.80 mm (A x C max) |
| `sod::SOD2613X110N` | SOD-323 | 0.70 x 0.70 mm | +/-1.075 mm (2.15 mm) | 2.85 mm | 3.4 x 2.0 mm | 2.80 x 1.40 mm (C x E max) |

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

## Existing lands for the same outlines

These lands are not moved or changed; both owners now exist side by side:

- `diode::FP_D_SOD_123` (Diodes DS30086 land, bound by `diode::D_1N4148W`)
  remains for existing users. `sod::SOD3716X135N` is the land for new parts.
- `contrib_esd::FP_SOD323_2P5X1P25mm` stays with its @contrib package.
  Official packages do not depend on @contrib, so `sod::SOD2613X110N` is the
  official owner of SOD-323 for new parts.

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
   could add SOD/DIOM to the checked set.
2. **Duplicates.** SOD-123 now has two official owners whose copper differs
   (see above). *Recommendation:* keep both (option (a) in the 3c plan) until
   CoHDL has re-exports; then point `diode::FP_D_SOD_123` users at the `sod`
   land or keep it as the Diodes-specific variant. When `@contrib/esd` is
   promoted, bind GBLC05C to `sod::SOD2613X110N` only if ProTek's span range
   is met (it is not today by 0.02 mm), otherwise keep its own land.
3. **Frame confirmation.** The -x cathode frame is inferred from other SOD
   footprints' preview readings, not measured on these. The wave-1 frame
   board must read them before the first publish (Tony's sign-off).
