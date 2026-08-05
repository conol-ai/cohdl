# Oscillator source manifest

Retrieved and audited on 2026-07-30 and 2026-08-05. The PDFs are stored
byte-for-byte as downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `abracon-abm8.pdf` | `ABM8-8.000MHZ-10-1-U-T`; ABM8 family package/land data also used by `ABM8-272-T3` | Abracon ABM8 family datasheet, revised 2020-07-29 | <https://abracon.com/Resonators/abm8.pdf> | `564ac331d8bf38b4ae1222444b0d9cfea3a6ba29148a8be906b2e0ffe40471d8` |
| `abracon-abm8-272-t3.pdf` | `ABM8-272-T3` | Abracon Source Control Drawing 456603 Rev. B, issued 2024-09-16 | <https://abracon.com/datasheets/ABM8-272-T3.pdf> | `aead22b6bd9d6f8ad4472352f70fce3ade633e90b9772ba80fabe8fd0856ae91` |
| `hosonic-e1sb.pdf` | `E1SB48E001G00E` package, pin, and land data | Hosonic E1SB series specification, modified 2026-07-20 | <https://www.hosonic.com/Upload/FTP/P-C-S-E1SB.pdf> | `af74ec52f0d175cd03ff64cd71a926d839fe79b023a21a3e2c2b6bceb7b7e62b` |
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
- Abracon's current catalog identifies `ABS07-32.768KHZ-9-T` as active with
  9 pF load, ±20 ppm tolerance, 70 kohm ESR, and -40 to +85 degC operation.
  ABS07 page 4 gives the 3.2 x 1.5 mm package and the recommended two-land
  pattern: 1.10 x 1.90 mm lands with a 1.40 mm inner gap.

All PDFs passed `pdfinfo`, Poppler text extraction, and full-page rendering.
The ABM8 package/land page, ABM8-272-T3 identity/electrical/mechanical pages,
Hosonic E1SB sheet, and ABS07 package/land page were visually inspected.
