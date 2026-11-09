# TMUX1072RUTR — 2-channel 2:1 analog switch with 18 V OVP, active-low OE and open-drain FLT, UQFN-12 2.0 × 1.7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/analog-switch::TMUX1072_RUT",
  "parts": ["@contrib/analog-switch::SW_TMUX1072RUTR"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TMUX1072RUTR", "lcsc": "C2673115"}
  ],
  "sources": [
    {"id": "ds", "title": "TMUX1072 2-Channel 2:1 Analog Multiplexer with Overvoltage Detection and Protection", "revision": "SCDS382C, April 2018, revised August 2019; packaging addendum and pack materials dated 5-Nov-2024; RUT0012A drawing 4220310/A 11/2016",
     "url": "https://www.ti.com/lit/ds/symlink/tmux1072.pdf", "sha256": "a54e8a9336f4ac62fb9a3e1f045499077558bb6ebadaca3004574099c9318a4f", "local": "lib/@contrib/analog-switch/docs/ti-tmux1072-scds382c.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal TI's printed page numbers on pp. 1–20. The packaging addendum (p. 21, "Addendum-Page 1"), pack materials (pp. 22–23) and mechanical drawings (pp. 24–29) carry no datasheet page numbers. The URL is the one recorded in `lib/@contrib/analog-switch/docs/README.md` and `tmux1072-application.md`.

The datasheet covers two packages. This note is the 12-pin UQFN (RUT). The 10-pin VSSOP, which has no OE and no FLT, is `SW_TMUX1072DGSR.md`, a separate library device.

## Summary

The TMUX1072 is a bidirectional 2-channel, 2:1 (or 1:2) analog switch for signals from 0 V to 5.5 V, including above VCC, on a 2.3–5.5 V supply (ds p. 1, p. 4). Its COM pins tolerate up to 18 V: above about 5.8 V the switch opens to protect what is behind it, also with VCC = 0 V (ds p. 6, p. 15). Over-voltage and over-temperature events pull the open-drain FLT pin low (ds p. 15). OE is active low with an internal pull-down, so the switch is enabled by default; OE high puts both channels in high-Z (ds p. 3, p. 16, p. 17). RON is 6 Ω typ, 18 Ω max; bandwidth is 1.2 GHz typ (ds p. 5, p. 8). Continuous signal current is ±50 mA (ds p. 4): it is a signal switch, not a power-path switch.

## Key parameters

TA = -40 °C to 125 °C, VCC = 2.3 V to 5.5 V; typical values at VCC = 3.3 V, TA = 25 °C, unless stated (ds p. 5).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Active supply current ICC | — | 75 | 110 | µA | OE = 0 V; SEL1, SEL2 = 0 V, 1.8 V or VCC; 0 V < VI/O < 3.6 V | ds p. 5 |
| Supply current during OVP | — | 65 | 98 | µA | OE = 0 V; VI/O > VPOS_THLD | ds p. 5 |
| Standby supply current ICC_PD | — | 3 | 10 | µA | OE = 1.8 V or VCC; SEL = 0 V, 1.8 V or VCC | ds p. 5 |
| UVLO | 1.65 | — | — | V | VCC rising and falling (only a minimum is printed) | ds p. 5 |
| ON resistance RON | — | 6 | 18 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| RON match between channels ΔRON | — | 0.07 | 0.5 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| RON flatness | — | 2.5 | 7 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| OFF leakage IOFF, powered | — | 3.6 | 10 | µA | VCOM = 0–5.5 V, VCC = 2.3–5.5 V, VNC/VNO = 5.5 V or 0 V | ds p. 6 |
| OFF leakage IOFF, unpowered | — | — | 15 | µA | VCOM = 5.5 V, VCC = 0 V, VNC/VNO = 5.5 V | ds p. 6 |
| OFF leakage IOFF, unpowered | — | — | 2 | µA | VCOM = 1 V, VCC = 0 V, VNC/VNO = 1 V | ds p. 6 |
| COM leakage during 18 V fault | — | 165 | 185 | µA | VCOM = 18 V, VCC = 0 V or 5.5 V, VNC/VNO = 0 V | ds p. 6 |
| ON leakage ION | — | — | 11.5 | µA | VCOM = 0–5.5 V, VCC = 2.3–5.5 V, NC/NO high-Z | ds p. 6 |
| Logic input high VIH (SEL1, SEL2, OE) | 1.45 | — | — | V | — | ds p. 6 |
| Logic input low VIL (SEL1, SEL2, OE) | — | — | 0.5 | V | — | ds p. 6 |
| FLT output low VOL | — | — | 0.3 | V | IOL = 3 mA | ds p. 6 |
| Input high leakage IIH | -1 | 2 | 5 | µA | SEL1, SEL2, OE = 1.8 V or VCC | ds p. 6 |
| Internal pull-down RPD, SEL1/SEL2 | — | 6 | 12 | MΩ | — | ds p. 6 |
| Internal pull-down RPD, OE | — | 3 | 6 | MΩ | — | ds p. 6 |
| OVP threshold VOVP_TH | 5.55 | 5.8 | 6.0 | V | COM pins | ds p. 6 |
| OVP hysteresis | 40 | 100 | 300 | mV | By design | ds p. 6 |
| Overtemperature detection threshold TOTD_TH | 135 | — | 165 | °C | By design | ds p. 7 |
| Thermal shutdown hysteresis | 3 | — | 8 | °C | By design | ds p. 7 |
| Voltage on NC/NO during OVP VCLAMP_V | 0 | — | 9.6 | V | VCOM = 0 to 18 V, 100 ns edges, RL open, OE = 0 V (9.0 V with RL = 50 Ω); by design | ds p. 7 |
| OVP enable time tEN_OVP | — | 0.6 | 3 | µs | RPU = 10 kΩ to VCC on FLT, CL = 35 pF; by design | ds p. 7 |
| OVP recovery time tREC_OVP | — | 1.5 | 5 | µs | RPU = 10 kΩ to VCC on FLT, CL = 35 pF; by design | ds p. 7 |
| COFF, CON | 1.2 / 1.4 | 4.0 | 6.2 | pF | f = 240 MHz; by design | ds p. 8 |
| Bandwidth | — | 1.2 | — | GHz | RL = 50 Ω; by design | ds p. 8 |
| Switching time tswitch (SEL to output) | — | 0.9 | 1 | µs | VNC = 0.8 V | ds p. 8 |
| Turn-on time ton (OE to output) | — | 200 | 250 | µs | VNC = 0.8 V, RL = 50 Ω, CL = 5 pF | ds p. 8 |
| Turn-off time toff (OE to output) | — | 1 | 10 | µs | VNC = 0.8 V, RL = 50 Ω, CL = 5 pF | ds p. 8 |
| Turn-off on supply loss toff_Vcc | — | — | 250 | µs | VCC ramp 2.3 V to 0 V in 250 µs | ds p. 8 |
| RθJA, RUT | — | 127 | — | °C/W | Thermal metric | ds p. 5 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | -0.5 to 6 | V | ds p. 4 |
| I/O DC voltage, COM1, COM2 | -0.5 to 20 | V | ds p. 4 |
| I/O DC voltage, NC1, NO1, NC2, NO2 | -0.5 to 6 | V | ds p. 4 |
| Digital input voltage (SEL1, SEL2, OE) | -0.5 to 6 | V | ds p. 4 |
| Digital output voltage (FLT) | -0.5 to 6 | V | ds p. 4 |
| I/O port diode current IK (VIN < 0) | -50 | mA | ds p. 4 |
| Logic input clamp current IIK (VI < 0) | -50 | mA | ds p. 4 |
| Continuous current through VCC | 100 | mA | ds p. 4 |
| Continuous current through GND | -100 | mA | ds p. 4 |
| Storage temperature | -65 to 150 | °C | ds p. 4 |
| Junction temperature ("operating", as printed) | -65 to 150 | °C | ds p. 4 |
| ESD, HBM (JS-001) | ±2000 | V | ds p. 4 |
| ESD, CDM (JESD22-C101) | ±1000 | V | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 2.3 | 5.5 | V | ds p. 4 |
| Analog I/O voltage, COM1, COM2 | 0 | 18 | V | ds p. 4 |
| Analog I/O voltage, NC1, NO1, NC2, NO2 | 0 | 5.5 | V | ds p. 4 |
| Analog I/O continuous current (all signal pins) | -50 | 50 | mA | ds p. 4 |
| Digital input voltage (SEL1, SEL2, OE) | 0 | 5.5 | V | ds p. 4 |
| Digital output voltage (FLT) | 0 | 5.5 | V | ds p. 4 |
| Digital output current IOL (FLT) | — | 3 | mA | ds p. 4 |
| Operating free-air temperature TA | -40 | 125 | °C | ds p. 4 |
| Junction temperature TJ | -40 | 125 | °C | ds p. 4 |

The 18 V COM range is the protected range: above VOVP_TH (5.55–6.0 V) the switch is open (ds p. 6, p. 15). Signals pass only up to 5.5 V (ds p. 1).

## Pins

RUT (UQFN-12), top view: pin 1 SEL1 upper left, pins 1–5 down the left side, pin 6 OE centred on the bottom edge, pins 7–11 up the right side, pin 12 FLT centred on the top edge (ds p. 3; same arrangement in the land pattern, ds p. 28).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | SEL1 | SEL1 | Input | Switch select 1: low COM1–NC1, high COM1–NO1; internal pull-down | ds p. 3, p. 16 |
| 2 | NO1 | NO1 | I/O | Signal path NO1 (normally open) | ds p. 3 |
| 3 | GND | GND | Ground | Ground | ds p. 3 |
| 4 | NO2 | NO2 | I/O | Signal path NO2 | ds p. 3 |
| 5 | SEL2 | SEL2 | Input | Switch select 2: low COM2–NC2, high COM2–NO2; internal pull-down | ds p. 3, p. 16 |
| 6 | OE_N | OE (overbar, active low) | Input | Output enable: low enables, high puts COM1 and COM2 in high-Z; internal pull-down | ds p. 3, p. 6, p. 16 |
| 7 | COM2 | COM2 | I/O | Common signal path 2; 18 V OVP | ds p. 3 |
| 8 | NC2 | NC2 | I/O | Signal path NC2 (normally closed, not "no connect") | ds p. 3 |
| 9 | VCC | VCC | Power | Supply | ds p. 3 |
| 10 | NC1 | NC1 | I/O | Signal path NC1 (normally closed) | ds p. 3 |
| 11 | COM1 | COM1 | I/O | Common signal path 1; 18 V OVP | ds p. 3 |
| 12 | FLT_N | FLT (overbar, active low) | Output, open drain | Fault: low on overvoltage or overtemperature; may float or go to ground if unused | ds p. 3, p. 15 |

Checked pad by pad against the RUT pinout and the Pin Functions table (ds p. 3); all twelve agree. The package has no exposed pad (ds p. 27).

## Typical application

- 100 nF bypass capacitor as close to VCC as possible (ds p. 17, p. 18).
- SEL1, SEL2 and OE have internal pull-downs, so no external resistors are needed: by default NC1 and NC2 are selected and the switch is enabled once VCC is applied (ds p. 17).
- To start disabled, pull OE high externally. The source does not size this resistor; the internal pull-down is 3 MΩ typ, 6 MΩ max (ds p. 6).
- FLT is open drain and needs an external pull-up to report faults; the timing specs use 10 kΩ to VCC (ds p. 7, p. 17). If unused, leave FLT floating or tie it to ground (ds p. 3, p. 17). Sink at most 3 mA (ds p. 4).
- TI recommends tying unused signal pins to ground through 50 Ω to prevent reflections (ds p. 17).
- Function table: OE high gives COM1, COM2 high-Z; OE low with SELx low connects COMx to NCx, SELx high connects COMx to NOx (ds p. 16).
- Put the side that may see a fault (connector, bus) on COM1/COM2; only COM has OVP (ds p. 14).

## Layout notes

- RUT0012A land (ds p. 28): twelve 0.7 × 0.2 mm pads with R0.05 corners. Side pads 1–5 and 7–11 at 0.4 mm pitch, rows 1.4 mm apart (centres at ±0.7 mm). End pads 6 and 12 are 0.2 mm wide and 0.7 mm long, oriented perpendicular to their edge, 1.7 mm apart (centres at ±0.85 mm). Non-solder-mask-defined is preferred, with 0.05 mm max mask opening. `FP_TI_RUT0012A` matches these: side pads 0.7 × 0.2 mm at x = ±0.7 mm, y = 0, ±0.4, ±0.8 mm; pads 6 and 12 0.2 × 0.7 mm at (0, +0.85) and (0, -0.85) mm, pad 12 on the pin-1 edge. It uses square corners and does not encode the mask.
- RUT0012A body (ds p. 27): 1.6–1.8 mm across the side rows, 1.9–2.1 mm along them, 0.55 mm max height; terminals 0.15–0.25 mm wide, 0.4–0.6 mm long. No exposed pad.
- Stencil: same openings, 0.1 mm stencil (ds p. 29).
- Layout example (ds p. 19): bypass capacitor next to VCC (pin 9).
- High-speed routing: equal-length traces no longer than 4 inches, few vias, no stubs, continuous reference plane, keep bypass caps away from signal traces (ds p. 18).

## Pitfalls

- OE is active low with a pull-down (ds p. 3, p. 17). An unconnected OE enables the switch at power-up; it does not isolate.
- `NC1`/`NC2` are the normally-closed throws, not no-connect pins (ds p. 3).
- OE to output turn-on is 200 µs typ, 250 µs max (ds p. 8). Do not expect a fast enable.
- NC/NO pins are rated only to 6 V absolute, 5.5 V operating (ds p. 4). Never put a fault-exposed node on them.
- During an 18 V COM transient NC/NO may still see up to 9.6 V (ds p. 7). OVP is not a low-voltage clamp.
- ±50 mA continuous on each signal pin (ds p. 4). Do not route a power rail through it.
- The two packages have different pin numbers from pin 6 on (COM2 is 7 on RUT, 6 on DGS; ds p. 3). Do not swap footprints between the parts.

## Not established by the sources

The library's pin numbers, names and pin 1 location agree with ds p. 3, and `FP_TI_RUT0012A` matches the land on ds p. 28 pad by pad, including the end-terminal positions. Points that differ from or go beyond the source:

- **FLT typed `output`.** The library declares `FLT_N` as `[output]`. The source calls it an open-drain output (ds p. 3, p. 15, p. 17). An ERC that treats `output` as push-pull will flag a wired-OR fault line and will not ask for the pull-up.
- **Library pin requirements.** `OE_N`, `SEL1` and `SEL2` are `required`, although each has an internal pull-down and the source says no external components are needed on the logic pins (ds p. 17). All signal pins are `required`; the source only recommends 50 Ω to ground for unused ones (ds p. 17).
- **Datasheet inconsistencies.**
  - The revision history says the feature changed from "0-V to 18-V OVP" to "0-V to 6-V OVP on common pins" (ds p. 2). Page 1 still lists "18-V Overvoltage protection (OVP) on common pins" and "6-V overvoltage ... detection" (ds p. 1), and the ROC allows COM up to 18 V (ds p. 4).
  - Insertion loss is given at "f = TBD MHz" (ds p. 8).
  - UVLO has only a minimum of 1.65 V, no maximum (ds p. 5).
  - Absolute maximum junction temperature is labelled "Operating Junction Temperature", -65 °C to 150 °C, while the ROC gives -40 °C to 125 °C (ds p. 4).
  - The ICC-during-OVP condition names "VPOS_THLD", which is not otherwise defined; the threshold elsewhere is VOVP_TH (ds p. 5, p. 6).
  - The thermal shutdown hysteresis is specified (ds p. 7), but the source does not say whether the switch shuts down on over-temperature or only flags it on FLT (ds p. 15).
- **Footprint details not encoded.** Corner radius R0.05 and the NSMD mask clearance (ds p. 28) are not in the library land. The 2.6 × 3.8 mm courtyard is a library margin; the source gives none.
- **Pin 1 in tape.** Pin 1 is in quadrant Q1 (ds p. 22).
- **`tmux1072-application.md`** agrees with the pages it cites (pp. 3–8, 14–18, 21, 25, 28). One wording point: it says OE high "disconnects both channels"; the source says COM1 and COM2 go high-Z (ds p. 16).
- **Not stated:** the external pull-up value for a disabled start-up, FLT state while VCC is below UVLO, behaviour between 0 V and the UVLO level, and RON over the 0–5.5 V range above VCC.
