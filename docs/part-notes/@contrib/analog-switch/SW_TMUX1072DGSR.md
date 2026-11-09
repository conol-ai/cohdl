# TMUX1072DGSR — 2-channel 2:1 analog switch with 18 V OVP on COM pins, VSSOP-10 (no OE, no FLT)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/analog-switch::TMUX1072_DGS",
  "parts": ["@contrib/analog-switch::SW_TMUX1072DGSR"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TMUX1072DGSR", "lcsc": "C2673257"}
  ],
  "sources": [
    {"id": "ds", "title": "TMUX1072 2-Channel 2:1 Analog Multiplexer with Overvoltage Detection and Protection", "revision": "SCDS382C, April 2018, revised August 2019; packaging addendum and pack materials dated 5-Nov-2024; DGS0010A drawing 4221984/A 05/2015",
     "url": "https://www.ti.com/lit/ds/symlink/tmux1072.pdf", "sha256": "a54e8a9336f4ac62fb9a3e1f045499077558bb6ebadaca3004574099c9318a4f", "local": "lib/@contrib/analog-switch/docs/ti-tmux1072-scds382c.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal TI's printed page numbers on pp. 1–20. The packaging addendum (p. 21, "Addendum-Page 1"), pack materials (pp. 22–23) and mechanical drawings (pp. 24–29) carry no datasheet page numbers. The URL is the one recorded in `lib/@contrib/analog-switch/docs/README.md` and `tmux1072-application.md`.

The datasheet covers two packages. This note is the 10-pin VSSOP (DGS), which has no OE and no FLT pin (ds p. 3). The 12-pin UQFN is `SW_TMUX1072RUTR.md`, a separate library device.

## Summary

The TMUX1072 is a bidirectional 2-channel, 2:1 (or 1:2) analog switch for signals from 0 V to 5.5 V, including above VCC, on a 2.3–5.5 V supply (ds p. 1, p. 4). Its COM pins tolerate up to 18 V: above about 5.8 V the switch opens to protect what is behind it, also with VCC = 0 V (ds p. 6, p. 15). RON is 6 Ω typ, 18 Ω max; bandwidth is 1.2 GHz typ (ds p. 5, p. 8). Each channel has its own select (SEL1, SEL2); low selects NC, high selects NO (ds p. 16). In this DGS package the fault indicator and the output enable are not bonded out, so an overvoltage event is not reported and the switch cannot be disabled (ds p. 3). Use the RUT package if you need either. Continuous signal current is ±50 mA (ds p. 4): it is a signal switch, not a power-path switch.

## Key parameters

TA = -40 °C to 125 °C, VCC = 2.3 V to 5.5 V; typical values at VCC = 3.3 V, TA = 25 °C, unless stated (ds p. 5).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Active supply current ICC | — | 75 | 110 | µA | OE = 0 V; SEL1, SEL2 = 0 V, 1.8 V or VCC; 0 V < VI/O < 3.6 V | ds p. 5 |
| Supply current during OVP | — | 65 | 98 | µA | OE = 0 V; VI/O > VPOS_THLD | ds p. 5 |
| UVLO | 1.65 | — | — | V | VCC rising and falling (only a minimum is printed) | ds p. 5 |
| ON resistance RON | — | 6 | 18 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| RON match between channels ΔRON | — | 0.07 | 0.5 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| RON flatness | — | 2.5 | 7 | Ω | VI/O = 0 V to VCC, ISINK = 8 mA | ds p. 5 |
| OFF leakage IOFF, powered | — | 3.6 | 10 | µA | VCOM = 0–5.5 V, VCC = 2.3–5.5 V, VNC/VNO = 5.5 V or 0 V (not tested on COM pins for DGS, note 2) | ds p. 6 |
| OFF leakage IOFF, unpowered | — | — | 15 | µA | VCOM = 5.5 V, VCC = 0 V, VNC/VNO = 5.5 V | ds p. 6 |
| OFF leakage IOFF, unpowered | — | — | 10 | µA | VCOM = 3.6 V, VCC = 0 V, VNC/VNO = 3.6 V | ds p. 6 |
| OFF leakage IOFF, unpowered | — | — | 2 | µA | VCOM = 1 V, VCC = 0 V, VNC/VNO = 1 V | ds p. 6 |
| COM leakage during 18 V fault | — | 165 | 185 | µA | VCOM = 18 V, VCC = 0 V or 5.5 V, VNC/VNO = 0 V | ds p. 6 |
| ON leakage ION | — | — | 11.5 | µA | VCOM = 0–5.5 V, VCC = 2.3–5.5 V, NC/NO high-Z | ds p. 6 |
| Logic input high VIH (SEL1, SEL2) | 1.45 | — | — | V | — | ds p. 6 |
| Logic input low VIL (SEL1, SEL2) | — | — | 0.5 | V | — | ds p. 6 |
| Input high leakage IIH | -1 | 2 | 5 | µA | SEL = 1.8 V or VCC | ds p. 6 |
| Internal pull-down RPD, SEL1/SEL2 | — | 6 | 12 | MΩ | — | ds p. 6 |
| Digital input capacitance CI | — | 8 | — | pF | f = 1 MHz; by design | ds p. 6 |
| OVP threshold VOVP_TH | 5.55 | 5.8 | 6.0 | V | COM pins | ds p. 6 |
| OVP hysteresis | 40 | 100 | 300 | mV | By design | ds p. 6 |
| Overtemperature detection threshold TOTD_TH | 135 | — | 165 | °C | By design | ds p. 7 |
| Voltage on NC/NO during OVP VCLAMP_V | 0 | — | 9.6 | V | VCOM = 0 to 18 V, 100 ns edges, RL open (9.0 V with RL = 50 Ω); by design | ds p. 7 |
| OVP enable / recovery time | — | 0.6 / 1.5 | 3 / 5 | µs | RPU = 10 kΩ on FLT, CL = 35 pF (FLT exists on RUT only); by design | ds p. 7 |
| COFF, CON | 1.2 / 1.4 | 4.0 | 6.2 | pF | f = 240 MHz; by design | ds p. 8 |
| Bandwidth | — | 1.2 | — | GHz | RL = 50 Ω; by design | ds p. 8 |
| Off isolation | — | -80 | — | dB | RL = 50 Ω, CL = 5 pF, f = 100 kHz (-22 dB at 240 MHz) | ds p. 8 |
| Crosstalk | — | -90 | — | dB | RL = 50 Ω, CL = 5 pF, f = 100 kHz | ds p. 8 |
| Switching time tswitch (SEL to output) | — | 0.9 | 1 | µs | VNC = 0.8 V | ds p. 8 |
| Turn-off on supply loss toff_Vcc | — | — | 250 | µs | VCC ramp 2.3 V to 0 V in 250 µs | ds p. 8 |
| Propagation delay tpd | — | 130 | 200 | ps | RL = 50 Ω, CL = 5 pF; by design | ds p. 8 |
| RθJA, DGS | — | 175 | — | °C/W | Thermal metric | ds p. 5 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | -0.5 to 6 | V | ds p. 4 |
| I/O DC voltage, COM1, COM2 | -0.5 to 20 | V | ds p. 4 |
| I/O DC voltage, NC1, NO1, NC2, NO2 | -0.5 to 6 | V | ds p. 4 |
| Digital input voltage (SEL1, SEL2) | -0.5 to 6 | V | ds p. 4 |
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
| Digital input voltage (SEL1, SEL2) | 0 | 5.5 | V | ds p. 4 |
| Operating free-air temperature TA | -40 | 125 | °C | ds p. 4 |
| Junction temperature TJ | -40 | 125 | °C | ds p. 4 |

The 18 V COM range is the protected range: above VOVP_TH (5.55–6.0 V) the switch is open (ds p. 6, p. 15). Signals pass only up to 5.5 V (ds p. 1).

## Pins

DGS (VSSOP-10), top view: pin 1 upper left, pins 1–5 down the left side, pins 6–10 up the right side, pin 10 opposite pin 1 (ds p. 3).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | SEL1 | SEL1 | Input | Switch select 1: low COM1–NC1, high COM1–NO1; internal pull-down | ds p. 3, p. 16 |
| 2 | NO1 | NO1 | I/O | Signal path NO1 (normally open) | ds p. 3 |
| 3 | GND | GND | Ground | Ground | ds p. 3 |
| 4 | NO2 | NO2 | I/O | Signal path NO2 | ds p. 3 |
| 5 | SEL2 | SEL2 | Input | Switch select 2: low COM2–NC2, high COM2–NO2; internal pull-down | ds p. 3, p. 16 |
| 6 | COM2 | COM2 | I/O | Common signal path 2; 18 V OVP | ds p. 3 |
| 7 | NC2 | NC2 | I/O | Signal path NC2 (normally closed, not "no connect") | ds p. 3 |
| 8 | VCC | VCC | Power | Supply | ds p. 3 |
| 9 | NC1 | NC1 | I/O | Signal path NC1 (normally closed) | ds p. 3 |
| 10 | COM1 | COM1 | I/O | Common signal path 1; 18 V OVP | ds p. 3 |

Checked pad by pad against the DGS pinout and the Pin Functions table (ds p. 3); all ten agree. The package has no exposed pad (ds p. 24).

## Typical application

- 100 nF bypass capacitor as close to VCC as possible (ds p. 17, p. 18).
- SEL1 and SEL2 have internal pull-downs (6 MΩ typ), so no external resistors are needed; left low they select NC1 and NC2 (ds p. 6, p. 17).
- TI recommends tying unused signal pins to ground through 50 Ω to prevent reflections (ds p. 17).
- Function: SELx low connects COMx to NCx; SELx high connects COMx to NOx (ds p. 16). The two channels are independent, so the part can cross-switch (ds p. 17).
- Put the side that may see a fault (connector, bus) on COM1/COM2; only COM has OVP (ds p. 14).

## Layout notes

- DGS0010A land (ds p. 25): ten 1.45 × 0.3 mm pads, 0.5 mm pitch, 4.4 mm between row centres, R0.05 corners. Non-solder-mask-defined: 0.05 mm max mask opening around metal. `FP_TI_DGS0010A` matches pad size, pitch and row spacing; it uses square corners and does not encode the mask or the R0.05 radius.
- DGS0010A body (ds p. 24): 2.9–3.1 mm × 2.9–3.1 mm, 4.75–5.05 mm across the leads, 1.1 mm max height; leads 0.17–0.27 mm wide.
- Stencil: same openings, 0.125 mm stencil (ds p. 26).
- High-speed routing: equal-length traces no longer than 4 inches, few vias, no stubs, continuous reference plane, keep bypass caps away from signal traces (ds p. 18).

## Pitfalls

- `NC1`/`NC2` are the normally-closed throws, not no-connect pins (ds p. 3). Do not leave them unrouted by mistake.
- This package cannot be disabled and does not report faults: OE and FLT exist only on RUT (ds p. 3). The supply and leakage specs that need OE or FLT were not tested on DGS (ds p. 5 note 1, p. 6 note 2).
- NC/NO pins are rated only to 6 V absolute, 5.5 V operating (ds p. 4). Never put a fault-exposed node on them.
- During an 18 V COM transient NC/NO may still see up to 9.6 V (ds p. 7). OVP is not a low-voltage clamp for what sits behind the switch.
- ±50 mA continuous on each signal pin (ds p. 4). Do not route a power rail through it.
- Logic thresholds are 1.45 V / 0.5 V (ds p. 6). 1.8 V logic works; a slow or mid-rail SEL is undefined.

## Not established by the sources

The library's pin numbers, names and pin 1 location agree with ds p. 3, and `FP_TI_DGS0010A` matches the land on ds p. 25. Points that differ from or go beyond the source:

- **Behaviour of the missing OE.** The datasheet does not say how OE is tied inside the DGS package. The function table needs OE low to connect anything (ds p. 16). The DGS pinout has no OE (ds p. 3), and the supply and leakage tests that use OE were not run on DGS (ds p. 5 note 1, p. 6 note 2). That DGS is always enabled is implied, not stated.
- **OVP and overtemperature detection on DGS.** The source does not say whether DGS keeps over-temperature detection or any behaviour beyond OVP, since FLT is not bonded out (ds p. 3, p. 15).
- **Library pin requirements.** SEL1 and SEL2 are `required`, although the source says no external components are needed on the logic pins because of internal pull-downs (ds p. 17). All signal pins are `required`; the source only recommends 50 Ω to ground for unused ones (ds p. 17).
- **Datasheet inconsistencies.**
  - The revision history says the feature changed from "0-V to 18-V OVP" to "0-V to 6-V OVP on common pins" (ds p. 2). Page 1 still lists "18-V Overvoltage protection (OVP) on common pins" and "6-V overvoltage ... detection" (ds p. 1), and the ROC allows COM up to 18 V (ds p. 4).
  - Insertion loss is given at "f = TBD MHz" (ds p. 8).
  - UVLO has only a minimum of 1.65 V, no maximum (ds p. 5).
  - Absolute maximum junction temperature is labelled "Operating Junction Temperature", -65 °C to 150 °C, while the ROC gives -40 °C to 125 °C (ds p. 4).
  - The ICC-during-OVP condition names "VPOS_THLD", which is not otherwise defined; the threshold elsewhere is VOVP_TH (ds p. 5, p. 6).
- **Footprint details not encoded.** Corner radius R0.05 and the NSMD mask clearance (ds p. 25) are not in the library land. The 7.85 × 3.6 mm courtyard is a library margin; the source gives none.
- **Lead finish.** The addendum lists TMUX1072DGSR lead finish as "NIPDAUAG | SN" and TMUX1072DGSR.A as "SN" (ds p. 21). Which finish a reel carries is not established. Marking is 1A9 (ds p. 21).
- **`tmux1072-application.md`** agrees with the pages it cites. It gives the signal current limit as "+/-50 mA" (ds p. 4) and calls 9.6 V the OVP exposure (ds p. 7); both match.
- **Not stated:** a DGS-specific standby current, maximum COM capacitance at frequencies other than 240 MHz, behaviour between 0 V and the UVLO level, and RON over the 0–5.5 V range above VCC.
