# PTN5110 (HX2QFN16) — USB PD TCPC PHY with VCONN switch

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::PTN5110_HX2QFN16",
  "parts": ["usb::TCPC_PTN5110HQZ", "usb::TCPC_PTN5110DHQZ", "usb::TCPC_PTN5110THQZ", "usb::TCPC_PTN5110NHQZ", "usb::TCPC_PTN5110NDHQZ", "usb::TCPC_PTN5110NTHQZ"],
  "identities": [
    {"mfr": "NXP", "mpn": "PTN5110HQZ", "lcsc": null},
    {"mfr": "NXP", "mpn": "PTN5110DHQZ", "lcsc": null},
    {"mfr": "NXP", "mpn": "PTN5110THQZ", "lcsc": null},
    {"mfr": "NXP", "mpn": "PTN5110NHQZ", "lcsc": "C2674062"},
    {"mfr": "NXP", "mpn": "PTN5110NDHQZ", "lcsc": "C3216375"},
    {"mfr": "NXP", "mpn": "PTN5110NTHQZ", "lcsc": "C3216376"}
  ],
  "sources": [
    {"id": "ds", "title": "PTN5110 USB PD TCPC PHY IC, product data sheet", "revision": "Rev. 1.6, 3 April 2020",
     "url": "https://www.nxp.com/docs/en/data-sheet/PTN5110.pdf", "sha256": "f79743918861d2b8369bcfd69d62fca5bac0353290f77cdb278cf1e8d7a99e86", "local": "lib/usb/docs/nxp-ptn5110.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF pages. In this document they equal the printed "N of 60" page numbers. The URL is the one recorded in `lib/usb/docs/ptn5110-application.md`; the PDF itself does not print it.

## Summary

The PTN5110 is a single-port USB Type-C Port Controller (TCPC): CC Rp/Rd/Ra detection and indication, the USB PD BMC PHY and protocol layer, a VCONN load switch with current limit, VBUS monitoring/discharge, and two enable outputs for external VBUS load switches (ds p. 1). PD policy runs in an external TCPM over I2C at up to 1 MHz (ds p. 3, p. 11). The six order codes differ in TCPC interface revision (Rev 1.0 v1.1 or Rev 2.0 v1.0) and in the CC role at power-on reset: DRP starting as sink, sink, or source (ds pp. 5–6). Choose it when a host MCU or EC runs a TCPM stack and you need a small (2.6 × 2.6 mm) TCPC (ds p. 2). Do not choose it for a passive CC monitor: it presents Rd or Rp from POR and clamps CC under dead battery (ds pp. 5–6, p. 17).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address, SLV_ADDR to GND | | 0x50 | | — | 8-bit 1010000x | ds p. 15 |
| 7-bit I2C address, SLV_ADDR 10 kΩ pull-up to BYPASS | | 0x51 | | — | 8-bit 1010001x | ds p. 15 |
| 7-bit I2C address, SLV_ADDR unconnected | | 0x52 | | — | 8-bit 1010010x | ds p. 15 |
| 7-bit I2C address, SLV_ADDR 100 kΩ pull-up to BYPASS | | 0x53 | | — | 8-bit 1010011x; SLV_ADDR sampled at POR only | ds p. 8, p. 15 |
| TCPC ID: VENDOR_ID / PRODUCT_ID | | 0x1FC9 / 0x5110 | | — | registers 00h / 02h | ds p. 16 |
| I2C clock frequency FI2C | 0 | | 1000 | kHz | | ds p. 42 |
| I2C VIH / VIL | 0.7 × Vpullup | | 0.3 × Vpullup | V | | ds p. 42 |
| I2C VOL | 0 | | 0.4 | V | 3 mA sink, Vpullup > 2 V | ds p. 42 |
| BYPASS voltage VBYPASS | 1.7 | | 1.9 | V | 2.2 µF ± 20 % on BYPASS | ds p. 36 |
| VDD active current, sink attached | | 0.55 | 0.7 | mA | VBUS ADC/monitor and VCONN off | ds p. 35 |
| VDD active current, source attached, 3 A advertised | | 0.8 | 1 | mA | | ds p. 35 |
| VDD current while transmitting PD | | 6.3 | 8 | mA | peak, not steady state | ds p. 35 |
| VDD idle current, sink / source / DRP unattached | | 30 / 35 / 40 | | µA | | ds p. 35 |
| VBUS current in dead battery | | 145 | | µA | sink, VBUS = 5 V, VDD = 0 | ds p. 35 |
| Rp current source, default / 1.5 A / 3 A | 64 / 166 / 314 | 80 / 180 / 330 | 96 / 194 / 346 | µA | | ds p. 38 |
| Rd pull-down Rpulldn | 4.6 | 5.1 | 5.6 | kΩ | after on-board calibration | ds p. 38 |
| zOPEN | 126 | | | kΩ | | ds p. 38 |
| Dead-battery clamp VCLAMPH / VCLAMPM / VCLAMPL | 0.85 / 0.45 / 0.25 | | 2.18 / 1.25 / 1.25 | V | sink, VDD = 0, VBUS = 0 | ds pp. 38–39 |
| CC receiver capacitance cReceiver | 200 | | 400 | pF | not transmitting, incl. VCONN switch | ds p. 38 |
| BMC vSwing | 1.05 | 1.125 | 1.2 | V | | ds p. 37 |
| VBUS present threshold | 3.5 | | 4 | V | | ds p. 38 |
| VBUS measurement LSB / accuracy | | 25 / ±2 | | mV / % | 2.5 V < VBUS < 25 V | ds p. 38 |
| VCONN switch RON | | 350 | 450 | mΩ | | ds p. 41 |
| VCONN current-limit settings | 100 | 150 / 300 / 450 / 600 | 750 | mA | four programmable thresholds; see p. 41 for each band | ds p. 41 |
| VCONN short-to-GND trigger IShort | 0.75 | | 2.4 | A | | ds p. 41 |
| VCONN_IN UVLO | 1.7 | | 1.9 | V | | ds p. 41 |
| VCONN OTP threshold | | 138 | | °C | re-enable after cooling to 85 °C | ds p. 13, p. 41 |
| EN_SRC / EN_SNK1 VOH | 1.4 | | | V | IOH = 1 mA | ds p. 44 |
| EN_SRC / EN_SNK1 VOL | | | 0.3 | V | IOL = 1 mA | ds p. 44 |
| GPIO (DBG_ACC, ILIM_5V_VBUS, FRS_EN) VOH | BYPASS − 0.3 | | | V | IOH = 1 mA | ds p. 44 |
| ALERT_N / FAULT_N VOL | | | 0.5 | V | IOL = 1 mA, 10 kΩ pull-up | ds p. 44 |
| I2C write to FET enable/disable tFET_EN / tFET_DIS | | | 50 | µs | | ds pp. 35–36 |
| VBUS debounce time | | 15 | | ms | | ds p. 20 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | −0.5 to +6.0 | V | ds p. 33 |
| VBUS | −0.5 to +28 | V | ds p. 33 |
| I2C pull-up voltage Vpullup (I2C_SCL, I2C_SDA) | −0.5 to +5.5 | V | ds p. 33 |
| CC1, CC2 | −0.5 to +6.0 | V | ds p. 33 |
| VCONN_IN | −0.5 to +6.0 | V | ds p. 33 |
| BYPASS | −0.5 to +2.5 | V | ds p. 33 |
| EN_SRC, EN_SNK1, ALERT_N, ILIM_5V_VBUS, DBG_ACC, FRS_EN, FAULT_N, SLV_ADDR | −0.5 to +4.6 | V | ds p. 33 |
| I2C_SCL, I2C_SDA input voltage VI | −0.5 to +4.6 | V | ds p. 33 |
| Storage temperature | −65 to +150 | °C | ds p. 33 |
| ESD HBM: VBUS, CC1, CC2 / other pins | 8000 / 2000 | V | ds p. 33 |
| ESD CDM | 1000 | V | ds p. 33 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD (typ 3.3) | 2.7 | 5.5 | V | ds p. 34 |
| VBUS (typ 5) | 3.9 | 25 | V | ds p. 34 |
| I2C pull-up voltage Vpullup ("recommended to be same/close to VDD") | 2.7 | 5.5 | V | ds p. 34, p. 43 |
| CC1, CC2 input voltage | −0.3 | 5.5 | V | ds p. 34 |
| VCONN_IN input voltage | −0.3 | 5.5 | V | ds p. 34 |
| VCONN_IN for switch operation VVCONN_IN | 2.7 | 5.5 | V | ds p. 41 |
| EN_SRC, EN_SNK1, ALERT_N, ILIM_5V_VBUS, DBG_ACC, FRS_EN, FAULT_N | −0.3 | 3.6 | V | ds p. 34 |
| SLV_ADDR | −0.3 | 2.0 | V | ds p. 34 |
| Ambient temperature Tamb, all six order codes | −40 | +85 | °C | ds p. 6, p. 34 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | FRS_EN | FRS_EN | Output, CMOS IO (BYPASS-referenced) | Fast Role Swap arm for a 5 V source switch; default LOW; usable as GPIO | ds p. 8 |
| 2 | EN_SNK1 | EN_SNK1 | Output, CMOS IO (BYPASS-referenced) | VBUS sink path enable; LOW at POR; may drive a second source rail | ds p. 8 |
| 3 | VDD | VDD | Power input | Core supply 2.7–5.5 V; needs 2.2 µF ± 10 % | ds p. 8 |
| 4 | BYPASS | BYPASS | Internal | Internal power rail; needs an external capacitor (e.g. 2.2 µF ± 20 %) | ds p. 8 |
| 5 | SLV_ADDR | SLV_ADDR | IO, quaternary input | I2C address select, sampled at POR only | ds p. 8 |
| 6 | ILIM_5V_VBUS | ILIM_5V_VBUS | IO, CMOS (BYPASS-referenced) | Current-limit select for the 5 V source switch; default LOW; usable as GPIO | ds p. 9 |
| 7 | I2C_SDA | I2C_SDA | IO, open drain | I2C data; pull up to VDD | ds p. 9 |
| 8 | I2C_SCL | I2C_SCL | Input, open drain IO | I2C clock; pull up to VDD | ds p. 9 |
| 9 | DBG_ACC | DBG_ACC | Output, CMOS IO (BYPASS-referenced) | Debug accessory indication, active LOW; HIGH at POR; usable as GPIO | ds p. 9 |
| 10 | ALERT_N | ALERT_N | Output, open drain | Level-triggered interrupt; 10 kΩ pull-up "to VDD" | ds p. 9 |
| 11 | FAULT_N | FAULT_N | Input, open drain | Fault input from load switches; 10 kΩ pull-up; pull HIGH if VBUS path is not controlled | ds p. 9, p. 14 |
| 12 | VCONN_IN | VCONN_IN | Power input | VCONN supply; e.g. 2.2 µF ± 10 % | ds p. 9 |
| 13 | CC1 | CC1 | IO, custom | Configuration channel 1; protection diode "shall be used" | ds p. 9 |
| 14 | CC2 | CC2 | IO, custom | Configuration channel 2; protection diode "shall be used" | ds p. 9 |
| 15 | VBUS | VBUS | Power input | VBUS supply/sense; needs 2.2 µF ± 10 % | ds p. 9 |
| 16 | EN_SRC | EN_SRC | Output, CMOS IO (BYPASS-referenced) | 5 V VBUS source path enable; LOW at POR | ds p. 9 |
| 17 | GND | GND (centre pad, unnumbered) | Ground | Centre pad as ground; pad 17 is the library's number | ds p. 8, p. 9 |

## Typical application

- VDD and VBUS each need external decoupling of 2.2 µF ± 10 %; BYPASS needs a capacitor such as 2.2 µF ± 20 %; VCONN_IN can take e.g. 2.2 µF ± 10 % (ds pp. 8–9).
- SLV_ADDR: GND, 10 kΩ to BYPASS, open, or 100 kΩ to BYPASS selects 0x50–0x53; it is wired to BYPASS (not VDD) for two of the four options (ds p. 8, p. 15).
- I2C_SCL/I2C_SDA pull up to VDD (ds p. 9); ALERT_N 10 kΩ pull-up (ds p. 9); FAULT_N 10 kΩ pull-up, and pulled HIGH when the PTN5110 does not control the VBUS power path (ds p. 9, p. 14).
- CC1 and CC2 each need a protection diode such as PESD5V0S1USF/BSF or PESD5V0S1UL/BL (ds p. 9). The use-case figures omit ESD/TVS parts (ds p. 18).
- EN_SRC drives the 5 V source switch with ILIM_5V_VBUS selecting default (0.9 A) or 1.5/3 A limit; EN_SNK1 drives the sink switch or a second source rail (ds p. 13). Table 5 lists NX5P3290 and NX20P5090 combinations (ds p. 13).
- Order-code choice: NXP recommends PTN5110HQ for a dead-battery DRP notebook whose power path takes longer than the 15 ms VBUS debounce, and PTN5110DHQ if VDD arrives before that debounce (ds p. 20, p. 32).
- Firmware register programming is in the "PTN5110 Application Programming guide" [5], available only from NXP (ds p. 15, p. 56).

## Layout notes

- Package SOT1883-1 (HX2QFN16): body 2.6 × 2.6 × 0.35 mm nominal (0.40 mm max), pitch 0.4 mm, terminal length 0.35–0.45 mm, exposed pad Dh × Eh = 1.1 × 1.0 mm nominal (ds p. 46). Mounted height 0.5 mm max (ds p. 5).
- Recommended land (Fig. 21, drawing EP17081): pitch 0.40 mm, outer span Ax = Ay = 3.60 mm, inner span Bx = By = 1.80 mm, land 0.24 × 0.90 mm, centre land 1.10 × 1.00 mm, solder resist 0.05 mm around copper, stencil 0.1 mm, centre paste 0.60 × 0.50 mm, lead paste 0.19 × 0.85 mm, occupied area 3.85 × 3.85 mm (ds p. 54). The drawing carries an "UNRELEASED" watermark (ds p. 54).
- Pin 1 is top-left in the transparent top view; 1–5 down the left side, 6–8 along the bottom, 9–13 up the right, 14–16 along the top (ds p. 8). The 5-terminal sides carry the 1.0 mm (Eh) dimension of the centre pad, the 3-terminal sides the 1.1 mm (Dh) one (ds p. 46, p. 54).
- The library footprint copies the copper values: 0.9 × 0.24 mm lands centred ±1.35 mm, a 1.1 × 1.0 mm centre pad, and a 4.1 mm courtyard (the courtyard is the library's own).

## Pitfalls

- Strapping SLV_ADDR to VDD: its operating maximum is 2.0 V (ds p. 34) and the strap options use BYPASS (1.7–1.9 V) (ds p. 15, p. 36).
- Pulling ALERT_N or FAULT_N up to a 5 V VDD: their operating maximum is 3.6 V and absolute maximum 4.6 V (ds pp. 33–34), even though p. 9 says "pulled up by 10 k to VDD" and p. 45 allows a 2.7–5.5 V ALERT_N pull-up.
- Pulling I2C up to 5 V: Vpullup may be 5.5 V (ds pp. 33–34) but the I2C_SCL/I2C_SDA input-voltage limit is 4.6 V (ds p. 33). The datasheet does not reconcile the two.
- Driving a load switch whose EN threshold is above 1.4 V: EN_SRC/EN_SNK1 VOH is only ≥ 1.4 V at 1 mA (ds p. 44), and the GPIOs swing only to BYPASS (ds p. 44).
- Leaving FAULT_N floating when EN_SRC/EN_SNK1 are unused: it "shall be pulled HIGH" (ds p. 14).
- Assuming EN_SNK1 stays low until firmware runs: in dead battery the PTN5110 asserts EN_SNK1 when VBUS is valid; on a multi-port platform several ports may sink at once (ds p. 13, p. 17).
- Assuming Try.SRC/Try.SNK on the N parts: it exists only on PTN5110HQ, DHQ and THQ (ds p. 10). The autonomous Type-C timing in Table 13 applies only to PTN5110HQ (ds p. 40).
- Confusing absolute maximum with operating: VBUS 28 V and CC 6.0 V are absolute maxima (ds p. 33); operation is VBUS ≤ 25 V and CC ≤ 5.5 V (ds p. 34).
- Treating the VCONN limit as exact: the 600 mA setting limits anywhere from 500 to 750 mA, the 150 mA setting from 100 to 230 mA (ds p. 41); the TCPM programs the setting (ds p. 12).

## Not established by the sources

- **Land pattern release.** The only land drawing (Fig. 21) is watermarked "UNRELEASED" (ds p. 54). The library footprint and its `#[intent]` say so; the land still needs NXP confirmation before fabrication.
- **Library pin types vs Table 4.** `BYPASS` is `power_out`, but the datasheet calls it an "Internal" node (ds p. 8, p. 17). `SLV_ADDR` is `input`; the datasheet calls it IO/quaternary input (ds p. 8). `ALERT_N` is `output` and `I2C_SDA` `bidirectional`, but both are open drain (ds p. 9); `FAULT_N` is an open-drain input. `FRS_EN`, `DBG_ACC` and `EN_SRC`/`EN_SNK1` are `output`, while the GPIO-capable pins (FRS_EN, DBG_ACC, ILIM_5V_VBUS) "can also be used for other GPIO purposes" (ds pp. 8–9). `GND` is `power_in`. None of these changes a pad.
- **`VCONN_IN` is `required`.** The datasheet does not say whether VCONN_IN may be left open or grounded when the port never sources VCONN.
- **TCPC revision in the ID register.** Table 2 says HQ/DHQ/THQ are TCPC Rev 1.0 v1.1 and the N parts Rev 2.0 v1.0 (ds pp. 5–6), but Table 7 gives one PD_INTERFACE_REV default, 0x0010 (= TCPC Rev 1.0, Version 1.0), without naming an order code (ds p. 16). Which value each order code reports is not established.
- **VBUS operating minimum.** Table 10 says 3.9 V (ds p. 34); the features list says 4 V (ds p. 3).
- **Dead-battery behaviour of the source-at-POR parts (THQ, NTHQ).** p. 17 says the device clamps both CC pins under dead battery without naming order codes; Table 2 says the T parts present a source role at POR (ds pp. 5–6).
- **Thermal data.** No θJA/θJC, junction-temperature limit, MSL or thermal-via guidance is given; the centre pad is only "Ground" (ds p. 9).
- **Paste and mask in the library.** The footprint has copper only; NXP's 0.05 mm resist expansion and reduced paste (0.60 × 0.50 mm centre, 0.19 × 0.85 mm leads) are not modelled (ds p. 54).
- **Register map and POR register values** beyond the ID registers: these are in the application programming guide, which is not in the repository (ds p. 15, p. 56).
- **LCSC numbers.** PTN5110HQZ, DHQZ and THQZ have no entry in `lcsc_map.json`; the other three numbers come from that map, not the datasheet.
