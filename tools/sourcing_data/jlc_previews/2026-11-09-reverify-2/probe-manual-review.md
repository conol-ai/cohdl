# Manual review of the 2026-11-08 probe preview: 2N7002W, AMS1117-3.3, W25Q32

Source: the reviewer's round 5 run, `review-78-101/.work/round5/jlc-run/` (preview `top.png`/`bottom.png`, `upload/board.json`, `measurements.json`), read only. No new upload. Crops: `../../prev7/`.

Pad distances are from each JLC pin 1 mark to our pads, in board millimetres, from `measurements.json` dots and `board.json` pads.

## 2N7002W C7420322 (SOT-323, U45–U48)

| Ref | Side, angle | Nearest pad | Next |
|---|---|---|---|
| U45 | top 0° | 1, 0.48 mm | 2, 1.35 mm |
| U46 | top 37° | 1, 0.51 mm | 2, 1.37 mm |
| U47 | bottom 0° | 1, 0.49 mm | 2, 1.35 mm |
| U48 | bottom 37° | 1, 0.46 mm | 2, 1.37 mm |

The mark sits just beyond the outer end of our pad 1. The model's own pin 1 dimple is in the same corner, its two leads lie on pads 1 and 2 and its single lead on pad 3. The 11° ray readings come from the mark sitting at the pad's outer end, not from a turned body. Read by hand: pin 1 on pad 1 at all four.

## AMS1117-3.3 C6186 (SOT-223, U49–U52)

| Ref | Side, angle | Nearest pad | Next |
|---|---|---|---|
| U49 | top 0° | 1, 1.38 mm | 2, 2.54 mm |
| U50 | top 37° | 1, 1.40 mm | 2, 2.58 mm |
| U51 | bottom 0° | 1, 1.33 mm | 2, 2.52 mm |
| U52 | bottom 37° | 1, 1.30 mm | 2, 2.54 mm |

The mark sits beyond the outer end of pad 1 (pitch 2.3 mm, so the next pad is clearly farther). The model's dimple is at the pad 1 corner, its three leads lie on pads 1–3 and its tab on pad 4. Read by hand: pin 1 on pad 1 and the tab on pad 4 at all four.

## W25Q32RVXHJQ C32246052 (XSON-8 2×3, U13–U16)

Datasheet Rev E (`lib/flash/docs/w25q32rv-datasheet.pdf` on #93): page 6 Figure 1b, top view, pads 1–4 down the left, 5–8 up the right, pin 1 top-left; page 78, top view D = 2.00 vertical, E = 3.00 horizontal, pin 1 indent top-left. Our `FP_W25Q32RV_XSON8_2X3` matches: pads at x = ±1.375, pad 1 at (−1.375, −0.75).

In the preview, JLC's model is 2.0 mm wide and 3.0 mm tall (scan through U13's centre: 2.98 mm tall; U15 from below: 1.96 × 3.02 mm), with its pin 1 dimple and the pin 1 mark at the lower-left, nearest our pad 4 (1.25 mm). A mirror keeps width and height; a quarter turn counter-clockwise swaps them and moves the top-left corner to the lower-left. So the model is a quarter turn off, as the first ray reading (95°) suggested, and the body is not aligned. Candidate 270, put on the second re-verify board. Unverified until that preview.
