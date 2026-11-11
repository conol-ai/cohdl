# Diode source manifest

Retrieved and revalidated on 2026-07-30; the three Hongjiacheng records were
added on 2026-11-11 (diode 0.1.2).

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
package (both 0.1.1). This package gains those two dependencies; no land of
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

### No supplier binding yet

LCSC lists these parts as C7420331 (B5819WS), C7420365 (SS34) and C7420320
(1N4148WT). None is bound in 0.1.2:

- **Placement evidence exists.** On the 2026-11-08 re-verify preview the
  assembler's model for each part number had its cathode at +x, a half turn
  from the land. With `rotate 180` applied, the model's minus sign and
  cathode band sat at the land's cathode bar on all four copies of each part
  (top and bottom, 0 and 37 degrees). This was read by eye: the measuring
  script gives no angle for a model that carries two marks. It is a polarity
  reading for that part number in that preview, not a measured angle and not
  a check of the part's identity (the SS34 model is labelled SMAJ12A). The
  candidate rows are archived in
  `tools/sourcing_data/jlc_previews/2026-11-08-reverify/frames.json`.
- **The map row is missing.** A binding needs the part number in
  `tools/sourcing_data/lcsc_map.json`, and rows reach that file only through
  a reviewed backfill run that reads every candidate page for the MPN. Those
  pages have not been fetched, so the clause is left out.

When the map rows exist, each part takes `supplier lcsc "<part number>"
rotate 180` and its archived row moves into `jlc_frames.json`, keyed to the
land package's version.
