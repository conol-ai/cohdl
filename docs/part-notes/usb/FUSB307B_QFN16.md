# FUSB307B (WQFN16) — USB Type-C port controller (TCPC) with USB PD 3.0, legacy

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::FUSB307B_QFN16",
  "parts": ["usb::TCPC_FUSB307BMPX", "usb::TCPC_FUSB307BVMPX"],
  "identities": [
    {"mfr": "onsemi", "mpn": "FUSB307BMPX", "lcsc": "C603073"},
    {"mfr": "onsemi", "mpn": "FUSB307BVMPX", "lcsc": "C603074"}
  ],
  "sources": [
    {"id": "ds", "title": "FUSB307B USB Type-C Port Controller with USB-PD (FUSB307B/D)", "revision": "Rev. 7, May 2026 (document discontinued)",
     "url": "https://www.onsemi.com/pdf/datasheet/fusb307b-d.pdf", "sha256": "f8adcc3df9f5ae85627e69c51a8635d2179dc6ace67a8afca53e6a36a45bb6ca", "local": "lib/usb/docs/onsemi-fusb307b-d.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF pages. In this document they equal the printed page numbers (the case outline is unnumbered PDF page 57). The URL is the one recorded in `lib/usb/docs/fusb307b-application.md`.

## Summary

The FUSB307B is an I2C-controlled USB Type-C Port Controller compliant with the USB-PD Interface Specification Rev 1.0 Ver 1.2 (TCPCI) and USB-PD Rev 3.0 Ver 1.1, with Type-C Rev 1.3 detection, a 3 W VCONN switch, a 10-bit VBUS ADC, SRC/SNK switch-control outputs, two GPIOs and four selectable I2C addresses (ds p. 1). A TCPM runs PD policy; the chip handles GoodCRC, retries and Hard Reset itself (ds p. 1, p. 12). Its internal LDO, fed from VBUS, can power the TCPM in dead battery (ds p. 6). The MPX part is rated −40 to 85 °C and the VMPX part, automotive, −40 to 105 °C (ds p. 3). Rev. 7 of the datasheet is "Document Discontinued" (14 May 2026) (ds p. 56) and its pages carry the watermark "DISCONTINUED — THIS DEVICE IS NOT AVAILABLE. PLEASE CONTACT YOUR onsemi REPRESENTATIVE FOR INFORMATION" (e.g. ds p. 3, p. 20). Do not choose it for new designs, nor as a passive CC monitor, because it presents Rd unpowered and auto-acknowledges messages (ds p. 5, p. 12).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address, pin 4 low, pin 11 = SCL, pin 12 = SDA | | 0x50 | | — | bits 7:1 = 1010000 | ds p. 7 |
| 7-bit I2C address, pin 4 high, pin 11 = SCL, pin 12 = SDA | | 0x51 | | — | bits 7:1 = 1010001 | ds p. 7 |
| 7-bit I2C address, pin 4 low, pin 11 = SDA, pin 12 = SCL | | 0x52 | | — | bits 7:1 = 1010010 | ds p. 7 |
| 7-bit I2C address, pin 4 high, pin 11 = SDA, pin 12 = SCL | | 0x53 | | — | bits 7:1 = 1010011; pin 4 read at power-up via pull-up/pull-down | ds p. 5, p. 7 |
| I2C clock fSCL | 0 | | 1000 | kHz | Fast Mode Plus | ds p. 27 |
| I2C VILI2C / VIHI2C | | | 0.4 / — | V | VIH min 1.2 V; VDD 3.0–5.5 V | ds p. 26 |
| I2C VOL | 0 | | 0.36 | V | 2 mA sink | ds p. 26 |
| INT_N, DBG_N VOL | | | 0.4 | V | IOL = 4 mA | ds p. 26 |
| GPIO/ORIENT VIL / VIH | | | 0.4 / — | V | VIH min 1.2 V | ds p. 26 |
| GPIO/ORIENT, SRC/SNK VOH | 0.7 × VDD | | | V | IOH = −2 mA | ds p. 26 |
| GPIO/ORIENT, SRC/SNK VOL | | | 0.4 | V | IOL = 4 mA | ds p. 26 |
| Disable current IDISABLE | | | 10 | µA | ROLECTRL = 0x0F | ds p. 21 |
| Standby current, unattached sink / DRP or source | | 6 / 7 | 10 / 20 | µA | | ds p. 21 |
| Rp current, default / 1.5 A / 3 A | 64 / 166 / 304 | 80 / 180 / 330 | 96 / 194 / 356 | µA | | ds p. 22 |
| Rd pull-down RDEVICE | 4.6 | 5.1 | 5.6 | kΩ | min/max only when powered (Note 12) | ds p. 22–23 |
| Dead-battery pull-down voltage VUFPDB | | | 2.18 | V | all DFP pull-up loads | ds p. 22 |
| zOPEN | 126 | | | kΩ | ROLECTRL = 0x0F | ds p. 23 |
| VCONN switch RDSON RSW_CCx | | 0.4 | 1.0 | Ω | | ds p. 22 |
| VCONN OCP limit | 600 | 800 | 1000 | mA | VCONN_OCP = 0x0F (reset value) | ds p. 22, p. 52 |
| VCONN valid threshold | | 2.4 | | V | | ds p. 8, p. 23 |
| VCONN switch thermal off / hysteresis | | 145 / 10 | | °C | | ds p. 25 |
| CC receiver capacitance cReceiver | | 25 | | pF | Note 11 | ds p. 21 |
| BMC zDriver | 33 | | 75 | Ω | 750 kHz, 220 pF load | ds p. 21 |
| VBUS ADC LSB / accuracy | | 25 / ±2 | | mV / % | ±5 % at 85–105 °C | ds p. 24 |
| VBUS_VAL threshold vVBUSthr | 3.5 | | 4.0 | V | VDD > VDDGOOD; VMPX max 4.05 V | ds p. 24 |
| vSafe0V threshold | | | 0.8 | V | | ds p. 24 |
| Bleed discharge RBLEED | 4 | | 7 | kΩ | VBUS 4.0–21.5 V | ds p. 25 |
| LDO output VV3P3 | 3.0 | | 3.6 | V | VDD < VDDGOOD, VBUS 4.0–21.5 V | ds p. 25 |
| LDO output current ILDO_MAX | 30 | | | mA | VDROP = 120 mV | ds p. 25 |
| VDDGOOD (switch from VBUS to VDD power) | 2.7 | | 3.0 | V | | ds p. 25 |
| CC debounce tTCPCfilter | 4 | | 500 | µs | | ds p. 23 |
| tCCDebounce | 100 | 150 | 200 | ms | | ds p. 23 |
| Watchdog THVWatchdog | 1500 | | 2000 | ms | when TCPC_CTRL.EN_WATCHDOG = 1 | ds p. 17, p. 25 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | −0.5 to 6.0 | V | ds p. 20 |
| CC pins as host, device or DRP | −0.5 to 6.0 | V | ds p. 20 |
| VBUS | −0.5 to 28.0 | V | ds p. 20 |
| Storage temperature | −65 to +150 | °C | ds p. 20 |
| Junction temperature | +150 | °C | ds p. 20 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 20 |
| ESD HBM, connector pins (VBUS, CCx) / others | 4 / 2 | kV | ds p. 20 |
| ESD CDM, all pins | 1 | kV | ds p. 20 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VBUS (typ 5.0) | 4.0 | 21.5 | V | ds p. 20 |
| VDD (typ 3.3); 2.8 V is functional only, electrical specs need ≥ 3.0 V (Note 8) | 2.8 | 5.5 | V | ds p. 20 |
| VCONN supply | 2.7 | 5.5 | V | ds p. 20 |
| VCONN supply current | — | 560 | mA | ds p. 20 |
| Ambient temperature, FUSB307BMPX | −40 | +85 | °C | ds p. 3, p. 20 |
| Ambient temperature, FUSB307BVMPX (automotive) | −40 | +105 | °C | ds p. 3, p. 20 |
| I2C pull-up supply VDDEXT (Note 15: between 1.71 V and VDD) | 1.8 | 3.6 | V | ds p. 26 |

## Pins

Pin 1 is top-left in the top view on p. 1. The p. 4 "Top Through View" is the same device rotated 180°; the p. 4 bottom view is mirrored (ds p. 1, p. 4).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CC1 | CC1 | I/O | Type-C configuration channel 1 | ds p. 1, p. 5 |
| 2 | VCONN | VCONN | Power switch input | Regulated input switched to the active CC pin as VCONN | ds p. 1, p. 5 |
| 3 | CC2 | CC2 | I/O | Type-C configuration channel 2 | ds p. 1, p. 5 |
| 4 | ORIENT_I2C_ADDR | ORIENT/I2C_ADDR | 3-state CMOS output (read at power-up) | I2C address select at power-up, then general-purpose/orientation output | ds p. 1, p. 5, p. 7 |
| 5 | LDO | LDO | LDO output | 3.3 V LDO output | ds p. 1, p. 5 |
| 6 | VBUS | VBUS | Power | VBUS attach/detach sense; LDO input in dead battery | ds p. 1, p. 5–6 |
| 7 | VDD | VDD | Power | Supply input | ds p. 1, p. 5 |
| 8 | DBG_N | DBG_N | Open-drain I/O | Debug accessory detection output | ds p. 1, p. 5 |
| 9 | GPIO2 | GPIO2 | 3-state CMOS I/O | GPIO 2; optional Fast Role Swap I/O | ds p. 1, p. 5, p. 53 |
| 10 | INT_N | INT_N | Open-drain output | Active-low interrupt | ds p. 1, p. 5 |
| 11 | SCL1_SDA2 | SCL1/SDA2 | Open-drain I/O | I2C SCL or SDA depending on address choice | ds p. 1, p. 5, p. 7 |
| 12 | SDA1_SCL2 | SDA1/SCL2 | Open-drain I/O | I2C SDA or SCL depending on address choice | ds p. 1, p. 5, p. 7 |
| 13 | SRC | SRC | CMOS output | External VBUS source switch enable, active high | ds p. 1, p. 5 |
| 14 | SNK | SNK | CMOS output | External VBUS sink switch enable, active high | ds p. 1, p. 5 |
| 15 | GND | GND | Ground | Ground | ds p. 1, p. 5 |
| 16 | GPIO1 | GPIO1 | 3-state CMOS I/O | GPIO 1 | ds p. 1, p. 5 |
| 17 | GND | GND (centre pad, unnumbered) | Ground | Labelled GND in the pin diagrams; pad 17 is the library's number | ds p. 1, p. 4 |

## Typical application

- Figure 2: 1 µF on VDD, 10 µF on VCONN, 1 µF on LDO (which feeds the TCPM VDD), 220 pF from CC1 and from CC2 to GND, 1 µF 25 V on connector VBUS and 10 µF 25 V on the sink rail; SRC and SNK drive FPF2895 load switches (ds p. 3).
- Pin 4 takes a pull-up or pull-down resistor at power-up (Figure 2 shows a resistor to GND, and pin 4 also drives the FUSB340 mux select) (ds p. 3, p. 7). Pins 11/12 are routed to SCL/SDA according to Table 3 (ds p. 7).
- I2C pull-ups go to 1.8–3.6 V and, per Note 15, no higher than VDD (ds p. 26). INT_N and DBG_N are open drain (ds p. 5), so they need external pull-ups; the datasheet gives no resistor value, and Figure 2 draws none (ds p. 3).
- Reset defaults: ROLECTRL = 0x4A when powered normally (DRP = 1, CC1/CC2 = Rd) and 0x0A in dead battery (Rd/Rd) (ds p. 38). Toggling starts only on COMMAND.LOOK4CON (ds p. 5, p. 38). RXDETECT = 0x00, so nothing is received until the TCPM enables it (ds p. 10, p. 29). PWRCTRL = 0x60, EN_VCONN = 0 (ds p. 39). ALERTMSKL = 0xFF (ds p. 28). GPIOs reset to high-Z with input buffers disabled (ds p. 52–53). VCONN_OCP = 0x0F (800 mA range, maximum) (ds p. 52).
- INT_N asserts after power-up or SW_RES; the TCPM clears interrupts by writing 1 (ds p. 7).
- Dead battery: VBUS powers the LDO and the chip enables the sink path when attached to any source; once VDD exceeds VDDGOOD the LDO is bypassed (ds p. 6).

## Layout notes

- Case 510BS, WQFN16 3 × 3 mm, 0.5 mm pitch, 0.80 mm max height, exposed pad 1.65–1.75 mm square, terminals 0.18–0.30 mm wide and 0.30–0.40 mm long (ds p. 57).
- Recommended land: 0.30 × 0.57 mm lands at 0.50 mm pitch, 3.30 mm outer and 2.16 mm inner span, 1.75 × 1.75 mm centre land (ds p. 57). The library footprint uses these values (pad-row centres ±1.365 mm).
- The exposed pad is labelled GND in the pin diagrams (ds p. 1, p. 4); the datasheet gives no thermal-via, mask or paste guidance.

## Pitfalls

- Designing it in: the datasheet watermark says "THIS DEVICE IS NOT AVAILABLE" (e.g. ds p. 3, p. 20) and Rev. 7 is "Document Discontinued" (ds p. 56).
- Expecting a fixed address: four addresses depend on both the pin 4 level at power-up and which of pins 11/12 is SCL (ds p. 7). Hard-strapping pin 4 to a rail fights its later output role (ds p. 5).
- Pulling I2C up to VDD = 5 V: VDDEXT is limited to 1.8–3.6 V (ds p. 26).
- Treating the part as passive: Rd is presented unpowered (ds p. 5), ROLECTRL resets to Rd/Rd (ds p. 38), and once RXDETECT is enabled it auto-sends GoodCRC within tTransmit (ds p. 10, p. 12).
- Running VDD between 2.8 and 3.0 V: functional only; all electrical specs need ≥ 3.0 V (ds p. 20).
- Assuming 105 °C for FUSB307BMPX: only FUSB307BVMPX is rated to 105 °C (ds p. 3, p. 20).
- Exposing CC to VBUS: CC is rated 6.0 V absolute maximum (ds p. 20); there is no VCONN or I/O absolute-maximum row.
- Relying on OCP for the 560 mA VCONN rating: the reset OCP trips at 600–1000 mA (ds p. 20, p. 22); lower VCONN_OCP (ds p. 52).
- Loading the LDO beyond the TCPM: it is specified for ≥ 30 mA (ds p. 25).

## Not established by the sources

- **Lifecycle vs the library.** The library's `#[intent]` says "Legacy ... manufacturer document discontinued May 2026", and the package README calls the part "legacy". The source goes further: its watermark says "THIS DEVICE IS NOT AVAILABLE" (e.g. ds p. 3, p. 20). Both parts are still mapped to LCSC part numbers (C603073, C603074, in `lcsc_map.json`, not from the datasheet). Neither has a supplier binding. Whether stock exists is not established.
- **Library pin types vs Table 2.** `ORIENT_I2C_ADDR` is `bidirectional`; Table 2 calls it a "3-State CMOS Output" that "selects I2C Address on Power up" (ds p. 5), and p. 7 calls it "I2C_ADDR_GPO input". `INT_N` is `output`, `DBG_N` `bidirectional`, `SCL1_SDA2`/`SDA1_SCL2` `bidirectional`; the datasheet says these are open drain (ds p. 5). `VCONN` is `power_in` (datasheet: "Power Switch"), `LDO` `power_out`, `GND` `power_in`. None of these changes a pad.
- **`required` pins.** The library makes VCONN, LDO and ORIENT_I2C_ADDR required. The datasheet does not say whether VCONN may be left open in a port that never sources VCONN, or whether the LDO pin may be left unconnected (Figure 2 always fits 1 µF) (ds p. 3).
- **Exposed pad.** Table 2 has no row for the exposed pad; it is only drawn as "GND" in the pin diagrams (ds p. 1, p. 4, p. 5). That it must be soldered, and thermal resistance (θJA/θJC), MSL and via pattern, are not given.
- **Datasheet residue from sister parts.** Table 16 has "MUX_SEL" and "SRC_HV" pins (ds p. 26), STD_OUT_CFG controls "MUX_S0/MUX_S1 outputs" (ds p. 37), and flow charts mention "FUSB305" and SRC_HV (ds p. 9, p. 10, p. 13–14, p. 19); none of these pins exists on the 16-lead package (ds p. 1).
- **Internal contradictions.**
  - I2C pull-up range: 1.8–3.6 V in Table 16 vs "between 1.71 V and VDD" in Note 15 (ds p. 26).
  - I2C input thresholds: VILI2C 0.4 V and VIHI2C 1.2 V in Table 16 (ds p. 26) vs "VIL = 0.3VDD, VIH = 0.7VDD" in Figure 19 (ds p. 27).
  - Default VCONN current limit: "210.5 mA" when PWRCTRL.VCONN_PWR = 0 (ds p. 8) vs "Writing this bit has no function. Please use VCONN_OCP" (ds p. 39) and the 600–1000 mA OCP at reset (ds p. 22).
  - The orientation bit is PWRCTRL.ORIENT on p. 12 but TCPC_CTRL.ORIENT on pp. 8 and 38.
  - Rd threshold for "default" and "1.5 A" sourcing is the same 1.50–1.65 V (ds p. 23).
- **Pull-up values** for INT_N, DBG_N and I2C, and the pin 4 strap resistor value.
- **Power-on time** from VDD/VBUS valid to I2C ready.
- **Back-powering** through I2C or GPIO pins when VDD is off; Table 16 gives only an "Off Input Leakage" of ±5 µA at VDD = 0 for GPIO/ORIENT (ds p. 26).
