# Preview board 11 — W3 samples of Yageo CC 0603 and Yageo RC 0603, 0805 and 1206

The board places 75 W3 samples and 4 controls, four times each: 0° and 37° on each side, 158 placements per side and 316 in all. `w3_boards_11_12.py` writes the manifest from the W3 report of 2026-11-23, which the inputs pin by hash. The samples are that report's choice, made among members with at least 30 in stock on their confirmed page. Each group's list alternates between this board and board 12, so each board carries about half of every group.

| Group | Samples | Placements per side | Other bodies per side if all draw |
|---|---:|---:|---:|
| CHIP_0603 ChipResistor Yageo RC | 23 | 46 | 112 |
| CHIP_0603 MLCC Yageo CC | 7 | 14 | 144 |
| CHIP_0805 ChipResistor Yageo RC | 22 | 44 | 114 |
| CHIP_1206 ChipResistor Yageo RC | 23 | 46 | 112 |

The split follows from the rule. A reading's residual counts only while its group fills less than half of the drawn bodies on its view (`w3_rule.bias_supported`), so no group may fill a side. Here the largest group is under a third of each side.

The controls all passed on board 6 or board 7, and each belongs to a group not sampled here: C113780 (CHIP_0402 MLCC Yageo CC), C11702 (CHIP_0402 ChipResistor UNI-ROYAL 0402WG), C21189 (CHIP_0603 ChipResistor UNI-ROYAL 0603WA), C17477 (CHIP_0805 ChipResistor UNI-ROYAL 0805W8). Their rows are copied from board 7's archived manifest.

Local parts select each exact C number with no frame correction. The part is the library's primary part for the confirmed MPN, looked up in the released passive 0.2.8. The stock is the confirmed page's (backfill of 2026-11-18). The BOM step of an authorized preview shows the stock of the day. A part number that is short there is left out of a rebuilt package, and the rebuilt package needs its own approval.

## Build

The released passive 0.2.8 and std 0.3.0 are frozen in inputs.json and cohdl.lock, as on boards 6 and 7. No land has changed since 0.2.6.

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-11 -o /tmp/preview-board-11.zip \
  --cohdl "$PWD/target/debug/cohdl" \
  --kicad-cli "$PWD/tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes" \
  --kicad-python "$K/Frameworks/Python.framework/Versions/Current/bin/python3"
```

The Gerber export leaves out reference text so it cannot cover a chip body. After rendering, read the nonpolar bodies by method 2 (orientation modulo 180°). Apply the current W3 rule to every copy, keeping missing and failed readings, and archive the exports, lossless pixels, BOM checks and readings.

## Shortages at the BOM step (2026-11-23)

The first packages (board 11 `1c047d9e`, board 12 `18df571b`, from the 2026-11-22 report) stopped at the assembler's BOM step with a shortage warning. Each part number needs 40 (four placements on each of ten boards), and some had fewer in stock that day than the page read: 11 on board 11, 5 on board 12. No placement preview was taken. This board's `w3.json` lists its numbers with the stock both readings showed. The 2026-11-23 report replaces each one with its group's nearest eligible member. The other samples, which the BOM step confirmed in stock, keep their places, so each replacement lands on the same board as the number it replaces. When a preview of this board is recorded, the list moves into that record's `w3.json`, as board 4's did.
