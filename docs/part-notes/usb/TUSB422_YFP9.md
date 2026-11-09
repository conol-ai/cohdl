# TUSB422 (9-ball DSBGA, YFP) — USB Type-C port controller (TCPC) with USB PD PHY

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::TUSB422_YFP9",
  "parts": ["usb::TCPC_TUSB422IYFPR", "usb::TCPC_TUSB422IYFPT"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TUSB422IYFPR", "lcsc": "C702378"},
    {"mfr": "Texas Instruments", "mpn": "TUSB422IYFPT", "lcsc": "C2674151"}
  ],
  "sources": [
    {"id": "ds", "title": "TUSB422 USB Type-C Port Control with Power Delivery (SLLSEW6C)", "revision": "Rev. C, June 2018; orderable addendum dated 10-Nov-2025",
     "url": "https://www.ti.com/lit/ds/symlink/tusb422.pdf", "sha256": "36c16474811dad1036722e164b7b4647aba669acc966fb342693ac41abc9c677", "local": "lib/usb/docs/ti-tusb422.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages (1-based). They equal TI's printed page numbers for pp. 1–70. On the appended pages the PDF and printed numbers differ:
- p. 71 is printed "Addendum-Page 1".
- pp. 72–73 are printed "Pack Materials-Page 1–2".

## Summary

The TUSB422 is a USB Type-C port controller (TCPC). An external processor running the TCPM controls it over I2C (ds p. 1, p. 11). It has the Type-C CC logic (Rp/Rd/Ra, DFP/UFP/autonomous DRP), a USB PD BMC PHY and protocol layer with automatic GoodCRC, VBUS measurement and internal discharge, and a VCONN switch (rated 2.5 W on p. 1; the example in §8.2 sizes it for 1 W) (ds p. 1, p. 11–16, p. 63–64). It has no GPIO and cannot drive a VBUS power switch. Source and sink path control stays with the host (ds p. 14). Its 7-bit address is fixed at 0x20, so only one fits on a bus (ds p. 19). It does not handle the PD 3.0 Get_Source_Cap_Extended message, and it supports Fast Role Swap TX but not RX (ds p. 12, p. 16).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address | | 0x20 | | — | bits 7:1 = 0100000, bit 0 = W/R; fixed | ds p. 19 |
| I2C SCL frequency fSCL | 0.001 | | 1 | MHz | | ds p. 8 |
| I2C VIH / VIL | 1.2 / — | | — / 0.4 | V | VDD must be above 3 V for 3.3 V I2C levels | ds p. 7 |
| I2C VOL / IOL | — / 6 | | 0.4 / — | V / mA | open drain | ds p. 7 |
| I2C bus capacitance, FM and FM+ | | | 150 | pF | | ds p. 7 |
| SDA/SCL pull-up, FM+ (1 MHz) | 620 | 820 | 910 | Ω | 150 pF bus | ds p. 7 |
| SDA/SCL pull-up, FM (400 kHz) | 620 | 1500 | 2200 | Ω | 150 pF bus | ds p. 7 |
| INT_N VOL | | | 0.4 | V | IOL = −2 mA | ds p. 7 |
| INT_N low after VDD(min) tINT_N_LOW | | | 4 | ms | marks end of initialization | ds p. 8, p. 65 |
| VDD rise time tVDD_RISE | | | 40 | ms | 0 V to VDD(min) | ds p. 8 |
| Current, unattached UFP / DRP / DFP | | 10 / 12 / 11 | | µA | VDD = 3.7 V | ds p. 5 |
| Current, attached sink, PD off / BIST carrier | | 330 µA / 5.2 mA | | — | VDD = 3.7 V | ds p. 5 |
| Rd R(CC_RD) | 4.6 | 5.1 | 5.6 | kΩ | VDD 2.7–5.5 V | ds p. 5 |
| Ra R(CC_RA) | 0.8 | 1 | 1.2 | kΩ | | ds p. 5 |
| CC leakage with VDD = 0 V ICC(LKG) | | | 1.36 | mA | VCONN = 0 V, CC = 5.5 V | ds p. 5 |
| Rp current default / 1.5 A / 3 A | 64 / 166 / 304 | 80 / 180 / 330 | 96 / 194 / 356 | µA | 3 A needs VDD > 3.0 V | ds p. 6, p. 14 |
| BMC TX output impedance RTX(PD) | 33 | 48 | 75 | Ω | 750 kHz | ds p. 6 |
| BMC TX VOH | 1.05 | 1.125 | 1.2 | V | | ds p. 6 |
| BMC RX input impedance RRX(PD) | 1 | | | MΩ | TX Hi-Z | ds p. 6 |
| CC shunt capacitance CRX(SHUNT) | 200 | | 450 | pF | external, on each of CC1 and CC2 | ds p. 3, p. 7 |
| PD bit rate | 270 | 300 | 330 | kbps | | ds p. 8 |
| VCONN switch RDS(ON) | | 0.4 | 0.75 | Ω | | ds p. 7 |
| VCONN current limit I(VCONN) | 500 | 650 | 850 | mA | switch opens above this | ds p. 7 |
| VCONN bulk capacitance C(VCONN) | 10 | | 200 | µF | on the VCONN supply | ds p. 7 |
| VCONN discharge resistance | 4.6 | 5.1 | 5.6 | kΩ | | ds p. 7 |
| VBUS source bulk capacitance C(BULK_SRC) | 10 | | 150 | µF | above this use external discharge | ds p. 7, p. 15 |
| VBUS sink bulk capacitance at connector / after PD | 1 / 1 | | 10 / 100 | µF | C(SNK) / C(SNKPD) | ds p. 7 |
| VBUS bleed discharge resistance | 8 | 10 | 12.5 | kΩ | | ds p. 7 |
| VBUS measurement accuracy | −2 | | 2 | % | 25 mV LSB | ds p. 7, p. 24 |
| Over-temperature trip T(OTSD1) | | 150 | | °C | opens VCONN | ds p. 7 |
| Thermal RθJA / RθJB | | 114.3 / 24.9 | | °C/W | table headed "YFF" | ds p. 5 |
| ROLE_CONTROL reset value | | 0x0A | | — | Rd on CC1 and CC2 | ds p. 22, p. 37 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | −0.3 to 6 | V | ds p. 4 |
| VCONN | −0.3 to 6 | V | ds p. 4 |
| INT_N, SDA, SCL | −0.3 to 6 | V | ds p. 4 |
| CC1, CC2 | −0.3 to 6 | V | ds p. 4 |
| VBUSIN | −0.3 to 26 | V | ds p. 4 |
| Storage temperature | −65 to 150 | °C | ds p. 4 |
| ESD HBM / CDM, all pins | ±1500 / ±1500 | V | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD (nom 3.7) | 2.7 | 5.5 | V | ds p. 4 |
| VCONN (nom 5) | 2.7 | 5.5 | V | ds p. 4 |
| VBUSIN (nom 5) | 0 | 24 | V | ds p. 4 |
| I2C pull-up rail VI2C_SYS (nom 1.8) | 1.65 | 3.6 | V | ds p. 4 |
| TA, VCONN supported in the system | −40 | 85 | °C | ds p. 4 |
| TA, VCONN not supported (but see the 85 °C orderable rating) | −40 | 105 | °C | ds p. 4, p. 71 |
| Junction temperature TJ | −40 | 125 | °C | ds p. 4 |

## Pins

Direction gives the datasheet I/O type (ds p. 3) and, in brackets, the library's pin kind. Ball positions are as viewed from the top (ds p. 3).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | CC2 | CC2 | I/O (FS) [bidirectional] | Type-C CC2; needs external CRX(SHUNT) | ds p. 3 |
| A2 | VBUSIN | VBUSIN | I [power_in] | 5–24 V VBUS input; tie directly to VBUS at the connector | ds p. 3 |
| A3 | VDD | VDD | P [power_in] | Supply 2.7–5.5 V | ds p. 3 |
| B1 | VCONN | VCONN | P [power_in, optional] | VCONN supply 2.7–5.5 V; may float if VCONN is not needed | ds p. 3 |
| B2 | INT_N | INT_N | O (FS), open drain [output, optional] | Active-low interrupt; external pull-up | ds p. 3 |
| B3 | SCL | SCL | I/O open drain (FS) [bidirectional] | I2C clock; external pull-up | ds p. 3 |
| C1 | CC1 | CC1 | I/O (FS) [bidirectional] | Type-C CC1; needs external CRX(SHUNT) | ds p. 3 |
| C2 | GND | GND | G [power_in] | Ground | ds p. 3 |
| C3 | SDA | SDA | I/O open drain (FS) [bidirectional] | I2C data; external pull-up | ds p. 3 |

## Typical application

- Figure 76 and Table 73 (DRP phone) give the example values (ds p. 63):
  - VDD from VBAT with 0.1 µF.
  - 220 pF CRX_SHUNT on each CC.
  - C(VCONN) 10 µF.
  - SDA/SCL pulled up to 1.8 V with 1.5 kΩ (150 pF bus, 400 kHz).
  - INT_N pulled up with 200 kΩ.
  - VBUS source path (boost with VSAFE5V_EN) and sink path (charger) are controlled by the SoC.
- INT_N may be pulled up to 1.8 V, 3.3 V or VDD, normally the same rail as SDA/SCL (ds p. 64).
- Start-up (ds p. 13, p. 65):
  1. Ramp VDD.
  2. Wait for INT_N low (≤ 4 ms).
  3. Read ALERT (0x10) and check POWER_STATUS.TCPC_INIT_STATUS is clear.
  4. Clear the alert.
  5. Write ROLE_CONTROL: 0x05 for DFP (0x15/0x25 for 1.5/3 A), 0x0A for UFP, or 0x4A for autonomous DRP.
  6. Issue Look4Connection.
- Alert interrupts are unmasked at reset: in ALERT_MASK (0x12 = 0xFF, 0x13 = 0x0F) a 1 means unmasked. Vendor interrupts are masked by default (ds p. 16, p. 22, p. 31).
- VBUS voltage monitoring and voltage alarms are disabled at reset (POWER_CONTROL = 0x60) (ds p. 22, p. 38). Auto-discharge on disconnect is off by default (ds p. 38).
- VCONN: make sure the VCONN supply is valid before setting ENABLE_VCONN, and keep it valid until the switch has opened (ds p. 3, p. 16).
  - To deliver 1 W at 3 V on CC, VCONN must exceed 3.25 V.
  - To deliver 1 W at 4.75 V, VCONN must exceed 4.9 V (ds p. 64).
- Dedicated source (DFP-only) designs: add a blocking diode in the source path, because the part's dead-battery Rd could otherwise accept VBUS from another source (ds p. 13).

## Layout notes

- Land pattern YFP0009-C01 (ds p. 69):
  - Nine 0.23 mm diameter copper lands on 0.4 mm pitch.
  - Non-solder-mask-defined (preferred), with the mask opening up to 0.05 mm larger than the copper.
  - The library uses 0.23 mm circular copper at ±0.4 mm.
- Stencil example: 0.25 mm square apertures with R0.05 corners, 0.1 mm stencil (ds p. 70).
- Package: 1.35–1.41 × 1.305–1.365 mm, 0.5 mm maximum height, ball diameter 0.21–0.25 mm. The ball A1 corner is marked on the top outline (ds p. 68).
- Ball map, top view: row A = CC2, VBUSIN, VDD; row B = VCONN, INT_N, SCL; row C = CC1, GND, SDA (ds p. 3). The library places row A at the top and column 1 at the left, matching the top view and the land example (ds p. 69).
- B2 (INT_N) is the inner ball. Figure 78 routes it with via-in-pad unless the PCB process allows traces of about 2 mil (ds p. 66).
- Put 0.1 µF as close as possible to VDD. Size CC1, CC2 and VCONN traces for at least 1 W (ds p. 66).
- No exposed pad (ds p. 3, p. 68).

## Pitfalls

- Pulling SDA/SCL up to 5 V: the pull-up rail is 1.65–3.6 V (ds p. 4). For 3.3 V I2C, VDD must be above 3 V (ds p. 7).
- Assuming a second TUSB422 can share the bus: the address 0x20 is fixed, with no strap pin (ds p. 3, p. 19).
- Using it as a passive PD monitor:
  - It presents Rd on both CC pins with VDD absent (ds p. 18).
  - It resets to Rd on both CC pins (ROLE_CONTROL = 0x0A) (ds p. 37).
  - It sinks up to 1.36 mA through CC when unpowered (ds p. 5).
  - It returns GoodCRC to received packets (ds p. 11).
- Expecting it to switch VBUS: it has no GPIO, and it ignores the Source/Sink VBUS commands (ds p. 14).
- Expecting automatic discharge on Hard Reset: it does not discharge VBUS on Hard Reset (ds p. 15).
- Expecting 3 A Rp with VDD ≤ 3.0 V: VDD must be greater than 3.0 V (ds p. 6, p. 14).
- Removing VCONN before the switch opens causes a false VCONN fault (ds p. 16).
- During a power-role swap the part disables VCONN, which software must re-enable (ds p. 18).
- Expecting the PD 3.0 Get_Source_Cap_Extended message to be answered: no GoodCRC is sent and no RX alert is raised; the partner retries and then sends Soft Reset (ds p. 12).
- Running the VCONN-supported design above 85 °C: TA max is 85 °C with VCONN (ds p. 4). The orderable addendum gives −40 to 85 °C for all four codes (ds p. 71).

## Not established by the sources

Library versus datasheet:
- **The datasheet names two different packages.** The pin page and the thermal table name a "YFF" package (ds p. 3, p. 5). The orderable codes are YFP (ds p. 71), and the mechanical pages are YFP0009-C01 (ds p. 68–70). The library uses the YFP drawings, which is right for the order codes. Two points remain open:
  - That the p. 3 ball map applies to YFP.
  - That the YFF thermal figures (ds p. 5) apply to YFP.
- **Temperature ratings conflict.** The library intent and parts give no temperature. The datasheet front page and the addendum say −40 to 85 °C (ds p. 1, p. 71). The recommended operating conditions allow 105 °C when VCONN is not used (ds p. 4). Take 85 °C for these order codes.
- **VBUSIN is declared `required … [power_in]`.** The datasheet type is "I", a sense and discharge input tied to connector VBUS (ds p. 3); it does not supply the chip.
- **INT_N, SDA and SCL are open-drain.** The library declares them `output`/`bidirectional`, and its pin kinds cannot express this (ds p. 3). The I2C pull-up rail limit of 1.65–3.6 V (ds p. 4) is not modelled.
- **The land models copper only.**
  - The solder-mask opening (up to 0.05 mm over copper) and the stencil apertures (ds p. 69–70) are not represented.
  - The 2.7 mm courtyard and the A1 dot at (−0.95, −0.95) mm are the library's own; the body is about 1.4 mm square (ds p. 68).

Datasheet inconsistencies:
- Table 1 maps POWER_ROLE = 0 to the source thresholds (ds p. 12). The electrical table selects source thresholds with POWER_ROLE = 1 (ds p. 6).
- tSU;DAT is given as 50 "µs" minimum (ds p. 8), almost certainly meant to be ns. tCC_SAMPLE_RATE repeats the VCONN-fault description (ds p. 8).
- The UFP and DRP initialization lists are headed "for DFP operation", and the DRP list ends with "Rd on both Rp pins" (ds p. 13–14).

Not in the datasheet:
- Power-on behaviour of CC between VDD rising and INT_N, beyond the roughly 100 µs Rd drop when leaving dead battery (ds p. 18).
- MSL is given as Level-1-260C-UNLIM only in the addendum (ds p. 71).
- INT_N leakage with VDD = 0 V above 3.3 V. It is specified only for 0–3.3 V (ds p. 7), although p. 64 allows a pull-up to VDD and the absolute maximum is 6 V (ds p. 4).
- The URL serves TI's current PDF. The application note says the pinned file was retrieved unchanged from it on 2026-10-17. TI regenerates the addendum (here 10-Nov-2025), so the hash may not reproduce later.
- The manifest table in `lib/usb/docs/README.md` has no row for this PDF; its hash is only in `tusb422-application.md`.
- The LCSC numbers come from the library's lcsc_map.json, not from the datasheet.
