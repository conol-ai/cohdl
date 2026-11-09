# Oscillator source manifest

Retrieved and audited on 2026-07-30 and 2026-08-05. The PDFs are stored
byte-for-byte as downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `abracon-abm8.pdf` | `ABM8-8.000MHZ-10-1-U-T`; ABM8 family package/land data also used by `ABM8-272-T3` | Abracon ABM8 family datasheet, revised 2020-07-29 | <https://abracon.com/Resonators/abm8.pdf> | `564ac331d8bf38b4ae1222444b0d9cfea3a6ba29148a8be906b2e0ffe40471d8` |
| `abracon-abm8-272-t3.pdf` | `ABM8-272-T3` | Abracon Source Control Drawing 456603 Rev. B, issued 2024-09-16 | <https://abracon.com/datasheets/ABM8-272-T3.pdf> | `aead22b6bd9d6f8ad4472352f70fce3ade633e90b9772ba80fabe8fd0856ae91` |
| `hosonic-e1sb.pdf` | `E1SB48E001G00E` and `E1SB40E001G00E` (MPN unverified) package, pin, and land data | Hosonic E1SB series specification, modified 2026-07-20 | <https://www.hosonic.com/Upload/FTP/P-C-S-E1SB.pdf> | `af74ec52f0d175cd03ff64cd71a926d839fe79b023a21a3e2c2b6bceb7b7e62b` |
| `abracon-abs07.pdf` | `ABS07-32.768KHZ-9-T` | Abracon ABS07 family datasheet, revised 2022-08-10 | <https://abracon.com/Resonators/ABS07.pdf> | `5b21d42d9ca704d86cfb8f1b93488f3f3671c55ca90f334aa66f931866277dd1` |

## Coverage and geometry

- The generic ABM8 ordering code table covers the exact 8.000 MHz, 10 pF,
  standard 0.80 mm maximum height, tolerance option 1 (plus/minus 10 ppm),
  stability option U (plus/minus 10 ppm), and `-T` tape-and-reel
  configuration.
- The exact ABM8-272-T3 source-control drawing supplies its 12 MHz electrical
  configuration, 3.2 x 2.5 mm package, terminal map, revision, and packaging.
- ABM8 family page 2 supplies the recommended land pattern used by both
  footprints: 1.30 x 1.05 mm lands, a 1.00 mm horizontal inner gap, and a
  0.70 mm vertical inner gap.
- Both manufacturer drawings show terminals 1 and 3 as the crystal electrodes
  and terminals 2 and 4 as ground. The manufacturer warns that the package
  chamfer can appear at pin 1, 3, or 4 and has no electrical effect; the
  footprint marker therefore records nominal terminal orientation, not a
  guaranteed physical chamfer location.
- No separate manufacturer CAD files were used; the footprints are derived
  from the dimensioned manufacturer package and land-pattern drawings.
- SiFli's current
  [SF32LB52X hardware guide](https://wiki.sifli.com/en/hardware/SF32LB52B-E-G-J-HW-Application.html)
  identifies Hosonic `E1SB48E001G00E` as a certified 48 MHz selection with
  -6/+8 ppm tolerance, 8.8 pF load, and 22 ohm maximum ESR. Hosonic's E1SB
  series sheet supplies its 2.0 x 1.6 mm package, terminal map, and exact
  0.85 x 0.75 mm recommended lands with 0.50/0.30 mm inner gaps.
- The E1SB sheet defines no index mark. Its top and bottom views draw the
  pin-2 terminal (a lid ground) with a cut inner corner; the
  `FP_Crystal_SMD_2016_4Pin_E1SB` pin-1 dot sits at pad 1, not at that
  corner. Turning the part 90 degrees to match the cut corner to the dot
  would put the crystal electrodes (pins 1, 3) on the ground pads (2, 4); a
  180 degree turn is electrically harmless.
- Abracon's current catalog identifies `ABS07-32.768KHZ-9-T` as active with
  9 pF load, ±20 ppm tolerance, 70 kohm ESR, and -40 to +85 degC operation.
  ABS07 page 4 gives the 3.2 x 1.5 mm package and the recommended two-land
  pattern: 1.10 x 1.90 mm lands with a 1.40 mm inner gap.

All PDFs passed `pdfinfo`, Poppler text extraction, and full-page rendering.
The ABM8 package/land page, ABM8-272-T3 identity/electrical/mechanical pages,
Hosonic E1SB sheet, and ABS07 package/land page were visually inspected.

## 40MHz (ESP32-S3 main crystal, Hosonic E1SB 2.0 x 1.6 mm 4-pad)

| Local file | Covered item | MPN | Note |
|---|---|---|---|
| `hosonic-e1sb.pdf` | `XTAL_40MHZ` | `E1SB40E001G00E` (unverified) | Package, terminal map and land (`FP_Crystal_SMD_2016_4Pin_E1SB`) from the E1SB series sheet p. 1. The MPN is patterned on `E1SB48E001G00E`; the sheet has no ordering-code key, so the part's existence, CL, tolerance and stability are not established. Confirm with Hosonic or a distributor before fitting. |

- Series limits at 40 MHz (p. 1): CL 6-16 pF "or specify", plus/minus
  10 ppm at 25 degC, plus/minus 15 ppm over temperature, ESR at most
  60 ohm, C0 at most 2 pF, drive level 1-100 uW.
- The "10 pF load class, C1 = C4 10 pF + 24 nH series inductor on XTAL_N"
  guidance comes from Espressif's ESP32-S3 hardware design guidelines. That
  document is not pinned in this repo, so the guidance is not source-checked;
  the E1SB sheet gives no external-component values.

## E1SB land for XTAL_40MHZ in 0.2.0 (2026-11-07)

`XTAL_40MHZ` (`E1SB40E001G00E`) is a Hosonic E1SB part, a 2.00 x 1.60 mm
package (`hosonic-e1sb.pdf` p. 1). Up to 0.1.1 it used `FP_Crystal_3225`,
the Abracon ABM8 3.2 x 2.5 mm land: 1.30 x 1.05 mm pads at x = +/-1.15 mm,
y = +/-0.875 mm. It now uses `FP_Crystal_SMD_2016_4Pin_E1SB`, the E1SB
recommended land already used by `XTAL_48MHZ`: 0.85 x 0.75 mm pads at
x = +/-0.675 mm, y = +/-0.525 mm (0.50 mm horizontal and 0.30 mm vertical
inner gaps), pin 1 bottom-left in the top view, and a 2.6 x 2.2 mm
courtyard around the 2.10 x 1.70 mm maximum body. This is a breaking change
for any board using the part, so the package takes a minor bump.
`FP_Crystal_3225` is unchanged and still used by `XTAL_8M`. The part's
comments no longer call the package "3225", mark the MPN as unverified, and
mark the load-capacitor guidance as coming from an unpinned source. Device,
pins and names are unchanged.
