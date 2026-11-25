# TPS25982 source audit

`tps25982.pdf` is Texas Instruments' official **TPS25982 Rev. D** data sheet
(SLVSEI3D, revised May 2026), retrieved from
<https://www.ti.com/lit/ds/symlink/tps25982.pdf> and audited on 2026-08-05.

- SHA-256: `655c21dbf6b91a3c98b7ab26cd3bf0020d3eb7d765f411c71030d22fa4b6bc1f`
- Table 5-1 supplies the logical pin names, physical assignments, and pin
  types. Thermal Pad 1 is IN (footprint pad 25); Thermal Pad 2 is GND
  (footprint pad 26).
- The Device Comparison Table identifies `TPS259823O` as the circuit-breaker
  variant with a 7.6V typical fixed overvoltage-lockout threshold. `RGET` is
  the RGE0024M 24-pin VQFN small-reel orderable suffix.
- Datasheet pages 55-57 contain the official RGE0024M package outline,
  example board layout, and 0.125mm-stencil design (drawing 4223975/B).

## Land-pattern transcription

The `TI_RGE0024M` footprint transcribes the TI drawing directly:

- 24 perimeter lands, 0.575mm x 0.24mm, R0.05mm, on 0.50mm pitch and a
  3.825mm opposing-row centre span.
- Exposed IN pad 25 is 2.70mm x 1.45mm at `(0, -0.625mm)`; exposed GND pad
  26 is 2.70mm x 0.85mm at `(0, +0.925mm)`.
- Six pad-25 vias and three pad-26 vias use TI's exact 1.10mm grid and
  Ø0.20mm drill locations.
- Four paste apertures implement the official stencil: two 1.188mm x 1.30mm
  apertures on pad 25 and two 1.188mm x 0.76mm apertures on pad 26, centred
  at `x = +/-0.694mm`. TI states 78% printed paste coverage under pad 25.
- The preferred NSMD mask opening uses TI's 0.07mm maximum expansion.

TI specifies the thermal-via drill and locations but not the copper annulus.
This library uses a 0.40mm via land around the official 0.20mm drill, giving a
0.10mm annular ring. Confirm that choice with the PCB fabricator. TI also
recommends that any vias under solder paste be filled, plugged, or tented;
that fabrication note cannot currently be encoded in a CoHDL footprint.

## Connection guide

- Join pins 1, 2, 3, 16 and exposed pad 25 to the input plane. Join pins 4,
  5, 14 and exposed pad 26 to the ground plane. Join pins 17-24 to the output
  plane. Keep all high-current copper short and wide.
- Place 0.1uF ceramic input bypass close to IN/GND. TI recommends
  0.001-0.1uF to damp input transients; add an appropriately rated TVS at IN
  and a Schottky clamp from GND to OUT when wiring inductance requires them.
- `ILIM` must have a resistor to GND. Use
  `RILIM(ohm) = 1460 / (ILIM(A) - 0.11)` within TI's 82-1650ohm range; for
  example 300ohm sets approximately 5A.
- `EN_UVLO` must not float. For a 5V always-on rail it may be tied to IN;
  otherwise use an IN-to-GND divider and
  `VIN_UVLO = 1.2V * (RVL1 + RVL2) / RVL2`. Do not exceed the pin's 6V
  recommended maximum.
- Tie `LDSTRT` to GND when the load-handshake feature is unused.
- Leave `ITIMER` open for the fastest circuit-breaker response, or use 4.7nF
  to GND for roughly 2ms of transient-overcurrent blanking.
- Leave `RETRY_DLY` and `NRETRY` open for four retries at minimum delay. A
  2.2nF capacitor from each pin to GND gives about four retries with a 100ms
  delay. Tie `RETRY_DLY` to GND for latch-off instead.
- Leave `dVdt` open for the fastest start, or use 3.3nF to GND for about
  1.4V/ms output slew (10nF gives about 0.46V/ms).
- `PG` is open drain; pull it up to a logic supply no higher than 6V (100kohm
  is TI's application-example value). `IMON` sources approximately 246uA/A;
  choose its resistor to GND so the maximum monitor voltage stays inside the
  ADC range, and optionally add a 10kohm-or-higher series RC filter.

## Families added in 0.1.1 (2026-11-25)

A family is modelled as one device per pin table and one part per stocked
member. Members whose pins behave differently are separate devices on the
same land.

| File | Document | Source | SHA-256 |
|---|---|---|---|
| `tps2663.pdf` | TPS2663x, SLVSE94G, September 2018, revised June 2024, 59 pages | <https://www.ti.com/lit/ds/symlink/tps2663.pdf> (retrieved 2026-11-25) | `bb61314ace65ed37684bd07f541fc676081d7cf81cb24020e7c5c2065bfe16b2` |
| `tps2662.pdf` | TPS2662x, SLVSDT4F, October 2017, revised December 2021, 46 pages | <https://www.ti.com/lit/ds/symlink/tps2662.pdf> (retrieved 2026-11-25) | `ff038eaa557fb618f93c0b6276a5ae08d0d3d9fbff1dddf0c45974a89120aefa` |

### TPS2663x family (60 V, 6 A; VQFN-24, RGE)

| Device | What sets it apart (p2, p4) | Member included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|
| `TPS26630` | pin 7 `OVP`, adjustable overvoltage cutoff; 1x active current limiting | `EFUSE_TPS26630RGER`: `TPS26630RGER`, C1850276, 48 |
| `TPS26631` | pin 7 `OVP`; 2x pulse-current support | `EFUSE_TPS26631RGER`: `TPS26631RGER`, C1850273, 639 |
| `TPS26632` | pin 7 `PLIM`, output power limiting; fixed 35 V overvoltage clamp | `EFUSE_TPS26632RGER`: `TPS26632RGER`, C1850278, 1,626 |

- **Pins read.** pp3-4, Figure 5-1 and the VQFN column of Table 5-1:
  IN 1 and 2, B_GATE 3, DRV 4, IN_SYS 5, UVLO 6, OVP or PLIM 7, GND 8,
  dVdT 9, ILIM 10, MODE 11, SHDN 12, IMON 13, FLT 14, PGTH 15, PGOOD 16,
  OUT 17 and 18, N.C 19 to 24, PowerPAD to the GND plane. Checked from the
  table to the model and back.
- **Obligations.** `optional`: B_GATE and DRV (floating without a blocking
  FET), dVdT (floating gives the fastest slew, p17), IMON, FLT and PGOOD
  (floating when unused), N.C. The PowerPAD is pad 25 and is listed with
  pin 8 under `GND`; TI says it must not be the only GND connection.
- **Values used.** IN and IN_SYS 4.5 V to 60 V; ILIM 3 kOhm to 30 kOhm;
  PLIM 60.4 kOhm to 150 kOhm (p5).
- **The demand part is thinly stocked.** `TPS26630RGER` has 48 pieces in
  the snapshot; `TPS26632RGER` has 1,626 but is a different device (PLIM
  and a fixed clamp), not a substitute.
- **Left out.** TPS26633, TPS26635, TPS26636 and TPS26637 in RGE (38
  pieces or fewer) and every PWP (HTSSOP-20) member, which is another pin
  table on the PWP0020T land of pp56-58.
- **Land.** `TI_RGE0024H`, drawing 4219016/A (08/2017), outline p52, board
  layout p53, stencil p54: 24X 0.58 x 0.24 mm lands, 0.5 mm pitch,
  3.825 mm between opposing row centres; one 2.7 mm square exposed-pad
  land (pad 25) with five 0.2 mm vias (centre, and 1.1 mm from it along
  each axis) and four 1.188 mm square paste apertures at 0.694 mm from
  each centre line (78% coverage). It is not `TI_RGE0024M`, whose exposed
  pad is split in two. The 0.4 mm via land is this library's choice, as
  for `TI_RGE0024M`. Courtyard 4.95 mm square.

### TPS2662x family (60 V, 800 mA; VSON-10, DRC)

| Device | What it covers (p4) | Members included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|
| `TPS26620` | adjustable overvoltage cutoff, input-side reverse-polarity protection | `EFUSE_TPS26620DRCR`: `TPS26620DRCR`, C1848361, 196; `EFUSE_TPS26621DRCR`: `TPS26621DRCR`, C1848341, 7,119 |

- **Not the TPS2663x.** The request grouped the two under "TPS266xx", but
  they are two datasheets, two packages and two pin tables, so they are
  two families here.
- **Membership.** TPS26620 latches off and TPS26621 retries after an
  overload or thermal fault (p4). No pin sets that, so they are two parts
  of one device, as the TPS2553 "-1" member is.
- **Pins read.** p4, Figure 6-1 and Table 6-1: IN 1, UVLO 2, OVP 3,
  SHDN 4, RTN 5, GND 6, ILIM 7, dVdT 8, FLT 9, OUT 10, PowerPAD to the RTN
  plane. RTN and GND are different nets by design. The PowerPAD is pad 11
  and is listed with pin 5 under `RTN`. dVdT may float (p18) and FLT may
  float; both are `optional`. Checked from the table to the model and
  back.
- **Left out.** `TPS26625DRCR` (C2862873, 2,132), `TPS26623DRCR`,
  `TPS26624DRCR` and `TPS26622DRCR`: the 22/23 members clamp at a fixed
  38 V with OVP tied to RTN, and the 24/25 members add output-side
  reverse-polarity protection, so each pair needs its own device.
- **Land.** `TI_DRC0010J`, drawing 4218878/B (07/2018), outline p43, board
  layout p44, stencil p45: 10X 0.6 x 0.24 mm lands, 0.5 mm pitch, 2.8 mm
  between row centres; an exposed-pad land of 1.65 x 2.4 mm with four
  0.25 mm fingers on 0.5 mm centres reaching 3.4 mm overall, all pad 11;
  paste apertures 2X 1.5 x 1.06 mm at 0.63 mm from the centre line and 4X
  0.25 x 0.34 mm on the fingers (80% coverage). The four 0.2 mm vias of
  the drawing are not in the land. Courtyard 3.9 mm square.

### TPS25982 family: TPS259827O

| Device | Overvoltage lockout (p4) | Members included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|
| `TPS25982` (existing) | 7.6 V typical, circuit breaker | `EFUSE_TPS259823ONRGET`: C2155764, 24 |
| `TPS259827O` | none, circuit breaker | `EFUSE_TPS259827ONRGET`: `TPS259827ONRGET`, C2155765, 1,715; `EFUSE_TPS259827ONRGER`: `TPS259827ONRGER`, C2155761, 2,935 |

- **Why a second device.** The Device Comparison Table (p4) gives the
  TPS259827O no overvoltage lockout. The pin numbers and names are those
  of Table 5-1, already modelled, but a part that does not turn off on
  input overvoltage is not interchangeable with one that does.
- **Reels.** `RGET` and `RGER` are the small and large reel of the same
  device and are two parts.
- **Land.** `TI_RGE0024M`, unchanged.
- **Left out.** `TPS259824ONRGER` (C2155766, 954; 16.9 V lockout) and the
  "LN" active-current-limiter members: each is its own device.

### LCSC part numbers (not bound)

No part here carries a `supplier lcsc` clause: a binding needs a row in
`tools/sourcing_data/lcsc_map.json` and a frame row with reviewed evidence
in `tools/sourcing_data/jlc_frames.json`, and neither exists. The candidate
numbers and stock are from the local jlcparts snapshot of 2026-11-25, which
has no manufacturer field; the part pages were not read. The preview
board input and build for these candidates is
`tools/frame_boards/demand-batch-2/`; it has not been previewed.
