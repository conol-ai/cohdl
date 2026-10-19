# STUSB1602A application contract

Source: [ST DS11254](https://www.st.com/resource/en/datasheet/stusb1602.pdf), Rev. 5 (July 2021), captured 2026-10-16 as `st-stusb1602-rev5.pdf`. SHA-256: `eb35151e98a6b9fbfb76fccdf277759b48ad72da472090989fb30b484a063e79`. Downloaded unchanged from the [USB-C driver repository mirror](https://github.com/usb-c/STUSB1602/blob/master/Docs/STUSB1602_datasheet_v5.pdf) because ST's download endpoint failed. The source identity is ST's document, not third-party pin data.

## Identity and firmware interface

The audited order code is **STUSB1602AQTR**, QFN24 EP 4x4 mm (p1); do not silently substitute the earlier non-A device. It combines Type-C control with a BMC transceiver and needs an external MCU/compatible protocol stack (p17). The library does not supply firmware.

Table 1 pp3-4 and Fig. 2 p3 fix the pins. MOSI pin 11 is an **output from the STUSB1602 to the MCU**; MISO pin 14 is an **input from the MCU**. SCLK pin 16 and NSS pin 12 are outputs: the MCU uses a compatible SPI/MSP **slave** interface. In RX, data are valid on SCLK's falling edge; TX samples on the rising edge (p9). TX_EN pin 15 must stay low during receive/standby. NSS/SCLK are open drain; use the external pull-ups shown in Table 1. CoHDL roles do not encode open-drain behavior.

I2C SCL/SDA and ALERT_N need external pull-ups. ADDR0 selects 0x28/0x29, 7-bit (Table 10 p18); do not confuse I2C control with the serial BMC data path. RESET is active high.

## Supplies and startup

Table 19 p26: VDD 4.1-22 V; VSYS 3.0-5.5 V; VCONN 2.7-5.5 V when used; ordinary digital I/O 0-4.5 V; CC operating -0.3 to 5.5 V. The 22 V CC protection and 28 V supply stress limits are not normal operating limits. Each VREG output needs its own 1 uF typical capacitor (0.5-10 uF); these are decoupling nodes, not external load rails (p6). Ground unused VSYS. VCONN may be left floating if unused (p5); disable/check VCONN monitoring for the selected application (Table 17 p23).

CC1DB/CC2DB connect to their corresponding CC pins only when dead-battery support is intended; otherwise ground them (p5). Internal Type-C terminations/attach state and VBUS discharge remain independent of the RX/TX_EN choice. The default configuration includes source mode with accessory support and enabled VBUS discharge (Table 17 p23).

## Passive-capture limitation and land

RX mode alone does **not** prove a passive sniffer. The BMC path follows the attached CC line (p9); the datasheet specifies at least 200 kohm CC input resistance with Rp/Rd off (Table 20 p27), not an isolated monitor across all power states. A sniffer requires separately evidenced termination control, power-loss isolation, orientation/VCONN handling and protocol decoding/timing.

Fig. 10 p30 supplies 0.6 x 0.25 mm leads, 0.5 mm pitch and 4.4 mm copper span. Centers are +/-1.9 mm. EP uses typical D2/E2 = 2.7 mm and library terminal 25, joined with ground pin 10. The shared ST land is qualified against this drawing and STUSB4500's corresponding drawing. Courtyard 4.9 mm is a library-authored 0.25 mm margin. Mask/paste/thermal-via design, electrical validation and stock checks remain board responsibilities.
