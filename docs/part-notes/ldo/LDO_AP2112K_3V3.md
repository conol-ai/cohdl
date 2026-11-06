# AP2112K-3.3TRG1 — 3.3 V, 600 mA CMOS LDO with enable, SOT25

## Identity

```json
{
  "schema": "part-note/1",
  "device": "ldo::AP2112K_3V3",
  "parts": ["ldo::LDO_AP2112K_3V3"],
  "identities": [
    {"mfr": "Diodes Incorporated", "mpn": "AP2112K-3.3TRG1", "lcsc": "C51118"}
  ],
  "sources": [
    {"id": "ds", "title": "AP2112 600mA CMOS LDO Regulator with Enable", "revision": "DS39724 Rev. 2-2, June 2017",
     "url": "https://www.diodes.com/datasheet/download/AP2112.pdf", "sha256": "ef8d376f2ec356e29172eb9e053819a0ebdcc576dba7fc9ab0505c568427920f", "local": "lib/ldo/docs/diodes-ap2112.pdf"}
  ],
  "status": "draft"
}
```

## Summary

Fixed 3.3 V CMOS low-dropout regulator in SOT25 (formerly SOT-23-5) with an active-high enable,
600 mA minimum rated output current and 60 Ω output auto-discharge when disabled (ds p. 1).
Choose it for a small 3.3 V rail from a 5 V (or ≥ 3.7 V worst-case) supply needing low quiescent
current (55 µA typ) and only 1 µF ceramic caps (ds p. 1, p. 8). The input is limited to 6.0 V
recommended / 6.5 V absolute (ds p. 3), and SOT25 θJA is 184 °C/W (ds p. 3), so it is wrong for
any rail above 6 V and for high-headroom, high-current loads.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output voltage | VOUT×98.5% | 3.3 | VOUT×101.5% | V | VIN = 4.3 V, 1 mA ≤ IOUT ≤ 30 mA | ds p. 8 |
| Maximum output current | 600 | — | — | mA | VIN = 4.3 V, VOUT = 3.251 V to 3.350 V | ds p. 8 |
| Load regulation | -1 | 0.2 | 1 | %/A | VIN = 4.3 V, 1 mA ≤ IOUT ≤ 600 mA | ds p. 8 |
| Line regulation | -0.1 | 0.02 | 0.1 | %/V | 4.3 V ≤ VIN ≤ 6 V, IOUT = 30 mA | ds p. 8 |
| Dropout voltage | — | 5 | 8 | mV | IOUT = 10 mA | ds p. 8 |
| Dropout voltage | — | 125 | 200 | mV | IOUT = 300 mA | ds p. 8 |
| Dropout voltage | — | 250 | 400 | mV | IOUT = 600 mA | ds p. 8 |
| Quiescent current | — | 55 | 80 | µA | VIN = 4.3 V, IOUT = 0 mA | ds p. 8 |
| Standby current | — | 0.01 | 1.0 | µA | VIN = 4.3 V, EN in OFF mode | ds p. 8 |
| PSRR | — | 65 | — | dB | VIN = 4.3 V, IOUT = 100 mA, ripple 0.5 Vp-p, f = 100 Hz and 1 kHz | ds p. 8 |
| Output voltage temperature coefficient | — | ±100 | — | ppm/°C | IOUT = 30 mA, TA = -40 °C to +85 °C | ds p. 8 |
| Short-circuit (foldback) current | — | 50 | — | mA | VOUT = 0 V | ds p. 8 |
| RMS output noise | — | 50 | — | µVRMS | No load, 10 Hz ≤ f ≤ 100 kHz | ds p. 8 |
| EN high voltage (regulator on) | 1.5 | — | 6.0 | V | Enable logic high | ds p. 8 |
| EN low voltage (regulator off) | 0 | — | 0.4 | V | Enable logic low | ds p. 8 |
| Start-up time | — | 20 | — | µs | No load | ds p. 8 |
| EN pull-down resistor | — | 3.0 | — | MΩ | — | ds p. 8 |
| VOUT discharge resistor | — | 60 | — | Ω | EN low | ds p. 8 |
| Thermal shutdown temperature | — | +160 | — | °C | — | ds p. 8 |
| Thermal shutdown hysteresis | — | +25 | — | °C | — | ds p. 8 |
| θJC, SOT25 | — | 96 | — | °C/W | Junction to case | ds p. 8 |

All p. 8 values: CIN = COUT = 1.0 µF ceramic, typical at TA = +25 °C; production tested at +25 °C,
over-temperature limits guaranteed by design only (ds p. 8, Note 6).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Power supply voltage VCC | 6.5 | V | ds p. 3 |
| Operating junction temperature | +150 | °C | ds p. 3 |
| Storage temperature | -65 to +150 | °C | ds p. 3 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 3 |
| θJA, SOT25, no heatsink | 184 | °C/W | ds p. 3 |
| ESD, machine model | 400 | V | ds p. 3 |
| ESD, human body model | 4000 | V | ds p. 3 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VIN (whole AP2112 family) | 2.5 | 6.0 | V | ds p. 3 |
| Ambient temperature TA | -40 | +85 | °C | ds p. 3 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VIN | VIN | Power input | Input voltage | ds p. 1, p. 2 |
| 2 | GND | GND | Ground | Ground | ds p. 1, p. 2 |
| 3 | EN | EN | Input | Chip enable: H = normal work, L = shutdown output (3 MΩ internal pull-down) | ds p. 2, p. 8 |
| 4 | NC | NC | — | No connection | ds p. 1 |
| 5 | VOUT | VOUT | Power output | Output voltage | ds p. 1, p. 2 |

## Typical application

- CIN = 1 µF from VIN to GND and COUT = 1 µF from VOUT to GND (ds p. 2); if 1.0 µF ceramics are
  used, X7R or X5R dielectric is recommended (ds p. 2, Note 4). Stable with 1.0 µF ceramic,
  tantalum or aluminium electrolytic (ds p. 1).
- EN: tie to VIN for always-on (VIH max 6.0 V equals the VIN max, ds p. 3, p. 8), or drive from
  logic: ≥ 1.5 V on, ≤ 0.4 V off (ds p. 8). The 3 MΩ pull-down makes a floating EN default to
  off (ds p. 2, p. 8); never leave EN unconnected when the rail must come up.
- Pad 4 (NC): leave unconnected (ds p. 1).
- Headroom: VIN ≥ 3.3 V + dropout, i.e. ≥ 3.7 V worst case at 600 mA, ≥ 3.5 V at 300 mA (ds p. 8).
  Electrical limits are specified at VIN = 4.3 V (ds p. 8); a 5 V rail is the natural source.
- Thermal estimate: P ≈ (VIN − 3.3 V) × IOUT + VIN × IQ; ΔTJ ≈ P × 184 °C/W (ds p. 3). Example:
  5 V in, 300 mA → 0.51 W → ≈ 94 °C rise; at TA = 85 °C the 150 °C TJ limit allows only ≈ 0.35 W
  (≈ 200 mA from 5 V). The 600 mA rating is not a continuous rating from 5 V in SOT25.

## Layout notes

- Use the SOT25 suggested land: X = 0.55 mm, Y = 0.80 mm, C2 = 0.95 mm pitch, C1 = 2.40 mm row
  centres, G = 1.60 mm, Z = 3.20 mm (ds p. 14); library footprint `SOT5P95X290X160N` follows it,
  rotated 90°.
- Place CIN and COUT next to pads 1 and 5 with a short return to pad 2 (reference circuit, ds p. 2).
- The datasheet gives no thermal-pad or copper-area guidance; θJA 184 °C/W is "no heatsink"
  (ds p. 3). Add copper on VIN/VOUT/GND if dissipation is significant.

## Pitfalls

- Never connect VIN to a 9/12/15/20 V (USB-PD) or any rail above 6.0 V; absolute max is 6.5 V
  (ds p. 3). Use a buck or a higher-voltage LDO upstream.
- A single Li-ion cell (≤ 3.7 V) drops out under load; 3.3 V needs up to 400 mV headroom at
  600 mA (ds p. 8). The 2.5 V VIN minimum on p. 3 is for the family, not for 3.3 V regulation.
- Floating EN = off (3 MΩ pull-down, ds p. 2, p. 8); EN above 6.0 V is outside spec (ds p. 8).
- Foldback current limit (50 mA short current, ds p. 8): a load or current source present before
  enable may stop the part from starting; evaluate multi-supply designs (ds p. 11).
- Auto-discharge (60 Ω when EN low, ds p. 8) pulls VOUT down; it is not reverse-current blocking.
- Do not pick a 1 µF ceramic of other dielectric than X5R/X7R (ds p. 2, Note 4); check its effective capacitance at 3.3 V/5 V bias (not given in the source).

## Not established by the sources

- Reverse-current behaviour (VOUT above VIN) and any reverse-blocking guarantee.
- Minimum or maximum COUT ESR range and any upper capacitance limit; inrush/soft-start current.
- Maximum EN pin current, or EN voltage above VIN behaviour; EN thresholds over temperature.
- θJA on a specific PCB copper area; θJA test board is not described (ds p. 3).
- Dropout and output accuracy over the full -40 °C to +85 °C range as guaranteed limits
  (over-temperature specs are by design only, ds p. 8).
- Pad 4 confirmation within the source: the p. 2 pin table gives "—" as the SOT25 NC pin number; only the p. 1 drawing shows pin 4 = NC.
