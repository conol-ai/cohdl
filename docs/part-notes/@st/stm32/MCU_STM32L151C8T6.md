# STM32L151C8T6 — ultra-low-power Arm Cortex-M3 MCU, 32 MHz, 64 KB flash, 10 KB RAM, 4 KB EEPROM, USB FS device, LQFP48 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32L151C8Tx",
  "parts": ["@st/stm32::MCU_STM32L151C8T6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32L151C8T6", "lcsc": "C9861"},
    {"mfr": "STMicroelectronics", "mpn": "STM32L151C8T6TR", "lcsc": "C2054894"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32L151x6/8/B STM32L152x6/8/B datasheet: Ultra-low-power 32-bit MCU ARM-based Cortex-M3, 128KB Flash, 16KB SRAM, 4KB EEPROM, LCD, USB, ADC, DAC",
     "revision": "Doc ID 17659 Rev 7, October 2012 (26-Oct-2012)",
     "url": "https://www.st.com/resource/en/datasheet/stm32l151c8.pdf",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/e1730fa9d3f744e99d38506f49990684.pdf",
     "sha256": "f6403fa4ebb08ec3f1e40ad8e040aaa25ccceb980c01c6c36af59a3bb3f07d0b"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS", 117 pages, marked "Datasheet - production data"). It was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C9861.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The `url` is the official ST URL recorded in `lib/@st/stm32/docs/stm32-part-catalog.md`. Nothing was fetched from st.com, so whether that URL serves the same revision is not established. The PDF is not in the repository; the SHA-256 is of the file that was read.

**Page numbers** are one-based PDF page indices. In this file they equal the printed page numbers ("N/117").

## Summary

The STM32L151C8T6 is an ultra-low-power 32 MHz Arm Cortex-M3 microcontroller with 64 KB flash, 10 KB RAM and 4 KB data EEPROM in a 7 x 7 mm LQFP48, for -40 to 85 °C ambient (ds p. 1, ds p. 10, ds p. 112). It has a USB 2.0 full-speed device, three USARTs, two SPIs, two I2Cs, a 12-bit ADC, a 2-channel DAC, two comparators and 37 GPIOs; the LCD controller is STM32L152 only (ds p. 10). Without the "D" option it runs from 1.8–3.6 V with BOR enabled (ds p. 50, ds p. 112). USB needs an HSE crystal and VDD of 3.0–3.6 V (ds p. 29, ds p. 89). Do not choose it if you need more than 32 MHz, crystal-less USB, or an LCD driver (use an STM32L152) (ds p. 10, ds p. 50).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB / APB1 / APB2 clock | 0 | | 32 | MHz | | ds p. 50 |
| Flash / RAM / data EEPROM | | 64 / 10 / 4 | | KB | STM32L15xC8 | ds p. 10 |
| GPIO count | | 37 | | | STM32L15xCx | ds p. 10 |
| HSE crystal/resonator frequency | 1 | | 24 | MHz | fOSC_IN | ds p. 68 |
| HSE startup time | | 1 | | ms | VDD stabilized | ds p. 68 |
| LSE startup time | | 1 | | s | VDD stabilized | ds p. 69 |
| HSI frequency | | 16 | | MHz | VDD = 3.0 V | ds p. 71 |
| HSI accuracy, factory calibrated | -4 | | 3 | % | VDDA = 1.65–3.6 V, TA = -40 to 105 °C | ds p. 71 |
| POR/PDR threshold, falling | 1 | 1.5 | 1.65 | V | | ds p. 51 |
| POR/PDR threshold, rising | 1.3 | 1.5 | 1.65 | V | | ds p. 51 |
| BOR threshold 0, falling | 1.67 | 1.7 | 1.74 | V | | ds p. 51 |
| BOR threshold 0, rising | 1.69 | 1.76 | 1.8 | V | | ds p. 51 |
| Reset temporization, BOR enabled | | 2 | 3.3 | ms | VDD rising | ds p. 51 |
| NRST weak pull-up | 30 | 45 | 60 | kΩ | VIN = VSS | ds p. 82 |
| NRST input low level VIL(NRST) | VSS | | 0.8 | V | design | ds p. 82 |
| NRST input high level VIH(NRST) | 1.4 | | VDD | V | design | ds p. 82 |
| I/O weak pull-up / pull-down | 30 | 45 | 60 | kΩ | | ds p. 79 |
| USB operating VDD | 3.0 | | 3.6 | V | | ds p. 89 |
| Run current from flash | | 9.6 | 11 | mA | 32 MHz, range 1, HSE bypass + PLL, VDD = 3.6 V | ds p. 54 |
| Standby current, RTC off | | 0.3 | 0.55 | µA | IWDG and LSI off, TA = -40 to 25 °C | ds p. 61 |
| Thermal resistance θJA | | 55 | | °C/W | LQFP48 7 x 7 mm | ds p. 110 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS (including VDDA and VDD) | -0.3 to 4.0 | V | ds p. 49 |
| VIN on five-volt-tolerant pins | VSS - 0.3 to VDD + 4.0 | V | ds p. 49 |
| VIN on any other pin | VSS - 0.3 to 4.0 | V | ds p. 49 |
| Variation between VDD pins / between ground pins | 50 / 50 | mV | ds p. 49 |
| Total current into VDD/VDDA / out of VSS | 80 / 80 | mA | ds p. 49 |
| Current per I/O pin, sunk / sourced | 25 / -25 | mA | ds p. 49 |
| Injected current, five-volt-tolerant I/O | +0 / -5 | mA | ds p. 49 |
| Injected current, any other pin | ±5 | mA | ds p. 49 |
| Total injected current | ±25 | mA | ds p. 49 |
| Junction temperature | 150 | °C | ds p. 50 |
| Storage temperature | -65 to +150 | °C | ds p. 50 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, BOR enabled at power-on (this part: no "D" option) | 1.8 | 3.6 | V | ds p. 50 |
| VDD, BOR disabled after power-on | 1.65 | 3.6 | V | ds p. 50 |
| VDDA, ADC and DAC not used (same voltage as VDD) | 1.65 | 3.6 | V | ds p. 50 |
| VDDA, ADC or DAC used | 1.8 | 3.6 | V | ds p. 50 |
| FT I/O input high level, CMOS, 2.0 V ≤ VDD ≤ 3.6 V | | 5.5 | V | ds p. 79 |
| FT I/O input high level, CMOS, 1.65 V ≤ VDD ≤ 2.0 V | | 5.25 | V | ds p. 79 |
| Standard I/O input high level | | VDD + 0.3 | V | ds p. 79 |
| Ambient temperature, maximum dissipation | -40 | 85 | °C | ds p. 50 |
| Ambient temperature, low dissipation | -40 | 105 | °C | ds p. 50 |
| Junction temperature | -40 | 105 | °C | ds p. 50 |

## Pins

Checked pad by pad against Figure 7, "STM32L15xCx LQFP48 pinout" (ds p. 34, top view, pin-1 dot at pad 1), and the "LQFP48 or UFQFPN48" column of Table 9 (ds pp. 36–40). All 48 pads carry the signal the library gives them. Functions are Table 9's without the LCD segment/common functions, which exist on the STM32L152 only (ds p. 10).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VLCD | VLCD | Power | LCD supply on STM32L152; on STM32L151 connect to VDD | ds p. 36 |
| 2 | PC13 | PC13-WKUP2 | I/O (FT) | RTC_TAMP1, RTC_TS, RTC_OUT, WKUP2 | ds p. 36 |
| 3 | PC14 | PC14-OSC32_IN | I/O | OSC32_IN (LSE) | ds p. 36 |
| 4 | PC15 | PC15-OSC32_OUT | I/O | OSC32_OUT (LSE) | ds p. 36 |
| 5 | PH0 | PH0-OSC_IN | I | OSC_IN (HSE) | ds p. 36 |
| 6 | PH1 | PH1-OSC_OUT | O | OSC_OUT (HSE) | ds p. 36 |
| 7 | NRST | NRST | I/O | Reset, active low | ds p. 36 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 36 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 36 |
| 10 | PA0 | PA0-WKUP1 | I/O (FT) | WKUP1, USART2_CTS, ADC_IN0, TIM2_CH1_ETR, COMP1_INP | ds p. 36 |
| 11 | PA1 | PA1 | I/O (FT) | USART2_RTS, ADC_IN1, TIM2_CH2, COMP1_INP | ds p. 36 |
| 12 | PA2 | PA2 | I/O (FT) | USART2_TX, ADC_IN2, TIM2_CH3, TIM9_CH1, COMP1_INP | ds p. 37 |
| 13 | PA3 | PA3 | I/O | USART2_RX, ADC_IN3, TIM2_CH4, TIM9_CH2, COMP1_INP | ds p. 37 |
| 14 | PA4 | PA4 | I/O | SPI1_NSS, USART2_CK, ADC_IN4, DAC_OUT1, COMP1_INP | ds p. 37 |
| 15 | PA5 | PA5 | I/O | SPI1_SCK, ADC_IN5, DAC_OUT2, TIM2_CH1_ETR, COMP1_INP | ds p. 37 |
| 16 | PA6 | PA6 | I/O (FT) | SPI1_MISO, ADC_IN6, TIM3_CH1, TIM10_CH1, COMP1_INP | ds p. 37 |
| 17 | PA7 | PA7 | I/O (FT) | SPI1_MOSI, ADC_IN7, TIM3_CH2, TIM11_CH1, COMP1_INP | ds p. 37 |
| 18 | PB0 | PB0 | I/O | ADC_IN8, TIM3_CH3, COMP1_INP, VREF_OUT | ds p. 37 |
| 19 | PB1 | PB1 | I/O (FT) | ADC_IN9, TIM3_CH4, COMP1_INP, VREF_OUT | ds p. 37 |
| 20 | PB2 | PB2 | I/O (FT) | PB2/BOOT1 | ds p. 37 |
| 21 | PB10 | PB10 | I/O (FT) | I2C2_SCL, USART3_TX, TIM2_CH3 | ds p. 37 |
| 22 | PB11 | PB11 | I/O (FT) | I2C2_SDA, USART3_RX, TIM2_CH4 | ds p. 37 |
| 23 | VSS | VSS_1 | Power | Ground | ds p. 38 |
| 24 | VDD | VDD_1 | Power | Digital supply | ds p. 38 |
| 25 | PB12 | PB12 | I/O (FT) | SPI2_NSS, I2C2_SMBA, USART3_CK, ADC_IN18, COMP1_INP, TIM10_CH1 | ds p. 38 |
| 26 | PB13 | PB13 | I/O (FT) | SPI2_SCK, USART3_CTS, ADC_IN19, COMP1_INP, TIM9_CH1 | ds p. 38 |
| 27 | PB14 | PB14 | I/O (FT) | SPI2_MISO, USART3_RTS, ADC_IN20, COMP1_INP, TIM9_CH2 | ds p. 38 |
| 28 | PB15 | PB15 | I/O (FT) | SPI2_MOSI, ADC_IN21, COMP1_INP, TIM11_CH1, RTC_REFIN | ds p. 38 |
| 29 | PA8 | PA8 | I/O (FT) | USART1_CK, MCO | ds p. 38 |
| 30 | PA9 | PA9 | I/O (FT) | USART1_TX | ds p. 38 |
| 31 | PA10 | PA10 | I/O (FT) | USART1_RX | ds p. 38 |
| 32 | PA11 | PA11 | I/O (FT) | USART1_CTS, USB_DM, SPI1_MISO | ds p. 38 |
| 33 | PA12 | PA12 | I/O (FT) | USART1_RTS, USB_DP, SPI1_MOSI | ds p. 38 |
| 34 | PA13 | PA13 | I/O (FT) | JTMS/SWDAT after reset | ds p. 38 |
| 35 | VSS | VSS_2 | Power | Ground | ds p. 38 |
| 36 | VDD | VDD_2 | Power | Digital supply | ds p. 39 |
| 37 | PA14 | PA14 | I/O (FT) | JTCK/SWCLK after reset | ds p. 39 |
| 38 | PA15 | PA15 | I/O (FT) | JTDI after reset; TIM2_CH1_ETR, PA15, SPI1_NSS | ds p. 39 |
| 39 | PB3 | PB3 | I/O (FT) | JTDO after reset; TIM2_CH2, PB3, SPI1_SCK, COMP2_INM | ds p. 39 |
| 40 | PB4 | PB4 | I/O (FT) | NJTRST after reset; TIM3_CH1, PB4, SPI1_MISO, COMP2_INP | ds p. 39 |
| 41 | PB5 | PB5 | I/O (FT) | I2C1_SMBA, TIM3_CH2, SPI1_MOSI, COMP2_INP | ds p. 39 |
| 42 | PB6 | PB6 | I/O (FT) | I2C1_SCL, TIM4_CH1, USART1_TX | ds p. 39 |
| 43 | PB7 | PB7 | I/O (FT) | I2C1_SDA, TIM4_CH2, USART1_RX, PVD_IN | ds p. 39 |
| 44 | BOOT0 | BOOT0 | Input | Boot memory selection | ds p. 39 |
| 45 | PB8 | PB8 | I/O (FT) | TIM4_CH3, I2C1_SCL, TIM10_CH1 | ds p. 39 |
| 46 | PB9 | PB9 | I/O (FT) | TIM4_CH4, I2C1_SDA, TIM11_CH1 | ds p. 39 |
| 47 | VSS | VSS_3 | Power | Ground | ds p. 40 |
| 48 | VDD | VDD_3 | Power | Digital supply | ds p. 40 |

## Typical application

- **Pad 1 (VLCD) goes to VDD on the STM32L151** (ds p. 40, note 4 of Table 9).
- Decouple VDD with 100 nF capacitors plus one 4.7 µF; Figure 12 is drawn for VDD1–5 and labels the bank "11 × 100 nF + 1 × 4.7 µF" (ds p. 48). Decouple VDDA with 10 nF + 1 µF (ds p. 48).
- Connect every VDD, VDDA, VSS and VSSA pin; VDDA and VSSA must be connected to VDD and VSS (ds p. 18, ds p. 49). Power VDD and VDDA from the same source; at most 300 mV between them in power-up and operation (ds p. 50).
- NRST has an internal 30–60 kΩ pull-up. Figure 19 adds 0.1 µF from NRST to ground (ds p. 82, ds p. 83).
- Boot: BOOT0 (pad 44) and BOOT1 on PB2 (pad 20) select flash, system memory or RAM. The bootloader uses USART1 or USART2 (ds p. 19, ds p. 37). Strap BOOT0 low for flash boot.
- HSE: a 1–24 MHz crystal on PH0/PH1 with CL1 = CL2 in 5–25 pF, sized with about 10 pF of pin and board capacitance (ds p. 68). It is required for USB, and the CPU must then run at 24 or 32 MHz (ds p. 21, ds p. 29).
- LSE: a 32.768 kHz crystal on PC14/PC15 with CL1 = CL2 in 5–15 pF. ST recommends CL ≤ 7 pF and never 12.5 pF (ds p. 70).
- USB: PA11 = USB_DM, PA12 = USB_DP (ds p. 38). For full-speed compliance D+ should be pulled up with 1.5 kΩ to 3.0–3.6 V, and VDD must be 3.0–3.6 V (ds p. 89).
- Debug: SWD on PA13 and PA14; full JTAG also takes PA15, PB3 and PB4 after reset (ds pp. 38–39).

## Layout notes

- ST's recommended land (Figure 41) is 0.50 mm pitch, 0.30 x 1.20 mm pads, 9.70 mm outer span, 7.30 mm inner span, 5.80 mm pad-row length (ds p. 109). The library uses a different land: see "Not established by the sources".
- Package: body D1 = E1 = 7.000 mm nominal, D = E = 9.000 mm over leads, lead width b = 0.170–0.270 mm, height A = 1.600 mm (ds p. 109). No exposed pad.
- Place crystals and load capacitors as close as possible to the oscillator pins (ds p. 68, ds p. 69).

## Pitfalls

- **Pad 1 is VLCD, not VBAT.** There is no backup-supply pin; on the L151 it must be tied to VDD (ds p. 34, ds p. 40).
- 32 MHz needs voltage range 1 (ds p. 15), and range 1 is available only for VDD from 2.0 V up (ds p. 14).
- Not every pin is 5 V tolerant. On the LQFP48, PC14, PC15, PH0, PH1, NRST, PA3, PA4, PA5 and PB0 have no FT marking (ds pp. 36–37). FT inputs above VDD + 0.5 V need the internal pull-up/down disabled (ds p. 79).
- USB below VDD = 3.0 V is not USB compliant (ds p. 14, ds p. 89).
- PA13–PA15, PB3 and PB4 come out of reset as JTAG/SWD pins (ds pp. 38–39).
- PH0/PH1 and PC14/PC15 become oscillator pins only when HSE or LSE is switched on; the oscillator then has priority over the GPIO (ds p. 40).
- An external reset circuit must pull NRST below VIL(NRST) max, or the reset is ignored (ds p. 83).

## Not established by the sources

- **The source revision is old.** It is Doc ID 17659 Rev 7 of October 2012 (ds p. 1, ds p. 116). This note does not know which revision st.com serves at the catalog URL; later revisions may change limits.
- **Library land is KiCad's, not ST's.** `qfp::KICAD_LQFP_48_7X7MM_P0_5MM` has 0.30 x 1.475 mm pads with centres at ±4.1625 mm, so its outer span is 9.80 mm and its inner span 6.85 mm. ST's Figure 41 gives 0.30 x 1.20 mm pads, 9.70 mm outer and 7.30 mm inner (ds p. 109). The library pads reach 0.225 mm further inward and 0.05 mm further outward per side. Pitch (0.50 mm), pad width (0.30 mm) and pad-row length (5.80 mm) agree. Pin 1 is at the lower left with numbering counter-clockwise in top view in both the library (+y-down frame, after its quarter turn) and Figure 41.
- **VLCD is typed `passive` and `required` in the library**, with no comment that the L151 needs it tied to VDD (ds p. 40).
- **NRST is typed `input` in the library**; Table 9 types it I/O, and Table 45 gives an NRST output-low level (ds p. 36, ds p. 82).
- **Missing operating conditions.** Table 14 of this revision gives a power-dissipation limit for BGA100 only and no I/O input-voltage range (ds p. 50). The FT and standard I/O limits above are the input-high-level maxima of Table 42 (ds p. 79).
- **PH0/PH1 pin types.** Table 9 types PH0 as input and PH1 as output (ds p. 36), but note 6 says both can be general-purpose I/O when HSE is off (ds p. 40). The library types both `bidirectional`.
- **Decoupling count.** Figure 12 labels "11 × 100 nF" for a 100-pin package with VDD1–5 (ds p. 48); the datasheet does not say how many to use on the LQFP48's three VDD pads.
- **D+ pull-up.** Table 51 says D+ "should be pulled up" with 1.5 kΩ (ds p. 89); the datasheet does not say whether the part has an internal pull-up.
- **BOOT0/BOOT1 truth table** is not in this datasheet; it refers to AN2606 (ds p. 19). No BOOT0 resistor value or BOOT0 input threshold is given.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- No crystal ESR or drive-level limit is given beyond AN2867 (ds p. 68).
