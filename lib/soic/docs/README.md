# SOIC/TSSOP land-pattern provenance

The package's existing hand-audited land patterns remain in
`src/footprints.cohdl`. `src/kicad_generated.cohdl` adds three public
`KICAD_*` patterns used by the source-backed STM32 catalog:

- `Package_SO:SOIC-8_3.9x4.9mm_P1.27mm`
- `Package_SO:TSSOP-14_4.4x5mm_P0.65mm`
- `Package_SO:TSSOP-20_4.4x6.5mm_P0.65mm`

The normalized source is
`tools/stm32_footprint_data/footprints.json`, imported from the official KiCad
footprint library at commit
`819223b66f96508feaeaa305301b5e6bb5c1038b` (footprint format `20260206`).
Each source-file SHA-256 is retained in the snapshot and beside its generated
declaration. Regenerate offline with `python3 tools/gen_stm32_footprints.py`.

The import preserves every pad number, round-rectangle copper size and corner
radius, position, rotation, and default copper-following mask/paste geometry.
KiCad's stepped SOIC/TSSOP courtyards cannot be authored as one CoHDL
courtyard, so they are intentionally projected to the conservative
axis-aligned bounding rectangle of the exact source outline. The source pin-1
polygon vertices and fill are retained; CoHDL emits its standard silkscreen
polygon hairline. Other package-outline/fabrication/3D graphics are outside
this focused land-pattern projection.

The generated subset is attributed to the KiCad project and contributors and
is redistributed under CC-BY-SA-4.0 with the KiCad library exception; see
`LICENSE.kicad.md`. Existing hand-authored declarations retain their original
MIT terms. The package manifest records both licenses for the aggregate.

## Assembler frame (library plan W3)

Version 0.3.0 moves these footprints onto the assembler's frame, measured on the
preview of 2026-10-30 (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`).
Each delta is the RFC-034 section 2 triple `(dx, dy, r)` from the old frame to the new one:

| Footprint | Delta |
|---|---|
| `SOIC8P127X790X216N` | `(0mm, 0mm, 270)` |
| `TI_D0008A_4214825C` | `(0mm, 0mm, 270)` |
| `TSSOP24P65_TI_PW0024A` | `(0mm, 0mm, 270)` |
| `KICAD_SOIC_8_3_9X4_9MM_P1_27MM` | `(0mm, 0mm, 270)` |
| `KICAD_TSSOP_20_4_4X6_5MM_P0_65MM` | `(0mm, 0mm, 270)` |

Pad numbers, pad symbols and land sizes are unchanged; pads, courtyard and
silkscreen turn together. This is a breaking change: a design that `place`s
one of these footprints keeps its copper only after each placement is composed
with the delta, which `tools/realign_place.py` does. Earlier versions stay
available unchanged.
