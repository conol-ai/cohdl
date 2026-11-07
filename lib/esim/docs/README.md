# eSIM — eUICC (eSIM) devices and parts

eUICC (eSIM) devices and purchasable parts in the MFF2 form factor. The
Truphone datasheet calls the package VFDFPN8 / DFN8 (MFF2) and says it is
"compatible with the MFF2 package defined by ETSI 102 671 release 12"
(PDF pp. 3, 5).

## ESIM_MFF2 — 8-contact MFF2 eUICC

5 x 6 x 1.27 mm package (PDF p. 2) with 8 contacts. Five carry the SIM
interface, which connects directly to a modem SIM port such as the Air780E's
USIM_VDD / USIM_RST / USIM_CLK / USIM_DAT pins. SWIO is not used and pins 4
and 5 are not connected internally.

Pin numbers and names follow Figure 1 ("VFDFPN8 pinout (top view)") and
Table 1 ("Pin descriptions") on PDF page 5 (printed page 4). Pins 1-4 run
down the left side and 5-8 up the right side, so pin 8 is opposite pin 1.

| Pin | Name | Datasheet name | Role | Obligation | Notes |
| --- | --- | --- | --- | --- | --- |
| 1 | GND | GND | power_in | required | ground supply |
| 2 | SWIO | SWIO | input | optional | not used; internal pull-up, leave open |
| 3 | IO | I/O | bidirectional | required | data; pull-down, then pull-up after card activation |
| 4 | NC4 | NC | passive | optional | not connected internally |
| 5 | NC5 | NC | passive | optional | not connected internally |
| 6 | CLK | CLK | input | required | external clock; internal pull-down |
| 7 | RST | RESET (overbar) | input | required | external reset, active low; internal pull-down |
| 8 | VCC | VCC | power_in | required | supply; class A 5 V, B 3 V or C 1.8 V (PDF p. 3), selected by the modem |

Figure 2 on the same page ("ST4SIM-200M PCB integration recommendations")
wires GND, VCC, RST, CLK and I/O to the modem, leaves SWIO and both NC pins
open, and shows a capacitor (1) from VCC to GND and a resistor (2) from VCC
to I/O. The page gives no values or footnotes for (1) and (2).

The device's pin map is this datasheet's. A part from another eUICC vendor
must be checked against it before it binds `ESIM_MFF2`.

## Parts

- `ESIM_MFF2_TRUPHONE` — Truphone SIM-S-IO3-MFF2-2 (LCSC C5122390), MFF2
  eUICC on an ST33G1M2 secure element, -40 to +105 °C (PDF pp. 2-3),
  footprint `FP_ESIM_MFF2_5x6mm`.

## Footprint `FP_ESIM_MFF2_5x6mm`

Drawn as the datasheet's top view in CoHDL's +y-down board frame: pad 1 at
the top left (-1.6, -1.95) mm with the pin-1 dot, pads 1-4 down the left
column, pads 5-8 up the right column, pad 8 at the top right
(1.6, -1.95) mm.

**The land itself is not sourced.** The Truphone datasheet gives only the
package size (PDF p. 2) and the pinout (PDF p. 5). It has no package drawing
and no contact size, pitch, row spacing, exposed pad, paste or land-pattern
data. The 0.8 x 0.8 mm pads, 1.3 mm pitch, 3.2 mm column spacing, the absence
of a centre pad, and the 6 mm body side running along the columns (the
5.6 x 6.6 mm courtyard) are carried over from 0.1.0 without a source. Check
them against the ETSI TS 102 671 MFF2 drawing or ST's VFDFPN8 package drawing
before board fabrication.

## Datasheet

| Local file | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- |
| `truphone-sim-s-io3-mff2-2.pdf` | Truphone "For Things SIM" product datasheet (eSIM as standard, single SKU, global network), dated Oct 27, 2020, 5 pages | <https://www.lcsc.com/datasheet/C5122390.pdf> (LCSC C5122390) | `3e8a48d3e17191ae87b8f25f60fa195fb43dbfd3bca97f01c5794cce181542b0` |

Page references above are 1-based PDF pages; the printed footer number is one
less on pp. 2-5. The URL is LCSC's mirror; no Truphone URL is known.

## Pin map and footprint correction in 0.2.0

Corrected 2026-11-07 against the rendered datasheet page 5 (Figure 1 and
Table 1). This is a breaking change.

- **Pin map.** 0.1.0 declared 1 VCC, 2 RST, 3 CLK, 4 IO, 5 GND, 6-8 NC, a
  logical map that matched no pad of the part. 0.2.0 declares the datasheet's
  map: 1 GND, 2 SWIO, 3 IO, 4 NC4, 5 NC5, 6 CLK, 7 RST, 8 VCC. The pin names
  VCC, RST, CLK, IO and GND are unchanged; `NC6`-`NC8` are replaced by `NC4`,
  `NC5` and the new optional `SWIO`.
- **Pad numbering.** 0.1.0 numbered both contact rows in the same direction,
  so pad 5 sat opposite pad 1 and no orientation of the part matched it. 0.2.0
  numbers the pads counter-clockwise in top view, pin 8 opposite pin 1, with
  pad 1 at the top left. The pad size, pitch and spacing are unchanged, but
  the land is turned a quarter turn (columns along y instead of rows along x),
  so the courtyard is now 5.6 x 6.6 mm and the reference text moved to
  (0, -4.1) mm.
- **Comments and docs.** The supply comment now lists class A 5 V as well as
  B 3 V and C 1.8 V (PDF p. 3). `#[doc]` on the device and part points at the
  PDF instead of this README. The package description no longer calls the
  land verified.

A board built with esim 0.1.0 has the eUICC's I/O on a pad the library called
`NC7`, and with one orientation no supply at all; it does not work. Update to
0.2.0, re-check the design's `place` rotation for the turned land, and
regenerate the board.
