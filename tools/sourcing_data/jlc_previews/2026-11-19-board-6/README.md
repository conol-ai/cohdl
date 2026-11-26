# Board 6 UNI-ROYAL placement preview, 2026-11-19

JLCPCB detected a two-layer, 86 × 96 mm board. Its PCBA BOM step confirmed all 18 C numbers and matched each of the 36 top and 36 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements. Its input was the preview-board-6 ZIP from commit `4bdcaa2`, SHA-256 `b69c84e8cedff555f76048ac6cbc384f1051f58258e89a0accd3426aeb80f529`. The board used the pinned passive 0.2.8 lands.

## Sample outcomes

Every sampled C number drew on both sides at 0° and 37°. All 56 sample placements and 16 control placements passed the nonpolar body check and W3 rule `w3-2026-11-12`. Values below are the maximum absolute angle, maximum raw body offset and maximum residual after subtracting the median vector of the same view.

| Group | Passing samples / required | Passing copies | Angle (deg) | Raw (mm) | Residual (mm) |
|---|---:|---:|---:|---:|---:|
| UNI-ROYAL 0603WA | 6 / 6 | 24 / 24 | 1.4 | 0.030 | 0.027 |
| UNI-ROYAL 0805W8 | 5 / 5 | 20 / 20 | 0.7 | 0.033 | 0.023 |
| UNI-ROYAL 1206W4 | 3 / 3 | 12 / 12 | 0.1 | 0.024 | 0.021 |

The top median was (−0.004, −0.009) mm, the bottom median (−0.010, −0.004) mm. Each uses 36 drawn bodies. For the 0603, 0805 and 1206 samples respectively, 24, 26 and 30 bodies on each side belong to another group. Thresholds were 10° and the per-land raw limit (0.206, 0.228 and 0.250 mm respectively), with a residual limit of half the raw limit. Controls C19666, C107039, C113780 and C11702 each passed at all four placements.

## Reproduction

The original exports, their lossless decoded pixels and decoder metadata are archived together. `pixels.json` gives all four SHA-256 hashes. `package/` holds the unpacked approved ZIP, including its manufacturing files and checksums. Run from the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-19-board-6
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_6-lcsc-cpl.csv" > /tmp/board6-checks.json
cmp "$D/checks.json" /tmp/board6-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board6-measurements.json
cmp "$D/measurements.json" /tmp/board6-measurements.json
```

`w3-evaluation.json` contains every designator, its C number, group, original check and method-2 W3 reading. The median is computed over all drawn chip bodies on the corresponding side. Each passing copy satisfies both `sample_ok` and `bias_supported` in `tools/sourcing/w3_rule.py`.

## Manual review and limits

The preview produced no missing models or placements needing manual review. The visual measurement establishes the body axis modulo 180° for these nonpolar samples. Other members of the three groups still require the normal W3 membership and stock checks before inferred bindings can be emitted. The BOM stock result describes this preview session.
