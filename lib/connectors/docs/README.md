# Connector source manifest

Retrieved and revalidated on 2026-07-30.

| Local file | Covered library item | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `raspberry-pi-pico-datasheet.pdf` | `HEADER_SWD_3W` / Raspberry Pi Pico Rev3 three-pad SWD board feature | Release 21, 2026-07-03 | [Raspberry Pi](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) | `757ff485227493b9fcc0c2c96c4dea9de020e1d8b2b11e2aa4f9ee8b25aa89eb` |
| `samtec-ssw-1xx-22-surface-mount-drawing.pdf` | Samtec `SSW-103-22-SM-D-VS` assembly, numbering, and package envelope | Rev BC, 2020-07-24 | [Samtec](https://suddendocs.samtec.com/prints/ssw-1xx-22-xxx-x-vs-xx-x-x-xx-mkt.pdf) | `b058e14102bd07ea190bdc2904126003354b68f092e492c4fe77b4545dc83061` |
| `samtec-ssw-double-row-smt-footprint.pdf` | Samtec SSW-DVS double-row SMT land pattern | Rev A, 2000-06-21 | [Samtec](https://suddendocs.samtec.com/prints/ssw-dvs.pdf) | `a8e837ed7fac38b784c95c2bcada75ed5d751a339400c7c85f204d258769d58c` |

## CAD and footprint audit

- `FP_Pico_Castellated_3` was cross-checked against Figure 3, Figure 4, and
  Appendix B of the Pico datasheet and Raspberry Pi's official
  [Pico design-files ZIP](https://pip.raspberrypi.com/documents/RP-008379-DS).
  The ZIP retrieved on 2026-07-30 has SHA-256
  `90d120b7664bb1b7a458c50ab65ed087e28116e849f627793dc1ecf1585d415a`.
  Its Rev3 Allegro board identifies the three-pad stack as
  `PINHDR_3W_2P54MM`; the modeled 2.54 mm pitch, 1.0 mm drill, and contact
  chirality agree with the official sources.
- `CON_PICO_3W` is Raspberry Pi's reference-design CAD identifier for a board
  feature, not an orderable MPN. It remains in the BOM so generated output
  retains provenance. The schematic maps D1/D2/D3 to SWCLK/SWDIO/GND; the
  reusable connector device intentionally exposes passive contact names
  P1/P2/P3 in that order.
- The former generic 2x3 socket entry is now bound to the exact active Samtec
  MPN `SSW-103-22-SM-D-VS`; see its
  [official product page](https://www.samtec.com/products/ssw-103-22-sm-d-vs).
  `FP_Socket_2x3_254_SMD` now uses the official 1.27 x 2.54 mm lands, 2.54 mm
  column pitch, 6.10 mm row-center spacing, and non-mirrored top-view
  numbering.

