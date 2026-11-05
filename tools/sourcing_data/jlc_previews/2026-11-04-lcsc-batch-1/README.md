# LCSC batch 1: assembler preview, 2026-11-04

This is the assembler's placement preview of the LCSC batch 1 acceptance board (`tools/frame_boards/lcsc-batch-1/`). The board's package was built by `tools/frame_boards/preview_package.py` at commit `150907f` (on `main` as `2f31371`, the same tree, after #61's rebase merge); the package's SHA-256 is `8f148bfb6f67fdc9eaab0f29304f41be0bb6d9bc89d48dd4cbce9a8024967a31`. It was uploaded for the preview only, and the flow was stopped at Component Placements. No order was placed and nothing was added to the cart.

## What was uploaded

The board places the six `@st/stm32` 0.3.0 parts as the library ships them, each with its own `supplier lcsc` binding. Each part is placed four times: top 0°, top 37°, bottom 0° and bottom 37°. The CPL was exported with `--bottom-rotation lcsc=mirrored`, and the three LQFP bindings carry `rotate 270`.

| File | sha256 |
|---|---|
| `lcsc_batch_1-gerbers.zip` | `0969c78e98a04429b420f299ee0c233994e2433bccac4fe32921525db8f3675e` |
| `lcsc_batch_1-lcsc-bom.csv` | `dcfe8faea0ce0a1f36dab9a784a5991648bf8d94b2b8a8a60234b8c0fcf71590` |
| `lcsc_batch_1-lcsc-cpl.csv` | `7a363758ded688b6dc944481c79870b016e6b962398c21f5c93153a14e9b3ef4` |

The assembler read the board as 2 layers, 211.19 × 139.57 mm. The settings were PCBA Standard, both sides, five boards, and edge rails added by the assembler (its defaults, apart from the side). The BOM step detected and confirmed all six part numbers, 20 of each, with the H743 row on its bound `STM32H743VIH6TR` alternate (C1857444). It showed no stock warning, and all 24 placements were made: 12 top, 12 bottom.

## Screenshots

`top.png` is the top view, and `bottom.png` the view from below, mirrored left to right. Both are the preview's own 2D whole-board exports, at 2723 × 4096 px, kept as exported: SHA-256 `22ba37b2034630bf33ad97d09667ca3752f81197df5045b179dcaa258f3ec994` (top) and `f4d41d3d741216f75206688535df8bc0aa71ce9e028c491b51b08ef6c806b1a5` (bottom). `measurements.json` is reproduced byte for byte from them (Method).

## Method

`measurements.json` is the output of `tools/frame_boards/measure_preview.py measure`, as for boards 1 and 2, on the two PNGs converted to BMP with `sips`. `delta_ccw_from_top` is how far the preview's pin 1 sits counter-clockwise from ours, seen from the top.

On the BGAs the script again finds two magenta marks near the A1 corner, so it leaves `delta_ccw_from_top` empty. Their angles were taken by hand from the same file, using the mark nearest our ball A1, as on board 2. That mark sits 0.10–0.15 mm from A1, and the other sits 0.5–0.9 mm outside the corner along the diagonal. No mark is within 3.5 mm of another corner, where a part turned by a quarter turn would put it.

## Results

With the bindings applied, every copy should read 0.

| Part | Binding | Top 0° / 37° | Bottom 0° / 37° (seen from top) | Read by |
|---|---|---|---|---|
| C117815 STM32F427VGT6 (LQFP100) | `rotate 270` | 2.6, 2.3 | 357.6, 357.5 | script |
| C717701 STM32L496VGT6 (LQFP100) | `rotate 270` | 2.4, 2.3 | 357.6, 357.6 | script |
| C129276 STM32F767BIT6 (LQFP208) | `rotate 270` | 1.2, 1.0 | 358.9, 358.8 | script |
| C529436 STM32L412K8T6 (LQFP32) | default | 3.4, 3.5 | 356.1, 356.2 | script |
| C1857444 STM32H743VIH6TR (TFBGA100, alt) | default | 359.2, 358.9 | 0.7, 0.6 | manual review, nearest mark |
| C730103 STM32G431RBI6 (UFBGA64) | default | 2.4, 1.5 | 358.3, 358.5 | manual review, nearest mark |

## What it shows

- **The three `rotate 270` bindings cancel the quarter turn that board 2 measured.** LQFP100 on both models and LQFP208 now read 0 on all four copies, where board 2, with no binding, read about 90 on top and 270 on the bottom.
- **The default bindings hold.** LQFP32 and both BGAs read 0, with pin 1 on pin 1 and A1 on A1.
- **Both sides and both angles agree.** Each bottom reading is the mirror image of its top reading, and each 37° copy reads like its 0° copy. That is what `--bottom-rotation lcsc=mirrored` requires.
- **The library's own clauses reach the assembler.** The BOM and CPL came from the shipped `supplier lcsc` clauses, not from a board-local mapping.

## Manual review and limits

- **The 8 BGA copies passed manual review** (2026-11-04). The script finds two marks on each and leaves the angle to a person. Taking the mark nearest ball A1 settles it: that mark lies 0.096–0.150 mm from A1, and no mark lies near another corner. A reviewer re-measured all 24 rows from the archived exports and confirmed the reading. The 16 QFP copies were read by the script.
- **Angles scatter between preview runs.** Each run makes a new project. A later run of the same package (with the updated `jlc_preview.py`) moved the marks by up to 0.34 mm, and the readings by up to 3°. Every copy still read near 0. The record keeps the first run's exports and readings.
- **The preview is the assembler's render, not a placed board.** It shows where the assembler's models land and where their pin 1 marks fall. It does not show paste, reflow or a physical part.
