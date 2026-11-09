# STM32F767BIT6 — Arm Cortex-M7 MCU, 216 MHz, 2 MB flash, 512 KB SRAM, LQFP208 28x28 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32F767BITx",
  "parts": ["@st/stm32::MCU_STM32F767BIT6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32F767BIT6", "lcsc": "C129276"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32F765xx STM32F767xx STM32F768Ax STM32F769xx datasheet: ARM Cortex-M7 32b MCU+FPU, 462DMIPS, up to 2MB Flash/512+16+4KB RAM, USB OTG HS/FS, 28 com IF, LCD, DSI",
     "revision": "DocID029041 Rev 5, August 2017",
     "url": "https://www.st.com/resource/en/datasheet/stm32f767bi.pdf",
     "sha256": "3d2a3b76d5ba156ecb71683b9e9e8678ebbb8b0e98f74ff211d7313ff248cdc3",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/d2fff8b73e404f64a2a2c22a27ee2e71.pdf"}
  ],
  "status": "draft"
}
```

**Source.** The file was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C129276.pdf>) on 2026-11-07, because st.com's robots.txt was unavailable to the policy-bound client that day. It is ST-authored: the PDF's Author field is STMICROELECTRONICS and every page carries ST's DocID029041 Rev 5 footer. The PDF is not in the repository. Page numbers below are PDF page numbers; in this file they equal the printed folios (1/256 to 256/256).

## Summary

The STM32F767BIT6 is an Arm Cortex-M7 microcontroller with double-precision FPU and L1 cache, up to 216 MHz, 2048 KB flash and 512 KB system SRAM plus 16 KB instruction RAM and 4 KB backup SRAM, in a 28 x 28 mm LQFP208 (ds p. 1, ds p. 16, ds p. 116, ds p. 252). It has USB OTG FS and HS, Ethernet, FMC, Quad-SPI, LCD-TFT, JPEG codec, camera interface, 3 CAN, 3 ADCs and 2 DACs, but no MIPI-DSI, which is STM32F769 only (ds p. 16, ds p. 17). On the LQFP208 the regulator is always on (no BYPASS_REG pin); the internal reset can be turned off through PDR_ON (pad 203) for operation down to VDD = 1.7 V with an external supervisor (ds p. 30, ds p. 35). It has separate supply pins for the USB pins (VDDUSB) and the SDMMC2 pins (VDDSDMMC) (ds p. 27). Suffix 6 is rated −40 to 85 °C ambient at full dissipation, and 216 MHz applies to that range (ds p. 17, ds p. 117).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB clock fHCLK | 0 | | 216 | MHz | Scale 1, over-drive on, suffix 6 | ds p. 116 |
| AHB clock fHCLK, no over-drive | 0 | | 180 | MHz | Scale 1, over-drive off | ds p. 116 |
| APB1 / APB2 clock | 0 | | 54 / 108 | MHz | Over-drive on | ds p. 116 |
| Flash | | 2048 | | KB | Flash size code I | ds p. 252 |
| SRAM: system / instruction / backup | | 512 / 16 / 4 | | KB | | ds p. 16 |
| HSE crystal frequency | 4 | | 26 | MHz | fOSC_IN | ds p. 142 |
| HSE startup time | | 2 | | ms | VDD stabilized, standard crystal | ds p. 142 |
| HSE maximum critical crystal gm | | | 1 | mA/V | Startup | ds p. 142 |
| LSE maximum critical crystal gm | | | 0.48 | µA/V | LSEDRV = 00, low drive | ds p. 144 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 144 |
| POR/PDR threshold, falling | 1.60 | 1.68 | 1.76 | V | | ds p. 120 |
| POR/PDR threshold, rising | 1.64 | 1.72 | 1.80 | V | | ds p. 120 |
| POR reset temporization | 0.5 | 1.5 | 3.0 | ms | tRSTTEMPO | ds p. 120 |
| Regulator inrush current | | 160 | 250 | mA | POR or wakeup from Standby | ds p. 120 |
| NRST weak pull-up | 30 | 40 | 50 | kΩ | VIN = VSS | ds p. 168 |
| NRST filtered pulse | | | 100 | ns | VF(NRST) | ds p. 168 |
| NRST not-filtered pulse | 300 | | | ns | VNF(NRST), VDD > 2.7 V | ds p. 168 |
| BOOT VIL | | | 0.1VDD + 0.1 | V | 1.75 V ≤ VDD ≤ 3.6 V, −40 °C ≤ TA ≤ 105 °C | ds p. 162 |
| BOOT VIH | 0.17VDD + 0.7 | | | V | Same conditions | ds p. 162 |
| VCAP capacitor CEXT | | 2.2 | | µF | Per VCAP pin, ESR < 2 Ω | ds p. 119 |
| Thermal resistance θJA | | 19 | | °C/W | LQFP208 28 x 28 mm | ds p. 251 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD − VSS (includes VDDA, VBAT, VDDUSB, VDDSDMMC) | −0.3 to 4.0 | V | ds p. 114 |
| VIN on FT pins | VSS − 0.3 to VDD + 4.0 | V | ds p. 114 |
| VIN on TTa pins and any other pin | VSS − 0.3 to 4.0 | V | ds p. 114 |
| VIN on BOOT pin | VSS to 9.0 | V | ds p. 114 |
| Variation between VDD pins, between ground pins | 50 | mV | ds p. 114 |
| Total current into all VDD / out of all VSS | 420 / −420 | mA | ds p. 115 |
| Total current into VDDUSB / VDDSDMMC | 25 / 60 | mA | ds p. 115 |
| Current per VDD / VSS pin | 100 / −100 | mA | ds p. 115 |
| Current per I/O pin, sunk / sourced | 25 / −25 | mA | ds p. 115 |
| Total I/O current sunk / sourced (all I/Os except USB for source) | 120 / −120 | mA | ds p. 115 |
| Injected current, FT, FTf, RST and B pins | −5 / +0 | mA | ds p. 115 |
| Injected current, TTa pins | ±5 | mA | ds p. 115 |
| Total injected current | ±25 | mA | ds p. 115 |
| Storage temperature | −65 to +150 | °C | ds p. 115 |
| Maximum junction temperature | 125 | °C | ds p. 115 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, internal reset on (PDR_ON = VDD) | 1.8 | 3.6 | V | ds p. 29, ds p. 116 |
| VDD, internal reset off (PDR_ON = VSS, external supervisor) | 1.7 | 3.6 | V | ds p. 30, ds p. 116 |
| VDDA, ADC limited to 1.2 Msps (same potential as VDD) | 1.7 | 2.4 | V | ds p. 116 |
| VDDA, ADC up to 2.4 Msps (same potential as VDD) | 2.4 | 3.6 | V | ds p. 116 |
| VDD − VDDA during power-up and power-down | | 0.3 | V | ds p. 117 |
| VDDUSB, USB used | 3.0 | 3.6 | V | ds p. 116 |
| VDDUSB, USB not used | 1.7 | 3.6 | V | ds p. 116 |
| VDDSDMMC (may differ from VDD) | 1.7 | 3.6 | V | ds p. 116 |
| VBAT | 1.65 | 3.6 | V | ds p. 116 |
| V12 on VCAP_1/VCAP_2, Scale 1 (216 MHz with over-drive) | 1.26 | 1.40 | V | ds p. 117 |
| VIN, RST and FT pins, 2 V ≤ VDD ≤ 3.6 V | −0.3 | 5.5 | V | ds p. 117 |
| VIN, RST and FT pins, VDD ≤ 2 V | −0.3 | 5.2 | V | ds p. 117 |
| VIN, TTa pins | −0.3 | VDDA + 0.3 | V | ds p. 117 |
| VIN, BOOT pin | 0 | 9 | V | ds p. 117 |
| Power dissipation, LQFP208, TA = 85 °C (suffix 6) | | 1053 | mW | ds p. 117 |
| Ambient temperature, suffix 6, maximum dissipation | −40 | 85 | °C | ds p. 117 |
| Ambient temperature, suffix 6, low dissipation | −40 | 105 | °C | ds p. 117 |
| Junction temperature, suffix 6 | −40 | 105 | °C | ds p. 117 |
| VDD rise and fall time rate (regulator on) | 20 | ∞ | µs/V | ds p. 119 |

## Pins

Generated from the library's pins block and checked by script against Table 11, the STM32F765xx/F767xx LQFP208 column (ds pp. 64-88), and by eye against Figure 16, the STM32F76xxx LQFP208 pinout (ds p. 59). Every pad's signal matches. The table's second LQFP208 column and Figure 17 are for the STM32F769 (DSI) and do not apply. The Source column gives the Table 11 page of each row.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | PE2 | PE2 | I/O (FT) | GPIO | ds p. 64 |
| 2 | PE3 | PE3 | I/O (FT) | GPIO | ds p. 64 |
| 3 | PE4 | PE4 | I/O (FT) | GPIO | ds p. 65 |
| 4 | PE5 | PE5 | I/O (FT) | GPIO | ds p. 65 |
| 5 | PE6 | PE6 | I/O (FT) | GPIO | ds p. 65 |
| 6 | VBAT | VBAT | Power (S) | Backup-domain supply (RTC, LSE, backup registers) when VDD is absent | ds p. 65 |
| 7 | PI8 | PI8 | I/O (FT) | GPIO | ds p. 65 |
| 8 | PC13 | PC13 | I/O (FT) | GPIO | ds p. 65 |
| 9 | PC14 | PC14-OSC32_IN | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 65 |
| 10 | PC15 | PC15-OSC32_OUT | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 65 |
| 11 | PI9 | PI9 | I/O (FT) | GPIO | ds p. 66 |
| 12 | PI10 | PI10 | I/O (FT) | GPIO | ds p. 66 |
| 13 | PI11 | PI11 | I/O (FT) | GPIO | ds p. 66 |
| 14 | VSS | VSS | Power (S) | Ground | ds p. 66 |
| 15 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 66 |
| 16 | PF0 | PF0 | I/O (FT) | GPIO | ds p. 66 |
| 17 | PF1 | PF1 | I/O (FT) | GPIO | ds p. 66 |
| 18 | PF2 | PF2 | I/O (FT) | GPIO | ds p. 66 |
| 19 | PI12 | PI12 | I/O (FT) | GPIO | ds p. 66 |
| 20 | PI13 | PI13 | I/O (FT) | GPIO | ds p. 66 |
| 21 | PI14 | PI14 | I/O (FT) | GPIO | ds p. 66 |
| 22 | PF3 | PF3 | I/O (FT) | GPIO | ds p. 66 |
| 23 | PF4 | PF4 | I/O (FT) | GPIO | ds p. 66 |
| 24 | PF5 | PF5 | I/O (FT) | GPIO | ds p. 66 |
| 25 | VSS | VSS | Power (S) | Ground | ds p. 66 |
| 26 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 66 |
| 27 | PF6 | PF6 | I/O (FT) | GPIO | ds p. 66 |
| 28 | PF7 | PF7 | I/O (FT) | GPIO | ds p. 67 |
| 29 | PF8 | PF8 | I/O (FT) | GPIO | ds p. 67 |
| 30 | PF9 | PF9 | I/O (FT) | GPIO | ds p. 67 |
| 31 | PF10 | PF10 | I/O (FT) | GPIO | ds p. 67 |
| 32 | PH0 | PH0-OSC_IN | I/O (FT) | GPIO; OSC_IN (HSE) | ds p. 67 |
| 33 | PH1 | PH1-OSC_OUT | I/O (FT) | GPIO; OSC_OUT (HSE) | ds p. 67 |
| 34 | NRST | NRST | I/O (RST) | Reset input, active low; also driven low by internal resets | ds p. 67 |
| 35 | PC0 | PC0 | I/O (FT) | GPIO | ds p. 67 |
| 36 | PC1 | PC1 | I/O (FT) | GPIO | ds p. 67 |
| 37 | PC2 | PC2 | I/O (FT) | GPIO | ds p. 68 |
| 38 | PC3 | PC3 | I/O (FT) | GPIO | ds p. 68 |
| 39 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 68 |
| 40 | VSSA | VSSA | Power (S) | Analog ground | ds p. 68 |
| 41 | VREFP | VREF+ | Power (S) | ADC/DAC reference, positive | ds p. 68 |
| 42 | VDDA | VDDA | Power (S) | Analog supply (ADC, DAC, PLL, RCs) | ds p. 68 |
| 43 | PA0 | PA0-WKUP | I/O (FT) | GPIO; WKUP | ds p. 68 |
| 44 | PA1 | PA1 | I/O (FT) | GPIO | ds p. 68 |
| 45 | PA2 | PA2 | I/O (FT) | GPIO | ds p. 69 |
| 46 | PH2 | PH2 | I/O (FT) | GPIO | ds p. 69 |
| 47 | PH3 | PH3 | I/O (FT) | GPIO | ds p. 69 |
| 48 | PH4 | PH4 | I/O (FT) | GPIO | ds p. 69 |
| 49 | PH5 | PH5 | I/O (FT) | GPIO | ds p. 69 |
| 50 | PA3 | PA3 | I/O (FT) | GPIO | ds p. 69 |
| 51 | VSS | VSS | Power (S) | Ground | ds p. 69 |
| 52 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 69 |
| 53 | PA4 | PA4 | I/O (TTa) | GPIO | ds p. 70 |
| 54 | PA5 | PA5 | I/O (TTa) | GPIO | ds p. 70 |
| 55 | PA6 | PA6 | I/O (FT) | GPIO | ds p. 70 |
| 56 | PA7 | PA7 | I/O (FT) | GPIO | ds p. 70 |
| 57 | PC4 | PC4 | I/O (FT) | GPIO | ds p. 70 |
| 58 | PC5 | PC5 | I/O (FT) | GPIO | ds p. 70 |
| 59 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 70 |
| 60 | VSS | VSS | Power (S) | Ground | ds p. 71 |
| 61 | PB0 | PB0 | I/O (FT) | GPIO | ds p. 71 |
| 62 | PB1 | PB1 | I/O (FT) | GPIO | ds p. 71 |
| 63 | PB2 | PB2 | I/O (FT) | GPIO | ds p. 71 |
| 64 | PI15 | PI15 | I/O (FT) | GPIO | ds p. 71 |
| 65 | PJ0 | PJ0 | I/O (FT) | GPIO | ds p. 71 |
| 66 | PJ1 | PJ1 | I/O (FT) | GPIO | ds p. 71 |
| 67 | PJ2 | PJ2 | I/O (FT) | GPIO | ds p. 71 |
| 68 | PJ3 | PJ3 | I/O (FT) | GPIO | ds p. 71 |
| 69 | PJ4 | PJ4 | I/O (FT) | GPIO | ds p. 71 |
| 70 | PF11 | PF11 | I/O (FT) | GPIO | ds p. 71 |
| 71 | PF12 | PF12 | I/O (FT) | GPIO | ds p. 71 |
| 72 | VSS | VSS | Power (S) | Ground | ds p. 72 |
| 73 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 72 |
| 74 | PF13 | PF13 | I/O (FT) | GPIO | ds p. 72 |
| 75 | PF14 | PF14 | I/O (FT) | GPIO | ds p. 72 |
| 76 | PF15 | PF15 | I/O (FT) | GPIO | ds p. 72 |
| 77 | PG0 | PG0 | I/O (FT) | GPIO | ds p. 72 |
| 78 | PG1 | PG1 | I/O (FT) | GPIO | ds p. 72 |
| 79 | PE7 | PE7 | I/O (FT) | GPIO | ds p. 72 |
| 80 | PE8 | PE8 | I/O (FT) | GPIO | ds p. 72 |
| 81 | PE9 | PE9 | I/O (FT) | GPIO | ds p. 72 |
| 82 | VSS | VSS | Power (S) | Ground | ds p. 72 |
| 83 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 72 |
| 84 | PE10 | PE10 | I/O (FT) | GPIO | ds p. 73 |
| 85 | PE11 | PE11 | I/O (FT) | GPIO | ds p. 73 |
| 86 | PE12 | PE12 | I/O (FT) | GPIO | ds p. 73 |
| 87 | PE13 | PE13 | I/O (FT) | GPIO | ds p. 73 |
| 88 | PE14 | PE14 | I/O (FT) | GPIO | ds p. 73 |
| 89 | PE15 | PE15 | I/O (FT) | GPIO | ds p. 73 |
| 90 | PB10 | PB10 | I/O (FT) | GPIO | ds p. 73 |
| 91 | PB11 | PB11 | I/O (FT) | GPIO | ds p. 74 |
| 92 | VCAP_1 | VCAP_1 | Power (S) | Core regulator output; 2.2 µF to VSS | ds p. 74 |
| 93 | VSS | VSS | Power (S) | Ground | ds p. 74 |
| 94 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 74 |
| 95 | PJ5 | PJ5 | I/O (FT) | GPIO | ds p. 74 |
| 96 | PH6 | PH6 | I/O (FT) | GPIO | ds p. 74 |
| 97 | PH7 | PH7 | I/O (FT) | GPIO | ds p. 74 |
| 98 | PH8 | PH8 | I/O (FT) | GPIO | ds p. 74 |
| 99 | PH9 | PH9 | I/O (FT) | GPIO | ds p. 74 |
| 100 | PH10 | PH10 | I/O (FT) | GPIO | ds p. 74 |
| 101 | PH11 | PH11 | I/O (FT) | GPIO | ds p. 74 |
| 102 | PH12 | PH12 | I/O (FT) | GPIO | ds p. 75 |
| 103 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 75 |
| 104 | PB12 | PB12 | I/O (FT) | GPIO; OTG_HS_ID | ds p. 75 |
| 105 | PB13 | PB13 | I/O (FT) | GPIO | ds p. 75 |
| 106 | PB14 | PB14 | I/O (FT) | GPIO; OTG_HS_DM | ds p. 75 |
| 107 | PB15 | PB15 | I/O (FT) | GPIO; OTG_HS_DP | ds p. 75 |
| 108 | PD8 | PD8 | I/O (FT) | GPIO | ds p. 76 |
| 109 | PD9 | PD9 | I/O (FT) | GPIO | ds p. 76 |
| 110 | PD10 | PD10 | I/O (FT) | GPIO | ds p. 76 |
| 111 | PD11 | PD11 | I/O (FT) | GPIO | ds p. 76 |
| 112 | PD12 | PD12 | I/O (FT) | GPIO | ds p. 76 |
| 113 | PD13 | PD13 | I/O (FT) | GPIO | ds p. 76 |
| 114 | VSS | VSS | Power (S) | Ground | ds p. 76 |
| 115 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 76 |
| 116 | PD14 | PD14 | I/O (FT) | GPIO | ds p. 76 |
| 117 | PD15 | PD15 | I/O (FT) | GPIO | ds p. 76 |
| 118 | PJ6 | PJ6 | I/O (FT) | GPIO | ds p. 76 |
| 119 | PJ7 | PJ7 | I/O (FT) | GPIO | ds p. 76 |
| 120 | PJ8 | PJ8 | I/O (FT) | GPIO | ds p. 77 |
| 121 | PJ9 | PJ9 | I/O (FT) | GPIO | ds p. 77 |
| 122 | PJ10 | PJ10 | I/O (FT) | GPIO | ds p. 77 |
| 123 | PJ11 | PJ11 | I/O (FT) | GPIO | ds p. 77 |
| 124 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 77 |
| 125 | VSS | VSS | Power (S) | Ground | ds p. 77 |
| 126 | PK0 | PK0 | I/O (FT) | GPIO | ds p. 77 |
| 127 | PK1 | PK1 | I/O (FT) | GPIO | ds p. 77 |
| 128 | PK2 | PK2 | I/O (FT) | GPIO | ds p. 77 |
| 129 | PG2 | PG2 | I/O (FT) | GPIO | ds p. 77 |
| 130 | PG3 | PG3 | I/O (FT) | GPIO | ds p. 77 |
| 131 | PG4 | PG4 | I/O (FT) | GPIO | ds p. 78 |
| 132 | PG5 | PG5 | I/O (FT) | GPIO | ds p. 78 |
| 133 | PG6 | PG6 | I/O (FT) | GPIO | ds p. 78 |
| 134 | PG7 | PG7 | I/O (FT) | GPIO | ds p. 78 |
| 135 | PG8 | PG8 | I/O (FT) | GPIO | ds p. 78 |
| 136 | VSS | VSS | Power (S) | Ground | ds p. 78 |
| 137 | VDDUSB | VDDUSB | Power (S) | Supply of PA11, PA12, PB14, PB15 and the USB PHYs | ds p. 78 |
| 138 | PC6 | PC6 | I/O (FT) | GPIO | ds p. 78 |
| 139 | PC7 | PC7 | I/O (FT) | GPIO | ds p. 78 |
| 140 | PC8 | PC8 | I/O (FT) | GPIO | ds p. 79 |
| 141 | PC9 | PC9 | I/O (FT) | GPIO | ds p. 79 |
| 142 | PA8 | PA8 | I/O (FT) | GPIO | ds p. 79 |
| 143 | PA9 | PA9 | I/O (FT) | GPIO | ds p. 79 |
| 144 | PA10 | PA10 | I/O (FT) | GPIO; OTG_FS_ID | ds p. 79 |
| 145 | PA11 | PA11 | I/O (FT) | GPIO; OTG_FS_DM | ds p. 79 |
| 146 | PA12 | PA12 | I/O (FT) | GPIO; OTG_FS_DP | ds p. 80 |
| 147 | PA13 | PA13(JTMS-SWDIO) | I/O (FT) | JTMS-SWDIO after reset | ds p. 80 |
| 148 | VCAP_2 | VCAP_2 | Power (S) | Core regulator output; 2.2 µF to VSS | ds p. 80 |
| 149 | VSS | VSS | Power (S) | Ground | ds p. 80 |
| 150 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 80 |
| 151 | PH13 | PH13 | I/O (FT) | GPIO | ds p. 80 |
| 152 | PH14 | PH14 | I/O (FT) | GPIO | ds p. 80 |
| 153 | PH15 | PH15 | I/O (FT) | GPIO | ds p. 80 |
| 154 | PI0 | PI0 | I/O (FT) | GPIO | ds p. 80 |
| 155 | PI1 | PI1 | I/O (FT) | GPIO | ds p. 80 |
| 156 | PI2 | PI2 | I/O (FT) | GPIO | ds p. 80 |
| 157 | PI3 | PI3 | I/O (FT) | GPIO | ds p. 81 |
| 158 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 81 |
| 159 | PA14 | PA14(JTCK-SWCLK) | I/O (FT) | JTCK-SWCLK after reset | ds p. 81 |
| 160 | PA15 | PA15(JTDI) | I/O (FT) | JTDI after reset | ds p. 81 |
| 161 | PC10 | PC10 | I/O (FT) | GPIO | ds p. 81 |
| 162 | PC11 | PC11 | I/O (FT) | GPIO | ds p. 81 |
| 163 | PC12 | PC12 | I/O (FT) | GPIO | ds p. 81 |
| 164 | PD0 | PD0 | I/O (FT) | GPIO | ds p. 82 |
| 165 | PD1 | PD1 | I/O (FT) | GPIO | ds p. 82 |
| 166 | PD2 | PD2 | I/O (FT) | GPIO | ds p. 82 |
| 167 | PD3 | PD3 | I/O (FT) | GPIO | ds p. 82 |
| 168 | PD4 | PD4 | I/O (FT) | GPIO | ds p. 82 |
| 169 | PD5 | PD5 | I/O (FT) | GPIO | ds p. 82 |
| 170 | VSS | VSS | Power (S) | Ground | ds p. 82 |
| 171 | VDDSDMMC | VDDSDMMC | Power (S) | Supply of the SDMMC2 pins PG9-PG12, PD6, PD7 | ds p. 82 |
| 172 | PD6 | PD6 | I/O (FT) | GPIO | ds p. 82 |
| 173 | PD7 | PD7 | I/O (FT) | GPIO | ds p. 83 |
| 174 | PJ12 | PJ12 | I/O (FT) | GPIO | ds p. 83 |
| 175 | PJ13 | PJ13 | I/O (FT) | GPIO | ds p. 83 |
| 176 | PJ14 | PJ14 | I/O (FT) | GPIO | ds p. 83 |
| 177 | PJ15 | PJ15 | I/O (FT) | GPIO | ds p. 83 |
| 178 | PG9 | PG9 | I/O (FT) | GPIO | ds p. 83 |
| 179 | PG10 | PG10 | I/O (FT) | GPIO | ds p. 83 |
| 180 | PG11 | PG11 | I/O (FT) | GPIO | ds p. 83 |
| 181 | PG12 | PG12 | I/O (FT) | GPIO | ds p. 83 |
| 182 | PG13 | PG13 | I/O (FT) | GPIO | ds p. 84 |
| 183 | PG14 | PG14 | I/O (FT) | GPIO | ds p. 84 |
| 184 | VSS | VSS | Power (S) | Ground | ds p. 84 |
| 185 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 84 |
| 186 | PK3 | PK3 | I/O (FT) | GPIO | ds p. 84 |
| 187 | PK4 | PK4 | I/O (FT) | GPIO | ds p. 84 |
| 188 | PK5 | PK5 | I/O (FT) | GPIO | ds p. 84 |
| 189 | PK6 | PK6 | I/O (FT) | GPIO | ds p. 84 |
| 190 | PK7 | PK7 | I/O (FT) | GPIO | ds p. 84 |
| 191 | PG15 | PG15 | I/O (FT) | GPIO | ds p. 84 |
| 192 | PB3 | PB3(JTDO/TRACESWO) | I/O (FT) | JTDO/TRACESWO after reset | ds p. 84 |
| 193 | PB4 | PB4(NJTRST) | I/O (FT) | NJTRST after reset | ds p. 84 |
| 194 | PB5 | PB5 | I/O (FT) | GPIO | ds p. 85 |
| 195 | PB6 | PB6 | I/O (FT) | GPIO | ds p. 85 |
| 196 | PB7 | PB7 | I/O (FT) | GPIO | ds p. 85 |
| 197 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 85 |
| 198 | PB8 | PB8 | I/O (FT) | GPIO | ds p. 85 |
| 199 | PB9 | PB9 | I/O (FT) | GPIO | ds p. 86 |
| 200 | PE0 | PE0 | I/O (FT) | GPIO | ds p. 86 |
| 201 | PE1 | PE1 | I/O (FT) | GPIO | ds p. 86 |
| 202 | VSS | VSS | Power (S) | Ground | ds p. 86 |
| 203 | PDR_ON | PDR_ON | Power (S) | Internal POR/PDR enable: VDD = on, VSS = off with an external supervisor | ds p. 86 |
| 204 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 86 |
| 205 | PI4 | PI4 | I/O (FT) | GPIO | ds p. 86 |
| 206 | PI5 | PI5 | I/O (FT) | GPIO | ds p. 86 |
| 207 | PI6 | PI6 | I/O (FT) | GPIO | ds p. 86 |
| 208 | PI7 | PI7 | I/O (FT) | GPIO | ds p. 86 |

## Typical application

- Decoupling, from Figure 25, the STM32F767xx power supply scheme (ds p. 113):
  - VDD: 100 nF per VDD pin plus one 4.7 µF ceramic on one of the VDD pins. The figure is drawn for 20 VDD pins; the LQFP208 has 15 (pads 15, 26, 39, 52, 59, 73, 83, 94, 103, 115, 124, 150, 158, 185, 204).
  - VDDA: 100 nF + 1 µF. VREF+: 100 nF + 1 µF.
  - VDDUSB (pad 137): 100 nF + 1 µF. VDDSDMMC (pad 171): 100 nF + 1 µF.
  - VCAP_1 (pad 92) and VCAP_2 (pad 148): one 2.2 µF ceramic each, ESR below 2 Ω (ds p. 119).
  - Place every capacitor as close as possible to its pins, or under them on the PCB underside (ds p. 114).
- Connect every VDD, VDDA, VDDSDMMC, VDDUSB, VSS and VSSA pin to its supply (ds p. 115). VDDA = VDD and VSSA = VSS (ds p. 27, ds p. 113).
- VDDUSB and VDDSDMMC may be tied to VDD or fed separately. A separate supply must come up last and go down first, staying below VDD while VDD is below its minimum (ds p. 27, ds p. 28).
- VREF+ (pad 41) must be supplied; keep VDDA − VREF+ below 1.2 V (ds p. 117).
- PDR_ON (pad 203): tie to VDD for the internal POR/PDR. Tie to VSS only with an external supervisor that holds NRST low below 1.7 V; VBAT then must be tied to VDD (ds p. 29, ds p. 30, ds p. 35).
- NRST (pad 34): internal 30-50 kΩ pull-up. Figure 40 shows a 0.1 µF capacitor to ground, placed as close as possible to the chip (ds p. 168).
- Boot: the BOOT pin (pad 197) together with the BOOT_ADDx option bytes selects the boot address; the bootloader is described in AN2606 (ds p. 27).
- HSE on PH0/PH1 (pads 32, 33): 4-26 MHz crystal with CL1 = CL2 of 5-25 pF, sized with about 10 pF of pin and board capacitance (ds p. 142, ds p. 143).
- LSE on PC14/PC15 (pads 9, 10): 32.768 kHz crystal with its load capacitors close to the pins (ds p. 143).
- USB: OTG_FS on PA11/PA12 (pads 145, 146), OTG_HS FS PHY on PB14/PB15 (pads 106, 107); VDDUSB = 3.0-3.6 V when USB is used (ds p. 75, ds p. 79, ds p. 80, ds p. 116).
- SWD: PA13 = JTMS-SWDIO (pad 147), PA14 = JTCK-SWCLK (pad 159) after reset (ds p. 80, ds p. 81).

## Layout notes

- Land: ST's recommended LQFP208 footprint, Figure 93, has 0.5 mm pitch, 0.3 mm pad width, 1.2 mm pad length, 30.7 mm outer span, 28.3 mm inner span and 25.8 mm across one row of 52 pads (ds p. 239).
- The library's land `qfp::KICAD_LQFP_208_28X28MM_P0_5MM` has 0.5 mm pitch and 0.3 mm width, but 1.5 mm pads centred at ±14.675 mm. That is a 30.85 mm outer and 27.85 mm inner span. See "Not established by the sources".
- Package: body D1 = E1 = 28.00 mm nominal, D = E = 30.00 mm across the leads, b = 0.17-0.27 mm, L = 0.45-0.75 mm, A ≤ 1.600 mm (ds p. 238).
- Pin 1: Figure 92 marks pin 1 at a corner, with pins numbered counter-clockwise in top view (ds p. 237). Figure 93 puts pad 1 at the top of the left row (ds p. 239), as the library land does, and both number counter-clockwise. The supplier binding's `rotate 270` turns this land onto the assembler's frame; it was read from the assembler's preview and is not reopened here.
- No exposed pad. θJA is 19 °C/W (ds p. 251).

## Pitfalls

- PA4 and PA5 are TTa, not 5 V tolerant: VIN ≤ VDDA + 0.3 V (ds p. 70, ds p. 117). FT pins are 5 V tolerant except in analog mode, and except PC14, PC15, PH0 and PH1 in oscillator mode (ds p. 88).
- An FT or RST pin above VDD + 0.3 V needs its internal pull-up and pull-down disabled (ds p. 163).
- PC13-PC15 and PI8 run from the backup power switch: in output mode at most 2 MHz with 30 pF, and never as a current source (ds p. 88).
- PA11, PA12, PB14 and PB15 are powered from VDDUSB, and PG9-PG12, PD6 and PD7 from VDDSDMMC, not from VDD (ds p. 113, ds p. 115, ds p. 116). An unpowered VDDUSB disables those GPIOs too.
- 216 MHz needs over-drive, which is not available at VDD = 1.7-2.1 V (ds p. 117).
- With PDR_ON = VSS, POR/PDR, BOR and PVD are off and VBAT is no longer available (ds p. 30).
- An external reset circuit must pull NRST below VIL(NRST) max, or the reset is ignored (ds p. 168).
- The library's pin names PC14, PC15, PH0, PH1 and PA0 are the datasheet's PC14-OSC32_IN, PC15-OSC32_OUT, PH0-OSC_IN, PH1-OSC_OUT and PA0-WKUP; VREFP is VREF+ (ds p. 65, ds p. 67, ds p. 68).

## Not established by the sources

- **Land pattern differs from ST's recommendation (minor).** `qfp::KICAD_LQFP_208_28X28MM_P0_5MM` uses 1.5 mm pads at ±14.675 mm (30.85 mm outer, 27.85 mm inner span). ST's Figure 93 gives 1.2 mm pads with a 30.7 mm outer and 28.3 mm inner span (ds p. 239). The library pads reach 0.075 mm further out and 0.225 mm further in. Pitch, pad width and pin-1 position agree. Figure 93 also prints a "1.25" dimension beside the top row that does not fit the 1.2 mm pad length; its meaning is unclear.
- **No pad-signal disagreement.** All 208 pads match Table 11 (ds pp. 64-88) and Figure 16 (ds p. 59).
- **PDR_ON is `optional` in the library.** The datasheet requires it at VDD (internal reset on) or at VSS with an external supervisor (ds p. 29, ds p. 30, ds p. 35). It gives no internal pull and no floating state. Table 11 lists it as pin type S (supply) (ds p. 86); the library types it `input`.
- **NRST is typed `input`.** Table 11 gives it as I/O, RST type, driven low by internal resets (ds p. 67). This is the library's known lack of an open-drain type.
- **The datasheet contradicts itself on the GPIO count.** Table 2 gives 159 GPIOs for the STM32F767/769Bx column (ds p. 17), but Figure 16 and Table 11 give the F767 LQFP208 168 GPIO pads (ds p. 59), which is what the library has. The 159 probably reflects the F769's DSI pins; the datasheet does not say so.
- **The datasheet gives two VDDSDMMC minimums**: 1.8 V in Section 2.17 (ds p. 27) and 1.7 V in Table 18 (ds p. 116). The SDMMC2 pin list also differs: "PG[12:9] and PD6" in Table 18 (ds p. 116), PD7 as well in Table 16 and Figure 25 (ds p. 113, ds p. 115).
- The boot-address truth table and bootloader pins are in AN2606 and the reference manual, not here (ds p. 27).
- No BOOT pull-down value, VBAT capacitor, or rule for VBAT without a battery (other than tying it to VDD when PDR_ON = VSS) is given.
- No crystal part, ESR limit or load-capacitor value beyond the 5-25 pF range is given (ds p. 143).
