# FUSB302B (MLP-14) — USB Type-C port controller with USB PD BMC PHY

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::FUSB302B_MLP",
  "parts": ["usb::PHY_FUSB302BMPX", "usb::PHY_FUSB302B01MPX", "usb::PHY_FUSB302B10MPX", "usb::PHY_FUSB302B11MPX"],
  "identities": [
    {"mfr": "onsemi", "mpn": "FUSB302BMPX", "lcsc": "C132291"},
    {"mfr": "onsemi", "mpn": "FUSB302B01MPX", "lcsc": "C488155"},
    {"mfr": "onsemi", "mpn": "FUSB302B10MPX", "lcsc": "C891305"},
    {"mfr": "onsemi", "mpn": "FUSB302B11MPX", "lcsc": "C891306"}
  ],
  "sources": [
    {"id": "ds", "title": "FUSB302B Programmable USB Type-C Controller w/PD (FUSB302B/D)", "revision": "Rev. 5, August 2021",
     "url": "https://www.onsemi.com/pdf/datasheet/fusb302b-d.pdf", "sha256": "712b6db97692f283b450b1f15d9018da0a3f83af35b1bdff792c0ef4ca81812f", "local": "lib/usb/docs/onsemi-fusb302b-d.pdf"}
  ],
  "status": "draft"
}
```

## Summary

The FUSB302B is an I2C-controlled USB Type-C CC controller: switchable Rp current sources and Rd pull-downs on CC1/CC2, CC/VBUS comparators with a 6-bit DAC, a VCONN switch with over-current limit, and a USB PD BMC PHY with 48-byte TX / 80-byte RX FIFOs (ds p. 6, p. 11). It supports PD 2.0 with automatic GoodCRC, retries, soft and hard reset, but the policy engine runs in host firmware (ds p. 1, p. 11–12). The four MLP variants differ only in their fixed 7-bit I2C address, 0x22–0x25, so up to four can share one bus (ds p. 18). It has no BC1.2 detection (ds p. 11). Do not choose it for a passive CC monitor: the CC pull-downs are on at reset (ds p. 20).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address, FUSB302BMPX | | 0x22 | | — | bits 7:1 = 0100010, bit 0 = R/W | ds p. 18 |
| 7-bit I2C address, FUSB302B01MPX | | 0x23 | | — | bits 7:1 = 0100011 | ds p. 18 |
| 7-bit I2C address, FUSB302B10MPX | | 0x24 | | — | bits 7:1 = 0100100 | ds p. 18 |
| 7-bit I2C address, FUSB302B11MPX | | 0x25 | | — | bits 7:1 = 0100101 | ds p. 18 |
| Device ID Product ID[1:0] (reg 0x01 bits 3:2) | | 00 / 01 / 10 / 11 | | — | BMPX / B01MPX / B10MPX / B11MPX | ds p. 19 |
| I2C SCL clock frequency fSCL | 0 | | 1000 | kHz | Fast Mode Plus | ds p. 17 |
| I2C bus capacitance Cb | | | 550 | pF | per line | ds p. 17 |
| I2C low-level input voltage VIL | | | 0.51 | V | VDD 3.0–5.5 V | ds p. 16 |
| I2C high-level input voltage VIH | 1.32 | | | V | VDD 3.0–5.5 V | ds p. 16 |
| INT_N output low VOLINTN | | | 0.4 | V | IOL = 4 mA, open drain | ds p. 16 |
| INT_N delay after global mask cleared TINT_Mask | 50 | | | µs | | ds p. 16 |
| Rd pull-down RDEVICE | 4.6 | 5.1 | 5.6 | kΩ | min/max guaranteed only when powered (Note 5) | ds p. 15 |
| SNK pull-down voltage in dead battery VUFPDB | | | 2.18 | V | under all SRC pull-up loads | ds p. 15 |
| CC resistance, disabled state zOPEN | 126 | | | kΩ | | ds p. 15 |
| Rp source current, default USB (HOST_CUR = 01) | 64 | 80 | 96 | µA | | ds p. 15 |
| Rp source current, 1.5 A (HOST_CUR = 10) | 166 | 180 | 194 | µA | | ds p. 15 |
| Rp source current, 3 A (HOST_CUR = 11) | 304 | 330 | 356 | µA | | ds p. 15 |
| VCONN switch RDSON (VCONN to CC1/CC2) | | 0.4 | 1.2 | Ω | | ds p. 15 |
| VCONN switch OCP limit | 600 | 800 | 1000 | mA | OCPreg = 0Fh (reset value) | ds p. 15 |
| VCONN soft-start, OCP disabled | | 1.5 | | ms | | ds p. 15 |
| VCONN switch thermal off / hysteresis | | 145 / 10 | | °C | | ds p. 15 |
| BC_LVL thresholds | 0.15 / 0.61 / 1.16 | 0.20 / 0.66 / 1.23 | 0.25 / 0.70 / 1.31 | V | BC = 00 / 01 / 10, PWR = 7h | ds p. 15 |
| MDAC step, CC / VBUS | | 42 / 420 | | mV | per MDAC[5:0] code | ds p. 15 |
| VBUS threshold for I_VBUSOK vVBUSthr | | | 4.0 | V | PWR[2] = 1 | ds p. 15 |
| DRP toggle tTOG1 (SNK) / tTOG2 (SRC) | 30 / 20 | 45 / 30 | 60 / 40 | ms | TOGGLE = 1 | ds p. 15 |
| BMC receiver input impedance zBmcRx | 1 | | | MΩ | | ds p. 14 |
| BMC receiver capacitance cReceiver | | 50 | | pF | driver off | ds p. 14 |
| BMC transmitter VOH | 1.05 | | 1.20 | V | | ds p. 14 |
| Disabled current Idisable | | 0.37 | 5.0 | µA | nothing attached, no I2C traffic | ds p. 16 |
| Toggle standby current Itog | | 25 | 40 | µA | TOGGLE = 1, PWR = 1h, TOG_SAVE_PWR = 01 | ds p. 16 |
| PD standby current Ipd_stby_meas | | 40 | | µA | attached, PWR = 7h, idle | ds p. 16 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD supply voltage | −0.5 to 6.0 | V | ds p. 13 |
| CC pins (host, device or DRP) | −0.5 to 6.0 | V | ds p. 13 |
| VBUS supply voltage | −0.5 to 28.0 | V | ds p. 13 |
| Storage temperature | −65 to +150 | °C | ds p. 13 |
| Junction temperature | +150 | °C | ds p. 13 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 13 |
| ESD, HBM (JS-001-2012) / CDM (JESD22-C101) | 4 / 1 | kV | ds p. 13 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VBUS (typ 5.0) | 4.0 | 21.0 | V | ds p. 13 |
| VDD (typ 3.3); 2.7 V is functional only, all electrical specs need ≥ 3.0 V (Note 3) | 2.7 | 5.5 | V | ds p. 13 |
| VCONN supply voltage | 2.7 | 5.5 | V | ds p. 13 |
| VCONN supply current | — | 560 | mA | ds p. 13 |
| Ambient temperature, the four MPX parts (+105 °C applies only to FUSB302BVMPX) | −40 | +85 | °C | ds p. 2, p. 13 |
| I2C pull-up voltage (Note 6) | 1.71 | VDD | V | ds p. 17 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CC2 | CC2 | I/O | Type-C configuration channel 2 | ds p. 5 |
| 2 | VBUS | VBUS | Input | VBUS attach/detach sense; expected OVP-protected | ds p. 5 |
| 3 | VDD | VDD | Power | Supply input | ds p. 5 |
| 4 | VDD | VDD | Power | Supply input | ds p. 5 |
| 5 | INT_N | INT_N | Open-drain output | Active-low interrupt to host | ds p. 5 |
| 6 | SCL | SCL | Input | I2C clock | ds p. 5 |
| 7 | SDA | SDA | Open-drain I/O | I2C data | ds p. 5 |
| 8 | GND | GND | Ground | Ground | ds p. 5 |
| 9 | GND | GND | Ground | Ground | ds p. 5 |
| 10 | CC1 | CC1 | I/O | Type-C configuration channel 1 | ds p. 5 |
| 11 | CC1 | CC1 | I/O | Type-C configuration channel 1 | ds p. 5 |
| 12 | VCONN | VCONN | Power switch | Regulated input switched to the correct CC pin | ds p. 5 |
| 13 | VCONN | VCONN | Power switch | Regulated input switched to the correct CC pin | ds p. 5 |
| 14 | CC2 | CC2 | I/O | Type-C configuration channel 2 | ds p. 5 |
| 15 | GND | exposed pad (unnumbered) | Ground | "Connect to GND for Thermal"; pad 15 is the library's number | ds p. 5, p. 31 |

## Typical application

- VDD: 0.1 µF (CVDD1) plus 1.0 µF (CVDD2); VCONN: 0.1 µF (CVCONN); VCONN source bulk 10–220 µF (CBULK) when the port sources VCONN (ds p. 30).
- CC1 and CC2: each gets a receiver capacitor CRECV of 200–600 pF to GND in the reference schematic (ds p. 30).
- SCL/SDA pull-ups 4.7 kΩ typ; INT_N pull-up 4.7 kΩ typ, 1.0 kΩ min; pull-up rail 1.71 V to VDD (ds p. 30, p. 17).
- VBUS pin goes to connector VBUS (ds p. 30); the pin is "expected to be an OVP protected input" (ds p. 5). The typical application powers VDD from a 3.0 V regulator (ds p. 3).
- No address or mode straps: the address is fixed per MPN (ds p. 18); role, Rp current and toggling are set over I2C (ds p. 7, p. 21–22).
- Reset defaults that matter: Switches0 = 0x03 (PDWN1/PDWN2 Rd on, no Rp, no VCONN, no measure) (ds p. 20); Control0 = 0x24 (INT_MASK = 1, all interrupts masked; HOST_CUR = 01) (ds p. 18, p. 21); Power = 0x01 (bandgap/wake only) (ds p. 23); OCPreg = 0x0F (800 mA range, max) (ds p. 24). SW_RES (reg 0x0C bit 0) restores defaults (ds p. 24).
- With no VDD, a source still sees the part as a sink (dead-battery support) (ds p. 1, p. 11).
- Interrupts: INT_N is active-low open drain (ds p. 5); Interrupt/Interrupta/Interruptb (0x42/0x3E/0x3F) are read-to-clear (ds p. 26, p. 28); per-source masks are 0x0A/0x0E/0x0F (ds p. 23–24). Read interrupts to clear them before setting TOGGLE (ds p. 7).

## Layout notes

- Solder the exposed pad to GND; the datasheet says "Connect to GND for Thermal" (ds p. 5).
- Recommended land (case 510BR): 0.30 × 0.58 mm lands at 0.50 mm pitch, opposing land-row centres 2.43 mm apart, 1.45 × 1.45 mm exposed land; body 2.50 × 2.50 mm, 0.75 ± 0.05 mm high (ds p. 31). The library footprint uses these values.
- Pin 1 is top-left in the top view; numbering runs counter-clockwise, 1–3 left, 4–7 bottom, 8–10 right, 11–14 top (ds p. 5).
- CC2 (1, 14), VDD (3, 4), GND (8, 9), CC1 (10, 11) and VCONN (12, 13) appear twice (ds p. 5); route both pads of each name to the same net.

## Pitfalls

- Using the 8-bit form (0x44/0x45 …) as the address: Table 15 bit 0 is R/W; the 7-bit addresses are 0x22–0x25 (ds p. 18).
- Expecting INT_N after power-up: INT_MASK resets to 1; clear Control0 bit 5 first (ds p. 21).
- Reading BC_LVL with the reset Power value: BC_LVL is defined only with PWR[2] = 1 and MEAS_CC1 or MEAS_CC2 = 1 (ds p. 27).
- Treating the chip as passive on CC: Rd is on from reset (ds p. 20) and the part presents as a sink even unpowered (ds p. 11). It also auto-sends GoodCRC when AUTO_CRC = 1 (ds p. 20).
- Exposing CC to VBUS: CC and VDD are rated 6.0 V absolute max, VBUS 28.0 V (ds p. 13).
- Running VDD at 2.7–3.0 V: functional only; electrical specs apply from 3.0 V (ds p. 13).
- Pulling I2C up to a rail above VDD (e.g. 5 V bus with 3.3 V VDD): pull-up must be 1.71 V–VDD (ds p. 17).
- Relying on OCP to enforce the VCONN rating: recommended max is 560 mA (ds p. 13) but OCP trips at 600–1000 mA at the reset setting (ds p. 15); lower OCPreg if needed (ds p. 24).
- Assuming 1 MHz I2C in low-power toggle: the datasheet says slow clocking in TOGGLE may not be fully 1 MHz compliant (ds p. 12).
- Assuming +105 °C: the AEC-Q100 Grade 2 range on p. 1 is the FUSB302BVMPX (Note 11); these four are −40 to +85 °C (ds p. 2, p. 19).
- Datasheet inconsistencies: I_VBUSOK is described at 4.5 V (ds p. 28) but vVBUSthr is 4.0 V max (ds p. 15); Control4 is at 0x10 in the register map (ds p. 19) but "00h" in Table 32 (ds p. 24).

## Not established by the sources

- Whether the duplicated pads (CC1, CC2, VDD, GND, VCONN) are internally connected; Figure 5 only gives them the same name (ds p. 5).
- Treatment of an unused VCONN pin (grounded, open or supplied).
- Thermal resistance (θJA/θJC), MSL rating, exposed-pad via pattern, solder mask and stencil.
- Decoupling placement distances and CC routing/ESD protection requirements; the reference schematic shows no TVS (ds p. 30).
- Power-on reset time from VDD rising to I2C ready.
- Back-powering through I2C or CC when VDD is off, beyond the dead-battery Rd behaviour.
- USB PD 3.0 support: the datasheet claims PD 2.0 (ds p. 1).
- That Figure 5, titled "FUSB302BMPX", applies to 01/10/11; Table 1 gives them the same 14-lead MLP package (ds p. 2). LCSC numbers come from the library's lcsc_map.json, not the datasheet.
