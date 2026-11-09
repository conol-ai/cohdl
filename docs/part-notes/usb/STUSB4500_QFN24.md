# STUSB4500 (QFN-24 EP 4x4) — standalone USB PD sink controller with NVM-stored PDOs

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::STUSB4500_QFN24",
  "parts": ["usb::SINK_STUSB4500QTR"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STUSB4500QTR", "lcsc": "C2678061"}
  ],
  "sources": [
    {"id": "ds", "title": "STUSB4500 Standalone USB PD sink controller with short-to-VBUS protections (DS12499)", "revision": "Rev 7, June 2021",
     "url": "https://www.st.com/resource/en/datasheet/stusb4500.pdf", "sha256": "407ee6767128cc4075926e01d945fe846440f932359ed81f45a957485778399e", "local": "lib/usb/docs/st-stusb4500-rev7.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages (1-based). In this document they equal ST's printed "page N/43" footer. The URL is ST's official link, which the package's application note says now serves Rev 8. The pinned Rev 7 file was taken from a GitHub mirror (see "Not established by the sources").

## Summary

The STUSB4500 is a USB Type-C and USB PD sink controller. Its "auto-run" mode negotiates a contract with no MCU, using up to three sink PDOs stored in on-chip NVM, up to 20 V and 5 A (ds p. 1, p. 11). It drives external PMOS switches through high-voltage open-drain outputs (VBUS_EN_SNK, POWER_OK2), has internal and external VBUS discharge paths, and protects CC1/CC2 against shorts to VBUS up to 22 V (ds p. 1, p. 8, p. 15). It is certified for USB PD rev 2.0 and is described only as "interoperable" with PD rev 3.0 (ds p. 1). It is a sink-only controller (ds p. 2, p. 10) and has no VCONN pin (ds p. 3). As shipped, its default PDOs are 5 V/1.5 A, 15 V/1.5 A and 20 V/1.0 A (ds p. 22). Any other power profile needs NVM programming over I2C, or I2C writes at every boot (ds p. 19, p. 23).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address | 0x28 | | 0x2B | — | bits 7:1 = 0 1 0 1 0 ADDR1 ADDR0; bit 0 = R/W | ds p. 17 |
| I2C SCL frequency Fscl | 0 | | 400 | kHz | VDD = 5 V | ds p. 18 |
| I2C bus capacitance Cb | | | 400 | pF | per line | ds p. 18 |
| Digital input VIH | 1.2 | | | V | SCL, SDA, ALERT, RESET, ATTACH, A_B_SIDE, POWER_OK3, GPIO, ADDR0, ADDR1 | ds p. 30 |
| Digital input VIL | | | 0.35 | V | same pins | ds p. 30 |
| Low-voltage open-drain VOL | | | 0.4 | V | 3 mA | ds p. 30 |
| 20 V open-drain VOL (VBUS_EN_SNK, DISCH, POWER_OK2) | | | 0.4 | V | 3 mA | ds p. 30 |
| Idle current IDD(SNK), VSYS supply | 115 | 140 | 165 | µA | VSYS = 3.3 V, not connected | ds p. 29 |
| Idle current IDD(SNK), VDD supply | 110 | 160 | 210 | µA | VDD = 5.0 V, not connected | ds p. 29 |
| NVM-to-register load time TLOAD | | | 30 | ms | after power-up or reset; no I2C access before | ds p. 24, p. 29 |
| CC pull-down Rd | −10 % | 5.1 | +10 % | kΩ | −40 °C < TA < +105 °C | ds p. 29 |
| CC input impedance RINCC | 200 | | | kΩ | terminations off | ds p. 29 |
| CC detection thresholds VTH0.2 / VTH0.66 / VTH1.23 / VTH2.6 | 0.15 / 0.61 / 1.16 / 2.45 | 0.20 / 0.66 / 1.23 / 2.60 | 0.25 / 0.71 / 1.31 / 2.75 | V | | ds p. 29 |
| CC overvoltage protection VOVP | 5.82 | 6 | 6.18 | V | | ds p. 29 |
| VBUS disconnect threshold VTHUSB | 3.2 | 3.3 | 3.4 | V | 5 V / PDO1 | ds p. 29 |
| VBUS safe-0 V threshold VTH0V | 0.5 | 0.6 | 0.7 | V | | ds p. 29 |
| VBUS_VS_DISCH discharge current IDISUSB | | | 50 | mA | set by external series resistor | ds p. 9, p. 29 |
| DISCH discharge current IDISPWR | | | 500 | mA | set by external series resistor | ds p. 6, p. 30 |
| Discharge time to 0 V TDISUSB0V | 70 × k | 84 × k | 100 × k | ms | k = TDISPAR0V, default 9 (756 ms typ) | ds p. 29 |
| Discharge time to lower PDO TDISUSBPDO | 20 × k | 24 × k | 28 × k | ms | k = TDISPARPDO, default 12 (288 ms typ) | ds p. 29 |
| VBUS high limit VMONUSBH (default) | | VBUS +15 % / +10 % | | V | 5 V or PDO1 / PDO2–PDO3 | ds p. 30 |
| VBUS low limit VMONUSBL (default) | | VBUS −20 % | | V | all PDOs | ds p. 30 |
| VBUS monitoring blanked at PDO transition (tSrcReady) | | | 285 | ms | | ds p. 13 |
| PDO2/PDO3 voltage range | 5 | | 20 | V | 50 mV steps; PDO1 fixed at 5 V | ds p. 11 |
| PDO current, look-up table | 0.5 | | 5 | A | 15 fixed values | ds p. 11, p. 22 |
| PDO current, flexible value | 0.01 | | 5 | A | 10 mA steps, shared by all PDOs | ds p. 11, p. 21 |
| Default PDOs (NVM) | | 5 V/1.5 A; 15 V/1.5 A; 20 V/1.0 A | | — | SNK_PDO_NUMB = 3 | ds p. 19, p. 22 |
| VREG_1V2 / VREG_2V7 decoupling | 0.5 | 1 | 10 | µF | each pin | ds p. 9 |
| Thermal resistance RθJA / RθJC | | 37 / 5 | | °C/W | package not stated | ds p. 35 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | 28 | V | ds p. 27 |
| VSYS | 6 | V | ds p. 27 |
| CC1, CC2, CC1DB, CC2DB | 22 | V | ds p. 27 |
| VBUS_EN_SNK, VBUS_VS_DISCH, DISCH, POWER_OK2 | 28 | V | ds p. 27 |
| SCL, SDA, ALERT, RESET, ATTACH, A_B_SIDE, POWER_OK3, GPIO, ADDR0, ADDR1 | −0.3 to 6 | V | ds p. 27 |
| Storage temperature | −55 to 150 | °C | ds p. 27 |
| Maximum junction temperature | 145 | °C | ds p. 27 |
| ESD HBM / CDM | 3 / 1.5 | kV | ds p. 27 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 4.1 | 22 | V | ds p. 28 |
| VSYS | 3.0 | 5.5 | V | ds p. 28 |
| CC1, CC2, CC1DB, CC2DB | 0 | 5.5 | V | ds p. 28 |
| High-voltage pins VBUS_EN_SNK, VBUS_VS_DISCH, DISCH, POWER_OK2 | 0 | 22 | V | ds p. 28 |
| I/O pins SCL, SDA, ALERT, RESET, ATTACH, A_B_SIDE, POWER_OK3, GPIO, ADDR0, ADDR1 | 0 | 4.5 | V | ds p. 28 |
| Operating temperature TA | −40 | 105 | °C | ds p. 28 |

## Pins

Direction gives the datasheet type (ds p. 4, legend p. 5) and, in brackets, the library's pin kind.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CC1DB | CC1DB | HV AIO [bidirectional] | Dead-battery enable on CC1; tie to CC1 if used, else ground | ds p. 4, p. 5 |
| 2 | CC1 | CC1 | HV AIO [bidirectional] | Type-C CC1, to receptacle A5; Hi-Z during reset | ds p. 4, p. 5 |
| 3 | NC | NC | — [passive, optional] | Not connected; "Floating" | ds p. 4 |
| 4 | CC2 | CC2 | HV AIO [bidirectional] | Type-C CC2, to receptacle B5 | ds p. 4 |
| 5 | CC2DB | CC2DB | HV AIO [bidirectional] | Dead-battery enable on CC2; tie to CC2 if used, else ground | ds p. 4, p. 5 |
| 6 | RESET | RESET | DI [input] | Reset, active high | ds p. 4, p. 5 |
| 7 | SCL | SCL | DI [input] | I2C clock; external pull-up | ds p. 4, p. 6 |
| 8 | SDA | SDA | DI/OD [bidirectional] | I2C data, open drain; external pull-up | ds p. 4, p. 6 |
| 9 | DISCH | DISCH | HV AI/OD [bidirectional] | Power-side discharge: internal path through series resistor, or open-drain drive of an external path | ds p. 4, p. 6 |
| 10 | GND | GND | GND [power_in] | Ground | ds p. 4 |
| 11 | ATTACH | ATTACH | OD [output] | Attach flag, active-low open drain; external pull-up | ds p. 4, p. 6 |
| 12 | ADDR0 | ADDR0 | DI [input] | I2C address bit 0; ground if no MCU | ds p. 4, p. 6 |
| 13 | ADDR1 | ADDR1 | DI [input] | I2C address bit 1; ground if no MCU | ds p. 4, p. 6 |
| 14 | POWER_OK3 | POWER_OK3 | OD [output] | Contract flag (default: PDO3 contract), active-low, low-voltage open drain | ds p. 4, p. 7 |
| 15 | GPIO | GPIO | OD [output] | NVM-configured flag (default ERROR_RECOVERY), active-low open drain | ds p. 4, p. 8 |
| 16 | VBUS_EN_SNK | VBUS_EN_SNK | HV OD [output] | Sink power-path enable, active low; drives a PMOS gate directly | ds p. 4, p. 8 |
| 17 | A_B_SIDE | A_B_SIDE | OD [output] | Orientation: Hi-Z = CC1 attached, 0 = CC2 | ds p. 4, p. 9 |
| 18 | VBUS_VS_DISCH | VBUS_VS_DISCH | HV AI [bidirectional] | VBUS sense, monitoring and discharge from the receptacle side; series resistor limits current | ds p. 4, p. 9 |
| 19 | ALERT | ALERT | OD [output] | I2C interrupt, active-low open drain | ds p. 4, p. 6 |
| 20 | POWER_OK2 | POWER_OK2 | HV OD [output] | Contract flag (default: PDO2 contract); high-voltage open drain, can drive a PMOS | ds p. 4, p. 7 |
| 21 | VREG_1V2 | VREG_1V2 | PWR [power_out] | 1.2 V internal regulator, decoupling only | ds p. 4, p. 9 |
| 22 | VSYS | VSYS | PWR [power_in] | Optional system supply 3.0–5.5 V; ground if unused | ds p. 4, p. 9 |
| 23 | VREG_2V7 | VREG_2V7 | PWR [power_out] | 2.7 V internal regulator, decoupling only | ds p. 4, p. 9 |
| 24 | VDD | VDD | HV PWR [power_in] | Supply from VBUS, receptacle side | ds p. 4, p. 9 |
| 25 | GND | EP (unnumbered) | GND [power_in] | Exposed pad, connected to ground; pad 25 is the library's number | ds p. 4 |

## Typical application

- Supply: ST recommends VDD only, with VSYS grounded, in most designs. VDD must be powered for VBUS monitoring, and dead-battery start-up needs it (ds p. 23). VSYS is otherwise 3.0–5.5 V from the system (ds p. 9).
- VREG_1V2 and VREG_2V7: 1 µF each (0.5–10 µF), for decoupling only (ds p. 9).
- Dead battery: tie CC1DB to CC1 and CC2DB to CC2 (Figure 9 does this). Otherwise ground both (ds p. 5, p. 14, p. 25).
- No MCU: ground ADDR0 and ADDR1 (ds p. 6). Figure 9 grounds them through 100 kΩ (R53, R54) and pulls RESET down with 10 kΩ (R55) (ds p. 25).
- With an MCU: use external pull-ups on SCL, SDA and ALERT (4.7 kΩ in Figure 10). The I/O pins are rated 0–4.5 V operating (ds p. 6, p. 26, p. 28).
- Power path: VBUS_EN_SNK drives a PMOS (STL6P3LLH6 in Figures 9 and 10) (ds p. 8, p. 25–26). POWER_OK2/POWER_OK3 can enable separate PDO2/PDO3 paths (configuration 2, the default) (ds p. 7).
- VBUS_VS_DISCH goes to receptacle VBUS through a series resistor sized for ≤ 50 mA (ds p. 9). DISCH needs a resistor for ≤ 500 mA internal discharge, or a pull-up when it drives an external path (ds p. 4, p. 6).
- Defaults that matter (NVM): 3 PDOs, 5 V/1.5 A, 15 V/1.5 A, 20 V/1.0 A (ds p. 22). POWER_OK_CFG = 10b, POWER_ONLY_ABOVE_5V = 0, GPIO_CFG = 01b (error recovery), VBUS discharge on, REQ_SRC_CURRENT = 0 (ds p. 21).
- If no PDO matches, the part requests a 5 V contract with the capability-mismatch bit set (ds p. 11). Set POWER_ONLY_ABOVE_5V = 1 if the load cannot run at 5 V (ds p. 23, p. 25).

## Layout notes

- Recommended land (Figure 12): 24 lands 0.6 × 0.25 mm on 0.5 mm pitch, 4.4 mm across the outer land edges. The centre land is dimensioned D2 × E2, which Table 23 gives as 2.55/2.70/2.80 mm (ds p. 32). The library footprint uses 0.6 × 0.25 mm lands centred at ±1.9 mm and a 2.7 × 2.7 mm centre pad.
- Package body: 4.00 ± 0.05 mm square, 0.80–1.00 mm high (ds p. 32).
- Solder the exposed pad to ground (ds p. 4).
- Pin 1 is at the top left in the top view, with numbering counter-clockwise: 1–6 left, 7–12 bottom, 13–18 right, 19–24 top (ds p. 3). The library footprint follows this order.
- Figures 9 and 10 put ESDA25P35 TVS diodes on VBUS and an ESDA25W on CC1/CC2 (ds p. 25–26). The datasheet gives no layout rules for them.

## Pitfalls

- Expecting 5 A or 20 V/3 A out of the box: the default PDO3 is 20 V/1.0 A (ds p. 22). The "up to 20 V; 5 A" claim needs NVM programming (ds p. 1, p. 11).
- Floating ADDR0/ADDR1 when there is no MCU: they "must be connected to ground" (ds p. 6).
- Leaving CC1DB/CC2DB open: tie each to its CC pin for dead battery, or to ground (ds p. 5).
- Pulling the open-drain flags, I2C or ALERT up to 5 V: those pins are rated 0–4.5 V operating (ds p. 28). Only VBUS_EN_SNK, DISCH and POWER_OK2 are high-voltage (ds p. 28, p. 30).
- Omitting the VBUS_VS_DISCH series resistor: the pin discharges VBUS and is limited to 50 mA (ds p. 9).
- Reading registers right after power-up or reset: wait TLOAD, max 30 ms (ds p. 24, p. 29).
- Powering a load from VREG_1V2/VREG_2V7: both are "used only for external decoupling" (ds p. 9).
- NVM changes take effect only after a power cycle or reset (ds p. 19).
- At detach VBUS_EN_SNK opens, but asserted POWER_OK pins stay asserted until the next attach or a reset (ds p. 8).
- Treating it as a passive monitor: it applies Rd, sends GoodCRC and negotiates (ds p. 10, p. 14).

## Not established by the sources

Library versus datasheet:
- **VREG_1V2 and VREG_2V7 are declared `power_out`.** The datasheet says both are "used only for external decoupling" (ds p. 9). Declared as power outputs, they could pass an ERC check as supply sources for other loads.
- **Open-drain outputs are declared `output`.** ATTACH, POWER_OK3, GPIO, A_B_SIDE, ALERT, VBUS_EN_SNK and POWER_OK2 are open-drain and need external pull-ups (ds p. 4). The library cannot express that a pull-up is needed. It also cannot show that VBUS_EN_SNK and POWER_OK2 tolerate 28 V absolute maximum while the others are limited to 4.5 V (ds p. 27–28).
- **Pad 3 (NC) is declared `optional NC [passive]`, not `no_connect`.** The datasheet says to leave it floating (ds p. 4), but the library would accept a net on it.
- **Exposed pad numbering.** The library numbers the exposed pad 25 and puts it on GND with pad 10. The datasheet leaves EP unnumbered (ds p. 4), and Figures 9 and 10 label it "ExpPAD 0" (ds p. 25–26).
- **Centre-pad and courtyard sizes are not datasheet values.**
  - Figure 12 labels the centre land only "D2"/"E2" (ds p. 32). The library's 2.7 mm takes the package's typical D2/E2.
  - Thermal vias, solder-mask and paste apertures for the exposed pad are not given.
  - The 4.9 mm courtyard and the pin-1 dot are the library's own.
- **Order code to package mapping.** The device summary lists STUSB4500QTR and STUSB4500BJR, and QFN-24 and WLCSP-25, in the same order (ds p. 1). It gives no explicit table pairing order codes with packages. The library's QTR = QFN mapping relies on that order. The WLCSP variant (ds p. 3, p. 33–34) is not in the library.

Package documentation:
- **The application note relies on a revision that is not pinned.** `lib/usb/docs/stusb4500-application.md` cites Rev 8 for two claims, and the pinned Rev 7 contains neither: Table 3 says nothing about SCL/SDA (ds p. 6), and Rev 7 gives no NVM figures.
  - SCL and SDA are pulled down when neither VDD nor VSYS is present.
  - NVM retention and cycling figures.

  No Rev 8 file or hash is in the repository.
- **The URL does not reproduce the pinned hash.** The official URL serves the current revision (Rev 8, according to the application note), not the pinned Rev 7. The pinned PDF came from `github.com/usb-c/STUSB4500` (application note).
- **Manifest gap.** The manifest table in `lib/usb/docs/README.md` has no row for this PDF. Its hash is recorded only in the application note.

Not in the datasheet:
- NVM programming procedure, password, retention and endurance. Rev 7 only says reprogramming needs a "customer password" (ds p. 19).
- The I2C register map. Only the address 2Dh for SW_CTRL_GPIO is given (ds p. 8).
- Whether RESET has an internal pull-down. Figure 9 adds 10 kΩ (ds p. 25).
- Behaviour of SCL/SDA/ALERT when the device is unpowered (Rev 7).
- Which package Table 25's thermal resistances apply to (ds p. 35), and the MSL rating.
- PD 3.0 features (PPS, extended messages): the datasheet claims only interoperability (ds p. 1).
- The LCSC number comes from the library's lcsc_map.json, not from the datasheet.
