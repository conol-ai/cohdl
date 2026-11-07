# AW21009 — 9-channel I²C constant-current LED driver, QFN 3×3-20L

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/led-driver::AW21009",
  "parts": ["@contrib/led-driver::LEDDRV_AW21009"],
  "identities": [
    {"mfr": "Awinic", "mpn": "AW21009QNR", "lcsc": "C5162564"}
  ],
  "sources": [
    {"id": "ds", "title": "AW21009 9 Multi-Function LED Driver with I2C Interface", "revision": "V1.1, July 2021, 34 pages",
     "url": "https://www.awinic.com", "sha256": "a49ae3a166609686a001bf45c0002fef231b7919d4e1a0f357f4cb4428ece6bc", "local": "lib/@contrib/led-driver/docs/aw21009.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices; they equal the printed page numbers on every page (1–34).

The source URL is only the manufacturer's home page. Neither `lib/@contrib/led-driver/docs/README.md` nor the PDF gives a document URL.

## Summary

The AW21009 sinks constant current on nine LED channels (LED1–LED9) and is controlled over I²C at up to 1 MHz (ds p. 1). Each channel has an 8-bit current scale and up to 12-bit PWM; a global 8-bit register and the ISET resistor set full scale, 40 mA per channel with REXT = 4.0 kΩ (ds p. 3, p. 10). It runs from 2.7 V to 5.5 V and has four pin-selected I²C addresses, 20h, 21h, 24h and 25h (ds p. 1, p. 16). EN low shuts the chip down and clears its registers; EN has an internal 1 MΩ pull-down (ds p. 6, p. 9). The LED pins are rated only to VCC, so the LED anode supply must not exceed VCC (ds p. 5; see Pitfalls).

## Key parameters

TA = 25 °C, VCC = 3.6 V, REXT = 4 kΩ, PWMRES = 8 bit unless stated (ds p. 6).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage VCC | 2.7 | — | 5.5 | V | — | ds p. 6 |
| Shutdown current | — | 0.1 | 1 | µA | VEN = GND | ds p. 6 |
| Standby current | — | 3 | 10 | µA | VEN = 3.6 V, CHIPEN = 0 | ds p. 6 |
| Power-save current | — | 3 | 10 | µA | GCR.APSE = 1, all LEDs off > 32 ms | ds p. 6 |
| Active quiescent current | — | 2 | 4 | mA | GCR.CHIPEN = 1 | ds p. 6 |
| Active current | — | 8 | 10 | mA | Iout = 20 mA per LEDx | ds p. 6 |
| LED output leakage | — | 0.1 | 1 | µA | VEN = 0 V, VLEDx = 5.5 V | ds p. 6 |
| Maximum global current per channel IMAX | 38 | 40 | 42 | mA | GCC = 0xFF, BRx = COLx = 0xFF | ds p. 6 |
| Output current match | −5 | — | 5 | % | GCC = 0xFF, SLx = BRx = 0xFF | ds p. 6 |
| Dropout voltage (5 % current drop) | 70 | 100 | 150 | mV | ILEDx = 20 mA | ds p. 6 |
| Dropout voltage | 100 | 120 | 200 | mV | ILEDx = 40 mA | ds p. 6 |
| Dropout voltage | 150 | 180 | 300 | mV | ILEDx = 60 mA | ds p. 6 |
| Oscillator frequency | 14.88 | 16 | 17.12 | MHz | — | ds p. 6 |
| Thermal shutdown threshold | — | 165 | — | °C | Hysteresis 25 °C | ds p. 6 |
| EN input low VIL | — | — | 0.4 | V | — | ds p. 6 |
| EN input high VIH | 1.4 | — | — | V | — | ds p. 6 |
| AD0, AD1 input low VIL | — | — | 0.3 × VCC | V | — | ds p. 6 |
| AD0, AD1 input high VIH | 0.7 × VCC | — | — | V | — | ds p. 6 |
| EN internal pull-down | — | 1 | — | MΩ | — | ds p. 6 |
| SDA output low | — | — | 0.1 | V | IOL = 10 mA | ds p. 6 |
| SCL, SDA input high VIH | 1.2 | — | — | V | — | ds p. 6 |
| SCL, SDA input low VIL | — | — | 0.4 | V | — | ds p. 6 |
| I²C clock fSCL | — | — | 1000 | kHz | Fast-mode Plus | ds p. 7 |
| UVLO threshold | — | 2.5 | — | V | VCC falling, typical | ds p. 15 |
| POR threshold VPOR | — | 2.0 | — | V | — | ds p. 8 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | −0.3 to 6 | V | ds p. 5 |
| Input voltage SCL, SDA, EN, AD0, AD1 | −0.3 to VCC | V | ds p. 5 |
| Output voltage LED1–LED9 | −0.3 to VCC | V | ds p. 5 |
| Junction-to-ambient θJA | 47 | °C/W | ds p. 5 |
| Maximum operating junction temperature | 150 | °C | ds p. 5 |
| Storage temperature | −65 to 150 | °C | ds p. 5 |
| Lead temperature (soldering, 10 s) | 260 | °C | ds p. 5 |
| ESD, HBM | ±2000 | V | ds p. 5 |
| ESD, CDM | ±1500 | V | ds p. 5 |
| Latch-up (JESD78D) | ±200 | mA | ds p. 5 |

## Recommended operating conditions

The datasheet has no recommended-operating-conditions table. The supply range is from the electrical characteristics; the temperature range is listed among the absolute maximum ratings and in the ordering table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 2.7 | 5.5 | V | ds p. 6 |
| I²C interface high level | 1.8 | 3.3 | V | ds p. 16 |
| Operating free-air temperature | −40 | 85 | °C | ds p. 5 |

## Pins

Top view: pins 1–5 down the left side, 6–10 along the bottom, 11–15 up the right side, 16–20 along the top from right to left, pad 21 the centre (ds p. 2, Figure 2).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | NC1 | NC | — | Not connected | ds p. 3 |
| 2 | LED1 | LED1 | current sink | LED cathode | ds p. 3 |
| 3 | LED2 | LED2 | current sink | LED cathode | ds p. 3 |
| 4 | LED3 | LED3 | current sink | LED cathode | ds p. 3 |
| 5 | LED4 | LED4 | current sink | LED cathode | ds p. 3 |
| 6 | LED5 | LED5 | current sink | LED cathode | ds p. 3 |
| 7 | LED6 | LED6 | current sink | LED cathode | ds p. 3 |
| 8 | LED7 | LED7 | current sink | LED cathode | ds p. 3 |
| 9 | LED8 | LED8 | current sink | LED cathode | ds p. 3 |
| 10 | LED9 | LED9 | current sink | LED cathode | ds p. 3 |
| 11 | NC11 | NC | — | Not connected | ds p. 3 |
| 12 | NC12 | NC | — | Not connected | ds p. 3 |
| 13 | NC13 | NC | — | Not connected | ds p. 3 |
| 14 | AD0 | AD0 | input | I²C address, to GND or VCC | ds p. 3, p. 16 |
| 15 | AD1 | AD1 | input | I²C address, to GND or VCC | ds p. 3, p. 16 |
| 16 | VCC | VCC | power | Supply 2.7–5.5 V | ds p. 3 |
| 17 | SDA | SDA | I/O | I²C data | ds p. 3 |
| 18 | SCL | SCL | input | I²C clock | ds p. 3 |
| 19 | EN | EN | input | Chip shuts down when low; 1 MΩ internal pull-down | ds p. 3, p. 6 |
| 20 | ISET | ISET | analog | REXT to GND; 4.0 kΩ gives 40 mA | ds p. 3 |
| 21 | GND | GND (exposed pad) | ground | Ground | ds p. 2, p. 3 |

Checked by hand against Figure 2 (ds p. 2) and the land drawing (ds p. 32): `QFN20N40P300X300_1EP170X170` places pad 1 at the upper left, 1–5 down the left column, 6–10 left to right along the bottom, 11–15 bottom to top on the right and 16–20 right to left along the top, which is the top-view order on both pages.

## Typical application

- Figure 1 / Figure 4 (ds p. 2, p. 4): C1 = 1 µF and C2 = 0.1 µF on VCC; C3 = 1 µF on VLED; REXT = 4 kΩ from ISET to GND; R1 = R2 = 4.7 kΩ I²C pull-ups to VIO (1 kΩ at 1 MHz); AD0 and AD1 tied to GND in the figure.
- Optional series resistors RR, RG, RB (or RLED) only shed heat: R = (VLED − VF − VDROPOUT) / ILED (ds p. 2, p. 4).
- Current: IOUT(x) = IMAX × GCC/255 × SLx/255 × BRx/2^PWMRES, IMAX = 40 mA (ds p. 10). IMAX is set by REXT; the source gives only the single point 4.0 kΩ → 40 mA (ds p. 3).
- I²C address: 7-bit 010 A4 A3 A2 A1; AD0 sets A2:A1, AD1 sets A4:A3. AD1/AD0 = GND/GND 20h, GND/VCC 21h, VCC/GND 24h, VCC/VCC 25h; broadcast address 1Ch (ds p. 16).
- Start-up: after VCC > 2.0 V and EN high the chip loads OTP for about 2 ms; then set CHIPEN = 1 in GCR (20h) and wait about 200 µs before using the outputs (ds p. 8).
- Pull-ups 1 kΩ to 10 kΩ; 4.7 kΩ for 400 kHz, 1 kΩ for 1 MHz (ds p. 16).

## Layout notes

- Land pattern (ds p. 32): 20 lands 0.50 × 0.20 mm at 0.40 mm pitch, 3.20 mm across outer land edges, centre land 1.70 × 1.70 mm. `QFN20N40P300X300_1EP170X170` puts the lands at ±1.35 mm (3.20 − 0.50 = 2.70 mm between centres), 0.40 mm pitch, 1.70 mm square centre land. It matches.
- Package (ds p. 31): body 3.00 ± 0.05 mm square, height 0.75 ± 0.05 mm, terminals 0.20 ± 0.05 × 0.40 ± 0.05 mm, exposed pad 1.70 ± 0.05 mm square.
- Solder mask: NSMD with 0.05 mm maximum clearance, or SMD with 0.05 mm minimum overlap (ds p. 32).
- The source gives no thermal-via or paste-aperture guidance for the exposed pad.

## Pitfalls

- **LED pins are rated −0.3 V to VCC** (ds p. 5). Figure 1 supplies the LEDs from a separate VLED (ds p. 2). If VLED is above VCC, an off channel can be pulled toward VLED through the LED. The source does not say how VLED relates to VCC.
- EN is required: it has a 1 MΩ pull-down, so a floating EN keeps the chip in shutdown, and shutdown resets all registers and blocks I²C (ds p. 6, p. 9).
- Registers cannot be written during the ~2 ms initialization after EN rises or after a software reset (ds p. 8, p. 9).
- UVLO and over-temperature clear CHIPEN; outputs stay off until the host writes CHIPEN = 1 again (ds p. 15).
- Dropout rises with current: up to 300 mV at 60 mA (ds p. 6). Allow VLED ≥ VF + dropout.
- PWM between 500 Hz and 20 kHz can make MLCCs sing; Awinic recommends below 500 Hz or above 20 kHz (ds p. 10).
- I²C high level 1.8–3.3 V only (ds p. 16), and SCL/SDA are rated to VCC (ds p. 5).
- Ambient −40 °C to 85 °C (ds p. 5).

## Not established by the sources

The library's pin numbers, names and footprint agree with the source (ds pp. 2–3, 31–32). Disagreements and gaps:

- **ISET resistor law.** The source gives only REXT = 4.0 kΩ → 40 mA (ds p. 3) and IMAX = 40 mA in the current formula (ds p. 10). It does not give IMAX as a function of REXT, a permitted REXT range or whether a smaller REXT raises IMAX. The library comment "external REXT to GND sets global current" is consistent but should not be read as a formula.
- **LED pin rating.** Nothing in the library warns that LED1–LED9 are rated only to VCC (ds p. 5).
- **Pin types.** LED1–LED9 are constant-current sinks (ds p. 3) typed `output`. SDA is open-drain by I²C convention but the source does not say so. AD0 and AD1 have no stated internal pull, so `required` is right.
- **Exposed pad.** Figure 2 labels pad 21 GND (ds p. 2). The source does not say whether it carries ground current or only heat, or how many vias it needs.
- **No recommended-operating-conditions table** in the source; VCC range from the EC table (ds p. 6).
- Not given: limits over temperature, the I²C pins' tolerance above VCC, θJA test board, and a document URL.
