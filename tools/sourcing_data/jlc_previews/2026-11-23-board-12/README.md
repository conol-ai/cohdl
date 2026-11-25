# Board 12 Yageo W3 placement preview, 2026-11-23

JLCPCB detected a two-layer, 206 × 166 mm board. Its PCBA BOM step confirmed all 76 C numbers and matched each of the 152 top and 152 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements; nothing was added to the cart. Its input was the preview-board-12 ZIP from commit `6f8f7ba`, SHA-256 `f66bc5b300875f50d47615fb342f9008bbbdee70b91831cee6a4d3fd9455dbae`. The board used the pinned passive 0.2.8 lands.

The first package of this board (`18df571b`, the same day) stopped at the BOM step with five part numbers short and took no placement preview. `w3.json` keeps those five as `short_at_preview`, moved from the board's own `w3.json`; none of them is on the package previewed here.

## Sample outcomes

Every C number but one drew on both sides at 0° and 37°. All placements of the 71 drawn samples and the 4 controls passed the nonpolar body check and W3 rule `w3-2026-11-12`. Values below are the maximum absolute angle, maximum raw body offset and maximum residual after subtracting the median vector of the same view.

| Group | Passing C numbers / on board | Passing copies | Angle (deg) | Raw (mm) | Residual (mm) |
|---|---:|---:|---:|---:|---:|
| Yageo RC 0603 | 22 / 22 | 88 / 88 | 5.7 | 0.080 | 0.073 |
| Yageo CC 0603 | 5 / 6 | 20 / 24 | 3.0 | 0.074 | 0.068 |
| Yageo RC 0805 | 22 / 22 | 88 / 88 | 2.4 | 0.069 | 0.054 |
| Yageo RC 1206 | 22 / 22 | 88 / 88 | 0.7 | 0.074 | 0.067 |

The largest angle is R18 (C113301, top) at −5.7°, inside the 10° limit; its raw offset is 0.041 mm and its residual 0.048 mm.

The top median was (−0.015, −0.017) mm, the bottom median (0.009, −0.012) mm. Each uses 150 drawn bodies. Each Yageo RC group has 44 bodies of its own on each side and 106 of other groups (71%); Yageo CC 0603 has 10 and 140. Thresholds were 10° and the per-land raw limit (0.206 mm for 0603, 0.228 mm for 0805, 0.250 mm for 1206), with a residual limit of half the raw limit.

Controls C113780 (Yageo CC 0402), C11702 (UNI-ROYAL 0402WG), C17888 (UNI-ROYAL 1206W4) and C19702 (Samsung CL 0603), all previously passed on board 6 or board 7, each passed at all four placements.

These samples are half of each group's sample in the 2026-11-23 report; board 11 carries the other half. No group here is complete on this board alone, so this record supports no binding yet.

## Reproduction

The original exports, their lossless decoded pixels and decoder metadata are archived together. `pixels.json` gives all four SHA-256 hashes: `top.png` `4a1520d4…`, `bottom.png` `a205b3b1…`, decoded by `sips-316` on macOS 27.0.1. `package/` holds the unpacked approved ZIP, including its manufacturing files and checksums. Run from the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-23-board-12
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_12-lcsc-cpl.csv" > /tmp/board12-checks.json
cmp "$D/checks.json" /tmp/board12-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board12-measurements.json
cmp "$D/measurements.json" /tmp/board12-measurements.json
```

`w3-evaluation.json` contains every designator, its C number, group, original check and method-2 W3 reading, as `tools/w3_groups.py` judges them. The median is computed over all drawn chip bodies on the corresponding side. Each passing copy satisfies both `sample_ok` and `bias_supported` in `tools/sourcing/w3_rule.py`.

## Manual review and limits

C576830 (CC0603JRNPOYBN681, Yageo CC 0603) showed bare pads and the centroid marker on all four copies, C29 to C32, after two exports of each side. It is unverified: not a passed sample and not a failure. Under the rule it is not inferred, and one part number not drawn does not pause its group. No placement needed manual review. The visual measurement establishes the body axis modulo 180° for these nonpolar samples. The BOM stock result describes this preview session.
