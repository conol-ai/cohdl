# Preview board 13 — W3 samples of Vishay CRCW 0201, Yageo CC 0201 and 1206, and Yageo RC 2010 and 2512

The board places 64 W3 samples and 4 controls, four times each: 0° and 37° on each side, 136 placements per side and 272 in all. `w3_boards_13_14.py` writes the manifest from the W3 report of 2026-11-23, the one boards 11 and 12 pin, which the inputs pin by hash. The samples are that report's choice, made among members with at least 30 in stock on their confirmed page. Each group's list alternates between this board and board 14, so each board carries about half of every group.

| Group | Samples | Placements per side | Other bodies per side if all draw |
|---|---:|---:|---:|
| CHIP_0201 ChipResistor Vishay CRCW | 16 | 32 | 104 |
| CHIP_0201 MLCC Yageo CC | 2 | 4 | 132 |
| CHIP_1206 MLCC Yageo CC | 3 | 6 | 130 |
| CHIP_2010 ChipResistor Yageo RC | 21 | 42 | 94 |
| CHIP_2512 ChipResistor Yageo RC | 22 | 44 | 92 |

The split follows from the rule. A reading's residual counts only while its group fills less than half of the drawn bodies on its view (`w3_rule.bias_supported`), so no group may fill a side. Here the largest group is about a third of each side. None of these groups is on boards 11 and 12, so their results do not change this board.

Each copy has a 9 mm box on the 10 mm grid: a 2512 land turned 37° spans about 8.4 mm, and the generator checks every land against the box.

The controls all passed on board 6 or board 7, and each belongs to a group not sampled here: C113780 (CHIP_0402 MLCC Yageo CC), C11702 (CHIP_0402 ChipResistor UNI-ROYAL 0402WG), C21189 (CHIP_0603 ChipResistor UNI-ROYAL 0603WA), C17477 (CHIP_0805 ChipResistor UNI-ROYAL 0805W8). Their rows are copied from board 7's archived manifest.

Local parts select each exact C number with no frame correction. The part is the library's part for the confirmed MPN, looked up in the released passive 0.2.8: the primary for Yageo, the part whose alternate it is for Vishay CRCW. The stock is the confirmed page's. The BOM step of an authorized preview shows the stock of the day. On this board C2078381 (CRCW0201511RFKED) read 30 in stock on its page, under the 40 the BOM step needs, so it is the likeliest to be short there. A part number that is short there is left out of a rebuilt package, and the rebuilt package needs its own approval.

## Build

The released passive 0.2.8 and std 0.3.0 are frozen in inputs.json and cohdl.lock, as on boards 7, 11 and 12. No land has changed since 0.2.6.

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py preview-board-13 -o /tmp/preview-board-13.zip \
  --cohdl "$PWD/target/debug/cohdl" \
  --kicad-cli "$PWD/tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes" \
  --kicad-python "$K/Frameworks/Python.framework/Versions/Current/bin/python3"
```

The Gerber export leaves out reference text so it cannot cover a chip body. After rendering, read the nonpolar bodies by method 2 (orientation modulo 180°). Apply the current W3 rule to every copy, keeping missing and failed readings, and archive the exports, lossless pixels, BOM checks and readings.
