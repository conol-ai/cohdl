# DRV2605 — I²C haptic driver for ERM and LRA with effect library, 9-ball DSBGA (YZF)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/haptic::DRV2605",
  "parts": ["@contrib/haptic::HAPTIC_DRV2605"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "DRV2605YZFR", "lcsc": "C81079"}
  ],
  "sources": [
    {"id": "ds", "title": "DRV2605 Haptic Driver for ERM and LRA With Built-In Library and Smart-Loop Architecture", "revision": "SLOS825E, December 2012, revised April 2018 (addendum dated 4-Dec-2025), 65 pages",
     "url": "https://www.ti.com/lit/ds/symlink/drv2605.pdf", "sha256": "1207dc57a2b216298944cfa2df28b866fa1c9affd1f0cd5c6deb448ec5c49dc3", "local": "lib/@contrib/haptic/docs/drv2605.pdf"},
    {"id": "aw", "title": "AW86224A/B Low Power F0 Detect and Tracking LRA Haptic Driver (cited only for its address)", "revision": "V1.7, November 2021",
     "url": "https://www.awinic.com", "sha256": "96b824cf1f6467ad11531ae8ad29e66c5dc14a7aeb6d84ec396cfccab26ae096", "local": "lib/@contrib/haptic/docs/aw86224.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. For `ds` they equal TI's printed page numbers on pp. 1–58; pp. 59–65 are the orderable addendum, tape-and-reel and YZF0009 mechanical pages, which carry no matching number. For `aw` they equal the printed numbers. The URL is the one in `lib/@contrib/haptic/docs/README.md`.

## Summary

The DRV2605 drives an ERM or LRA actuator from a 2.5 V to 5.5 V supply through a differential H-bridge, controlled over I²C (ds p. 1, p. 5). It contains a licensed Immersion effect library, real-time playback over I²C, PWM or analog input on IN/TRIG, and closed-loop auto-resonance and braking (ds p. 1). Its I²C address is fixed at 0x5A, and it also answers the broadcast address 0x58 (ds p. 23, p. 24). EN must be high to read or write registers; with EN low the device still acknowledges but ignores the transfer (ds p. 19). SDA, SCL, EN and IN/TRIG are rated only to VDD + 0.3 V, so the I²C pull-ups must not go above VDD (ds p. 5, p. 23).

## Key parameters

TA = 25 °C, VDD = 3.6 V unless stated (ds p. 6).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| REG pin voltage | — | 1.84 | — | V | — | ds p. 6 |
| Digital input low current IIL | — | — | 1 | µA | VDD = 5.5 V, VI = 0 V | ds p. 6 |
| Digital input high current IIH | — | — | 3.5 | µA | VDD = 5.5 V, VI = VDD | ds p. 6 |
| SDA output low VOL | — | — | 0.4 | V | IOL = 4 mA | ds p. 6 |
| EN pull-down resistance | — | 2 | — | MΩ | — | ds p. 6 |
| Shutdown current | — | 1.75 | 4 | µA | VEN = 0 V | ds p. 6 |
| Standby current | — | 1.9 | 5 | µA | VEN = 1.8 V, STANDBY = 1 | ds p. 6 |
| Quiescent current | — | 0.6 | 1 | mA | VEN = 1.8 V, STANDBY = 0, no signal | ds p. 6 |
| IN/TRIG input impedance | — | 100 | — | kΩ | To V(CM_ANA) | ds p. 6 |
| IN/TRIG common-mode voltage | — | 0.9 | — | V | AC_COUPLE = 1 | ds p. 6 |
| Output impedance in shutdown | — | 15 | — | kΩ | OUT+ / OUT− to GND | ds p. 6 |
| Load impedance threshold for over-current | — | 4 | — | Ω | — | ds p. 6 |
| Average battery current | — | 2.5 | 3.25 | mA | 90 % duty, LRA or ERM, no load | ds p. 6 |
| Start-up time | — | 0.7 | — | ms | GO bit or trigger to output | ds p. 6 |
| Start-up time | — | 1.5 | — | ms | EN high to output (PWM/analog modes) | ds p. 6 |
| PWM output frequency | 19.5 | 20.5 | 21.5 | kHz | — | ds p. 6 |
| SCL frequency | — | — | 400 | kHz | No wait states | ds p. 6 |
| Thermal resistance RθJA | — | 145.2 | — | °C/W | YZF (DSBGA) | ds p. 5 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | −0.3 to 6 | V | ds p. 5 |
| EN, SDA, SCL, IN/TRIG | −0.3 to VDD + 0.3 | V | ds p. 5 |
| Operating free-air temperature TA | −40 to 85 | °C | ds p. 5 |
| Operating junction temperature TJ | −40 to 150 | °C | ds p. 5 |
| Storage temperature | −65 to 150 | °C | ds p. 5 |
| ESD, HBM (JS-001, all pins) | ±2000 | V | ds p. 5 |
| ESD, CDM (JESD22-C101, all pins) | ±500 | V | ds p. 5 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD | 2.5 | 5.5 | V | ds p. 5 |
| PWM input frequency (IN/TRIG) | 10 | 250 | kHz | ds p. 5 |
| Load impedance ZL at VDD = 5.2 V | 8 | — | Ω | ds p. 5 |
| Digital input low VIL (EN, IN/TRIG, SDA, SCL) | — | 0.5 | V | ds p. 5 |
| Digital input high VIH (EN, IN/TRIG, SDA, SCL) | 1.3 | — | V | ds p. 5 |
| Analog input voltage (IN/TRIG) | 0 | 1.8 | V | ds p. 5 |
| LRA frequency range | 125 | 300 | Hz | ds p. 5 |

The PWM frequency, load impedance and LRA range are "ensured by design, not production tested" (ds p. 5).

## Pins

YZF package, top view: row A along the top, column 1 at the left (ds p. 4).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | EN | EN | input | Device enable, active high; 2 MΩ internal pull-down | ds p. 4, p. 6, p. 19 |
| A2 | REG | REG | output | 1.8 V regulator output; 1 µF required | ds p. 4 |
| A3 | OUTP | OUT+ | output | Positive driver output | ds p. 4 |
| B1 | IN_TRIG | IN/TRIG | input | PWM, analog or trigger input; connect to GND if unused | ds p. 4 |
| B2 | SDA | SDA | I/O | I²C data | ds p. 4 |
| B3 | GND | GND | power | Supply ground | ds p. 4 |
| C1 | SCL | SCL | input | I²C clock | ds p. 4 |
| C2 | VDD | VDD | power | Supply 2.5–5.5 V | ds p. 4 |
| C3 | OUTN | OUT− | output | Negative driver output | ds p. 4 |

Checked by hand against the rendered pin configuration (ds p. 4) and the YZF0009 land example (ds p. 63): `BGA9C50P3X3_145X145X50N` puts A1 at (−0.5, −0.5) mm, row A along the top and column 1 on the left, which is the top view on both pages.

## Typical application

- Table 30 (ds p. 49): C(VDD) = 1 µF, C(REG) = 1 µF, optional C(IN) = 1 µF AC coupling, R(PU) = 2.2 kΩ. Figure 56 uses the same values (ds p. 50).
- Do not power external loads from REG (ds p. 49).
- I²C: address 0x5A (7-bit; 0xB4 write, 0xB5 read) (ds p. 23, p. 24). Broadcast address 0x58 for multi-driver GO (ds p. 23). Pull-ups 660 Ω to 4.7 kΩ; SDA and SCL must not exceed VDD (ds p. 23).
- EN: high to operate and to access registers; low is shutdown with registers kept (ds p. 19). DEV_RESET in register 0x01 resets to power-up state (ds p. 19).
- STANDBY (register 0x01) is set by default; clear it before playback (ds p. 19).
- IN/TRIG: tie to GND when unused (ds p. 4).
- Actuator drive levels: RATED_VOLTAGE (0x16) and OD_CLAMP (0x17) set the steady-state and peak voltages through Equations 2–7; write them before auto-calibration (ds p. 22–23).

## Layout notes

- YZF0009 land example (ds p. 63): nine Φ 0.245 mm lands, 0.5 mm pitch, NSMD preferred. `BGA9C50P3X3_145X145X50N` uses Φ 0.245 mm at ±0.5 mm. It matches p. 63.
- Package (ds p. 62): D and E 1.41–1.47 mm, 0.625 mm maximum height, ball Φ 0.25–0.35 mm. Paste example: 0.25 mm apertures with R0.05 corners for a 0.1 mm stencil (ds p. 64).
- Table 32 (ds p. 55) gives a different NSMD land: 275 µm copper, 375 µm mask opening, 275 µm square stencil, 125 µm stencil. See "Not established".
- Traces at the lands 75–100 µm wide, symmetric, until escaped (ds p. 55, p. 56).
- VDD and REG capacitors close to their pins; GND ball via to the ground plane (ds p. 55, p. 56 Figure 62).

## Pitfalls

- **Broadcast address 0x58.** The DRV2605 responds to 0x58 as well as 0x5A (ds p. 23). The AW86224 in the same library package uses 0x58 as its fixed address (aw p. 20). The two cannot share a bus.
- Pull-ups to a rail above VDD violate the absolute maximum (VDD + 0.3 V) and TI's instruction (ds p. 5, p. 23). With a 1.8 V host and a 3.6 V battery-powered DRV2605 this is fine; with a 3.3 V host and VDD = 2.5 V it is not.
- With EN low the DRV2605 still ACKs its address but ignores reads and writes (ds p. 19). A probe that only checks for ACK does not prove the device is enabled.
- Load: ZL ≥ 8 Ω at VDD = 5.2 V (ds p. 5); over-current trips near 4 Ω (ds p. 6).
- LRA resonant frequency must lie within 125–300 Hz (ds p. 5).
- Leaving IN/TRIG floating is against the pin description (ds p. 4).

## Not established by the sources

The library's ball names and positions agree with the source (ds p. 4, p. 63). Disagreements and gaps:

- **VDD capacitor.** The library comment says "0.1uF cap required (p4)", which is what the p. 4 pin table says. Revision E changed C(VDD) to 1 µF in Table 30 and Figure 56 (ds p. 2, p. 49, p. 50). The datasheet itself is inconsistent; the newer application value is 1 µF.
- **Two land sizes in one datasheet.** The library follows the YZF0009 example board layout, Φ 0.245 mm (ds p. 63). Table 32 in the Layout section gives a 275 µm NSMD copper pad (ds p. 55). The source does not say which governs.
- **IN_TRIG `optional`.** The source says to connect IN/TRIG to GND if unused (ds p. 4). The library marks it `optional`, so a design can leave it floating without a warning; the comment says to tie it to GND.
- **SDA output type.** The source types SDA as I/O and relies on a wired-AND bus (ds p. 4, p. 23) but does not call it open-drain. The library types it `bidirectional`.
- **Body size.** The device information table gives 1.50 × 1.50 mm maximum (ds p. 1); the YZF0009 drawing gives 1.41–1.47 mm (ds p. 62). The library comment uses the drawing.
- Not given: limits over temperature for most parameters (EC at 25 °C, ds p. 6), and the thermal rating of the 9-ball package beyond the thermal-metric table (ds p. 5).
