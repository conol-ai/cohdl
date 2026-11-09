# RT6150B-33GQW — fixed 3.3 V, 1 MHz buck-boost converter, WDFN-10L 2.5×2.5

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@richtek/dcdc::RT6150B",
  "parts": ["@richtek/dcdc::BUCKBOOST_RT6150B"],
  "identities": [
    {"mfr": "Richtek", "mpn": "RT6150B-33GQW", "lcsc": "C3187669"}
  ],
  "sources": [
    {"id": "ds", "title": "RT6150A/B Current Mode Buck-Boost Converter", "revision": "DS6150A/B-06, July 2018",
     "url": "https://www.richtek.com/assets/product_file/RT6150A%3DRT6150B/DS6150AB-06.pdf", "sha256": "35fcd01e0c71fc4249341b3704bb3ce4a032d13d2f78db494e34238d75bc8b93", "local": "lib/@richtek/dcdc/docs/rt6150a-rt6150b-datasheet.pdf"},
    {"id": "fp", "title": "Footprints drawing V/W/U/XDFN2.5x2.5-10", "revision": "No revision printed; package README: PDF modified 2022-12-28",
     "url": "https://www.richtek.com/assets/podfiles/Footprint-%28V%29%28W%29%28U%29XDFN2.5x2.5-10-R-B.pdf", "sha256": "94f3723956f11dd35bacf2ae70226d2726e485cc37a163560ea2dc5688431a10", "local": "lib/@richtek/dcdc/docs/wdfn2.5x2.5-10-footprint.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. In `ds` they equal the printed page numbers (1–13). `fp` is a single page.

## Summary

The RT6150B is a synchronous buck-boost converter with four integrated switches, 1.8–5.5 V input, 1 MHz fixed frequency and up to 800 mA continuous output (ds p. 1). It regulates when the input is above, below or equal to the output (ds p. 3). RT6150B-33GQW is the fixed 3.3 V option in WDFN-10L 2.5×2.5; its FB pin must be tied to VOUT (ds p. 1, p. 2). Choose it for a 3.3 V rail from a single Li-ion cell or 2–3 alkaline cells, where the input crosses 3.3 V (ds p. 9). The output is disconnected from the input in shutdown (ds p. 1). The 800 mA is a headline figure: the available output current falls at low input voltage (typical curve only, ds p. 7).

## Key parameters

VIN = VOUT = 3.6 V, TA = 25 °C unless stated (ds p. 4).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Continuous output current | — | — | 800 | mA | "Up to" (Features) | ds p. 1 |
| Input UVLO, high level (rising) | — | 1.65 | 1.8 | V | Sensed on VINA | ds p. 4, p. 9 |
| Input UVLO, low level (falling) | 1.4 | 1.55 | — | V | Sensed on VINA | ds p. 4, p. 9 |
| Feedback voltage VFB | 0.495 | 0.5 | 0.505 | V | VPS = VIN (adjustable versions) | ds p. 4 |
| Feedback input current | — | 1 | 50 | nA | VFB = 0.5 V | ds p. 4 |
| Quiescent current, power-save mode | — | 60 | — | µA | IOUT = 0 mA, PS = 0 V, not switching | ds p. 4, p. 5 |
| Shutdown current | — | 0.1 | 1 | µA | EN = 0 V, excluding switch leakage | ds p. 4 |
| N-MOSFET / P-MOSFET switch leakage | — | 0.1 | 5 / 10 | µA | — | ds p. 4 |
| N-MOSFET / P-MOSFET RDS(ON) | — | 0.15 | — | Ω | — | ds p. 4 |
| Switch current limit ILIM | 1.6 | — | — | A | VIN = 3.6 V | ds p. 4 |
| Oscillator frequency | 0.8 | 1 | 1.2 | MHz | — | ds p. 4 |
| Soft-start time | — | 0.65 | 1 | ms | EN asserted to output in regulation, IOUT = 0 mA | ds p. 4 |
| EN and PS logic-high voltage | 1.2 | — | — | V | — | ds p. 5 |
| EN and PS logic-low voltage | — | — | 0.4 | V | — | ds p. 5 |
| EN and PS input current | — | 0.01 | 1 | µA | VEN = VPS = VIN | ds p. 5 |
| Thermal shutdown | — | 140 | — | °C | — | ds p. 5 |
| θJA / θJC, WDFN-10L 2.5×2.5 | — | 40.9 / 18.6 | — | °C/W | JEDEC 51-7 four-layer board; θJC at the exposed pad | ds p. 4, p. 5 |
| Maximum power dissipation, WDFN-10L 2.5×2.5 | — | — | 2.44 | W | TA = 25 °C, TJ(MAX) = 125 °C | ds p. 4, p. 11 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VOUT, VIN, EN, PS, VINA, FB | -0.3 to 6 | V | ds p. 4 |
| LX1, LX2 | -0.3 to 6 | V | ds p. 4 |
| LX1, LX2, < 10 ns | -2 to 7.5 | V | ds p. 4 |
| Power dissipation, WDFN-10 2.5×2.5, TA = 25 °C | 2.44 | W | ds p. 4 |
| Junction temperature | 150 | °C | ds p. 4 |
| Storage temperature | -65 to 150 | °C | ds p. 4 |
| Lead temperature (soldering, 10 s) | 260 | °C | ds p. 4 |
| ESD, HBM | 2 | kV | ds p. 4 |
| ESD, MM | 200 | V | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply input voltage VIN | 1.8 | 5.5 | V | ds p. 4 |
| Junction temperature | -40 | 125 | °C | ds p. 4 |
| Ambient temperature | -40 | 85 | °C | ds p. 4 |

## Pins

Top view: pins 1–5 (VOUT, LX2, GND, LX1, VIN) down the left, pins 6–10 (EN, PS, VINA, GND, FB) up the right, exposed pad 11 in the centre (ds p. 2). The same pinout covers WDFN-10L 3x3 (RT6150A) and 2.5x2.5 (RT6150B) (ds p. 2).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VOUT | VOUT | power output | Converter output; capacitor to GND | ds p. 2 |
| 2 | LX2 | LX2 | passive | Second switch node; to the inductor | ds p. 2 |
| 3 | GND | GND | ground | Power ground | ds p. 2 |
| 4 | LX1 | LX1 | passive | First switch node; to the inductor | ds p. 2 |
| 5 | VIN | VIN | power input | Power input; ≥ 10 µF to GND | ds p. 2 |
| 6 | EN | EN | input | Enable; high = on, low = shutdown | ds p. 2, p. 9 |
| 7 | PS | PS | input | Low = power-save mode, high = fixed-frequency PWM | ds p. 2, p. 9 |
| 8 | VIN | VINA | power input | Control-circuit supply; UVLO is sensed here. Shares the library pin `VIN` with pad 5 | ds p. 2, p. 6, p. 9 |
| 9 | GND | GND | ground | Power ground | ds p. 2 |
| 10 | FB | FB | input | Feedback; on the fixed version connect directly to VOUT | ds p. 2, p. 9 |
| 11 | GND | GND (exposed pad) | ground | Solder to a large GND area for heat | ds p. 2 |

## Typical application

- FB: connect directly to VOUT for RT6150B-33 (ds p. 2, p. 9). The 487 kΩ / 86.6 kΩ divider in the typical circuit (ds p. 6) is for the adjustable versions, where R1 = R2 × (VOUT / VFB − 1) and R2 = 80–500 kΩ (ds p. 9).
- CIN ≥ 10 µF ceramic, as close as possible to VIN and GND (ds p. 2, p. 6, p. 10).
- COUT: the typical circuit uses 20 µF (ds p. 6). The text recommends at least 10 µF X7R in 1206, as close as possible to VOUT and GND (ds p. 10).
- L between LX1 and LX2: 2.2 µH for 3.3 V from a 1.8–4.2 V Li-ion range; the recommended range is 1.5–4.7 µH (ds p. 6, p. 10). Minimum L is the larger of L1 = VOUT × (VIN(MAX) − VOUT) / (f × ΔIL × VIN(MAX)) for buck mode and L2 = VIN(MIN) × (VOUT − VIN(MIN)) / (f × ΔIL × VOUT) for boost mode, with f the minimum switching frequency (ds p. 10).
- VINA (pad 8): tied to VIN (pad 5) in the typical circuit (ds p. 6).
- EN: drive above 1.2 V to enable, below 0.4 V to shut down (ds p. 5). Soft-start completes within 1 ms (ds p. 4, p. 9).
- PS: tie low for power-save mode at light load (60 µA IQ) or high for fixed-frequency PWM (ds p. 2, p. 4, p. 9).
- Thermal: PD(MAX) = (125 °C − TA) / 40.9 °C/W for WDFN-10L 2.5×2.5 (ds p. 10, p. 11).

## Layout notes

- Recommended land (fp p. 1): pitch P = 0.50 mm, overall span A = 3.30 mm, inner gap B = 1.60 mm, pad length C = 0.85 mm, pad width D = 0.30 mm, exposed pad Sx = 2.00 mm (along the pin rows) by Sy = 1.20 mm, pad row length M = 2.30 mm, ±0.05 mm.
- Library `qfn::QFN10N50P250X250_1EP120X200` uses 0.85 × 0.30 mm pads at x = ±1.225 mm, y = 0, ±0.5, ±1.0 mm, and a 1.2 × 2.0 mm exposed pad. That is the fp p. 1 land rotated 90°, so the pin rows run vertically as on ds p. 2; the exposed pad's 2.00 mm side runs along the rows as in fp p. 1.
- Package: D = E = 2.40–2.60 mm, exposed pad D2 = 1.95–2.05 mm by E2 = 1.15–1.25 mm, lead width b = 0.20–0.30 mm, lead length L = 0.35–0.45 mm, height A = 0.70–0.80 mm (ds p. 13).
- Place CIN and COUT as close as possible to the input and output pins (ds p. 11).
- Keep the main power traces wide and short, and the LX-to-inductor node area small (ds p. 11).
- Place the feedback components close to FB, away from noise. Connect the output capacitors directly to the feedback network (ds p. 11).
- Connect GND and the exposed pad to a strong ground plane (ds p. 11).

## Pitfalls

- **Do not add a divider on the fixed part.** RT6150B-33 needs FB tied straight to VOUT (ds p. 2, p. 9). The divider on ds p. 6 belongs to the adjustable versions.
- **Absolute maximum is 6 V on every pin** (ds p. 4). Do not use it on 5 V USB rails with overshoot without margin; the recommended maximum input is 5.5 V (ds p. 4).
- **Start-up at 1.8 V is not guaranteed.** The UVLO rising threshold can be as high as 1.8 V (ds p. 4), which equals the minimum recommended VIN.
- **Output current depends on VIN.** 800 mA is "up to" (ds p. 1); the maximum output current against input voltage is given only as a typical curve (ds p. 7).
- **Output disconnect.** In shutdown the load is disconnected from the input and VOUT may fall below VIN (ds p. 9). Do not count on VIN back-powering the load through the switches.
- The source states no internal pull on EN or PS (input current 1 µA max, ds p. 5). Drive both or tie them to a defined level.

## Not established by the sources

- **VINA merged into VIN.** The library declares pads 5 and 8 as one pin `VIN`, with the comment that VINA "must be tied to pin 5 VIN". The source describes VINA as the control-circuit supply input (ds p. 2) and ties it to VIN in the typical circuit (ds p. 6), but it does not say "must". The library cannot express a separate filter on VINA.
- **EN comment.** The library comment on EN reads "3V3 enable". The source gives EN thresholds of 1.2 V high and 0.4 V low (ds p. 5), not a 3.3 V level.
- **Fixed-output accuracy.** The source specifies VFB (0.495–0.505 V) for the adjustable versions (ds p. 4) but no output-voltage tolerance for the fixed 3.3 V version.
- **Footprint comment.** `lib/qfn/src/footprints.cohdl` cites `DS6150AB-04.pdf` for this land. The shipped datasheet is DS6150A/B-06, and the land values come from `fp` p. 1, which match the library.
- **Pin 1 on the footprint drawing.** fp p. 1 shows no pin numbers or pin-1 mark. Pin 1 is located from the pin configuration (ds p. 2) and the package's pin-1 options (ds p. 12, p. 13).
- **COUT minimum.** The typical circuit uses 20 µF (ds p. 6); the text asks for at least 10 µF X7R 1206 (ds p. 10). `examples/sf32-miniboard/src/display_connector.cohdl` states that the part "requires … 20uF at its output", which is more than ds p. 10 asks.
- **Example use of the adjustable version.** `examples/sf32-miniboard` instantiates this device as `RT6150BGQW` (adjustable, ds p. 2) with its own divider. That MPN is not an entry of the library part and is outside this note.
- **Not given:** limits over temperature (25 °C only, ds p. 4), output voltage accuracy and load regulation in power-save mode, inductor saturation current, thermal shutdown hysteresis, current-limit behaviour in boost mode, and moisture sensitivity level.
