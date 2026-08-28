# STM32-focused KiCad footprint snapshot

`footprints.json` is the normalized fabrication-geometry source for the 103
KiCad footprints selected by `tools/stm32_data/kicad_parts.json`:

| KiCad library | Owning CoHDL package | Footprints | Electrical pads |
|---|---|---:|---:|
| `Package_BGA` | `bga` | 20 | 2,700 |
| `Package_CSP` | `csp` | 70 | 5,345 |
| `Package_QFP` | `qfp` | 10 | 1,060 |
| `Package_SO` | `soic` | 3 | 42 |
| **Total** | | **103** | **9,147** |

The source is the official KiCad footprint repository at immutable commit
`819223b66f96508feaeaa305301b5e6bb5c1038b`, using footprint format
`20260206`. The importer retains the path and SHA-256 of each selected source
file. The complete canonical `footprints.json` is pinned as
`173fe24d5e881ec4bfb4d5e9b50ee490ee577ac0509b619954714d74435109e8`;
generation refuses any byte drift until that whole-snapshot hash is reviewed
and deliberately updated. `LICENSE.kicad.md` is the upstream license notice
byte-for-byte; its SHA-256 is
`45d2bce75e5a4208f5afb01b8fb2c406e700371c4fe2b5f5cd5c443d46db4d8f`.

Ordinary generation is completely offline:

```text
python3 tools/gen_stm32_footprints.py
python3 tools/gen_stm32_footprints.py --check
```

A maintainer refresh requires an exact local checkout. The script validates
its Git commit and license checksum before reading anything:

```text
python3 tools/gen_stm32_footprints.py \
  --import-source /path/to/kicad-footprints
```

The normalized contract retains electrical pad numbers/order, copper
shape/size/position/rotation, local mask expansion, evaluated circular paste
diameter, source courtyard primitives/bounds, the filled pin-1 polygon, and
the reference anchor. KiCad paste ratios are evaluated with KiCad's
one-nanometre internal-grid rounding. Rectangular courtyards remain exact;
stepped QFP/SO courtyards are intentionally projected to their conservative
axis-aligned bounding rectangles because CoHDL has one courtyard shape. The
source pin-1 vertices/fill are retained under CoHDL's standard polygon
hairline. Other package-outline, fabrication drawing, and 3D-model graphics
are not part of this focused land-pattern projection.

The importer fails closed on any new pad kind/shape/layer/property, duplicate
electrical number, non-whole rotation, unsupported paste geometry,
non-closing courtyard, or missing/changed pin-1 marker. Generation then
re-parses its own CoHDL output and proves the exact ordered pad-number list for
every public footprint. `tests/stm32_footprints.rs` independently resolves all
four generated packages and compares every normalized geometry field, then
checks all emitted KiCad copper and paste pad lines.
