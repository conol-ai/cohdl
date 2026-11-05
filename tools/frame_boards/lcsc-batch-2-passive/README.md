# LCSC batch 2 frame board: mapped passives

This is the preview board for the second batch of library bindings (implementation plan D1): the `lib/passive` part numbers that `tools/sourcing_data/lcsc_map.json` maps.

`lcsc_map.json` has 79 such rows. All of them come through the reviewed aliases (#62, #63): 64 Yageo primaries and 15 Vishay alternates. No exact-match row lands on `lib/passive`.

None of the 79 can be bound yet, because none has frame evidence that meets the library plan's W3 rule:

- **Own evidence.** No passive part number on this list has been on a preview. The passive rows already in `jlc_frames.json` (C2 board 1, C1) are for other part numbers. Most of those are UNI-ROYAL and Samsung parts. C107045 and C107110 are Yageo, but `lcsc_map.json` does not map them.
- **`same_supplier_footprint`.** This reason needs a supplier footprint identifier. The only candidate the part pages show is the package field (`0603` and so on), and it is not one: on C2 board 1, C7972 and C7722 both read `SOT-23-5` but needed rotate 180 and 270 (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`). Two chip parts that read `0603` are therefore not shown to share a model. The sampling rule (at least 3 or 5% of a group) only applies once a group has a valid reason, so there is no group to sample.
- **`same_preview_board`.** None of the 79 has been on a board.

So each part number needs its own evidence, and this board provides it in one preview. Each row declares a local part with the supplier page's manufacturer and MPN, on the library entry's device type and footprint. The CPL carries no map, so the preview shows each footprint's raw frame. Chip resistors and MLCCs are not polarised, and C1 settled the bottom-side convention, so each part gets one top copy at 0°. That gives 47 placements on a board of about 100 × 20 mm.

| Footprint | Rows | LCSC |
|---|---|---|
| `CHIP_0201` | 5 | C138084, C138161, C273318, C541229, C718246 |
| `CHIP_0402` | 10 | C93943, C106254, C107011, C114763, C137837, C137927, C477801, C482239 (alt), C2091530 (alt), C3009257 (alt) |
| `CHIP_0603` | 7 | C105575, C137787, C163421, C163429, C309563, C482832, C482876 |
| `CHIP_0805` | 12 | C107108, C137422, C141703, C163411, C185267, C273815, C482966, C483001, C483188, C527206, C527232, C576985 |
| `CHIP_1206` | 6 | C137203, C274047, C274074, C488841, C488876, C488985 |
| `CHIP_1210` | 3 | C309589, C728389, C874860 |
| `CHIP_2010` | 2 | C137046, C877226 |
| `CHIP_2512` | 2 | C878096, C878293 |

What to expect: every part sits centred on its land, along the land's long axis. That reads as rotate 0; a symmetric chip cannot tell 0 from 180. A part turned by a quarter turn, or offset from its land, is an exception and must be recorded per part number.

## Left off: no stock

The other 32 part numbers read stock 0 on their pages (2026-10-30). The assembler shows such a part with a stock warning and does not place it, as it did with C32346 on C2 board 1. `jlc_preview.py` also stops on any stock warning. Until stock returns, these 32 have no route to evidence, except the plan's fallback of inspecting a single part's page by hand. The generator prints the list on every run:

- `CHIP_0201`: C474681, C2114736, C2114977, C2115376, C2116893, C4018219, C4245720, C5716557
- `CHIP_0402`: C482284, C482301, C2085212, C3990886, C4031839, C4045510
- `CHIP_0805`: C273816, C274369
- `CHIP_1210`: C874738, C874837, C875076, C875124, C875242
- `CHIP_2010`: C877191, C877294
- `CHIP_2512`: C877726, C877732, C877740, C877770, C877787, C877827, C878167, C878177, C6757826

A few rows on the board have low stock: C2091530 3, C137787 3, C482832 5, C185267 1, C274074 2, C488841 4, C728389 5. Each needs one piece, but if any shows a stock warning at preview time, drop it and regenerate.

## After the preview

Each part that reads rotate 0 gets its own `jlc_frames.json` row with that preview as evidence. The library then binds it:
1. Bump `passive` to a new patch version. No footprint moves, so this is not breaking. Frame rows key on the version they name.
2. Name that version in the frame rows.
3. Regenerate with `tools/gen_passive.py`.
4. Update the three examples' pins and locks, and the `rfc034/bottom-preview` fixture's.

## Preview package

```sh
K=/Applications/KiCad/KiCad.app/Contents
python3 -I tools/frame_boards/preview_package.py lcsc-batch-2-passive -o lcsc-batch-2-passive-preview.zip \
  --cohdl target/debug/cohdl --kicad-cli $K/MacOS/kicad-cli \
  --kicad-python $K/Frameworks/Python.framework/Versions/Current/bin/python3
```

`preview_package.py` regenerates `manifest.json` with `lcsc_batch_2_passive.py` and refuses a manifest that differs. Uploading the package to the assembler's preview needs its own authorization.
