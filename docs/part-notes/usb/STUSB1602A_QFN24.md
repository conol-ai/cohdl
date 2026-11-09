# STUSB1602A (QFN24 EP 4x4) — USB Type-C controller with USB PD BMC transceiver for an external MCU

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::STUSB1602A_QFN24",
  "parts": ["usb::PHY_STUSB1602AQTR"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STUSB1602AQTR", "lcsc": "C713192"}
  ],
  "sources": [
    {"id": "ds", "title": "STUSB1602 USB Type-C controller with TX/RX line driver and BMC (DS11254)", "revision": "Rev 5, July 2021",
     "url": "https://www.st.com/resource/en/datasheet/stusb1602.pdf", "sha256": "eb35151e98a6b9fbfb76fccdf277759b48ad72da472090989fb30b484a063e79", "local": "lib/usb/docs/st-stusb1602-rev5.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages (1-based). In this document they equal ST's printed "page N/39" footer. The URL is ST's official link. The pinned file was taken from a GitHub mirror (see "Not established by the sources").

## Summary

The STUSB1602A is a 20 V-technology USB Type-C port controller with source, sink and dual-role support. It also contains a USB PD BMC transmitter/receiver (ds p. 1). It is not a complete PD controller: an external MCU runs the PD protocol stack, exchanging bits over a serial interface whose clock and chip select the STUSB1602 drives, and configuring the part over I2C (ds p. 1, p. 5–6, p. 9, p. 17). It integrates a VCONN switch (100–600 mA limit) and VBUS monitoring and discharge. Two high-voltage open-drain outputs drive external PMOS switches for source (VBUS_EN_SRC) and sink (VBUS_EN_SNK) power paths (ds p. 1, p. 6, p. 13). CC pins tolerate shorts to VBUS up to 22 V (ds p. 14). Choose it for an MCU-based PD source, sink or DRP; for a stand-alone sink, the STUSB4500 needs no firmware.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| 7-bit I2C address | 0x28 | | 0x29 | — | bits 7:1 = 0 1 0 1 0 0 ADDR0; bit 0 = R/W | ds p. 18 |
| I2C SCL frequency Fscl | 0 | | 400 | kHz | VDD = 5 V | ds p. 19 |
| I2C bus capacitance Cb | | | 400 | pF | per line | ds p. 19 |
| Digital input VIH | 1.2 | | | V | SCL, SDA, ALERT#, A_B_SIDE, MOSI, MISO, NSS, TX_EN, SCLK | ds p. 28 |
| Digital input VIL | | | 0.35 | V | same pins | ds p. 28 |
| Digital output VOL | | | 0.4 | V | 3 mA | ds p. 28 |
| 20 V open-drain VOL (VBUS_EN_SRC, VBUS_EN_SNK) | | | 0.4 | V | 3 mA | ds p. 28 |
| Idle current IDD(SRC) | | 158 / 188 | | µA | VSYS 3.3 V / VDD 5.0 V | ds p. 27 |
| Idle current IDD(SNK) | | 113 / 140 | | µA | VSYS 3.3 V / VDD 5.0 V | ds p. 27 |
| Standby current ISTDBY | | 33 / 53 | | µA | VSYS 3.3 V / VDD 5.0 V | ds p. 27 |
| Rp current sources IP-USB / IP-1.5 / IP-3.0 | −20 % / −8 % / −8 % | 80 / 180 / 330 | +20 % / +8 % / +8 % | µA | VCC = −0.3 to 2.6 V | ds p. 27 |
| CC pull-down Rd | −10 % | 5.1 | +10 % | kΩ | | ds p. 27 |
| CC voltage in dead battery VCCDB-1.5 / VCCDB-3.0 | | | 1.2 / 2.0 | V | 180 / 330 µA into CC, VDD = 0 V | ds p. 27 |
| CC input impedance RINCC | 200 | | | kΩ | pull-up and pull-down off | ds p. 27 |
| CC detection thresholds VTH0.2 / 0.4 / 0.66 / 0.8 | 0.15 / 0.35 / 0.61 / 0.75 | 0.20 / 0.40 / 0.66 / 0.80 | 0.25 / 0.45 / 0.70 / 0.85 | V | | ds p. 27 |
| CC detection thresholds VTH1.23 / 1.6 / 2.6 | 1.16 / 1.50 / 2.45 | 1.23 / 1.60 / 2.60 | 1.31 / 1.65 / 2.75 | V | | ds p. 27 |
| VCONN path resistance RVCONN | 0.25 | 0.50 | 0.975 | Ω | IVCONN = 0.2 A | ds p. 27 |
| VCONN current limit IOCP at 100 / 350 / 600 mA setting | 85 / 300 / 550 | 100 / 350 / 600 | 125 / 400 / 650 | mA | programmable 100–600 mA, 50 mA steps; default 350 mA | ds p. 13, p. 20, p. 27–28 |
| VCONN output OVP | 5.9 | 6.0 | 6.1 | V | | ds p. 28 |
| VCONN input UVLO, low / high (default) | 2.6 / 4.6 | | 2.7 / 4.8 | V | | ds p. 28 |
| VBUS presence threshold VTHUSB | 3.8 | 3.9 | 4.0 | V | | ds p. 28 |
| VBUS safe-0 V threshold VTH0V (default) | 0.5 | 0.6 | 0.7 | V | VSYS = 3.0 to 5.5 V | ds p. 28 |
| VBUS discharge resistor RDISUSB | 600 | 700 | 800 | Ω | | ds p. 28 |
| VBUS discharge time to 0 V / to PDO, per coefficient step | 70 / 20 | 84 / 24 | 100 / 28 | ms | defaults 840 ms / 200 ms | ds p. 21, p. 28 |
| VBUS monitoring window (default) | | VBUS ±10 % | | V | programmable ±5 % to ±20 % | ds p. 28 |
| Default Type-C mode CC_POWER_MODE | | 011b | | — | dual power role with accessory support | ds p. 20, p. 21 |
| Default Rp advertisement CC_CURRENT_ADVERTISED | | 1.5 | | A | | ds p. 20 |
| Thermal resistance RθJA / RθJC | | 37 / 5 | | °C/W | | ds p. 31 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | 28 | V | ds p. 26 |
| VSYS | 6 | V | ds p. 26 |
| CC1, CC2, CC1DB, CC2DB | 22 | V | ds p. 26 |
| VBUS_EN_SRC, VBUS_EN_SNK, VBUS_SENSE | 28 | V | ds p. 26 |
| SCL, SDA, ALERT#, RESET, A_B_SIDE, MOSI, MISO, NSS, TX_EN, SCLK | −0.3 to 6 | V | ds p. 26 |
| VCONN | 6 | V | ds p. 26 |
| Storage temperature | −55 to 150 | °C | ds p. 26 |
| Maximum junction temperature | 145 | °C | ds p. 26 |
| ESD HBM / CDM | 4 / 1.5 | kV | ds p. 26 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 4.1 | 22 | V | ds p. 26 |
| VSYS | 3.0 | 5.5 | V | ds p. 26 |
| CC1, CC2, CC1DB, CC2DB (transients to −0.3 V during BMC) | −0.3 | 5.5 | V | ds p. 26 |
| VBUS_EN_SRC, VBUS_EN_SNK, VBUS_SENSE | 0 | 22 | V | ds p. 26 |
| I/O pins SCL, SDA, ALERT#, RESET, A_B_SIDE, MOSI, MISO, NSS, TX_EN, SCLK | 0 | 4.5 | V | ds p. 26 |
| VCONN voltage | 2.7 | 5.5 | V | ds p. 26 |
| VCONN current (default limit 0.35 A) | 0.1 | 0.6 | A | ds p. 26 |
| Operating temperature TA | −40 | 105 | °C | ds p. 26 |

## Pins

Direction gives the datasheet type (ds p. 3–4) and, in brackets, the library's pin kind. "MCU" means the external microcontroller.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CC1DB | CC1DB | HV AIO [bidirectional] | Dead-battery enable on CC1; tie to CC1 if used, ground in source role or if unused | ds p. 3, p. 5 |
| 2 | CC1 | CC1 | HV AIO [bidirectional] | Type-C CC1, to receptacle A5 | ds p. 3 |
| 3 | VCONN | VCONN | PWR [power_in, optional] | VCONN supply input (5 V); may float if unused | ds p. 3, p. 5 |
| 4 | CC2 | CC2 | HV AIO [bidirectional] | Type-C CC2, to receptacle B5 | ds p. 3 |
| 5 | CC2DB | CC2DB | HV AIO [bidirectional] | Dead-battery enable on CC2 | ds p. 3, p. 5 |
| 6 | RESET | RESET | DI [input] | Reset, active high | ds p. 3, p. 5 |
| 7 | SCL | SCL | DI [input] | I2C clock; external pull-up | ds p. 3 |
| 8 | SDA | SDA | DI/OD [bidirectional] | I2C data, open drain; external pull-up | ds p. 3 |
| 9 | ALERT_N | ALERT# | OD [output, optional] | I2C interrupt, active-low open drain | ds p. 3 |
| 10 | GND | GND | GND [power_in] | Ground | ds p. 3 |
| 11 | MOSI | MOSI | DO [output] | Decoded BMC RX data **from** the STUSB1602 **to** the MCU; valid on SCLK falling edge | ds p. 3, p. 5 |
| 12 | NSS | NSS | OD [output] | Chip select driven by the STUSB1602, active low; pull-up to MCU VIO | ds p. 3, p. 5 |
| 13 | ADDR0 | ADDR0 | DI [input] | I2C address bit 0, static | ds p. 4 |
| 14 | MISO | MISO | DI [input] | TX data **from** the MCU, sampled on SCLK rising edge | ds p. 4, p. 6 |
| 15 | TX_EN | TX_EN | DI [input] | TX enable from the MCU, active high; hold low (or pull down) for receive/standby | ds p. 4, p. 6 |
| 16 | SCLK | SCLK | DO, open drain [output] | Serial clock driven by the STUSB1602; pull-up to MCU VIO | ds p. 4, p. 6 |
| 17 | A_B_SIDE | A_B_SIDE | OD [output, optional] | Orientation: Hi-Z = CC1 attached, 0 = CC2 | ds p. 4, p. 6 |
| 18 | VBUS_SENSE | VBUS_SENSE | HV AI [bidirectional] | VBUS monitoring and discharge path | ds p. 4, p. 6 |
| 19 | VBUS_EN_SNK | VBUS_EN_SNK | HV AIO [bidirectional, optional] | Sink power-path enable, active-low open drain | ds p. 4, p. 6 |
| 20 | VBUS_EN_SRC | VBUS_EN_SRC | HV AIO [bidirectional, optional] | Source power-path enable, active-low open drain | ds p. 4, p. 6 |
| 21 | VREG_1V2 | VREG_1V2 | PWR [power_out] | 1.2 V internal regulator, decoupling only | ds p. 4, p. 6 |
| 22 | VSYS | VSYS | PWR [power_in] | System supply 3.3 V or 5 V; ground if unused | ds p. 4, p. 6 |
| 23 | VREG_2V7 | VREG_2V7 | PWR [power_out] | 2.7 V internal regulator, decoupling | ds p. 4, p. 6 |
| 24 | VDD | VDD | HV PWR [power_in] | Supply from VBUS; in source role, optional UVLO/OVLO sense of the source supply | ds p. 4, p. 7 |
| 25 | GND | EP (unnumbered) | GND [power_in] | Exposed pad, connected to ground; pad 25 is the library's number | ds p. 4 |

## Typical application

- Figure 8 (source application, ds p. 23):
  - VDD on VBUS; VSYS on 3.3 V with 1 µF.
  - VBUS_EN_SRC drives back-to-back PMOS (STL9P3LLH6) through 1 kΩ, with 10 kΩ gate pull-up. VBUS has 10 µF.
  - VCONN from 5 V with 10 µF.
  - CC1DB/CC2DB grounded.
  - ESD: SMM4F24A on VBUS, ESDA25L on CC1/CC2.
- Pull-ups and pull-downs in Figure 8: 10 kΩ pull-ups to VIO on MISO, MOSI, NSS, SCLK, SCL, SDA and ALERT#; 10 kΩ pull-down on TX_EN; 100 kΩ on RESET (ds p. 23).
- VREG_1V2 and VREG_2V7: 1 µF each, 0.5–10 µF (ds p. 6).
- Supply choice: VDD only, VSYS only, or both. With both present, VSYS is used when it is above 3.1 V (ds p. 22).
- Dead battery works only in sink or DRP role, with CC1DB tied to CC1 and CC2DB to CC2. Ground them in source role (ds p. 5, p. 14).
- ADDR0 is static: ground gives 0x28, pull-up gives 0x29 (ds p. 4, p. 18).
- Serial interface:
  - In RX mode (default, TX_EN low), the STUSB1602 drives NSS low, clocks SCLK and outputs data on MOSI. The MCU samples on the falling edge (ds p. 5, p. 9).
  - In TX mode (TX_EN high), the MCU presents data on MISO, which the STUSB1602 samples on the rising edge of the SCLK it generates (ds p. 6, p. 9).
  - The MCU therefore runs its SPI as a **slave**.
- ST's MCU firmware packages are named on p. 1 (STSW-STUSB010/012/014/015). The library supplies no firmware.
- Default (NVM) settings (ds p. 20–21):
  - Mode: dual power role with accessory support (011b); Rp 1.5 A.
  - VCONN supply enabled, 350 mA limit; VCONN discharge off.
  - VBUS discharge enabled: 840 ms to 0 V, 200 ms at transitions.
  - VDD OVLO detection on, VDD UVLO detection off.
  - All three alert masks set (interrupts masked).
  - Software changes are lost at reset (ds p. 20).

## Layout notes

- Recommended land (Figure 10): 24 lands 0.6 × 0.25 mm on 0.5 mm pitch, 4.4 mm across the outer land edges. The centre land is dimensioned D2 × E2, which Table 21 gives as 2.55/2.70/2.80 mm (ds p. 30). The library footprint uses 0.6 × 0.25 mm lands centred at ±1.9 mm and a 2.7 × 2.7 mm centre pad.
- Package body: 4.00 ± 0.05 mm square, 0.80–1.00 mm high (ds p. 30).
- Solder the exposed pad to ground (ds p. 4).
- Pin 1 is at the top left of Figure 2, with numbering counter-clockwise: 1–6 left, 7–12 bottom, 13–18 right, 19–24 top (ds p. 3).
- Tape: 8 mm pitch, 12 mm carrier, 13-inch reel (ds p. 31).

## Pitfalls

- Reading the SPI names from the MCU's side: here MOSI is an **output** of the STUSB1602 and MISO an **input** (ds p. 3–6). The STUSB1602 drives SCLK and NSS (ds p. 6, p. 9). Configure the MCU SPI as a slave.
- Letting TX_EN float: it must be held low by the MCU or pulled down while receiving (ds p. 4). TX mode overrides RX (ds p. 6).
- Assuming the part is in source mode at power-up: the NVM default is DRP with accessory support (ds p. 20–21). Source mode (000b) appears in Table 15 only as an NVM-customized setting for the source example (ds p. 23).
- Planning to change NVM more than once: NVM reprogramming "is possible only once" with a customer password (ds p. 20).
- Pulling I/O up to 5 V: all digital I/O, including the MCU serial lines, is rated 0–4.5 V operating (ds p. 26).
- Leaving CC1DB/CC2DB tied to CC in a source-only design: they must be grounded in source role (ds p. 5).
- Using VDD UVLO without connecting VDD to the main supply: with UVLO enabled, VDD must be connected for VBUS_EN_SRC to assert (ds p. 7).
- Treating RX mode as passive: the BMC block follows the attached CC line and the Type-C logic still applies terminations and VCONN (ds p. 8–9, p. 11).
- It does not supply VCONN in sink role (ds p. 12).

## Not established by the sources

Library versus datasheet:
- **The package application note misstates the default mode.** `lib/usb/docs/stusb1602-application.md` says "The default configuration includes source mode with accessory support … (Table 17 p23)".
  - Table 14 gives the default CC_POWER_MODE as 011b, dual power role with accessory support (ds p. 20, p. 21).
  - Source mode (000b) appears only in Table 15, the source-application example, marked as NVM-customized (ds p. 23–24).
  - Table 17 is on p. 24, not p. 23. It describes source-role connection states, not defaults.
  - The same note cites "Table 17 p23" for VCONN monitoring. The VCONN_MONITORING_EN default is in Table 14 (ds p. 20) and Table 15 (ds p. 23).
- **VREG_1V2 and VREG_2V7 are declared `power_out`.** VREG_1V2 is "used only for external decoupling" (ds p. 6). The library would let an ERC accept them as supply sources.
- **Open-drain pins are declared `output` or `bidirectional`.** NSS and ALERT# are OD; SCLK is DO but described as an "open drain output pin" (ds p. 3–4). They need pull-ups to the MCU's I/O rail, which the library cannot express.
  - VBUS_EN_SRC and VBUS_EN_SNK are high-voltage open drain (28 V absolute maximum) (ds p. 26, p. 28). The library makes them `bidirectional` with no voltage domain.
- **ALERT# is named ALERT_N in the library** (ds p. 3). This is a name change only.
- **Exposed pad numbering.** The library numbers the exposed pad 25 and puts it on GND with pad 10. The datasheet leaves EP unnumbered (ds p. 4), and Figure 8 labels it "EP 0" (ds p. 23).
- **Centre-pad and courtyard sizes are not datasheet values.**
  - Figure 10 labels the centre land only "D2"/"E2" (ds p. 30). The library's 2.7 mm takes the package's typical value.
  - Paste, mask and thermal vias are not given.
  - The 4.9 mm courtyard is the library's own.

Package documentation:
- **The pinned PDF is a third-party copy.** It came from `github.com/usb-c/STUSB1602` (application note). Whether ST's URL still serves Rev 5 was not checked (no network). The manifest table in `lib/usb/docs/README.md` has no row for this PDF; its hash is only in the application note.

Datasheet inconsistencies:
- Debug accessory detection is said to be advertised on "DEBUG1 and DEBUG2 pins" (ds p. 15). The QFN24 pinout has no such pins (ds p. 3–4).
- Figure 2 is not labelled "top view" (ds p. 3). The counter-clockwise numbering and the library both assume it is.
- Figure 8 labels pins 1 and 5 "CC1GND"/"CC2GND" (ds p. 23); Table 1 calls them CC1DB/CC2DB (ds p. 3).
- The VTH0V rows conflict. The first row gives 0.5/0.6/0.7 V, but the row labelled "Default VTH0V = 0.6 V" gives 1.7/1.8/1.9 V (ds p. 28).
- The temperature condition on IP and Rd reads "40 °C < TA < 105 °C" with no minus sign (ds p. 27).
- RESET has absolute and operating limits (ds p. 26) but is missing from the VIH/VIL list (ds p. 28). Its input thresholds are not specified.
- The power-supply text on p. 22 ("VDD pin only for applications powered by VBUS that operate either in source power role") is garbled.

Not in the datasheet:
- The I2C register map (only some addresses appear in Tables 14–15).
- Serial-interface timing (SCLK frequency, setup and hold).
- NVM programming procedure.
- Power-on time to I2C ready.
- MSL rating.
- How TX/RX behave when VDD and VSYS are both absent.
- The LCSC number comes from the library's lcsc_map.json, not from the datasheet.
