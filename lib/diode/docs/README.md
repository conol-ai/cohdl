# Diode source manifest

Retrieved and revalidated on 2026-07-30; the three Hongjiacheng records were
added on 2026-11-11 (diode 0.1.2); the SMBJ family was added on
2026-11-25 (diode 0.1.3).

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
| `vishay-smbj-88392.pdf` | Vishay `SMBJ30A-E3/52`, `SMBJ30CA-E3/52` | SMBJ5.0A thru SMBJ188A, document 88392, revision 09-Jan-2024, 6 pages | [Vishay](https://www.vishay.com/docs/88392/smbj.pdf) (retrieved 2026-11-25) | `1b65b57a4311128068c1dec747bbb99f6846ae932d77fcfd39904cb2d7a5cd50` |
| `mdd-smbj.evidence.md` | Microdiode Semiconductor (MDD) `SMBJ30A`, `SMBJ30CA` | SMBJ5.0(C)A thru SMBJ440(C)A, Rev:2025A7, 5 pages | Microdiode Semiconductor (PDF served by LCSC for C113998, see the record) | `95385273b5009dc9ec2f2a432e07e215f71626b825bd53a9f5525f4186566d3b` |
| `hongjiacheng-smbj.evidence.md` | Zhuhai Hongjiacheng `SMBJ30A`, `SMBJ30CA` | SMBJ series, Rev 1.1, 5 pages | Zhuhai Hongjiacheng Technology Co., Ltd (PDF served by LCSC for C19077584, see the record) | `3a3b185ee9f21932d5b2c70f5eb21f5f680c3bef1c2e5d2741a7b849bf1d2e39` |

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

## SMBJ family (diode 0.1.3)

600 W transient voltage suppressors in SMB (DO-214AA), added 2026-11-25. A
family is modelled as one device per pin table and one part per stocked
member. The SMBJ series is second-sourced under the same type numbers, so
the device is the polarity and every maker's member is a part on it, each
with its own maker's document.

| Family | Device | Pins | Members included (part: maker MPN, LCSC candidate, page stock) |
|---|---|---|---|
| SMBJ unidirectional ("A"), 30 V | `TVS_SMBJ_Uni` | `Cathode` 1 (band end), `Anode` 2 | `TVS_VISHAY_SMBJ30A`: Vishay `SMBJ30A-E3/52`, C1973126, 2,289; `TVS_MDD_SMBJ30A`: MDD `SMBJ30A`, C113998, 136,164; `TVS_HONGJIACHENG_SMBJ30A`: Hongjiacheng `SMBJ30A`, C19077584, 231,959 |
| SMBJ bidirectional ("CA"), 30 V | `TVS_SMBJ_Bi` | `IO1` 1, `IO2` 2, interchangeable, no band | `TVS_VISHAY_SMBJ30CA`: Vishay `SMBJ30CA-E3/52`, C553967, 4,043; `TVS_MDD_SMBJ30CA`: MDD `SMBJ30CA`, C113999, 111,763; `TVS_HONGJIACHENG_SMBJ30CA`: Hongjiacheng `SMBJ30CA`, C19077585, 215,839 |

- **Membership rule.** A part joins a device only when its own datasheet
  gives the same two-terminal pin table: the same outline and, for the
  unidirectional device, a cathode band. Unidirectional and bidirectional
  parts never share a device. Standoff voltage, clamping voltage, leakage
  and maker are part-level.
- **Pins read.** No sheet numbers the terminals. Vishay p1: "for
  unidirectional types the band denotes cathode end, no marking on
  bidirectional types". MDD p1 and Hongjiacheng p2: "Cathode Band (for
  uni-directional products only)". The device puts the cathode on pad 1,
  the convention of the `do214` lands (`lib/README.md`), so the cathode
  sits at the land's band mark. Checked from each sheet to the model and
  from the model back to each sheet.
- **Two polarities, two devices.** A "CA" part has no cathode: Vishay p1
  says "no marking on bidirectional types" and "Electrical characteristics
  apply in both directions". `TVS_SMBJ_Bi` therefore has two
  interchangeable pins, `IO1` and `IO2`, does not implement `Polarized` or
  `Diode`, and takes its "D" designator from the `TVS_Bidirectional`
  marker trait. A bidirectional part is never an `alt` of a unidirectional
  one. The bidirectional values are the same table rows as the "A" parts
  (markings CK instead of MK).
- **Other bidirectional TVS parts** of Hongjiacheng (`SMBJ6.0CA`,
  `SMCJ36CA`, `SMF15CA`) are in the `esd` package, added there before this
  family existed. They are not moved or duplicated here.
- **Values used (page).**

| Part | VRWM | VBR at IT | VC at IPP | Leakage at VRWM | Source |
|---|---|---|---|---|---|
| `TVS_VISHAY_SMBJ30A` | 30 V | 33.3 to 36.8 V at 1.0 mA | 48.4 V at 12.4 A | 1.0 uA | 88392 p2; PPPM 600 W p1; `-E3/52` p1 and p3 |
| `TVS_MDD_SMBJ30A` | 30.0 V | 33.30 to 36.80 V at 1 mA | 48.4 V at 12.4 A | 1 uA | Rev 2025A7 p2; PPPM p1 |
| `TVS_HONGJIACHENG_SMBJ30A` | 30.0 V | 33.30 to 36.80 V at 1 mA | 48.4 V at 12.40 A | 5 uA | Rev 1.1 p3; PPPM p1 |

- **Stock** is the value on each JLCPCB part page on 2026-11-25.
- **Left out, and why.**

| MPN | LCSC | Why it is not here |
|---|---|---|
| Brightking `SMBJ30CA/TR13` | C78418 (169,717 in stock) | The Brightking sheet served for it (TVS-SMBJ SERIES-001(A1), 6 pages, SHA-256 `aadc6abfe6518ac88091cd8a96b49eed4d05f510ea309403ee631b6570a023d8`) builds type numbers as SMBJ, voltage, C, A and prints no `/TR13` code, so the exact orderable MPN cannot be confirmed from the maker's document. |
| Vishay `-M3`, `HE3`, `HM3` grades and `/5B` reels | none confirmed | Listed on 88392 p1 and p3; no LCSC part number was confirmed for them. |
| Every other standoff voltage | none confirmed | Not requested; see the next item. |

- **Adding the next voltage.** Do not enumerate the series. For a voltage
  a design needs: (1) take the row from each maker's table (Vishay p2, MDD
  p2 to p3, Hongjiacheng p2 to p3) and record VRWM, VBR, VC, IPP and
  leakage with the page; (2) confirm an LCSC part number on its part page
  for that exact MPN and maker; (3) add one `part` per stocked maker on
  `TVS_SMBJ_Uni` for an "A" type or on the bidirectional device for a "CA"
  type. No new device and no new land. For bidirectional parts of 10 V and
  under, each sheet doubles the leakage limit; put that in the part's
  comment.

### Land

All parts use `do214::DIOM5336X265N` (`do214` 0.1.2, unchanged): two
2.10 x 2.26 mm lands at +/-2.42 mm, 2.74 mm gap, 6.94 mm outer span, drawn
from the Hongjiacheng SMBJ sheet's page 2. It is the Hongjiacheng parts'
own land. For the other two makers it is the same package with a wider gap
than their sheets suggest:

| Source | Land (length x width) | Gap | Outer span |
|---|---|---|---|
| `do214::DIOM5336X265N` | 2.10 x 2.26 mm | 2.74 mm | 6.94 mm |
| Vishay 88392 p5, "Mounting Pad Layout" | 1.52 min x 2.18 min | 2.159 max | 5.59 ref |
| MDD Rev 2025A7 p5, "Suggested Pad Layout" | 2.4 x 2.8 mm | 2.2 mm | 7.0 mm |

- **Fit, Vishay outline (p5):** overall length 5.21 to 5.59 mm puts the
  lead tips 2.605 to 2.795 mm from the centre, 0.675 to 0.865 mm inside the
  3.47 mm outer land edge. Lead width 1.95 to 2.20 mm fits the 2.26 mm
  land. The lead foot is 0.76 to 1.52 mm long, so its inner end is 1.085
  to 2.035 mm from the centre against the 1.37 mm inner land edge: on the
  shortest body with the longest foot, 0.285 mm of the heel is off the
  land. Body 4.06 to 4.57 by 3.30 to 3.94 mm.
- **Fit, MDD outline (p1):** overall length 4.90 to 5.60 mm, tips 2.45 to
  2.80 mm, 0.67 to 1.02 mm inside the outer edge. Lead width 1.80 to
  2.20 mm. Foot 0.75 to 1.55 mm, inner end 0.90 to 2.05 mm: at worst
  0.47 mm of the heel is off the land. Body 4.05 to 4.70 by 3.30 to
  3.95 mm.
- **Recorded deviation.** The land's 2.74 mm gap is larger than Vishay's
  2.159 mm maximum and MDD's 2.2 mm. The toe side and the width meet both
  sheets; the heel side does not at the tolerance corner above. This is
  the shared land used as it is, not a land drawn for these two makers. A
  design that needs their exact layout needs a new `do214` land.
- **Courtyard and marks.** 7.5 x 4.5 mm: 0.28 mm beyond the copper in x
  and at least 0.275 mm beyond the widest body (3.95 mm) in y. The band
  mark is outboard of pad 1, the cathode.
- **Coordinates.** Each sheet draws the part along x with the band at the
  left; the land keeps that view, cathode (pad 1) at -x, +y down. Nothing
  is mirrored: the part is symmetric about its long axis.

### LCSC part numbers (not bound)

No SMBJ part carries a `supplier lcsc` clause. The clauses added on
2026-11-25 were removed the same day: the library binds a part only with a
row in `tools/sourcing_data/lcsc_map.json` and a frame row with evidence in
`tools/sourcing_data/jlc_frames.json` (`tests/library/supplier.rs`), and
neither exists for these part numbers. The numbers below are candidates for
that work, not bindings.

| Part | Candidate | Stock, jlcparts snapshot 2026-11-25 | Stock, JLCPCB part page 2026-11-25 |
|---|---|---|---|
| `TVS_VISHAY_SMBJ30A` | C1973126 | 4,183 | 2,289 |
| `TVS_MDD_SMBJ30A` | C113998 | 144,108 | 136,164 |
| `TVS_HONGJIACHENG_SMBJ30A` | C19077584 | 335,509 | 231,959 |
| `TVS_VISHAY_SMBJ30CA` | C553967 | 6,205 | 4,043 |
| `TVS_MDD_SMBJ30CA` | C113999 | 150,452 | 111,763 |
| `TVS_HONGJIACHENG_SMBJ30CA` | C19077585 | 160,291 | 215,839 |

- **Identity.** Each candidate was read on its JLCPCB part page on
  2026-11-25 (`https://jlcpcb.com/partdetail/<C-number>`, through the
  shared fetch client): manufacturer ("Vishay Intertech", "MDD(Microdiode
  Semiconductor)", "hongjiacheng"), exact MPN, an SMB package, class
  Extended. The snapshot rows for C19077584 and C19077585 have an empty
  description; their part pages name "hongjiacheng" and the exact MPN.
- **The two stock columns differ, and both are kept.** The snapshot and
  the part page carry the same date and do not agree for any of the six.
  Neither is corrected to the other: stock is a reading of one source at
  one time.
- **Missing for a map row.** A reviewed `tools/lcsc_backfill.py` run that
  confirms every candidate page of the MPN, promoted with
  `tools/promote_lcsc_map.py`. The local jlcparts snapshot cannot stand in
  for it: it has no manufacturer field. For the MDD parts,
  "MDD(Microdiode Semiconductor)" is also not a reviewed alias in
  `tools/sourcing_data/mfr_aliases.json`.
- **Missing for a frame row.** An assembler preview of the part number on
  `do214::DIOM5336X265N`, reviewed by a person. The three Hongjiacheng
  diodes already bound in this package each needed `rotate 180`, and a
  unidirectional TVS half a turn off is fitted backwards, so no rotation
  can be assumed. A bidirectional part is read by its body only, per part
  number.
- **Preview board input.** `tools/frame_boards/demand-batch-1/` holds the
  manifest for that preview. It has not been built or previewed.
