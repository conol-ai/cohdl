# W3 sampling pilot board: three 0402 groups

This board is the pilot of the sampled-inference proposal (`docs/library-supplier-w3-sampling.md`, §4). It carries the samples that the grouping report (`tools/sourcing_data/w3_sampling/2026-11-07/`) chose for three `passive` 0.2.5 groups on the same land:

| Group | Members | Own evidence | Samples here |
|---|---:|---:|---:|
| `CHIP_0402` ChipResistor Yageo `RC` | 969 | 5 | 44 |
| `CHIP_0402` ChipResistor Vishay `CRCW` | 968 | 2 | 47 |
| `CHIP_0402` MLCC Yageo `CC` | 199 | 2 | 8 |

That makes 99 part numbers, one top copy each at 0°, on a board of about 100 × 20 mm.
- **Stock.** Every part had at least 5 pieces in stock on its confirmed page (B2 run, 2026-11-05/06). Stock may have changed since. `jlc_preview.py` stops on any stock warning. A sample that has run out is then taken off and replaced by hand from the group's eligible members, and the regenerated package needs its own approval.
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
