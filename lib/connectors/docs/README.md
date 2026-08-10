# Connector source manifest

Retrieved and revalidated on 2026-08-05.

| Local file | Covered library item | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `raspberry-pi-pico-datasheet.pdf` | `HEADER_SWD_3W` / Raspberry Pi Pico Rev3 three-pad SWD board feature | Release 21, 2026-07-03 | [Raspberry Pi](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) | `757ff485227493b9fcc0c2c96c4dea9de020e1d8b2b11e2aa4f9ee8b25aa89eb` |
| `samtec-ssw-1xx-22-surface-mount-drawing.pdf` | Samtec `SSW-103-22-SM-D-VS` assembly, numbering, and package envelope | Rev BC, 2020-07-24 | [Samtec](https://suddendocs.samtec.com/prints/ssw-1xx-22-xxx-x-vs-xx-x-x-xx-mkt.pdf) | `b058e14102bd07ea190bdc2904126003354b68f092e492c4fe77b4545dc83061` |
| `samtec-ssw-double-row-smt-footprint.pdf` | Samtec SSW-DVS double-row SMT land pattern | Rev A, 2000-06-21 | [Samtec](https://suddendocs.samtec.com/prints/ssw-dvs.pdf) | `a8e837ed7fac38b784c95c2bcada75ed5d751a339400c7c85f204d258769d58c` |
| `molex-43045-0612-sales-drawing.pdf` | Molex Micro-Fit 3.0 vertical THT headers `43045-0212`, `43045-0412`, and `43045-0612`; the drawing's 2–24-circuit table covers all three exact MPNs | SD-43045-005, Rev G1, 2021-01-11 | [Molex](https://www.molex.com/content/dam/molex/molex-dot-com/products/automated/en-us/salesdrawingpdf/430/43045/430450612_sd.pdf) | `549ffa2364c390fcb90f9df06ff1a8a4aac621818f54e6d2f86de5a91d6febba` |
| External specification | Micro-Fit 3.0 dual-row electrical, environmental, and current-derating requirements | PS-43045 Rev R, 2025-11-14 | [Molex PDF](https://www.molex.com/content/dam/molex/molex-dot-com/products/automated/en-us/productspecificationpdf/430/43045/PS-43045-001.pdf) | Not vendored; audited facts are summarized below |

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
- `MOLEX_43045_0212`, `MOLEX_43045_0412`, and `MOLEX_43045_0612` use the
  SD-43045-005 component-side layout rotated 180 degrees into footprint
  coordinates, never reflected. Thus the official circuit-1 position, the two
  PCB-polarization pegs, and the pin-1 marker retain the same chirality. All
  electrical PTHs and both NPTH locator holes are 1.02 mm; electrical contacts
  are on a 3.00 mm row/column pitch. The rectangular courtyards enclose the
  maximum drawing envelope plus 0.50 mm nominal placement clearance.

## Molex Micro-Fit 3.0 mating BOM and ratings

| Board header | Circuits | Mating receptacle housing | Female crimp contacts |
|---|---:|---|---|
| `43045-0212` | 2 | `43025-0200` | Molex 43030 series: `43030-0038` (18 AWG, reel), `43030-0001` (20–24 AWG, reel), or `43030-0007` (20–24 AWG, loose) |
| `43045-0412` | 4 | `43025-0400` | Same 43030-series selection rules |
| `43045-0612` | 6 | `43025-0600` | Same 43030-series selection rules |

The current Molex catalog gives the 43045 header family a maximum headline
rating of 8.5 A per contact and 600 V, but PS-43045 requires application
derating. With every circuit powered in wire-to-board service, the Rev R table
lists 8.5/7/6/5.5 A per contact for the 2-circuit connector at 18/20/22/24 AWG,
and 6.5/5.5/4.5/4.5 A for the 6-circuit connector. PCB copper, ambient
temperature, wire, crimp quality, and adjacent heating still require end-use
validation. Tin-plated headers are specified from -40 to +105 °C and for 30
mating cycles.

For an 8 A board power input, the 4-circuit pair can allocate two contacts to
the positive rail and two to return (4 A/contact). PS-43045 does not tabulate a
separate 4-circuit derating point, so that allocation is a conservative design
target to validate thermally, not a new manufacturer rating. Using 4 circuits
for power and 6 circuits for actuator outputs also prevents the two harnesses
from being physically interchanged.
