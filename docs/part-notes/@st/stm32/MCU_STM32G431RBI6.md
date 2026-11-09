# STM32G431RBI6 — Arm Cortex-M4F MCU, 170 MHz, 128 KB flash, USB FS device + UCPD, UFBGA64 5x5 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@st/stm32::STM32G431RBIx",
  "parts": ["@st/stm32::MCU_STM32G431RBI6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "STM32G431RBI6", "lcsc": "C730103"},
    {"mfr": "STMicroelectronics", "mpn": "STM32G431RBI6TR", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "STM32G431x6 STM32G431x8 STM32G431xB datasheet: Arm Cortex-M4 32-bit MCU+FPU, 170 MHz / 213 DMIPS, up to 128 KB Flash, 32 KB SRAM, rich analog, math accelerator",
     "revision": "DS12589 Rev 2, 2019-10-28",
     "url": "https://www.st.com/resource/en/datasheet/stm32g431rb.pdf",
     "sha256": "5994e7e78a451db7cb0b7ab3f475aea094901f1298a311e0f5826d27e55db98f",
     "retrieved_from": "https://datasheet.lcsc.com/datasheet/pdf/855d365316bdc93c0f165ebc0e08fa9f.pdf"}
  ],
  "status": "draft"
}
```

**Source file.** The PDF is ST-authored (PDF Author "STMICROELECTRONICS"; ST's DS12589 Rev 2 on every page). It was retrieved from LCSC's datasheet mirror (part page <https://www.lcsc.com/datasheet/C730103.pdf>) because st.com's robots.txt was unavailable to the policy-bound client on 2026-11-07. The file is not stored in the repository; the SHA-256 above identifies it. Its PDF metadata title is stale: it reads "Datasheet - STM32G474xB STM32G474xC STM32G474xE", but page 1 and every page footer are the STM32G431x6/x8/xB datasheet DS12589 Rev 2, and this note uses only the body. Page numbers below are PDF page numbers; in this file they equal the printed page numbers (PDF page 51 is printed "51/200").

## Summary

The STM32G431RBI6 is a 170 MHz Arm Cortex-M4 microcontroller with FPU, 128 KB flash and 32 KB SRAM (16 + 6 + 10 KB CCM) in a 5 x 5 mm, 0.5 mm-pitch UFBGA64, rated -40 to 85 °C ambient (suffix 6) (ds p. 1, ds p. 14, ds p. 71, ds p. 188, ds p. 198). Its deciding features are the analog set (two 12-bit ADCs, four DAC channels, four comparators, three op-amps, VREFBUF), the CORDIC/FMAC math accelerators, FDCAN, and a USB FS device plus a USB Type-C / Power Delivery controller (UCPD) on the same chip (ds p. 1, ds p. 13, ds p. 42). The 64-ball variant has 52 GPIOs (ds p. 14). In this package NRST and BOOT0 share balls with GPIOs PG10 and PB8 (ds p. 48). Do not choose it when you need USB host/OTG (the datasheet describes a USB device peripheral only, ds p. 42) or a 0.5 mm BGA cannot be routed on the target board.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU / AHB / APB clock | 0 | | 170 | MHz | fHCLK, fPCLK1, fPCLK2 | ds p. 71 |
| Flash / SRAM | | 128 / 32 | | KB | SRAM = 16 + 6 + 10 KB (CCM) | ds p. 14 |
| GPIO count | | 52 | | | STM32G431Rx | ds p. 14 |
| HSE crystal/resonator frequency | 4 | 8 | 48 | MHz | fOSC_IN | ds p. 104 |
| HSE startup time | | 2 | | ms | VDD stabilized, 8 MHz standard crystal | ds p. 104 |
| LSE startup time | | 2 | | s | VDD stabilized | ds p. 106 |
| HSI48 accuracy, factory calibrated | | | ±3 | % | VDD = 3.0-3.6 V, TA = -15 to 85 °C | ds p. 108 |
| BOR0 threshold, rising / falling | 1.62 / 1.60 | 1.66 / 1.64 | 1.70 / 1.69 | V | VBOR0 | ds p. 72 |
| Reset temporization after BOR0 | | 250 | 400 | µs | tRSTTEMPO, VDD rising | ds p. 72 |
| NRST internal pull-up | 25 | 40 | 55 | kΩ | RPU, VIN = VSS | ds p. 120 |
| NRST input low / high level | | | 0.3 VDD / - | V | VIL(NRST) max; VIH(NRST) min is 0.7 VDD | ds p. 120 |
| USB_DP embedded pull-up, idle | 900 | 1250 | 1500 | Ω | RPUI | ds p. 163 |
| USB driver impedance | 28 | 36 | 44 | Ω | ZDRV, no series resistors needed | ds p. 163 |
| Run current | | 25.5 | 27.0 | mA | 170 MHz, Range 1 boost, all peripherals disabled, TA = 25 °C | ds p. 76 |
| Thermal resistance θJA | | 44.2 | | °C/W | UFBGA64 5 x 5 mm | ds p. 194 |
| Package height A | 0.460 | 0.530 | 0.600 | mm | UFBGA64 | ds p. 188 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD - VSS (including VDD, VDDA, VBAT) | -0.3 to 4.0 | V | ds p. 69 |
| VIN on FT_xxx pins except FT_c | VSS - 0.3 to min(VDD, VDDA) + 4.0 (pull-up/down disabled above 4 V) | V | ds p. 69, ds p. 70 |
| VIN on FT_c pins (PB4, PB6) | VSS - 0.3 to 5.5 | V | ds p. 69 |
| VIN on TT_xx pins and any other pin | VSS - 0.3 to 4.0 | V | ds p. 69 |
| Total current into all VDD / out of all VSS lines | 150 / 150 | mA | ds p. 70 |
| Current into each VDD pin / out of each VSS pin | 100 / 100 | mA | ds p. 70 |
| Output current per I/O, sunk or sourced | 20 | mA | ds p. 70 |
| Total output current of all I/Os, sunk / sourced | 100 / 100 | mA | ds p. 70 |
| Injected current per pin (FT_xxx, TT_xx, NRST) | -5 / 0 | mA | ds p. 70 |
| Total injected current | ±25 | mA | ds p. 70 |
| Junction temperature | 150 | °C | ds p. 70 |
| Storage temperature | -65 to +150 | °C | ds p. 70 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 1.71 | 3.6 | V | ds p. 71 |
| VDDA, ADC or COMP used | 1.62 | 3.6 | V | ds p. 71 |
| VDDA, DAC used | 1.71 | 3.6 | V | ds p. 71 |
| VDDA, OPAMP used | 2.0 | 3.6 | V | ds p. 71 |
| VDDA, VREFBUF used | 2.4 | 3.6 | V | ds p. 71 |
| VDDA, no analog peripheral used | 0 | 3.6 | V | ds p. 71 |
| VBAT | 1.55 | 3.6 | V | ds p. 71 |
| VREF+, VDDA >= 2 V | 2 | VDDA | V | ds p. 22 |
| VREF+, VDDA < 2 V | VDDA | VDDA | V | ds p. 22 |
| VDD for full USB electrical characteristics | 3.0 | 3.6 | V | ds p. 163 |
| VIN, TT_xx I/O | -0.3 | VDD + 0.3 | V | ds p. 71 |
| VIN, FT_c I/O | -0.3 | 5 | V | ds p. 71 |
| VIN, other I/O | -0.3 | min(min(VDD, VDDA) + 3.6, 5.5) | V | ds p. 71 |
| Ambient temperature, suffix 6, max dissipation | -40 | 85 | °C | ds p. 71 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 71 |
| USB crystal-less operation temperature | -15 | 85 | °C | ds p. 163 |

## Pins

The table was generated by script from the library's `STM32G431RBIx` pins block and the UFBGA64 column of Table 12 (ds pp. 51-60), and every ball was compared with both Table 12 and the ballout of Figure 12 (ds p. 48): all 64 balls match. The datasheet names H1 and G2 `VDD_2` and `VSS_2`; the library declares them as `VDD` and `VSS`.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | VDD | VDD | Power | Digital supply | ds p. 60 |
| A2 | PB9 | PB9 | I/O (FT_f) | GPIO | ds p. 60 |
| A3 | PB7 | PB7 | I/O (FT_f) | GPIO; PVD_IN | ds p. 59 |
| A4 | PB6 | PB6 | I/O (FT_c) | GPIO; UCPD1_CC1 | ds p. 59 |
| A5 | PB3 | PB3 | I/O (FT) | JTDO/TRACESWO after reset | ds p. 58 |
| A6 | PC12 | PC12 | I/O (FT) | GPIO | ds p. 58 |
| A7 | PA15 | PA15 | I/O (FT_f) | JTDI after reset | ds p. 57 |
| A8 | VDD | VDD | Power | Digital supply | ds p. 57 |
| B1 | PC13 | PC13 | I/O (FT) | GPIO; WKUP2, RTC_TAMP1, RTC_TS, RTC_OUT1 | ds p. 51 |
| B2 | VSS | VSS | Power | Ground | ds p. 60 |
| B3 | PB8 | PB8-BOOT0 | I/O (FT_f) | BOOT0 (boot select) or GPIO; FDCAN1_RX, I2C1_SCL | ds p. 59 |
| B4 | PB5 | PB5 | I/O (FT_f) | GPIO | ds p. 59 |
| B5 | PD2 | PD2 | I/O (FT) | GPIO | ds p. 58 |
| B6 | PC11 | PC11 | I/O (FT_f) | GPIO | ds p. 57 |
| B7 | VSS | VSS | Power | Ground | ds p. 57 |
| B8 | PA12 | PA12 | I/O (FT_u) | GPIO; USB_DP | ds p. 57 |
| C1 | PC14 | PC14-OSC32_IN | I/O (FT) | GPIO; OSC32_IN (LSE) | ds p. 51 |
| C2 | VBAT | VBAT | Power | Backup supply (RTC, LSE, backup registers) | ds p. 51 |
| C3 | PC1 | PC1 | I/O (TT_a) | GPIO | ds p. 52 |
| C4 | PB4 | PB4 | I/O (FT_c) | NJTRST after reset; UCPD1_CC2 | ds p. 59 |
| C5 | PC10 | PC10 | I/O (FT) | GPIO | ds p. 57 |
| C6 | PA14 | PA14 | I/O (FT_f) | SWCLK-JTCK after reset | ds p. 57 |
| C7 | PA13 | PA13 | I/O (FT_f) | SWDIO-JTMS after reset | ds p. 57 |
| C8 | PA11 | PA11 | I/O (FT_u) | GPIO; USB_DM | ds p. 56 |
| D1 | PC15 | PC15-OSC32_OUT | I/O (FT) | GPIO; OSC32_OUT (LSE) | ds p. 51 |
| D2 | PG10 | PG10-NRST | I/O (FT) | NRST (reset input, embedded pull-up) or GPIO; MCO | ds p. 51 |
| D3 | PC2 | PC2 | I/O (FT_a) | GPIO | ds p. 52 |
| D4 | PA4 | PA4 | I/O (TT_a) | GPIO; DAC1_OUT1 | ds p. 53 |
| D5 | PC4 | PC4 | I/O (FT_fa) | GPIO | ds p. 53 |
| D6 | PA10 | PA10 | I/O (FT_da) | GPIO; UCPD1_DBCC2 | ds p. 56 |
| D7 | PA9 | PA9 | I/O (FT_fd) | GPIO; UCPD1_DBCC1 | ds p. 56 |
| D8 | PC9 | PC9 | I/O (FT_f) | GPIO | ds p. 56 |
| E1 | PF0 | PF0-OSC_IN | Input (FT_fa) | GPIO; OSC_IN (HSE) | ds p. 51 |
| E2 | PC0 | PC0 | I/O (FT_a) | GPIO | ds p. 52 |
| E3 | PA1 | PA1 | I/O (TT_a) | GPIO | ds p. 52 |
| E4 | PA5 | PA5 | I/O (TT_a) | GPIO; DAC1_OUT2 | ds p. 53 |
| E5 | PB0 | PB0 | I/O (TT_a) | GPIO | ds p. 53 |
| E6 | PA8 | PA8 | I/O (FT_f) | GPIO | ds p. 56 |
| E7 | PC7 | PC7 | I/O (FT) | GPIO | ds p. 56 |
| E8 | PC6 | PC6 | I/O (FT) | GPIO | ds p. 56 |
| F1 | PF1 | PF1-OSC_OUT | Output (FT_a) | GPIO; OSC_OUT (HSE) | ds p. 51 |
| F2 | PA0 | PA0 | I/O (TT_a) | GPIO; WKUP1 | ds p. 52 |
| F3 | PA2 | PA2 | I/O (TT_a) | GPIO | ds p. 52 |
| F4 | PC5 | PC5 | I/O (TT_a) | GPIO | ds p. 53 |
| F5 | PB1 | PB1 | I/O (TT_a) | GPIO | ds p. 53 |
| F6 | PC8 | PC8 | I/O (FT_f) | GPIO | ds p. 56 |
| F7 | PB15 | PB15 | I/O (FT_a) | GPIO | ds p. 55 |
| F8 | PB14 | PB14 | I/O (TT_a) | GPIO | ds p. 55 |
| G1 | PC3 | PC3 | I/O (FT_a) | GPIO | ds p. 52 |
| G2 | VSS | VSS_2 | Power | Ground | ds p. 52 |
| G3 | PA6 | PA6 | I/O (TT_a) | GPIO | ds p. 53 |
| G4 | VSSA | VSSA | Power | Analog ground; VREF- is double-bonded to VSSA (ds p. 22) | ds p. 54 |
| G5 | VREFP | VREF+ | Power | ADC/DAC reference input; VREFBUF output | ds p. 54 |
| G6 | PB13 | PB13 | I/O (TT_a) | GPIO | ds p. 55 |
| G7 | VSS | VSS | Power | Ground | ds p. 54 |
| G8 | PB12 | PB12 | I/O (FT_a) | GPIO | ds p. 55 |
| H1 | VDD | VDD_2 | Power | Digital supply | ds p. 52 |
| H2 | PA3 | PA3 | I/O (TT_a) | GPIO | ds p. 52 |
| H3 | PA7 | PA7 | I/O (TT_a) | GPIO | ds p. 53 |
| H4 | PB2 | PB2 | I/O (TT_a) | GPIO | ds p. 53 |
| H5 | VDDA | VDDA | Power | Analog supply | ds p. 54 |
| H6 | PB10 | PB10 | I/O (TT_a) | GPIO | ds p. 54 |
| H7 | PB11 | PB11 | I/O (FT_a) | GPIO | ds p. 54 |
| H8 | VDD | VDD | Power | Digital supply | ds p. 54 |

## Typical application

- Decouple the VDD balls with n x 100 nF plus one 4.7 µF, VDDA with 10 nF + 1 µF, and VREF+ with 100 nF + 1 µF (Figure 16) (ds p. 68).
- Connect every VDD, VDDA and VBAT ball and every VSS and VSSA ball; none may be left open (ds p. 69).
- VDDA may differ from VDD. When the analog peripherals are not used, the datasheet prefers VDDA connected to VDD (ds pp. 21-22).
- VREF+ (G5) is a separate ball on this package. Feed it from VDDA when VDDA < 2 V, otherwise from 2 V to VDDA. The internal VREFBUF can drive it with 2.048, 2.5 or 2.95 V, which needs VDDA >= 2.4 V (ds p. 22, ds p. 71).
- NRST is ball D2 (`PG10` in the library). It has an internal 25-55 kΩ pull-up; add 0.1 µF from NRST to VSS against parasitic resets, as close as possible to the device (Figure 27) (ds p. 120, ds p. 121).
- BOOT0 is ball B3 (`PB8` in the library). BOOT0 comes from this ball or from the nBOOT0 option bit, chosen by the nBOOT_SEL option bit; with nBOOT1 it selects user flash, system memory or SRAM. The system-memory bootloader reprograms flash over USART, I2C, SPI or USB DFU (ds p. 19). If PB8 is left unconnected after startup, put it in a mode other than analog to limit consumption (ds p. 60).
- Optional HSE: a 4-48 MHz crystal on PF0-OSC_IN (E1) and PF1-OSC_OUT (F1), with CL1 = CL2 in the 5-20 pF range sized with about 10 pF of pin and board capacitance included; REXT depends on the crystal (ds p. 104, ds p. 105).
- Optional LSE: a 32.768 kHz crystal on PC14-OSC32_IN (C1) and PC15-OSC32_OUT (D1). Do not fit a resistor between OSC32_IN and OSC32_OUT (ds p. 106).
- USB device: route PA11 = USB_DM (C8) and PA12 = USB_DP (B8) to the connector. The D+ pull-up is embedded and no series termination resistors are needed (ds p. 42, ds p. 163). The 48 MHz USB clock comes from the PLL with an HSE crystal, or from HSI48 trimmed to USB SOF for crystal-less operation (ds p. 42), which is specified only from -15 to 85 °C (ds p. 163).
- USB Type-C / PD: UCPD1_CC1 is PB6 (A4) and UCPD1_CC2 is PB4 (C4); the dead-battery functions UCPD1_DBCC1 and UCPD1_DBCC2 are on PA9 (D7) and PA10 (D6) (ds p. 50, ds p. 56, ds p. 59). The controller provides Rp and Rd and supports dead battery (ds p. 42).
- SWD: bring out PA13 (SWDIO, C7), PA14 (SWCLK, C6), NRST (D2), VDD and VSS. After reset these are debug pins, with internal pull-ups on PA15, PA13 and PB4 and a pull-down on PA14 (ds p. 60).

## Layout notes

- Place each decoupling capacitor as close as possible to its supply/ground pair, or below it on the PCB underside (ds p. 68).
- Place the HSE and LSE crystals and their load capacitors as close as possible to the oscillator pins (ds p. 104, ds p. 105).
- Land: ST's rules for this 0.5 mm-pitch BGA are a 0.280 mm pad (Dpad), a 0.370 mm typical solder-mask opening (Dsm), a 0.280 mm stencil opening, a 0.100-0.125 mm stencil and 0.100 mm pad traces (Table 100, Figure 62) (ds p. 189). The library land `bga::KICAD_UFBGA_64_5X5MM_LAYOUT8X8_P0_5MM` matches: 0.28 mm round pads, 0.045 mm mask expansion (0.37 mm opening), 0.28 mm paste, 0.5 mm pitch.
- Ball A1: in ST's top view the A1 index is at the top-left corner, with rows A-H downwards and columns 1-8 to the right; the ball matrix spans D1 = E1 = 3.50 mm (Figure 61, Table 99) (ds p. 188). The library land has A1 at (-1.75, -1.75) mm, top left, with the same row and column directions. The supplier binding carries no rotation.
- Body 5.00 x 5.00 mm nominal, height 0.600 mm max, ball diameter b 0.17-0.33 mm (Table 99) (ds p. 188).
- No exposed pad. θJA is 44.2 °C/W (ds p. 194).

## Pitfalls

- Two control functions hide behind GPIO names in the library: D2 is `PG10` (NRST) and B3 is `PB8` (BOOT0), both `optional` (ds p. 48, ds p. 51, ds p. 59). A design that leaves D2 unrouted has no reset pin; one that leaves B3 floating leaves the boot source to chance unless nBOOT_SEL selects the option bit (ds p. 19).
- VREF+ (G5) is a separate ball from VDDA (H5) on UFBGA64 and must be supplied within the VREF+ limits; an agent copying a 32-pin G431 design, where they share one VDDA/VREF+ pin, will miss it (ds p. 22, ds p. 48, ds p. 54).
- PC13-PC15 are behind the backup power switch: at most 3 mA, at most 2 MHz with 30 pF, and never as a current source such as an LED driver (ds p. 60).
- Only FT pins tolerate 5 V, and then only with the internal pull-up/pull-down disabled above min(VDD, VDDA) + 0.3 V. TT_a pins (PA0-PA7, PB0-PB2, PB10, PB13, PB14, PC1, PC5) are 3.6 V tolerant only (VDD + 0.3 V operating max). The UCPD pins PB4 and PB6 (FT_c) are limited to 5.5 V absolute and 5 V operating (ds p. 50, ds p. 69, ds p. 71).
- There is no separate VDDUSB: the USB transceiver needs VDD of 3.0-3.6 V for full electrical characteristics (it works, degraded, down to 2.7 V) (ds p. 163).
- 170 MHz needs the main regulator in Range 1 boost mode; Range 1 normal mode allows 150 MHz and Range 2 26 MHz (ds p. 22).
- VBAT is a separate ball (C2) and must be connected (ds p. 69).

## Not established by the sources

- **Library vs datasheet: no ball signal differs.** All 64 balls of `STM32G431RBIx` match Table 12 and Figure 12 (ds pp. 48, 51-60). The library land matches Table 100 and Figure 61 (ds pp. 188-189). The only naming difference is `VDD`/`VSS` for the datasheet's `VDD_2`/`VSS_2` (H1, G2).
- The PDF's metadata title names STM32G474; the body is DS12589 Rev 2 for STM32G431x6/x8/xB. Page 1 states "22 Kbytes of SRAM" plus "10 Kbytes" CCM SRAM, while the title and Table 2 state 32 KB total (16 + 6 + 10) (ds p. 1, ds p. 14); the figures are consistent only if read that way.
- The default state of the PG10-NRST and PB8-BOOT0 option bits (NRST mode, nBOOT_SEL, nBOOT0, nBOOT1) and the boot truth table are not in the datasheet; it refers to the reference manual (ds p. 19).
- Whether the UCPD dead-battery pull-downs are active on PB4/PB6 at reset, and how PA9/PA10 (DBCC1/DBCC2) should be wired when UCPD is unused, is not stated; the datasheet only names the functions (ds pp. 42, 50, 56, 59).
- No BOOT0 pull resistor value, no VBAT capacitor value and no rule for VBAT without a battery are given.
- How many of the n x 100 nF capacitors to use for the four VDD balls, and how to escape a 0.5 mm-pitch 8 x 8 BGA (via size, layer count), are not given; only the land rules of Table 100 are (ds p. 68, ds p. 189).
- No crystal part, ESR limit or exact load-capacitor value is given beyond the 5-20 pF range and AN2867 (ds p. 104, ds p. 105).
- No USB ESD protection, VBUS sensing circuit or Type-C connector circuit is specified.
