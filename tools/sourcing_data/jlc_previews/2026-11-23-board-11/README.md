# Board 11 Yageo W3 placement preview, 2026-11-23

JLCPCB detected a two-layer, 206 × 166 mm board. Its PCBA BOM step confirmed all 79 C numbers and matched each of the 158 top and 158 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-11 ZIP from commit `771143b`, SHA-256 `e458932cd9d80c03056cbd1e355bacc15ab6f66d85d1e04932a61c37f286f74f`. The board used the pinned passive 0.2.8 lands.

The board's first two packages stopped at the BOM step and took no placement preview: `1c047d9e` with eleven part numbers short, and `5c9034a6` with C163370 short. `w3.json` keeps those twelve as `short_at_preview`, moved from the board's own `w3.json`; none of them is on the package previewed here.

## Sample outcomes

Every C number but one drew on both sides at 0° and 37°. All placements of the 74 drawn samples and the 4 controls passed the nonpolar body check and W3 rule `w3-2026-11-12`. Values below are the maximum absolute angle, maximum raw body offset and maximum residual after subtracting the median vector of the same view.

| Group | Passing C numbers / on board | Passing copies | Angle (deg) | Raw (mm) | Residual (mm) |
|---|---:|---:|---:|---:|---:|
| Yageo RC 0603 | 23 / 23 | 92 / 92 | 3.4 | 0.070 | 0.050 |
| Yageo CC 0603 | 6 / 7 | 24 / 28 | 3.7 | 0.074 | 0.066 |
| Yageo RC 0805 | 22 / 22 | 88 / 88 | 2.3 | 0.072 | 0.058 |
| Yageo RC 1206 | 23 / 23 | 92 / 92 | 1.0 | 0.076 | 0.076 |

The largest angle is C6 (C108079, top) at −3.7°, inside the 10° limit; its raw offset is 0.037 mm and its residual 0.018 mm.

The top median was (−0.012, −0.017) mm, the bottom median (0.009, −0.011) mm. Each uses 156 drawn bodies. Yageo RC 0603 and 1206 have 46 bodies of their own on each side and 110 of other groups (71%), Yageo RC 0805 44 and 112, Yageo CC 0603 12 and 144. Thresholds were 10° and the per-land raw limit (0.206 mm for 0603, 0.228 mm for 0805, 0.250 mm for 1206), with a residual limit of half the raw limit.

Controls C113780 (Yageo CC 0402), C11702 (UNI-ROYAL 0402WG), C21189 (UNI-ROYAL 0603WA) and C17477 (UNI-ROYAL 0805W8), all previously passed on board 6 or board 7, each passed at all four placements.

With board 12, which carries the other half of each group's sample, Yageo RC 0603, 0805 and 1206 each reach their 49 required samples. Yageo CC 0603 has 12 passed of 14 required and two members not drawn (C576830 on board 12, C576858 here): two of its 14 judged members, above the rule's 10%, so the group is paused.

## Reproduction

The original exports, their lossless decoded pixels and decoder metadata are archived together. `pixels.json` gives all four SHA-256 hashes: `top.png` `5f94eeb7…`, `bottom.png` `2f75c9fe…`, decoded by `sips-316` on macOS 27.0.1. `package/` holds the unpacked approved ZIP, including its manufacturing files and checksums. Run from the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-11
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_11-lcsc-cpl.csv" > /tmp/board11-checks.json
cmp "$D/checks.json" /tmp/board11-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board11-measurements.json
cmp "$D/measurements.json" /tmp/board11-measurements.json
```

`w3-evaluation.json` contains every designator, its C number, group, original check and method-2 W3 reading, as `tools/w3_groups.py` judges them. The median is computed over all drawn chip bodies on the corresponding side. Each passing copy satisfies both `sample_ok` and `bias_supported` in `tools/sourcing/w3_rule.py`.

## Manual review and limits

C576858 (CC0603KRX7RYBB471, Yageo CC 0603) showed bare pads and the centroid marker on all four copies, C29 to C32, after two exports of each side. It is unverified: not a passed sample and not a failure, and not inferred. No placement needed manual review. The visual measurement establishes the body axis modulo 180° for these nonpolar samples. The BOM stock result describes this preview session.
