# STM32F030C8T6 — Arm Cortex-M0 value-line MCU, 64 KB flash, 8 KB SRAM, LQFP48 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32F030C8Tx",
  "parts": ["@st/stm32::MCU_STM32F030C8T6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32F030C8T6", "lcsc": "C23922"},
    {"mfr": "STMicroelectronics", "mpn": "STM32F030C8T6TR", "lcsc": "C40053"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32F030x4 STM32F030x6 STM32F030x8 datasheet (target specification): Value-line ARM-based 32-bit MCU with 16 to 64-KB Flash, timers, ADC, communication interfaces, 2.4-3.6 V operation",
     "revision": "DocID024849 Rev 1, July 2013 (initial release, 04-Jul-2013)",
     "url": "https://www.st.com/resource/en/datasheet/stm32f030c8.pdf",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/f6a02cf57a5841a5b72f3eaa84b79677.pdf",
     "sha256": "2cb57cb91b7694e1e0f76b6847aaf4fb3205e7903aa044886d0b3ff7265c8519"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS", 88 pages). It was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C23922.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The `url` is the official ST URL recorded in `lib/@st/stm32/docs/stm32-part-catalog.md`. Nothing was fetched from st.com, so whether that URL serves the same revision is not established. The PDF is not in the repository; the SHA-256 is of the file that was read.

**Page numbers** are one-based PDF page indices. In this file they equal the printed page numbers ("N/88").

**This source is a target specification, not a production datasheet** (cover, ds p. 1). Every number below is from that preliminary document. See "Not established by the sources".

## Summary

The STM32F030C8T6 is a 48 MHz Arm Cortex-M0 value-line microcontroller with 64 KB flash and 8 KB SRAM in a 7 x 7 mm LQFP48, for -40 to 85 °C ambient (ds p. 1, ds p. 10, ds p. 86). The LQFP48 has 39 GPIOs, two SPIs, two I2Cs (one with Fast-mode Plus pins), two USARTs and a 12-bit ADC (ds p. 1, ds p. 10). Its supply range is 2.4–3.6 V on both VDD and VDDA, with VDDA at or above VDD (ds p. 13, ds p. 40). It has no USB, no CAN, no DAC and no VBAT pin: pad 1 is a third VDD (ds p. 24, ds p. 26). Do not choose it if you need USB, a battery-backed RTC domain, operation below 2.4 V, or more than 48 MHz (ds p. 40).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB / APB clock | 0 | | 48 | MHz | fHCLK, fPCLK | ds p. 40 |
| Flash / SRAM | | 64 / 8 | | KB | STM32F030C8 | ds p. 10 |
| GPIO count | | 39 | | | STM32F030C6/C8 | ds p. 10 |
| HSE crystal/resonator frequency | 4 | 8 | 32 | MHz | fOSC_IN | ds p. 50 |
| HSE startup time | | 2 | | ms | VDD stabilized | ds p. 50 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 51 |
| HSI frequency | | 8 | | MHz | | ds p. 52 |
| POR/PDR threshold, falling | 1.8 | 1.88 | 2.06 | V | | ds p. 41 |
| POR/PDR threshold, rising | 1.84 | 1.92 | 2.10 | V | | ds p. 41 |
| Reset temporization | 1.5 | 2.5 | 4.5 | ms | tRSTTEMPO | ds p. 41 |
| NRST weak pull-up | 30 | 40 | 50 | kΩ | VIN = VSS | ds p. 62 |
| NRST input low level VIL(NRST) | -0.3 | | 0.8 | V | design | ds p. 62 |
| NRST input high level VIH(NRST) | 2 | | VDD + 0.3 | V | design | ds p. 62 |
| I/O weak pull-up / pull-down | 25 | 40 | 55 | kΩ | | ds p. 58 |
| BOOT0 VIL | | | 0.3 VDD - 0.3 | V | design simulation | ds p. 57 |
| BOOT0 VIH | 0.2 VDD + 0.95 | | | V | design simulation | ds p. 58 |
| Run current from VDD | | 22 | 22.8 | mA | 48 MHz HSI + PLL, code from flash, all peripherals on, VDD = 3.6 V, max at 85 °C | ds p. 43 |
| Thermal resistance θJA | | 55 | | °C/W | LQFP48 7 x 7 mm | ds p. 83 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS (including VDDA and VDD) | -0.3 to 4.0 | V | ds p. 38 |
| VDD - VDDA (VDD > VDDA) | 0.4 max | V | ds p. 38 |
| VIN on FT and FTf pins | VSS - 0.3 to VDD + 4.0 | V | ds p. 38 |
| VIN on TTa pins and any other pin | VSS - 0.3 to 4.0 | V | ds p. 38 |
| Current into all VDD / out of all VSS | 120 / -120 | mA | ds p. 39 |
| Current per VDD / VSS pin | 100 / -100 | mA | ds p. 39 |
| Current per I/O pin, sunk / sourced | 25 / -25 | mA | ds p. 39 |
| Total I/O current, sunk / sourced | 80 / -80 | mA | ds p. 39 |
| Injected current, FT, FTf and B pins | -5 / +0 | mA | ds p. 39 |
| Injected current, TC, RST and TTa pins | ±5 | mA | ds p. 39 |
| Total injected current | ±25 | mA | ds p. 39 |
| Junction temperature | 150 | °C | ds p. 39 |
| Storage temperature | -65 to +150 | °C | ds p. 39 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 2.4 | 3.6 | V | ds p. 40 |
| VDDA (must be at or above VDD) | 2.4 | 3.6 | V | ds p. 40 |
| VIN, FT and FTf pins | VSS - 0.3 | VDD + 4.0 | V | ds p. 40 |
| VIN, TTa pins and any other pin | VSS - 0.3 | 4.0 | V | ds p. 40 |
| Power dissipation, LQFP48, TA = 85 °C (suffix 6) | | 364 | mW | ds p. 40 |
| Ambient temperature, suffix 6, maximum dissipation | -40 | 85 | °C | ds p. 40 |
| Ambient temperature, suffix 6, low dissipation | -40 | 105 | °C | ds p. 40 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 40 |

## Pins

Checked pad by pad against Figure 4, "LQFP48 48-pin package pinout" (ds p. 24, top view), and the LQFP48 column of Table 11, "Pin definitions" (ds pp. 26–30). All 48 pads agree with the library. Functions listed are the STM32F030x8 ones; features marked "(2)" in Table 11 exist only on x4/x6 and are left out (ds p. 30).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VDD | VDD | Power | Complementary power supply (no VBAT on this part) | ds p. 26 |
| 2 | PC13 | PC13 | I/O (TC) | RTC_TAMP1, RTC_TS, RTC_OUT, WKUP2 | ds p. 26 |
| 3 | PC14 | PC14-OSC32_IN | I/O (TC) | OSC32_IN (LSE) | ds p. 26 |
| 4 | PC15 | PC15-OSC32_OUT | I/O (TC) | OSC32_OUT (LSE) | ds p. 26 |
| 5 | PF0 | PF0-OSC_IN | I/O (FT) | OSC_IN (HSE) | ds p. 26 |
| 6 | PF1 | PF1-OSC_OUT | I/O (FT) | OSC_OUT (HSE) | ds p. 26 |
| 7 | NRST | NRST | I/O (RST) | Reset input / internal reset output, active low | ds p. 26 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 26 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 26 |
| 10 | PA0 | PA0 | I/O (TTa) | USART2_CTS; ADC_IN0, RTC_TAMP2, WKUP1 | ds p. 26 |
| 11 | PA1 | PA1 | I/O (TTa) | USART2_RTS, EVENTOUT; ADC_IN1 | ds p. 26 |
| 12 | PA2 | PA2 | I/O (TTa) | USART2_TX, TIM15_CH1; ADC_IN2 | ds p. 26 |
| 13 | PA3 | PA3 | I/O (TTa) | USART2_RX, TIM15_CH2; ADC_IN3 | ds p. 26 |
| 14 | PA4 | PA4 | I/O (TTa) | SPI1_NSS, USART2_CK, TIM14_CH1; ADC_IN4 | ds p. 27 |
| 15 | PA5 | PA5 | I/O (TTa) | SPI1_SCK; ADC_IN5 | ds p. 27 |
| 16 | PA6 | PA6 | I/O (TTa) | SPI1_MISO, TIM3_CH1, TIM1_BKIN, TIM16_CH1; ADC_IN6 | ds p. 27 |
| 17 | PA7 | PA7 | I/O (TTa) | SPI1_MOSI, TIM3_CH2, TIM14_CH1, TIM1_CH1N, TIM17_CH1; ADC_IN7 | ds p. 27 |
| 18 | PB0 | PB0 | I/O (TTa) | TIM3_CH3, TIM1_CH2N; ADC_IN8 | ds p. 27 |
| 19 | PB1 | PB1 | I/O (TTa) | TIM3_CH4, TIM14_CH1, TIM1_CH3N; ADC_IN9 | ds p. 27 |
| 20 | PB2 | PB2 | I/O (FT) | GPIO | ds p. 27 |
| 21 | PB10 | PB10 | I/O (FT) | I2C2_SCL | ds p. 27 |
| 22 | PB11 | PB11 | I/O (FT) | I2C2_SDA, EVENTOUT | ds p. 27 |
| 23 | VSS | VSS | Power | Ground | ds p. 27 |
| 24 | VDD | VDD | Power | Digital supply | ds p. 27 |
| 25 | PB12 | PB12 | I/O (FT) | SPI2_NSS, TIM1_BKIN, EVENTOUT | ds p. 27 |
| 26 | PB13 | PB13 | I/O (FT) | SPI2_SCK, TIM1_CH1N | ds p. 28 |
| 27 | PB14 | PB14 | I/O (FT) | SPI2_MISO, TIM1_CH2N, TIM15_CH1 | ds p. 28 |
| 28 | PB15 | PB15 | I/O (FT) | SPI2_MOSI, TIM1_CH3N, TIM15_CH1N, TIM15_CH2; RTC_REFIN | ds p. 28 |
| 29 | PA8 | PA8 | I/O (FT) | USART1_CK, TIM1_CH1, EVENTOUT, MCO | ds p. 28 |
| 30 | PA9 | PA9 | I/O (FT) | USART1_TX, TIM1_CH2, TIM15_BKIN | ds p. 28 |
| 31 | PA10 | PA10 | I/O (FT) | USART1_RX, TIM1_CH3, TIM17_BKIN | ds p. 28 |
| 32 | PA11 | PA11 | I/O (FT) | USART1_CTS, TIM1_CH4, EVENTOUT | ds p. 28 |
| 33 | PA12 | PA12 | I/O (FT) | USART1_RTS, TIM1_ETR, EVENTOUT | ds p. 28 |
| 34 | PA13 | PA13 | I/O (FT) | SWDIO (after reset), IR_OUT | ds p. 28 |
| 35 | PF6 | PF6 | I/O (FT) | I2C2_SCL | ds p. 28 |
| 36 | PF7 | PF7 | I/O (FT) | I2C2_SDA | ds p. 29 |
| 37 | PA14 | PA14 | I/O (FT) | SWCLK (after reset), USART2_TX | ds p. 29 |
| 38 | PA15 | PA15 | I/O (FT) | SPI1_NSS, USART2_RX, EVENTOUT | ds p. 29 |
| 39 | PB3 | PB3 | I/O (FT) | SPI1_SCK, EVENTOUT | ds p. 29 |
| 40 | PB4 | PB4 | I/O (FT) | SPI1_MISO, TIM3_CH1, EVENTOUT | ds p. 29 |
| 41 | PB5 | PB5 | I/O (FT) | SPI1_MOSI, I2C1_SMBA, TIM16_BKIN, TIM3_CH2 | ds p. 29 |
| 42 | PB6 | PB6 | I/O (FTf) | I2C1_SCL, USART1_TX, TIM16_CH1N | ds p. 29 |
| 43 | PB7 | PB7 | I/O (FTf) | I2C1_SDA, USART1_RX, TIM17_CH1N | ds p. 29 |
| 44 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 29 |
| 45 | PB8 | PB8 | I/O (FTf) | I2C1_SCL, TIM16_CH1 | ds p. 29 |
| 46 | PB9 | PB9 | I/O (FTf) | I2C1_SDA, IR_OUT, TIM17_CH1, EVENTOUT | ds p. 29 |
| 47 | VSS | VSS | Power | Ground | ds p. 30 |
| 48 | VDD | VDD | Power | Digital supply | ds p. 30 |

## Typical application

- Decouple VDD with 2 x 100 nF plus 1 x 4.7 µF and VDDA with 10 nF plus 1 µF, as Figure 10 draws them (ds p. 37). The figure shows two VDD pins; the LQFP48 has three (pads 1, 24, 48) (ds p. 24).
- Connect every VDD, VDDA, VSS and VSSA pin to the supply (ds p. 38).
- VDDA must be at or above VDD and must be applied first. The POR watches only VDD (ds p. 13).
- NRST has an internal 30–50 kΩ pull-up. Figure 19 adds 0.1 µF from NRST to ground against parasitic resets (ds p. 62, ds p. 63).
- BOOT0 and the boot-selector option bit choose flash, system memory or SRAM. The system-memory bootloader uses USART on PA14/PA15 or PA9/PA10 (ds p. 12). Strap BOOT0 low for flash boot and give a way to drive it high.
- Optional HSE: a 4–32 MHz crystal on PF0/PF1 with CL1 = CL2 in the 5–20 pF range, sized with about 10 pF of pin and board capacitance. REXT depends on the crystal (ds p. 50, ds p. 51).
- Optional LSE: a 32.768 kHz crystal on PC14/PC15 with CL1 and CL2. Do not fit a resistor between OSC32_IN and OSC32_OUT (ds p. 52).
- SWD: bring out PA13 (SWDIO), PA14 (SWCLK), NRST, VDD and GND. After reset these pins are SWD, with a pull-up on SWDIO and a pull-down on SWCLK (ds p. 30).

## Layout notes

- Place each decoupling capacitor as close as possible to its pins, or under them on the PCB underside (ds p. 37).
- Place the crystals and their load capacitors as close as possible to the oscillator pins (ds p. 50, ds p. 51).
- ST's recommended land (Figure 29) is 0.50 mm pitch, 0.30 x 1.20 mm pads, 9.70 mm outer span, 7.30 mm inner span, 5.80 mm pad-row length (ds p. 78). The library uses a different land: see "Not established by the sources".
- Package: body D1 = E1 = 7.000 mm nominal, D = E = 9.000 mm over leads, lead width b = 0.17–0.27 mm, height A = 1.600 mm max (ds p. 77). No exposed pad.

## Pitfalls

- Pad 1 is a VDD pin, not a backup supply: this part has no VBAT (ds p. 24, ds p. 26). Do not wire a coin cell to it.
- 5 V tolerance: only FT and FTf pins, and only with internal pull-up/pull-down disabled above VDD + 0.3 V (ds p. 58). PA0–PA7, PB0 and PB1 are TTa (4.0 V max); PC13–PC15 are TC (ds pp. 26–27, ds p. 38). The FT limit is written as VDD + 4.0 V, so a 5 V signal is in range only while VDD is at least 1 V; the datasheet gives no separate rule for FT inputs while VDD is off (ds p. 38, ds p. 40).
- PC13–PC15 sit behind a power switch that sinks only 3 mA: at most 2 MHz with 30 pF in output mode, and never as a current source such as an LED driver (ds p. 30).
- Only PB6–PB9 are FTf (Fast-mode Plus capable) (ds p. 29).
- I2C1 on PA9/PA10 is an x4/x6 feature only; on the x8 use PB6/PB7 or PB8/PB9 for I2C1 (ds p. 28, ds p. 30).
- An external reset circuit must pull NRST below VIL(NRST) max (0.8 V), or the reset is ignored (ds p. 62, ds p. 63).
- The library pin names PC14, PC15, PF0 and PF1 are the datasheet's PC14-OSC32_IN, PC15-OSC32_OUT, PF0-OSC_IN and PF1-OSC_OUT (ds p. 26).

## Not established by the sources

- **The source is preliminary.** It is DocID024849 Rev 1, the July 2013 initial release, marked "Datasheet target specification" (ds p. 1, ds p. 87). No production-data revision was read. Every limit in this note may have changed in later ST revisions. This note does not know which revision st.com serves at the catalog URL.
- **Library land is KiCad's, not ST's.** `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` has 0.30 x 1.475 mm pads with centres at ±4.1625 mm, so its outer span is 9.80 mm and its inner span 6.85 mm. ST's Figure 29 gives 0.30 x 1.20 mm pads, 9.70 mm outer and 7.30 mm inner (ds p. 78). The library pads reach 0.225 mm further inward and 0.05 mm further outward per side. Pitch (0.50 mm), pad width (0.30 mm) and pad-row length (5.80 mm) agree. Pin 1 is at the lower left with numbering counter-clockwise in top view in both the library (+y-down frame, after its quarter turn) and Figure 29.
- **NRST is typed `input` in the library**; the datasheet calls it a bidirectional reset pin that also drives the internal reset out (ds p. 26).
- **HSI accuracy.** Table 33 gives ±1 % at 25 °C and ±5 % over -40 to 85 °C, typical, but footnote 3 says "With user calibration" (ds p. 52). Whether these are factory-trimmed figures is not clear.
- **Decoupling count.** Figure 10 is drawn for two VDD pins (ds p. 37); the datasheet does not say how many 100 nF capacitors to use for the LQFP48's three VDD pads.
- **Boot selection.** The datasheet names BOOT0 and a boot-selector option bit and the bootloader's USART pins, but gives no truth table (ds p. 12). No BOOT0 pull-down value is given.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- No crystal ESR or drive-level limit is given; the datasheet points to AN2867 (ds p. 50).
