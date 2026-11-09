# AW86224A — I²C LRA haptic driver with F0 tracking, FCQFN 1.37×1.37-9L

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/haptic::AW86224",
  "parts": ["@contrib/haptic::HAPTIC_AW86224"],
  "identities": [
    {"mfr": "Awinic", "mpn": "AW86224AFCR", "lcsc": "C5162563"}
  ],
  "sources": [
    {"id": "ds", "title": "AW86224A/B Low Power F0 Detect and Tracking LRA Haptic Driver", "revision": "V1.7, November 2021, 42 pages",
     "url": "https://www.awinic.com", "sha256": "96b824cf1f6467ad11531ae8ad29e66c5dc14a7aeb6d84ec396cfccab26ae096", "local": "lib/@contrib/haptic/docs/aw86224.pdf"},
    {"id": "drv", "title": "DRV2605 Haptic Driver for ERM and LRA (cited only for its broadcast address)", "revision": "SLOS825E, December 2012, revised April 2018",
     "url": "https://www.ti.com/lit/ds/symlink/drv2605.pdf", "sha256": "1207dc57a2b216298944cfa2df28b866fa1c9affd1f0cd5c6deb448ec5c49dc3", "local": "lib/@contrib/haptic/docs/drv2605.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. For `ds` they equal the printed page numbers on every page (1–42). For `drv` (TI SLOS825E) they equal TI's printed numbers on pp. 1–58.

The source URL is only the manufacturer's home page. Neither `lib/@contrib/haptic/docs/README.md` nor the PDF gives a document URL.

## Summary

The AW86224A/B is an H-bridge driver for a linear resonant actuator (LRA), with back-EMF F0 detection and tracking, an auto-brake engine, 3 KB of waveform SRAM and an I²C interface up to 1 MHz (ds p. 1). It runs from 3.0 V to 5.5 V (ds p. 6). The A model includes an Immersion IP licence; the B model adds a dedicated software licence (ds p. 1). The I²C address is fixed at 0x58 (ds p. 20). RSTN low puts the whole chip, I²C included, in power-down and clears its registers and memory (ds p. 2, p. 10). It drives LRAs only; for ERMs use another part.

## Key parameters

TA = 25 °C, VDD = 3.6 V, RL = 8 Ω + 100 µH, f = 160 Hz unless stated (ds p. 6).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage VVDD | 3 | — | 5.5 | V | On pin VDD | ds p. 6 |
| VREG pin voltage | 1.65 | 1.8 | 1.95 | V | Internal LDO output | ds p. 6, p. 3 |
| Logic input low VIL | — | — | 0.5 | V | RSTN, TRIG/INTN, SCL, SDA | ds p. 6 |
| Logic input high VIH | 1.3 | — | — | V | RSTN, TRIG/INTN, SCL, SDA | ds p. 6 |
| Logic output low VOL | — | — | 0.4 | V | TRIG/INTN, SDA; IOUT = 4 mA | ds p. 6 |
| Output offset voltage | −30 | 0 | 30 | mV | I²C signal input 0 | ds p. 6 |
| Shutdown current | 0 | 0.1 | 1 | µA | RSTN = 0 V | ds p. 6 |
| Standby current | 0 | 3 | 8 | µA | RSTN = 1.8 V | ds p. 6 |
| Quiescent current | 0.8 | 1.3 | 1.8 | mA | — | ds p. 6 |
| Under-voltage protection | 2.6 | 2.7 | 2.8 | V | Hysteresis 50 / 100 / 150 mV | ds p. 6 |
| Over-temperature threshold | — | 160 | — | °C | Recovery 130 °C | ds p. 6 |
| Shutdown to standby time | — | — | 3 | ms | — | ds p. 6 |
| Waveform start-up time | 0.2 | 0.4 | 0.8 | ms | Trigger to output | ds p. 6 |
| H-bridge Rdson (NMOS + PMOS) | 500 | 750 | 1000 | mΩ | VDD = 4.2 V | ds p. 6 |
| Load impedance threshold for OCP | — | 2 | — | Ω | — | ds p. 6 |
| PWM output frequency | 95.04 | 96 | 96.96 | kHz | VDD = 4.2 V, PD_HWM = 0 | ds p. 6 |
| PWM output frequency | 47.52 | 48 | 48.48 | kHz | VDD = 4.2 V, PD_HWM = 1 | ds p. 6 |
| F0 calibration accuracy | F0 − 2 | F0 | F0 + 2 | Hz | — | ds p. 6 |
| Output peak voltage | 3.3 | 3.6 | 4 | V | RL = 8 Ω + 100 µH, VDD = 4.2 V | ds p. 6 |
| Output peak voltage | 3.6 | 3.8 | 4.2 | V | RL = 16 Ω + 100 µH, VDD = 4.2 V | ds p. 6 |
| I²C clock | — | — | 1000 | kHz | Fast-mode Plus | ds p. 7, p. 20 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VDD | −0.3 to 6.0 | V | ds p. 5 |
| VREG | −0.3 to 2.0 | V | ds p. 5 |
| HDP, HDN (also ≤ VDD + 0.3 V) | −0.3 to 6.0 | V | ds p. 5 |
| TRIG/INTN, SDA, SCL, RSTN | −0.3 to 6.0 | V | ds p. 5 |
| Minimum load resistance RL | 5 | Ω | ds p. 5 |
| Junction-to-ambient θJA | 120 | °C/W | ds p. 5 |
| Maximum junction temperature | 150 | °C | ds p. 5 |
| Storage temperature | −65 to 150 | °C | ds p. 5 |
| Lead temperature (soldering, 10 s) | 260 | °C | ds p. 5 |
| ESD, HBM | ±2 | kV | ds p. 5 |
| ESD, CDM | ±1.5 | kV | ds p. 5 |
| Latch-up | ±200 | mA | ds p. 5 |

## Recommended operating conditions

The datasheet has no recommended-operating-conditions table. The supply range is from the electrical characteristics; the temperature range is listed among the absolute maximum ratings and in the ordering table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VVDD | 3 | 5.5 | V | ds p. 6 |
| I²C interface high level | 1.8 | 3.3 | V | ds p. 20 |
| I²C pull-up resistance | 1 | 10 | kΩ | ds p. 20 |
| Operating free-air temperature | −40 | 85 | °C | ds p. 5, p. 4 |

## Pins

Top view, A1 at the upper left; row A along the top, column 1 at the left (ds p. 2, Figure 1).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | TRIG_INTN | TRIG/INTN | I/O | Trigger input or interrupt output (default INTN); pull-up needed as INTN | ds p. 2 |
| A2 | VREG | VREG | power | Internal LDO output (1.8 V); capacitor to GND | ds p. 2, p. 3 |
| A3 | HDP | HDP | output | Positive H-bridge output | ds p. 2 |
| B1 | SDA | SDA | I/O, open-drain | I²C data | ds p. 2 |
| B2 | RSTN | RSTN | input | Active-low reset; high = standby/active, low = power-down | ds p. 2 |
| B3 | GND | GND | ground | Supply ground | ds p. 2 |
| C1 | SCL | SCL | input | I²C clock | ds p. 2 |
| C2 | VDD | VDD | power | Chip supply | ds p. 2 |
| C3 | HDN | HDN | output | Negative H-bridge output | ds p. 2 |

Checked by hand against Figure 1 (ds p. 2) and the land drawing (ds p. 40): `QFN9N40P137X137` puts A1 at (−0.4, −0.4) mm, row A along the top, column 1 on the left, which is the top view on both pages.

## Typical application

- Figure 3 (ds p. 3): C1 = 0.1 µF on VREG; C2 = 0.1 µF and C3 = 10 µF on VDD; R1 = R2 = 4.7 kΩ I²C pull-ups to VIO; R3 = 4.7 kΩ pull-up on TRIG/INTN; RSTN driven by a GPIO; on each output a ferrite bead B1/B2 and C4 = C5 = 0.1 nF to GND.
- Decoupling: a low-ESR 0.1 µF ceramic close to the device plus 10 µF on the VDD trace (ds p. 36).
- Output capacitors: 0.1 nF ceramic rated ≥ 10 V; 1 nF improves EMI but raises quiescent current (ds p. 36).
- I²C address: 0x58, 7-bit, fixed (ds p. 20). Pull-ups 1–10 kΩ, typically 4.7 kΩ; I²C high level 1.8–3.3 V (ds p. 20).
- TRIG/INTN defaults to INTN (open-drain style, pulled low on events) and needs a pull-up; INTN_PIN = 0 in register 0x44 makes it a trigger input (ds p. 2, p. 17–18, p. 33).
- RSTN must be high for I²C to work; the chip leaves power-down when VDD > 2.7 V and RSTN is high (ds p. 10).

## Layout notes

- Land pattern (ds p. 40): nine round lands Φ 0.22 mm, 0.40 mm pitch, 1.02 mm across outer edges. `QFN9N40P137X137` places Φ 0.22 mm lands at ±0.4 mm. It matches.
- Package (ds p. 39): body 1.370 ± 0.050 mm square, height 0.400 ± 0.050 mm, terminal Φ 0.220 ± 0.050 mm. The A1 terminal is drawn with a chamfered, non-round outline (eight terminals are labelled round, "8X"); the land pattern still uses nine round lands (ds p. 40).
- Place C1, C2, C3 as close as possible and on the same layer as the chip (ds p. 4).
- VDD, VREG and HDP/HDN traces short and wide; shield I²C and HDP/HDN with ground, away from charge-pump flying capacitors, or F0 detection can be inaccurate (ds p. 37).

## Pitfalls

- **Address 0x58 is fixed** (ds p. 20). TI's DRV2605, in the same library package, also answers broadcast address 0x58 (drv p. 23). Do not put both on one I²C bus.
- RSTN low clears all registers and memory, including loaded waveforms (ds p. 10). Reload after every reset.
- Minimum load resistance is 5 Ω (ds p. 5); the OCP trips near 2 Ω (ds p. 6).
- Supply below 3 V is outside the specified range; UVP is at 2.6–2.8 V (ds p. 6).
- I²C high level 1.8–3.3 V (ds p. 20), although the pins are rated to 6 V (ds p. 5).
- The output is a ~48/96 kHz PWM square wave (ds p. 6); large output capacitors raise losses (ds p. 36).
- Ambient −40 °C to 85 °C (ds p. 4, p. 5).

## Not established by the sources

The library's ball names and positions agree with the source (ds p. 2, p. 40). Disagreements and gaps:

- **TRIG_INTN pin type.** The source says the pin is pulled down on interrupts and needs a pull-up (ds p. 2, p. 26–27), i.e. an open-drain output in INTN mode. The library types it `bidirectional`; it cannot express open-drain, so an ERC will not ask for the pull-up.
- **"Ball" wording.** The library comments speak of "ball pitch" and "ball diameter"; the package is an FCQFN with flat terminals and a 0.003–0.005 mm stand-off (ds p. 39), not a ball array. The land values themselves match ds p. 40.
- **A1 terminal shape.** The bottom view draws A1 as a non-round terminal (ds p. 39); the land pattern draws nine round lands (ds p. 40). The library follows p. 40. The source does not reconcile the two.
- **VREG loading.** The source calls VREG "Output of LDO" (ds p. 2) and shows 0.1 µF on it (ds p. 3) but does not say whether external loads are allowed. The library types it `power_out`.
- **RSTN pull.** No internal pull-up or pull-down is stated; RSTN must be driven or pulled up (ds p. 2, p. 10).
- **Model.** The library MPN is the A model (Immersion IP licence); the B model differs only by licence (ds p. 1, p. 4).
- Not given: limits over temperature, thermal pad (none), θJA test board, rated voltage of C1–C3, and a document URL.
