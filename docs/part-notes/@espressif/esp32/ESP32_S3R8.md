# ESP32-S3R8 — ESP32-S3 SoC with 8 MB in-package Octal SPI PSRAM (3.3 V), QFN56 7 × 7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@espressif/esp32::ESP32_S3R8_SOC",
  "parts": ["@espressif/esp32::ESP32_S3R8"],
  "identities": [
    {"mfr": "Espressif", "mpn": "ESP32-S3R8", "lcsc": "C2913194"}
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

Page numbers are PDF page indices (`pdftotext -f N -l N`). In this PDF they equal the printed page numbers. The URL is the one recorded in `lib/@espressif/esp32/docs/README.md`; the PDF itself prints `https://www.espressif.com/documentation/esp32-s3_datasheet_en.pdf` (soc p. 7). This note covers what differs from the base chip. The shared electrical, RF and package data are in `ESP32_S3.md`, which cites the same source.

## Summary

The ESP32-S3R8 is the ESP32-S3 SoC with 8 MB of Octal SPI PSRAM inside the package. It has no in-package flash, its VDD_SPI is 3.3 V, and it is rated -40 to 65 °C ambient (soc p. 13). With the PSRAM ECC function enabled, the ambient maximum rises to 85 °C, but only 15/16 of the PSRAM remains usable (soc p. 13). The PSRAM occupies SPICS1 and GPIO33–GPIO37 and shares SPICLK, SPID, SPIQ, SPIWP and SPIHD with the external flash. Those pins are not available for anything else (soc p. 31). The chip uses the same QFN56 land pattern as the base ESP32-S3 (soc p. 77). Choose it when a bare-chip design needs 8 MB of fast PSRAM and can live within 65 °C, or 85 °C with ECC. Above that, use ESP32-S3 with external memory, or a module.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| In-package PSRAM | | 8 | | MB | Octal SPI | soc p. 13 |
| In-package flash | | none | | | external SPI flash required | soc p. 13 |
| VDD_SPI voltage | | 3.3 | | V | ESP32-S3R8 | soc p. 13 |
| PSRAM supply (3.3 V part) | 2.7 | 3.3 | 3.6 | V | vendor data, not production-tested | soc p. 69 |
| PSRAM clock | 80 | | | MHz | "maximum clock frequency", listed as Min | soc p. 69 |
| Chip revision under this MPN | | v0.1 / v0.2 | | | | soc p. 13 |
| Light-sleep current | | 240 + 140 | | µA | base figure plus 8 MB 8-line PSRAM (3.3 V) | soc p. 68 |
| Usable PSRAM with ECC enabled | | 15/16 | | | permits 85 °C ambient | soc p. 13 |
| VDD_SPI output, 3.3 V mode: RSPI | | 14 | | Ω | from VDD3P3_RTC | soc p. 65 |
| Exposed pad D2 × E2 | 3.95 | 4.00 | 4.05 | mm | same land pattern as ESP32-S3 | soc p. 77 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Input power pins (VDDA, VDD3P3, VDD3P3_RTC, VDD3P3_CPU, VDD_SPI as input) | -0.3 to 3.6 | V | soc p. 29, soc p. 64 |
| Cumulative IO output current | 1500 | mA | soc p. 64 |
| Storage temperature | -40 to 150 | °C | soc p. 64 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDDA, VDD3P3, VDD3P3_RTC, VDD3P3_CPU (typ 3.3 V) | 3.0 | 3.6 | V | soc p. 64 |
| VDD3P3_CPU while burning eFuses | | 3.3 | V | soc p. 64 |
| Cumulative input current the supply can deliver | 0.5 | | A | soc p. 64 |
| Ambient temperature, ESP32-S3R8 | -40 | 65 | °C | soc p. 13 |
| Ambient temperature with PSRAM ECC enabled | -40 | 85 | °C | soc p. 13 |

## Pins

Same pad-out and names as ESP32-S3 (soc p. 15). The library makes SPICS1 and GPIO33–GPIO37 `required` here, where the base chip has them optional, so a design must net them or mark them `nc`.

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
| 28 | SPICS1 | SPICS1 | I/O (VDD_SPI) | GPIO26; SPICS1 = CE# of the in-package PSRAM; do not use | soc p. 17, soc p. 21, soc p. 31 |
| 29 | VDD_SPI | VDD_SPI | Power (in/out) | Flash/PSRAM supply and SPI IO domain; input or output | soc p. 17, soc p. 29 |
| 30 | SPIHD | SPIHD | I/O (VDD_SPI) | GPIO27; SPIHD = HOLD# / SIO3 / DQ3 | soc p. 17, soc p. 21, soc p. 31 |
| 31 | SPIWP | SPIWP | I/O (VDD_SPI) | GPIO28; SPIWP = WP# / SIO2 / DQ2 | soc p. 17, soc p. 21, soc p. 31 |
| 32 | SPICS0 | SPICS0 | I/O (VDD_SPI) | GPIO29; SPICS0 = flash CS# | soc p. 17, soc p. 21, soc p. 31 |
| 33 | SPICLK | SPICLK | I/O (VDD_SPI) | GPIO30; SPICLK = CLK | soc p. 17, soc p. 21, soc p. 31 |
| 34 | SPIQ | SPIQ | I/O (VDD_SPI) | GPIO31; SPIQ = DO / SIO1 / DQ1 | soc p. 17, soc p. 21, soc p. 31 |
| 35 | SPID | SPID | I/O (VDD_SPI) | GPIO32; SPID = DI / SIO0 / DQ0 | soc p. 17, soc p. 22, soc p. 31 |
| 36 | SPICLK_N | SPICLK_N | I/O (VDD_SPI or VDD3P3_CPU) | GPIO48, SPICLK_N_DIFF, SUBSPICLK_N_DIFF | soc p. 17, soc p. 22 |
| 37 | SPICLK_P | SPICLK_P | I/O (VDD_SPI or VDD3P3_CPU) | GPIO47, SPICLK_P_DIFF, SUBSPICLK_P_DIFF | soc p. 17, soc p. 22 |
| 38 | GPIO33 | GPIO33 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO33, FSPIHD, SUBSPIHD, SPIIO4; Octal DQ4 of the in-package PSRAM; do not use | soc p. 17, soc p. 22, soc p. 31 |
| 39 | GPIO34 | GPIO34 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO34, FSPICS0, SUBSPICS0, SPIIO5; Octal DQ5 of the in-package PSRAM; do not use | soc p. 17, soc p. 22, soc p. 31 |
| 40 | GPIO35 | GPIO35 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO35, FSPID, SUBSPID, SPIIO6; Octal DQ6 of the in-package PSRAM; do not use | soc p. 17, soc p. 22, soc p. 31 |
| 41 | GPIO36 | GPIO36 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO36, FSPICLK, SUBSPICLK, SPIIO7; Octal DQ7 of the in-package PSRAM; do not use | soc p. 17, soc p. 22, soc p. 31 |
| 42 | GPIO37 | GPIO37 | I/O (VDD_SPI or VDD3P3_CPU) | GPIO37, FSPIQ, SUBSPIQ, SPIDQS; Octal DQS/DM of the in-package PSRAM; do not use | soc p. 17, soc p. 22, soc p. 31 |
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

- Everything in `ESP32_S3.md` under "Typical application" applies: supplies, CHIP_PU timing, main crystal, strapping, USB and RF (soc p. 28, soc p. 30, soc p. 32, soc p. 42, soc p. 64).
- External flash on SPICS0 / SPICLK / SPID / SPIQ / SPIWP / SPIHD, powered from VDD_SPI (soc p. 29, soc p. 31). VDD_SPI is 3.3 V on this chip (soc p. 13), so use a 3.3 V flash.
- Leave SPICS1 and GPIO33–GPIO37 unconnected on the board. Mark them `nc` in the design: the datasheet says not to use pins connected to in-package PSRAM for anything else (soc p. 31).
- Keep GPIO45 low at reset. With the VDD_SPI eFuse not burnt, GPIO45 = 1 selects 1.8 V (soc p. 34), below the 2.7 V minimum of the 3.3 V PSRAM (soc p. 69).

## Layout notes

- Use the same QFN56 land pattern as the base ESP32-S3; only ESP32-S3FH4R2 differs (soc p. 77). The library uses `qfn::ESPRESSIF_QFN56_0P4_7B` for both.
- Treat the VDD_SPI net as a supply. It powers both the in-package PSRAM and the external flash (soc p. 29). No decoupling value is given.

## Pitfalls

- The 65 °C ambient rating, 85 °C only with ECC, is well below the base ESP32-S3's 105 °C (soc p. 13). This is easy to miss when swapping in ESP32-S3R8 for memory.
- SPICLK, SPID, SPIQ, SPIWP and SPIHD carry PSRAM traffic as DQ0–DQ3 and CLK as well as flash traffic (soc p. 31). Keep stubs short. The library comments describe them only as flash pins.
- The current-consumption tables were measured on ESP32-S3 / "ESP32-S3FH8". Chips with PSRAM may draw more (soc p. 67), and light-sleep needs +140 µA for this PSRAM (soc p. 68).
- GPIO33–GPIO37 are not free GPIOs on this chip, although they are on the base ESP32-S3 (soc p. 26, soc p. 31).

## Not established by the sources

- **The datasheet does not say whether ESP32-S3R8 leaves the factory with EFUSE_VDD_SPI_FORCE / EFUSE_VDD_SPI_TIEH burnt.** VDD_SPI is listed as 3.3 V (soc p. 13), but the control table still lets GPIO45 choose the voltage when the force bit is 0 (soc p. 34). The README and the `VDD_SPI` comment state "fixed at 3.3 V"; the datasheet gives the required voltage but not the mechanism that fixes it.
- **VDD_SPI is declared output-only.** The library declares it `[power_out]`, but the datasheet allows VDD_SPI as an input (1.8–3.6 V) as well (soc p. 29, soc p. 64).
- **The required/nc choice is the library's, not the datasheet's.** The library makes SPICS1 and GPIO33–GPIO37 `required` so that a design must acknowledge them. The datasheet says only "do not use for any other purposes" (soc p. 31). It does not say whether the package pads may be connected to anything.
- **The in-package PSRAM part number, its current and its timing are not given** beyond the supply range and the 80 MHz clock (soc p. 69). The size of the ECC penalty is given only as "1/16" (soc p. 13).
- **The datasheet does not say whether the external flash may run in Octal mode alongside the in-package Octal PSRAM.** Table 2-14 lists the pins for each mode only (soc p. 31).
- **The crystal frequency, decoupling, the CHIP_PU RC delay and RF matching are not specified** (see `ESP32_S3.md`). The datasheet gives no thermal resistance.
- **ESP32-S3R8V (1.8 V, EOL) and ESP32-S3R16V are not modeled in the library** (soc p. 13). Do not substitute either for this part: their VDD_SPI is 1.8 V (soc p. 13, soc p. 18).
