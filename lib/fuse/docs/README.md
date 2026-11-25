# Fuse source manifest

Created 2026-11-25 (fuse 0.1.0). Entries ending in `.evidence.md` are
repository-authored evidence records; their SHA-256 values identify
manufacturer PDFs that are not redistributed here.

| Local file | Covered exact MPN | Document version/date | Source | SHA-256 |
|---|---|---|---|---|
| `littelfuse-451.evidence.md` | Littelfuse `0451.500MRL`, `0451001.MRL`, `0451002.MRL`, `0451003.MRL`, `0451004.MRL` | NANO2 451/453 Series, revised 09/10/15, 4 pages | Littelfuse (PDF served by LCSC for C3099, see the record) | `ade583d66dc303591035fa92a9f3874828f567f59a3cb4f5d24984c3de96ea87` |

## 451 family

A family is modelled as one device per pin table and one part per stocked
member. Every 451 fuse has the same two terminals on the same land; the
ampere rating is what the part is, so each rating is its own device with
its own `current_rating`, and a fuse of one rating is never an alternate
of another.

| Device | Rating | Member included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|
| `Fuse_451_0A5` | 0.5 A, 125 V | `FUSE_451_0A5`: `0451.500MRL`, C142836, 15,355 |
| `Fuse_451_1A` | 1 A, 125 V | `FUSE_451_1A`: `0451001.MRL`, C3099, 8,019 (11,642 on the part page, 2026-11-25) |
| `Fuse_451_2A` | 2 A, 125 V | `FUSE_451_2A`: `0451002.MRL`, C99547, 23,916 |
| `Fuse_451_3A` | 3 A, 125 V | `FUSE_451_3A`: `0451003.MRL`, C13982, 38,916 |
| `Fuse_451_4A` | 4 A, 125 V | `FUSE_451_4A`: `0451004.MRL`, C27515, 20,383 |

- **Pins.** The sheet numbers no terminal and marks no polarity. The
  devices name the terminals `A` (pad 1) and `B` (pad 2), implement std's
  `TwoTerminal`, and take the designator "F" from this package's `Fuse`
  trait.
- **Values.** The ratings table is in the evidence record, with pages.
- **Left out.** The other ratings of the series. Many are well stocked
  (5 A C48467, 8 A C142876, 10 A C44479, 15 A C44480, 20 A C178980, and
  0.25 A C151116, in the snapshot); they were not requested, and the
  rows above 4 A have not been read. The 453 series (silver-plated caps)
  is not modelled.
- **Adding a rating.** Read its row on page 2 (voltage and interrupting
  rating change above 8 A in the snapshot's descriptions, so read them,
  do not copy 125 V), build the MPN from the amp code, and add one
  device with its `current_rating` and `voltage_rating` and one part. The
  land does not change.

### Land

`FP_LITTELFUSE_NANO2`: the sheet's "Recommended pad layout" (page 4), two
1.96 x 3.15 mm lands.

- **One rounding choice.** The drawing gives 1.96 mm lands, 2.95 mm
  between them and 6.86 mm overall, which do not add up (6.87). The land
  keeps the land size and the overall dimension: centres at +/-2.45 mm, a
  2.94 mm gap.
- **Coordinates.** The part lies along x. It is symmetric, so there is no
  mirroring question and no pin 1 or polarity mark.
- **Courtyard.** 7.4 x 3.7 mm: 0.27 mm beyond the 6.86 mm copper in x and
  0.275 mm beyond the 3.15 mm copper in y; it contains the 6.30 x 2.94 mm
  maximum body.

### LCSC part numbers (not bound)

No part here carries a `supplier lcsc` clause: a binding needs a row in
`tools/sourcing_data/lcsc_map.json` and a frame row with reviewed evidence
in `tools/sourcing_data/jlc_frames.json`, and neither exists. The candidate
numbers and stock are from the local jlcparts snapshot of 2026-11-25, which
has no manufacturer field; the part pages were not read, except C3099, read for its datasheet link. The preview
board input and build for these candidates is
`tools/frame_boards/demand-batch-2/`; it has not been previewed.
