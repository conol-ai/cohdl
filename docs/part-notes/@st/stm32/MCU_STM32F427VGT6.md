# STM32F427VGT6 — Arm Cortex-M4F MCU, 180 MHz, 1 MB flash, 256 KB SRAM, LQFP100 14x14 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32F427VGTx",
  "parts": ["@st/stm32::MCU_STM32F427VGT6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32F427VGT6", "lcsc": "C117815"},
    {"mfr": "STMicroelectronics", "mpn": "STM32F427VGT6TR", "lcsc": "C2053693"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32F427xx STM32F429xx datasheet (Chinese edition): ARM Cortex-M4 32b MCU+FPU, 225DMIPS, up to 2MB Flash/256+4KB RAM, USB OTG HS/FS, Ethernet, 17 TIMs, 3 ADCs, 20 comm. interfaces, camera & LCD-TFT",
     "revision": "DocID024030 Rev 4, January 2015",
     "url": "https://www.st.com/resource/en/datasheet/stm32f427vg.pdf",
     "sha256": "9eccb942b0df757c906979b15deb36ac24a52f1f9f425e220da25ccc68266451",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/04048fced4844b75bea9eec7a66e5bf4.pdf"}
  ],
  "status": "draft"
}
```

**Source.** The file was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C117815.pdf>) on 2026-11-07, because st.com's robots.txt was unavailable to the policy-bound client that day. It is ST-authored: the PDF's Author field is STMICROELECTRONICS and every page carries ST's DocID024030 Rev 4 footer. It is the **Chinese-language edition** ("数据手册 - 生产数据") of the STM32F427xx/F429xx datasheet; the English edition at the official URL was not compared. The PDF is not in the repository. Page numbers below are PDF page numbers; in this file they equal the printed folios (1/226 to 226/226).

## Summary

The STM32F427VGT6 is an Arm Cortex-M4 microcontroller with FPU, up to 180 MHz, 1024 KB flash and 256 KB system SRAM plus 4 KB backup SRAM, in a 14 x 14 mm LQFP100 (ds p. 1, ds p. 14, ds p. 93, ds p. 216). It has USB OTG FS and HS (FS PHY on both), Ethernet MAC, FMC, camera interface, 3 ADCs and 2 DACs; the F427 has no LCD-TFT controller (ds p. 15). The LQFP100 has 82 GPIOs (ds p. 15). On this package the voltage regulator and the internal power-on reset are always on: there is no BYPASS_REG or PDR_ON pin, so VDD must stay at or above 1.8 V (ds p. 15, ds p. 24, ds p. 30). In the LQFP100 the FMC is limited to Bank1 (multiplexed NOR/PSRAM on NE1) or Bank2 (NAND on NE2), and port G is absent (ds p. 15). Suffix 6 is rated −40 to 85 °C ambient at full dissipation (ds p. 94).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB clock fHCLK | 0 | | 180 | MHz | Scale 1, regulator on, over-drive on | ds p. 93 |
| AHB clock fHCLK, no over-drive | 0 | | 168 | MHz | Scale 1, over-drive off | ds p. 93 |
| APB1 / APB2 clock | 0 | | 45 / 90 | MHz | Over-drive on | ds p. 93 |
| Flash | | 1024 | | KB | STM32F427VG | ds p. 14 |
| SRAM (system + backup) | | 256 + 4 | | KB | | ds p. 14 |
| GPIO count | | 82 | | | STM32F427Vx | ds p. 15 |
| HSE crystal frequency | 4 | | 26 | MHz | fOSC_IN | ds p. 118 |
| HSE startup time | | 2 | | ms | VDD stabilized, standard crystal | ds p. 118 |
| HSE maximum critical crystal gm | | | 1 | mA/V | Startup | ds p. 118 |
| LSE startup time | | 2 | | s | 32.768 kHz, VDD stabilized | ds p. 119 |
| LSE maximum critical crystal gm | | | 0.56 | µA/V | Startup | ds p. 119 |
| POR/PDR threshold, falling | 1.60 | 1.68 | 1.76 | V | | ds p. 97 |
| POR/PDR threshold, rising | 1.64 | 1.72 | 1.80 | V | | ds p. 97 |
| POR reset temporization | 0.5 | 1.5 | 3.0 | ms | tRSTTEMPO | ds p. 97 |
| Regulator inrush current | | 160 | 200 | mA | POR or wakeup from Standby | ds p. 98 |
| NRST weak pull-up | 30 | 40 | 50 | kΩ | VIN = VSS | ds p. 136 |
| NRST filtered pulse | | | 100 | ns | VF(NRST) | ds p. 136 |
| NRST not-filtered pulse | 300 | | | ns | VNF(NRST), VDD > 2.7 V | ds p. 136 |
| BOOT0 VIL | | | 0.1VDD + 0.1 | V | 1.75 V ≤ VDD ≤ 3.6 V, −40 °C ≤ TA ≤ 105 °C | ds p. 131 |
| BOOT0 VIH | 0.17VDD + 0.7 | | | V | Same conditions | ds p. 132 |
| FT/TTa/NRST VIL / VIH (tested) | | | 0.3VDD / 0.7VDD (min) | V | 1.7 V ≤ VDD ≤ 3.6 V | ds p. 131 |
| VCAP capacitor CEXT | | 2.2 | | µF | Per VCAP pin, ESR < 2 Ω | ds p. 96 |
| Thermal resistance θJA | | 43 | | °C/W | LQFP100 14 x 14 mm | ds p. 215 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD − VSS (includes VDDA and VBAT) | −0.3 to 4.0 | V | ds p. 91 |
| VIN on FT pins | VSS − 0.3 to VDD + 4.0 | V | ds p. 91 |
| VIN on TTa pins and any other pin | VSS − 0.3 to 4.0 | V | ds p. 91 |
| VIN on BOOT0 | VSS to 9.0 | V | ds p. 91 |
| Variation between VDD pins, between ground pins | 50 | mV | ds p. 91 |
| Total current into all VDD / out of all VSS | 270 / −270 | mA | ds p. 92 |
| Current per VDD / VSS pin | 100 / −100 | mA | ds p. 92 |
| Current per I/O pin, sunk / sourced | 25 / −25 | mA | ds p. 92 |
| Total I/O current, sunk / sourced | 120 / −120 | mA | ds p. 92 |
| Injected current, FT, NRST and BOOT0 pins | −5 / +0 | mA | ds p. 92 |
| Injected current, TTa pins | ±5 | mA | ds p. 92 |
| Total injected current | ±25 | mA | ds p. 92 |
| Storage temperature | −65 to +150 | °C | ds p. 92 |
| Maximum junction temperature | 125 | °C | ds p. 92 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, LQFP100 (internal reset always on) | 1.8 | 3.6 | V | ds p. 15, ds p. 24 |
| VDDA, ADC limited to 1.2 Msps (must equal VDD) | 1.7 | 2.4 | V | ds p. 94 |
| VDDA, ADC up to 2.4 Msps (must equal VDD) | 2.4 | 3.6 | V | ds p. 94 |
| VDD − VDDA during power-up and power-down | | 0.3 | V | ds p. 95 |
| VBAT | 1.65 | 3.6 | V | ds p. 94 |
| VDD for the USB OTG FS transceiver | 3.0 | 3.6 | V | ds p. 147 |
| V12 on VCAP_1/VCAP_2, Scale 1 (180 MHz with over-drive) | 1.26 | 1.40 | V | ds p. 94 |
| VIN, RST and FT pins, 2 V ≤ VDD ≤ 3.6 V | −0.3 | 5.5 | V | ds p. 94 |
| VIN, RST and FT pins, VDD ≤ 2 V | −0.3 | 5.2 | V | ds p. 94 |
| VIN, TTa pins | −0.3 | VDDA + 0.3 | V | ds p. 94 |
| VIN, BOOT0 | 0 | 9 | V | ds p. 94 |
| Power dissipation, LQFP100, TA = 85 °C (suffix 6) | | 465 | mW (printed "mΩ") | ds p. 94 |
| Ambient temperature, suffix 6, maximum dissipation | −40 | 85 | °C | ds p. 94 |
| Ambient temperature, suffix 6, low dissipation | −40 | 105 | °C | ds p. 94 |
| Junction temperature, suffix 6 | −40 | 105 | °C | ds p. 95 |
| VDD rise and fall time rate (regulator on) | 20 | ∞ | µs/V | ds p. 96 |

## Pins

Generated from the library's pins block and checked by script against Table 10, the LQFP100 column (ds pp. 51-69), and by eye against Figure 11, the LQFP100 pinout (ds p. 43). Every pad's signal matches. The Source column gives the Table 10 page of each row.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | PE2 | PE2 | I/O (FT) | GPIO | ds p. 51 |
| 2 | PE3 | PE3 | I/O (FT) | GPIO | ds p. 51 |
| 3 | PE4 | PE4 | I/O (FT) | GPIO | ds p. 51 |
| 4 | PE5 | PE5 | I/O (FT) | GPIO | ds p. 52 |
| 5 | PE6 | PE6 | I/O (FT) | GPIO | ds p. 52 |
| 6 | VBAT | VBAT | Power (S) | Backup-domain supply (RTC, LSE, backup registers) when VDD is absent | ds p. 52 |
| 7 | PC13 | PC13 | I/O (FT) | GPIO | ds p. 52 |
| 8 | PC14 | PC14-OSC32_IN(PC14) | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 52 |
| 9 | PC15 | PC15-OSC32_OUT(PC15) | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 52 |
| 10 | VSS | VSS | Power (S) | Ground | ds p. 53 |
| 11 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 53 |
| 12 | PH0 | PH0-OSC_IN(PH0) | I/O (FT) | GPIO; OSC_IN (HSE) | ds p. 54 |
| 13 | PH1 | PH1-OSC_OUT(PH1) | I/O (FT) | GPIO; OSC_OUT (HSE) | ds p. 54 |
| 14 | NRST | NRST | I/O (RST) | Reset input, active low; also driven low by internal resets | ds p. 54 |
| 15 | PC0 | PC0 | I/O (FT) | GPIO | ds p. 54 |
| 16 | PC1 | PC1 | I/O (FT) | GPIO | ds p. 54 |
| 17 | PC2 | PC2 | I/O (FT) | GPIO | ds p. 54 |
| 18 | PC3 | PC3 | I/O (FT) | GPIO | ds p. 54 |
| 19 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 55 |
| 20 | VSSA | VSSA | Power (S) | Analog ground | ds p. 55 |
| 21 | VREFP | VREF+ | Power (S) | ADC/DAC reference, positive | ds p. 55 |
| 22 | VDDA | VDDA | Power (S) | Analog supply (ADC, DAC, PLL, RCs) | ds p. 55 |
| 23 | PA0 | PA0-WKUP(PA0) | I/O (FT) | GPIO; WKUP | ds p. 55 |
| 24 | PA1 | PA1 | I/O (FT) | GPIO | ds p. 55 |
| 25 | PA2 | PA2 | I/O (FT) | GPIO | ds p. 55 |
| 26 | PA3 | PA3 | I/O (FT) | GPIO | ds p. 56 |
| 27 | VSS | VSS | Power (S) | Ground | ds p. 56 |
| 28 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 56 |
| 29 | PA4 | PA4 | I/O (TTa) | GPIO | ds p. 56 |
| 30 | PA5 | PA5 | I/O (TTa) | GPIO | ds p. 56 |
| 31 | PA6 | PA6 | I/O (FT) | GPIO | ds p. 56 |
| 32 | PA7 | PA7 | I/O (FT) | GPIO | ds p. 56 |
| 33 | PC4 | PC4 | I/O (FT) | GPIO | ds p. 57 |
| 34 | PC5 | PC5 | I/O (FT) | GPIO | ds p. 57 |
| 35 | PB0 | PB0 | I/O (FT) | GPIO | ds p. 57 |
| 36 | PB1 | PB1 | I/O (FT) | GPIO | ds p. 57 |
| 37 | PB2 | PB2-BOOT1(PB2) | I/O (FT) | GPIO; BOOT1 | ds p. 57 |
| 38 | PE7 | PE7 | I/O (FT) | GPIO | ds p. 58 |
| 39 | PE8 | PE8 | I/O (FT) | GPIO | ds p. 58 |
| 40 | PE9 | PE9 | I/O (FT) | GPIO | ds p. 58 |
| 41 | PE10 | PE10 | I/O (FT) | GPIO | ds p. 58 |
| 42 | PE11 | PE11 | I/O (FT) | GPIO | ds p. 58 |
| 43 | PE12 | PE12 | I/O (FT) | GPIO | ds p. 58 |
| 44 | PE13 | PE13 | I/O (FT) | GPIO | ds p. 58 |
| 45 | PE14 | PE14 | I/O (FT) | GPIO | ds p. 58 |
| 46 | PE15 | PE15 | I/O (FT) | GPIO | ds p. 58 |
| 47 | PB10 | PB10 | I/O (FT) | GPIO | ds p. 59 |
| 48 | PB11 | PB11 | I/O (FT) | GPIO | ds p. 59 |
| 49 | VCAP_1 | VCAP_1 | Power (S) | Core regulator output; 2.2 µF to VSS | ds p. 59 |
| 50 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 59 |
| 51 | PB12 | PB12 | I/O (FT) | GPIO; OTG_HS_ID | ds p. 60 |
| 52 | PB13 | PB13 | I/O (FT) | GPIO | ds p. 60 |
| 53 | PB14 | PB14 | I/O (FT) | GPIO; OTG_HS_DM | ds p. 60 |
| 54 | PB15 | PB15 | I/O (FT) | GPIO; OTG_HS_DP | ds p. 61 |
| 55 | PD8 | PD8 | I/O (FT) | GPIO | ds p. 61 |
| 56 | PD9 | PD9 | I/O (FT) | GPIO | ds p. 61 |
| 57 | PD10 | PD10 | I/O (FT) | GPIO | ds p. 61 |
| 58 | PD11 | PD11 | I/O (FT) | GPIO | ds p. 61 |
| 59 | PD12 | PD12 | I/O (FT) | GPIO | ds p. 61 |
| 60 | PD13 | PD13 | I/O (FT) | GPIO | ds p. 61 |
| 61 | PD14 | PD14 | I/O (FT) | GPIO | ds p. 61 |
| 62 | PD15 | PD15 | I/O (FT) | GPIO | ds p. 61 |
| 63 | PC6 | PC6 | I/O (FT) | GPIO | ds p. 62 |
| 64 | PC7 | PC7 | I/O (FT) | GPIO | ds p. 63 |
| 65 | PC8 | PC8 | I/O (FT) | GPIO | ds p. 63 |
| 66 | PC9 | PC9 | I/O (FT) | GPIO | ds p. 63 |
| 67 | PA8 | PA8 | I/O (FT) | GPIO | ds p. 63 |
| 68 | PA9 | PA9 | I/O (FT) | GPIO | ds p. 63 |
| 69 | PA10 | PA10 | I/O (FT) | GPIO; OTG_FS_ID | ds p. 63 |
| 70 | PA11 | PA11 | I/O (FT) | GPIO; OTG_FS_DM | ds p. 63 |
| 71 | PA12 | PA12 | I/O (FT) | GPIO; OTG_FS_DP | ds p. 63 |
| 72 | PA13 | PA13(JTMS-SWDIO) | I/O (FT) | JTMS-SWDIO after reset | ds p. 63 |
| 73 | VCAP_2 | VCAP_2 | Power (S) | Core regulator output; 2.2 µF to VSS | ds p. 64 |
| 74 | VSS | VSS | Power (S) | Ground | ds p. 64 |
| 75 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 64 |
| 76 | PA14 | PA14(JTCK-SWCLK) | I/O (FT) | JTCK-SWCLK after reset | ds p. 64 |
| 77 | PA15 | PA15(JTDI) | I/O (FT) | JTDI after reset | ds p. 65 |
| 78 | PC10 | PC10 | I/O (FT) | GPIO | ds p. 65 |
| 79 | PC11 | PC11 | I/O (FT) | GPIO | ds p. 65 |
| 80 | PC12 | PC12 | I/O (FT) | GPIO | ds p. 65 |
| 81 | PD0 | PD0 | I/O (FT) | GPIO | ds p. 65 |
| 82 | PD1 | PD1 | I/O (FT) | GPIO | ds p. 65 |
| 83 | PD2 | PD2 | I/O (FT) | GPIO | ds p. 65 |
| 84 | PD3 | PD3 | I/O (FT) | GPIO | ds p. 65 |
| 85 | PD4 | PD4 | I/O (FT) | GPIO | ds p. 65 |
| 86 | PD5 | PD5 | I/O (FT) | GPIO | ds p. 65 |
| 87 | PD6 | PD6 | I/O (FT) | GPIO | ds p. 66 |
| 88 | PD7 | PD7 | I/O (FT) | GPIO | ds p. 66 |
| 89 | PB3 | PB3(JTDO/TRACESWO) | I/O (FT) | JTDO/TRACESWO after reset | ds p. 67 |
| 90 | PB4 | PB4(NJTRST) | I/O (FT) | NJTRST after reset | ds p. 67 |
| 91 | PB5 | PB5 | I/O (FT) | GPIO | ds p. 67 |
| 92 | PB6 | PB6 | I/O (FT) | GPIO | ds p. 68 |
| 93 | PB7 | PB7 | I/O (FT) | GPIO | ds p. 68 |
| 94 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 68 |
| 95 | PB8 | PB8 | I/O (FT) | GPIO | ds p. 68 |
| 96 | PB9 | PB9 | I/O (FT) | GPIO | ds p. 68 |
| 97 | PE0 | PE0 | I/O (FT) | GPIO | ds p. 68 |
| 98 | PE1 | PE1 | I/O (FT) | GPIO | ds p. 68 |
| 99 | VSS | VSS | Power (S) | Ground | ds p. 68 |
| 100 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 68 |

## Typical application

- Decoupling, from Figure 22 (ds p. 90):
  - VDD: 100 nF per VDD pin plus one 4.7 µF ceramic on one of the VDD pins. The figure is drawn for 15 VDD pins (15 x 100 nF + 1 x 4.7 µF); the LQFP100 has six (pads 11, 19, 28, 50, 75, 100).
  - VDDA: 100 nF + 1 µF. VREF+: 100 nF + 1 µF.
  - VCAP_1 (pad 49) and VCAP_2 (pad 73): one 2.2 µF ceramic each to VSS, ESR below 2 Ω (ds p. 96).
  - Place every capacitor as close as possible to its pins, or under them on the PCB underside (ds p. 90).
- Connect every VDD, VDDA, VSS and VSSA pin; none may be left open (ds p. 91). VDDA = VDD and VSSA = VSS (ds p. 24, ds p. 90).
- VREF+ (pad 21) is a separate pin on this package and must be supplied. Keep VDDA − VREF+ below 1.2 V (ds p. 95).
- VBAT (pad 6): 1.65-3.6 V backup supply (ds p. 94). The datasheet gives no capacitor and no rule for VBAT without a battery.
- Regulator and reset: on the LQFP100 the regulator is always on and the internal POR/PDR is always on, so no BYPASS_REG or PDR_ON strap exists (ds p. 26, ds p. 30).
- NRST (pad 14): internal 30-50 kΩ pull-up. Figure 37 shows an optional 0.1 µF capacitor from NRST to ground against parasitic resets (ds p. 136, ds p. 137).
- Boot: BOOT0 (pad 94) and BOOT1 on PB2 (pad 37) select user flash, system memory or SRAM boot. The datasheet gives no truth table and refers to AN2606 (ds p. 24, ds p. 57).
- HSE on PH0/PH1 (pads 12, 13): 4-26 MHz crystal with CL1 = CL2 of 5-25 pF, sized with about 10 pF of pin and board capacitance; REXT depends on the crystal (ds p. 118).
- LSE on PC14/PC15 (pads 8, 9): 32.768 kHz crystal with its load capacitors close to the pins (ds p. 118, ds p. 119).
- USB OTG FS: PA11 = OTG_FS_DM (pad 70), PA12 = OTG_FS_DP (pad 71), PA10 = OTG_FS_ID (pad 69). The transceiver needs VDD = 3.0-3.6 V (ds p. 63, ds p. 147).
- SWD: PA13 = JTMS-SWDIO (pad 72) and PA14 = JTCK-SWCLK (pad 76) after reset (ds p. 63, ds p. 64).

## Layout notes

- Land: ST's recommended LQFP100 footprint, Figure 82, has 0.5 mm pitch, 0.3 x 1.2 mm pads, 16.7 mm outer span, 14.3 mm inner span and 12.3 mm across one row of 25 pads (ds p. 194).
- The library's land `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` has 0.5 mm pitch and 0.3 mm pad width, but 1.6 mm pads centred at ±7.675 mm. That is a 16.95 mm outer and 13.75 mm inner span. See "Not established by the sources".
- Package: body D1 = E1 = 14.00 mm nominal, D = E = 16.00 mm across the leads, lead width b = 0.17-0.27 mm, foot L = 0.45-0.75 mm, height A ≤ 1.600 mm (ds p. 193).
- Pin 1: Figure 81 marks pin 1 at a corner, with pins numbered counter-clockwise in top view (1-25, then 26-50, 51-75, 76-100) (ds p. 192). The library land has pad 1 at a corner with the same counter-clockwise order, so the land is not mirrored. The supplier binding's `rotate 270` turns this land onto the assembler's frame; it was read from the assembler's preview and is not reopened here.
- No exposed pad. θJA is 43 °C/W (ds p. 215).

## Pitfalls

- PA4 and PA5 are TTa, not 5 V tolerant: VIN ≤ VDDA + 0.3 V (ds p. 56, ds p. 94). FT pins are 5 V tolerant except in analog mode, and except PC14, PC15, PH0 and PH1 in oscillator mode (ds p. 69).
- An FT pin above VDD + 0.3 V needs its internal pull-up and pull-down disabled (ds p. 95).
- PC13-PC15 run from the backup power switch. In output mode they are limited to 2 MHz with 30 pF and must not source current, for example to drive an LED (ds p. 69).
- VDD minimum is 1.8 V on this package. The 1.7 V figures in Table 17 need an external supervisor through PDR_ON, which the LQFP100 does not have (ds p. 24, ds p. 25, ds p. 94).
- Over-drive, and so 180 MHz, is not available at VDD = 1.7-2.1 V (ds p. 95).
- An external reset circuit must pull NRST below VIL(NRST) max, or the reset is ignored (ds p. 137).
- BOOT0 tolerates 9 V absolute, but its thresholds differ from the standard I/O levels (ds p. 91, ds p. 131, ds p. 132).
- The library's pin names PC14, PC15, PH0, PH1, PA0 and PB2 are the datasheet's PC14-OSC32_IN, PC15-OSC32_OUT, PH0-OSC_IN, PH1-OSC_OUT, PA0-WKUP and PB2-BOOT1; VREFP is VREF+ (ds p. 52, ds p. 54, ds p. 55, ds p. 57).

## Not established by the sources

- **Land pattern differs from ST's recommendation (minor).** `qfp::KICAD_LQFP_100_14X14MM_P0_5MM` uses 1.6 mm pads at ±7.675 mm (16.95 mm outer, 13.75 mm inner span). ST's Figure 82 recommends 1.2 mm pads with a 16.7 mm outer and 14.3 mm inner span (ds p. 194). The library pads reach 0.125 mm further out and 0.275 mm further in. Pitch (0.5 mm), pad width (0.3 mm) and pin-1 position agree. This is KiCad's generic land, not ST's; it was not checked against IPC-7351.
- **No pad-signal disagreement.** All 100 pads match Table 10 (ds pp. 51-69) and Figure 11 (ds p. 43).
- **Library pin types the datasheet does not match exactly.** NRST is typed `input`, but Table 10 gives it as I/O, RST type: internal resets drive it low (ds p. 54, ds p. 137). This is the library's known lack of an open-drain type.
- **Source edition.** The pinned file is the Chinese edition DocID024030 Rev 4 (January 2015). Whether the English edition at the official URL has a later revision with changed values was not checked. The Chinese edition has typographic errors:
  - the LQFP100 power dissipation unit is printed "mΩ" (ds p. 94);
  - the LSE startup footnote says "HSE" (ds p. 119);
  - a pin-table footnote names "PC18" where PI8 is meant (ds p. 69).
- The BOOT0/BOOT1 boot-mode truth table and the bootloader pin assignments are not in the datasheet; it points to AN2606 (ds p. 24).
- No BOOT0 pull-down value, VBAT capacitor value, or rule for VBAT when no battery is fitted is given.
- No crystal part, ESR limit or exact load-capacitor value is given beyond the 5-25 pF range (ds p. 118).
- The datasheet gives no USB ESD protection, VBUS sensing circuit or series resistors for PA11/PA12.
