# STUSB4500 application contract

Source: [ST DS12499](https://www.st.com/resource/en/datasheet/stusb4500.pdf). The bundled `st-stusb4500-rev7.pdf` is **Rev. 7 (June 2021)**, captured 2026-10-16 from the [USB-C driver repository mirror](https://github.com/usb-c/STUSB4500/blob/master/Doc/stusb4500_datasheet_v7.pdf), SHA-256 `407ee6767128cc4075926e01d945fe846440f932359ed81f45a957485778399e`. ST's binary download failed; Rev. 8 (November 2022) was additionally read at the official URL. Its p37 change log updates Table 3's warning and adds NVM retention/cycling parameters. All page references below are to bundled Rev. 7 unless marked Rev. 8.

## Identity and purpose

**STUSB4500QTR** is the QFN24 EP variant. It autonomously negotiates sink power using up to three configured PDOs, up to 20 V / 5 A. ST describes PD 2.0 certification and PD 3.0 interoperability (p1); this is not a claim of full PD 3.0/PPS/EPR support. It is not a passive sniffer, a source controller, or interchangeable with the Type-C-only STUSB4500L. The WLCSP variant is not bound to this footprint.

## Connections

Table 1 p4 and Fig. 2 p3 are the pin oracle. NC pin 3 must float. GND pin 10 and the exposed pad (library pad 25) go to ground. VREG_1V2 and VREG_2V7 each require separate 1 uF typical decoupling (0.5-10 uF); do not power external loads from them (p9).

Table 21 p28: VDD 4.1-22 V; VSYS 3.0-5.5 V; digital I/O 0-4.5 V; CC 0-5.5 V. The 28 V VDD and 22 V CC absolute/protection limits are not operating ratings. Ground unused VSYS. Connect CC1DB/CC2DB to corresponding CC pins for dead-battery operation; otherwise ground them (p5). RESET is active high.

I2C address is 0x28 | (ADDR1 << 1) | ADDR0, 7-bit (p17); use external pull-ups for SCL/SDA/ALERT. Ground both address pins when there is no MCU. **Rev. 8 Table 3 p6 clarifies that SCL and SDA are pulled down when neither VDD nor VSYS is present**; account for this on a shared powered I2C bus. POWER_OK2 is a high-voltage open-drain output; POWER_OK3/GPIO/ATTACH/A_B_SIDE/ALERT are low-voltage open-drain outputs. They are not interchangeable pull-up domains.

VBUS_VS_DISCH both senses and discharges VBUS. A series resistor must limit its discharge current to 50 mA (p9); it is not a high-impedance-only sense pin. DISCH may control an external path or discharge internally with a current-limiting resistor, max 500 mA (p6). The controller drives external power FETs; it does not itself carry the full 5 A load. Select power path, reverse blocking, connector/cable ratings and protection from the application's actual requirements. NVM PDO programming must match them.

## Land and validation boundary

Fig. 12 p32: 0.6 x 0.25 mm copper leads, 0.5 mm pitch, 4.4 mm outer copper span; +/-1.9 mm row centers. EP uses typical D2/E2 2.7 mm. The shared ST QFN24 land is also checked against STUSB1602 DS11254 Fig. 10. Courtyard 4.9 mm adds a library-authored 0.25 mm margin. Mask/paste/thermal vias, sourcing and electrical/fabrication sign-off are separate checks.
