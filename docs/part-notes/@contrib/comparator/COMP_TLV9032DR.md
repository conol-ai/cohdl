# TLV9032DR — dual precision comparator, push-pull outputs, fault-tolerant inputs, SOIC-8

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/comparator::TLV9032_D8",
  "parts": ["@contrib/comparator::COMP_TLV9032DR"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TLV9032DR", "lcsc": "C2862758"}
  ],
  "sources": [
    {"id": "ds", "title": "TLV902x and TLV903x Precision Comparator Family", "revision": "SNOSDA3H, June 2020, revised November 2025; package option addendum dated 16-May-2026; D0008A drawing 4214825/C 02/2019",
     "url": "https://www.ti.com/lit/ds/symlink/tlv9022.pdf", "sha256": "874c1d4f5ec1c1b4bf6c714d3fe8623d57ad6bff18b83930e3894061a0701c8a", "local": "lib/@contrib/comparator/docs/ti-tlv90x2-snosda3h.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal TI's printed page numbers on pp. 1–37. The package option addendum (pp. 38–41, "Addendum-Page 1–4"), the packaging materials pages (pp. 42–45) and the mechanical drawings (D0008A on pp. 68–70) carry no data-sheet page numbers. The URL is the one recorded in `lib/@contrib/comparator/docs/tlv90x2-application.md`. It is the TLV9022 product link; the same family datasheet covers the TLV9032 (ds p. 1).

The datasheet covers the whole TLV902x/TLV903x family. Only the TLV90x2 (dual) tables, and the push-pull (TLV903x) rows of them, apply to this part. The open-drain sibling TLV9022DR has its own note, `COMP_TLV9022DR.md`.

## Summary

The TLV9032 is a dual comparator with push-pull outputs that source and sink current. It runs from 1.65 V to 5.5 V, draws 16 µA typical per comparator, and has typical propagation delays of 100 ns falling and 115 ns rising (ds p. 1, p. 7, p. 11, p. 12). Its inputs are rail-to-rail, go 200 mV beyond both rails, and are fault tolerant: they keep their high impedance between 0 V and 5.5 V even while the supply is off or ramping (ds p. 22). Choose it to drive logic, LEDs or MOSFET gates without a pull-up, or where edge symmetry matters, e.g. oscillators (ds p. 1, p. 22, p. 28). Do not use it where outputs must be wired together or pulled to a different rail than V+: use the open-drain TLV9022 then (ds p. 22, p. 27). It has no internal hysteresis (ds p. 23), and its timing figures are typical only (ds p. 12).

## Key parameters

TLV90x2 electrical characteristics: VS = 5 V, VCM = V−, TA = 25 °C unless stated (ds p. 11). Switching: VS = 5 V, VCM = VS/2, CL = 15 pF, TA = 25 °C (ds p. 12).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Input offset voltage VOS | -1.5 | ±0.3 | 1.5 | mV | VS = 1.8 V and 5 V, 25 °C | ds p. 11 |
| Input offset voltage VOS | -2 | — | 2 | mV | VS = 1.8 V and 5 V, -40 °C to 125 °C | ds p. 11 |
| Offset drift dVIO/dT | — | ±0.5 | — | µV/°C | VS = 1.8 V and 5 V, -40 °C to 125 °C | ds p. 11 |
| Quiescent current per comparator IQ | — | 16 | 30 | µA | VS = 1.8 V and 5 V, no load, output low, 25 °C | ds p. 11 |
| Quiescent current per comparator IQ | — | — | 35 | µA | As above, -40 °C to 125 °C | ds p. 11 |
| PSRR (push-pull version) | 75 | 95 | — | dB | VS = 1.8 V to 5 V, -40 °C to 125 °C | ds p. 11 |
| Input bias current IB | — | 5 | — | pA | VCM = VS/2 (typical only) | ds p. 11 |
| Input offset current IOS | — | 1 | — | pA | VCM = VS/2 | ds p. 11 |
| Input capacitance, differential CID | — | 2 | — | pF | VCM = VS/2 | ds p. 11 |
| Input capacitance, common mode CIC | — | 3 | — | pF | VCM = VS/2 | ds p. 11 |
| Common-mode voltage range | (V−) − 0.2 | — | (V+) + 0.2 | V | VS = 1.8 V and 5 V, -40 °C to 125 °C | ds p. 11 |
| CMRR | 60 | 70 | — | dB | VS = 5 V, full VCM range, -40 °C to 125 °C | ds p. 11 |
| CMRR | 50 | 60 | — | dB | VS = 1.8 V, full VCM range, -40 °C to 125 °C | ds p. 11 |
| VOL, swing from V− | — | 75 | 125 | mV | ISINK = 4 mA, 25 °C | ds p. 11 |
| VOL, swing from V− | — | — | 175 | mV | ISINK = 4 mA, -40 °C to 125 °C | ds p. 11 |
| VOH, swing from V+ | — | 75 | 125 | mV | ISOURCE = 4 mA, 25 °C (push-pull only) | ds p. 11 |
| VOH, swing from V+ | — | — | 175 | mV | ISOURCE = 4 mA, -40 °C to 125 °C (push-pull only) | ds p. 11 |
| Short-circuit current ISC, sinking | 90 | 100 | — | mA | VS = 5 V | ds p. 11 |
| Short-circuit current ISC, sourcing | 90 | 100 | — | mA | VS = 5 V (push-pull only) | ds p. 11 |
| Propagation delay tPD-HL | — | 100 | — | ns | VID = -100 mV, mid-point to mid-point | ds p. 12 |
| Propagation delay tPD-LH | — | 115 | — | ns | VID = 100 mV (push-pull only) | ds p. 12 |
| Output fall time, 80% to 20% | — | 3 | — | ns | VID = -100 mV, 5 V | ds p. 12 |
| Output rise time, 20% to 80% | — | 3 | — | ns | VID = 100 mV, 5 V (push-pull only) | ds p. 12 |
| Toggle frequency FTOGGLE | — | 3 | — | MHz | VID = 100 mV, 5 V | ds p. 12 |
| Power-on time PON | — | 20 | — | µs | VS = 1.8 V and 5 V, VCM = V−, VID = -0.1 V, delay from VS/2 to VOUT = 0.1 × VS/2 | ds p. 12 |
| Sink/source current per output (typical capability) | — | 100 | — | mA | All channels together must be < 200 mA | ds p. 22 |
| RθJA, D (SOIC-8) | — | 167.7 | — | °C/W | Thermal metric | ds p. 8 |
| RθJC(top) / RθJB, D (SOIC-8) | — | 107.0 / 111.2 | — | °C/W | Thermal metric | ds p. 8 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VS = (V+) − (V−) | -0.3 to 6 | V | ds p. 7 |
| Input pins IN+, IN− from V− | -0.3 to 6 | V | ds p. 7 |
| Current into input pins | -10 to 10 | mA | ds p. 7 |
| Output OUT from V−, push-pull | -0.3 to (V+) + 0.3 | V | ds p. 7 |
| Output short-circuit duration (to V− or V+) | 10 | s | ds p. 7 |
| Junction temperature TJ | 150 | °C | ds p. 7 |
| Storage temperature Tstg | -65 to 150 | °C | ds p. 7 |
| ESD, HBM (JS-001) | ±2000 | V | ds p. 7 |
| ESD, CDM (JESD22-C101) | ±1000 | V | ds p. 7 |

Notes on p. 7: inputs are diode-clamped to V−, and signals that can swing more than 0.3 V beyond the rails must be limited to 10 mA. Inputs may be above V+ and OUT as long as they stay within -0.3 V to 6 V (ds p. 7, note 2). The push-pull output has an ESD diode to V+ and must not exceed the supply rails (ds p. 23).

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VS = (V+) − (V−) | 1.65 | 5.5 | V | ds p. 7 |
| Input voltage IN+, IN− from V− | -0.2 | 5.7 | V | ds p. 7 |
| Ambient temperature TA | -40 | 125 | °C | ds p. 7 |

## Pins

D (SOIC-8), Figure 4-3 and Table 4-2 (ds p. 4). Checked pad by pad against a rendering of p. 4.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | OUT1 | OUT1 | Output (push-pull) | Output of comparator 1 | ds p. 4, p. 22 |
| 2 | IN1_N | IN1− | Input | Inverting input of comparator 1 | ds p. 4 |
| 3 | IN1_P | IN1+ | Input | Noninverting input of comparator 1 | ds p. 4 |
| 4 | V_MINUS | V− | Supply | Negative (low) supply; GND in single-supply use | ds p. 4, p. 34 |
| 5 | IN2_P | IN2+ | Input | Noninverting input of comparator 2 | ds p. 4 |
| 6 | IN2_N | IN2− | Input | Inverting input of comparator 2 | ds p. 4 |
| 7 | OUT2 | OUT2 | Output (push-pull) | Output of comparator 2 | ds p. 4, p. 22 |
| 8 | V_PLUS | V+ | Supply | Positive supply | ds p. 4 |

Top view: pads 1–4 down the left side from the pin-1 mark, pads 5–8 up the right side, with 8 (V+) across from 1 (ds p. 4). The SOIC has no exposed pad; the thermal pad in Table 4-2 belongs to the DSG (WSON) package only (ds p. 4).

## Typical application

- Bypass V+ with a low-ESR 0.1 µF ceramic capacitor directly between the supply and ground pins. The push-pull output draws narrow current peaks during transitions, so bypassing matters more than for the open-drain part (ds p. 34).
- Single supply: V− to GND. On split supplies the output swings low to V−, not GND (ds p. 34).
- No pull-up is needed; the output swings to V+ and V− (ds p. 22, p. 31). Its high level is V+, so V+ must equal the receiving logic's supply.
- Never connect an output to another output (ds p. 22).
- Unused channel: leave the output floating; never tie it to a supply, ground or another output (ds p. 22). Do not tie its inputs together. Tie them to two valid levels at least 50 mV apart, for example one to GND and the other to V+ directly at the V+ pin (ds p. 23).
- Low-impedance input sources: add a series resistor that limits clamp current to 10 mA or less (ds p. 22, p. 23).
- For slow signals add external hysteresis (ds p. 23); see the inverting and non-inverting networks and Equations 1–6 (ds pp. 25–27).

## Layout notes

- D0008A land (ds p. 69, drawing 4214825/C): eight 0.061 × 0.024 in [1.55 × 0.6 mm] pads, R0.002 in [0.05 mm] corners, 0.050 in [1.27 mm] pitch, 0.213 in [5.4 mm] between row centres. Non-solder-mask-defined openings 0.0028 in [0.07 mm] max all around (ds p. 69). Inches are controlling (ds p. 68, note 1).
- Library `soic::TI_D0008A_4214825C` (soic 0.3.0): 1.5494 × 0.6096 mm pads, R0.0508 mm, 1.27 mm pitch, rows at ±2.7051 mm (5.4102 mm). This is the inch values converted exactly, turned 90° counter-clockwise so pad 1 is at the left end of the lower row; numbering stays counter-clockwise in top view. It matches the source land.
- Stencil (ds p. 70): apertures equal to the pads, for a 0.125 mm stencil.
- Body (ds p. 68): 4.81–5.00 mm long, 3.81–3.98 mm wide, 5.80–6.19 mm across the leads, 1.75 mm max height.
- Output edges are tens of nanoseconds; treat the outputs as high-speed logic. Place the bypass capacitor as close to the supply pin as possible, preferably directly between V+ and GND, over a solid ground plane (ds p. 34). Keep input and output traces apart; place input series resistors close to the device; a < 100 Ω series output resistor can damp ringing on long traces (ds p. 34). Figure 7-20 shows the dual layout (ds p. 35).

## Pitfalls

- Outputs cannot be wired-OR'd and cannot be pulled to a rail other than V+ (ds p. 7, p. 22, p. 23). Window comparators with joined outputs and level shifting to a different rail need the TLV9022 (ds p. 27, p. 31).
- Tying an unused output to GND or V+ is forbidden, unlike on the TLV9022 (ds p. 21, p. 22).
- No internal hysteresis: slow or noisy inputs make the output chatter (ds p. 23, p. 24).
- Timing (100 ns / 115 ns) and input bias (5 pA) are typical values only, with no maxima (ds p. 11, p. 12). Do not use them as guaranteed limits.
- The "300 µV" headline offset is typical; the guaranteed limit is ±1.5 mV at 25 °C and ±2 mV over temperature (ds p. 1, p. 11).
- Output state is defined only while at least one input is in the valid input range and the supply is valid; with both inputs out of range the output is random (ds p. 22).
- Fault tolerance (inputs 0–5.5 V with VS off) is about input impedance. It does not mean the inputs may exceed the absolute 6 V from V− (ds p. 7, p. 22). Above-range inputs and inputs below V− forward-bias clamps, and bias current rises exponentially (ds p. 22).
- Total output current of both channels must stay below 200 mA (ds p. 22). Output short circuits to either rail are rated for only 10 s (ds p. 7).

## Not established by the sources

The library's pin numbers and names, pin-1 location, land and output typing agree with the source (ds p. 4, p. 22, p. 69). Where the library goes beyond or differs from the source:

- **Solder mask.** The source asks for non-solder-mask-defined openings with 0.07 mm max clearance (ds p. 69). The soic pad sets no mask expansion, so the clearance depends on the tool default. Paste is not encoded separately; the source stencil equals the pad (ds p. 70).
- **Courtyard and pin-1 dot** are library additions (`lib/soic/docs/ti-d0008a.md`); the source gives a "pin 1 ID area" only on the body (ds p. 68).
- **Application note (`docs/tlv90x2-application.md`) check.** Its pin table, supply and temperature range (p. 7), typical timing (p. 12), typical bias and capacitance (p. 11), no-hysteresis (p. 23), unused-input rule (p. 23), 0.1 µF bypass (p. 34), addendum date (pp. 38–39) and land pages (pp. 68–69) all agree with the source. "TLV9032 outputs must not be shorted to rails or other outputs (pages 21-23)" is stricter than the absolute maximum table, which allows a 10 s short to V− or V+ (ds p. 7); the "never tie" rule is for unused outputs and for output-to-output connections (ds p. 22).
- **Datasheet inconsistencies:**
  - Quiescent current is "16 µA per channel" in Features and "18 µA per channel" in the Description (ds p. 1); the table gives 16 µA typ (ds p. 11).
  - Inputs "can go up to 6V without damage" (ds p. 1); fault tolerance is "up to 5.5V independent of VS" (ds p. 22); the recommended input range is up to 5.7 V from V− (ds p. 7).
  - The layout example names the pins 1OUT, 1IN−, 1IN+, GND, 2IN+, 2IN−, 2OUT, VCC (ds p. 35), and the supply text says VCC (ds p. 34). These are the same pins as V−/V+ on p. 4.
- **Not stated by the source:** maximum propagation delay, maximum input bias current or input leakage while unpowered, VOH and VOL at currents other than 4 mA, behaviour during a supply ramp other than the 20 µs typical power-on time (ds p. 12), and the board behind the RθJA value (ds p. 8).
- **Ordering:** TLV9032DR is Active, marked TL9032, NiPdAu, MSL 1 at 260 °C, 2500 per reel, pin 1 in quadrant Q1 (ds p. 39, p. 43). TLV9032DRG4 is also Active with the same package and marking (ds p. 39); the library lists no alternate.
