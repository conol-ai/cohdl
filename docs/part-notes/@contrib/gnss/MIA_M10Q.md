# u-blox MIA-M10Q-00B — u-blox M10 GNSS module (GPS, Galileo, GLONASS, BeiDou), UART + I²C, 53-pad M-LGA 4.5 × 4.5 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/gnss::MIA_M10Q",
  "parts": ["@contrib/gnss::GNSS_MIA_M10Q"],
  "identities": [
    {"mfr": "u-blox", "mpn": "MIA-M10Q-00B", "lcsc": "C6990279"}
  ],
  "sources": [
    {"id": "ds", "title": "MIA-M10Q Standard precision GNSS module, Data sheet", "revision": "UBX-22015849 R08, 30-Jan-2026",
     "url": "https://www.u-blox.com", "sha256": "bd8e04d2520251d70565a5187369db84b4e979fc14cf0a0b47c1adfdc2131a43", "local": "lib/@contrib/gnss/docs/mia-m10q.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal u-blox's printed "Page N of 27". The URL is the one in `lib/@contrib/gnss/docs/README.md`; it is u-blox's home page, not a document link (the PDF prints `www.u-blox.com`, ds p. 1). The library's land pattern comes from the MIA-M10Q Integration Manual UBX-21028173 R05, which the repository does not ship; this note does not cite it.

## Summary

The MIA-M10Q is a 4.5 × 4.5 mm System-in-Package GNSS receiver on the u-blox M10 platform, with an integrated LNA, SAW filter and TCXO, and concurrent GPS, GLONASS, Galileo and BeiDou reception (ds p. 4, p. 8, p. 23). It tracks at 8–15 mA from a 1.8–3.3 V supply (ds pp. 13–14) and reaches −167 dBm tracking sensitivity (ds p. 5). It has a UART (38400 baud default) and an I²C peripheral at address 0x42 (ds p. 18). VCC, V_IO and V_BCKP are separate supplies; VIO_SEL selects a 1.8 V or 3.3 V I/O range (ds p. 11, p. 13). The −00B ordering code is the professional-grade part without carrier-phase raw data (ds p. 24). The product status of every listed type number is "Initial production" (ds p. 2).

## Key parameters

25 °C, default configuration, unless stated.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Tracking current IVCC, continuous | — | 10.5 | — | mA | 3.0 V, GPS+GAL+BDS B1I (default) | ds p. 14 |
| Acquisition current IVCC | — | 12.5 | — | mA | 3.0 V, default constellations | ds p. 14 |
| Tracking current IVCC, continuous | — | 15 | — | mA | 1.8 V, default constellations | ds p. 14 |
| V_IO current, acquisition and tracking | — | 2.4 | — | mA | 3.0 V, default constellations | ds p. 14 |
| Start-up inrush | — | — | 100 | mA | Supply must deliver this | ds p. 15 |
| Hardware backup current IV_BCKP | — | 28 | — | µA | V_BCKP = 3.3 V, V_IO = VCC = 0 V | ds p. 15 |
| Sensitivity, tracking / cold start | — | −167 / −148 | — | dBm | Multi-GNSS, good external LNA | ds p. 5 |
| TTFF cold start | — | 27 | — | s | GPS+GAL+BDS B1I | ds p. 5 |
| Position accuracy | — | 1.5 | — | m CEP | Multi-GNSS, 24 h static | ds p. 5 |
| Time-pulse accuracy | — | 30 / 60 | — | ns | RMS / 99 % | ds p. 4 |
| VCC_RF output voltage | — | VCC − 0.1 | — | V | — | ds p. 13 |
| VCC_RF output current | — | — | 50 | mA | — | ds p. 13 |
| RF_IN input impedance | — | 50 | — | Ω | Built-in DC block | ds p. 13 |
| External gain at RF_IN, low-gain mode (default) | — | — | 30 | dB | — | ds p. 13 |
| Vil | — | — | 0.63 | V | Digital inputs | ds p. 13 |
| Vih | 0.68 × V_IO | — | — | V | Digital inputs | ds p. 13 |
| Vol / Voh | — / V_IO − 0.4 | — | 0.4 / — | V | ±2 mA | ds p. 13 |
| RESET_N pull-up | 7 | 10 | 13 | kΩ | — | ds p. 13 |
| Digital I/O pull-up | 8 | 18 | 40 | kΩ | VIO_SEL open | ds p. 13 |
| RESET_N low time | 1 | — | — | ms | To trigger a reset | ds p. 10 |
| UART baud rate | 9600 | 38400 | 921600 | bit/s | 38400 default, 9600 in safe boot | ds p. 16, p. 18 |
| I²C clock fSCL | 0 | — | 400 | kHz | Fast-mode only, peripheral | ds p. 16, p. 17 |
| I²C clock stretching | — | — | 20 | ms | — | ds p. 16 |
| RTC crystal ESR (optional RTC) | — | — | 100 | kΩ | 32.768 kHz | ds p. 13 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VCC | −0.3 to 3.6 | V | ds p. 12 |
| V_IO, VIO_SEL = GND | −0.3 to VCC + 0.3 (max 1.98) | V | ds p. 12 |
| V_IO, VIO_SEL = open | −0.3 to VCC + 0.3 (max 3.6) | V | ds p. 12 |
| Voltage ramp on V_IO | 25 to 35000 | µs/V | ds p. 12 |
| V_BCKP | −0.3 to 3.6 | V | ds p. 12 |
| RTC_I | −0.3 to 1.155 | V | ds p. 12 |
| RESET_N and digital pins | −0.3 to V_IO + 0.3 (max 1.98 or 3.6 per VIO_SEL) | V | ds p. 12 |
| Digital pin source / sink current | ±10 | mA | ds p. 12 |
| VCC_RF source current | 200 | mA | ds p. 12 |
| DC voltage at RF_IN | −5.5 to +5.5 | V | ds p. 12 |
| RF input power at RF_IN | 0 | dBm | ds p. 12 |
| Ambient temperature | −40 to +85 | °C | ds p. 12 |
| Storage temperature | −40 to +85 | °C | ds p. 12 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VCC (typ 1.8 or 3.3 V) | 1.76 | 3.6 | V | ds p. 12 |
| V_IO, VIO_SEL = GND (typ 1.8 V) | 1.76 | VCC (max 1.98) | V | ds p. 13 |
| V_IO, VIO_SEL = open (typ 3.3 V) | 2.7 | VCC (max 3.6) | V | ds p. 13 |
| V_BCKP | 1.65 | 3.6 | V | ds p. 13 |
| Operating temperature | −40 | 85 | °C | ds p. 13 |
| Input pin voltage | 0 | V_IO | V | ds p. 13 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | GND_A1 | GND | ground | Connect to GND | ds p. 9 |
| A2 | GND_A2 | GND | ground | Connect to GND | ds p. 9 |
| A3 | GND_A3 | GND | ground | Connect to GND | ds p. 9 |
| A4 | RTC_I | RTC_I | I | RTC input; leave open if unused | ds p. 9 |
| A5 | RTC_O | RTC_O | O | RTC output; connect to GND if unused | ds p. 9 |
| A6 | EXTINT | EXTINT | I | External interrupt (PIO5); leave open if unused | ds p. 9 |
| A7 | TIMEPULSE | TIMEPULSE | O | Time pulse (PIO4), tied to SAFEBOOT_N through 1 kΩ | ds p. 9, p. 10 |
| A8 | GND_A8 | GND | ground | Connect to GND | ds p. 9 |
| A9 | GND_A9 | GND | ground | Connect to GND | ds p. 9 |
| B1 | VCC | VCC | I | Main supply | ds p. 9 |
| B2 | GND_B2 | GND | ground | Connect to GND | ds p. 9 |
| B8 | GND_B8 | GND | ground | Connect to GND | ds p. 9 |
| B9 | RFIN | RF_IN | I | RF input | ds p. 10 |
| C1 | NC_C1 | Reserved | — | Leave open | ds p. 10 |
| C3 | GND_C3 | GND | ground | Connect to GND | ds p. 10 |
| C4 | RESETN | RESET_N | I | System reset, active low, ≥ 1 ms | ds p. 10 |
| C5 | NC_C5 | Reserved | — | Leave open | ds p. 10 |
| C6 | VCCRF | VCC_RF | O | Filtered supply for an active antenna or LNA | ds p. 10 |
| C7 | SAFEBOOTN | SAFEBOOT_N | I | Safe boot, active low; leave open if unused | ds p. 10 |
| C9 | GND_C9 | GND | ground | Connect to GND | ds p. 10 |
| D1 | SDA | SDA | I/O | I²C data (PIO2); leave open if unused | ds p. 10 |
| D2, E2 | RESERVED_D2_E2 | Reserved | — | Connect D2 to E2 | ds p. 10 |
| D9 | NC_D9 | Reserved | — | Leave open | ds p. 10 |
| E1 | SCL | SCL | I | I²C clock (PIO3); leave open if unused | ds p. 10 |
| E3 | GND_E3 | GND | ground | Connect to GND | ds p. 10 |
| E4 | GND_E4 | GND | ground | Connect to GND | ds p. 10 |
| E7 | NC_E7 | Reserved | — | Leave open | ds p. 10 |
| E9 | GND_E9 | GND | ground | Connect to GND | ds p. 10 |
| F1 | GND_F1 | GND | ground | Connect to GND | ds p. 10 |
| F3 | GND_F3 | GND | ground | Connect to GND | ds p. 10 |
| F4 | GND_F4 | GND | ground | Connect to GND | ds p. 10 |
| F7 | PIO6 | PIO6 | I/O | Digital I/O; leave open if unused | ds p. 10 |
| F9 | RESERVED_F9 | Reserved | — | Connect to GND through a 0 Ω resistor (dual-band compatibility) | ds p. 10 |
| G1 | TX | TX | O | UART TX (PIO1); leave open if unused | ds p. 10 |
| G3 | GND_G3 | GND | ground | Connect to GND | ds p. 10 |
| G4 | GND_G4 | GND | ground | Connect to GND | ds p. 10 |
| G5 | GND_G5 | GND | ground | Connect to GND | ds p. 10 |
| G6 | GND_G6 | GND | ground | Connect to GND | ds p. 10 |
| G7 | RESERVED_G7 | Reserved | — | Connect to GND through a 0 Ω resistor (crystal-variant compatibility) | ds p. 10 |
| G9 | NC_G9 | Reserved | — | Leave open | ds p. 10 |
| H1 | RX | RX | I | UART RX (PIO0); leave open if unused | ds p. 10 |
| H8 | GND_H8 | GND | ground | Connect to GND | ds p. 10 |
| H9 | LNAEN | LNA_EN | O | External LNA / active antenna on-off | ds p. 10 |
| J1 | NC_J1 | Reserved | — | Leave open | ds p. 10 |
| J2 | NC_J2 | Reserved | — | Leave open | ds p. 10 |
| J3 | NC_J3 | Reserved | — | Leave open | ds p. 10 |
| J4 | VIO | V_IO | I | I/O supply | ds p. 10 |
| J5 | VBCKP | V_BCKP | I | Backup supply; leave open if none | ds p. 10 |
| J6 | VIOSEL | VIO_SEL | I | GND = 1.8 V V_IO, open = 3.3 V V_IO | ds p. 11 |
| J7 | NC_J7 | Reserved | — | Leave open | ds p. 11 |
| J8 | GND_J8 | GND | ground | Connect to GND | ds p. 11 |
| J9 | GND_J9 | GND | ground | Connect to GND | ds p. 11 |

Top view, 9 × 9 grid at 0.5 mm pitch: rows A–J run top to bottom, columns 1–9 left to right, A1 upper left; 53 of the 81 positions are populated (Figure 2, ds p. 9). Checked position by position against the rendered Figure 2, and against the bottom view of Figure 4 (ds p. 19), which shows the same population mirrored. `FP_Ublox_MIA_M10Q_MLGA53_45x45mm` places A1 at (−2, −2) mm, columns +0.5 mm to the right and rows +0.5 mm down, with only those 53 positions; it matches.

## Typical application

- Supplies: VCC 1.76–3.6 V; V_IO in the range VIO_SEL selects; V_BCKP 1.65–3.6 V or open (ds pp. 12–13). The supply must deliver 100 mA inrush at start-up (ds p. 15).
- Power-down with a 1.8 V V_IO and V_BCKP supplied: switch V_IO off 100 ms before VCC, or send UBX-RXM-PMREQ first; with 3 V supplies both may go off together (ds p. 12).
- V_IO must ramp no faster than 25 µs/V (and no slower than 35000 µs/V); a faster ramp may damage the device (ds p. 12).
- Unused inputs may be left open; they have internal pull-ups (ds p. 16). External I²C pull-ups may be needed for 400 kHz (ds p. 16).
- SAFEBOOT_N low at start-up enters safe boot; it is tied to TIMEPULSE through 1 kΩ inside the module (ds p. 10).
- Pin states: LNA_EN is driven high in continuous mode and has a pull-down in software standby (Table 11, ds p. 11). Do not drive PIOs in hardware backup (VCC = V_IO = 0 V) (ds p. 11).
- RF: RF_IN is 50 Ω with a built-in DC block; the internal chain is notch, LNA, SAW, LNA (ds p. 8, p. 13). In default low-gain mode, external gain ahead of RF_IN must not exceed 30 dB (ds p. 13). VCC_RF (VCC − 0.1 V, 50 mA) can feed an active antenna or external LNA (ds p. 13).
- Reserved pads: connect D2 to E2; F9 and G7 to GND through 0 Ω; leave the rest open (ds p. 10).
- If RTC_O is unused, connect it to GND; leave RTC_I open (ds p. 9).

## Layout notes

- Package (Figure 4, ds p. 19): 4.5 ± 0.1 × 4.5 ± 0.1 mm, 1 mm max height, Ø 0.27 mm terminals on a 0.5 mm grid, outer terminal centres 0.25 mm from the body edge. The library pads are Ø 0.27 mm at ±2.0 mm, i.e. 0.25 mm in from a 4.5 mm body; this agrees with the drawing.
- MSL 3 (ds p. 20). Reflow per J-STD-020 (ds p. 22).
- Copper land, mask and stencil are not in this data sheet; the library takes them from the Integration Manual (see below).

## Pitfalls

- With VIO_SEL = GND, V_IO and every digital pin are limited to 1.98 V absolute (ds p. 12); do not leave VIO_SEL open on a 1.8 V host, or tie it to GND on a 3.3 V host.
- RF input power at RF_IN is limited to 0 dBm absolute (ds p. 12).
- RESET_N needs ≥ 1 ms low (ds p. 10); it has a 7–13 kΩ internal pull-up (ds p. 13).
- I²C works only in Fast-mode; Standard-mode is not supported (ds p. 16).
- The −01B part adds carrier-phase raw data; −00B does not (ds p. 24).

## Not established by the sources

The library pad positions, names and pin-1 location agree with the data sheet (Figure 2 and Table 10, ds pp. 9–11; Figure 4, ds p. 19). Gaps and disagreements:

- **Land pattern source not shipped:** the library's 0.27 mm lands, 0.05 mm mask expansion (0.37 mm NSMD opening) and paste rule cite Integration Manual UBX-21028173 R05 §4.5.1, Figure 35, recorded only by URL and SHA-256 in `lib/@contrib/gnss/docs/README.md`. The shipped data sheet gives the 0.27 mm terminal size but no land, mask or stencil recommendation (ds p. 19). They cannot be checked from the repository.
- **Source URL:** the README gives only `https://www.u-blox.com`, u-blox's home page, for the data sheet.
- **Product status:** every type number in R08, including MIA-M10Q-00B-01 and -02, is "Initial production", whose content may still be revised (ds p. 2).
- **Pin types:** the library types RTC_O `required output`, and the source says to connect it to GND if unused (ds p. 9). An ERC that forbids an output on a ground net would reject the source's own recommendation. RESERVED_D2_E2, RESERVED_F9 and RESERVED_G7 are `required passive`, which matches the source's "connect" instructions (ds p. 10).
- **Library comment:** `mia-m10q.cohdl` gives "Pin assignment p9-11", which matches, and "VCC 1.76-3.6V", which matches Table 13 (ds p. 12).
- **Not given:** thermal data, ESD ratings, and a recommended antenna matching network.
