# AOTA-B201610S3R3-101-T application note

Sources (unmodified bundled PDFs; page numbers are one-based PDF pages):

- [Abracon datasheet](https://abracon.com/datasheets/AOTA-B201610S3R3-101-T.pdf), Rev A, 2024-09-13; `abracon-aota-b201610s3r3-101-t.pdf`, SHA-256 `b15dcb32224c50ddf5c7643358bdef49177b451d72933d7fbb02f3756aca7827`.
- [Raspberry Pi hardware design guide](https://pip-assets.raspberrypi.com/categories/1214-rp2350/documents/RP-008280-DS-2-hardware-design-with-rp2350.pdf), Release 3, build 2026-08-20; `rp2350-hardware-design.pdf`, SHA-256 `cef88bc7d87e67b4262ee15f2b94b03566667143f4e8daf6931828bef6408059`.

## Part, land and winding mark

Abracon p1 specifies 3.3 uH, tolerance code M, 21 MHz minimum SRF, DCR 115/140 milliohm typical/maximum, and 20 V operating voltage. Its current table labels Isat 2.8/2.4 A and Irms 2.3/2.1 A as Typ/Max; preserve those labels rather than inventing a guaranteed minimum. Isat uses 30% inductance reduction; Irms uses approximately 40 C heating at 25 C ambient. Operating temperature includes self-heating (-40 to +125 C).

P4 recommends two 1.00 x 1.60 mm copper lands separated by 1.00 mm. Centres are x = +/-1.00 mm; overall span is 3.00 mm. Body maximum is 2.20 x 1.80 x 1.00 mm. The top-view white dot is upper right.

The manufacturer does not number terminals. This library assigns DOT/pad 1 to the right terminal and UNDOTTED/pad 2 to the left. Its silk dot preserves the same orientation outside the copper. Courtyard and silk are library choices; solder mask, paste, assembly rotation and inspection require board/process review. Identical body size does not justify substituting the Sunlord or Murata lands.

## RP2350 reference use

Raspberry Pi section 2.1, Figures 3 and 4 (PDF pp6-7) connects the dotted terminal to the 1.1 V output/DVDD and the other terminal to VREG_LX. Rotate the footprint to preserve both net assignment and the reference layout's winding direction. The figure's pad 1 is dotted; our default orientation is rotated 180 degrees from that board view.

The guide explains magnetic coupling to the output capacitor and recommends this exact orientation-marked part. Its surrounding circuit includes 4.7 uF input/output capacitors and a separate 33 ohm / 4.7 uF VREG_AVDD filter (pp6-8). Merely compiling the inductor does not qualify that circuit, placement, regulator stability or thermals. No supplier stock or assembly-service eligibility is asserted.

Manufacturer PDF notices remain intact; MIT covers the authored definitions and note.
