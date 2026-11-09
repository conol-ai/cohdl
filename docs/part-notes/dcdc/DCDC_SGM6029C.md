# SGM6029CYG/TR — 4 MHz, 1 A buck, resistor-selected 1.8–3.3 V output, WLCSP-0.74×1.09-6B

## Identity

```json
{
  "schema": "part-note/1",
  "device": "dcdc::SGM6029",
  "parts": ["dcdc::DCDC_SGM6029C"],
  "identities": [
    {"mfr": "SGMICRO", "mpn": "SGM6029CYG/TR", "lcsc": "C5153500"}
  ],
  "sources": [
    {"id": "ds", "title": "SGM6029 Ultra-Low Quiescent Current, Synchronous Buck Converter", "revision": "Rev. A.1, November 2022",
     "url": "https://www.sg-micro.com/rect/assets/3323f5dc-718f-4b81-b820-398ddb702989/SGM6029.pdf", "sha256": "79d7e25abc3acfd18ab9c394e082663e07dc169151b4bfe0314c13510ced63e0", "local": "lib/dcdc/docs/sgm6029.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. Pages 1–21 equal the printed page numbers. Pages 22–24 (package outline, tape and reel, carton) carry no printed page number.

## Summary

The SGM6029 is a synchronous buck for 1.95–5.5 V input with 2.3 µA typical quiescent current and an internal feedback divider (ds p. 1, p. 3). One resistor on the VSEL/MODE pin, read once at start-up, selects one of 16 output voltages; afterwards the same pin chooses power-save or forced-PWM mode (ds p. 1, p. 15). The library part is the C version: 4.0 MHz, 1.8–3.3 V in 100 mV steps, 1.8 V with VSEL/MODE grounded (ds p. 2). Choose it for a tiny low-IQ rail from a single Li-ion cell or a 3.3–5 V bus. It delivers 1 A only at VIN ≥ 2.3 V (0.7 A below, ds p. 3). It has no thermal protection in power-save mode (ds p. 16). The 0.35 mm-pitch WLCSP needs a fine-pitch PCB process (ds p. 22).

## Key parameters

VIN = 3.6 V, TJ = -40 °C to +125 °C, typical at TJ = +25 °C, unless stated (ds p. 5).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output voltage range, SGM6029C | 1.8 | — | 3.3 | V | 100 mV steps | ds p. 5 |
| Output voltage accuracy | -2 | 0 | 2 | % | PWM mode, IOUT = 0 mA | ds p. 5 |
| Maximum output current | — | — | 1 | A | SGM6029A/B/C, VIN ≥ 2.3 V | ds p. 3 |
| Maximum output current | — | — | 0.7 | A | SGM6029A/B/C, VIN < 2.3 V | ds p. 3 |
| Quiescent current, power-save, not switching | — | 2.3 | 5.5 | µA | VEN = VIN, IOUT = 0 µA, VOUT = 1.2 V | ds p. 5 |
| Quiescent current, power-save, switching | — | 2.8 | — | µA | VEN = VIN, IOUT = 0 µA, VOUT = 1.2 V | ds p. 5 |
| Quiescent current, PWM mode | — | 8.5 | — | mA | VSEL/MODE = VIN after power-up, switching | ds p. 5 |
| Shutdown current ISD | — | 120 | 250 | nA | EN = GND, VSEL/MODE = GND, TJ = -40 to +85 °C | ds p. 5 |
| UVLO threshold VUVLO | — | 1.76 | 1.95 | V | VIN rising | ds p. 5 |
| EN high-level input voltage VIH | 0.85 | — | — | V | — | ds p. 5 |
| EN low-level input voltage VIL | — | — | 0.40 | V | — | ds p. 5 |
| EN internal pull-down RPD | — | 570 | — | kΩ | EN low; connected only during start-up and while disabled | ds p. 4, p. 5 |
| EN input bias current | — | 10 | 100 | nA | EN high, TJ = -40 to +85 °C | ds p. 5 |
| VSEL/MODE high-level input voltage VIH | 0.89 | — | — | V | Digital input | ds p. 5 |
| VSEL/MODE low-level input voltage VIL | — | — | 0.40 | V | Digital input | ds p. 5 |
| High-side / low-side RDSON | — | 185 / 125 | 310 / 230 | mΩ | IOUT = 100 mA | ds p. 5 |
| High-side switch current limit ILIMF | 1.58 | 1.78 | 1.98 | A | TJ = +25 °C | ds p. 5 |
| Low-side switch current limit | 0.90 | 1.20 | 1.50 | A | TJ = +25 °C | ds p. 5 |
| Discharge MOSFET on-resistance RDSCH_SW | — | 4 | 11 | Ω | EN = GND, ISW = -10 mA into SW, TJ = -40 to +85 °C | ds p. 5 |
| VOS bias current | — | 250 | 450 | nA | EN = VIN, VOUT = 1.2 V, TJ = -40 to +85 °C | ds p. 5 |
| Switching frequency, SGM6029A/B/C | — | 4 | — | MHz | — | ds p. 6 |
| Start-up delay | — | 700 | 1170 | µs | EN low-to-high until switching, VSEL = 16 | ds p. 6 |
| Soft-start time, SGM6029C | — | 410 | 785 | µs | VOUT from 0 V (see "Not established") | ds p. 6 |
| Thermal shutdown / hysteresis | — | 160 / 20 | — | °C | Rising, PWM mode only | ds p. 5, p. 16 |
| θJA, WLCSP-0.74×1.09-6B | — | 214 | — | °C/W | Board not stated | ds p. 3 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VIN | -0.3 to 6 | V | ds p. 3 |
| SW | -0.3 to VIN + 0.3 | V | ds p. 3 |
| SW, AC, less than 10 ns while switching | -1 to 8 | V | ds p. 3 |
| EN, VSEL/MODE | -0.3 to VIN + 0.3 | V | ds p. 3 |
| VOS | -0.3 to 5 | V | ds p. 3 |
| Junction temperature | +150 | °C | ds p. 3 |
| Storage temperature | -65 to +150 | °C | ds p. 3 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 3 |
| ESD, HBM | 4000 | V | ds p. 3 |
| ESD, CDM | 1000 | V | ds p. 3 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Input voltage VIN | 1.95 | 5.5 | V | ds p. 3 |
| Effective inductance L, SGM6029A/B/C (0.47 µH typical) | 0.33 | 1 | µH | ds p. 3 |
| Effective output capacitance COUT, SGM6029A/B/C | 5 | 10 | µF | ds p. 3 |
| Effective input capacitance CIN (printed "2.2µF to 4.7µF (TYP)") | 2.2 | 4.7 | µF | ds p. 3 |
| Parasitic capacitance at VSEL/MODE | — | 30 | pF | ds p. 3 |
| RVSEL range (E96, 1%) | 10 | 249 | kΩ | ds p. 3 |
| RVSEL tolerance | — | 1 | % | ds p. 3 |
| RVSEL temperature coefficient | -200 | +200 | ppm/°C | ds p. 3 |
| Operating junction temperature | -40 | +125 | °C | ds p. 3 |
| Specified temperature range (ordering table) | -40 | +85 | °C | ds p. 2 |

## Pins

Top view: column 1 on the left, column 2 on the right; row A at the top with the A1 dot at the upper-left (ds p. 4). The bottom view is mirrored (column 2 left, ds p. 22).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | GND | GND | ground | Ground; return CIN and COUT close to this pin | ds p. 4 |
| A2 | VOS | VOS | input | Output sense for the internal feedback and the output discharge; short trace to COUT | ds p. 4 |
| B1 | VIN | VIN | power input | Supply input; ceramic CIN close to this pin and GND | ds p. 4 |
| B2 | SW | SW | power output | Switching node; to the inductor | ds p. 4 |
| C1 | VSEL_MODE | VSEL/MODE | input | Start-up: RVSEL to GND selects VOUT. After start-up: high = FPWM, low = power-save | ds p. 4, p. 15 |
| C2 | EN | EN | input | Active-high enable; 570 kΩ pull-down only during start-up | ds p. 4 |

## Typical application

- Output voltage for the C version, RVSEL from VSEL/MODE to GND (E96, 1%, TC ≤ ±200 ppm/°C): short = 1.8 V, 10.0 kΩ = 1.8 V, 12.1 kΩ = 1.9 V, 15.4 kΩ = 2.0 V, 18.7 kΩ = 2.1 V, 23.7 kΩ = 2.2 V, 28.7 kΩ = 2.3 V, 36.5 kΩ = 2.4 V, 44.2 kΩ = 2.5 V, 56.2 kΩ = 2.6 V, 68.1 kΩ = 2.7 V, 86.6 kΩ = 2.8 V, 105 kΩ = 2.9 V, 133 kΩ = 3.0 V, 162 kΩ = 3.1 V, 205 kΩ = 3.2 V, 249 kΩ or larger = 3.3 V (ds p. 6). Figure 7 sets 3.3 V with 249 kΩ (ds p. 19).
- CIN = 4.7 µF ceramic close to VIN and GND, e.g. GRT155R61A475ME13D (4.7 µF, X5R, 10 V, 0402) (ds p. 17, p. 18). Use ≥ 10 µF from a high-impedance source such as a coin cell (ds p. 18).
- COUT = 10 µF, e.g. GRM155R61A106ME44D (10 µF, X5R, 10 V, 0402) (ds p. 17, p. 18). The recommended effective range is 5–10 µF (ds p. 3), and ceramic capacitance can drop by up to 50% under bias (ds p. 18).
- L = 0.47 µH or 1 µH (ds p. 18), e.g. Murata DFE18SANR47MG0L: 0.47 µH, 54 mΩ DCR max, 2.6 A Imax, 3.3 A Isat (ds p. 17). Choose an inductor whose saturation current is at least the high-side current limit ILIMF (ds p. 17).
- Ripple: ΔIL = VOUT × (1 − VOUT / VIN) / (L × fSW), and IL_MAX = IOUT_MAX + ΔIL / 2 (ds p. 17).
- VOS: connect to the output capacitor with a short trace (ds p. 4). There is no external divider.
- EN: drive it high to enable (ds p. 4); the typical circuits show an ON/OFF switch (ds p. 1). It is high above 0.85 V and low below 0.40 V (ds p. 5). Tying it to VIN stays within its VIN + 0.3 V rating (ds p. 3).
- VSEL/MODE after start-up: logic high = FPWM, low = power-save (ds p. 15). With only RVSEL to GND the pin is low after start-up, so the part runs in power-save mode (inferred from ds p. 15).

## Layout notes

- Recommended land: six round lands, Φ0.17–0.19 mm, on a 0.35 mm × 0.35 mm grid (ds p. 22). Library `BGA6C35P2X3_75X110X44N` uses Φ0.19 mm lands at x = ±0.175 mm and y = 0, ±0.35 mm, with A1 at the upper-left, which matches the top view (ds p. 4, p. 22).
- Package: D = 0.715–0.775 mm (along a row), E = 1.065–1.125 mm, A = 0.362–0.438 mm, ball d = 0.187–0.247 mm, pitch 0.350 mm BSC (ds p. 22).
- Place CIN as close as possible with the shortest traces (ds p. 21).
- Return CIN and COUT to one common GND point close to the GND pin; place the inductor close to SW with a short trace (ds p. 21).
- Keep the VOS sense trace away from SW and other noisy nodes (ds p. 21). Figure 11 shows the recommended placement (ds p. 21).
- Keep the VSEL/MODE node small: parasitic capacitance above 30 pF can give a wrong VOUT (ds p. 3, p. 15).

## Pitfalls

- **VSEL/MODE is read once.** RVSEL is read only during start-up, after enable or recovery from UVLO (ds p. 14, p. 15). Changing it later does nothing until the next enable. Any leakage or more than 30 pF on the pin during detection can select the wrong voltage (ds p. 15). A logic driver on this pin must not disturb RVSEL during start-up.
- **The C version's grounded default is 1.8 V, not 3.3 V.** Shorting VSEL/MODE to GND gives 1.8 V; 3.3 V needs 249 kΩ or larger (ds p. 6, p. 15).
- **EN must be driven.** The 570 kΩ pull-down is removed once the part has started (ds p. 4, p. 14). A floating EN is then undefined.
- **No thermal protection in power-save mode** (ds p. 16). Thermal shutdown at 160 °C acts only in PWM mode (ds p. 5).
- **Current derating.** 1 A needs VIN ≥ 2.3 V; below that the rating is 0.7 A (ds p. 3).
- **Discharge only after the first enable.** The 4 Ω output discharge works only after the device has been enabled once since power was applied, and only while VIN stays above the UVLO falling threshold (ds p. 15).
- **100% duty cycle.** When VIN falls close to VOUT, the high-side switch stays on, so VOUT follows VIN downward (ds p. 16).
- SW, EN and VSEL/MODE are limited to VIN + 0.3 V (ds p. 3). Do not drive EN or MODE from a rail that is up before VIN.

## Not established by the sources

- **Library pin name.** The library uses `VSEL_MODE` for the datasheet's VSEL/MODE (ds p. 4); this is only the character set. The pin map matches ds p. 4 pad by pad.
- **Open VSEL/MODE.** Table 1 says "249.0kΩ or larger" gives 3.3 V (ds p. 6), but the source does not say what an open pin selects.
- **Soft-start wording.** tSS is specified "VOUT rising from 0V to 0.95% × VOUT nominal" (ds p. 6), which reads as a typo for 95%. The device table gives 400 µs for the C version (ds p. 2), the electrical table 410 µs typical (ds p. 6).
- **Power-save accuracy.** The accuracy row prints "0.5" for power-save mode without a clear column (ds p. 5). Only the PWM figure (±2%) is used here.
- **UVLO at minimum VIN.** The UVLO rising threshold maximum (1.95 V) equals the minimum recommended VIN (ds p. 3, p. 5). Start-up at exactly 1.95 V is not guaranteed.
- **CIN range.** The recommended conditions print "2.2µF to 4.7µF (TYP)" (ds p. 3); whether 4.7 µF is a maximum or a typical value is not clear. Elsewhere the source says there is no upper limit on CIN (ds p. 18).
- **θJA board.** The 214 °C/W θJA has no stated test board (ds p. 3).
- **Land details.** Solder-mask definition (SMD or NSMD), stencil and via-in-pad guidance are not given (ds p. 22). Rev. A.1 changed the package outline dimensions (ds p. 21).
- **Product status of other versions.** SGM6029A, D and E are marked "Product Preview" (ds p. 2). The library part, C, is not.
- **Not given:** limits outside -40 °C to +85 °C for several rows (ds p. 5), load and line regulation, PSRR, maximum EN or MODE input current at high temperature, and moisture sensitivity level.
