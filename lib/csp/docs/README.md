# CSP land-pattern provenance

This package contains 70 public `KICAD_*` WLCSP footprints used by the
source-backed STM32 catalog. They are generated from normalized, checked-in
source data; no STM32 component package owns or duplicates their fabrication
geometry.

The normalized source is
`tools/stm32_footprint_data/footprints.json`, imported from the official KiCad
footprint library at commit
`819223b66f96508feaeaa305301b5e6bb5c1038b` (footprint format `20260206`).
Each source-file SHA-256 is retained in the snapshot and beside its generated
declaration. Regenerate offline with:

```text
python3 tools/gen_stm32_footprints.py
```

The importer is deliberately strict. It preserves all 5,345 WLCSP pad
numbers, circular copper diameters, positions, rotations, solder-mask
expansions, and effective circular paste-aperture diameters. KiCad paste
ratios are evaluated on KiCad's one-nanometre internal grid before the result
is frozen. The rectangular courtyards remain exact. Source pin-1 polygon
vertices and fill are retained; CoHDL emits its standard silkscreen polygon
hairline. Other source package-outline/fabrication/3D graphics are outside
this focused land-pattern projection. Any new unsupported electrical geometry
causes the importer to fail rather than approximate it.

The generated library subset is attributed to the KiCad project and
contributors and is redistributed under CC-BY-SA-4.0 with the KiCad library
exception; see `LICENSE.kicad.md`.
