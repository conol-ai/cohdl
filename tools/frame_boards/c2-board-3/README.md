# C2 frame board 3

The third frame-alignment board of library plan phase C2. It holds 23 LCSC parts. Most are placed four times: top 0°, top 37°, bottom 0° and bottom 37°. The two BGAs are placed once, at top 0°. That makes 86 placements on a board of about 120 × 90 mm. No row carries a supplier mapping.

The 2026-10-06 list of 14 families was not kept, so this board takes up what boards 1 and 2 left open, and the most-used footprints that still have no row in `jlc_frames.json`. Every part number is a confirmed page: the W1 snapshot of 2026-10-30 (`jlc_catalog/2026-10-30/`), or the B2 sample's confirmations (`lcsc_backfill/2026-10-30-sample200/`).

## What it checks

**Chip sizes no board has covered.** These four footprints have about 1,000 library parts each. Each row is a library part the official map binds (`lcsc_map.json`), so a confirmed frame is that binding's own evidence.

| Row | Part | Footprint |
|---|---|---|
| C0201_680P | C718246 CC0201KRX7R9BB681 | `passive::CHIP_0201` |
| C0201_330P | C541229 CC0201KRX7R7BB331 | same |
| R0201 | C138161 RC0201FR-07221RL | same |
| R1210_510R | C309589 RC1210FR-07510RL | `passive::CHIP_1210` |
| R1210_25R5 | C874860 RC1210FR-0725R5L | same; 30 in stock on 2026-10-30 |
| R2010 | C877226 RC2010FK-07200KL | `passive::CHIP_2010` |
| R2512_4M7 | C878293 RC2512JK-074M7L | `passive::CHIP_2512` |
| R2512_4R7 | C878096 RC2512FK-074R7L | same |

**More models on the moved footprints.** transistor 0.2.0, soic 0.3.0 and qfp 0.3.0 moved these lands, and boards 1 and 2 checked one model each. These rows bring each group to the three direct checks that the shared-evidence rule asks for. Each should read rotate 0.

| Row | Part | Footprint |
|---|---|---|
| LQFP48_F030 | C23922 STM32F030C8T6 | `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` |
| LQFP48_L151 | C9861 STM32L151C8T6 | same |
| TQFP32_M328 | C14877 ATMEGA328P-AU | `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` |
| SOIC8_LM2904 | C18229 LM2904DR2G | `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` |
| SOIC8_SP3485 | C8963 SP3485EN-L/TR | same |

**A second model on the per-model lands.** Board 1 found that SOT-23-5 and SOT-23-6 lands need a correction that depends on the model. These two rows sit on lands where board 1 read 180°, so they show whether another model reads the same.

| Row | Part | Footprint | Board 1 |
|---|---|---|---|
| SOT235_TP4054 | C32574 TP4054-42-SOT25R | `ldo::SOT5P95X290X145N` | C7972: 180 |
| SOT236_SRV05 | C7420376 SRV05-4 | `esd::FP_SOT_23_6` | C85364: 180 |

**Families with no frame row.** Each row is a catalog part of the footprint's package.

| Row | Part | Footprint |
|---|---|---|
| QFN24 | C24112 MPU-6050 | `qfn::ESPRESSIF_QFN24_0P5_4_E2P8` |
| SOD323_1N4148 | C2128 1N4148WS | `@contrib/esd::FP_SOD323_2P5X1P25mm` (polarity) |
| SOD323_1N5819 | C191023 1N5819WS | same (polarity) |
| ESOP8 | C16581 TP4056-42-ESOP8 | `audio-amp::FP_NS4168_ESOP8` |
| SW_TS1088 | C720477 TS-1088-AR02016 | `switches::FP_XUNPU_TS_1088R` |

**Retries.** Board 1's preview did not place C32346. The BGAs were short of the 20 parts board 2 needed. The assembler builds at least five boards, so a single copy needs only five parts.

| Row | Part | Footprint | Copies |
|---|---|---|---|
| XTAL3215 | C32346 Q13FC13500004 | `osc::FP_Crystal_SMD_3215_2Pin_ABS07` | 4 |
| UFBGA100 | C1338995 STM32L152VCH6 | `bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM` | 1 (10 in stock) |
| TFBGA169_ST | C24622621 STM32U595AJH6 | `bga::KICAD_ST_TFBGA_169_7X7MM_LAYOUT13X13_P0_5MM` | 1 (10 in stock) |

Rows marked "device supplies pins only" in the manifest place a catalog part whose MPN the library does not have. They reuse the footprint's library device only for its pins. The evidence is for that footprint and that part number, as on board 1.

On a BGA, pin 1 is ball A1, and `measure_preview.py` reads it as pad 1.

## Not covered

The B2 sample has no in-stock part for LQFP64, LQFP144, LQFP176, LQFP80 or the larger BGAs. It has none at all for TSSOP-14. Those families need parts from the B2 full run. Board 1's DFN1006 polarity cannot be read from the preview, because the model shows no polarity mark. LQFP100 and LQFP208 were measured on board 2 (`rotate 270`), so moving them needs no preview first. The move does need a confirming board afterwards.

## Regenerate

```sh
python3 -I tools/frame_boards/c2_board_3.py tools/frame_boards/c2-board-3/manifest.json
python3 -I tools/frame_boards/preview_package.py c2-board-3 -o PKG.zip --cohdl target/debug/cohdl \
  --kicad-cli /Applications/KiCad/KiCad.app/Contents/MacOS/kicad-cli \
  --kicad-python /Applications/KiCad/KiCad.app/Contents/Frameworks/Python.framework/Versions/Current/bin/python3
```

The generator indexes the library itself, and `preview_package.py` refuses a manifest that is not its current output. The supplier CPL is exported with `--bottom-rotation lcsc=mirrored`. The Gerbers, drill files and CPL all measure from the outline's lower-left corner.

## Checks before upload (2026-11-05)

- The compiler CPL equals `smt_pos.py --neutral` and `--map` on the board, byte for byte.
- With the drill/place origin, it also equals KiCad's own position export in position, side and top-side angle: 86 of 86 rows match.

Upload for preview only and record the result in `jlc_frames.json` with the screenshots. Do not order. A part the assembler will not select is unverified, not a pass.
