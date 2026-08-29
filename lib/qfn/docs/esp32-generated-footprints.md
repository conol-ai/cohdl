# Generated ESP32 footprint geometry

This package contains 15 generated ESP32-family land patterns frozen by
`tools/gen_esp32_footprints.py`.  Ordinary generation is offline.  The
normalized snapshot records every pad stack and placement, source URL/path and
SHA-256, ownership, and the projection contract.

KiCad's generic footprint library is pinned only as a secondary symbol-reference
cross-check and is CC-BY-SA-4.0 with the KiCad library exception; its exact
notice is shipped as `LICENSE.kicad.md`.  Generated QFN land geometry comes
from the direct Espressif PADS evidence described below.  Direct website PADS files are used only as dimensional
evidence: raw files are not bundled.

CoHDL cannot emit footprint keepout zones.  Exact source antenna keepout
polygons are therefore visible silkscreen guides only and remain unenforced;
apply the module datasheet's RF clearance in board layout.  Non-rectangular
courtyards are conservative bounding rectangles.  Pin 1 uses CoHDL's semantic
marker, and non-land body/fabrication graphics are omitted.

Direct PADS exposed-pad copper and repeated-number VIA16_10/VIA20_10 thermal
vias are retained.  The pinned files contain no level-123 Paste Mask Top
polygons, so paste follows the exact copper islands (or the continuous EP).
Levels 121/128 are top/bottom solder-mask evidence, not stencil windows; their
independent arbitrary polygons cannot be represented and are counted
separately in the snapshot rather than projected as invented paste.

PADS integer coordinates use 1/1,500,000 mm database units.  Recurring values
are rounded half-up once to CoHDL's 10^-15 mm literal grid, so the maximum
coordinate projection is 0.5 femtometres.
