# STM32F072CBT6 — Arm Cortex-M0 MCU, 128 KB flash, crystal-less USB FS, LQFP48 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32F072CBTx",
  "parts": ["@st/stm32::MCU_STM32F072CBT6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32F072CBT6", "lcsc": "C81720"},
    {"mfr": "STMicroelectronics", "mpn": "STM32F072CBT6TR", "lcsc": "C2969805"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32F072x8 STM32F072xB datasheet: Arm-based 32-bit MCU, up to 128 KB Flash, crystal-less USB FS 2.0, CAN, 12 timers, ADC, DAC & comm. interfaces, 2.0 - 3.6 V",
     "revision": "DS9826 Rev 6, 2019-09-03",
     "url": "https://www.st.com/resource/en/datasheet/stm32f072cb.pdf",
     "sha256": "660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453",
     "local": "lib/@st/stm32/docs/stm32f072cb-datasheet.pdf"}
  ],
  "status": "draft"
}
```

## Summary

The STM32F072CBT6 is a 48 MHz Arm Cortex-M0 microcontroller with 128 KB flash and 16 KB SRAM in a 7 x 7 mm LQFP48, rated -40 to 85 °C ambient (suffix 6) (ds p. 1, ds p. 11, ds p. 124). Its deciding feature is a USB 2.0 full-speed device with an embedded D+ pull-up that can run from the internal 48 MHz oscillator trimmed by the CRS from USB SOF, so a USB device needs no crystal (ds p. 27). It also has CAN, four USARTs, two SPI/I2S, two I2C, a 12-bit ADC and a 2-channel DAC, and a separate VDDIO2 rail for 19 I/Os (ds p. 1). The LQFP48 variant has 37 GPIOs and 10 external ADC channels (ds p. 11). Do not choose it when you need more than 48 MHz (ds p. 53) or USB host/OTG: the datasheet describes a USB device peripheral only (ds p. 27).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU / AHB / APB clock | 0 | | 48 | MHz | fHCLK, fPCLK | ds p. 53 |
| Flash / SRAM | | 128 / 16 | | KB | STM32F072CB | ds p. 11 |
| GPIO count | | 37 | | | STM32F072Cx | ds p. 11 |
| HSE crystal/resonator frequency | 4 | 8 | 32 | MHz | fOSC_IN | ds p. 68 |
| HSE startup time | | 2 | | ms | VDD stabilized, standard crystal | ds p. 68 |
| HSI48 accuracy (untrimmed by CRS) | -4.9 | | 4.7 | % | TA = -40 to 105 °C | ds p. 73 |
| HSI48 startup time | | | 6 | µs | | ds p. 73 |
| POR/PDR threshold, rising | 1.84 | 1.92 | 2.00 | V | | ds p. 54 |
| POR/PDR threshold, falling | 1.80 | 1.88 | 1.96 | V | | ds p. 54 |
| Reset temporization | 1.50 | 2.50 | 4.50 | ms | tRSTTEMPO | ds p. 54 |
| NRST internal pull-up | 25 | 40 | 55 | kΩ | VIN = VSS | ds p. 84 |
| BOOT0 VIH | 0.2 VDDIOx + 0.95 | | | V | | ds p. 78 |
| BOOT0 VIL | | | 0.3 VDDIOx - 0.3 | V | | ds p. 78 |
| USB D+ pull-up, idle | 1.1 | 1.26 | 1.5 | kΩ | RPUI, embedded | ds p. 99 |
| USB driver impedance | 28 | 40 | 44 | Ω | ZDRV, no series resistors needed | ds p. 99 |
| Run current from VDD | | 24.3 | 26.9 | mA | HSI48, 48 MHz, all peripherals on, VDD = 3.6 V, max at 25 °C | ds p. 56 |
| Thermal resistance θJA | | 54 | | °C/W | LQFP48 7 x 7 mm | ds p. 121 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS, VDDIO2 - VSS, VDDA - VSS, VBAT - VSS | -0.3 to 4.0 | V | ds p. 51 |
| VDD - VDDA (VDD > VDDA) | 0.4 max | V | ds p. 51 |
| VIN on FT/FTf pins | VSS - 0.3 to VDDIOx + 4.0 (4.0 if internal pull-up/down enabled) | V | ds p. 51 |
| VIN on TTa pins and any other pin | VSS - 0.3 to 4.0 | V | ds p. 51 |
| VIN on BOOT0 | 0 to 9.0 | V | ds p. 51 |
| Current per I/O pin, sunk / sourced | 25 / -25 | mA | ds p. 52 |
| Total I/O current, sunk / sourced | 80 / -80 | mA | ds p. 52 |
| Total sourced by VDDIO2-supplied I/Os | -40 | mA | ds p. 52 |
| Total into VDD lines / out of VSS lines | 120 / -120 | mA | ds p. 52 |
| Injected current, B/FT/FTf pins | -5 / +0 | mA | ds p. 52 |
| Junction temperature | 150 | °C | ds p. 52 |
| Storage temperature | -65 to +150 | °C | ds p. 52 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 2.0 | 3.6 | V | ds p. 53 |
| VDDA, ADC and DAC not used | VDD | 3.6 | V | ds p. 53 |
| VDDA, ADC or DAC used | 2.4 | 3.6 | V | ds p. 53 |
| VDDIO2 (only with VDD present) | 1.65 | 3.6 | V | ds p. 53 |
| VDDIO2 for full USB electrical spec | 3.0 | 3.6 | V | ds p. 99 |
| VBAT | 1.65 | 3.6 | V | ds p. 53 |
| VIN, FT and FTf I/O | -0.3 | 5.5 | V | ds p. 53 |
| VIN, TTa I/O | -0.3 | VDDA + 0.3 | V | ds p. 53 |
| VIN, TC and RST I/O | -0.3 | VDDIOx + 0.3 | V | ds p. 53 |
| VIN, BOOT0 | 0 | 5.5 | V | ds p. 53 |
| Power dissipation, LQFP48, TA = 85 °C (suffix 6) | | 370 | mW | ds p. 53 |
| Ambient temperature, suffix 6, max dissipation | -40 | 85 | °C | ds p. 53 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 53 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VBAT | VBAT | Power | Backup supply (RTC, LSE, backup registers) | ds p. 36 |
| 2 | PC13 | PC13 | I/O (TC) | GPIO; WKUP2, RTC_TAMP1, RTC_TS, RTC_OUT | ds p. 37 |
| 3 | PC14 | PC14-OSC32_IN | I/O (TC) | GPIO; OSC32_IN (LSE) | ds p. 37 |
| 4 | PC15 | PC15-OSC32_OUT | I/O (TC) | GPIO; OSC32_OUT (LSE) | ds p. 37 |
| 5 | PF0 | PF0-OSC_IN | I/O (FT) | GPIO; OSC_IN (HSE), CRS_SYNC | ds p. 37 |
| 6 | PF1 | PF1-OSC_OUT | I/O (FT) | GPIO; OSC_OUT (HSE) | ds p. 37 |
| 7 | NRST | NRST | I/O (RST) | Reset input / internal reset output, active low | ds p. 37 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 37 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 37 |
| 10 | PA0 | PA0 | I/O (TTa) | ADC_IN0, WKUP1, USART2_CTS, TIM2_CH1_ETR, USART4_TX | ds p. 37 |
| 11 | PA1 | PA1 | I/O (TTa) | ADC_IN1, USART2_RTS, TIM2_CH2, USART4_RX | ds p. 38 |
| 12 | PA2 | PA2 | I/O (TTa) | ADC_IN2, USART2_TX, TIM2_CH3, WKUP4 | ds p. 38 |
| 13 | PA3 | PA3 | I/O (TTa) | ADC_IN3, USART2_RX, TIM2_CH4 | ds p. 38 |
| 14 | PA4 | PA4 | I/O (TTa) | ADC_IN4, DAC_OUT1, SPI1_NSS, I2S1_WS | ds p. 38 |
| 15 | PA5 | PA5 | I/O (TTa) | ADC_IN5, DAC_OUT2, SPI1_SCK, CEC | ds p. 38 |
| 16 | PA6 | PA6 | I/O (TTa) | ADC_IN6, SPI1_MISO, TIM3_CH1, TIM16_CH1 | ds p. 38 |
| 17 | PA7 | PA7 | I/O (TTa) | ADC_IN7, SPI1_MOSI, TIM3_CH2, TIM1_CH1N | ds p. 38 |
| 18 | PB0 | PB0 | I/O (TTa) | ADC_IN8, TIM3_CH3, TIM1_CH2N, USART3_CK | ds p. 39 |
| 19 | PB1 | PB1 | I/O (TTa) | ADC_IN9, TIM3_CH4, TIM14_CH1, USART3_RTS | ds p. 39 |
| 20 | PB2 | PB2 | I/O (FT) | GPIO; TSC_G3_IO4 | ds p. 39 |
| 21 | PB10 | PB10 | I/O (FT) | I2C2_SCL, USART3_TX, SPI2_SCK, TIM2_CH3 | ds p. 39 |
| 22 | PB11 | PB11 | I/O (FT) | I2C2_SDA, USART3_RX, TIM2_CH4 | ds p. 39 |
| 23 | VSS | VSS | Power | Ground | ds p. 39 |
| 24 | VDD | VDD | Power | Digital supply | ds p. 40 |
| 25 | PB12 | PB12 | I/O (FT) | SPI2_NSS, I2S2_WS, TIM1_BKIN, USART3_CK | ds p. 40 |
| 26 | PB13 | PB13 | I/O (FTf) | SPI2_SCK, I2C2_SCL, TIM1_CH1N, USART3_CTS | ds p. 40 |
| 27 | PB14 | PB14 | I/O (FTf) | SPI2_MISO, I2C2_SDA, TIM1_CH2N, USART3_RTS | ds p. 40 |
| 28 | PB15 | PB15 | I/O (FT) | SPI2_MOSI, TIM1_CH3N, RTC_REFIN, WKUP7 | ds p. 40 |
| 29 | PA8 | PA8 | I/O (FT, VDDIO2) | MCO, TIM1_CH1, USART1_CK, CRS_SYNC | ds p. 41 |
| 30 | PA9 | PA9 | I/O (FT, VDDIO2) | USART1_TX, TIM1_CH2 | ds p. 41 |
| 31 | PA10 | PA10 | I/O (FT, VDDIO2) | USART1_RX, TIM1_CH3 | ds p. 41 |
| 32 | PA11 | PA11 | I/O (FT, VDDIO2) | USB_DM; CAN_RX, USART1_CTS, TIM1_CH4 | ds p. 41 |
| 33 | PA12 | PA12 | I/O (FT, VDDIO2) | USB_DP; CAN_TX, USART1_RTS, TIM1_ETR | ds p. 41 |
| 34 | PA13 | PA13 | I/O (FT, VDDIO2) | SWDIO (after reset), IR_OUT, USB_NOE | ds p. 41 |
| 35 | VSS | VSS | Power | Ground | ds p. 41 |
| 36 | VDDIO2 | VDDIO2 | Power | Supply for the VDDIO2 I/O group | ds p. 41 |
| 37 | PA14 | PA14 | I/O (FT, VDDIO2) | SWCLK (after reset), USART2_TX | ds p. 41 |
| 38 | PA15 | PA15 | I/O (FT, VDDIO2) | SPI1_NSS, USART2_RX, TIM2_CH1_ETR, USART4_RTS | ds p. 41 |
| 39 | PB3 | PB3 | I/O (FT) | SPI1_SCK, I2S1_CK, TIM2_CH2 | ds p. 42 |
| 40 | PB4 | PB4 | I/O (FT) | SPI1_MISO, TIM3_CH1, TIM17_BKIN | ds p. 42 |
| 41 | PB5 | PB5 | I/O (FT) | SPI1_MOSI, I2C1_SMBA, TIM3_CH2, WKUP6 | ds p. 42 |
| 42 | PB6 | PB6 | I/O (FTf) | I2C1_SCL, USART1_TX, TIM16_CH1N | ds p. 42 |
| 43 | PB7 | PB7 | I/O (FTf) | I2C1_SDA, USART1_RX, USART4_CTS, TIM17_CH1N | ds p. 43 |
| 44 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 43 |
| 45 | PB8 | PB8 | I/O (FTf) | I2C1_SCL, CAN_RX, CEC, TIM16_CH1 | ds p. 43 |
| 46 | PB9 | PB9 | I/O (FTf) | I2C1_SDA, CAN_TX, IR_OUT, SPI2_NSS | ds p. 43 |
| 47 | VSS | VSS | Power | Ground | ds p. 43 |
| 48 | VDD | VDD | Power | Digital supply | ds p. 43 |

## Typical application

- Decouple VDD with 100 nF ceramics plus one 4.7 µF (Figure 13 shows 3 x 100 nF + 1 x 4.7 µF for the VDD pins), VDDIO2 with 100 nF + 4.7 µF, and VDDA with 10 nF + 1 µF (ds p. 49).
- Connect all VDD, VDDA, VSS and VSSA pins to the supply; none may be left open (ds p. 51).
- VDDA must be at least VDD and must be established first. Use VDDA >= 2.4 V when the ADC or DAC is used (ds p. 14, ds p. 53). Feeding VDDA from the VDD rail is the usual way to meet this, but the datasheet does not show that circuit.
- VDDIO2 may be 1.65-3.6 V independent of VDD, but must not be powered without VDD; for USB on PA11/PA12, keep VDDIO2 at 3.0-3.6 V (ds p. 14, ds p. 43, ds p. 99).
- NRST: internal 25-55 kΩ pull-up; add a 0.1 µF capacitor from NRST to VSS against parasitic resets (Figure 25) (ds p. 84).
- BOOT0 selects the boot memory together with the boot-selector option bit (flash, system memory or SRAM). Strap it low for flash boot and provide a way to drive it high to reach the system-memory bootloader (USART on PA14/PA15 or PA9/PA10, I2C on PB6/PB7, or USB DFU) (ds p. 13, ds p. 43).
- Crystal-less USB: route PA11 = USB_DM and PA12 = USB_DP straight to the connector. No external D+ pull-up and no series termination resistors are required (ds p. 27, ds p. 41, ds p. 99). Firmware must clock USB from HSI48 with CRS synchronized to USB SOF (ds p. 27).
- Optional HSE: a 4-32 MHz crystal on PF0/PF1 with CL1 = CL2 in the 5-20 pF range, sized with about 10 pF of pin and board capacitance included. REXT depends on the crystal (ds p. 68, ds p. 69).
- Optional LSE: a 32.768 kHz crystal on PC14/PC15 with load capacitors. Do not fit a resistor between OSC32_IN and OSC32_OUT (ds p. 70).
- SWD: bring out PA13 (SWDIO), PA14 (SWCLK), NRST, VDD and GND. After reset these pins are SWD with an internal pull-up on SWDIO and a pull-down on SWCLK (ds p. 43).

## Layout notes

- Place each decoupling capacitor as close as possible to its supply/ground pair, or directly under it on the PCB underside (ds p. 49).
- Place the HSE and LSE crystals and their load capacitors as close as possible to the oscillator pins (ds p. 68, ds p. 69).
- Footprint: 0.50 mm pitch, 0.30 x 1.20 mm pads, 9.70 mm outer pad span, 7.30 mm inner span (Figure 50). Body is 7.00 mm nominal, 1.600 mm max height (ds p. 116).
- The LQFP48 has no exposed pad. Its θJA is 54 °C/W, so heat leaves through the leads and ground pins (ds p. 121).
- Keep the PA11/PA12 USB pair short and matched. The impedance matching is in the on-chip driver (ds p. 99).

## Pitfalls

- Only FT/FTf pins are 5 V tolerant (5.5 V operating max), and only with the internal pull-up disabled above VDDIOx + 0.3 V. PA0-PA7, PB0 and PB1 are TTa (VDDA + 0.3 V max), and PC13-PC15 are TC (VDDIOx + 0.3 V max) (ds p. 37, ds p. 38, ds p. 39, ds p. 53).
- PC13-PC15 sit behind the backup power switch: no more than 3 mA, at most 2 MHz with 30 pF in output mode, and never as a current source such as an LED driver (ds p. 43).
- PA8-PA15 (pads 29-34, 37, 38) are powered from VDDIO2, not VDD. If VDDIO2 drops below the VREFINT-based threshold, hardware disables these I/Os, including USB and SWD (ds p. 14, ds p. 43).
- The HSE-from-PLL path for USB needs an HSE crystal. Only HSI48 with CRS gives crystal-less USB (ds p. 27).
- BOOT0 accepts up to 9.0 V absolute but 5.5 V operating, and its thresholds are not the standard I/O levels (ds p. 51, ds p. 53, ds p. 78).
- Any external reset circuit must let NRST fall below VIL(NRST) max, or the device ignores the reset (ds p. 84).
- VDD may exceed VDDA by at most 0.4 V (absolute maximum) (ds p. 51).
- VDDIO2-supplied I/Os can source at most -40 mA in total (ds p. 52).
- The library pin names PC14, PC15, PF0 and PF1 are the datasheet's PC14-OSC32_IN, PC15-OSC32_OUT, PF0-OSC_IN and PF1-OSC_OUT (ds p. 34).

## Not established by the sources

- The full BOOT0 / nBOOT1 / option-byte truth table and the bootloader pin assignments per interface are in the reference manual and AN2606, not in this datasheet (ds p. 13).
- No BOOT0 pull-down resistor value is recommended.
- How many VDD decoupling capacitors to use for the LQFP48's two VDD pads (24, 48): Figure 13 is drawn for three VDD pins (ds p. 49).
- The datasheet gives no VBAT capacitor value and no rule for VBAT when no battery is used.
- No USB ESD protection, VBUS sensing circuit or connector shielding is specified.
- The datasheet gives no HSI48 accuracy figure for operation once CRS has locked to SOF.
- No crystal part, ESR limit or exact load-capacitor value is given beyond the 5-20 pF range and the pointer to AN2867 (ds p. 68).
