# Board 7 Samsung CL placement preview, 2026-11-21

JLCPCB detected a two-layer, 126 × 86 mm board. Its PCBA BOM step confirmed all 24 C numbers and matched each of the 48 top and 48 bottom designators exactly once on the expected side, with all rows selected and no stock warning. The run ended at Component Placements. Its input was the preview-board-7 ZIP from commit `c1b9060`, SHA-256 `3e5ab2e0961066e1975fbdd1fb48461f5e4f9ea94489e3bec6b576c615fbd6d9`. The board used the pinned passive 0.2.8 lands.

## Sample outcomes

Every C number drew on both sides at 0° and 37°. All 72 sample placements and 24 control placements passed the nonpolar body check and W3 rule `w3-2026-11-12`. Values below are the maximum absolute angle, maximum raw body offset and maximum residual after subtracting the median vector of the same view.

| Group | Passing C numbers / on board | Passing copies | Angle (deg) | Raw (mm) | Residual (mm) |
|---|---:|---:|---:|---:|---:|
| Samsung CL 0402 | 11 / 11 | 44 / 44 | 6.2 | 0.041 | 0.041 |
| Samsung CL 0603 | 1 / 1 | 4 / 4 | 0.6 | 0.026 | 0.035 |
| Samsung CL 0805 | 6 / 6 | 24 / 24 | 1.2 | 0.040 | 0.046 |

The 0402 and 0805 rows cover every mapped member of those groups. The 0603 sample, C19702, joins C19666 and C23630, judged on earlier records, which gives that group its required three.

The largest angle is C70 (C52923, top, the 37° copy) at −6.2°, inside the 10° limit; its raw offset is 0.018 mm and its residual 0.024 mm. The other three copies of C52923 read within 1.5°. No other reading exceeds 3.6°.

The top median was (0.011, 0.004) mm, the bottom median (0.009, 0.006) mm. Each uses 48 drawn bodies. The Samsung 0402 samples have 22 bodies of their own group on each side and 26 of other groups (54%); the 0805 samples 12 and 36; the 0603 sample 2 and 46. Thresholds were 10° and the per-land raw limit (0.127 or 0.128 mm for 0402, 0.206 mm for 0603, 0.228 mm for 0805, 0.250 mm for 1206), with a residual limit of half the raw limit.

Controls C113780 (Yageo CC 0402), C107039 (Yageo CC 0603), C11702 (UNI-ROYAL 0402WG), C17477, C17888 and C21189 (UNI-ROYAL 0603WA, 0805W8 and 1206W4 jumpers), all previously passed on board 6, each passed at all four placements.

## Reproduction

The original exports, their lossless decoded pixels and decoder metadata are archived together. `pixels.json` gives all four SHA-256 hashes. `package/` holds the unpacked approved ZIP, including its manufacturing files and checksums. Run from the repository root:

```sh
D=tools/sourcing_data/jlc_previews/2026-11-21-board-7
python3 -I tools/frame_boards/jlc_preview.py --recheck "$D/top.png" "$D/bottom.png" \
  --board "$D/package/board.json" --cpl "$D/package/preview_board_7-lcsc-cpl.csv" > /tmp/board7-checks.json
cmp "$D/checks.json" /tmp/board7-checks.json
python3 -I tools/frame_boards/measure_preview.py measure "$D/package/board.json" \
  "$D/top.pixels.png" "$D/bottom.pixels.png" /tmp/board7-measurements.json
cmp "$D/measurements.json" /tmp/board7-measurements.json
```

`w3-evaluation.json` contains every designator, its C number, group, original check and method-2 W3 reading. The median is computed over all drawn chip bodies on the corresponding side. Each passing copy satisfies both `sample_ok` and `bias_supported` in `tools/sourcing/w3_rule.py`. The record's `w3.json`, which lets `tools/w3_groups.py` judge it, was added on 2026-11-22 with the bindings that cite it (passive 0.2.11). The 16 Samsung C numbers with no earlier row have their own frame rows (`pb7-`). C19702 is the third sample of the Samsung CL 0603 group (`w3_records/2026-11-22-chip0603-samsung-cl.json`).

## Manual review and limits

The preview produced no missing models or placements needing manual review. The visual measurement establishes the body axis modulo 180° for these nonpolar samples. The 0603 group's other 21 members are inferred under the W3 rule in that group record. The BOM stock result describes this preview session.
