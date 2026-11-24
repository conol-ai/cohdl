# Diode source manifest

Retrieved and revalidated on 2026-07-30; the three Hongjiacheng records were
added on 2026-11-11 (diode 0.1.2), the Zener record on 2026-11-21 (diode 0.1.3),
the SOD-323/SOD-523 Zener record on 2026-11-24 (diode 0.1.5).

Entries ending in `.evidence.md` are repository-authored evidence records.
Their SHA-256 values identify manufacturer PDFs that are not redistributed
here.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `nexperia-pmeg6010elr-datasheet.pdf` | Nexperia `PMEG6010ELR` | Version 5, 2023-01-01 | [Nexperia](https://assets.nexperia.com/documents/data-sheet/PMEG6010ELR.pdf) | `c0eb3503d558ac894117e0dc520c33b0fe4e832f98602b6f56dfc4fc9538f112` |
| `diodes-inc-bav16w-1n4148w-datasheet.pdf` | Diodes Incorporated `1N4148W-7-F` | `DS30086` Rev 31-2, 2024-09-02 | [Diodes Incorporated](https://www.diodes.com/assets/Datasheets/BAV16W_1N4148W.pdf) | `39c16a6888bdab22418e93e17182174aad763a66957a4632e70c944194e3fc08` |
| `hongjiacheng-b5819ws.evidence.md` | Zhuhai Hongjiacheng `B5819WS` | B5817WS-B5819WS, Rev 2.0, 3 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C7420331](https://www.lcsc.com/product-detail/C7420331.html)) | `d0140bf852f26e98f404651f9055c2b99fef6bea17326ee6be37854c714ede94` |
| `hongjiacheng-ss34.evidence.md` | Zhuhai Hongjiacheng `SS34` | SS32-SS320, Rev 2.0, 3 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C7420365](https://www.lcsc.com/product-detail/C7420365.html)) | `f83a234fc3a645c2fe6d495d363a8b1bef79562769c813a72ca58e70f6b98927` |
| `hongjiacheng-1n4148wt.evidence.md` | Zhuhai Hongjiacheng `1N4148WT` | 1N4148WT, Rev 2.0, 3 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by [LCSC C7420320](https://www.lcsc.com/product-detail/C7420320.html)) | `71312a1ff0a0911bd02cb41941bbc00a9abdea0146b9843db0cd9ad03fe8365a` |
| `hongjiacheng-zener.evidence.md` | Zhuhai Hongjiacheng BZT52C, BZT52B, MMSZ52xxB, MMSZ46xx, MM1Z..B, MM1Z and MM1W SOD-123 Zeners (285 MPNs) | Seven sheets, Rev 2.0-2.1 (per-sheet revisions, dates and checksums in the record) | Zhuhai Hongjiacheng Technology Co., Ltd (PDFs served by LCSC) | seven SHA-256 values, listed in the record |
| `hongjiacheng-zener-sod323-sod523.evidence.md` | Zhuhai Hongjiacheng BZT52C..S, BZT52B..S, MM3Z, MMSZ52xxBS (SOD-323) and BZX584C (SOD-523) Zeners (173 MPNs) | Five sheets, Rev 2.0-2.2 (per-sheet revisions, dates and checksums in the record) | Zhuhai Hongjiacheng Technology Co., Ltd (PDFs served by LCSC) | five SHA-256 values, listed in the record |

## Footprint and pin audit

- The Nexperia pin table assigns pin 1 to cathode and pin 2 to anode. The
  previous model was reversed and is corrected. `FP_D_SOD_123W` uses Figure
  17's 1.20 x 1.20 mm lands on 2.80 mm center spacing and encloses the
  published 4.4 x 2.1 mm occupied area.
- Diodes Incorporated identifies the cathode by the package band; the model
  keeps cathode on pin 1. `FP_D_SOD_123` now follows the official suggested
  layout exactly: X = 0.90 mm, Y = 0.95 mm, X1 = 4.05 mm, giving 3.15 mm
  center spacing.
- Product-status references are the official
  [PMEG6010ELR page](https://www.nexperia.com/product/PMEG6010ELR) and
  [1N4148W page](https://www.diodes.com/part/view/1N4148W).

## Hongjiacheng parts (diode 0.1.2)

Three Zhuhai Hongjiacheng diodes, each on a land of the `sod` or `do214`
package (`sod` 0.1.3, `do214` 0.1.2). This package gains those two dependencies; no land of
its own changes.

| Part | MPN | Land | Ratings used (datasheet page) |
|---|---|---|---|
| `SCHOTTKY_B5819WS` | `B5819WS` | `sod::SOD2613X110N` (SOD-323) | VRRM 40 V, IF(AV) 1.0 A, IFSM 10 A (p1); VF 0.6 V max at 1.0 A, IR 1.0 mA max at 40 V (p2) |
| `SCHOTTKY_SS34` | `SS34` | `do214::DIOM5026X241N` (SMA) | VRRM 40 V, IF(AV) 3.0 A, IFSM 80 A (p1); VF 0.55 V max at 3.0 A, IR 0.5 mA max at 40 V and 25 C (p2) |
| `D_1N4148WT` | `1N4148WT` | `sod::SOD1608X70N` (SOD-523) | VRRM 100 V, IF(AV) 150 mA, Pd 150 mW (p1); VF 1.25 V max at 150 mA, trr 4.0 ns max (p2) |

- **Sources.** The same three manufacturer PDFs that the lands were drawn
  from (`lib/sod/docs/` and `lib/do214/docs/` hold the land records). They
  are manufacturer documents served by LCSC, the fallback the maintainer
  approved for those lands. The PDFs are not redistributed; each
  `.evidence.md` here records the SHA-256 and the page of every value. The
  values were read on 2026-11-11 from the copies retrieved on 2026-11-07,
  after checking each SHA-256; nothing was fetched.
- **Pins.** Each sheet marks the cathode with the band ("Polarity: Cathode
  line denotes the cathode end", p1) and numbers no terminal. The devices
  put the cathode on pad 1, the library convention for these lands
  (`lib/README.md`), so the cathode sits at the land's band mark.
- **The 1N4148WT is not an alternate of `D_1N4148W`.** It is a SOD-523 part
  with 150 mA average current and 150 mW dissipation; `D_1N4148W`
  (Diodes 1N4148W-7-F) is SOD-123. The two have different lands and ratings,
  so they are separate parts.
- **SMA land width.** `do214::DIOM5026X241N` keeps the SS-series sheet's
  1.70 mm minimum land width as a recorded waiver (`lib/do214/docs/README.md`).

### LCSC bindings

Each part binds its primary with a half turn: `supplier lcsc "C7420331"
rotate 180` (B5819WS), `"C7420365"` (SS34) and `"C7420320"` (1N4148WT).

- **Map rows.** The three part numbers are in
  `tools/sourcing_data/lcsc_map.json` from the backfill run of 2026-11-11
  (`tools/sourcing_data/lcsc_backfill/2026-11-11-hongjiacheng/`),
  which read every candidate page for each MPN (51, 81 and 53 pages). Each
  page names the manufacturer "hongjiacheng", matched through the reviewed
  alias; the catalog class is Preferred.
- **Placement evidence, and its limits.** On the 2026-11-08 re-verify
  preview the assembler's model for each part number had its cathode at +x, a
  half turn from the land. With `rotate 180` applied, the model's minus sign
  and cathode band sat at the land's cathode bar on all four copies of each
  part (top and bottom, 0 and 37 degrees). This was read by eye: the
  measuring script gives no angle for a model that carries two marks. It is a
  polarity reading for that part number in that preview, not a measured angle
  and not a check of the part's identity (the SS34 model is labelled
  SMAJ12A).
- **Frame rows.** `w4d-sod2613x110n-c7420331`, `w4d-diom5026x241n-c7420365`
  and `w4d-sod1608x70n-c7420320` in `tools/sourcing_data/jlc_frames.json` are
  copies of the rows archived in
  `tools/sourcing_data/jlc_previews/2026-11-08-reverify/frames.json`, keyed
  to `sod` 0.1.3 and `do214` 0.1.2. The readings were made on the 0.1.0
  lands; no pad has changed since (on these lands only the SOD1608X70N
  silkscreen body lines differ).

## Hongjiacheng SOD-123 Zeners (diode 0.1.3)

`src/zener.cohdl` adds the `Zener<VZ, P>` device and 285 parts, one per row of
seven Hongjiacheng electrical-characteristics tables: BZT52C (38, 350 mW),
BZT52B (33, 500 mW), MMSZ52xxB (47, 500 mW), MMSZ46xx (40, 500 mW), MM1Z..B
(39, 500 mW), MM1Z (38, 500 mW) and MM1W (50, 1 W). The MM1Z sheet prints
MM1Z43 twice with different values; it is left out and the record says why. The file is generated by `tools/gen_zener.py` from
`tools/diode_data/hongjiacheng_zener.json`; do not hand-edit it.

- **Land.** All seven sheets print the outline and suggested pad layout the
  `sod::SOD3716X135N` land was drawn from, so every part uses it. No land changes.
- **Pins.** Pad 1 is the cathode (the band end), as for the other parts here.
- **Sources.** `hongjiacheng-zener.evidence.md` records each sheet's URL,
  revision and SHA-256, and the page of every value.

### LCSC bindings (123 of the 285)

A Zener is polarized and outside W3 sampled inference, so each part number is
bound only with its own preview reading. 123 parts bind their primary: 98 with
no mapping, `supplier lcsc "<part number>"`, and 25 with a half turn,
`supplier lcsc "<part number>" rotate 180`.

- **Map rows.** The B1 snapshot lists 128 of these MPNs on the SOD-123
  package. Their part numbers are in `tools/sourcing_data/lcsc_map.json` from
  the backfill run of 2026-11-21
  (`tools/sourcing_data/lcsc_backfill/2026-11-21-hongjiacheng-zener/`), which
  read every candidate page for each MPN (2 to 46 pages). For each MPN exactly
  one page names the manufacturer "hongjiacheng", matched through the
  reviewed alias; the catalog class is Preferred.
- **Placement evidence, and its limits.** Preview boards 8 and 9
  (`tools/sourcing_data/jlc_previews/2026-11-21-board-8/` and `-board-9/`)
  placed 124 of the 128 four times each (top and bottom, 0 and 37 degrees)
  with no mapping. For 98 the assembler's model showed its cathode band and
  minus sign at our pad 1 on all four copies, its long axis within 2.4 degrees
  of the pad axis and its centre within 0.176 mm of the footprint origin
  (`body.json` in each record). This was read by eye: the measuring script
  gives no angle for a model that carries two marks. It is a polarity reading
  for that part number in that preview, not a measured angle and not a check
  of the part's identity.
- **The 25 with `rotate 180`.** On boards 8 and 9 these part numbers' model
  (the assembler's larger SOD-123 model, in consecutive part-number runs) had
  its band and minus sign at our pad 2 on all four copies. Preview board 10
  (`tools/sourcing_data/jlc_previews/2026-11-21-board-10/`) placed them again
  with `rotate 180` applied: on all four copies of each the band and minus
  sign sat at our pad 1, the body within 1.1 degrees and 0.129 mm. The half
  turn belongs to these part numbers' model, not to the series or the land.
- **Frame rows.** One row per part number in
  `tools/sourcing_data/jlc_frames.json` (`zb8-…`, `zb9-…` and `zb10-…`),
  keyed to `sod` 0.1.3, the land version the boards placed.
- **Not bound.**
  - C22466344 (MM1Z6V2B) was not drawn on board 9.
  - Four were left out of the boards for low stock and have no reading:
    C19077397 (BZT52C3V6), C22395534 (MM1Z2V0B), C22395535 (MM1Z2V7B)
    and C22466340 (MM1Z3V3B).
  - The other 157 parts have no map row: the snapshot does not list them.

## SOT-23 parts (`diode` 0.1.4)

`src/sot23.cohdl` holds 39 SOT-23 switching, Schottky and Zener diodes, single and paired (`*_SOT23`), generated by `tools/gen_sot23_diodes.py` from the manufacturers' sheets (evidence in `docs/sot23.evidence.md`). Every part sits on `sot::SOT3P190X290X130N`; the device's pins follow the sheet's numbered SOT-23 diagram.

### LCSC bindings (38 of the 39)

A polarized part binds only with its own preview reading: the assembler's model, with the binding's mapping, must put its pin 1 mark at our pad 1 on four copies (top and bottom, 0 and 37 degrees). The readings are preview boards 17 and 19 (`tools/sourcing_data/jlc_previews/2026-11-23-board-*`); each binding's frame row (`tools/frame_boards/sot23_frames.py`) cites its record, and its part number is the map row of the 2026-11-23 and 2026-11-24 backfills (`tools/sourcing_data/lcsc_backfill/2026-11-23-sot23/`, `2026-11-24-sot23-2/`). 38 parts are bound: 20 with no mapping, 7 with `rotate 180`, 11 with `rotate 90`. The turned ones are Zhuhai Hongjiacheng parts drawn a quarter turn (11 diodes) or, the 7 BZX84C Zeners, a half turn against the land on board 17, each read at pad 1 with its rotation on board 19. 1SS226 (C7502731) had no stock for board 17 and has no reading, so it is not bound.

## Hongjiacheng SOD-323 and SOD-523 Zeners (`diode` 0.1.5)

`src/zener_sod323_sod523.cohdl` adds 173 parts on the `Zener<VZ, P>` device, one
per row of five Hongjiacheng electrical-characteristics tables: BZT52C..S (38),
BZT52B..S (31), MM3Z (38) and MMSZ52xxBS (35) in SOD-323 at 200 mW, and BZX584C
(31) in SOD-523 at 150 mW. The file is generated by `tools/gen_zener.py` from
`tools/diode_data/hongjiacheng_zener_sod323.json`; do not hand-edit it.

- **Land.** The four SOD-323 sheets print the outline and suggested pad layout
  `sod::SOD2613X110N` was drawn from, the SOD-523 sheet the one of
  `sod::SOD1608X70N`; each series' data names its land. No land changes.
- **Pins.** Pad 1 is the cathode (the band end).
- **Sources.** `hongjiacheng-zener-sod323-sod523.evidence.md` records each
  sheet's URL, revision and SHA-256, the page of every value and two
  transcription corrections (a letter O in BZT52B3V0S's printed MPN, a doubled
  decimal point in one MM3Z temperature coefficient).
- **Bindings.** None yet: each part number needs its own preview reading.
