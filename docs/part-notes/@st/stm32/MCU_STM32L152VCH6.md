# STM32L152VCH6 — ultra-low-power Arm Cortex-M3 MCU, 32 MHz, 256 KB flash, 32 KB RAM, 8 KB EEPROM, LCD and USB FS device, UFBGA100 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32L152VCHx",
  "parts": ["@st/stm32::MCU_STM32L152VCH6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32L152VCH6", "lcsc": "C1338995"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32L151xC STM32L152xC datasheet: Ultra-low-power 32-bit MCU ARM-based Cortex-M3, 256KB Flash, 32KB SRAM, 8KB EEPROM, LCD, USB, ADC, DAC",
     "revision": "DocID022799 Rev 13, August 2017 (25-Aug-2017)",
     "url": "https://www.st.com/resource/en/datasheet/stm32l152vc.pdf",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/dec7bad63fa1eb65659278a4d35d4909.pdf",
     "sha256": "5e6d5674cd06c1088d8cff4a0daba2283ea6d022277c94899326fe0298b928e6"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS", 136 pages, marked "Datasheet - production data"). It was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C1338995.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The `url` is the official ST URL recorded in `lib/@st/stm32/docs/stm32-part-catalog.md`. Nothing was fetched from st.com, so whether that URL serves the same revision is not established. The PDF is not in the repository; the SHA-256 is of the file that was read. Its device summary lists STM32L152VCH6 by name (ds p. 1).

**Page numbers** are one-based PDF page indices. In this file they equal the printed page numbers ("N/136").

## Summary

The STM32L152VCH6 is an ultra-low-power 32 MHz Arm Cortex-M3 microcontroller with 256 KB flash, 32 KB RAM and 8 KB data EEPROM in a 7 x 7 mm, 0.5 mm-pitch UFBGA100, for -40 to 85 °C ambient (ds p. 1, ds p. 11, ds p. 130). It has an LCD driver for up to 8x40 segments with an on-chip step-up converter, a USB 2.0 full-speed device, two op-amps, a 12-bit ADC on up to 25 channels, a 2-channel DAC and 83 GPIOs (ds p. 1, ds p. 11). Without the "D" option it runs from 1.8–3.6 V with BOR enabled (ds p. 56, ds p. 130). USB needs an HSE crystal and VDD of 3.0–3.6 V (ds p. 30, ds p. 95). Do not choose it if you need more than 32 MHz, crystal-less USB, or a board process that cannot route a 0.5 mm-pitch BGA (ds p. 56, ds p. 123).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB / APB1 / APB2 clock | 0 | | 32 | MHz | | ds p. 56 |
| Flash / RAM / data EEPROM | | 256 / 32 / 8 | | KB | | ds p. 11 |
| GPIO count | | 83 | | | STM32L15xVC | ds p. 11 |
| HSE crystal/resonator frequency | 1 | | 24 | MHz | fOSC_IN | ds p. 74 |
| HSE startup time | | 1 | | ms | VDD stabilized | ds p. 74 |
| LSE startup time | | 1 | | s | VDD stabilized | ds p. 75 |
| HSI frequency | | 16 | | MHz | VDD = 3.0 V | ds p. 77 |
| HSI accuracy, factory calibrated | -4 | | 3 | % | VDDA = 1.65–3.6 V, TA = -40 to 105 °C | ds p. 77 |
| POR/PDR threshold, falling | 1 | 1.5 | 1.65 | V | | ds p. 57 |
| POR/PDR threshold, rising | 1.3 | 1.5 | 1.65 | V | | ds p. 57 |
| BOR threshold 0, falling | 1.67 | 1.7 | 1.74 | V | | ds p. 57 |
| BOR threshold 0, rising | 1.69 | 1.76 | 1.8 | V | | ds p. 57 |
| Reset temporization, BOR enabled | | 2 | 3.3 | ms | VDD rising | ds p. 57 |
| NRST weak pull-up | 25 | 45 | 65 | kΩ | VIN = VSS | ds p. 88 |
| NRST input low level VIL(NRST) | | | 0.3 VDD | V | design | ds p. 88 |
| NRST input high level VIH(NRST) | 0.39 VDD + 0.59 | | | V | design | ds p. 88 |
| I/O weak pull-up / pull-down | 25 | 45 | 65 | kΩ | | ds p. 85 |
| BOOT0 VIL | | | 0.14 VDD | V | design | ds p. 85 |
| BOOT0 VIH | 0.15 VDD + 0.56 | | | V | design | ds p. 85 |
| VREF+ (ADC) | 1.8 | | VDDA | V | | ds p. 98 |
| VLCD, external | | | 3.6 | V | | ds p. 109 |
| VLCD step-up capacitor Cext | 0.1 | | 2 | µF | | ds p. 109 |
| USB operating VDD | 3.0 | | 3.6 | V | | ds p. 95 |
| Run current from flash | | 8.25 | 9.4 | mA | 32 MHz, range 1, HSE bypass + PLL | ds p. 61 |
| Standby current, RTC off | | 0.29 | 0.6 | µA | IWDG and LSI off, TA = -40 to 25 °C | ds p. 68 |
| Thermal resistance θJA | | 59 | | °C/W | UFBGA100 7 x 7 mm | ds p. 128 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS (including VDDA and VDD) | -0.3 to 4.0 | V | ds p. 55 |
| VIN on five-volt-tolerant pins | VSS - 0.3 to VDD + 4.0 | V | ds p. 55 |
| VIN on any other pin | VSS - 0.3 to 4.0 | V | ds p. 55 |
| VREF+ - VDDA (VREF+ > VDDA) | 0.4 max | V | ds p. 55 |
| Variation between VDD pins / between ground pins (incl. VREF-) | 50 / 50 | mV | ds p. 55 |
| Total current into all VDD / out of all VSS | 100 / 100 | mA | ds p. 55 |
| Current per VDD / VSS pin | 70 / -70 | mA | ds p. 55 |
| Current per I/O pin, sunk / sourced | 25 / -25 | mA | ds p. 55 |
| Total I/O current, sunk / sourced | 60 / -60 | mA | ds p. 55 |
| Injected current, FT, RST and B pins | -5 / +0 | mA | ds p. 55 |
| Injected current, any other pin | ±5 | mA | ds p. 55 |
| Total injected current | ±25 | mA | ds p. 55 |
| Junction temperature | 150 | °C | ds p. 56 |
| Storage temperature | -65 to +150 | °C | ds p. 56 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, BOR enabled at power-on (this part: no "D" option) | 1.8 | 3.6 | V | ds p. 56 |
| VDD, BOR disabled after power-on | 1.65 | 3.6 | V | ds p. 56 |
| VDDA, ADC and DAC not used (same voltage as VDD) | 1.65 | 3.6 | V | ds p. 56 |
| VDDA, ADC or DAC used | 1.8 | 3.6 | V | ds p. 56 |
| VIN, FT pins, VDD ≥ 2.0 V | -0.3 | 5.5 | V | ds p. 56 |
| VIN, FT pins, VDD < 2.0 V | -0.3 | 5.25 | V | ds p. 56 |
| VIN, BOOT0 | 0 | 5.5 | V | ds p. 56 |
| VIN, any other pin | -0.3 | VDD + 0.3 | V | ds p. 56 |
| Power dissipation, UFBGA100, TA = 85 °C (suffix 6) | | 339 | mW | ds p. 56 |
| Ambient temperature, suffix 6, maximum dissipation | -40 | 85 | °C | ds p. 57 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 57 |

## Pins

Checked ball by ball against Figure 3, "STM32L15xVC UFBGA100 ballout" (ds p. 32, "package top view"), and the UFBGA100 column of Table 9 (ds pp. 38–45). All 100 balls carry the signal the library gives them, and the library's ball set is exactly the package's: rows A, B, L and M full; C and K without columns 6–7; D, E, H and J without 4–9; F and G without 3–10 (ds p. 32, ds p. 122). Table 9's "alternate; additional" functions are separated by a semicolon. LCD functions work on the STM32L152 only (ds p. 11).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | PE3 | PE3 | I/O (FT) | TIM3_CH1, LCD_SEG39, TRACED0 | ds p. 38 |
| A2 | PE1 | PE1 | I/O (FT) | TIM11_CH1, LCD_SEG37 | ds p. 45 |
| A3 | PB8 | PB8 | I/O (FT) | TIM4_CH3, TIM10_CH1, I2C1_SCL, LCD_SEG16 | ds p. 45 |
| A4 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 45 |
| A5 | PD7 | PD7 | I/O (FT) | TIM9_CH2, USART2_CK | ds p. 44 |
| A6 | PD5 | PD5 | I/O (FT) | USART2_TX | ds p. 44 |
| A7 | PB4 | PB4 | I/O (FT) | NJTRST after reset; TIM3_CH1, SPI1_MISO, SPI3_MISO, LCD_SEG8; COMP2_INP | ds p. 44 |
| A8 | PB3 | PB3 | I/O (FT) | JTDO after reset; TIM2_CH2, SPI1_SCK, SPI3_SCK, I2S3_CK, LCD_SEG7; COMP2_INM | ds p. 44 |
| A9 | PA15 | PA15 | I/O (FT) | JTDI after reset; TIM2_CH1_ETR, SPI1_NSS, SPI3_NSS, I2S3_WS, LCD_SEG17 | ds p. 43 |
| A10 | PA14 | PA14 | I/O (FT) | JTCK-SWCLK after reset | ds p. 43 |
| A11 | PA13 | PA13 | I/O (FT) | JTMS-SWDIO after reset | ds p. 43 |
| A12 | PA12 | PA12 | I/O (FT) | USART1_RTS, SPI1_MOSI; USB_DP | ds p. 43 |
| B1 | PE4 | PE4 | I/O (FT) | TIM3_CH2, TRACED1 | ds p. 38 |
| B2 | PE2 | PE2 | I/O (FT) | TIM3_ETR, LCD_SEG38, TRACECLK | ds p. 38 |
| B3 | PB9 | PB9 | I/O (FT) | TIM4_CH4, TIM11_CH1, I2C1_SDA, LCD_COM3 | ds p. 45 |
| B4 | PB7 | PB7 | I/O (FT) | TIM4_CH2, I2C1_SDA, USART1_RX; COMP2_INP, PVD_IN | ds p. 45 |
| B5 | PB6 | PB6 | I/O (FT) | TIM4_CH1, I2C1_SCL, USART1_TX; COMP2_INP | ds p. 45 |
| B6 | PD6 | PD6 | I/O (FT) | USART2_RX | ds p. 44 |
| B7 | PD4 | PD4 | I/O (FT) | SPI2_MOSI, I2S2_SD, USART2_RTS | ds p. 44 |
| B8 | PD3 | PD3 | I/O (FT) | SPI2_MISO, USART2_CTS | ds p. 44 |
| B9 | PD1 | PD1 | I/O (FT) | SPI2_SCK, I2S2_CK | ds p. 44 |
| B10 | PC12 | PC12 | I/O (FT) | SPI3_MOSI, I2S3_SD, USART3_CK, LCD_SEG30, LCD_SEG42, LCD_COM6 | ds p. 44 |
| B11 | PC10 | PC10 | I/O (FT) | SPI3_SCK, I2S3_CK, USART3_TX, LCD_SEG28, LCD_SEG40, LCD_COM4 | ds p. 43 |
| B12 | PA11 | PA11 | I/O (FT) | USART1_CTS, SPI1_MISO; USB_DM | ds p. 43 |
| C1 | PC13 | PC13-WKUP2 | I/O (FT) | WKUP2, RTC_TAMP1, RTC_TS, RTC_OUT | ds p. 39 |
| C2 | PE5 | PE5 | I/O (FT) | TIM9_CH1, TRACED2 | ds p. 38 |
| C3 | PE0 | PE0 | I/O (FT) | TIM4_ETR, TIM10_CH1, LCD_SEG36 | ds p. 45 |
| C4 | VDD | VDD_3 | Power | Digital supply | ds p. 45 |
| C5 | PB5 | PB5 | I/O (FT) | TIM3_CH2, I2C1_SMBA, SPI1_MOSI, SPI3_MOSI, I2S3_SD, LCD_SEG9; COMP2_INP | ds p. 44 |
| C8 | PD2 | PD2 | I/O (FT) | TIM3_ETR, LCD_SEG31, LCD_SEG43, LCD_COM7 | ds p. 44 |
| C9 | PD0 | PD0 | I/O (FT) | TIM9_CH1, SPI2_NSS, I2S2_WS | ds p. 44 |
| C10 | PC11 | PC11 | I/O (FT) | SPI3_MISO, USART3_RX, LCD_SEG29, LCD_SEG41, LCD_COM5 | ds p. 44 |
| C11 | PH2 | PH2 | I/O (FT) | GPIO | ds p. 43 |
| C12 | PA10 | PA10 | I/O (FT) | USART1_RX, LCD_COM2 | ds p. 43 |
| D1 | PC14 | PC14-OSC32_IN | I/O (TC) | OSC32_IN (LSE) | ds p. 39 |
| D2 | PE6 | PE6-WKUP3 | I/O (FT) | TIM9_CH2, TRACED3; WKUP3, RTC_TAMP3 | ds p. 38 |
| D3 | VSS | VSS_3 | Power | Ground | ds p. 45 |
| D10 | PA9 | PA9 | I/O (FT) | USART1_TX, LCD_COM1 | ds p. 43 |
| D11 | PA8 | PA8 | I/O (FT) | USART1_CK, MCO, LCD_COM0 | ds p. 43 |
| D12 | PC9 | PC9 | I/O (FT) | TIM3_CH4, LCD_SEG27 | ds p. 43 |
| E1 | PC15 | PC15-OSC32_OUT | I/O (TC) | OSC32_OUT (LSE) | ds p. 39 |
| E2 | VLCD | VLCD | Power | LCD supply (STM32L152); on STM32L151 connect to VDD | ds p. 38 |
| E3 | VSS | VSS_4 | Power | Ground | ds p. 40 |
| E10 | PC8 | PC8 | I/O (FT) | TIM3_CH3, LCD_SEG26 | ds p. 43 |
| E11 | PC7 | PC7 | I/O (FT) | TIM3_CH2, I2S3_MCK, LCD_SEG25 | ds p. 43 |
| E12 | PC6 | PC6 | I/O (FT) | TIM3_CH1, I2S2_MCK, LCD_SEG24 | ds p. 42 |
| F1 | PH0 | PH0-OSC_IN | I/O (TC) | OSC_IN (HSE) | ds p. 39 |
| F2 | VSS | VSS_5 | Power | Ground | ds p. 39 |
| F11 | VSS | VSS_2 | Power | Ground | ds p. 43 |
| F12 | VSS | VSS_1 | Power | Ground | ds p. 41 |
| G1 | PH1 | PH1-OSC_OUT | I/O (TC) | OSC_OUT (HSE) | ds p. 39 |
| G2 | VDD | VDD_5 | Power | Digital supply | ds p. 39 |
| G11 | VDD | VDD_2 | Power | Digital supply | ds p. 43 |
| G12 | VDD | VDD_1 | Power | Digital supply | ds p. 41 |
| H1 | PC0 | PC0 | I/O (FT) | LCD_SEG18; ADC_IN10, COMP1_INP | ds p. 39 |
| H2 | NRST | NRST | I/O (RST) | Reset, active low | ds p. 39 |
| H3 | VDD | VDD_4 | Power | Digital supply | ds p. 40 |
| H10 | PD15 | PD15 | I/O (FT) | TIM4_CH4, LCD_SEG35 | ds p. 42 |
| H11 | PD14 | PD14 | I/O (FT) | TIM4_CH3, LCD_SEG34 | ds p. 42 |
| H12 | PD13 | PD13 | I/O (FT) | TIM4_CH2, LCD_SEG33 | ds p. 42 |
| J1 | VSSA | VSSA | Power | Analog ground | ds p. 39 |
| J2 | PC1 | PC1 | I/O (FT) | LCD_SEG19; ADC_IN11, COMP1_INP | ds p. 39 |
| J3 | PC2 | PC2 | I/O (FT) | LCD_SEG20; ADC_IN12, COMP1_INP | ds p. 39 |
| J10 | PD12 | PD12 | I/O (FT) | TIM4_CH1, USART3_RTS, LCD_SEG32 | ds p. 42 |
| J11 | PD11 | PD11 | I/O (FT) | USART3_CTS, LCD_SEG31 | ds p. 42 |
| J12 | PD10 | PD10 | I/O (FT) | USART3_CK, LCD_SEG30 | ds p. 42 |
| K1 | VREFM | VREF- | Power | ADC/DAC negative reference | ds p. 39 |
| K2 | PC3 | PC3 | I/O (TC) | LCD_SEG21; ADC_IN13, COMP1_INP | ds p. 39 |
| K3 | PA2 | PA2 | I/O (FT) | TIM2_CH3, TIM5_CH3, TIM9_CH1, USART2_TX, LCD_SEG1; ADC_IN2, COMP1_INP, OPAMP1_VINM | ds p. 40 |
| K4 | PA5 | PA5 | I/O (TC) | TIM2_CH1_ETR, SPI1_SCK; ADC_IN5, DAC_OUT2, COMP1_INP | ds p. 40 |
| K5 | PC4 | PC4 | I/O (FT) | LCD_SEG22; ADC_IN14, COMP1_INP | ds p. 40 |
| K8 | PD9 | PD9 | I/O (FT) | USART3_RX, LCD_SEG29 | ds p. 42 |
| K9 | PD8 | PD8 | I/O (FT) | USART3_TX, LCD_SEG28 | ds p. 42 |
| K10 | PB15 | PB15 | I/O (FT) | TIM11_CH1, SPI2_MOSI, I2S2_SD, LCD_SEG15; ADC_IN21, COMP1_INP, RTC_REFIN | ds p. 42 |
| K11 | PB14 | PB14 | I/O (FT) | TIM9_CH2, SPI2_MISO, USART3_RTS, LCD_SEG14; ADC_IN20, COMP1_INP | ds p. 42 |
| K12 | PB13 | PB13 | I/O (FT) | TIM9_CH1, SPI2_SCK, I2S2_CK, USART3_CTS, LCD_SEG13; ADC_IN19, COMP1_INP | ds p. 42 |
| L1 | VREFP | VREF+ | Power | ADC/DAC positive reference | ds p. 39 |
| L2 | PA0 | PA0-WKUP1 | I/O (FT) | TIM2_CH1_ETR, TIM5_CH1, USART2_CTS; WKUP1, RTC_TAMP2, ADC_IN0, COMP1_INP | ds p. 39 |
| L3 | PA3 | PA3 | I/O (TC) | TIM2_CH4, TIM5_CH4, TIM9_CH2, USART2_RX, LCD_SEG2; ADC_IN3, COMP1_INP, OPAMP1_VOUT | ds p. 40 |
| L4 | PA6 | PA6 | I/O (FT) | TIM3_CH1, TIM10_CH1, SPI1_MISO, LCD_SEG3; ADC_IN6, COMP1_INP, OPAMP2_VINP | ds p. 40 |
| L5 | PC5 | PC5 | I/O (FT) | LCD_SEG23; ADC_IN15, COMP1_INP | ds p. 40 |
| L6 | PB2 | PB2 | I/O (FT) | BOOT1 (main function after reset); VLCDRAIL1, ADC_IN0b | ds p. 41 |
| L7 | PE8 | PE8 | I/O (TC) | ADC_IN23, COMP1_INP | ds p. 41 |
| L8 | PE10 | PE10 | I/O (TC) | TIM2_CH2; ADC_IN25, COMP1_INP | ds p. 41 |
| L9 | PE12 | PE12 | I/O (FT) | TIM2_CH4, SPI1_NSS; VLCDRAIL3 | ds p. 41 |
| L10 | PB10 | PB10 | I/O (FT) | TIM2_CH3, I2C2_SCL, USART3_TX, LCD_SEG10 | ds p. 41 |
| L11 | PB11 | PB11 | I/O (FT) | TIM2_CH4, I2C2_SDA, USART3_RX, LCD_SEG11 | ds p. 41 |
| L12 | PB12 | PB12 | I/O (FT) | TIM10_CH1, I2C2_SMBA, SPI2_NSS, I2S2_WS, USART3_CK, LCD_SEG12; ADC_IN18, COMP1_INP, VLCDRAIL2 | ds p. 42 |
| M1 | VDDA | VDDA | Power | Analog supply | ds p. 39 |
| M2 | PA1 | PA1 | I/O (FT) | TIM2_CH2, TIM5_CH2, USART2_RTS, LCD_SEG0; ADC_IN1, COMP1_INP, OPAMP1_VINP | ds p. 40 |
| M3 | PA4 | PA4 | I/O (TC) | SPI1_NSS, SPI3_NSS, I2S3_WS, USART2_CK; ADC_IN4, DAC_OUT1, COMP1_INP | ds p. 40 |
| M4 | PA7 | PA7 | I/O (FT) | TIM3_CH2, TIM11_CH1, SPI1_MOSI, LCD_SEG4; ADC_IN7, COMP1_INP, OPAMP2_VINM | ds p. 40 |
| M5 | PB0 | PB0 | I/O (TC) | TIM3_CH3, LCD_SEG5; ADC_IN8, COMP1_INP, OPAMP2_VOUT, VLCDRAIL3, VREF_OUT | ds p. 40 |
| M6 | PB1 | PB1 | I/O (FT) | TIM3_CH4, LCD_SEG6; ADC_IN9, COMP1_INP, VREF_OUT | ds p. 41 |
| M7 | PE7 | PE7 | I/O (TC) | ADC_IN22, COMP1_INP | ds p. 41 |
| M8 | PE9 | PE9 | (TC) | TIM2_CH1_ETR, TIM5_ETR; ADC_IN24, COMP1_INP | ds p. 41 |
| M9 | PE11 | PE11 | I/O (FT) | TIM2_CH3; VLCDRAIL2 | ds p. 41 |
| M10 | PE13 | PE13 | I/O (FT) | SPI1_SCK | ds p. 41 |
| M11 | PE14 | PE14 | I/O (FT) | SPI1_MISO | ds p. 41 |
| M12 | PE15 | PE15 | I/O (FT) | SPI1_MOSI | ds p. 41 |

## Typical application

- Decouple VDD with N x 100 nF plus 1 x 4.7 µF, N being the number of VDD/VSS pairs (five on this package), and VDDA with 100 nF + 1 µF; VREF+ gets its own 100 nF + 1 µF when it is not tied to VDDA (Figure 12, ds p. 53).
- Connect every VDD, VDDA, VSS and VSSA pin; VDDA and VSSA must be connected to VDD and VSS (ds p. 19, ds p. 55). Power VDD and VDDA from the same source: at most 300 mV between them in power-up and 140 mV in operation (ds p. 57).
- VREF+ (L1) between 1.8 V and VDDA for the ADC, never more than 0.4 V above VDDA; VREF- (K1) is the ADC/DAC negative reference and counts as a ground pin (ds p. 55, ds p. 98).
- VLCD (E2): with the LCD, either feed it from a dedicated supply with 100 nF (option 1) or let the internal step-up converter drive it with Cext of 0.1–2 µF (option 2) (ds p. 54, ds p. 109). On an STM32L151xC it is tied to VDD (ds p. 45).
- NRST has an internal 25–65 kΩ pull-up. Figure 20 adds 0.1 µF from NRST to ground, placed as close as possible to the chip (ds p. 88, ds p. 89).
- Boot: BOOT0 (A4) and BOOT1 on PB2 (L6) select flash, system memory or RAM. The bootloader uses USART1, USART2 or USB (ds p. 20, ds p. 41). Strap BOOT0 low for flash boot.
- HSE: a 1–24 MHz crystal on PH0/PH1 (F1/G1) with CL1 = CL2 in 5–25 pF, sized with about 10 pF of pin and board capacitance (ds p. 74). It is required for USB (ds p. 30).
- LSE: a 32.768 kHz crystal on PC14/PC15 (D1/E1) with CL1 = CL2 in 5–15 pF. ST recommends CL ≤ 7 pF and never 12.5 pF (ds p. 76).
- USB: PA11 (B12) = USB_DM, PA12 (A12) = USB_DP, VDD 3.0–3.6 V (ds p. 43, ds p. 95).
- Debug: SWD on PA13 (A11) and PA14 (A10); full JTAG also takes PA15 (A9), PB3 (A8) and PB4 (A7) after reset (ds pp. 43–44).

## Layout notes

- ST's recommended land (Figure 45, Table 71): 0.5 mm pitch, 0.280 mm pad (Dpad), 0.370 mm solder-mask opening (Dsm, typical, depends on mask registration), 0.280 mm stencil opening, 0.100–0.125 mm stencil (ds p. 123).
- Package (Figure 44, Table 70): body 7.000 mm (6.950–7.050), ball pitch 0.500 mm, ball diameter 0.200–0.300 mm, ball-array span D1 = E1 = 5.500 mm, height A = 0.600 mm max (ds p. 122).
- Ball A1 is at the top left of the top view, letters run down and numbers run right (ds p. 32). The A1 index is a corner mark on the top of the package (ds p. 122).
- Place crystals and load capacitors as close as possible to the oscillator pins (ds p. 74, ds p. 75).

## Pitfalls

- 32 MHz needs voltage range 1 (ds p. 16); in Table 3 every supply range from 1.71 V up allows range 1 (ds pp. 15–16). The ADC/DAC need VDDA ≥ 1.8 V and USB needs VDD ≥ 3.0 V (ds p. 15, ds p. 56, ds p. 95).
- Not every pin is 5 V tolerant. PC14, PC15, PH0, PH1, PC3, PA3, PA4, PA5, PB0 and PE7–PE10 are TC (standard 3.3 V) (ds pp. 39–41); the cover gives 70 of 83 I/Os as 5 V tolerant (ds p. 1). FT inputs above VDD + 0.3 V need the internal pull-up/down disabled (ds p. 57).
- PA13–PA15, PB3 and PB4 come out of reset as JTAG/SWD pins (ds pp. 43–44).
- PH0/PH1 and PC14/PC15 become oscillator pins only when HSE or LSE is switched on; the oscillator then has priority (ds p. 45).
- The library binds this part with `supplier lcsc "C1338995" rotate 270`. The datasheet establishes the ball map and the A1 corner, not the assembler's tape or preview frame, so it cannot confirm the quarter turn. `lib/@st/stm32/docs/README.md` records that binding as resting on weaker evidence than the LQFP48 ones.

## Not established by the sources

- **Ball map: no disagreement.** All 100 library balls match Figure 3 and Table 9 (ds p. 32, ds pp. 38–45). The footprint `bga::KICAD_UFBGA_100_7X7MM_LAYOUT12X12_P0_5MM` puts A1 at (-2.75, -2.75) mm, top left in the library's +y-down frame, with B1 below it and A2 to its right; that is the top view of Figure 3. Its pads are 0.28 mm circles with 0.045 mm mask expansion (0.37 mm opening) at 0.5 mm pitch, which matches Table 71 (ds p. 123).
- **NRST is typed `input` in the library**; Table 9 gives it I/O structure RST and Table 46 gives an NRST output-low level (ds p. 39, ds p. 88).
- **VLCD is typed `passive` and `required`.** The datasheet says what to do with it on an STM32L151xC (tie to VDD) and with the LCD in use, but not what to do on an STM32L152 whose LCD is unused (ds p. 45, ds p. 54).
- **VLCD rail pins disagree inside the datasheet.** Table 6 and Table 9 put VLCDRAIL1 on PB2, VLCDRAIL2 on PB12 or PE11 and VLCDRAIL3 on PB0 or PE12 (ds p. 25, ds pp. 40–42). Figure 13 labels PB2 as VLCDrail2 and PB12 or PE11 as VLCDrail1 (ds p. 54).
- **Small source errors.** Table 9 gives PE9 (M8) no pin type (ds p. 41). Note 4 of Table 9 says the LSE pins can be used as "PH0/PH1" I/Os; the row it annotates is PC14/PC15 (ds p. 45).
- **D+ pull-up.** Table 52 gives the USB levels with a 1.5 kΩ load but does not say whether the part has an internal D+ pull-up (ds p. 95).
- **BOOT0/BOOT1 truth table** is not in this datasheet; it refers to AN2606 (ds p. 20). No BOOT0 resistor value is given.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- **Via and escape rules** for the 0.5 mm-pitch array are not given beyond Table 71 (ds p. 123).
- No crystal ESR or drive-level limit is given beyond AN2867 (ds p. 74).
