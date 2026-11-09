# EZ-PD CCG6DF (CYPD6227-96BZXI) — dual-port USB Type-C / PD 3.0 controller with provider load switches

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::CCG6DF",
  "parts": ["usb::TYPEC_CCG6DF"],
  "identities": [
    {"mfr": "Infineon", "mpn": "CYPD6227-96BZXI", "lcsc": "C22453211"}
  ],
  "sources": [
    {"id": "ds", "title": "EZ-PD CCG6DF, CCG6SF USB Type-C port controller, datasheet 002-27161", "revision": "Rev. *K, 2023-08-30",
     "url": "https://www.infineon.com/assets/row/public/documents/24/49/infineon-ez-pd-ccg6df-ccg6sf-usb-type-c-port-controller-datasheet-en.pdf?fileId=8ac78c8c7d0d8da4017d0ee8e2c571be", "sha256": "9c3e7291e38d663c356b5107705f39254d95574720006f59fbf9f2b5a2c0d9ff", "local": "lib/usb/docs/infineon-ez-pd-ccg6df-ccg6sf-datasheet.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF pages. In this document they equal the printed page numbers. The URL is the one recorded in `lib/usb/docs/README.md`. The datasheet covers the CCG6SF (48-QFN) too; this note covers only the 96-ball BGA part the library declares.

## Summary

The CCG6DF is a two-port USB Type-C and USB PD 3.0 controller built around a 48 MHz Arm Cortex-M0 with 64 KB flash, 96 KB ROM and 16 KB SRAM (ds p. 1–2). It is not a TCPC: the PD stack runs as firmware on the chip (ds p. 7). Per port it integrates Rp/Rd and dead-battery Rd, VCONN FETs, a 5 V/3 A VBUS provider load switch with OVP/UVP/OCP/SCP/RCP, a consumer NFET gate driver, an SBU pass-through switch and a USB 2.0 HS 2 × 2 mux, all with 24 V-tolerant CC and SBU pins (ds p. 1–4, p. 9, p. 40). Infineon targets notebooks and Thunderbolt or non-Thunderbolt hosts (ds p. 1). Its specified ambient range ends at 55 °C (ds p. 40–41, p. 63); do not choose it for a hotter enclosure or for a passive monitor (Rd is present unpowered, ds p. 7).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| I2C address (SCB0 to EC), SWD_CLK (F10) strap open | | 0x08 | | — | "1K resistors not populated" (default) | ds p. 22 |
| I2C address, 1 kΩ from SWD_CLK to GND | | 0x40 | | — | | ds p. 22 |
| I2C address, 1 kΩ from SWD_CLK to VDDD | | 0x42 | | — | | ds p. 22 |
| SCB I2C bit rate FI2C1 | | | 1 | Mbps | | ds p. 46 |
| Power-up to ready for I2C/CC command T_PWR_RDY | | 5 | 25 | ms | | ds p. 43 |
| External reset pulse TXRES | 5 | | | µs | | ds p. 43 |
| VDDD output, VSYS powered | VSYS − 0.05 | | VSYS | V | | ds p. 41 |
| VDDD output, VBUS powered | 3 | | 3.65 | V | | ds p. 41 |
| VCCD core regulator | | 1.8 | | V | 100 nF bypass | ds p. 41 |
| Active supply current IDD12 | | 10 | | mA | VSYS = 3.3 V, CPU 24 MHz, PD port active | ds p. 41 |
| Deep Sleep current, Type-C not attached | | 200 | | µA | VSYS = 3.3 V, CC wakeup on | ds p. 42 |
| Rp current, default / 1.5 A / 3 A | 64 / 165.6 / 303.6 | | 96 / 194.4 / 356.4 | µA | | ds p. 51 |
| Rd / dead-battery Rd | 4.59 / 4.08 | | 5.61 / 6.12 | kΩ | | ds p. 51 |
| zOPEN | 108 | | | kΩ | | ds p. 51 |
| BMC vSwing | 1.05 | | 1.2 | V | | ds p. 51 |
| Provider switch RDSON (96-BGA) | | 40 | 55 | mΩ | 3 A, short pulse | ds p. 55 |
| Provider switch continuous current Isw | | | 3.0 | A | if one port sources 3 A, the other only 1.5 A | ds p. 55 |
| Provider Rsense | 4.95 | 5 | 5.05 | mΩ | external | ds p. 53 |
| OCP trip, 2–5 A contracts | | 130 ± 10 % | | % of contract | or user programmable | ds p. 53 |
| Provider over-temperature shutdown / restart | | 125 / 90 | | °C | junction | ds p. 55 |
| VCONN switch Ron | | 0.7 | 1.3 | Ω | V5V = 5 V, 215 mA | ds p. 60 |
| VCONN OCP detection | 550 | | | mA | | ds p. 60 |
| VCONN power per port | 1.5 | | | W | VCONN 4.85–5.5 V | ds p. 8 |
| Consumer gate drive GD_VGS (on) | 4.5 | | 10 | V | | ds p. 50 |
| VBUS_C_CTRL output current | 3.44 | 4.3 | 5.16 | µA | slew setting 27 | ds p. 50 |
| SBU switch Ron | | 4 / 3 | 7 / 5 | Ω | at 3.3 V / 1 V | ds p. 56 |
| DP/DM switch Ron, HS / FS | | | 8 / 12 | Ω | | ds p. 58 |
| DP/DM switch −3 dB bandwidth, HS | 700 | | | MHz | | ds p. 59 |
| GPIO pull-up / pull-down | 3.5 | 5.6 | 8.5 | kΩ | when enabled | ds p. 44 |
| GPIO VOL | | | 0.6 | V | IOL = 10 mA, 3 V VDDIO | ds p. 44 |
| ADC resolution | | 8 | | bit | | ds p. 52 |
| θJA / θJC, 96-ball BGA | | | 56.9 / 36.9 | °C/W | | ds p. 63 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VSYS relative to VSS | 6 | V | ds p. 39 |
| V5V relative to VSS | 6 | V | ds p. 39 |
| VBUS_C relative to VSS | −0.3 to 24 | V | ds p. 39 |
| VDDIO | VDDD | V | ds p. 39 |
| GPIO, DP/DM mux and SBU mux inputs | −0.5 to VDDIO + 0.5 | V | ds p. 39 |
| CC1/CC2 pins | 24 | V | ds p. 40 |
| SBU1/SBU2 pins | 24 | V | ds p. 40 |
| OVT pins ("16, 17") | −0.5 to 6 | V | ds p. 40 |
| Current per GPIO | −25 to 25 | mA | ds p. 39 |
| GPIO injection current | −0.5 to 0.5 | mA | ds p. 39 |
| Latch-up pin current | −100 to 100 | mA | ds p. 39 |
| ESD HBM: other pins / SBU / CC / VBUS_P and VBUS_C | 2200 / 1100 / 1100 / 500 | V | ds p. 39 |
| ESD CDM | 500 | V | ds p. 39 |
| Junction temperature | −40 to 125 | °C | ds p. 40 |
| Storage temperature (max) | 150 | °C | ds p. 39 |
| Reflow peak / time within 5 °C of peak | 260 / 30 | °C / s | ds p. 63 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VSYS, UFP applications | 2.75 | 5.5 | V | ds p. 41 |
| VSYS, DFP/DRP applications | 3 | 5.5 | V | ds p. 41 |
| VBUS_C, port 0/1 | 4 | 21.5 | V | ds p. 41 |
| VBUS_C maximum (far-end short to 30 V allowed under the p. 41 conditions) | — | 24 | V | ds p. 41 |
| VBUS_P, port 0/1 ("not a power pin") | 4.9 | 5.5 | V | ds p. 41 |
| V5V (VCONN supply) | 4.85 | 5.5 | V | ds p. 41 |
| VDDIO (must equal VDDD) | VDDD | VDDD | V | ds p. 41 |
| Ambient temperature TA ("internal system ambient") | −40 | 55 | °C | ds p. 40, p. 63 |
| Junction temperature TJ | — | 125 | °C | ds p. 41, p. 63 |

## Pins

Ball map from Table 2 and the Figure 4 top view (ds p. 14–17). Figure 4 has row A at the top and column 1 on the left; the 25 empty grid positions have no ball.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| B3 | CC1_P0 | CC1_P0 | Analog | Port 0 configuration channel 1 | ds p. 14, p. 17 |
| B5 | CC2_P0 | CC2_P0 | Analog | Port 0 configuration channel 2 | ds p. 14, p. 17 |
| K5 | CC1_P1 | CC1_P1 | Analog | Port 1 configuration channel 1 | ds p. 14, p. 17 |
| K3 | CC2_P1 | CC2_P1 | Analog | Port 1 configuration channel 2 | ds p. 14, p. 17 |
| K11 | DP_SYS_P0 | DP_SYS_P0 | Analog | USB 2.0 D+ from host, port 0 | ds p. 14, p. 17 |
| J11 | DM_SYS_P0 | DM_SYS_P0 | Analog | USB 2.0 D− from host, port 0 | ds p. 14, p. 17 |
| H10 | UART_TX_P0 | UART_TX_P0/P1.4 | GPIO | UART Tx from host, port 0 / GPIO | ds p. 14, p. 17 |
| G10 | UART_RX_P0 | UART_RX_P0/P1.3 | GPIO | UART Rx from host, port 0 / GPIO | ds p. 14, p. 17 |
| K10 | DP_BOT_P0 | DP_BOT_P0 | Analog | D+ to connector bottom, port 0 | ds p. 14, p. 17 |
| J10 | DM_BOT_P0 | DM_BOT_P0 | Analog | D− to connector bottom, port 0 | ds p. 14, p. 17 |
| H11 | DM_TOP_P0 | DM_TOP_P0 | Analog | D− to connector top, port 0 | ds p. 14, p. 17 |
| G11 | DP_TOP_P0 | DP_TOP_P0 | Analog | D+ to connector top, port 0 | ds p. 14, p. 17 |
| A4 | SBU2_P0 | SBU2_P0 | Analog | SBU2, connector side, port 0 | ds p. 14, p. 17 |
| A5 | SBU1_P0 | SBU1_P0 | Analog | SBU1, connector side, port 0 | ds p. 14, p. 17 |
| B6 | SBU1_SYS_P0 | SBU1_SYS_P0 | Analog | SBU1, system side, port 0 | ds p. 14, p. 17 |
| A6 | SBU2_SYS_P0 | SBU2_SYS_P0 | Analog | SBU2, system side, port 0 | ds p. 14, p. 17 |
| L6 | DP_SYS_P1 | DP_SYS_P1 | Analog | USB 2.0 D+ from host, port 1 | ds p. 14, p. 17 |
| L5 | DM_SYS_P1 | DM_SYS_P1 | Analog | USB 2.0 D− from host, port 1 | ds p. 14, p. 17 |
| L4 | UART_TX_P1 | UART_TX_P1/P0.0 | GPIO | UART Tx from host, port 1 / GPIO | ds p. 14, p. 17 |
| K6 | UART_RX_P1 | UART_RX_P1/P0.1 | GPIO | UART Rx from host, port 1 / GPIO | ds p. 14, p. 17 |
| K8 | DP_BOT_P1 | DP_BOT_P1 | Analog | D+ to connector bottom, port 1 | ds p. 14, p. 17 |
| K7 | DM_BOT_P1 | DM_BOT_P1 | Analog | D− to connector bottom, port 1 | ds p. 14, p. 17 |
| L7 | DM_TOP_P1 | DM_TOP_P1 | Analog | D− to connector top, port 1 | ds p. 14, p. 17 |
| L8 | DP_TOP_P1 | DP_TOP_P1 | Analog | D+ to connector top, port 1 | ds p. 14, p. 17 |
| A9 | SBU2_P1 | SBU2_P1 | Analog | SBU2, connector side, port 1 | ds p. 14, p. 17 |
| A8 | SBU1_P1 | SBU1_P1 (Fig. 4 prints "U1_P1") | Analog | SBU1, connector side, port 1 | ds p. 14, p. 17 |
| B7 | SBU1_SYS_P1 | SBU1_SYS_P1 | Analog | SBU1, system side, port 1 | ds p. 14, p. 17 |
| A7 | SBU2_SYS_P1 | SBU2_SYS_P1 | Analog | SBU2, system side, port 1 | ds p. 14, p. 17 |
| A3 | VBUS_C_CTRL_P0 | VBUS_C_CTRL_P0 | Analog | Gate control of port 0 consumer NFET | ds p. 15, p. 17 |
| L3 | VBUS_C_CTRL_P1 | VBUS_C_CTRL_P1 | Analog | Gate control of port 1 consumer NFET | ds p. 15, p. 17 |
| A11 | CSP_P0 | CSP_P0 | Analog | Current sense +, VBUS side of Rsense, port 0 | ds p. 15, p. 17 |
| A10 | CSN_P0 | CSN_P0 | Analog | Current sense −, port 0 | ds p. 15, p. 17 |
| L11 | CSP_P1 | CSP_P1 | Analog | Current sense +, port 1 | ds p. 15, p. 17 |
| L10 | CSN_P1 | CSN_P1 | Analog | Current sense −, port 1 | ds p. 15, p. 17 |
| K9 | SDA_SCB1 | I2C_SDA_SCB1/P0.2 | GPIO | SCB1 data to SoC/TBT controller / GPIO | ds p. 15, p. 17 |
| H7 | SCL_SCB1 | I2C_SCL_SCB1/P0.3 | GPIO | SCB1 clock / GPIO | ds p. 15, p. 17 |
| F11 | INT_EC | I2C_INT_EC/P1.2 | GPIO | EC interrupt / GPIO | ds p. 15, p. 17 |
| D10 | P2_4 | P2.4 | GPIO | GPIO | ds p. 15, p. 17 |
| L9 | INT_TBT_P0 | I2C_INT_TBT_P0/P0.4 | GPIO | Thunderbolt interrupt, port 0 / GPIO | ds p. 15, p. 17 |
| G8 | SWDIO | SWD_IO/P1.1 | GPIO | SWD I/O / GPIO | ds p. 15, p. 17 |
| F10 | SWDCLK | SWD_CLK/P1.0 | GPIO | SWD clock / GPIO; I2C address strap | ds p. 15, p. 17, p. 22 |
| F8 | HPD_P0 | HPD_P0/P2.0 | GPIO | Hot-plug detect, port 0 / GPIO | ds p. 15, p. 17 |
| E11 | SDA_SCB2 | I2C_SDA_SCB2/P2.1 | GPIO | SCB2 data (retimer/mux) / GPIO | ds p. 15, p. 17 |
| E10 | SCL_SCB2 | I2C_SCL_SCB2/P2.2 | GPIO | SCB2 clock / GPIO | ds p. 15, p. 17 |
| E8 | P2_3 | P2.3 | GPIO | GPIO | ds p. 15, p. 17 |
| H8 | INT_TBT_P1 | I2C_INT_TBT_P1/P0.5 | GPIO | Thunderbolt interrupt, port 1 / GPIO | ds p. 15, p. 17 |
| D8 | P2_5 | P2.5 | GPIO | GPIO; leave unconnected for 8/16 or 10/16 vias | ds p. 15, p. 17, p. 25 |
| D11 | SCL_SCB0 | I2C_SCL_SCB0/P4.0 | GPIO | SCB0 clock to EC / GPIO | ds p. 15, p. 17 |
| C11 | SDA_SCB0 | I2C_SDA_SCB0/P4.1 | GPIO | SCB0 data to EC / GPIO | ds p. 15, p. 17 |
| B10 | P3_0 | P3.0 | GPIO | GPIO | ds p. 15, p. 17 |
| B11 | SDA_SCB3 | I2C_SDA_SCB3/P3.1 | GPIO | SCB3 data / GPIO | ds p. 15, p. 17 |
| B9 | SCL_SCB3 | I2C_SCL_SCB3/P3.2 | GPIO | SCB3 clock / GPIO | ds p. 15, p. 17 |
| B8 | HPD_P1 | HPD_P1/P3.3 | GPIO | Hot-plug detect, port 1 / GPIO | ds p. 15, p. 17 |
| C10 | XRES | XRES | Analog | Reset input | ds p. 15, p. 17 |
| A1, A2, B1, B2, C1 | VBUS_C_P0 | VBUS_C_P0 | Power | Type-C VBUS, port 0 (4–21.5 V) | ds p. 16–17 |
| J1, K1, K2, L1, L2 | VBUS_C_P1 | VBUS_C_P1 | Power | Type-C VBUS, port 1 (4–21.5 V) | ds p. 16–17 |
| C2, D1, D2, E1, E2 | VBUS_P_P0 | VBUS_P_P0 | Power | Provider input, port 0 (4–5.5 V in Table 2) | ds p. 16–17 |
| G1, G2, H1, H2, J2 | VBUS_P_P1 | VBUS_P_P1 | Power | Provider input, port 1 (4–5.5 V in Table 2) | ds p. 16–17 |
| F4 | VSYS | VSYS | Power | System supply 2.75–5.5 V | ds p. 16–17 |
| G7 | VDDD | VDDD | Power | VDDD supply output | ds p. 16–17 |
| F7 | VDDIO | VDDIO | Power | I/O supply; short to VDDD | ds p. 16–17 |
| E7 | VCCD | VCCD | Power | 1.8 V regulator output for filter capacitor only | ds p. 16–17 |
| B4 | V5V_P0 | V5V_P0 | Power | VCONN FET supply, port 0 (4.85–5.5 V) | ds p. 16–17 |
| K4 | V5V_P1 | V5V_P1 | Power | VCONN FET supply, port 1 (4.85–5.5 V) | ds p. 16–17 |
| D7, E5, E6, F1, F2, F5, F6, G4, G5, G6 | VSSD | VSSD | Ground | Ground | ds p. 16–17 |
| D5 | DNU_D5 | DNU | — | Do not use | ds p. 17 |
| D6 | DNU_D6 | DNU | — | Do not use | ds p. 17 |
| E4 | DNU_E4 | DNU | — | Do not use | ds p. 17 |
| H4 | DNU_H4 | DNU | — | Do not use | ds p. 17 |
| H5 | DNU_H5 | DNU | — | Do not use | ds p. 17 |
| H6 | DNU_H6 | DNU | — | Do not use | ds p. 17 |

## Typical application

From Figure 6, the dual-port Thunderbolt notebook DRP diagram (ds p. 22):
- Provider path per port: 5 V supply through a 0.005 Ω Rsense, sensed by CSP/CSN, into VBUS_P, with 57 µF on the 5 V provider rail. VBUS_C goes to the connector VBUS with 10 µF 50 V. The consumer path is a back-to-back NFET pair driven by VBUS_C_CTRL, with 1 µF 35 V.
- VBUS_C_CTRL must be OFF for the controller to enter Deep Sleep; to use the consumer path in Deep Sleep, replace the back-to-back FETs with a GPIO-controlled load switch (ds p. 22, p. 42).
- CC1/CC2: 390 pF to GND on each line.
- VDDD tied to VDDIO with 1 µF and 0.1 µF; VSYS from 3.3 V with 0.1 µF 10 V and 1 µF 10 V; VCCD 0.1 µF; V5V_P0/V5V_P1 from the system 5 V with 0.1 µF. Table 6 asks for 100 nF on VCCD and 1 µF X5R on VDDD (ds p. 41).
- XRES: 4.7 kΩ to VDDD and 0.1 µF to GND.
- I2C and interrupt lines to the EC (SCB0, INT_EC) and Thunderbolt controller (SCB1, INT_TBT_P0/P1): 2.2 kΩ pull-ups to 3.3 V.
- SWD_IO: 10 kΩ pull-up to 3.3 V, routed to the Thunderbolt controller's RESET_N. SWD_CLK: optional 1 kΩ to GND or VDDD sets the I2C address (0x08 open, 0x40, 0x42).
- HPD_P0/HPD_P1: 10 kΩ pull-up to 3.3 V (DNP) and 100 kΩ pull-down.
- Shown unconnected ("X"): UART_TX/RX of both ports, SCB2, SCB3, P3.0, P2.3, P2.4, P2.5.
- The device needs PD firmware in its flash (ds p. 7); the datasheet does not supply it.
- Power: the chip runs from VSYS when present, otherwise from either port's VBUS_C (dead battery) (ds p. 9, p. 12). V5V does not power the chip, and a floating V5V does not make it malfunction (ds p. 8, p. 12).

## Layout notes

- Package PG-VFBGA-96-800 (002-10631): 6.00 × 6.00 mm, 1.00 mm max height, 0.50 mm pitch, 11 × 11 matrix with 96 balls, ball diameter 0.25–0.35 mm (0.30 nom), A1 standoff ≥ 0.16 mm (ds p. 64).
- Infineon recommends the Figure 8 "MDI" footprint: oval pads on the perimeter balls except the four corners, so inner balls can escape on the top layer without HDI (ds p. 24). Figure 9 gives a 10 mil round outer-row pad and a 14 mil round inner-row pad, with minimum vias of 8/16 or 10/16 mil (ds p. 25).
- Using 8/16 or 10/16 vias makes ball D8 (P2.5) unusable; leave it unconnected, or use 6/12 mil vias (ds p. 25). Vias larger than 10/16 mil are not feasible for inner pads (ds p. 25).
- GND: short the central GND pads with a small top-layer split plane and at least two vias, plus one via on D7 with a 7 mil trace; too few vias can raise the package top above 100 °C (ds p. 27).
- Provider pads: copper pour around the five provider balls on top and bottom with at least 7 vias; split a power plane on another layer (ds p. 27).
- Keep Rsense close to the chip, route its sense pair Kelvin-connected and length-matched, and put the provider capacitor close to Rsense (ds p. 29).
- Trace widths for 1 oz copper: CC1/CC2 8 mil (500 mA), GPIO 5 mil, DP/DM 6 mil width / 8 mil spacing at 90 Ω, VDDIO/VDDD/VCCD/VSYS/V5V 10 mil (ds p. 30).
- MSL 3; reflow peak 260 °C (ds p. 63).
- The library footprint `BGA96C50P11X11_600X600X100N` has the 96 balls of Figure 4 with Figure 8's shape pattern: 0.254 mm round corners, 0.254 × 0.356 mm ovals on the other perimeter balls, 0.356 mm round inner pads, 6.5 mm courtyard.

## Pitfalls

- Using it above 55 °C ambient: TA is −40 to 55 °C everywhere it appears, even in the absolute-maximum table (ds p. 40–41, p. 63).
- Loading two ports at 3 A: if one port sources 3 A, the other may source only 1.5 A (ds p. 55).
- Feeding VDDIO from a separate rail: VDDIO must equal VDDD (ds p. 39, p. 41) and is shorted to VDDD at board level (ds p. 12, p. 16).
- Using VDDD or VCCD to power other circuits: not recommended; VCCD cannot drive an external load (ds p. 12, p. 16).
- Forgetting the I2C address strap on SWD_CLK, or loading SWD_CLK so that the address changes (ds p. 22).
- Routing to D8 with 8/16 or 10/16 vias (ds p. 25).
- Consumer NFETs that cannot take −VBUS_MAX gate-source when off (ds p. 9, p. 50).
- Expecting Deep Sleep with VBUS_C_CTRL on (ds p. 22, p. 42).
- Reading the VBUS_P range from Table 2 (4–5.5 V, ds p. 16): Table 6 says 4.9–5.5 V (ds p. 41). FRS assumes the provider supply is above 4.85 V (ds p. 8).

## Not established by the sources

- **Every pin is `required` in the library.** Infineon's own Figure 6 leaves UART_TX/RX_P0/P1, SCB2, SCB3, P3.0, P2.3, P2.4 and P2.5 unconnected (ds p. 22), says D8 (P2.5) must stay unconnected when 8/16 or 10/16 vias are used (ds p. 25), and says a floating V5V does not cause malfunction (ds p. 8). A single-port design would also leave the port 1 pins open. The library forces all 90 signal and power pins to be connected.
- **Pad sizes of the library footprint.** Figure 8 is undimensioned (ds p. 24). Figure 9 gives only round pads: 10 mil "outer row" and 14 mil "inner row" (ds p. 25). The library's 0.356 mm (14 mil) oval length on perimeter pads is not in the source. Which balls are "outer row" (only the perimeter, as the library assumes) is not stated. In the Figure 8 image the corner pads look about the size of the inner pads, not 10 mil; the figure has no dimensions to confirm either reading.
- **VBUS_C direction.** The library declares VBUS_C_P0/P1 `power_in`. They are also the output of the integrated provider switch to the connector (ds p. 4, p. 22); Table 2 calls them "Type-C VBUS connector input" (ds p. 16).
- **VBUS_P.** The library declares VBUS_P `power_in`; Table 6 says "This is not a power pin" (ds p. 41) while Table 2 lists it in the Power group (ds p. 16). Its range is 4–5.5 V on p. 16 and 4.9–5.5 V on p. 41. Table 5 gives no absolute maximum for VBUS_P, CSP/CSN, VBUS_C_CTRL or XRES (ds p. 39–40).
- **Other library pin types.** `VSSD` is `passive`, not a ground type. `VDDD` and `VCCD` are `power_out`, though VCCD "cannot drive external load" and VDDD is not to be used to power other circuits (ds p. 12, p. 16). `VBUS_C_CTRL_P0/P1` are `output`; Table 2 calls them "Analog", "full rail control I/O" (ds p. 15). `XRES` and CSP/CSN are `input` (Table 2: "Analog"). GPIO-capable pins are `bidirectional`.
- **DNU balls.** The library comment says DNU balls are "internally unconnected"; the datasheet only labels them DNU (ds p. 17) and does not say they may be tied to a net or are internally open.
- **VDDIO not tied to VDDD in the library.** The device declares them as separate pins; the requirement to short them is only in the datasheet (ds p. 12, p. 16, p. 41).
- **Overvoltage-tolerant pins.** The absolute-maximum table names "OVT pins (16, 17)" (ds p. 40), which are QFN pin numbers (ds p. 19–20); which BGA balls are OVT is not stated. p. 2 says the SCB0 I2C pins are hot-swappable.
- **VDDD when VBUS powered.** 3.0–3.6 V in Table 2 (ds p. 16) vs 3–3.65 V in Table 6 (ds p. 41).
- **GND via guidance** speaks of "the 6 GND pads" (ds p. 27); the device has 10 VSSD balls (ds p. 16).
- **CYPD6228-96BZXI** (optimized ROM code) shares the pinout (ds p. 14, p. 62); the library does not model it.
- **Firmware, the I2C host-interface register map and the boot behaviour** are not in this datasheet.
- **Power-off isolation** of DP/DM, SBU and GPIO pins: p. 10 says the I/Os "do not draw current on power down", without numbers.
- **LCSC number** C22453211 comes from `lcsc_map.json`, not the datasheet.
