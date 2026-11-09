# STM32H743VIH6 — Arm Cortex-M7 MCU, up to 480 MHz, 2 MB flash, 1 MB RAM, TFBGA100 8x8 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32H743VIHx",
  "parts": ["@st/stm32::MCU_STM32H743VIH6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32H743VIH6", "lcsc": "C964146"},
    {"mfr": "STMicroelectronics", "mpn": "STM32H743VIH6TR", "lcsc": "C1857444"}
  ],
  "sources": [
    {"id": "ds", "title": "STM32H742xI/G STM32H743xI/G datasheet: 32-bit Arm Cortex-M7 480MHz MCUs, up to 2MB Flash, up to 1MB RAM, 46 com. and analog interfaces",
     "revision": "DS12110 Rev 8, April 2021",
     "url": "https://www.st.com/resource/en/datasheet/stm32h743vi.pdf",
     "sha256": "1aaff172abf59e6d78504e735a428865f636b22f3af0d4871e30ff4ba7129b13",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/a7aa1bba41ecd40a0ffdd37ccbadb76c.pdf"}
  ],
  "status": "draft"
}
```

**Source.** The file was retrieved from LCSC (part page <https://www.lcsc.com/datasheet/C1857444.pdf>) on 2026-11-07, because st.com's robots.txt was unavailable to the policy-bound client that day. It is ST-authored: the PDF's Author field is STMICROELECTRONICS and every page carries ST's DS12110 Rev 8 footer. The PDF is not in the repository.

**Page numbering.** Citations are PDF page numbers. They equal the printed folios up to p. 321. From PDF p. 322 on, the file has one extra page: PDF page = printed folio + 1. Two consecutive pages, PDF pp. 321-322, both print "321/357". So the TFBGA100 package pages, printed 324-325, are PDF pp. 325-326; the thermal table, printed 348-349, is PDF pp. 349-350; and the ordering information, printed 351, is PDF p. 352.

**Two silicon revisions.** The datasheet has two electrical chapters: Section 6 for silicon revision Y (PDF pp. 104 ff.) and Section 7 for revision V (PDF pp. 210 ff.). 480 MHz is reached only on revision V (ds p. 22). This note quotes Section 7 (rev V) unless it says otherwise. Which revision a given reel carries is not established.

## Summary

The STM32H743VIH6 is an Arm Cortex-M7 microcontroller with double-precision FPU and 16 KB I/D caches, up to 480 MHz, 2 MB flash (2 banks of 1 MB) and about 1 MB of RAM, in an 8 x 8 mm, 0.8 mm-pitch TFBGA100 (ds p. 1, ds p. 19, ds p. 215, ds p. 325). The RAM is 512 KB AXI SRAM, 128 + 128 + 32 KB SRAM1-3, 64 KB SRAM4, 64 KB ITCM, 128 KB DTCM and 4 KB backup SRAM (ds p. 19). It has USB OTG FS and HS, Ethernet, FMC, Quad-SPI, LCD-TFT, 3 ADCs, 2 DACs and an embedded LDO; it has no SMPS (ds p. 1, ds p. 27). The 100-pin parts have 82 GPIOs (ds p. 19). On this package VREF+ is not a separate ball: VDDA doubles as the ADC/DAC reference, and the internal reference buffer is unavailable (ds p. 88). **ST's Rev 8 marks the TFBGA100 as "under development"** (ds p. 21, ds p. 22). Suffix 6 is rated −40 to 85 °C ambient at full dissipation (ds p. 215).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU clock fCPU | | | 480 | MHz | VOS0, LDO, TJ ≤ 105 °C, silicon rev V | ds p. 215 |
| CPU clock fCPU | | | 400 | MHz | VOS1 | ds p. 215 |
| AHB / APB clock | | | 240 / 120 | MHz | VOS0 | ds p. 215 |
| Flash | | 2 x 1 | | MB | STM32H743xI | ds p. 19 |
| GPIO count | | 82 | | | 100-pin packages | ds p. 19 |
| HSE crystal frequency | 4 | | 48 | MHz | | ds p. 234 |
| HSE startup time | | 2 | | ms | VDD stabilized | ds p. 234 |
| HSE maximum critical crystal gm | | | 1.5 | mA/V | Startup | ds p. 234 |
| LSE maximum critical crystal gm | | | 0.5 to 2.7 | µA/V | LSEDRV = 00 to 11 | ds p. 235 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 235 |
| BOR0 (power-on) threshold, rising | 1.62 | 1.67 | 1.71 | V | | ds p. 217 |
| BOR0 threshold, falling | 1.58 | 1.62 | 1.68 | V | | ds p. 217 |
| Reset temporization after BOR0 | | 377 | | µs | tRSTTEMPO | ds p. 217 |
| NRST weak pull-up | 30 | 40 | 50 | kΩ | VIN = VSS | ds p. 253 |
| NRST filtered pulse | | | 50 | ns | 1.71 V < VDD < 3.6 V | ds p. 253 |
| NRST not-filtered pulse | 300 | | | ns | 1.71 V < VDD < 3.6 V | ds p. 253 |
| BOOT0 VIL | | | 0.19VDD + 0.1 | V | 1.62 V < VDDIOx < 3.6 V | ds p. 246 |
| BOOT0 VIH | 0.17VDD + 0.6 | | | V | 1.62 V < VDDIOx < 3.6 V | ds p. 246 |
| I/O VIL / VIH (CMOS) | | | 0.3VDD / 0.7VDD (min) | V | All I/Os except BOOT0 | ds p. 246 |
| GPIO drive | | | ±8 (±20 relaxed VOL/VOH) | mA | | ds p. 247 |
| VCAP capacitor CEXT | | 2.2 | | µF | Per VCAP ball, ±20 %, ESR < 100 mΩ | ds p. 216 |
| Thermal resistance θJA / θJB / θJC | | 39.3 / 21.1 / 17.1 | | °C/W | TFBGA100 | ds p. 349, ds p. 350 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDDX − VSS (VDD, VDDLDO, VDDA, VDD33USB, VBAT) | −0.3 to 4.0 | V | ds p. 212 |
| VIN on FT_xxx pins | VSS − 0.3 to Min(VDD, VDDA, VDD33USB, VBAT) + 4.0 | V | ds p. 212 |
| VIN on TT_xx pins and any other pin | VSS − 0.3 to 4.0 | V | ds p. 212 |
| VIN on BOOT0 | VSS to 9.0 | V | ds p. 212 |
| Variation between VDDX pins of one domain, between ground pins | 50 | mV | ds p. 212 |
| Total current into all VDD / out of all VSS | 620 / 620 | mA | ds p. 213 |
| Current per VDD / VSS pin | 100 / 100 | mA | ds p. 213 |
| Current sunk by any I/O pin | 20 | mA | ds p. 213 |
| Total I/O current, sunk / sourced | 140 / 140 | mA | ds p. 213 |
| Injected current, FT_xxx, TT_xx, RST and B pins except PA4, PA5 | −5 / +0 | mA | ds p. 213 |
| Injected current, PA4, PA5 | −0 / 0 | mA | ds p. 213 |
| Total injected current | ±25 | mA | ds p. 213 |
| Storage temperature | −65 to +150 | °C | ds p. 213 |
| Maximum junction temperature | 125 | °C | ds p. 213 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, internal supervisor on | 1.71 | 3.6 | V | ds p. 22 |
| VDD, PDR_ON = VSS with an external supervisor | 1.62 | 3.6 | V | ds p. 22, ds p. 214 |
| VDDLDO (≤ VDD) | 1.62 | 3.6 | V | ds p. 214 |
| VDDLDO for VOS0 (480 MHz) | 1.7 | 3.6 | V | ds p. 215 |
| VDD33USB, USB used | 3.0 | 3.6 | V | ds p. 214 |
| VDD33USB, USB not used | 0 | 3.6 | V | ds p. 214 |
| VDDA, ADC or COMP used | 1.62 | 3.6 | V | ds p. 214 |
| VDDA, DAC used | 1.8 | 3.6 | V | ds p. 214 |
| VDDA, OPAMP used | 2.0 | 3.6 | V | ds p. 214 |
| VDDA, no analog peripheral used | 0 | 3.6 | V | ds p. 214 |
| VBAT | 1.2 | 3.6 | V | ds p. 27 |
| VCORE (VCAP) with LDO, VOS0 | 1.26 | 1.40 | V | ds p. 214 |
| VIN, TT_xx pins | −0.3 | VDD + 0.3 | V | ds p. 214 |
| VIN, all I/O except BOOT0 and TT_xx | −0.3 | Min(VDD, VDDA, VDD33USB) + 3.6, < 5.5 | V | ds p. 214 |
| VIN, BOOT0 | 0 | 9 | V | ds p. 214 |
| Ambient temperature, suffix 6, maximum dissipation | −40 | 85 | °C | ds p. 215 |
| Ambient temperature, suffix 6, low dissipation | −40 | 105 | °C | ds p. 215 |
| Junction temperature, suffix 6 (105 °C max at VOS0) | −40 | 125 | °C | ds p. 215 |
| VDD, VDDA and VDDUSB fall time rate | 10 | ∞ | µs/V | ds p. 216 |

## Pins

Generated from the library's pins block and checked by script against Table 9, the TFBGA100 column (ds pp. 64-88), and by a second script against Figure 6, the TFBGA100 ballout (ds p. 56). Every ball's signal matches. The only naming difference is the library's `VDD33_USB` for the datasheet's VDD33USB. The Source column gives the Table 9 page of each row. I/O structure suffixes: _h high-speed low-voltage, _a analog option (supplied by VDDA), _u USB option (supplied by VDD33USB), _f I2C FM+ (ds p. 63).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | PC14 | PC14-OSC32_IN(OSC32_IN) | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 65 |
| A2 | PC13 | PC13 | I/O (FT) | GPIO | ds p. 64 |
| A3 | PE2 | PE2 | I/O (FT_h) | GPIO | ds p. 64 |
| A4 | PB9 | PB9 | I/O (FT_fh) | GPIO | ds p. 87 |
| A5 | PB7 | PB7 | I/O (FT_fa) | GPIO | ds p. 86 |
| A6 | PB4 | PB4(NJTRST) | I/O (FT) | NJTRST after reset | ds p. 85 |
| A7 | PB3 | PB3(JTDO/TRACESWO) | I/O (FT) | JTDO/TRACESWO after reset | ds p. 85 |
| A8 | PA15 | PA15(JTDI) | I/O (FT) | JTDI after reset | ds p. 82 |
| A9 | PA14 | PA14(JTCK/SWCLK) | I/O (FT) | JTCK-SWCLK after reset | ds p. 81 |
| A10 | PA13 | PA13(JTMS/SWDIO) | I/O (FT) | JTMS-SWDIO after reset | ds p. 80 |
| B1 | PC15 | PC15-OSC32_OUT(OSC32_OUT) | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 65 |
| B2 | VBAT | VBAT | Power (S) | Backup-domain supply (RTC, LSE, backup registers) when VDD is absent | ds p. 64 |
| B3 | PE3 | PE3 | I/O (FT_h) | GPIO | ds p. 64 |
| B4 | PB8 | PB8 | I/O (FT_fh) | GPIO | ds p. 86 |
| B5 | PB6 | PB6 | I/O (FT_f) | GPIO | ds p. 86 |
| B6 | PD5 | PD5 | I/O (FT_h) | GPIO | ds p. 83 |
| B7 | PD2 | PD2 | I/O (FT_h) | GPIO | ds p. 83 |
| B8 | PC11 | PC11 | I/O (FT_h) | GPIO | ds p. 82 |
| B9 | PC10 | PC10 | I/O (FT_ha) | GPIO | ds p. 82 |
| B10 | PA12 | PA12 | I/O (FT_u) | GPIO; OTG_FS_DP | ds p. 80 |
| C1 | PH0 | PH0-OSC_IN(PH0) | I/O (FT) | GPIO; OSC_IN (HSE) | ds p. 66 |
| C2 | VSS | VSS | Power (S) | Ground | ds p. 65 |
| C3 | PE4 | PE4 | I/O (FT_h) | GPIO | ds p. 64 |
| C4 | PE1 | PE1 | I/O (FT_h) | GPIO | ds p. 87 |
| C5 | PB5 | PB5 | I/O (FT) | GPIO | ds p. 86 |
| C6 | PD6 | PD6 | I/O (FT_h) | GPIO | ds p. 83 |
| C7 | PD3 | PD3 | I/O (FT_h) | GPIO | ds p. 83 |
| C8 | PC12 | PC12 | I/O (FT_h) | GPIO | ds p. 82 |
| C9 | PA9 | PA9 | I/O (FT_u) | GPIO; OTG_FS_VBUS | ds p. 80 |
| C10 | PA11 | PA11 | I/O (FT_u) | GPIO; OTG_FS_DM | ds p. 80 |
| D1 | PH1 | PH1-OSC_OUT(PH1) | I/O (FT) | GPIO; OSC_OUT (HSE) | ds p. 66 |
| D2 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 65 |
| D3 | PE5 | PE5 | I/O (FT_h) | GPIO | ds p. 64 |
| D4 | PE0 | PE0 | I/O (FT_h) | GPIO | ds p. 87 |
| D5 | BOOT0 | BOOT0 | Input (B) | Boot memory selection | ds p. 86 |
| D6 | PD7 | PD7 | I/O (FT_h) | GPIO | ds p. 83 |
| D7 | PD4 | PD4 | I/O (FT_h) | GPIO | ds p. 83 |
| D8 | PD0 | PD0 | I/O (FT_h) | GPIO | ds p. 82 |
| D9 | PA8 | PA8 | I/O (FT_fha) | GPIO | ds p. 79 |
| D10 | PA10 | PA10 | I/O (FT_u) | GPIO; OTG_FS_ID | ds p. 80 |
| E1 | NRST | NRST | I/O (RST) | Reset input, active low; also driven low by internal resets | ds p. 66 |
| E2 | PC2_C | PC2_C | Analog (TT_a) | Analog pad of PC2 (ADC input; PC2 functions through the analog switch) | ds p. 67 |
| E3 | PE6 | PE6 | I/O (FT_h) | GPIO | ds p. 64 |
| E4 | VSS | VSS | Power (S) | Ground | ds p. 73 |
| E5 | VSS | VSS | Power (S) | Ground | ds p. 80 |
| E6 | VSS | VSS | Power (S) | Ground | ds p. 67 |
| E7 | VCAP | VCAP | Power (S) | Core regulator output (VCORE); 2.2 µF to VSS | ds p. 80 |
| E8 | PD1 | PD1 | I/O (FT_h) | GPIO | ds p. 82 |
| E9 | PC9 | PC9 | I/O (FT_fh) | GPIO | ds p. 79 |
| E10 | PC7 | PC7 | I/O (FT_h) | GPIO | ds p. 79 |
| F1 | PC0 | PC0 | I/O (FT_a) | GPIO | ds p. 67 |
| F2 | PC1 | PC1 | I/O (FT_ha) | GPIO | ds p. 67 |
| F3 | PC3_C | PC3_C | Analog (TT_a) | Analog pad of PC3 (ADC input; PC3 functions through the analog switch) | ds p. 67 |
| F4 | VDDLDO | VDDLDO | Power (S) | Supply of the core LDO regulator | ds p. 87 |
| F5 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 67 |
| F6 | VDD33_USB | VDD33USB | Power (S) | USB PHY supply (3.0-3.6 V when USB is used) | ds p. 78 |
| F7 | PDR_ON | PDR_ON | Input (FT) | Internal POR/PDR enable: VDD = on, VSS = off with an external supervisor | ds p. 87 |
| F8 | VCAP | VCAP | Power (S) | Core regulator output (VCORE); 2.2 µF to VSS | ds p. 73 |
| F9 | PC8 | PC8 | I/O (FT_h) | GPIO | ds p. 79 |
| F10 | PC6 | PC6 | I/O (FT_h) | GPIO | ds p. 78 |
| G1 | VSSA | VSSA | Power (S) | Analog ground | ds p. 67 |
| G2 | PA0 | PA0 | I/O (FT_a) | GPIO; WKUP | ds p. 68 |
| G3 | PA4 | PA4 | I/O (TT_a) | GPIO | ds p. 69 |
| G4 | PC4 | PC4 | I/O (TT_a) | GPIO | ds p. 70 |
| G5 | PB2 | PB2 | I/O (FT_ha) | GPIO | ds p. 70 |
| G6 | PE10 | PE10 | I/O (FT_ha) | GPIO | ds p. 72 |
| G7 | PE14 | PE14 | I/O (FT_h) | GPIO | ds p. 72 |
| G8 | PD15 | PD15 | I/O (FT_h) | GPIO | ds p. 76 |
| G9 | PD11 | PD11 | I/O (FT_h) | GPIO | ds p. 76 |
| G10 | PB15 | PB15 | I/O (FT_u) | GPIO; OTG_HS_DP | ds p. 75 |
| H1 | VDDA | VDDA | Power (S) | Analog supply; also VREF+ on this package (double-bonded) | ds p. 67 |
| H2 | PA1 | PA1 | I/O (FT_ha) | GPIO | ds p. 68 |
| H3 | PA5 | PA5 | I/O (TT_ha) | GPIO | ds p. 69 |
| H4 | PC5 | PC5 | I/O (TT_a) | GPIO | ds p. 70 |
| H5 | PE7 | PE7 | I/O (TT_ha) | GPIO | ds p. 71 |
| H6 | PE11 | PE11 | I/O (FT_ha) | GPIO | ds p. 72 |
| H7 | PE15 | PE15 | I/O (FT_h) | GPIO | ds p. 72 |
| H8 | PD14 | PD14 | I/O (FT_h) | GPIO | ds p. 76 |
| H9 | PD10 | PD10 | I/O (FT_h) | GPIO | ds p. 75 |
| H10 | PB14 | PB14 | I/O (FT_u) | GPIO; OTG_HS_DM | ds p. 75 |
| J1 | VSS | VSS | Power (S) | Ground | ds p. 68 |
| J2 | PA2 | PA2 | I/O (FT_a) | GPIO | ds p. 68 |
| J3 | PA6 | PA6 | I/O (FT_a) | GPIO | ds p. 69 |
| J4 | PB0 | PB0 | I/O (FT_a) | GPIO | ds p. 70 |
| J5 | PE8 | PE8 | I/O (TT_ha) | GPIO | ds p. 71 |
| J6 | PE12 | PE12 | I/O (FT_h) | GPIO | ds p. 72 |
| J7 | PB10 | PB10 | I/O (FT_f) | GPIO | ds p. 73 |
| J8 | PB13 | PB13 | I/O (FT_u) | GPIO | ds p. 74 |
| J9 | PD9 | PD9 | I/O (FT_h) | GPIO | ds p. 75 |
| J10 | PD13 | PD13 | I/O (FT_fh) | GPIO | ds p. 76 |
| K1 | VDD | VDD | Power (S) | Digital and I/O supply | ds p. 68 |
| K2 | PA3 | PA3 | I/O (FT_ha) | GPIO | ds p. 69 |
| K3 | PA7 | PA7 | I/O (TT_a) | GPIO | ds p. 69 |
| K4 | PB1 | PB1 | I/O (TT_u) | GPIO | ds p. 70 |
| K5 | PE9 | PE9 | I/O (TT_ha) | GPIO | ds p. 71 |
| K6 | PE13 | PE13 | I/O (FT_h) | GPIO | ds p. 72 |
| K7 | PB11 | PB11 | I/O (FT_f) | GPIO | ds p. 73 |
| K8 | PB12 | PB12 | I/O (FT_u) | GPIO; OTG_HS_ID | ds p. 74 |
| K9 | PD8 | PD8 | I/O (FT_h) | GPIO | ds p. 75 |
| K10 | PD12 | PD12 | I/O (FT_fh) | GPIO | ds p. 76 |

## Typical application

- Decoupling, from Figure 68 (ds p. 211):
  - VDD: one 100 nF per VDD ball (D2, F5, K1) plus one 4.7 µF.
  - VDDLDO (F4): 4.7 µF. VCAP (E7, F8): 2 x 2.2 µF, ESR below 100 mΩ (ds p. 216).
  - VDDA (H1): 100 nF + 1 µF. The figure's separate VREF+ 100 nF + 1 µF goes on VDDA here, because VREF+ is double-bonded to VDDA on this package (ds p. 88).
  - VDD33USB (F6): 100 nF + 1 µF.
  - Capacitors may vary ±20 % (ds p. 211). Place each as close as possible to its pins, or under them on the PCB underside (ds p. 211, ds p. 212).
- Connect every VDD, VDDA, VDD33USB, VBAT, VSS and VSSA ball; none may be left open (ds p. 212).
- VDDA is also the ADC/DAC reference on this package. It may be connected to an external reference; the internal reference buffer must stay disabled (ds p. 88).
- Power sequence: while VDD is below 1 V, VDDA and VDD33USB must stay below VDD + 300 mV (ds p. 28).
- USB: the TFBGA100 has no VDD50USB ball, so the USB regulator cannot be fed; VDD33USB (F6) must be supplied directly, 3.0-3.6 V when USB is used (ds p. 27, ds p. 56, ds p. 214). OTG_FS is on PA11/PA12 (C10, B10), OTG_HS's FS PHY on PB14/PB15 (H10, G10) (ds p. 75, ds p. 80).
- PDR_ON (F7): tie to VDD for the internal power-on/power-down reset. Tie to VSS only with an external supervisor, which allows VDD down to 1.62 V; VBAT functionality is then lost (ds p. 22, ds p. 29, ds p. 37).
- NRST (E1): internal 30-50 kΩ pull-up. Figure 76 shows a 0.1 µF capacitor to ground against parasitic resets (ds p. 253).
- Boot: BOOT0 (D5) together with the BOOT_ADDx option bytes selects the boot address; the bootloader (USART, I2C, SPI, USB-DFU) is described in AN2606 (ds p. 27).
- HSE on PH0/PH1 (C1, D1): 4-48 MHz crystal with CL1 = CL2 of 5-25 pF, sized with about 10 pF of pin and board capacitance (ds p. 234).
- LSE on PC14/PC15 (A1, B1): 32.768 kHz crystal. A resistor between OSC32_IN and OSC32_OUT is forbidden (ds p. 236).
- SWD: PA13 = JTMS/SWDIO (A10), PA14 = JTCK/SWCLK (A9) after reset (ds p. 80, ds p. 81).

## Layout notes

- Land: Table 219 recommends, for this 0.8 mm-pitch BGA, a 0.400 mm pad (Dpad), a 0.470 mm typical solder-mask opening (Dsm), a 0.400 mm stencil opening, a 0.100-0.125 mm stencil and 0.120 mm pad traces (ds p. 326).
- The library's `bga::KICAD_TFBGA_100_8X8MM_LAYOUT10X10_P0_8MM` matches it: 0.4 mm round pads, 0.035 mm mask expansion (0.47 mm opening), 0.4 mm paste, a 10 x 10 grid at 0.8 mm, rows A-K without I.
- Package: D = E = 8.00 mm nominal, D1 = E1 = 7.20 mm, e = 0.80 mm, ball b = 0.35-0.45 mm, A ≤ 1.100 mm (ds p. 325, ds p. 326).
- Ball A1: Figure 123's top view puts the A1 index at the top-left corner; the bottom view has columns 10-1 from left to right (ds p. 325). The library has A1 at the top left with columns increasing to the right in top view, which agrees. The LCSC binding carries no rotation; it is not reopened here.
- θJA is 39.3 °C/W (ds p. 349).
- Escape routing for 100 balls at 0.8 mm, and via-in-pad, are not covered by the datasheet.

## Pitfalls

- 480 MHz needs silicon revision V, VOS0, VDDLDO ≥ 1.7 V and TJ ≤ 105 °C (ds p. 22, ds p. 215).
- PA4 and PA5 accept no injected current at all (ds p. 213). On this package the TT_xx pins are PA4, PA5, PA7, PB1, PC2_C, PC3_C, PC4, PC5, PE7, PE8 and PE9 (Table 9, ds pp. 64-88). They are not 5 V tolerant: VIN ≤ VDD + 0.3 V (ds p. 214).
- FT pins above Min(VDD, VDDA, VDD33USB) + 0.3 V need their internal pull-up and pull-down disabled (ds p. 215).
- `_u` pins (here PA9-PA12, PB1 and PB12-PB15) are supplied by VDD33USB, and `_a` pins by VDDA (ds p. 63, Table 9). An unpowered VDD33USB or VDDA affects those GPIOs.
- PC2_C and PC3_C are analog-only pads (pin type ANA). PC2 and PC3 digital functions reach them only through the SYSCFG analog switch (ds p. 67, ds p. 88).
- VDDA is the ADC reference on this package. Noise on VDDA is reference noise.
- BOOT0 tolerates 9 V absolute, but its thresholds differ from other I/Os (ds p. 212, ds p. 246).
- An external reset circuit must pull NRST below VIL(NRST) max, or the reset is ignored (ds p. 253).
- The library's pin names PC14, PC15, PH0, PH1 are the datasheet's PC14-OSC32_IN, PC15-OSC32_OUT, PH0-OSC_IN, PH1-OSC_OUT, and `VDD33_USB` is VDD33USB (ds p. 56, ds p. 65, ds p. 66, ds p. 78).

## Not established by the sources

- **The package is marked "under development".** Table 2's footnote 7 on the TFBGA100 says "This package is under development. Please contact STMicroelectronics for details." (ds p. 21, ds p. 22). DS12110 Rev 8 does not establish that the STM32H743VIH6 is in production, although LCSC lists both MPNs.
- **Silicon revision.** Rev 8 specifies revision Y (400 MHz) and revision V (480 MHz) separately (ds p. 22). Nothing ties either MPN or reel to a revision.
- **No pad-signal disagreement.** All 100 balls match Table 9 (ds pp. 64-88) and Figure 6 (ds p. 56).
- **PDR_ON is `optional` in the library.** The datasheet uses it as a strap, VDD for internal reset or VSS with an external supervisor (ds p. 22, ds p. 29). It gives no internal pull and no floating state, so leaving it open is not covered.
- **NRST is typed `input`.** Table 9 gives it as I/O, RST type, a bidirectional reset pin (ds p. 63, ds p. 66). This is the library's known lack of an open-drain type.
- **USB without VDD50USB.** The datasheet does not say how the USB regulator is handled on packages without a VDD50USB ball. This note's reading, that VDD33USB must be supplied directly, follows from the supply list (ds p. 27) and the ballout (ds p. 56).
- **VDDLDO.** Footnote 8 says VDDLDO is tied to VDD inside packages without the pin (ds p. 88). The TFBGA100 has a VDDLDO ball (F4). The datasheet does not say whether it may be tied straight to VDD; Figure 68 shows it with its own 4.7 µF (ds p. 211).
- The full BOOT0 / BOOT_ADDx behaviour and the bootloader pins are in AN2606, not here (ds p. 27).
- No BOOT0 pull-down value, VBAT capacitor value, or rule for VBAT without a battery is given.
- No crystal part, ESR limit or load-capacitor value beyond the 5-25 pF range is given (ds p. 234).
