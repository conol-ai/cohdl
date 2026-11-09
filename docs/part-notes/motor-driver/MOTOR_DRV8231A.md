# DRV8231ADDAR — 4.5–33 V, 3.7 A peak brushed-DC H-bridge with current-mirror sensing, HSOP-8 PowerPAD

## Identity

```json
{
  "schema": "part-note/1",
  "device": "motor-driver::DRV8231A",
  "parts": ["motor-driver::MOTOR_DRV8231A"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "DRV8231ADDAR", "lcsc": "C5218849"}
  ],
  "sources": [
    {"id": "ds", "title": "DRV8231A 3.7-A Brushed DC Motor Driver with Integrated Current Sense and Regulation", "revision": "SLVSFZ8A, November 2021, revised January 2026, 44 pages",
     "url": "https://www.ti.com/lit/ds/symlink/drv8231a.pdf", "sha256": "96545c004b2fa920eb71e99287d27fd1803626814c448f291ef500dbc24749c9", "local": "lib/motor-driver/docs/drv8231a.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed page numbers on pp. 1–34. Pages 35–44 are the orderable addendum, package drawings and notice; they carry no matching printed number. Page 37 is drawing DDA0008B (4214849/B), page 38 its example board layout, page 39 its stencil.

## Summary

The DRV8231A drives one brushed DC motor (or a relay) from a single 4.5–33 V supply. It has an N-channel H-bridge, a charge pump, 3.7 A peak output, and about 600 mΩ total RDS(on) (300 mΩ high side plus 300 mΩ low side, typ) (ds p. 1, p. 4, p. 5, p. 9). Two logic inputs, IN1 and IN2, select forward, reverse, brake or coast; holding both low puts it to sleep at under 1 µA (ds p. 5, p. 11). The low-side FETs feed a current mirror: IPROPI sources 1500 µA per amp of load current, and a resistor to ground turns that into a voltage for an ADC (ds p. 5, p. 13). Comparing that voltage with VREF gives fixed-off-time current regulation with no MCU in the loop (ds p. 15). There is no fault output pin. UVLO, overcurrent and thermal shutdown recover by themselves (ds p. 16, p. 17). The library binds only the DDA (HSOP-8 PowerPAD) package; a 2 × 2 mm WSON (DSG) option also exists (ds p. 1, p. 35).

## Key parameters

4.5 V ≤ VVM ≤ 33 V, −40 °C ≤ TJ ≤ 150 °C; typical values at TJ = 25 °C, VVM = 24 V (ds p. 5).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| VM sleep current IVMQ | — | — | 1 | µA | VVM = 24 V, IN1 = IN2 = 0, TJ = 25 °C | ds p. 5 |
| VM active current IVM | — | 3 | 4 | mA | VVM = 24 V, IN1 = IN2 = 1 | ds p. 5 |
| Turn-on time tWAKE | — | — | 250 | µs | Control signal to active mode | ds p. 5 |
| Turn-off time tSLEEP | 0.8 | — | 1.5 | ms | Control signal to sleep | ds p. 5 |
| Logic low VIL | — | — | 0.5 | V | INx | ds p. 5 |
| Logic high VIH | 1.5 | — | — | V | INx | ds p. 5 |
| Input pulldown RPD | — | 100 | — | kΩ | INx to GND | ds p. 5 |
| High-side RDS(on) | — | 300 | — | mΩ | VVM = 24 V, I = 1 A | ds p. 5 |
| Low-side RDS(on) | — | 300 | — | mΩ | VVM = 24 V, I = 1 A | ds p. 5 |
| Output rise / fall time | — | 220 | — | ns | VVM = 24 V, 10–90 % | ds p. 5 |
| Propagation delay tPD | — | 0.7 | 1 | µs | INx to OUTx | ds p. 5 |
| Dead time tDEAD | — | 200 | — | ns | — | ds p. 5 |
| Current-mirror scaling AIPROPI | — | 1500 | — | µA/A | — | ds p. 5 |
| Current-mirror total error AERR | −6 | — | 6 | % | IOUT = 1 A, VVM ≥ 6.5 V, VIPROPI ≤ 3.0 V | ds p. 5 |
| Current-regulation off time tOFF | — | 25 | — | µs | — | ds p. 5 |
| Current-regulation blanking tBLK | — | 1.4 | — | µs | — | ds p. 5 |
| UVLO, VM rising | 4.15 | 4.3 | 4.45 | V | — | ds p. 5 |
| UVLO, VM falling | 4.05 | 4.2 | 4.35 | V | — | ds p. 5 |
| Overcurrent trip IOCP | 3.7 | — | — | A | — | ds p. 5 |
| OCP retry time tRETRY | — | 3 | — | ms | — | ds p. 6 |
| Thermal shutdown TTSD | 150 | 175 | — | °C | — | ds p. 6 |
| Thermal shutdown hysteresis | — | 40 | — | °C | — | ds p. 6 |
| RθJA, DDA (HSOP) | — | 42.8 | — | °C/W | JEDEC metric | ds p. 4 |
| RθJC(bot), DDA | — | 6.2 | — | °C/W | — | ds p. 5 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VM pin voltage | −0.3 to 35 | V | ds p. 4 |
| VM transient ramp | 0 to 2 | V/µs | ds p. 4 |
| INx pin voltage | −0.3 to 7 | V | ds p. 4 |
| VREF pin voltage | −0.3 to 6 | V | ds p. 4 |
| OUTx pin voltage | −0.7 to VM + 0.7 | V | ds p. 4 |
| IPROPI pin voltage | −0.3 to 5.75 | V | ds p. 4 |
| OUTx current | Internally limited | — | ds p. 4 |
| Ambient temperature TA | −40 to 125 | °C | ds p. 4 |
| Junction temperature TJ | −40 to 150 | °C | ds p. 4 |
| Storage temperature | −65 to 150 | °C | ds p. 4 |
| ESD, HBM | ±2000 | V | ds p. 4 |
| ESD, CDM | ±500 | V | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VVM | 4.5 | 33 | V | ds p. 4 |
| Reference voltage VVREF | 0 | 3.6 | V | ds p. 4 |
| Logic input voltage VIN | 0 | 5.5 | V | ds p. 4 |
| PWM frequency fPWM | 0 | 200 | kHz | ds p. 4 |
| Peak output current IOUT (power and thermal limits apply) | 0 | 3.7 | A | ds p. 4 |
| IPROPI output current IIPROPI | 0 | 3 | mA | ds p. 4 |
| Operating ambient TA | −40 | 125 | °C | ds p. 4 |
| Operating junction TJ | −40 | 150 | °C | ds p. 4 |

## Pins

Checked pad by pad against Figure 5-1 (DDA top view) and Table 5-1 (ds p. 3). The drawing numbers the exposed thermal pad 9 (ds p. 37, p. 38).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | IPROPI | IPROPI | PWR (analog current output) | Current-mirror output; resistor to GND | ds p. 3 |
| 2 | IN2 | IN2 | I | Logic input, internal pulldown | ds p. 3 |
| 3 | IN1 | IN1 | I | Logic input, internal pulldown | ds p. 3 |
| 4 | VREF | VREF | I | Analog reference for current regulation | ds p. 3 |
| 5 | VM | VM | PWR | Supply; 0.1 µF plus bulk to GND | ds p. 3 |
| 6 | OUT1 | OUT1 | O | H-bridge output to the motor | ds p. 3 |
| 7 | GND | GND | PWR | Device power ground | ds p. 3 |
| 8 | OUT2 | OUT2 | O | H-bridge output to the motor | ds p. 3 |
| 9 | PAD | PAD (thermal pad) | — | Thermal pad; connect to board ground | ds p. 3, p. 37 |

Pad positions agree: pins 1–4 down the left side, 5–8 up the right side, pin 1 at the top left in the top view (ds p. 3, p. 37, p. 38). `FP_TI_DRV8231A_DDA` places them the same way, with y increasing downward.

## Typical application

- VM: 0.1 µF low-ESR ceramic (X5R or X7R), VM-rated, plus bulk capacitance, both close to the pins (ds p. 3, p. 10, p. 32). Figure 8-1 shows 0.1 µF and 47 µF (ds p. 19). The source sizes bulk only by system testing (ds p. 32).
- IPROPI: resistor RIPROPI to GND. VIPROPI = IIPROPI × RIPROPI, with IIPROPI = (ILS1 + ILS2) × AIPROPI (ds pp. 13–14). Keep IIPROPI ≤ 3 mA (ds p. 4), which is 2 A of load current at 1500 µA/A.
- Current regulation trips at ITRIP = VVREF / (RIPROPI × AIPROPI) (ds p. 15). TI's example: 3.3 V, 1375 Ω gives about 1.6 A (ds p. 15). Optional 10 nF on IPROPI near the device filters transients (ds p. 15).
- To disable regulation and feedback, tie IPROPI to GND and set VREF above GND (ds p. 15).
- IN1/IN2: 00 coast then sleep after 1 ms, 01 reverse, 10 forward, 11 brake (ds p. 11). The inputs may be driven before VM is applied (ds p. 11). Internal 100 kΩ pulldowns hold the device asleep when the MCU is not driving them (ds p. 5).
- Fault behaviour: OCP disables the bridge and retries after tRETRY; TSD and UVLO recover automatically (ds p. 16, p. 17).

## Layout notes

- DDA land (ds p. 38): eight 1.55 × 0.6 mm pads, 1.27 mm pitch, 5.4 mm between the pad-row centrelines. `FP_TI_DRV8231A_DDA` matches (pads at ±2.7 mm).
- Thermal pad (ds p. 38): metal pad (2.95 × 4.9 mm, "may vary", note 9) with a solder-mask-defined opening of 2.71 × 3.4 mm. Optional ø0.2 mm vias on a 1.3 mm grid; fill, plug or tent them if under paste (note 10).
- Stencil (ds p. 39): 2.71 × 3.40 mm exposed-pad opening for a 0.125 mm stencil, 100 % coverage; 3.03 × 3.80 mm at 0.1 mm, 2.47 × 3.10 mm at 0.15 mm.
- Package (ds p. 37): body 4.8–5.0 × 3.8–4.0 mm, lead span 5.8–6.2 mm, 1.7 mm max height, exposed pad 2.11–2.71 × 2.8–3.4 mm.
- Heat: tie the thermal pad to large ground planes on several layers with several nearby vias (ds p. 3, p. 33). RθJA depends heavily on copper area (ds pp. 27–29).
- VM, OUT1, OUT2 and GND carry motor current; route them wide (ds p. 32).

## Pitfalls

- VREF must stay within 0–3.6 V for current regulation to work (ds p. 4, p. 15). Table 5-1's "0 to 5 V" disagrees with the recommended operating conditions (ds p. 3).
- IPROPI is limited to 3 mA (ds p. 4), so the mirror covers only 2 A of load current. The 3.7 A peak output current (ds p. 4) gives 5.55 mA at 1500 µA/A, beyond that limit. AERR is specified only for VIPROPI ≤ 3.0 V (ds p. 5).
- IPROPI does not report current in coast or through body diodes (ds p. 13). In brake, it reports one low-side FET only (ds p. 13).
- IN1 = IN2 = 0 coasts and then sleeps. Waking takes up to 250 µs (ds p. 5, p. 17).
- There is no nFAULT pin. Detect faults from IPROPI or motor behaviour; the source suggests firmware stall detection on IPROPI (ds pp. 20–22).
- Overcurrent protection is independent of VREF and IPROPI (ds p. 16). It does not replace the current-regulation setting.
- Moisture sensitivity is level 2, 260 °C peak, for the DDA package (ds p. 35).

## Not established by the sources

Disagreements between the library and the source, most serious first:

- **VREF range in the library comment.** `VREF: 4 [input] // 0-5V analog input` follows Table 5-1 ("between 0 to 5 V", ds p. 3). The recommended operating conditions give 0–3.6 V (ds p. 4), and Section 7.4.2.2 requires VREF within that range for regulation to work (ds p. 15). The data sheet contradicts itself; the library repeats the looser figure.
- **IPROPI range in the example.** `examples/joint-motor-controller/src/motor_stage.cohdl` sizes RIPROPI = 560 Ω so "the 3.7A peak" reads 3.11 V. That is 5.55 mA of IPROPI current, above the 3 mA recommended maximum (ds p. 4), and above the VIPROPI ≤ 3.0 V where AERR is specified (ds p. 5). The library device does not state the 3 mA limit.
- **IPROPI pin type.** The library types IPROPI `output`. Table 5-1 types it PWR (ds p. 3), and the absolute-maximum table calls it a "current sense input pin" (ds p. 4). It is an analog current source; no library type fits exactly.
- **Thermal-pad copper.** The library pad 9 is 2.71 × 3.4 mm copper. TI's example is 2.95 × 4.9 mm metal with a 2.71 × 3.4 mm solder-mask-defined opening (ds p. 38). The exposed solderable area matches; the copper that spreads heat is smaller, and the pad is not solder-mask defined.
- **README wording.** `docs/README.md` calls the 5.4 mm dimension the "outer extent". On p. 38 it runs between pad centrelines. The footprint and its comment use it correctly as centre spacing.
- **VM range in the source.** Table 5-1 says "4.5-V to 48-V power supply" (ds p. 3), but the recommended range is 4.5–33 V and the absolute maximum 35 V (ds p. 4). The library uses 4.5–33 V, which is correct; the 48 V looks like a copy from DRV8251A.
- **PAD pin type.** The library types the thermal pad `power_in`; the source gives it no type ("—", ds p. 3).
- **Thermal vias.** The library leaves vias to the board. The source says they are optional (ds p. 38, note 10) but also advises "multiple nearby vias" for good dissipation (ds p. 3).
- **2S pack comment.** "A 2S lithium pack (6.0-8.4V) sits comfortably inside the range" is the library's own statement; the source says nothing about batteries. 6.0–8.4 V is inside 4.5–33 V (ds p. 4).

Pin numbers and names, IN1/IN2 order, the current-sense pin and the absence of a fault pin agree with the source (ds p. 3).

Not stated by the source:

- Maximum RDS(on). Only typical values are given (ds p. 5).
- A minimum or maximum for AIPROPI apart from AERR (ds p. 5).
- Behaviour when VREF is above 3.6 V.
- A specific bulk capacitance value. Figure 8-1 shows 47 µF without stating it as a requirement (ds p. 19, p. 32).
