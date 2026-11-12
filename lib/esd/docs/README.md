# ESD source manifest

Retrieved and revalidated on 2026-07-30; `h5vl10b.evidence.md` added 2026-09-04.

Entries ending in `.evidence.md` are repository-authored evidence records.
Their SHA-256 values identify upstream manufacturer PDFs that are not
redistributed here.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `st-usblc6-2-datasheet.pdf` | STMicroelectronics `USBLC6-2SC6` | `DS4260` Rev 7, 2021-12-24 | [STMicroelectronics](https://www.st.com/resource/en/datasheet/usblc6-2.pdf) | `bc30154f310cd631043214ed52571daefe04270587ec55072d49a23bd18b9068` |
| `tpd1e05u06.evidence.md` | Texas Instruments `TPD1E05U06DPYR` | `SLVSBO7O`, revised 2024-08 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/tpd1e05u06.pdf) | `c167cf1e72a5473a4d2c59b6a3c0251498701da05b7785919b9ceaae3b3e02c6` |
| `tpd1e1b04.evidence.md` | Texas Instruments `TPD1E1B04DPYR` | `SLVSDL0A`, revised 2016-07 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/tpd1e1b04.pdf) | `f32ea1cd8859cb177c75b6eed470012971a1abd6fbc93907fd12820012ec5d29` |
| `esd122.evidence.md` | Texas Instruments `ESD122DMYR` | `SLVSDP5A`, revised 2018-08 | [Texas Instruments](https://www.ti.com/lit/ds/symlink/esd122.pdf) | `16ba54c64476aa5abbadc84ce72e60cee6baf710c958be0aad90cf363393f3ab` |
| `h5vl10b.evidence.md` | Zhuhai Hongjiacheng `H5VL10B` | `Rev 2.0`, 3 pages | Zhuhai Hongjiacheng Technology Co., Ltd (retrieved via LCSC C7420372) | `0d0aaef1b2af0c641cfc5b2135b7242b346aa0a792611d7dcd0fbefd8bf58048` |

## Footprint and pin audit

- The top-view pin map is 1/6 = I/O1, 3/4 = I/O2, 2 = GND, and 5 = VBUS;
  the device model matches it in both directions.
- `FP_SOT_23_6` is rotated 90 degrees from Figure 19 without reflection. It
  uses ST's 0.60 x 1.20 mm lands, 0.95 mm lead pitch, and 2.30 mm row-center
  spacing. Pin 1 remains the upper-left lead in top view.
- ST also publishes CAD models from the
  [USBLC6-2 product page](https://www.st.com/en/protections-and-emi-filters/usblc6-2.html);
  the datasheet's dimensioned Figure 19 is the geometry authority used here.
- `FP_X1SON_DPY_2` follows TI drawing DPY0002A: 0.30 x 0.50 mm lands on
  0.70 mm pitch. It is shared by the two exact DPYR parts above.
- `FP_X2SON_DMY_3` follows TI drawing DMY0003A: three 0.20 mm circular lands,
  with 0.25 mm horizontal and 1.00 mm vertical center spacing.
- The pin models preserve TI's top-view numbering: TPD1E05U06 pin 2 is GND;
  TPD1E1B04 has two equivalent I/O terminals; ESD122 pin 1 is GND and pins
  2/3 are IO1/IO2.
- `FP_Hongjiacheng_DFN1006_2L` follows the H5VL10B Rev 2.0 suggested pad
  layout (p3): lands J 0.50-0.60mm long x M 0.55-0.65mm wide with a K
  0.25-0.35mm gap, nominal 0.55 x 0.60mm lands centred 0.85mm apart, inside
  the DFN1006-2L body outline (A 0.95-1.05 x B 0.55-0.65mm). H5VL10B is a
  symmetric bidirectional protector, so pins 1/2 are interchangeable
  electrically; the datasheet function diagram (1 PB 2) fixes the numbering.

## H5VL10B retrieval notes

- The manufacturer document (footer: Zhuhai Hongjiacheng Technology co., Ltd)
  was retrieved from the LCSC listing C7420372, whose `productModel` is the
  exact string `H5VL10B` with package DFN1006-2L. LCSC shows the distributor
  brand string `R+O` on that listing; the datasheet footer is the
  manufacturer identity used for the part.
- TDSEMIC's `H5VL10B-TD` (LCSC C51901858) is a different order code from a
  different vendor and is NOT this part; the accepted exact identity is the
  exact string `H5VL10B`.

## LCSC binding (esd 0.1.2)

`ESD_H5VL10B` binds its primary to LCSC C7420372 with a zero map: `supplier lcsc "C7420372"`. The part number is in `tools/sourcing_data/lcsc_map.json` since the B2 full run, and its frame row `c2b1-dfn1006-c7420372` cites C2 frame board 1 (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`), where all four copies sat centred on their pads. That preview shows no polarity mark for this part. The H5VL10B is symmetric and bidirectional (`h5vl10b.evidence.md`), so the two pads are interchangeable and a half turn would not matter. No land changed from 0.1.1. Both `esd` frame rows now name 0.1.2. C85364's row keeps its per-part `rotate 180`, but no `esd` part lists that part number, so it binds nothing.

## LCSC binding for USBLC6-2SC6 (esd 0.1.3)

`ESD_USBLC6` binds its primary to LCSC C7519 with a quarter-turn map: `supplier lcsc "C7519" rotate 270`. The part number is in `tools/sourcing_data/lcsc_map.json` (exact rule, 2026-11-07).

- **Why a map.** On preview board 4 (`tools/sourcing_data/jlc_previews/2026-11-11-board-4/`, section A), placed with no mapping, the assembler's pin 1 for C7519 lay a quarter turn from our pad 1 on all four copies, top and bottom, at 0° and 37°: 100.1°, 100.8°, 100.3° and 100.1°. The model's leads ran above and below where our pads are left and right. `FP_SOT_23_6` follows the datasheet and stays where it is; the binding carries the turn.
- **Confirmed with the map applied.** The frame row `owtc-fpsot236-c7519` cites a real board, OpenWalkieTalkie Rev C (`tools/sourcing_data/jlc_previews/2026-11-12-owt-rev-c/`). With `rotate 270` applied, U20's pin 1 mark read 11.7° from pad 1: the model's six leads sit on our six pads and its dimple is at pad 1's corner. The mark is drawn beside the lead, outside the body, so it reads about 10° past the pad on both boards. That board has one copy of this part; board 4 has four, without the map. Both boards pinned the same `esd` 0.1.2 content.
- **The map is this part number's, not the land's.** C2 frame board 1 read `rotate 180` for C85364 on this same `FP_SOT_23_6`. Do not copy `rotate 270` to another part number on this land: each needs its own preview reading.
- **What changes in output.** No land changed from 0.1.2. A plain `build` writes the same BOM and CPL; a design that places `ESD_USBLC6` gains the part's `LCSC` field in its netlist. `--supplier lcsc` writes the turn into the LCSC CPL. All three `esd` frame rows now name 0.1.3; C85364's row still binds nothing.
