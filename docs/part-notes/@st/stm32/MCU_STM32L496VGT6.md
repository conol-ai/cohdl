# STM32L496VGT6 — ultra-low-power Arm Cortex-M4F MCU, 80 MHz, 1 MB flash, 320 KB SRAM, USB OTG FS, LQFP100 14x14 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32L496VGTx",
  "parts": ["@st/stm32::MCU_STM32L496VGT6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32L496VGT6", "lcsc": "C717701"},
    {"mfr": "STMicroelectronics", "mpn": "STM32L496VGT6TR", "lcsc": "C2055934"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32L496xx datasheet: Ultra-low-power Arm Cortex-M4 32-bit MCU+FPU, 100DMIPS, up to 1MB Flash, 320KB SRAM, USB OTG FS, audio, ext. SMPS",
     "revision": "DS11585 Rev 11, 2020-01-20",
     "url": "https://www.st.com/resource/en/datasheet/stm32l496vg.pdf",
     "sha256": "f285c0ab8ccf5d95df61cf9aad87c3766bc7f1bb49956970701ea088f24fbdaa",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/89ef549cd18e11b8d07356c25dfa0dcc.pdf"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS"; ST's DS11585 Rev 11 on every page). It was retrieved from LCSC's datasheet mirror (part page <https://www.lcsc.com/datasheet/C717701.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The file is not stored in the repository; the SHA-256 above identifies it. Page numbers below are PDF page numbers; in this file they equal the printed page numbers (PDF page 66 is printed "66/281").

## Summary

The STM32L496VGT6 is an ultra-low-power 80 MHz Arm Cortex-M4 microcontroller with FPU, 1 MB of dual-bank flash and 320 KB SRAM in a 14 x 14 mm, 0.5 mm-pitch LQFP100, rated -40 to 85 °C ambient (suffix 6) (ds p. 1, ds p. 15, ds p. 126, ds p. 277). Its deciding features are the large memory, USB OTG full-speed (device, host and OTG) with an internal PHY, two CAN controllers, an LCD driver, a camera interface and Chrom-ART graphics (ds p. 1, ds p. 15, ds p. 58). The LQFP100 variant has 83 GPIOs (ds p. 16). Do not choose it when you need more than 80 MHz (ds p. 126). The external-SMPS variant STM32L496VGT6P has a different LQFP100 pinout (VDD12 on pins 48 and 98) and is not covered here (ds p. 67, ds p. 277).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU / AHB / APB clock | 0 | | 80 | MHz | fHCLK, fPCLK1, fPCLK2 | ds p. 126 |
| Flash / SRAM | | 1024 / 320 | | KB | STM32L496VG (G = 1 MB) | ds p. 15, ds p. 277 |
| GPIO count | | 83 | | | STM32L496Vx | ds p. 16 |
| HSE crystal/resonator frequency | 4 | 8 | 48 | MHz | fOSC_IN | ds p. 163 |
| HSE startup time | | 2 | | ms | VDD stabilized, 8 MHz standard crystal | ds p. 163 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 164 |
| HSI48 accuracy, factory calibrated | | | ±3 | % | VDD = 3.0-3.6 V, TA = -15 to 85 °C | ds p. 171 |
| BOR0 threshold, rising / falling | 1.62 / 1.60 | 1.66 / 1.64 | 1.70 / 1.69 | V | VBOR0 | ds p. 128 |
| Reset temporization after BOR0 | | 250 | 400 | µs | tRSTTEMPO, VDD rising | ds p. 128 |
| NRST internal pull-up | 25 | 40 | 55 | kΩ | RPU, VIN = VSS | ds p. 184 |
| NRST input low / high level | | | 0.3 VDDIOx / - | V | VIL(NRST) max; VIH(NRST) min is 0.7 VDDIOx | ds p. 184 |
| USB_FS_DP pull-up, idle | 0.9 | 1.25 | 1.575 | kΩ | PA12, VIN = VSS | ds p. 228 |
| USB driver impedance | 28 | | 44 | Ω | ZDRV, no series resistors needed | ds p. 229 |
| Run current | | 9.44 | 10.3 | mA | 80 MHz, PLL on, all peripherals disabled, flash, TA = 25 °C | ds p. 133 |
| Thermal resistance θJA | | 42 | | °C/W | LQFP100 14 x 14 mm | ds p. 274 |
| Package height A | | | 1.600 | mm | LQFP100 | ds p. 264 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDDX - VSS (including VDD, VDDA, VDDIO2, VDDUSB, VLCD, VBAT) | -0.3 to 4.0 | V | ds p. 125 |
| VREF+ above VDDA | 0.4 max | V | ds p. 125 |
| VIN on FT_xxx pins | VSS - 0.3 to min(VDD, VDDA, VDDIO2, VDDUSB, VLCD) + 4.0 (pull-up/down disabled above 4 V) | V | ds p. 125 |
| VIN on TT_xx pins and any other pin | VSS - 0.3 to 4.0 | V | ds p. 125 |
| VIN on BOOT0 | VSS to 9.0 | V | ds p. 125 |
| Total current into all VDD / out of all VSS lines | 150 / 150 | mA | ds p. 125 |
| Current into each VDD pin / out of each VSS pin | 100 / 100 | mA | ds p. 125 |
| Output current per I/O, sunk or sourced | 20 | mA | ds p. 125 |
| Total output current of all I/Os, sunk / sourced | 100 / 100 | mA | ds p. 125 |
| Injected current per pin | -5 / +0 | mA | ds p. 125 |
| Total injected current | 25 | mA | ds p. 125 |
| Junction temperature | 150 | °C | ds p. 126 |
| Storage temperature | -65 to +150 | °C | ds p. 126 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 1.71 | 3.6 | V | ds p. 126 |
| VDDA, ADC or COMP used | 1.62 | 3.6 | V | ds p. 126 |
| VDDA, DAC or OPAMP used | 1.8 | 3.6 | V | ds p. 126 |
| VDDA, VREFBUF used | 2.4 | 3.6 | V | ds p. 126 |
| VDDA, no analog peripheral used | 0 | 3.6 | V | ds p. 126 |
| VREF+, VDDA >= 2 V | 2 | VDDA | V | ds p. 186 |
| VREF+, VDDA < 2 V | VDDA | VDDA | V | ds p. 186 |
| VBAT | 1.55 | 3.6 | V | ds p. 127 |
| VDDUSB, USB used | 3.0 | 3.6 | V | ds p. 127 |
| VDDUSB, USB not used | 0 | 3.6 | V | ds p. 127 |
| VIN, TT_xx I/O | -0.3 | VDDIOx + 0.3 | V | ds p. 127 |
| VIN, BOOT0 | 0 | 9 | V | ds p. 127 |
| VIN, other I/O | -0.3 | min(min(VDD, VDDA, VDDIO2, VDDUSB, VLCD) + 3.6, 5.5) | V | ds p. 127 |
| Power dissipation, LQFP100, TA = 85 °C (suffix 6) | | 476 | mW | ds p. 127 |
| Ambient temperature, suffix 6, max dissipation | -40 | 85 | °C | ds p. 127 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 127 |

## Pins

The table was generated by script from the library's `STM32L496VGTx` pins block and the standard (non-SMPS) LQFP100 column of Table 15 (ds pp. 71-97), and every pad was compared with both Table 15 and the pinout of Figure 13 (ds p. 66): all 100 pad signals match. As a control, the same comparison against the LQFP100_SMPS column and Figure 14 fails on pins 48 and 98 (VDD12), so the check tells the two pinouts apart. Library `VREFP`/`VREFM` are the datasheet's VREF+/VREF-. Pin 19 is `VSSA` in the library and in Figure 13, and `VSSA/VREF-` in Table 15.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | PE2 | PE2 | I/O (FT_l) | GPIO | ds p. 71 |
| 2 | PE3 | PE3 | I/O (FT_l) | GPIO | ds p. 71 |
| 3 | PE4 | PE4 | I/O (FT) | GPIO | ds p. 71 |
| 4 | PE5 | PE5 | I/O (FT) | GPIO | ds p. 71 |
| 5 | PE6 | PE6 | I/O (FT) | GPIO | ds p. 71 |
| 6 | VBAT | VBAT | Power | Backup supply (RTC, LSE, backup registers) | ds p. 71 |
| 7 | PC13 | PC13 | I/O (FT) | GPIO; WKUP2, RTC_TAMP1/RTC_TS/RTC_OUT | ds p. 71 |
| 8 | PC14 | PC14-OSC32_IN | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 72 |
| 9 | PC15 | PC15-OSC32_OUT | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 72 |
| 10 | VSS | VSS | Power | Ground | ds p. 72 |
| 11 | VDD | VDD | Power | Digital supply | ds p. 72 |
| 12 | PH0 | PH0-OSC_IN (PH0) | I/O (FT) | GPIO; OSC_IN (HSE) | ds p. 73 |
| 13 | PH1 | PH1-OSC_OUT | I/O (FT) | GPIO; OSC_OUT (HSE) | ds p. 73 |
| 14 | NRST | NRST | I/O (RST) | Reset input/output, active low, embedded pull-up | ds p. 73 |
| 15 | PC0 | PC0 | I/O (FT_fla) | GPIO | ds p. 73 |
| 16 | PC1 | PC1 | I/O (FT_fla) | GPIO | ds p. 74 |
| 17 | PC2 | PC2 | I/O (FT_la) | GPIO | ds p. 74 |
| 18 | PC3 | PC3 | I/O (FT_la) | GPIO | ds p. 74 |
| 19 | VSSA | VSSA/VREF- | Power | Analog ground (Table 15: VSSA/VREF-) | ds p. 74 |
| 20 | VREFM | VREF- | Power | ADC/DAC negative reference (VREF-) | ds p. 74 |
| 21 | VREFP | VREF+ | Power | ADC/DAC reference input; VREFBUF output | ds p. 74 |
| 22 | VDDA | VDDA | Power | Analog supply | ds p. 74 |
| 23 | PA0 | PA0 | I/O (FT_a) | GPIO | ds p. 75 |
| 24 | PA1 | PA1 | I/O (FT_la) | GPIO | ds p. 75 |
| 25 | PA2 | PA2 | I/O (FT_la) | GPIO | ds p. 75 |
| 26 | PA3 | PA3 | I/O (TT_la) | GPIO | ds p. 75 |
| 27 | VSS | VSS | Power | Ground | ds p. 76 |
| 28 | VDD | VDD | Power | Digital supply | ds p. 76 |
| 29 | PA4 | PA4 | I/O (TT_a) | GPIO | ds p. 76 |
| 30 | PA5 | PA5 | I/O (TT_a) | GPIO | ds p. 76 |
| 31 | PA6 | PA6 | I/O (FT_la) | GPIO | ds p. 76 |
| 32 | PA7 | PA7 | I/O (FT_fla) | GPIO | ds p. 77 |
| 33 | PC4 | PC4 | I/O (FT_la) | GPIO | ds p. 77 |
| 34 | PC5 | PC5 | I/O (FT_la) | GPIO | ds p. 77 |
| 35 | PB0 | PB0 | I/O (TT_la) | GPIO | ds p. 77 |
| 36 | PB1 | PB1 | I/O (FT_la) | GPIO | ds p. 77 |
| 37 | PB2 | PB2 | I/O (FT_la) | GPIO | ds p. 78 |
| 38 | PE7 | PE7 | I/O (FT) | GPIO | ds p. 79 |
| 39 | PE8 | PE8 | I/O (FT) | GPIO | ds p. 79 |
| 40 | PE9 | PE9 | I/O (FT) | GPIO | ds p. 79 |
| 41 | PE10 | PE10 | I/O (FT) | GPIO | ds p. 79 |
| 42 | PE11 | PE11 | I/O (FT) | GPIO | ds p. 79 |
| 43 | PE12 | PE12 | I/O (FT) | GPIO | ds p. 80 |
| 44 | PE13 | PE13 | I/O (FT) | GPIO | ds p. 80 |
| 45 | PE14 | PE14 | I/O (FT) | GPIO | ds p. 80 |
| 46 | PE15 | PE15 | I/O (FT) | GPIO | ds p. 80 |
| 47 | PB10 | PB10 | I/O (FT_fl) | GPIO | ds p. 80 |
| 48 | PB11 | PB11 | I/O (FT_fl) | GPIO | ds p. 81 |
| 49 | VSS | VSS | Power | Ground | ds p. 82 |
| 50 | VDD | VDD | Power | Digital supply | ds p. 82 |
| 51 | PB12 | PB12 | I/O (FT_l) | GPIO | ds p. 82 |
| 52 | PB13 | PB13 | I/O (FT_fl) | GPIO | ds p. 83 |
| 53 | PB14 | PB14 | I/O (FT_fl) | GPIO | ds p. 83 |
| 54 | PB15 | PB15 | I/O (FT_l) | GPIO | ds p. 83 |
| 55 | PD8 | PD8 | I/O (FT_l) | GPIO | ds p. 84 |
| 56 | PD9 | PD9 | I/O (FT_l) | GPIO | ds p. 84 |
| 57 | PD10 | PD10 | I/O (FT_l) | GPIO | ds p. 84 |
| 58 | PD11 | PD11 | I/O (FT_l) | GPIO | ds p. 84 |
| 59 | PD12 | PD12 | I/O (FT_fl) | GPIO | ds p. 84 |
| 60 | PD13 | PD13 | I/O (FT_fl) | GPIO | ds p. 85 |
| 61 | PD14 | PD14 | I/O (FT_l) | GPIO | ds p. 85 |
| 62 | PD15 | PD15 | I/O (FT_l) | GPIO | ds p. 85 |
| 63 | PC6 | PC6 | I/O (FT_l) | GPIO | ds p. 86 |
| 64 | PC7 | PC7 | I/O (FT_l) | GPIO | ds p. 86 |
| 65 | PC8 | PC8 | I/O (FT_l) | GPIO | ds p. 87 |
| 66 | PC9 | PC9 | I/O (FT_fl) | GPIO | ds p. 87 |
| 67 | PA8 | PA8 | I/O (FT_l) | GPIO | ds p. 87 |
| 68 | PA9 | PA9 | I/O (FT_lu) | GPIO; OTG_FS_VBUS | ds p. 87 |
| 69 | PA10 | PA10 | I/O (FT_lu) | GPIO; OTG_FS_ID | ds p. 88 |
| 70 | PA11 | PA11 | I/O (FT_u) | GPIO; OTG_FS_DM | ds p. 88 |
| 71 | PA12 | PA12 | I/O (FT_u) | GPIO; OTG_FS_DP | ds p. 88 |
| 72 | PA13 | PA13 (JTMS/SWDIO) | I/O (FT) | JTMS/SWDIO after reset | ds p. 88 |
| 73 | VDDUSB | VDDUSB | Power | USB transceiver supply | ds p. 88 |
| 74 | VSS | VSS | Power | Ground | ds p. 88 |
| 75 | VDD | VDD | Power | Digital supply | ds p. 89 |
| 76 | PA14 | PA14 (JTCK/SWCLK) | I/O (FT) | JTCK/SWCLK after reset | ds p. 90 |
| 77 | PA15 | PA15 (JTDI) | I/O (FT_l) | JTDI after reset | ds p. 90 |
| 78 | PC10 | PC10 | I/O (FT_l) | GPIO | ds p. 91 |
| 79 | PC11 | PC11 | I/O (FT_l) | GPIO | ds p. 91 |
| 80 | PC12 | PC12 | I/O (FT_l) | GPIO | ds p. 91 |
| 81 | PD0 | PD0 | I/O (FT) | GPIO | ds p. 91 |
| 82 | PD1 | PD1 | I/O (FT) | GPIO | ds p. 92 |
| 83 | PD2 | PD2 | I/O (FT_l) | GPIO | ds p. 92 |
| 84 | PD3 | PD3 | I/O (FT) | GPIO | ds p. 92 |
| 85 | PD4 | PD4 | I/O (FT) | GPIO | ds p. 92 |
| 86 | PD5 | PD5 | I/O (FT) | GPIO | ds p. 92 |
| 87 | PD6 | PD6 | I/O (FT) | GPIO | ds p. 93 |
| 88 | PD7 | PD7 | I/O (FT) | GPIO | ds p. 93 |
| 89 | PB3 | PB3 (JTDO/TRACESWO) | I/O (FT_la) | JTDO/TRACESWO after reset | ds p. 94 |
| 90 | PB4 | PB4 (NJTRST) | I/O (FT_fla) | NJTRST after reset | ds p. 95 |
| 91 | PB5 | PB5 | I/O (FT_la) | GPIO | ds p. 95 |
| 92 | PB6 | PB6 | I/O (FT_fa) | GPIO | ds p. 95 |
| 93 | PB7 | PB7 | I/O (FT_fla) | GPIO | ds p. 96 |
| 94 | PH3 | PH3-BOOT0 | I/O (FT) | BOOT0 (boot select) or GPIO | ds p. 96 |
| 95 | PB8 | PB8 | I/O (FT_fl) | GPIO | ds p. 96 |
| 96 | PB9 | PB9 | I/O (FT_fl) | GPIO | ds p. 96 |
| 97 | PE0 | PE0 | I/O (FT_l) | GPIO | ds p. 97 |
| 98 | PE1 | PE1 | I/O (FT_l) | GPIO | ds p. 97 |
| 99 | VSS | VSS | Power | Ground | ds p. 97 |
| 100 | VDD | VDD | Power | Digital supply | ds p. 97 |

## Typical application

- Decouple VDD with n x 100 nF plus one 4.7 µF, VDDA with 10 nF + 1 µF, and VREF+ with 100 nF + 1 µF (Figure 22) (ds p. 123).
- Connect all VDD, VDDA, VDDUSB and VBAT pins and all VSS and VSSA pins; none may be left open (ds p. 125). VREF- (pin 20) is the negative reference and sits at VSSA (ds p. 186).
- When VDDA or VDDUSB functions are not used, those supplies should preferably be shorted to VDD; if they are tied to ground, the I/Os they supply are not 5 V tolerant (ds p. 23). LQFP100 has no VDDIO2 pin and no PG[15:2] port (ds p. 23, ds p. 66).
- Power sequencing: while VDD is below 1 V, VDDA, VDDUSB, VDDIO2 and VLCD must stay below VDD + 300 mV; above 1 V they are independent (ds p. 24).
- VREF+ (pin 21) is a separate pin. Feed it from VDDA when VDDA < 2 V, otherwise from 2 V to VDDA; VREF+ may exceed VDDA by at most 0.4 V (absolute). The internal VREFBUF can drive it with 2.048 V or 2.5 V, which needs VDDA >= 2.4 V (ds p. 44, ds p. 125, ds p. 126, ds p. 186).
- NRST (pin 14): internal 25-55 kΩ pull-up; add 0.1 µF from NRST to VSS against parasitic resets (Figure 34) (ds p. 184, ds p. 185).
- BOOT0 is pin 94 (`PH3` in the library). BOOT0 comes from this pin or from an option bit; with nBOOT1 it selects user flash, system memory or SRAM. An empty-flash check forces system-memory boot when the first flash location is unprogrammed. The bootloader reprograms flash over USART, I2C, SPI, CAN or USB OTG FS DFU (ds p. 22).
- Optional HSE: a 4-48 MHz crystal on PH0-OSC_IN (pin 12) and PH1-OSC_OUT (pin 13), with CL1 = CL2 in the 5-20 pF range sized with about 10 pF of pin and board capacitance included (ds p. 163).
- Optional LSE: a 32.768 kHz crystal on PC14-OSC32_IN (pin 8) and PC15-OSC32_OUT (pin 9). Do not fit a resistor between OSC32_IN and OSC32_OUT (ds p. 165).
- USB OTG FS: PA11 = OTG_FS_DM (pin 70), PA12 = OTG_FS_DP (pin 71), PA10 = OTG_FS_ID (pin 69), PA9 = OTG_FS_VBUS (pin 68) (ds pp. 87-88). No external series resistors are needed on DP and DM (ds p. 229), and the PHY needs no external resistor for HNP/SRP (ds p. 59). With VBUS sensing enabled, leave PA9 in its default floating-input state; the sensing block draws about 200 µA on it (ds p. 228). For host/OTG modes, a power switch is needed for bus-powered devices (ds p. 59). The 48 MHz USB clock can come from MSI trimmed by the LSE or from HSI48 with clock recovery from SOF, so device mode needs no HSE crystal (ds p. 58, ds p. 59).
- SWD: bring out PA13 (SWDIO, pin 72), PA14 (SWCLK, pin 76), NRST, VDD and VSS. After reset these are debug pins, with internal pull-ups on PA15, PA13 and PB4 and a pull-down on PA14 (ds p. 98). Unless a note says otherwise, every I/O is an analog input during and after reset (ds p. 70).

## Layout notes

- Place each decoupling capacitor as close as possible to its supply/ground pair, or below it on the PCB underside (ds p. 123).
- Place the HSE and LSE crystals and their load capacitors as close as possible to the oscillator pins (ds p. 163, ds p. 164).
- ST's recommended land (Figure 76) is 0.30 x 1.20 mm pads at 0.5 mm pitch, 16.7 mm outer span and 14.3 mm inner span, 12.3 mm along each row (ds p. 265).
- The library land `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` (KiCad's generic land) has the same pitch and numbering order but 0.3 x 1.6 mm pads at ±7.675 mm: 16.95 mm outer and 13.75 mm inner span. Each pad is 0.40 mm longer than ST's, reaching 0.125 mm further out and 0.275 mm further in.
- Pin 1: ST's outline and land (Figures 75 and 76) put pin 1 at the left end of the bottom row, numbered counter-clockwise (1-25 bottom, 26-50 right, 51-75 top, 76-100 left) (ds p. 264, ds p. 265). The library land puts pin 1 at the top of the left column with the same counter-clockwise order, a quarter turn from ST's drawing. The supplier binding's `rotate 270` was set from the assembler's previews and is not re-examined here. The pinout figure, by contrast, draws pin 1 at the top of the left side (ds p. 66).
- Body D1 = 14.00 mm nominal, tip-to-tip D = 16.00 mm, lead width b 0.17-0.27 mm, foot length L 0.45-0.75 mm, height 1.600 mm max (Table 128) (ds p. 264, ds p. 265).
- No exposed pad. θJA is 42 °C/W (ds p. 274).

## Pitfalls

- BOOT0 hides behind a GPIO name: pin 94 is `PH3`, `optional` in the library (ds p. 96). Leaving it floating leaves the boot source to chance unless an option bit selects BOOT0 (ds p. 22).
- PC13-PC15 are behind the backup power switch: at most 3 mA, at most 2 MHz with 30 pF, and never as a current source such as an LED driver (ds p. 98).
- Only FT pins tolerate 5 V, and only with the internal pull-up/pull-down disabled above the lowest relevant supply + 0.3 V. PA3, PA4, PA5 and PB0 (TT) are 3.6 V tolerant only (ds p. 70, ds pp. 75-77, ds p. 127).
- PA9, PA10, PA11 and PA12 are `_u` I/Os supplied by VDDUSB (ds p. 70, ds pp. 87-88); VDDUSB must be 3.0-3.6 V when USB is used (ds p. 127).
- Do not use the external-SMPS (P-suffix) pinout or Figure 14: on that variant pins 48 and 98 are VDD12, not PB11 and PE1 (ds p. 67, ds p. 81, ds p. 97).

## Not established by the sources

- **Library vs datasheet: no pad signal differs.** All 100 pads of `STM32L496VGTx` match the standard LQFP100 column of Table 15 and Figure 13 (ds pp. 66, 71-97).
- **Pin 19 name.** The library and Figure 13 call it `VSSA` (ds p. 66); Table 15 calls it `VSSA/VREF-`, with a separate `VREF-` on pin 20 (ds p. 74). The datasheet does not say whether pin 19 also serves as VREF- on this package.
- **Land differs from ST's recommended footprint.** `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` uses 0.3 x 1.6 mm pads spanning 16.95 mm outer and 13.75 mm inner; ST's Figure 76 gives 0.30 x 1.20 mm pads spanning 16.7 mm and 14.3 mm (ds p. 265). Pitch and numbering order agree; pin 1 is a quarter turn away, which the supplier binding's `rotate 270` accounts for at assembly. Whether the longer KiCad pads matter for assembly is not established here.
- `NRST` is declared `[input]` in the library; the datasheet types it I/O, "bidirectional reset pin" (ds p. 70, ds p. 73).
- Table 19 gives BOOT0 a 9.0 V absolute maximum and Table 22 a 0-9 V operating range (ds p. 125, ds p. 127), but on this package BOOT0 shares pin 94 with PH3, an FT I/O (ds p. 96). The datasheet does not say which limit applies to PH3-BOOT0; keep it within the FT limits.
- The BOOT0/nBOOT1 truth table and option-byte defaults are in the reference manual, not this datasheet (ds p. 22). No BOOT0 pull resistor value is given.
- How many 100 nF capacitors to use for the five VDD pins is not given beyond "n x 100 nF"; no capacitor is shown for VDDUSB or VBAT (ds p. 123).
- No USB ESD protection, VBUS divider or connector circuit is specified.
- No crystal part, ESR limit or exact load-capacitor value is given beyond the 5-20 pF range and AN2867 (ds p. 163).
