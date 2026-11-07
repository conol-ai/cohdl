# STM32L412K8T6 — ultra-low-power Arm Cortex-M4F MCU, 80 MHz, 64 KB flash, crystal-less USB FS, LQFP32 7x7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32L412K8Tx",
  "parts": ["@st/stm32::MCU_STM32L412K8T6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32L412K8T6", "lcsc": "C529436"},
    {"mfr": "STMicroelectronics", "mpn": "STM32L412K8T6TR", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "STM32L412xx datasheet: Ultra-low-power Arm Cortex-M4 32-bit MCU+FPU, 100DMIPS, up to 128KB Flash, 40KB SRAM, analog, ext. SMPS",
     "revision": "DS12469 Rev 7, 2019-09-26",
     "url": "https://www.st.com/resource/en/datasheet/stm32l412k8.pdf",
     "sha256": "5104ae3be2936485c977a38fc8a3da92018581bb5c13c6e0f3b501e1f6a7902e",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/c135c7e1d055bf5e5346d026c14b6f49.pdf"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS"; ST's DS12469 Rev 7 on every page). It was retrieved from LCSC's datasheet mirror (part page <https://www.lcsc.com/datasheet/C529436.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The file is not stored in the repository; the SHA-256 above identifies it. Page numbers below are PDF page numbers; in this file they equal the printed page numbers (PDF page 56 is printed "56/193"). Several figures in this file have a font-encoded text layer (for example Figures 12, 17, 29 and 57); they were read from renderings of the pages.

## Summary

The STM32L412K8T6 is an ultra-low-power 80 MHz Arm Cortex-M4 microcontroller with FPU, 64 KB flash and 40 KB SRAM in a 7 x 7 mm, 0.8 mm-pitch LQFP32, rated -40 to 85 °C ambient (suffix 6) (ds p. 1, ds p. 13, ds p. 18, ds p. 78, ds p. 191). Its deciding features are low-power modes and a crystal-less USB 2.0 full-speed device with an embedded D+ pull-up (ds p. 1, ds p. 47). The LQFP32 variant has 26 GPIOs (ds p. 14). In it, several supply and clock pins are not bonded out: there is no VBAT, no VDDUSB, no separate VSSA or VREF+, and no PH0-OSC_IN/PH1-OSC_OUT, so no HSE crystal (ds pp. 56-62). Do not choose it when you need an HSE crystal, a battery-backed RTC, or more than 80 MHz (ds p. 78).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU / AHB / APB clock | 0 | | 80 | MHz | fHCLK, fPCLK1, fPCLK2 | ds p. 78 |
| Flash / SRAM | | 64 / 40 | | KB | STM32L412K8 | ds p. 13 |
| GPIO count | | 26 | | | STM32L412K8 | ds p. 14 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 117 |
| HSI48 accuracy, factory calibrated | | | ±3 | % | VDD = 3.0-3.6 V, TA = -15 to 85 °C | ds p. 124 |
| BOR0 threshold, rising / falling | 1.62 / 1.60 | 1.66 / 1.64 | 1.70 / 1.69 | V | VBOR0 | ds p. 79 |
| Reset temporization after BOR0 | | 250 | 400 | µs | tRSTTEMPO, VDD rising | ds p. 79 |
| NRST internal pull-up | 25 | 40 | 55 | kΩ | RPU, VIN = VSS | ds p. 136 |
| NRST input low / high level | | | 0.3 VDDIOx / - | V | VIL(NRST) max; VIH(NRST) min is 0.7 VDDIOx | ds p. 136 |
| USB_DP embedded pull-up, idle | 900 | 1250 | 1600 | Ω | RPUI | ds p. 164 |
| USB driver impedance | 28 | 36 | 44 | Ω | ZDRV, no series resistors needed | ds p. 164 |
| Run current | | 7.30 | 7.75 | mA | 80 MHz, PLL on, all peripherals disabled, flash, TA = 25 °C | ds p. 85 |
| Thermal resistance θJA | | 68 | | °C/W | LQFP32 7 x 7 mm | ds p. 188 |
| Package height A | | | 1.600 | mm | LQFP32 | ds p. 185 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDDX - VSS (including VDD, VDDA, VDDUSB, VBAT) | -0.3 to 4.0 | V | ds p. 75 |
| VIN on FT_xxx pins | VSS - 0.3 to min(VDD, VDDA, VDDUSB) + 4.0 (pull-up/down disabled above 4 V) | V | ds p. 75, ds p. 76 |
| VIN on TT_xx pins and any other pin | VSS - 0.3 to 4.0 | V | ds p. 75 |
| Total current into all VDD / out of all VSS lines | 140 / 140 | mA | ds p. 76 |
| Current into each VDD pin / out of each VSS pin | 100 / 100 | mA | ds p. 76 |
| Output current per I/O, sunk or sourced | 20 | mA | ds p. 76 |
| Total output current of all I/Os, sunk / sourced | 100 / 100 | mA | ds p. 76 |
| Injected current per pin | -5 / +0 | mA | ds p. 76 |
| Total injected current | 25 | mA | ds p. 76 |
| Junction temperature | 150 | °C | ds p. 77 |
| Storage temperature | -65 to +150 | °C | ds p. 77 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 1.71 | 3.6 | V | ds p. 78 |
| VDDA, ADC or COMP used | 1.62 | 3.6 | V | ds p. 78 |
| VDDA, OPAMP used | 1.8 | 3.6 | V | ds p. 78 |
| VDDA, ADC, OPAMP, COMP not used | 0 | 3.6 | V | ds p. 78 |
| VREF+, VDDA >= 2 V | 2 | VDDA | V | ds p. 138 |
| VREF+, VDDA < 2 V | VDDA | VDDA | V | ds p. 138 |
| VDDUSB, USB used | 3.0 | 3.6 | V | ds p. 78 |
| VIN, TT_xx I/O | -0.3 | VDDIOx + 0.3 | V | ds p. 78 |
| VIN, other I/O | -0.3 | min(min(VDD, VDDA, VDDUSB) + 3.6, 5.5) | V | ds p. 78 |
| Power dissipation, LQFP32, TA = 85 °C (suffix 6) | | 294 | mW | ds p. 78 |
| Ambient temperature, suffix 6, max dissipation | -40 | 85 | °C | ds p. 79 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 79 |
| USB crystal-less operation temperature | -15 | 85 | °C | ds p. 164 |

## Pins

The table was generated by script from the library's `STM32L412K8Tx` pins block and the LQFP32 column of Table 14 (ds pp. 56-62), and every pad was compared with both Table 14 and the pinout of Figure 12 (ds p. 54, transcribed from a rendering because its text layer is font-encoded): all 32 pads match. The library name `VDDA_VREFP` is the datasheet's `VDDA/VREF+`.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VDD | VDD | Power | Digital supply | ds p. 62 |
| 2 | PC14 | PC14-OSC32_IN (PC14) | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 56 |
| 3 | PC15 | PC15-OSC32_OUT | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 56 |
| 4 | NRST | NRST | I/O (RST) | Reset input/output, active low, embedded pull-up | ds p. 56 |
| 5 | VDDA_VREFP | VDDA/VREF+ | Power | Analog supply and ADC reference on one pin | ds p. 57 |
| 6 | PA0 | PA0-CK_IN | I/O (FT_a) | GPIO; CK_IN, WKUP1, ADC1_IN5 | ds p. 57 |
| 7 | PA1 | PA1 | I/O (FT_a) | GPIO | ds p. 57 |
| 8 | PA2 | PA2 | I/O (FT_a) | GPIO | ds p. 57 |
| 9 | PA3 | PA3 | I/O (TT_a) | GPIO | ds p. 57 |
| 10 | PA4 | PA4 | I/O (TT_a) | GPIO | ds p. 57 |
| 11 | PA5 | PA5 | I/O (TT_a) | GPIO | ds p. 57 |
| 12 | PA6 | PA6 | I/O (FT_a) | GPIO | ds p. 58 |
| 13 | PA7 | PA7 | I/O (FT_fa) | GPIO | ds p. 58 |
| 14 | PB0 | PB0 | I/O (FT_a) | GPIO | ds p. 58 |
| 15 | PB1 | PB1 | I/O (FT_a) | GPIO | ds p. 58 |
| 16 | VSS | VSS | Power | Ground | ds p. 59 |
| 17 | VDD | VDD | Power | Digital supply | ds p. 59 |
| 18 | PA8 | PA8 | I/O (FT) | GPIO | ds p. 60 |
| 19 | PA9 | PA9 | I/O (FT_f) | GPIO | ds p. 60 |
| 20 | PA10 | PA10 | I/O (FT_f) | GPIO | ds p. 60 |
| 21 | PA11 | PA11 | I/O (FT_u) | GPIO; USB_DM | ds p. 60 |
| 22 | PA12 | PA12 | I/O (FT_u) | GPIO; USB_DP | ds p. 60 |
| 23 | PA13 | PA13 (JTMS/SWDIO) | I/O (FT) | JTMS/SWDIO after reset | ds p. 60 |
| 24 | PA14 | PA14 (JTCK/SWCLK) | I/O (FT) | JTCK/SWCLK after reset | ds p. 60 |
| 25 | PA15 | PA15 (JTDI) | I/O (FT) | JTDI after reset | ds p. 60 |
| 26 | PB3 | PB3 (JTDO/TRACESWO) | I/O (FT_a) | JTDO/TRACESWO after reset | ds p. 61 |
| 27 | PB4 | PB4 (NJTRST) | I/O (FT_fa) | NJTRST after reset | ds p. 61 |
| 28 | PB5 | PB5 | I/O (FT) | GPIO | ds p. 61 |
| 29 | PB6 | PB6 | I/O (FT_fa) | GPIO | ds p. 61 |
| 30 | PB7 | PB7 | I/O (FT_fa) | GPIO; PVD_IN | ds p. 61 |
| 31 | PH3 | PH3-BOOT0 (BOOT0) | I/O (FT) | BOOT0 (boot select) or GPIO | ds p. 61 |
| 32 | VSS | VSS | Power | Ground | ds p. 62 |

## Typical application

- Decouple VDD with n x 100 nF plus one 4.7 µF (Figure 17) (ds p. 74). Figure 17 draws VDDA with 10 nF + 1 µF and VREF+ with 100 nF + 1 µF; on LQFP32 both rails are pin 5, so both sets land on that one pin (ds p. 57, ds p. 74).
- Connect all VDD and VSS pins and VDDA/VREF+; none may be left open (ds p. 76).
- When the VDDA functions are not used, VDDA should preferably be shorted to VDD (ds p. 19). With VREF+ on the same pin, VDDA >= 2 V keeps VREF+ within its range automatically (ds p. 138).
- Power sequencing: while VDD is below 1 V, the other supplies must stay below VDD + 300 mV; above 1 V they are independent (ds p. 20).
- NRST (pin 4): internal 25-55 kΩ pull-up; add 0.1 µF from NRST to VSS, as close as possible to the device (Figure 29) (ds p. 136, ds p. 137).
- BOOT0 is pin 31 (`PH3` in the library). BOOT0 comes from this pin or from the nSWBOOT0 option bit; with BOOT1 it selects user flash, system memory or SRAM. An empty-flash check forces system-memory boot when the first flash location is unprogrammed. The bootloader reprograms flash over USART, I2C, SPI or USB FS DFU (ds p. 19).
- Clock: LQFP32 has no HSE crystal pins (PH0-OSC_IN and PH1-OSC_OUT are not bonded) (ds p. 56). Use the internal oscillators: HSI16, MSI (auto-trimmed by LSE) or HSI48 with clock recovery (ds p. 1).
- Optional LSE: a 32.768 kHz crystal on PC14-OSC32_IN (pin 2) and PC15-OSC32_OUT (pin 3). Do not fit a resistor between OSC32_IN and OSC32_OUT (ds p. 117, ds p. 118).
- USB device: route PA11 = USB_DM (pin 21) and PA12 = USB_DP (pin 22) to the connector. The D+ pull-up is embedded and no series termination resistors are needed (ds p. 47, ds p. 164). Crystal-less operation uses the internal 48 MHz oscillator trimmed to USB SOF (ds p. 47), specified only from -15 to 85 °C (ds p. 164).
- SWD: bring out PA13 (SWDIO, pin 23), PA14 (SWCLK, pin 24), NRST, VDD and VSS (ds p. 60). Unless a note says otherwise, every I/O is an analog input during and after reset (ds p. 55).

## Layout notes

- Place each decoupling capacitor as close as possible to its supply/ground pair, or below it on the PCB underside (ds p. 74).
- Place the LSE crystal and its load capacitors as close as possible to the oscillator pins (ds p. 117).
- ST's recommended land (Figure 57) is 0.50 x 1.20 mm pads at 0.80 mm pitch, 9.70 mm outer span and 7.30 mm inner span across opposite rows, with pin 1 at the left end of the bottom row and numbering counter-clockwise (ds p. 186).
- The library land `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` (KiCad's generic land) has the same pitch, pin-1 corner and numbering order, but 0.5 x 1.5 mm pads at ±4.175 mm: 9.85 mm outer and 6.85 mm inner span. Each pad is 0.30 mm longer than ST's, reaching 0.075 mm further out and 0.225 mm further in.
- Pin 1: Figure 56 marks pin 1 at the lower left of the top view, with pins 1-8 along the bottom (ds p. 184), which is where the library land puts it. The supplier binding carries no rotation.
- Body D1 = 7.00 mm nominal, tip-to-tip D = 9.00 mm, lead width b 0.30-0.45 mm, foot length L 0.45-0.75 mm, height 1.600 mm max (Table 102) (ds p. 185).
- No exposed pad. θJA is 68 °C/W (ds p. 188).

## Pitfalls

- BOOT0 hides behind a GPIO name: pin 31 is `PH3`, `optional` in the library (ds p. 61). Leaving it floating leaves the boot source to chance unless the option bits select nSWBOOT0 (ds p. 19).
- No HSE crystal on this package: firmware or an example that assumes an 8 MHz crystal on PH0/PH1 cannot be built on LQFP32 (ds p. 56). PA0 is named PA0-CK_IN on 32-pin packages (ds p. 57).
- No VBAT pin: the RTC and backup registers have no separate battery supply on LQFP32 (ds p. 56).
- Only FT pins tolerate 5 V, and only with the internal pull-up/pull-down disabled above min(VDD, VDDA, VDDUSB) + 0.3 V. PA3, PA4 and PA5 (TT_a) are 3.6 V tolerant only (ds p. 55, ds p. 57, ds p. 78, ds p. 79).
- USB needs 3.0-3.6 V on the USB supply for full electrical characteristics; it works, degraded, down to 2.7 V (ds p. 164).
- The external-SMPS variants (suffix P) have a different pinout; this note covers only the standard K8T6 (ds p. 191).

## Not established by the sources

- **Library vs datasheet: no pad signal differs.** All 32 pads of `STM32L412K8Tx` match Table 14 and Figure 12 (ds pp. 54, 56-62).
- **Land differs from ST's recommended footprint.** `qfp::KICAD_LQFP_32_7X7MM_P0_8MM` uses 0.5 x 1.5 mm pads spanning 9.85 mm outer and 6.85 mm inner; ST's Figure 57 gives 0.50 x 1.20 mm pads spanning 9.70 mm and 7.30 mm (ds p. 186). Pitch, pin-1 corner and order agree. Whether the longer KiCad pads matter for assembly is not established here.
- `NRST` is declared `[input]` in the library; the datasheet types it I/O, "bidirectional reset pin" (ds p. 55, ds p. 56).
- How the USB transceiver is supplied on LQFP32 is not stated: the `_u` I/O structure of PA11/PA12 is "supplied by VDDUSB", but LQFP32 has no VDDUSB pin (ds p. 55, ds p. 60). Likewise, where VSSA goes on LQFP32 is not stated beyond "VREF- can be internally connected to VSSA, depending on the package" (ds p. 139).
- The datasheet contradicts itself on the voltage reference buffer: page 1 lists an "Accurate 2.5 V or 2.048 V reference voltage buffered output", while Table 2 lists the internal voltage reference buffer as "No" (ds p. 1, ds p. 14). On LQFP32, VREF+ shares pin 5 with VDDA either way (ds p. 57).
- What CK_IN on PA0 is for (for example an external clock input) is not described beyond the pin name (ds p. 57).
- The BOOT0/BOOT1/nSWBOOT0 truth table and option-byte defaults are in the reference manual, not this datasheet (ds p. 19). No BOOT0 pull resistor value is given.
- How many 100 nF capacitors to use for the two VDD pins (1, 17) is not given beyond "n x 100 nF" (ds p. 74).
- No USB ESD protection, VBUS sensing circuit or connector shielding is specified.
- No crystal part, ESR limit or exact load-capacitor value for the LSE is given beyond AN2867 (ds p. 117).
