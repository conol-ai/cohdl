# Generated ESP32 footprint geometry

This package contains 40 generated ESP32-family land patterns frozen by
`tools/gen_esp32_footprints.py`.  Ordinary generation is offline.  The
normalized snapshot records every pad stack and placement, source URL/path and
SHA-256, ownership, and the projection contract.

Espressif's pinned KiCad 3.2.1 library is CC-BY-SA-4.0 with the KiCad library
exception; its notice text is shipped as `LICENSE.kicad.md`.  Direct website PADS files are used only as dimensional
evidence: raw files are not bundled.

Exact source antenna rectangles are enforced as all-copper keepouts in
KiCad footprint and board output: no tracks, vias, pads, copper pours or
components. Silkscreen outlines remain as visible guides. Additional RF
clearance and board-edge placement follow the module datasheet.  Non-rectangular
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

The pinned S31-WROOM-3 source lists F.Cu, B.Cu, and only In1.Cu; this
release conservatively extends that rectangle to all copper layers. The
source layer list remains recorded in the normalized snapshot.

This release requires a CoHDL compiler with `keepout` support (the ESP32
antenna-keepout fix, after v0.8.1). v0.8.1 rejects this syntax. IPC-2581
export rejects these footprints rather than discarding the restrictions;
use `build --emit kicad_pcb`. Pin definitions and pad geometry are unchanged.
