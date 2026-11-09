# STM32F103C8T6 — Arm Cortex-M3 MCU, 72 MHz, 64 KB flash, 20 KB SRAM, USB FS device and CAN, LQFP48 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32F103C8Tx",
  "parts": ["@st/stm32::MCU_STM32F103C8T6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32F103C8T6", "lcsc": "C8734"},
    {"mfr": "STMicroelectronics", "mpn": "STM32F103C8T6TR", "lcsc": "C2969777"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32F103x8 STM32F103xB datasheet: Medium-density performance line Arm-based 32-bit MCU with 64 or 128 KB Flash, USB, CAN, 7 timers, 2 ADCs, 9 com. interfaces",
     "revision": "DS5319 Rev 18, March 2022 (29-Mar-2022)",
     "url": "https://www.st.com/resource/en/datasheet/stm32f103c8.pdf",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/ac66debd4c13334db1d28037926f8881.pdf",
     "sha256": "0596dd57693c93d214c9cb69186429883c853bad8650926def57c1ed3cdf2e9f"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS", 116 pages, marked "Datasheet - production data"). It was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C8734.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The `url` is the official ST URL recorded in `lib/@st/stm32/docs/stm32-part-catalog.md`. Nothing was fetched from st.com, so whether that URL serves the same revision is not established. The PDF is not in the repository; the SHA-256 is of the file that was read.

**Page numbers** are one-based PDF page indices. In this file they equal the printed page numbers ("N/116").

## Summary

The STM32F103C8T6 is a 72 MHz Arm Cortex-M3 microcontroller with 64 KB flash and 20 KB SRAM in a 7 x 7 mm LQFP48, for -40 to 85 °C ambient (suffix 6) (ds p. 1, ds p. 10, ds p. 107). It has a USB 2.0 full-speed device, CAN 2.0B, three USARTs, two SPIs, two I2Cs, two 12-bit ADCs on 10 channels and 37 GPIOs (ds p. 10, ds p. 19). It runs from 2.0–3.6 V, with VDDA tied to VDD and a separate VBAT for the RTC and backup registers (ds p. 15, ds p. 38). USB needs an HSE crystal (the 48 MHz USB clock comes from the PLL) and an external 1.5 kΩ D+ pull-up (ds p. 19, ds p. 73). Do not choose it for crystal-less USB, USB host, or more than 72 MHz (ds p. 19, ds p. 38).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB / APB2 clock | 0 | | 72 | MHz | fHCLK, fPCLK2 | ds p. 38 |
| APB1 clock | 0 | | 36 | MHz | fPCLK1 | ds p. 38 |
| Flash / SRAM | | 64 / 20 | | KB | STM32F103C8 | ds p. 10 |
| GPIO count | | 37 | | | STM32F103Cx | ds p. 10 |
| HSE crystal/resonator frequency | 4 | 8 | 16 | MHz | fOSC_IN | ds p. 52 |
| HSE startup time | | 2 | | ms | VDD stabilized | ds p. 52 |
| LSE startup time | | 2.5 | | s | TA = 25 °C | ds p. 53 |
| HSI accuracy, factory calibrated | -2 | | 2.5 | % | TA = -40 to 105 °C | ds p. 54 |
| HSI accuracy, factory calibrated | -1.1 | | 1.8 | % | TA = 25 °C | ds p. 54 |
| POR/PDR threshold, falling | 1.8 | 1.88 | 1.96 | V | | ds p. 39 |
| POR/PDR threshold, rising | 1.84 | 1.92 | 2.0 | V | | ds p. 39 |
| Reset temporization | 1.0 | 2.5 | 4.5 | ms | TRSTTEMPO | ds p. 39 |
| NRST weak pull-up | 30 | 40 | 50 | kΩ | VIN = VSS | ds p. 66 |
| NRST input low level VIL(NRST) | -0.5 | | 0.8 | V | design | ds p. 66 |
| NRST input high level VIH(NRST) | 2 | | VDD + 0.5 | V | design | ds p. 66 |
| I/O weak pull-up / pull-down | 30 | 40 | 50 | kΩ | | ds p. 61 |
| USB operating VDD | 3.0 | | 3.6 | V | full USB electrical spec | ds p. 73 |
| Run current | | | 50.0 | mA | 72 MHz, external clock, all peripherals on, from flash, TA = 85 °C | ds p. 41 |
| VBAT current, RTC on | | 1.4 | 1.9 | µA | VBAT = 3.3 V typ, max at 85 °C | ds p. 44 |
| Thermal resistance θJA | | 55 | | °C/W | LQFP48 7 x 7 mm | ds p. 104 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS (including VDDA and VDD) | -0.3 to 4.0 | V | ds p. 37 |
| VIN on 5 V-tolerant pins | VSS - 0.3 to VDD + 4.0 | V | ds p. 37 |
| VIN on any other pin | VSS - 0.3 to 4.0 | V | ds p. 37 |
| Variation between VDD pins / between ground pins | 50 / 50 | mV | ds p. 37 |
| Total current into VDD/VDDA / out of VSS | 150 / 150 | mA | ds p. 37 |
| Current per I/O pin, sunk / sourced | 25 / -25 | mA | ds p. 37 |
| Injected current, 5 V-tolerant pins | -5 / +0 | mA | ds p. 37 |
| Injected current, any other pin | ±5 | mA | ds p. 37 |
| Total injected current | ±25 | mA | ds p. 37 |
| Junction temperature | 150 | °C | ds p. 37 |
| Storage temperature | -65 to +150 | °C | ds p. 37 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 2 | 3.6 | V | ds p. 38 |
| VDDA, ADC not used (same potential as VDD) | 2 | 3.6 | V | ds p. 38 |
| VDDA, ADC used | 2.4 | 3.6 | V | ds p. 38 |
| VBAT | 1.8 | 3.6 | V | ds p. 38 |
| VIN, standard I/O | -0.3 | VDD + 0.3 | V | ds p. 38 |
| VIN, FT I/O, 2 V < VDD ≤ 3.6 V | -0.3 | 5.5 | V | ds p. 38 |
| VIN, FT I/O, VDD = 2 V | -0.3 | 5.2 | V | ds p. 38 |
| VIN, BOOT0 | 0 | 5.5 | V | ds p. 38 |
| Power dissipation, LQFP48, TA = 85 °C (suffix 6) | | 363 | mW | ds p. 38 |
| Ambient temperature, suffix 6, maximum dissipation | -40 | 85 | °C | ds p. 38 |
| Ambient temperature, suffix 6, low dissipation | -40 | 105 | °C | ds p. 38 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 38 |

## Pins

Checked pad by pad against Figure 8, "STM32F103xx performance line LQFP48 pinout" (ds p. 26, a top view with the pin-1 dot at pad 1), and the LQFP48/UFQFPN48 column of Table 5 (ds pp. 28–33). All 48 pads carry the signal the library gives them. "Default" and "remap" are Table 5's alternate-function columns.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VBAT | VBAT | Power | Backup supply (RTC, LSE, backup registers) | ds p. 28 |
| 2 | PC13 | PC13-TAMPER-RTC | I/O | TAMPER-RTC | ds p. 28 |
| 3 | PC14 | PC14-OSC32_IN | I/O | OSC32_IN (LSE) | ds p. 28 |
| 4 | PC15 | PC15-OSC32_OUT | I/O | OSC32_OUT (LSE) | ds p. 28 |
| 5 | PD0 | OSC_IN | Input | OSC_IN (HSE) after reset; PD0 by remap | ds p. 28 |
| 6 | PD1 | OSC_OUT | Output | OSC_OUT (HSE) after reset; PD1 by remap | ds p. 28 |
| 7 | NRST | NRST | I/O | Reset, active low | ds p. 28 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 28 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 28 |
| 10 | PA0 | PA0-WKUP | I/O | WKUP, USART2_CTS, ADC12_IN0, TIM2_CH1_ETR | ds p. 29 |
| 11 | PA1 | PA1 | I/O | USART2_RTS, ADC12_IN1, TIM2_CH2 | ds p. 29 |
| 12 | PA2 | PA2 | I/O | USART2_TX, ADC12_IN2, TIM2_CH3 | ds p. 29 |
| 13 | PA3 | PA3 | I/O | USART2_RX, ADC12_IN3, TIM2_CH4 | ds p. 29 |
| 14 | PA4 | PA4 | I/O | SPI1_NSS, USART2_CK, ADC12_IN4 | ds p. 29 |
| 15 | PA5 | PA5 | I/O | SPI1_SCK, ADC12_IN5 | ds p. 29 |
| 16 | PA6 | PA6 | I/O | SPI1_MISO, ADC12_IN6, TIM3_CH1; remap TIM1_BKIN | ds p. 29 |
| 17 | PA7 | PA7 | I/O | SPI1_MOSI, ADC12_IN7, TIM3_CH2; remap TIM1_CH1N | ds p. 29 |
| 18 | PB0 | PB0 | I/O | ADC12_IN8, TIM3_CH3; remap TIM1_CH2N | ds p. 29 |
| 19 | PB1 | PB1 | I/O | ADC12_IN9, TIM3_CH4; remap TIM1_CH3N | ds p. 29 |
| 20 | PB2 | PB2 | I/O (FT) | PB2/BOOT1 | ds p. 30 |
| 21 | PB10 | PB10 | I/O (FT) | I2C2_SCL, USART3_TX; remap TIM2_CH3 | ds p. 30 |
| 22 | PB11 | PB11 | I/O (FT) | I2C2_SDA, USART3_RX; remap TIM2_CH4 | ds p. 30 |
| 23 | VSS | VSS_1 | Power | Ground | ds p. 30 |
| 24 | VDD | VDD_1 | Power | Digital supply | ds p. 30 |
| 25 | PB12 | PB12 | I/O (FT) | SPI2_NSS, I2C2_SMBAl, USART3_CK, TIM1_BKIN | ds p. 30 |
| 26 | PB13 | PB13 | I/O (FT) | SPI2_SCK, USART3_CTS, TIM1_CH1N | ds p. 30 |
| 27 | PB14 | PB14 | I/O (FT) | SPI2_MISO, USART3_RTS, TIM1_CH2N | ds p. 30 |
| 28 | PB15 | PB15 | I/O (FT) | SPI2_MOSI, TIM1_CH3N | ds p. 30 |
| 29 | PA8 | PA8 | I/O (FT) | USART1_CK, TIM1_CH1, MCO | ds p. 31 |
| 30 | PA9 | PA9 | I/O (FT) | USART1_TX, TIM1_CH2 | ds p. 31 |
| 31 | PA10 | PA10 | I/O (FT) | USART1_RX, TIM1_CH3 | ds p. 31 |
| 32 | PA11 | PA11 | I/O (FT) | USBDM, CANRX, USART1_CTS, TIM1_CH4 | ds p. 31 |
| 33 | PA12 | PA12 | I/O (FT) | USBDP, CANTX, USART1_RTS, TIM1_ETR | ds p. 31 |
| 34 | PA13 | PA13 | I/O (FT) | JTMS/SWDIO after reset; PA13 by remap | ds p. 31 |
| 35 | VSS | VSS_2 | Power | Ground | ds p. 31 |
| 36 | VDD | VDD_2 | Power | Digital supply | ds p. 31 |
| 37 | PA14 | PA14 | I/O (FT) | JTCK/SWCLK after reset; PA14 by remap | ds p. 32 |
| 38 | PA15 | PA15 | I/O (FT) | JTDI after reset; remap TIM2_CH1_ETR, PA15, SPI1_NSS | ds p. 32 |
| 39 | PB3 | PB3 | I/O (FT) | JTDO after reset; remap TIM2_CH2, PB3, TRACESWO, SPI1_SCK | ds p. 32 |
| 40 | PB4 | PB4 | I/O (FT) | NJTRST after reset; remap TIM3_CH1, PB4, SPI1_MISO | ds p. 32 |
| 41 | PB5 | PB5 | I/O | I2C1_SMBAl; remap TIM3_CH2, SPI1_MOSI | ds p. 32 |
| 42 | PB6 | PB6 | I/O (FT) | I2C1_SCL, TIM4_CH1; remap USART1_TX | ds p. 32 |
| 43 | PB7 | PB7 | I/O (FT) | I2C1_SDA, TIM4_CH2; remap USART1_RX | ds p. 32 |
| 44 | BOOT0 | BOOT0 | Input | Boot memory selection | ds p. 32 |
| 45 | PB8 | PB8 | I/O (FT) | TIM4_CH3; remap I2C1_SCL, CANRX | ds p. 33 |
| 46 | PB9 | PB9 | I/O (FT) | TIM4_CH4; remap I2C1_SDA, CANTX | ds p. 33 |
| 47 | VSS | VSS_3 | Power | Ground | ds p. 33 |
| 48 | VDD | VDD_3 | Power | Digital supply | ds p. 33 |

## Typical application

- Decouple VDD with 100 nF per pin plus one 4.7 µF; Figure 14 draws 5 x 100 nF + 1 x 4.7 µF for VDD1–5, and the 4.7 µF must go on VDD3 (pad 48 on the LQFP48) (ds p. 36, ds p. 33). Decouple VDDA with 10 nF + 1 µF (ds p. 36).
- Connect every VDD, VDDA, VSS and VSSA pin; VDDA and VSSA must be connected to VDD and VSS (ds p. 15, ds p. 37). Power VDD and VDDA from the same source; at most 300 mV between them in power-up and operation (ds p. 38).
- VBAT: 1.8–3.6 V powers the RTC, LSE and backup registers through a power switch when VDD is absent (ds p. 15).
- NRST has an internal 30–50 kΩ pull-up. Figure 31 adds 0.1 µF from NRST to ground (ds p. 66, ds p. 67).
- Boot: BOOT0 (pad 44) and BOOT1, which is shared with PB2 (pad 20), select flash, system memory or SRAM. The system-memory bootloader reprograms flash over USART1 (ds p. 15, ds p. 30). Strap BOOT0 low for flash boot.
- HSE: a 4–16 MHz crystal on OSC_IN/OSC_OUT with CL1 = CL2 in the 5–25 pF range, sized with about 10 pF of pin and board capacitance (ds p. 52, ds p. 53). It is required for USB (ds p. 12, ds p. 19).
- LSE: a 32.768 kHz crystal on PC14/PC15 with CL1 = CL2 in 5–15 pF. ST recommends a crystal with CL ≤ 7 pF and never one with CL = 12.5 pF (ds p. 54).
- USB: PA11 = USBDM, PA12 = USBDP (ds p. 31). For full-speed compliance, pull D+ up with 1.5 kΩ to 3.0–3.6 V, and keep VDD at 3.0–3.6 V (ds p. 73).
- Debug: SWD on PA13 (SWDIO) and PA14 (SWCLK); full JTAG also takes PA15, PB3 and PB4 after reset (ds pp. 31–32, ds p. 20).

## Layout notes

- ST's recommended land (Figure 63) is 0.50 mm pitch, 0.30 x 1.20 mm pads, 9.70 mm outer span, 7.30 mm inner span, 5.80 mm pad-row length (ds p. 102). The library uses a different land: see "Not established by the sources".
- Package: body D1 = E1 = 7.000 mm nominal, D = E = 9.000 mm over leads, lead width b = 0.17–0.27 mm, height A = 1.600 mm max (ds p. 101, ds p. 102). No exposed pad.
- Place crystals and load capacitors as close as possible to the oscillator pins (ds p. 52, ds p. 53).

## Pitfalls

- Not every pin is 5 V tolerant. On the LQFP48, PC13–PC15, OSC_IN/OSC_OUT, NRST, PA0–PA7, PB0, PB1 and PB5 have no FT marking (ds pp. 28–32). FT inputs above VDD + 0.3 V need the internal pull-up/down disabled (ds p. 38).
- PB2 is also BOOT1 (ds p. 30). The datasheet leaves its boot role to AN2606 (ds p. 15), so treat PB2's level at reset as part of the boot strap rather than as a free GPIO.
- PA13–PA15, PB3 and PB4 come out of reset as JTAG/SWD pins (ds pp. 31–32). Using PA15, PB3 or PB4 as GPIO needs a remap in software.
- Pads 5 and 6 are OSC_IN/OSC_OUT after reset. PD0/PD1 are reached only by remap, and as outputs they are limited to the 50 MHz output mode (ds p. 33).
- PC13–PC15 are behind the backup power switch: at most 3 mA sink, 2 MHz, 30 pF, and never as a current source such as an LED driver (ds p. 33).
- The USB clock needs HSE and the PLL; HSI cannot clock USB (ds p. 12, ds p. 19).
- An external reset circuit must pull NRST below VIL(NRST) max (0.8 V), or the reset is ignored (ds p. 66, ds p. 67).

## Not established by the sources

- **Library land is KiCad's, not ST's.** `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` has 0.30 x 1.475 mm pads with centres at ±4.1625 mm, so its outer span is 9.80 mm and its inner span 6.85 mm. ST's Figure 63 gives 0.30 x 1.20 mm pads, 9.70 mm outer and 7.30 mm inner (ds p. 102). The library pads reach 0.225 mm further inward and 0.05 mm further outward per side. Pitch (0.50 mm), pad width (0.30 mm) and pad-row length (5.80 mm) agree. Pin 1 is at the lower left with numbering counter-clockwise in top view in both the library (+y-down frame, after its quarter turn) and Figure 63.
- **Pad 5 and 6 names.** The library calls them `PD0` and `PD1`; the datasheet's pin names are OSC_IN and OSC_OUT, with PD0/PD1 only by remap (ds p. 28, ds p. 33). The pads are right; the names hide the oscillator function.
- **NRST is typed `input` in the library**; Table 5 types it I/O (ds p. 28).
- **BOOT0/BOOT1 truth table.** The datasheet lists three boot sources and names BOOT1 on PB2, but gives no truth table and refers to AN2606 (ds p. 15, ds p. 30). No BOOT0 resistor value is given.
- **VBAT with no battery.** The datasheet gives VBAT's range and purpose but no rule for tying it to VDD when no battery is used, and no VBAT decoupling value (ds p. 15, ds p. 36).
- **Decoupling count.** Figure 14 is drawn for five VDD pins (ds p. 36); the LQFP48 has three (pads 24, 36, 48).
- **BOOT0 input thresholds.** Table 35 gives levels for "all I/Os except BOOT0" only (ds p. 61); no BOOT0 VIL/VIH is given.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- No USB ESD protection, VBUS sensing or crystal ESR limit is specified; the datasheet points to AN2867 for crystals (ds p. 53).
