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
