# TCA8418 — I²C keypad scanner, 18 GPIO (80 keys), WQFN-24 4×4 (RTW)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/keyscan::TCA8418",
  "parts": ["@contrib/keyscan::KEYSCAN_TCA8418"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TCA8418RTWR", "lcsc": "C138713"}
  ],
  "sources": [
    {"id": "ds", "title": "TCA8418 I2C Controlled Keypad Scan IC With Integrated ESD Protection", "revision": "SCPS215G, September 2009, revised June 2018 (addendum dated 13-May-2026), 53 pages",
     "url": "https://www.ti.com/lit/ds/symlink/tca8418.pdf", "sha256": "fdd74a30911f0a92a592be42f5f040600ce41609a1ce5d87b287ce435df6a944", "local": "lib/@contrib/keyscan/docs/tca8418.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal TI's printed page numbers on pp. 1–45; pp. 46–53 are the orderable addendum, tape-and-reel and RTW0024B mechanical pages, which carry no matching number. The URL is the one in `lib/@contrib/keyscan/docs/README.md`.

## Summary

The TCA8418 scans a key matrix of up to 8 rows × 10 columns (80 keys) and reports presses and releases through a 10-event FIFO over I²C at up to 1 MHz (ds p. 1). Unused row and column pins work as GPIOs (ds p. 1, p. 3). It runs from 1.65 V to 3.6 V; inputs and the bus are rated to 4.6 V absolute and 3.6 V in operation, so it is not a 5 V part (ds p. 4). The I²C address is fixed at 0x34 (0110100) (ds p. 25). INT is an open-drain, active-low output that needs a pull-up (ds p. 3, p. 20).

## Key parameters

VCC = 1.65 V to 3.6 V over the recommended temperature range unless stated (ds p. 5).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Power-on reset, VCC rising VPORR | 1.03 | — | 1.43 | V | — | ds p. 5 |
| Power-on reset, VCC falling VPORF | 0.76 | — | 1.15 | V | — | ds p. 5 |
| ROW/COL output high VOH | 2.6 | — | — | V | IOH = −8 mA, VCC = 3 V | ds p. 5 |
| ROW/COL output low VOL | — | — | 0.25 | V | IOL = 8 mA, VCC = 3 V | ds p. 5 |
| SDA sink current IOL | 3 | — | — | mA | VOL = 0.4 V | ds p. 5 |
| INT sink current IOL | 3 | — | — | mA | VOL = 0.4 V | ds p. 5 |
| Input leakage II (SCL, SDA, ROW, COL, RESET) | — | — | 1 | µA | VI = VCC or GND | ds p. 5 |
| ROW/COL internal pull-up | — | 105 | — | kΩ | — | ds p. 5 |
| Supply current, bus idle, oscillator off | — | — | 10 | µA | fSCL = 0 kHz | ds p. 5 |
| Supply current, bus idle, oscillator on | — | — | 18 | µA | fSCL = 0 kHz | ds p. 5 |
| Standby (idle) current | — | 3 | — | µA | Features list; no conditions | ds p. 1 |
| Supply current, 1 key pressed | — | — | 90 | µA | fSCL = 400 kHz, VCC = 3.6 V | ds p. 5 |
| Supply current, 1 key pressed | — | — | 153 | µA | fSCL = 1 MHz, VCC = 3.6 V | ds p. 5 |
| Input capacitance, SCL | — | 6 | 8 | pF | — | ds p. 5 |
| I/O capacitance, SDA | — | 10 | 12.5 | pF | — | ds p. 5 |
| I²C clock fSCL | 0 | — | 1000 | kHz | Fast-mode Plus | ds p. 6 |
| Reset pulse duration tW | 120 | — | — | µs | — | ds p. 6 |
| Reset recovery time tREC | 120 | — | — | µs | — | ds p. 6 |
| Interrupt valid time tIV | 20 | — | 60 | µs | Key event, key unlock or overflow | ds p. 7 |
| Interrupt reset delay tIR | — | — | 200 | ns | SCL to INT | ds p. 7 |
| Thermal resistance RθJA | — | 38.9 | — | °C/W | RTW (WQFN) | ds p. 4 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | −0.5 to 4.6 | V | ds p. 4 |
| Input voltage VI | −0.5 to 4.6 | V | ds p. 4 |
| Output voltage, high-impedance or power-off | −0.5 to 4.6 | V | ds p. 4 |
| Output voltage, high or low state | −0.5 to 4.6 | V | ds p. 4 |
| Input clamp current IIK (VI < 0) | ±20 | mA | ds p. 4 |
| Output clamp current IOK (VO < 0) | ±20 | mA | ds p. 4 |
| Continuous output low current, P port and SDA | 50 | mA | ds p. 4 |
| Continuous output low current, INT | 25 | mA | ds p. 4 |
| Continuous output high current, P port | 50 | mA | ds p. 4 |
| Storage temperature | −65 to 150 | °C | ds p. 4 |
| ESD, HBM | ±2000 | V | ds p. 4 |
| ESD, CDM | ±1000 | V | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 1.65 | 3.6 | V | ds p. 4 |
| VIH, SCL, SDA, ROW0–7, COL0–9, RESET | 0.7 × VCC | 3.6 | V | ds p. 4 |
| VIL, same pins | −0.5 | 0.3 × VCC | V | ds p. 4 |
| High-level output current, ROW0–7, COL0–9 | — | 10 | mA | ds p. 4 |
| Low-level output current, ROW0–7, COL0–9 | — | 25 | mA | ds p. 4 |
| Operating free-air temperature | −40 | 85 | °C | ds p. 4 |

## Pins

RTW package, top view: pins 1–6 down the left side, 7–12 along the bottom, 13–18 up the right side, 19–24 along the top from right to left, thermal pad in the centre (ds p. 3).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | ROW7 | ROW7 | I/O | Keypad row 7 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 2 | ROW6 | ROW6 | I/O | Keypad row 6 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 3 | ROW5 | ROW5 | I/O | Keypad row 5 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 4 | ROW4 | ROW4 | I/O | Keypad row 4 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 5 | ROW3 | ROW3 | I/O | Keypad row 3 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 6 | ROW2 | ROW2 | I/O | Keypad row 2 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 7 | ROW1 | ROW1 | I/O | Keypad row 1 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 8 | ROW0 | ROW0 | I/O | Keypad row 0 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 9 | COL0 | COL0 | I/O | Keypad column 0 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 10 | COL1 | COL1 | I/O | Keypad column 1 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 11 | COL2 | COL2 | I/O | Keypad column 2 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 12 | COL3 | COL3 | I/O | Keypad column 3 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 13 | COL4 | COL4 | I/O | Keypad column 4 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 14 | COL5 | COL5 | I/O | Keypad column 5 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 15 | COL6 | COL6 | I/O | Keypad column 6 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 16 | COL7 | COL7 | I/O | Keypad column 7 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 17 | COL8 | COL8 | I/O | Keypad column 8 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 18 | COL9 | COL9 | I/O | Keypad column 9 or GPIO; if unused, pull up to VCC | ds p. 3 |
| 19 | GND | GND | ground | Ground | ds p. 3 |
| 20 | RESET | RESET (overbar) | input | Active-low reset; pull up to VCC if not driven | ds p. 3 |
| 21 | VCC | VCC | power | Supply 1.65–3.6 V | ds p. 3 |
| 22 | SDA | SDA | I/O | I²C data; pull up to VCC | ds p. 3 |
| 23 | SCL | SCL | input | I²C clock; pull up to VCC | ds p. 3 |
| 24 | INT | INT (overbar) | output, open-drain | Active-low interrupt; pull up to VCC | ds p. 3 |
| 25 | EPAD | Thermal pad (pad 25) | — | Exposed thermal pad; connection not stated | ds p. 3, p. 50 |

Checked by hand against the rendered pin configuration (ds p. 3) and the land example (ds p. 51): `QFN24N50P400X400_1EP245X245` puts pad 1 at (−1.9, −1.25) mm, 1–6 down the left, 7–12 left to right along the bottom, 13–18 bottom to top on the right and 19–24 right to left along the top, so INT (24) is top left and GND (19) top right, as on p. 3.

## Typical application

- Pull-ups: SDA, SCL and INT to VCC; RESET to VCC unless actively driven (ds p. 3). Pull-up size from bus capacitance (ds p. 21).
- Unused ROW/COL pins: to VCC through a pull-up, 10 kΩ in the design example (ds p. 3, p. 39).
- I²C address: 0110100 + R/W, i.e. 0x34 7-bit (ds p. 25). No address pins.
- Decoupling: a larger and a smaller capacitor on VCC, placed close (ds p. 44); the source gives no values.
- All registers default to 0 (ds p. 25); configure the key matrix through the registers before use.
- Ghosting: keep keys of three-key combinations on separate rows and columns (ds p. 37).

## Layout notes

- RTW0024B land example (ds p. 51): 24 lands 0.6 × 0.24 mm, 0.5 mm pitch, 3.8 mm between opposite land-row centres, 2.45 mm square thermal land with five Φ 0.2 mm vias (0.97 mm offsets), NSMD preferred with 0.07 mm clearance. `QFN24N50P400X400_1EP245X245` places 0.6 × 0.24 mm lands at ±1.9 mm with 0.5 mm pitch and a 2.45 mm square centre pad. It matches; the vias are not modelled (the library comment says so).
- Stencil (ds p. 52): four 1.08 mm square apertures on the thermal pad at 0.64 mm offsets, 78 % coverage, 0.125 mm stencil. The library does not model paste apertures.
- Package (ds p. 50): body 3.85–4.15 mm square, 0.8 mm maximum height, terminals 0.18–0.30 mm wide and 0.3–0.5 mm long, exposed pad 2.45 ± 0.1 mm.
- Figure 38 routes all signals on a four-layer board (ds p. 44).

## Pitfalls

- 3.6 V maximum operating input level on every logic pin (ds p. 4). Do not pull the bus up to 5 V.
- The address 0x34 is fixed (ds p. 25). Only one TCA8418 per bus.
- INT is open-drain (ds p. 3, p. 20). Without a pull-up the host never sees an interrupt.
- Unused ROW/COL pins left floating go against the pin table, which asks for a pull-up (ds p. 3), even though internal 105 kΩ pull-ups exist (ds p. 5).
- A brown-out that does not drop VCC below VPORF (0.76–1.15 V) does not reset the device (ds p. 43). Use RESET or a full power cycle after a glitch.
- Errata sections for CAD interrupt and overflow handling exist; read them before writing the driver (ds p. 2 revision history, p. 35–36).

## Not established by the sources

The library's pin numbers, names and footprint geometry agree with the source (ds p. 3, pp. 50–51). Disagreements and gaps:

- **Thermal pad connection.** The library comment says "Exposed thermal pad — must be soldered to GND", and the pin is `required ... [power_in]`. The source names it only "Thermal Pad" / "EXPOSED THERMAL PAD", pad 25 (ds p. 3, p. 50), and shows vias in it (ds p. 51); it does not say to connect it to GND, to leave it floating, or that it must be soldered.
- **INT pin type.** The source says open-drain (ds p. 3). The library types INT `output`, which it cannot qualify as open-drain; an ERC will not ask for the pull-up or allow wired-OR.
- **Unused ROW/COL pins.** The library marks all 18 `optional`. The source asks for a pull-up on each unused one (ds p. 3); an `optional` pin left open passes the library check.
- **Decoupling values.** The source gives none (ds p. 44); the layout example shows two 0603 parts without values (ds p. 44).
- Not given: operating junction temperature limit, ESD-safe pull-up values, and supply current at 1.65 V in every mode (ds p. 5 lists selected cases).
