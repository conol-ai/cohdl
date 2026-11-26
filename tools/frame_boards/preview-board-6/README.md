# Preview board 6 — UNI-ROYAL larger chip resistors

The board places the 14 [selected samples](../uniroyal-larger-samples.md) and four controls. Each C number is placed four times, at 0° and 37° on each side. Local parts select the exact ordered C numbers with zero frame correction. The generator pins the 2026-11-18 backfill and looks up each chip resistor by its confirmed MPN. The recorded stock is from its individual part page. The manufacturer's BOM step must check actual availability.

| Series | Members | Samples | Placements per side | Other bodies per side if all draw |
|---|---:|---:|---:|---:|
| 0603WA | 116 | 6 | 12 | 24 |
| 0805W8 | 97 | 5 | 10 | 26 |
| 1206W4 | 17 | 3 | 6 | 30 |

C19666, C107039, C113780 and C11702 are controls from board 4 with measured bodies. They span three MLCC groups and one resistor group. The two sides each have 36 bodies. Use the actual drawn bodies on each exported side to evaluate W3 median support. Apply the current W3 rule to every copy, retaining missing and failed readings. This manifest records the experiment design. Placement evidence comes from the manufacturer's rendered models.

## Build

The released passive 0.2.8 and std 0.3.0 are frozen in inputs.json and cohdl.lock. The frame board tool restores them from git. Sign-off on passive 0.2.9 and its release are independent of this preview.

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-6 -o /tmp/preview-board-6.zip \
  --cohdl "$PWD/target/debug/cohdl" \
  --kicad-cli "$PWD/tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes" \
  --kicad-python "$K/Frameworks/Python.framework/Versions/Current/bin/python3"
```

The Gerber export omits reference text so it cannot cover a chip body. The checklist locates each designator. Use the repository's jlc-preview procedure for the exact approved ZIP. After rendering, use method 2 for nonpolar bodies (orientation modulo 180°) and archive the original images, lossless pixels, BOM checks and readings.
