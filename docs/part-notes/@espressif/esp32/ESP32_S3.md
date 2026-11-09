# ESP32-S3 — 2.4 GHz Wi-Fi + Bluetooth LE SoC, dual-core LX7, no in-package memory, QFN56 7 × 7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@espressif/esp32::ESP32_S3_SOC",
  "parts": ["@espressif/esp32::ESP32_S3"],
  "identities": [
    {"mfr": "Espressif", "mpn": "ESP32-S3", "lcsc": "C2913192"}
  ],
  "sources": [
    {"id": "soc", "title": "ESP32-S3 Series Datasheet", "revision": "v2.2, 2026-03-05",
     "url": "https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf",
     "sha256": "2d5a7cb7fd559d8d972bd88db32669c0196d23f22d7afaafb0f63d099b589a3f",
     "local": "lib/@espressif/esp32/docs/esp32-s3_datasheet_en.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF page indices (`pdftotext -f N -l N`). In this PDF they equal the printed page numbers. The URL is the one recorded in `lib/@espressif/esp32/docs/README.md`; the PDF itself prints `https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf` (soc p. 7).

## Summary

The ESP32-S3 is Espressif's 2.4 GHz Wi-Fi (802.11b/g/n) + Bluetooth LE 5 SoC. It has a dual-core Xtensa LX7 at up to 240 MHz, 512 KB SRAM, 384 KB ROM and 45 programmable GPIOs, and comes in a 7 × 7 mm QFN56 (soc p. 3, soc p. 4, soc p. 77). This base variant has no flash or PSRAM in the package, so a board must add SPI flash on the SPI0/1 pins, which are powered from VDD_SPI. It is rated -40 to 105 °C ambient (soc p. 13, soc p. 31). The chip also needs an external main crystal (soc p. 42), and LNA_IN must reach an antenna through a matching network that this datasheet does not specify (soc p. 28, soc p. 70). Choose it over a module only if you will do the RF, crystal and flash design and the certification yourself. Otherwise use an ESP32-S3-WROOM-1 part (see `ESP32_S3_WROOM_1_SERIES.md`).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU clock | | | 240 | MHz | Xtensa dual-core LX7 | soc p. 3 |
| ROM / SRAM / RTC SRAM | | 384 / 512 / 16 | | KB | | soc p. 4 |
| Programmable GPIOs | | 45 | | | 4 are strapping pins | soc p. 4 |
| In-package flash / PSRAM | | none | | | ESP32-S3 base variant | soc p. 13 |
| Chip revision under this MPN | | v0.1 / v0.2 | | | | soc p. 13 |
| VDD_SPI output, 3.3 V mode: series resistance RSPI | | 14 | | Ω | from VDD3P3_RTC | soc p. 65 |
| VDD_SPI output, 1.8 V mode: current ISPI | | 40 | | mA | flash voltage regulator | soc p. 65 |
| Internal fast RC oscillator | | 17.5 | | MHz | "typically about", adjustable | soc p. 41 |
| Internal slow RC oscillator | | 136 | | kHz | "typically about", adjustable | soc p. 42 |
| Wi-Fi channel centre frequency | 2412 | | 2484 | MHz | | soc p. 70 |
| Wi-Fi TX power, 802.11b 1 Mbps | | 21.0 | | dBm | 0 Ω front end, 3.3 V, 25 °C | soc p. 70 |
| Wi-Fi TX power, 802.11n HT40 MCS7 | | 18.0 | | dBm | | soc p. 70 |
| Bluetooth LE TX power control range | -24.00 | 0 | 20.00 | dBm | 3 dB steps | soc p. 73 |
| Supply current, Wi-Fi TX peak | | 340 | | mA | 802.11b 1 Mbps @ 21 dBm, 100 % duty | soc p. 66 |
| Supply current, Wi-Fi RX peak | | 88 / 91 | | mA | HT20 / HT40 | soc p. 66 |
| Supply current, Bluetooth LE TX peak | | 335 | | mA | @ 21.0 dBm | soc p. 67 |
| Modem-sleep current | | 91.7 / 107.9 | | mA | 240 MHz, both cores 128-bit access; peripheral clocks off / on | soc p. 68 |
| Light-sleep / Deep-sleep / power-off current | | 240 / 7 / 1 | | µA | Deep-sleep: RTC memory on, RTC peripherals off | soc p. 68 |
| VIH / VIL | 0.75 × VDD | | 0.25 × VDD | V | VIH min / VIL max; VDD of the pin's power domain | soc p. 65 |
| VOH / VOL | 0.8 × VDD | | 0.1 × VDD | V | VOH min / VOL max | soc p. 65 |
| IOH / IOL | | 40 / 28 | | mA | VDD = 3.3 V, PAD_DRIVER = 3 | soc p. 65 |
| Default drive strength | | 20 | | mA | GPIO17/18: 10 mA; GPIO19/20: 40 mA | soc p. 18 |
| Internal weak pull-up / pull-down | | 45 | | kΩ | | soc p. 65 |
| CHIP_PU release / reset voltage | 0.75 × VDD | | 0.25 × VDD | V | VIH_nRST min / VIL_nRST max | soc p. 65 |
| Rail stabilisation before CHIP_PU high, tSTBL | 50 | | | µs | | soc p. 30 |
| CHIP_PU low time to reset, tRST | 50 | | | µs | | soc p. 30 |
| Strapping hold after CHIP_PU high, tH | 3 | | | ms | | soc p. 33 |
| ADC DNL / INL | -4 / -8 | | 4 / 8 | LSB | 100 nF on input, 25 °C, Wi-Fi off | soc p. 66 |
| ADC sampling rate | | | 100 | kSPS | | soc p. 66 |
| ADC total error after calibration | -50 | | 50 | mV | ATTEN3, 0–2900 mV (±5 mV at ATTEN0, 0–850 mV) | soc p. 66 |
| ESD HBM / CDM | | ±2000 / ±1000 | | V | JS-001 / JS-002 | soc p. 69 |
| Package body D × E | 6.95 | 7.00 | 7.05 | mm | QFN56, 0.40 mm pitch | soc p. 77 |
| Exposed pad D2 × E2 | 3.95 | 4.00 | 4.05 | mm | all variants except ESP32-S3FH4R2 | soc p. 77 |
| Package height A | 0.80 | 0.85 | 0.90 | mm | | soc p. 77 |
| Terminal width b / length L | 0.15 / 0.35 | 0.20 / 0.40 | 0.25 / 0.45 | mm | | soc p. 77 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Input power pins (VDDA, VDD3P3, VDD3P3_RTC, VDD3P3_CPU, VDD_SPI as input) | -0.3 to 3.6 | V | soc p. 29, soc p. 64 |
| Cumulative IO output current | 1500 | mA | soc p. 64 |
| Storage temperature | -40 to 150 | °C | soc p. 64 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDDA, VDD3P3 (typ 3.3 V) | 3.0 | 3.6 | V | soc p. 64 |
| VDD3P3_RTC (typ 3.3 V); account for the RSPI drop if it feeds VDD_SPI | 3.0 | 3.6 | V | soc p. 64 |
| VDD3P3_CPU (typ 3.3 V); ≤ 3.3 V while burning eFuses | 3.0 | 3.6 | V | soc p. 64 |
| VDD_SPI when used as an input (typ 3.3 V) | 1.8 | 3.6 | V | soc p. 64 |
| Cumulative input current the supply can deliver | 0.5 | | A | soc p. 64 |
| Ambient temperature, ESP32-S3 | -40 | 105 | °C | soc p. 13 |

## Pins

The library names follow the datasheet pin names. Pins 36 and 37 are named `SPICLK_N` and `SPICLK_P` but are GPIO48 and GPIO47. Pins 21 and 22 (`XTAL_32K_P`, `XTAL_32K_N`) are GPIO15 and GPIO16. `MTCK`, `MTDO`, `MTDI` and `MTMS` are GPIO39–GPIO42, and `U0TXD` and `U0RXD` are GPIO43 and GPIO44 (soc p. 21, soc p. 22).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | LNA_IN | LNA_IN | Analog | RF LNA input/output (antenna port) | soc p. 16, soc p. 28 |
| 2 | VDD3P3 | VDD3P3 | Power | Analog power domain supply | soc p. 16, soc p. 29 |
| 3 | VDD3P3 | VDD3P3 | Power | Analog power domain supply | soc p. 16, soc p. 29 |
| 4 | CHIP_PU | CHIP_PU | Input (analog) | High enables the chip, low powers it down; do not leave floating | soc p. 16, soc p. 28 |
| 5 | GPIO0 | GPIO0 | I/O (RTC) | RTC_GPIO0, GPIO0, sar_i2c_scl_0; strapping (boot mode), weak pull-up at reset | soc p. 16, soc p. 21, soc p. 23 |
| 6 | GPIO1 | GPIO1 | I/O (RTC) | RTC_GPIO1, GPIO1, TOUCH1, ADC1_CH0, sar_i2c_sda_0 | soc p. 16, soc p. 21, soc p. 24 |
| 7 | GPIO2 | GPIO2 | I/O (RTC) | RTC_GPIO2, GPIO2, TOUCH2, ADC1_CH1, sar_i2c_scl_1 | soc p. 16, soc p. 21, soc p. 24 |
| 8 | GPIO3 | GPIO3 | I/O (RTC) | RTC_GPIO3, GPIO3, TOUCH3, ADC1_CH2, sar_i2c_sda_1; strapping (JTAG source), no pull | soc p. 16, soc p. 21, soc p. 24 |
| 9 | GPIO4 | GPIO4 | I/O (RTC) | RTC_GPIO4, GPIO4, TOUCH4, ADC1_CH3 | soc p. 16, soc p. 21, soc p. 24 |
| 10 | GPIO5 | GPIO5 | I/O (RTC) | RTC_GPIO5, GPIO5, TOUCH5, ADC1_CH4 | soc p. 16, soc p. 21, soc p. 24 |
| 11 | GPIO6 | GPIO6 | I/O (RTC) | RTC_GPIO6, GPIO6, TOUCH6, ADC1_CH5 | soc p. 16, soc p. 21, soc p. 24 |
| 12 | GPIO7 | GPIO7 | I/O (RTC) | RTC_GPIO7, GPIO7, TOUCH7, ADC1_CH6 | soc p. 16, soc p. 21, soc p. 24 |
| 13 | GPIO8 | GPIO8 | I/O (RTC) | RTC_GPIO8, GPIO8, TOUCH8, ADC1_CH7, SUBSPICS1 | soc p. 16, soc p. 21, soc p. 24 |
| 14 | GPIO9 | GPIO9 | I/O (RTC) | RTC_GPIO9, GPIO9, TOUCH9, ADC1_CH8, SUBSPIHD, FSPIHD | soc p. 16, soc p. 21, soc p. 24 |
| 15 | GPIO10 | GPIO10 | I/O (RTC) | RTC_GPIO10, GPIO10, TOUCH10, ADC1_CH9, FSPIIO4, SUBSPICS0, FSPICS0 | soc p. 16, soc p. 21, soc p. 24 |
| 16 | GPIO11 | GPIO11 | I/O (RTC) | RTC_GPIO11, GPIO11, TOUCH11, ADC2_CH0, FSPIIO5, SUBSPID, FSPID | soc p. 16, soc p. 21, soc p. 24 |
| 17 | GPIO12 | GPIO12 | I/O (RTC) | RTC_GPIO12, GPIO12, TOUCH12, ADC2_CH1, FSPIIO6, SUBSPICLK, FSPICLK | soc p. 16, soc p. 21, soc p. 24 |
| 18 | GPIO13 | GPIO13 | I/O (RTC) | RTC_GPIO13, GPIO13, TOUCH13, ADC2_CH2, FSPIIO7, SUBSPIQ, FSPIQ | soc p. 16, soc p. 21, soc p. 24 |
| 19 | GPIO14 | GPIO14 | I/O (RTC) | RTC_GPIO14, GPIO14, TOUCH14, ADC2_CH3, FSPIDQS, SUBSPIWP, FSPIWP | soc p. 17, soc p. 21, soc p. 24 |
| 20 | VDD3P3_RTC | VDD3P3_RTC | Power | RTC and part of digital domain, RTC IO; feeds VDD_SPI through RSPI in 3.3 V mode | soc p. 17, soc p. 29, soc p. 34 |
| 21 | XTAL_32K_P | XTAL_32K_P | I/O (RTC) | GPIO15, RTC_GPIO15, U0RTS, ADC2_CH4; 32 kHz crystal P | soc p. 17, soc p. 21, soc p. 24 |
| 22 | XTAL_32K_N | XTAL_32K_N | I/O (RTC) | GPIO16, RTC_GPIO16, U0CTS, ADC2_CH5; 32 kHz crystal N | soc p. 17, soc p. 21, soc p. 24 |
| 23 | GPIO17 | GPIO17 | I/O (RTC) | RTC_GPIO17, GPIO17, U1TXD, ADC2_CH6 | soc p. 17, soc p. 21, soc p. 24 |
| 24 | GPIO18 | GPIO18 | I/O (RTC) | RTC_GPIO18, GPIO18, U1RXD, CLK_OUT3, ADC2_CH7 | soc p. 17, soc p. 21, soc p. 24 |
| 25 | GPIO19 | GPIO19 | I/O (RTC) | RTC_GPIO19, GPIO19, U1RTS, CLK_OUT2, ADC2_CH8, USB_D- | soc p. 17, soc p. 21, soc p. 24 |
| 26 | GPIO20 | GPIO20 | I/O (RTC) | RTC_GPIO20, GPIO20, U1CTS, CLK_OUT1, ADC2_CH9, USB_D+; USB pull-up at reset | soc p. 17, soc p. 21, soc p. 24 |
| 27 | GPIO21 | GPIO21 | I/O (RTC) | RTC_GPIO21, GPIO21 | soc p. 17, soc p. 21, soc p. 23 |
| 28 | SPICS1 | SPICS1 | I/O (VDD_SPI) | GPIO26; SPICS1 = PSRAM CE# (off-package PSRAM, recommended) | soc p. 17, soc p. 21, soc p. 31 |
| 29 | VDD_SPI | VDD_SPI | Power (in/out) | Flash/PSRAM supply and SPI IO domain; input or output | soc p. 17, soc p. 29 |
| 30 | SPIHD | SPIHD | I/O (VDD_SPI) | GPIO27; SPIHD = HOLD# / SIO3 / DQ3 | soc p. 17, soc p. 21, soc p. 31 |
| 31 | SPIWP | SPIWP | I/O (VDD_SPI) | GPIO28; SPIWP = WP# / SIO2 / DQ2 | soc p. 17, soc p. 21, soc p. 31 |
| 32 | SPICS0 | SPICS0 | I/O (VDD_SPI) | GPIO29; SPICS0 = flash CS# | soc p. 17, soc p. 21, soc p. 31 |
| 33 | SPICLK | SPICLK | I/O (VDD_SPI) | GPIO30; SPICLK = CLK | soc p. 17, soc p. 21, soc p. 31 |
| 34 | SPIQ | SPIQ | I/O (VDD_SPI) | GPIO31; SPIQ = DO / SIO1 / DQ1 | soc p. 17, soc p. 21, soc p. 31 |
| 35 | SPID | SPID | I/O (VDD_SPI) | GPIO32; SPID = DI / SIO0 / DQ0 | soc p. 17, soc p. 22, soc p. 31 |
| 36 | SPICLK_N | SPICLK_N | I/O (VDD_SPI or VDD3P3_CPU) | GPIO48, SPICLK_N_DIFF, SUBSPICLK_N_DIFF | soc p. 17, soc p. 22 |
| 37 | SPICLK_P | SPICLK_P | I/O (VDD_SPI or VDD3P3_CPU) | GPIO47, SPICLK_P_DIFF, SUBSPICLK_P_DIFF | soc p. 17, soc p. 22 |
| 38 | GPIO33 | GPIO33 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO33, FSPIHD, SUBSPIHD, SPIIO4; Octal SPI DQ4 if 8-line memory is used | soc p. 17, soc p. 22, soc p. 31 |
| 39 | GPIO34 | GPIO34 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO34, FSPICS0, SUBSPICS0, SPIIO5; Octal SPI DQ5 if 8-line memory is used | soc p. 17, soc p. 22, soc p. 31 |
| 40 | GPIO35 | GPIO35 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO35, FSPID, SUBSPID, SPIIO6; Octal SPI DQ6 if 8-line memory is used | soc p. 17, soc p. 22, soc p. 31 |
| 41 | GPIO36 | GPIO36 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO36, FSPICLK, SUBSPICLK, SPIIO7; Octal SPI DQ7 if 8-line memory is used | soc p. 17, soc p. 22, soc p. 31 |
| 42 | GPIO37 | GPIO37 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO37, FSPIQ, SUBSPIQ, SPIDQS; Octal SPI DQS/DM if 8-line memory is used | soc p. 17, soc p. 22, soc p. 31 |
| 43 | GPIO38 | GPIO38 | I/O (VDD3P3_CPU) | GPIO38, FSPIWP, SUBSPIWP | soc p. 17, soc p. 22 |
| 44 | MTCK | MTCK | I/O (VDD3P3_CPU) | MTCK (JTAG), GPIO39, CLK_OUT3, SUBSPICS1 | soc p. 17, soc p. 22 |
| 45 | MTDO | MTDO | I/O (VDD3P3_CPU) | MTDO (JTAG), GPIO40, CLK_OUT2 | soc p. 17, soc p. 22 |
| 46 | VDD3P3_CPU | VDD3P3_CPU | Power | Digital power domain and digital IO | soc p. 17, soc p. 29 |
| 47 | MTDI | MTDI | I/O (VDD3P3_CPU) | MTDI (JTAG), GPIO41, CLK_OUT1 | soc p. 17, soc p. 22 |
| 48 | MTMS | MTMS | I/O (VDD3P3_CPU) | MTMS (JTAG), GPIO42 | soc p. 17, soc p. 22 |
| 49 | U0TXD | U0TXD | I/O (VDD3P3_CPU) | U0TXD, GPIO43, CLK_OUT1; weak pull-up at reset | soc p. 17, soc p. 22 |
| 50 | U0RXD | U0RXD | I/O (VDD3P3_CPU) | U0RXD, GPIO44, CLK_OUT2; weak pull-up at reset | soc p. 17, soc p. 22 |
| 51 | GPIO45 | GPIO45 | I/O (VDD3P3_CPU) | GPIO45; strapping (VDD_SPI voltage), weak pull-down at reset | soc p. 17, soc p. 22, soc p. 32 |
| 52 | GPIO46 | GPIO46 | I/O (VDD3P3_CPU) | GPIO46; strapping (boot mode, ROM messages), weak pull-down at reset | soc p. 17, soc p. 22, soc p. 32 |
| 53 | XTAL_N | XTAL_N | Analog | Main crystal / oscillator, negative | soc p. 17, soc p. 28 |
| 54 | XTAL_P | XTAL_P | Analog | Main crystal / oscillator, positive | soc p. 17, soc p. 28 |
| 55 | VDDA | VDDA | Power | Analog power domain supply | soc p. 17, soc p. 29 |
| 56 | VDDA | VDDA | Power | Analog power domain supply | soc p. 17, soc p. 29 |
| 57 | GND | GND | Power | Exposed pad, external ground connection | soc p. 17, soc p. 29 |

## Typical application

- Power: VDDA (55, 56), VDD3P3 (2, 3), VDD3P3_RTC (20) and VDD3P3_CPU (46) from 3.0–3.6 V. A single supply should be able to deliver at least 500 mA (soc p. 64). This datasheet gives no decoupling values.
- Exposed pad 57 is the only GND pin in the pin list, so it must be soldered to ground (soc p. 29).
- CHIP_PU: never leave it floating (soc p. 28). Hold it low until all rails have been above 2.8 V for tSTBL ≥ 50 µs; pulling it low for ≥ 50 µs resets the chip (soc p. 30). This datasheet gives no RC values.
- Main crystal on XTAL_P/XTAL_N is mandatory: "ESP32-S3 is unable to operate without an external main crystal clock" (soc p. 28, soc p. 42).
- External flash: the base ESP32-S3 has no in-package memory (soc p. 13). Connect SPI flash as in Table 2-14: SPICS0 = CS#, SPICLK = CLK, SPID = DI/SIO0, SPIQ = DO/SIO1, SPIWP = WP#/SIO2, SPIHD = HOLD#/SIO3. An external PSRAM goes on SPICS1 = CE# (soc p. 31). Power the memory from VDD_SPI (soc p. 29).
- VDD_SPI voltage: by default it is 3.3 V, taken from VDD3P3_RTC through RSPI = 14 Ω. With GPIO45 high at reset (eFuse not burnt) it is 1.8 V from the flash regulator, good for 40 mA (soc p. 34, soc p. 65). In 3.3 V mode, VDD3P3_RTC must exceed VDD_flash_min + I_flash_max × RSPI (soc p. 65).
- Boot strapping: GPIO0 has a weak pull-up (1 = SPI boot). GPIO0 = 0 with GPIO46 = 0 selects joint download boot, over UART0 or USB (soc p. 32, soc p. 33). GPIO3 floats and needs an external level only if EFUSE_STRAP_JTAG_SEL is burnt (soc p. 34, soc p. 35). Hold the levels for tH ≥ 3 ms after CHIP_PU goes high (soc p. 33).
- USB: GPIO19 = USB_D-, GPIO20 = USB_D+, internal PHY (soc p. 24, soc p. 27).
- Optional 32 kHz crystal on XTAL_32K_P/XTAL_32K_N (GPIO15/GPIO16) for the RTC slow clock (soc p. 24, soc p. 42).
- RF: LNA_IN (pin 1) is the RF input/output (soc p. 28). The RF figures are measured with a 0 Ω front end (soc p. 70). The matching network and antenna are not in this datasheet.

## Layout notes

- QFN56 7.00 × 7.00 mm body, 0.40 mm pitch, 4.00 × 4.00 mm exposed pad. Pins are numbered anticlockwise from pin 1 in the top view (soc p. 77).
- The datasheet gives no land-pattern dimensions. It points to a downloadable `.asc` source file instead (soc p. 77). ESP32-S3FH4R2 needs a larger EPAD; every other variant, including this one and ESP32-S3R8, shares the land pattern (soc p. 77).
- Layout rules for crystal, RF trace, decoupling and ground are in the ESP32-S3 Hardware Design Guidelines, which this package does not pin (soc p. 82).

## Pitfalls

- SPICS0, SPICLK, SPID, SPIQ, SPIWP and SPIHD (GPIO26–GPIO32 with SPICS1) are reserved for flash/PSRAM. The datasheet ranks them priority 4, "not recommended" as GPIO (soc p. 26).
- GPIO33–GPIO37 are free only if no 8-line (Octal) flash or PSRAM is attached (soc p. 25, soc p. 26).
- GPIO33–GPIO37 and GPIO47/GPIO48 are powered from VDD_SPI or VDD3P3_CPU, chosen by eFuse (soc p. 17, soc p. 18). The SPI pins (28, 30–35) always swing at VDD_SPI, so they are at 1.8 V when GPIO45 selects the 1.8 V mode (soc p. 17).
- If a 3.3 V flash is used, GPIO45 must read 0 at reset. A board pull-up or an external driver on GPIO45 sets VDD_SPI to 1.8 V (soc p. 34).
- GPIO19/GPIO20 come up as USB Serial/JTAG with the USB pull-up enabled and glitch high twice at power-up (soc p. 17, soc p. 19). GPIO1–GPIO18 show a 60 µs low-level glitch at power-up (soc p. 18).
- U0TXD/U0RXD carry ROM boot messages by default (soc p. 34).
- Do not exceed 3.3 V on VDD3P3_CPU while burning eFuses (soc p. 64).
- The library pin names `SPICLK_N`/`SPICLK_P` hide that they are GPIO48/GPIO47, and `XTAL_32K_P`/`XTAL_32K_N` that they are GPIO15/GPIO16 (soc p. 21, soc p. 22).

## Not established by the sources

- **VDD_SPI is declared output-only.** The library declares it `required ... [power_out]`, and the source comment says "configurable input/output; default is 3.3 V output". The datasheet allows it as an input of 1.8–3.6 V (soc p. 29, soc p. 64). An agent that feeds VDD_SPI from an external rail will get a power-output pin on a supply net.
- **The main crystal frequency and its load capacitors are not given.** This datasheet states only that the crystal is required (soc p. 42). Do not take the 40 MHz from the module datasheet: that is another part's document.
- **Decoupling, the CHIP_PU RC delay, the RF matching network and the antenna are not specified.** All are deferred to the ESP32-S3 Hardware Design Guidelines (soc p. 82), which this package does not pin.
- **The land pattern cannot be checked against this PDF.** It is only in the `.asc` file (soc p. 77). The library EPAD for `qfn::ESPRESSIF_QFN56_0P4_7B` is nine 1.0 mm copper squares on a 1.5 mm grid (4.0 mm overall, matching D2/E2 = 4.00 mm nominal) plus sixteen 0.4 mm / 0.25 mm-drill vias, taken from the `.asc`.
- **The datasheet gives no thermal resistance and no junction-temperature limit.**
- **Chip revisions are not distinguished.** Revisions v0.1 and v0.2 ship under the same MPN (soc p. 13), and the library does not tell them apart. The errata and the ESP-IDF version support are in other documents (soc p. 14).
- **Datasheet inconsistencies:**
  - The modem-sleep note names ESP32-S3FH8 and ESP32-S3FN4R2 (soc p. 67), but the comparison table lists ESP32-S3FN8 and ESP32-S3FH4R2 (soc p. 13).
  - Bluetooth LE is "up to 20 dBm" (soc p. 3, soc p. 73), yet the current table has a "@ 21.0 dBm" row (soc p. 67).
- **The other memory variants are not modeled.** The library models only ESP32-S3 and ESP32-S3R8 among the bare chips. ESP32-S3FN8, ESP32-S3RH2, ESP32-S3R16V and ESP32-S3FH4R2 (soc p. 13) are omitted, per `docs/esp32-part-catalog.md`.
