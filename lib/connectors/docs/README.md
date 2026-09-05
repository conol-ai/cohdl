# Connector source manifest

Retrieved and revalidated on 2026-08-05.

Entries ending in `.evidence.md` are repository-authored evidence records.
Their SHA-256 values identify upstream manufacturer drawings that are not
redistributed here.

| Local file | Covered library item | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `raspberry-pi-pico-datasheet.pdf` | `HEADER_SWD_3W` / Raspberry Pi Pico Rev3 three-pad SWD board feature | Release 21, 2026-07-03 | [Raspberry Pi](https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf) | `757ff485227493b9fcc0c2c96c4dea9de020e1d8b2b11e2aa4f9ee8b25aa89eb` |
| `samtec-ssw-1xx-22-surface-mount-drawing.pdf` | Samtec `SSW-103-22-SM-D-VS` assembly, numbering, and package envelope | Rev BC, 2020-07-24 | [Samtec](https://suddendocs.samtec.com/prints/ssw-1xx-22-xxx-x-vs-xx-x-x-xx-mkt.pdf) | `b058e14102bd07ea190bdc2904126003354b68f092e492c4fe77b4545dc83061` |
| `samtec-ssw-double-row-smt-footprint.pdf` | Samtec SSW-DVS double-row SMT land pattern | Rev A, 2000-06-21 | [Samtec](https://suddendocs.samtec.com/prints/ssw-dvs.pdf) | `a8e837ed7fac38b784c95c2bcada75ed5d751a339400c7c85f204d258769d58c` |
| `molex-43045-0612-sales-drawing.pdf` | Molex Micro-Fit 3.0 vertical THT headers `43045-0212`, `43045-0412`, and `43045-0612`; the drawing's 2–24-circuit table covers all three exact MPNs | SD-43045-005, Rev G1, 2021-01-11 | [Molex](https://www.molex.com/content/dam/molex/molex-dot-com/products/automated/en-us/salesdrawingpdf/430/43045/430450612_sd.pdf) | `549ffa2364c390fcb90f9df06ff1a8a4aac621818f54e6d2f86de5a91d6febba` |
| `lailan-phc125-2p.evidence.md` | LAILAN `LAIL-PHC1.25-2P-01-PB-WT` right-angle 2-contact header and recommended PCB layout | Rev A, retrieved 2026-09-04 | [LCSC C54905971 manufacturer drawing](https://datasheet.lcsc.com/datasheet/pdf/ed062a34b8a33b6bfd770152d865d754.pdf?productCode=C54905971) | `db858cd19d16297a52f0ecd62abc0df81e90cfc4e206161172b3881a3a9aac7a` |
| `xfcn-pz127v-11-04-0720.evidence.md` | XFCN `PZ127V-11-04-0720` four-pin 1.27 mm vertical header | Rev A1, 2024-06-17 | [LCSC C541879 manufacturer drawing](https://datasheet.lcsc.com/datasheet/pdf/d235e62233ad54b54d9e8641381a1659.pdf) | `febe17bae503c6abf777a69386350fd4b5bb964b093a446397b8574a03e4e647` |
| `lailan-fpc-cx01-31p0.3-gw.evidence.md` | LAILAN `LAIL-FPC-CX01-31P0.3-GW` 31-contact staggered FPC connector | Rev A, retrieved 2026-09-04 | [LCSC C55172970 manufacturer drawing](https://datasheet.lcsc.com/datasheet/pdf/8d1d72e9ef181a8e41c7604fe1e40f44.pdf) | `a1d77fa3ac22bbbc6e2c833b37e51edd05d5c436ee207b1e2411d449427cfb5b` |
| `lailan-pz127-4p.evidence.md` | LAILAN `LAIL-PZ1.27-4P-L` four-pin 1.27 mm vertical header | Rev A, 2026-05 | [LCSC C54950573 manufacturer drawing](https://datasheet.lcsc.com/datasheet/pdf/837a22e906052b604df03c5cafb07fe6.pdf) | `156253cf1a3b99a7b16bebd6bb2c276905f54cf0a8e6e02e819f5f9a4890b908` |
| `hirose-ufl-r-smt-1.evidence.md` | Hirose `U.FL-R-SMT-1(10)` 50-ohm receptacle | EDC3-302540-10 Rev 2, one page | [manufacturer drawing mirror](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/8942/HIROS08829-1.pdf) | `27580d7e4b3c323ea916fa0421ac156e0be62e364cfd70498cf9dfb6ddb6a6c6` |
| `hirose-fh12-26s-0.5sh.evidence.md` | Hirose `FH12-26S-0.5SH(55)` 26-contact FPC | FH12 series, 2016-11-01 | [Hirose](https://www.hirose.com/product/p/CL0586-0576-2-55) | Existing `fh12-series.pdf` hash plus evidence record |
| `jae-sf72s006.evidence.md` | JAE `SF72S006VBA(R2500)` nano-SIM socket | MB-0282-1, August 2014, 5 pages | [manufacturer document mirror](https://datasheet.lcsc.com/datasheet/pdf/b13cbe906d2905e1a5778f79cd6e142e.pdf?productCode=C2977289) | `e55440fbf6ea70bbe5cf8821b1838ab011151fbc53f16dba835e3dd6224c6707` |
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
- `CON_LAILAN_PHC125_2P_01_PB_WT` uses the manufacturer PCB layout directly:
  0.70 x 1.70 mm signal lands on 1.25 mm pitch and 1.80 x 2.50 mm shell lands
  on 6.06 mm centers with a 0.40 mm vertical gap. The exact C54905971 EDA
  model establishes pad chirality omitted from the dimension drawing: signal
  pads 1/2 are left/right and shell pads 4/3 are left/right in that view.
- `CON_XFCN_PZ127V_11_04_0720` uses the manufacturer's 0.65 mm PCB holes on
  1.27 mm pitch. The exact C541879 EDA model confirms left-to-right pin
  numbering and 1.00 mm copper pads.
- `CON_LAILAN_FPC_CX01_31P0_3_GW` transcribes the manufacturer's staggered
  layout: 0.30 mm pitch; 0.30 x 0.80 mm odd lands; 0.30 x 0.65 mm even lands;
  2.875 mm row-centre separation; and 0.30 x 1.15 mm shell lands on 10.30 mm
  centres. The C55172970 EDA model confirms odd/even numbering and shell pads
  32/33.
- `CON_LAILAN_PZ1_27_4P_L` uses the manufacturer PCB layout directly: four
  0.70 mm plated holes at 1.27 mm pitch. The exact C54950573 EDA model confirms
  1.00 mm copper pads, left-to-right numbering 1..4 and the rectangular pin-1
  pad; it is distinct from the source's XFCN PZ127V connector.

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
