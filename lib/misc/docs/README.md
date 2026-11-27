# Miscellaneous PCB fabrication primitives

This package contains reusable fabrication features that are part-bound so
they can participate in normal CoHDL builds while retaining their electrical
meaning.

## Test point

`TEST_POINT_ROUND_1MM` is a one-terminal passive SMD test point. Its footprint
has a 1.0 mm circular top-copper land and the default zero solder-mask
expansion. `paste: none` deliberately suppresses the paste aperture. The
1.5 mm circular courtyard represents probe-access clearance around the land.

## M2 mounting holes

`MOUNTING_HOLE_M2_NPTH` is a mechanical, zero-terminal device. Its footprint
contains one 2.2 mm non-plated `mount_hole`, not an electrical pad, within a
4.4 mm circular courtyard.

`MOUNTING_HOLE_M2_PTH` is the electrically connectable alternative. It exposes
one passive `PAD` terminal backed by a 4.0 mm circular plated-through copper
land with a 2.2 mm drill and the same 4.4 mm circular courtyard.

The `PCB Fabrication` identities describe fabrication features rather than
purchasable components.

## Named by drill diameter

The holes added after the M2 pair are named by drill diameter —
`MOUNTING_HOLE_NPTH_1P0`, `_1P7`, `_2P0`, `_2P7`, `_3P2` — not by fastener size.
A 3.2 mm hole is M3 clearance and a 1.7 mm hole is M1.6 clearance under the usual
tables, but that is an inference about the screw a designer may fit, while the
drill diameter is the fact the fabrication house acts on. The two M2 parts keep
their names because renaming a published part changes a locked version's
content; they are the older exception, not the convention.

Each non-plated hole keeps the courtyard rule the M2 part set: a circle of twice
the drill diameter, centred on the hole. That is a clearance convention of this
package, not a vendor dimension.

## Alternative spellings

`alt` entries on a fabrication primitive record **another catalogue spelling of
the same feature**, not an alternate supplier — a bare hole or land has no
supplier to alternate between. Designs in the wild write the decimal point three
ways, so all three are recorded rather than one being declared correct:

| Drill | Recorded spellings |
|---|---|
| 2.0 mm | `MOUNTING-HOLE-2-NPTH`, `MOUNTING-HOLE-2.0-NPTH` |
| 2.2 mm | `MOUNTING-HOLE-2.2-NPTH`, `MOUNTING-HOLE-2P2-NPTH` |
| 2.7 mm | `MOUNTING-HOLE-2.7-NPTH` |
| 3.2 mm | `MOUNTING-HOLE-3.2-NPTH`, `MOUNTING-HOLE-3P2-NPTH`, `MOUNTING-HOLE-3V2-NPTH` |

`3V2` is not a typo the library should correct quietly: it is the spelling a
design already carries, and a resolver that answers it costs one line while
failing to answer it costs a re-authoured part in every project that used it.
The drilled geometry is the primary entry's, and an `alt` never changes it.

`TEST_POINT_ROUND_1P5MM` follows the 1.0 mm part: a 1.5 mm circular
top-copper land, `paste: none`, no solder-mask expansion, and a 2.0 mm
courtyard — the same 0.25 mm clearance per side the 1.0 mm part carries.

## Not added

`PlatedMountingHole` at 3.2 mm is not in the package yet. A plated hole needs a
copper land diameter, and this package has no vendor document to take one from:
the M2 part's 4.0 mm land is an existing choice, not a rule that can be scaled.
Until a fabrication standard or a vendor recommendation is cited, the 3.2 mm
hole is offered non-plated only.
