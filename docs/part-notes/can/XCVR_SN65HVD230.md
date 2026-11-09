# SN65HVD230DR — 3.3 V CAN transceiver (ISO 11898-2) with standby and slope control, SOIC-8

## Identity

```json
{
  "schema": "part-note/1",
  "device": "can::CAN_SN65HVD230",
  "parts": ["can::XCVR_SN65HVD230"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "SN65HVD230DR", "lcsc": "C12084"},
    {"mfr": "Texas Instruments", "mpn": "SN65HVD230D", "lcsc": "C1543812"}
  ],
  "sources": [
    {"id": "ds", "title": "SN65HVD23x 3.3-V CAN Bus Transceivers", "revision": "SLOS346O, March 2001, revised April 2018 (package addendum 2026-03-05)",
     "url": "https://www.ti.com/lit/ds/symlink/sn65hvd230.pdf", "sha256": "e98fc0c59168c035d7958111924dae7e21e46ab45bdfd6400aef048299d6d716", "local": "lib/can/docs/sn65hvd230.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. For pp. 1–35 they equal TI's printed page numbers; pp. 36–42
are the package addendum, materials and outline pages.

## Summary

CAN physical-layer transceiver for a single 3.0 V to 3.6 V supply, ISO 11898-2 compatible, up to
1 Mbps (ds p. 1, p. 6). The RS pin selects high-speed, slope-control (10 kΩ to 100 kΩ to GND) or
a 370 µA typ listen-only standby mode (ds p. 1, p. 5, p. 23). It interoperates with 5 V
transceivers on the same bus (ds p. 31) and allows up to 120 nodes (ds p. 1, p. 28). Bus pins are
rated only –4 V to 16 V (ds p. 5), and the operating range is –40 °C to 85 °C (ds p. 6); check
both before using it where the bus can be shorted to a battery rail or run hot.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Data rate | — | — | 1 | Mbps | — | ds p. 1 |
| Dominant output CANH VOH | 2.45 | — | VCC | V | VI = 0 V | ds p. 7 |
| Dominant output CANL VOH | 0.5 | — | 1.25 | V | VI = 0 V | ds p. 7 |
| Recessive output CANH, CANL | — | 2.3 | — | V | VI = 3 V | ds p. 7 |
| Differential output, dominant VOD(D) | 1.5 | 2 | 3 | V | VI = 0 V, Figure 18 (60 Ω) | ds p. 7 |
| Differential output, dominant VOD(D) | 1.2 | 2 | 3 | V | VI = 0 V, Figure 19 (common-mode network) | ds p. 7 |
| Differential output, recessive VOD(R) | –120 | 0 | 12 | mV | VI = 3 V, Figure 18 | ds p. 7 |
| Short-circuit output current IOS | –250 | — | 250 | mA | VCANH = –2 V or VCANL = 7 V | ds p. 7 |
| Supply current, standby | — | 370 | 600 | µA | V(Rs) = VCC | ds p. 7 |
| Supply current, dominant / recessive | — | 10 | 17 | mA | No load | ds p. 7 |
| Receiver threshold VIT+ | — | 750 | 900 | mV | Table 1 | ds p. 7 |
| Receiver threshold VIT– | 500 | 650 | — | mV | Table 1 | ds p. 7 |
| Receiver hysteresis | — | 100 | — | mV | — | ds p. 7 |
| Receiver output VOH (R) | 2.4 | — | — | V | IO = –8 mA | ds p. 7 |
| Receiver output VOL (R) | — | — | 0.4 | V | IO = 8 mA | ds p. 7 |
| Differential input resistance RDiff | 40 | 70 | 100 | kΩ | Pin-to-pin, V(D) = 3 V | ds p. 7 |
| CANH, CANL input resistance RI | 20 | 35 | 50 | kΩ | — | ds p. 7 |
| CANH/CANL input capacitance CI / differential CDiff | — | 32 / 16 | — | pF | V(D) = 3 V | ds p. 7 |
| Driver propagation tPLH / tPHL | — | 35 / 70 | 85 / 120 | ns | V(Rs) = 0 V, CL = 50 pF | ds p. 8 |
| Driver rise / fall time | 25 / 40 | 50 / 55 | 100 / 80 | ns | V(Rs) = 0 V | ds p. 8 |
| Driver rise / fall time | 600 / 600 | 800 / 825 | 1200 / 1000 | ns | RS 100 kΩ to GND | ds p. 8 |
| Receiver propagation tPLH, tPHL | — | 35 | 50 | ns | Figure 23 | ds p. 8 |
| Loop delay recessive→dominant t(LOOP1) | — | 70 | 115 | ns | V(Rs) = 0 V | ds p. 8 |
| Loop delay dominant→recessive t(LOOP2) | — | 100 | 135 | ns | V(Rs) = 0 V | ds p. 8 |
| Wake-up time from standby t(WAKE) | — | 0.55 | 1.5 | µs | via RS | ds p. 9 |
| Vref output | 0.45 VCC | — | 0.55 VCC | V | –5 µA < I(Vref) < 5 µA | ds p. 9 |
| Vref output | 0.4 VCC | — | 0.6 VCC | V | –50 µA < I(Vref) < 50 µA | ds p. 9 |
| RS input current, high-speed | –450 | — | 0 | µA | V(Rs) < 1 V | ds p. 9 |
| Thermal resistance RθJA (D package) | — | 76.8 | — | °C/W | SN65HVD230 | ds p. 6 |
| Thermal shutdown / hysteresis | — | 165 / 10 | — | °C | Listed in the ROC table | ds p. 6 |
| ESD HBM, CANH, CANL, GND / all pins | — | ±16000 / ±4000 | — | V | JS-001 | ds p. 6 |
| ESD CDM | — | ±1000 | — | V | JESD22-C101 | ds p. 6 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | –0.3 to 6 | V | ds p. 5 |
| Voltage at CANH or CANL | –4 to 16 | V | ds p. 5 |
| Transient pulse, CANH and CANL, through 100 Ω (Figure 24) | –25 to 25 | V | ds p. 5 |
| Digital input/output voltage VI (D or R) | –0.5 to VCC + 0.5 | V | ds p. 5 |
| Receiver output current IO | –11 to 11 | mA | ds p. 5 |
| Storage temperature Tstg | –40 to 85 | °C | ds p. 5 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 3 | 3.6 | V | ds p. 6 |
| Bus voltage, common mode VIC | –2 | 7 | V | ds p. 6 |
| Bus voltage, each terminal VI | –2.5 | 7.5 | V | ds p. 6 |
| High-level input voltage VIH (D, R) | 2 | — | V | ds p. 6 |
| Low-level input voltage VIL (D, R) | — | 0.8 | V | ds p. 6 |
| Differential input voltage VID | –6 | 6 | V | ds p. 6 |
| Input voltage V(Rs) | 0 | VCC | V | ds p. 6 |
| V(Rs) for standby | 0.75 VCC | VCC | V | ds p. 6 |
| Wave-shaping resistance Rs | 0 | 100 | kΩ | ds p. 6 |
| High-level output current IOH, driver / receiver | –40 / –8 | — | mA | ds p. 6 |
| Low-level output current IOL, driver / receiver | — | 48 / 8 | mA | ds p. 6 |
| Operating free-air temperature TA | –40 | 85 | °C | ds p. 6 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | D | D (TXD) | Input | Driver input; low = dominant, high = recessive | ds p. 5 |
| 2 | GND | GND | Ground | Ground | ds p. 5 |
| 3 | VCC | VCC | Power | 3.3 V supply | ds p. 5 |
| 4 | R | R (RXD) | Output | Receiver output; low = dominant, high = recessive | ds p. 5 |
| 5 | VREF | Vref | Output | VCC/2 reference; may float if unused | ds p. 5, p. 20 |
| 6 | CANL | CANL | I/O | Low-level CAN bus line | ds p. 5 |
| 7 | CANH | CANH | I/O | High-level CAN bus line | ds p. 5 |
| 8 | RS | RS | Input | Mode select: strong pull-down = high speed, 10–100 kΩ to GND = slope control, high = standby | ds p. 5, p. 23 |

## Typical application

- D to the CAN controller TXD, R to its RXD; CANH/CANL to a 120 Ω twisted pair terminated with
  120 Ω at each end (ds p. 26). Split termination (two 60 Ω with a capacitor to GND) is an option;
  Vref can feed its midpoint (ds p. 20, p. 27, p. 33).
- Decouple VCC with 100 nF ceramic as close to VCC and GND as possible (ds p. 32).
- RS: tie to GND for high speed, or 10 kΩ (~15 V/µs) to 100 kΩ (~2 V/µs) to GND for slope control
  (ds p. 21). For standby, drive RS from a GPIO: < 1.2 V for active, > 0.75 VCC for standby
  (ds p. 20, p. 23).
- If an open-drain host drives D, add a 1 kΩ to 10 kΩ pull-up (ds p. 33). Internal biasing of
  floating pins is weak; use 1 kΩ to 10 kΩ external pull-up/down against transients (ds p. 33).
- Vref unused: leave it floating (ds p. 20, p. 33).

## Layout notes

- TI D0008A example land (ds p. 41): 8 pads 1.55 x 0.6 mm, 1.27 mm pitch, 5.4 mm between row
  centres. The library `soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` uses KiCad's 1.95 x 0.6 mm pads at
  ±2.475 mm instead; TI notes IPC-7351 may give alternate designs (ds p. 41).
- Package: SOIC-8, 4.90 x 3.91 mm nominal body, 1.75 mm max height, JEDEC MS-012 AA (ds p. 1,
  p. 40).
- Put TVS and bus filter capacitors at the connector, in the signal path; use VCC/GND planes and
  at least two vias per bypass/protection ground (ds p. 33).

## Pitfalls

- RS left floating is not a defined mode: the internal bias is weak (ds p. 33), and the mode
  table defines only < 1.2 V and > 0.75 VCC (ds p. 23).
- Standby is listen-only: the driver is off, the receiver still mirrors the bus; the host must
  pull RS low to transmit again (ds p. 22).
- Slope control lengthens loop delay (70 ns typ at 0 Ω, 535 ns at 100 kΩ), which shortens the
  allowed bus length (ds p. 8, p. 28).
- Bus pins tolerate only –4 V to 16 V (ds p. 5); a short to a higher supply exceeds that
  (inference). TI recommends external TVS protection at the connector for system-level ESD,
  EFT and surge (ds p. 29, p. 33).
- Storage temperature max is 85 °C (ds p. 5), lower than usual for an IC.
- SN65HVD231 and SN65HVD232 share the pinout but not the function: on 231, RS high is sleep
  (receiver off); on 232, pins 5 and 8 are NC (ds p. 4, p. 5).
- Thermal shutdown disables the driver; the receiver keeps working (ds p. 20).

## Not established by the sources

- **Alternate MPN is obsolete.** The library lists `SN65HVD230D` as an alternate; the package
  addendum marks SN65HVD230D "Obsolete", with SN65HVD230DR (and DR.A, DRG4) "Active" (ds p. 36).
- Maximum junction temperature: not given; only thermal shutdown (165 °C typ) and RθJA (ds p. 6).
- The ROC table lists VIH/VIL for "D, R", although R is an output (ds p. 6, p. 5).
- Supply current at 1 Mbps into 60 Ω with RS at 0 V is shown only as a curve (Figure 1, ds p. 10).
- RS levels between 1.2 V and 0.75 VCC: the SN65HVD230 mode table defines only < 1.2 V and
  > 0.75 VCC (ds p. 23). The library comment's "high is low-power standby" means > 0.75 VCC.
- Land pattern: the library uses KiCad's SOIC-8 land, not TI's example land (ds p. 41); pin map,
  pin 1 position and pin names agree with the datasheet (ds p. 5).
- Page 1 says bus ESD "exceeds ±16 kV HBM"; the ESD table gives ±16000 V for CANH, CANL and GND
  only (ds p. 1, p. 6).
- The local PDF carries a package addendum dated 2026-03-05 and materials pages dated 2025-12-23
  (ds p. 36, p. 38), newer than the April 2018 datasheet body; a re-download may differ.
