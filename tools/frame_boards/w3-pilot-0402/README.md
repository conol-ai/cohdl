# W3 sampling pilot board: three 0402 groups

This board is the pilot of the sampled-inference proposal (`docs/library-supplier-w3-sampling.md`, §4). It carries the samples that the grouping report (`tools/sourcing_data/w3_sampling/2026-11-07/`) chose for three `passive` 0.2.5 groups on the same land:

| Group | Members | Own evidence | Samples here |
|---|---:|---:|---:|
| `CHIP_0402` ChipResistor Yageo `RC` | 969 | 5 | 44 |
| `CHIP_0402` ChipResistor Vishay `CRCW` | 968 | 2 | 47 |
| `CHIP_0402` MLCC Yageo `CC` | 199 | 2 | 8 |

That makes 99 part numbers, one top copy each at 0°, on a board of about 100 × 20 mm.
- **Stock.** Every part had at least 30 pieces in stock on its confirmed page (B2 run, 2026-11-05/06), the report's `min_stock`. Stock may have changed since. `jlc_preview.py` stops on any stock warning, and the preview is then unverified. A new package with other samples needs its own approval.
- **First run (2026-11-07), unverified.** The first package (SHA-256 `8f056fde04c64fb2632675a91bdd2eafb5a3214f6a3af517e5e1325c446dab0f`) drew its samples from members with at least 5 pieces. Its BOM step read "99 parts detected, 88 Parts confirmed" with a shortage warning. Exactly 11 of its samples had read fewer than 20 pieces (5 to 19), and none of the others fewer than 21. The script did not list the unconfirmed rows, because a top-only board's BOM step has no Bottom Designator column and the check stopped at the columns. Both are fixed: `min_stock` is now 30, and `jlc_preview.py` no longer needs that column on a board with no bottom parts. Nothing was measured or recorded from that run.
- **Second run (2026-11-07), unverified.** The second package (SHA-256 `85e55a26ed141e42e04d540000f650f72f2431cb6cf61fee5fab71f44fb1adfb`) read "99 parts detected, 97 Parts confirmed" with a shortage warning, and named the two rows: C185430 (its page read 100) and C185414 (39). Others with less on their page, down to 33, were confirmed, so these two had sold since. `tools/w3_groups.py` lists them under `SHORT_AT_PREVIEW` and replaces each with the nearest eligible member of its group (C138018 and C273339), so the other 97 samples stay as that preview confirmed them. Nothing was measured or recorded from that run either.
- **Third run (2026-11-07), measured.** Package `7f49d085…` (commit `e6ba33d`) passed the BOM step with all 99 confirmed and was measured. The results are in `tools/sourcing_data/jlc_previews/2026-11-07-w3-pilot-0402/`:
  - 88 bodies lie along and are centred on their pads. Each has its own frame row, archived in that record's `frames.json` and not bound: `jlc_frames.json` is unchanged, so `passive` 0.2.5 stays as published.
  - 10 models were not drawn, and C113781 needs manual review.
  - Yageo `RC` passed (44 of 44). Vishay `CRCW` (9 not drawn) and Yageo `CC` (1 not drawn, 1 manual review) are paused under the rule below.
- **No map.** As on `lcsc-batch-2-passive`, each row declares the map row's identity on the library entry's device type and footprint. The CPL carries no map, so the preview shows each footprint's raw frame.

## What to expect, and what counts

Every part should sit centred on its land, along the land's long axis: rotate 0 (mod 180). `measure_preview.py` applies the chip limits: the long axis within 10° of the land's axis, the centre within `min(0.25, 0.25 × half pitch)` mm, and a drawn, elongated body.

- **Pass:** all 99 within the limits. Each gets its own `jlc_frames.json` row with this preview as evidence. The group records are then written for review. No other member is bound until Tony accepts the proposal.
- **Fail:** any sample outside the limits, or not drawn, pauses its group. The readings are recorded here, and the proposal is revised before any other group is sampled.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py w3-pilot-0402 -o w3-pilot-0402-preview.zip \
  --cohdl target/debug/cohdl --kicad-cli $K/MacOS/kicad-cli \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

`preview_package.py` regenerates `manifest.json` with `w3_pilot_0402.py` and refuses a manifest that differs.

**Uploading the package to the assembler's preview needs Alex's approval**, like every preview. Preview runs stop at Component Placements: no order, no cart, no terms ticked.
