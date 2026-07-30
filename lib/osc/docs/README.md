# Oscillator source manifest

Retrieved and audited on 2026-07-30. The PDFs are stored byte-for-byte as
downloaded; neither was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `abracon-abm8.pdf` | `ABM8-8.000MHZ-10-1-U-T`; ABM8 family package/land data also used by `ABM8-272-T3` | Abracon ABM8 family datasheet, revised 2020-07-29 | <https://abracon.com/Resonators/abm8.pdf> | `564ac331d8bf38b4ae1222444b0d9cfea3a6ba29148a8be906b2e0ffe40471d8` |
| `abracon-abm8-272-t3.pdf` | `ABM8-272-T3` | Abracon Source Control Drawing 456603 Rev. B, issued 2024-09-16 | <https://abracon.com/datasheets/ABM8-272-T3.pdf> | `aead22b6bd9d6f8ad4472352f70fce3ade633e90b9772ba80fabe8fd0856ae91` |

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

Both PDFs passed `pdfinfo`, Poppler text extraction, and full-page rendering.
The ABM8 package/land page and the ABM8-272-T3 identity, electrical,
mechanical, and packaging pages were visually inspected.
