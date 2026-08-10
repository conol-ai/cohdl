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
