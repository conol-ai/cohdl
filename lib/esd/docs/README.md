# ESD source manifest

Retrieved and revalidated on 2026-07-30; `h5vl10b.evidence.md` added 2026-09-04; the three Hongjiacheng TVS records added 2026-11-11 (esd 0.1.3).

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
| `hongjiacheng-smbj6v0ca.evidence.md` | Zhuhai Hongjiacheng `SMBJ6.0CA` | SMBJ series, `Rev 1.1`, 5 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C19077561](https://www.lcsc.com/product-detail/C19077561.html)) | `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39` |
| `hongjiacheng-smcj36ca.evidence.md` | Zhuhai Hongjiacheng `SMCJ36CA` | SMCJ series, `Rev 2.2`, 5 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C19077609](https://www.lcsc.com/product-detail/C19077609.html)) | `5cbe7c34174b49058f9e13aa94ae9d17b521feea2d4537cc46876c764fc3cefe` |
| `hongjiacheng-smf15ca.evidence.md` | Zhuhai Hongjiacheng `SMF15CA` | SMF series, `Rev 2.2`, 5 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C19077510](https://www.lcsc.com/product-detail/C19077510.html)) | `c8f137df222c1c36b0356096e10d601434b420c2f65407c222229dbd725df74a` |

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

## Hongjiacheng TVS parts (esd 0.1.3)

Three Zhuhai Hongjiacheng bidirectional TVS diodes, each on a land of the
`do214` or `sod` package (`do214` 0.1.2, `sod` 0.1.3). This package gains those two
dependencies; no land of its own changes, and `ESD_H5VL10B` keeps its binding.
Both `esd` frame rows now name 0.1.3.

| Part | MPN | Land | Ratings used (datasheet page), 10/1000 us pulse |
|---|---|---|---|
| `TVS_SMBJ6V0CA` | `SMBJ6.0CA` | `do214::DIOM5336X265N` (SMB) | PPPM 600 W (p1); VRWM 6.0 V, VBR 6.67-7.37 V at 10 mA, VC 10.3 V at 58.25 A, IR 800 uA, doubled to 1600 uA for a bidirectional part of 10 V and under (p2, p3 note 3) |
| `TVS_SMCJ36CA` | `SMCJ36CA` | `do214::DIOM7959X261N` (SMC) | PPPM 1500 W (p1); VRWM 36 V, VBR 40-44.2 V at 1 mA, VC 58.1 V at 25.9 A, IR 1 uA (p3) |
| `TVS_SMF15CA` | `SMF15CA` | `sod::SODFL3818X120N` (SOD-123FL) | PPPM 200 W (p1); VRWM 15 V, VBR 16.7-18.5 V at 1 mA, VC 24.4 V at 8.2 A, IR 1 uA (p2) |

- **Why here.** These are board-level transient suppressors, which is what
  `ESD_Protector` covers; they are surge-rated (10/1000 us) parts, larger
  than the signal-line protectors above. They are not in `diode` because a
  bidirectional TVS is not `Polarized`.
- **Pins.** A bidirectional ("CA") device has no polarity and no cathode
  band (SMCJ and SMF outline notes; the SMBJ sheet labels the band "for
  uni-directional products only"). The two terminals are interchangeable, so
  each device follows `H5VL10B` and `TPD1E1B04`: `IO1` on pad 1 and `IO2` on
  pad 2. The lands' cathode silk bar at pad 1 has no meaning for these parts.
- **Sources.** The same three manufacturer PDFs that the lands' outlines
  were taken from (`lib/do214/docs/` and `lib/sod/docs/` hold the land
  records), manufacturer documents served by LCSC. The PDFs are not
  redistributed; each `.evidence.md` here records the SHA-256 and the page
  of every value. The values were read on 2026-11-11 from the copies
  retrieved on 2026-11-07, after checking each SHA-256; nothing was fetched.
- **Lands.** The SMCJ and SMF sheets give an outline and no land, so
  `do214::DIOM7959X261N` and `sod::SODFL3818X120N` take their land from the
  Diodes package files; the fit on the Hongjiacheng outlines is in the
  lands' records.

### LCSC bindings

Each part binds its primary with no rotation and no offset: `supplier lcsc
"C19077561"` (SMBJ6.0CA), `"C19077609"` (SMCJ36CA) and `"C19077510"`
(SMF15CA).

- **Map rows.** The three part numbers are in
  `tools/sourcing_data/lcsc_map.json` from the backfill run of 2026-11-11
  (`tools/sourcing_data/lcsc_backfill/2026-11-11-hongjiacheng/`),
  which read every candidate page for each MPN (112, 52 and 46 pages). Each
  page names the manufacturer "hongjiacheng", matched through the reviewed
  alias; the catalog class is Preferred.
- **Placement evidence, and its limits**, under the bidirectional-TVS rule
  (`docs/library-supplier-plan.md`, W3): on the 2026-11-08 probe preview,
  built with zero correction, the drawn body of each part number lay along
  the pad axis within 1.0 degree and centred within 0.045 mm on all four
  copies (top and bottom, 0 and 37 degrees). That is a reading of body
  alignment and centring only. The preview draws a plain rectangle with no
  band, mark or text, so it shows neither identity nor direction; the
  orientation is known mod 180, which is enough only because the part is
  symmetric. It is not a polarity or pin 1 reading. The SMCJ36CA reading
  carries a caveat: it was made with a window 0.8 mm wider than the stock
  body script, which clips that body's corners and gives no long axis for a
  body only 1.18 times as long as wide.
- **Frame rows.** `w4d-diom5336x265n-c19077561`,
  `w4d-diom7959x261n-c19077609` and `w4d-sodfl3818x120n-c19077510` in
  `tools/sourcing_data/jlc_frames.json` are copies of the rows archived in
  `tools/sourcing_data/jlc_previews/2026-11-08-probe/frames.json`, keyed to
  `do214` 0.1.2 and `sod` 0.1.3. The readings were made on the 0.1.0 lands;
  no pad has changed since.

## LCSC binding for USBLC6 (esd 0.1.4)

`ESD_USBLC6` binds its primary to LCSC C7519 with a quarter-turn map: `supplier lcsc "C7519" rotate 270`. The part number is in `tools/sourcing_data/lcsc_map.json` (exact rule, 2026-11-07). This version is `esd` 0.1.3 plus this binding and this section; no land changes.

- **Why a map.** Preview board 4 placed C7519 with no mapping (`tools/sourcing_data/jlc_previews/2026-11-11-board-4/`): on all four copies the assembler's model lay a quarter turn from `FP_SOT_23_6`, pin 1 at 100.1° and 100.8° (top, 0° and 37°) and 259.7° and 259.9° (bottom) counter-clockwise from our pad 1. The land follows ST's Figure 19 and stays where it is; the binding carries the turn.
- **Evidence.** The frame row `pb5-fpsot236-c7519` cites preview board 5 (`tools/sourcing_data/jlc_previews/2026-11-14-board-5/`), which placed the part with `rotate 270` applied. Its pin 1 mark read 10.6° and 10.0° (top) and 349.4° and 349.2° (bottom), 0.53–0.56 mm from pad 1. The mark sits outside the body beside the pad 1 lead, which is where the 10° comes from (board 4 read the same 10° past the quarter turn). By eye, on all four copies the model's six leads lie on our pads, three a side, with its pin 1 dimple at the pad 1 corner.
- **Per part number, not per land.** Other part numbers on SOT-23-6 lands read differently on C2 frame board 1: C85364 on this land needs 180 (its row `c2b1-sot236_esd-c85364` still binds nothing here), and C12044 on `load-switch::FP_TI_DBV0006A` needs 270. A rotation proved for C7519 says nothing about another part number on `FP_SOT_23_6`.
- **Frame rows.** All three `esd` rows now name 0.1.4.
- **What changes in output.** A plain `build` writes the same BOM and CPL; the netlist gains the part's `LCSC` field. `--supplier lcsc` writes the turn into the LCSC CPL.
