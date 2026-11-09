# GD32F303CBT6 — Arm Cortex-M4 MCU, 120 MHz, 128 KB flash, 32 KB SRAM, USB FS device and CAN, LQFP48 7 × 7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@gigadevice/gd32::GD32F303CB_LQFP48",
  "parts": ["@gigadevice/gd32::MCU_GD32F303CBT6"],
  "identities": [
    {"mfr": "GigaDevice", "mpn": "GD32F303CBT6", "lcsc": "C3013779"}
  ],
  "sources": [
    {"id": "ds", "title": "GD32F303xB Datasheet, Arm Cortex-M4 32-bit MCU",
     "revision": "Revision 1.1, Feb. 2026",
     "url": "https://www.gd32mcu.com/data/documents/datasheet/GD32F303xB%20Datasheet_Rev1.1.pdf",
     "sha256": "a485cbdce30f6e52f787537220230324421cbfd4864fc74f2d852830bf602e74",
     "local": "lib/@gigadevice/gd32/docs/gd32f303xb-rev1.1.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are one-based PDF page indices, cover included (`pdftotext -f N -l N`). The printed page number is the PDF page minus one (the LQFP48 pinout Figure 2-4 is PDF p. 12, printed 11). The URL and SHA-256 are the ones recorded in `lib/@gigadevice/gd32/docs/sources.md`; the local file hashes to that SHA-256. Nothing was downloaded to confirm the URL still serves the same bytes. This is the xB (128 KB) datasheet, not the larger-memory GD32F303xx document.

## Summary

The GD32F303CBT6 is GigaDevice's Arm Cortex-M4 microcontroller at up to 120 MHz with 128 KB flash and 32 KB SRAM in a 7 × 7 mm LQFP48, grade 6 (-40 to +85 °C) (ds p. 8, ds p. 9, ds p. 85). It has a USB 2.0 full-speed device (USBD) that can run crystal-less from the IRC48M, one CAN 2.0B, three USARTs, two I2Cs, three SPIs, three 12-bit ADCs on 10 external channels, a 2-channel DAC and 37 GPIOs (ds p. 9, ds p. 40). It runs from 2.6–3.6 V, with VDDA equal to VDD and a separate VBAT for the backup domain (ds p. 43). The datasheet makes no compatibility claim with any other vendor's part, and the library declares no cross-vendor alternates. Do not choose it if you need a supply below 2.6 V, USB host, or operation above 85 °C (the -T7 grade is not modeled).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB clock fHCLK | | | 120 | MHz | | ds p. 43 |
| APB1 / APB2 clock | | | 60 / 120 | MHz | | ds p. 43 |
| Flash / SRAM | | 128 / 32 | | KB | GD32F303CBT6 | ds p. 9 |
| GPIO count / ADC external channels | | 37 / 10 | | | CBT6; 3 ADC units | ds p. 9 |
| HXTAL crystal frequency | 4 | 8 | 32 | MHz | 2.6 V ≤ VDD ≤ 3.6 V | ds p. 52 |
| HXTAL recommended matching capacitance (each pin) | | 20 | 30 | pF | CHXTAL1 = CHXTAL2 = 2 × (CLOAD − CS) | ds p. 52 |
| HXTAL startup time | | 1.8 | | ms | VDD = 3.3 V, TA = 25 °C | ds p. 52 |
| LXTAL recommended matching capacitance (each pin) | | 15 | | pF | 32.768 kHz; CLXTAL1 = CLXTAL2 = 2 × (CLOAD − CS) | ds p. 53 |
| IRC8M accuracy, factory trimmed | -2.0 | | +2.0 | % | VDD = VDDA = 3.3 V, TA = -40 to +85 °C | ds p. 54 |
| IRC48M accuracy, factory trimmed | -3.3 | | +3.3 | % | VDD = VDDA = 3.3 V, TA = -40 to +85 °C | ds p. 55 |
| Power-on reset threshold VPOR | | 2.4 | | V | | ds p. 51 |
| Power-down reset threshold VPDR | | 1.8 | | V | PDR hysteresis 600 mV typ | ds p. 51 |
| Reset temporization tRSTTEMPO | | 2.43 | | ms | | ds p. 51 |
| Start-up time, NRST high to main() | 40 | 74 | 110 | ms | 128 KB code area, PLL off | ds p. 44 |
| NRST input low level VIL | -0.3 | | 0.3 VDD | V | 2.6 V ≤ VDD = VDDA ≤ 3.6 V | ds p. 57 |
| NRST input high level VIH | 0.7 VDD | | VDD + 0.3 | V | 2.6 V ≤ VDD = VDDA ≤ 3.6 V | ds p. 57 |
| NRST internal pull-up | | 40 | | kΩ | | ds p. 57 |
| GPIO internal pull-up, all pins except PA10 | 33.76 | 40 | 52.09 | kΩ | VIN = VSS | ds p. 57 |
| GPIO internal pull-up / pull-down, PA10 | 8.84 | 10 | 13.53 | kΩ | pull-up 8.84–13.41, pull-down 9.14–13.53 | ds p. 57, ds p. 58 |
| USBD operating VDD | 3.0 | | 3.6 | V | | ds p. 69 |
| Run current IDD + IDDA | | 35.5 | | mA | 120 MHz from 25 MHz HXTAL, all peripherals on, 3.3 V | ds p. 44 |
| ESD, HBM / CDM | | | 4000 / 800 | V | TA = 25 °C | ds p. 52 |
| θJA, LQFP48 | | 64.40 | | °C/W | natural convection, 2S2P, simulated | ds p. 83 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | VSS − 0.3 to VSS + 3.6 | V | ds p. 42 |
| VDDA | VSSA − 0.3 to VSSA + 3.6 | V | ds p. 42 |
| VBAT | VSS − 0.3 to VSS + 3.6 | V | ds p. 42 |
| VIN on 5 V-tolerant pins | VSS − 0.3 to VDD + 3.6, never above 5.5 | V | ds p. 42 |
| VIN on other I/O | VSS − 0.3 to 3.6 | V | ds p. 42 |
| Difference between VDD and VDDA, power-up and operation | 300 max | mV | ds p. 42 |
| Variation between VDD pins / between ground pins | 50 / 50 | mV | ds p. 42 |
| Current per GPIO pin / total for all GPIO pins | ±25 / ±80 | mA | ds p. 42 |
| Power dissipation, LQFP48, TA = 85 °C (grade 6) | 621 | mW | ds p. 42 |
| Junction temperature | 125 | °C | ds p. 42 |
| Storage temperature | -65 to +150 | °C | ds p. 42 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 2.6 | 3.6 | V | ds p. 43 |
| VDDA (same as VDD) | 2.6 | 3.6 | V | ds p. 43 |
| VBAT (and VBAT ≤ VDD + 0.3 V when both are powered) | 1.8 | 3.6 | V | ds p. 43 |
| VDD for the USBD | 3.0 | 3.6 | V | ds p. 69 |
| VDD rise time rate | 0 | ∞ | µs/V | ds p. 44 |
| VDD fall time rate | 20 | ∞ | µs/V | ds p. 44 |
| Ambient temperature, grade 6 | -40 | 85 | °C | ds p. 41, ds p. 42 |

## Pins

Pin map from Figure 2-4 (ds p. 12) and Table 2-5 (ds pp. 30–32), checked by rendering both pages. Functions are the defaults and alternates; "Remap" functions need AFIO configuration. Functions the table marks as GD32F303CG-only (TIMER8, TIMER9, TIMER10, TIMER11, TIMER12, TIMER13) are omitted (ds p. 32, note 3). "5VT" is the datasheet's 5 V-tolerant level.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VBAT | VBAT | Power | Backup-domain supply | ds p. 30 |
| 2 | PC13 | PC13-TAMPER-RTC | I/O | GPIO PC13; TAMPER-RTC | ds p. 30 |
| 3 | PC14 | PC14-OSC32IN | I/O | GPIO PC14; LXTAL OSC32IN | ds p. 30 |
| 4 | PC15 | PC15-OSC32OUT | I/O | GPIO PC15; LXTAL OSC32OUT | ds p. 30 |
| 5 | PD0 | OSCIN (PD0-OSCIN in Figure 2-4) | Input | HXTAL OSCIN after reset; PD0 by remap | ds p. 30 |
| 6 | PD1 | OSCOUT (PD1-OSCOUT in Figure 2-4) | Output | HXTAL OSCOUT after reset; PD1 by remap | ds p. 30 |
| 7 | NRST | NRST | I/O | Reset, active low, internal pull-up | ds p. 30 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 30 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 30 |
| 10 | PA0 | PA0-WKUP | I/O | WKUP, USART1_CTS, ADC012_IN0, TIMER1_CH0/ETI, TIMER4_CH0 | ds p. 30 |
| 11 | PA1 | PA1 | I/O | USART1_RTS, ADC012_IN1, TIMER1_CH1, TIMER4_CH1 | ds p. 30 |
| 12 | PA2 | PA2 | I/O | USART1_TX, ADC012_IN2, TIMER1_CH2, TIMER4_CH2, SPI0_IO2 | ds p. 30 |
| 13 | PA3 | PA3 | I/O | USART1_RX, ADC012_IN3, TIMER1_CH3, TIMER4_CH3, SPI0_IO3 | ds p. 30 |
| 14 | PA4 | PA4 | I/O | SPI0_NSS, USART1_CK, ADC01_IN4, DAC0_OUT0 | ds p. 30 |
| 15 | PA5 | PA5 | I/O | SPI0_SCK, ADC01_IN5, DAC0_OUT1 | ds p. 30 |
| 16 | PA6 | PA6 | I/O | SPI0_MISO, ADC01_IN6, TIMER2_CH0 | ds p. 30 |
| 17 | PA7 | PA7 | I/O | SPI0_MOSI, ADC01_IN7, TIMER2_CH1 | ds p. 31 |
| 18 | PB0 | PB0 | I/O | ADC01_IN8, TIMER2_CH2 | ds p. 31 |
| 19 | PB1 | PB1 | I/O | ADC01_IN9, TIMER2_CH3 | ds p. 31 |
| 20 | PB2 | PB2 | I/O, 5VT | GPIO PB2; BOOT1 | ds p. 31 |
| 21 | PB10 | PB10 | I/O, 5VT | I2C1_SCL, USART2_TX | ds p. 31 |
| 22 | PB11 | PB11 | I/O, 5VT | I2C1_SDA, USART2_RX | ds p. 31 |
| 23 | VSS_1 | VSS_1 | Power | Ground | ds p. 31 |
| 24 | VDD_1 | VDD_1 | Power | Supply | ds p. 31 |
| 25 | PB12 | PB12 | I/O, 5VT | SPI1_NSS, I2C1_SMBA, USART2_CK, TIMER0_BRKIN, I2S1_WS | ds p. 31 |
| 26 | PB13 | PB13 | I/O, 5VT | SPI1_SCK, USART2_CTS, TIMER0_CH0_ON, I2S1_CK | ds p. 31 |
| 27 | PB14 | PB14 | I/O, 5VT | SPI1_MISO, USART2_RTS, TIMER0_CH1_ON | ds p. 31 |
| 28 | PB15 | PB15 | I/O, 5VT | SPI1_MOSI, TIMER0_CH2_ON, I2S1_SD | ds p. 31 |
| 29 | PA8 | PA8 | I/O, 5VT | USART0_CK, TIMER0_CH0, CK_OUT0, CTC_SYNC | ds p. 31 |
| 30 | PA9 | PA9 | I/O, 5VT | USART0_TX, TIMER0_CH1 | ds p. 31 |
| 31 | PA10 | PA10 | I/O, 5VT | USART0_RX, TIMER0_CH2 | ds p. 31 |
| 32 | PA11 | PA11 | I/O, 5VT | USBDM, CAN0_RX, USART0_CTS, TIMER0_CH3 | ds p. 31 |
| 33 | PA12 | PA12 | I/O, 5VT | USBDP, CAN0_TX, USART0_RTS, TIMER0_ETI | ds p. 31, ds p. 32 |
| 34 | PA13 | PA13 | I/O, 5VT | JTMS/SWDIO after reset; PA13 by remap | ds p. 32 |
| 35 | VSS_2 | VSS_2 | Power | Ground | ds p. 32 |
| 36 | VDD_2 | VDD_2 | Power | Supply | ds p. 32 |
| 37 | PA14 | PA14 | I/O, 5VT | JTCK/SWCLK after reset; PA14 by remap | ds p. 32 |
| 38 | PA15 | PA15 | I/O, 5VT | JTDI after reset; SPI2_NSS, I2S2_WS; remap PA15, SPI0_NSS, TIMER1_CH0 | ds p. 32 |
| 39 | PB3 | PB3 | I/O, 5VT | JTDO after reset; SPI2_SCK, I2S2_CK; remap PB3, TRACESWO, SPI0_SCK | ds p. 32 |
| 40 | PB4 | PB4 | I/O, 5VT | NJTRST after reset; SPI2_MISO; remap PB4, SPI0_MISO, TIMER2_CH0 | ds p. 32 |
| 41 | PB5 | PB5 | I/O | I2C0_SMBA, SPI2_MOSI, I2S2_SD; remap SPI0_MOSI, TIMER2_CH1 | ds p. 32 |
| 42 | PB6 | PB6 | I/O, 5VT | I2C0_SCL, TIMER3_CH0; remap USART0_TX | ds p. 32 |
| 43 | PB7 | PB7 | I/O, 5VT | I2C0_SDA, TIMER3_CH1; remap USART0_RX | ds p. 32 |
| 44 | BOOT0 | BOOT0 | Input | Boot selection | ds p. 32 |
| 45 | PB8 | PB8 | I/O, 5VT | TIMER3_CH2; remap I2C0_SCL, CAN0_RX | ds p. 32 |
| 46 | PB9 | PB9 | I/O, 5VT | TIMER3_CH3; remap I2C0_SDA, CAN0_TX | ds p. 32 |
| 47 | VSS_3 | VSS_3 | Power | Ground | ds p. 32 |
| 48 | VDD_3 | VDD_3 | Power | Supply | ds p. 32 |

## Typical application

- Decoupling (Figure 4-1): 100 nF on VBAT; one 4.7 µF plus 100 nF per VDD pin on the VDD rail (three VDD pins here: 24, 36, 48); 1 µF plus 10 nF on VDDA; all as close to the pins as possible (ds p. 43). On packages below 100 pins VREFP/VREFN are not bonded out and are connected internally to VDDA/VSSA, so the LQFP48 needs no reference capacitors (ds p. 43).
- Power VDD and VDDA from the same source: VDDA is "same as VDD", and the two may differ by at most 300 mV during power-up and operation (ds p. 42, ds p. 43). VDDA and VSSA must be connected to VDD and VSS (ds p. 34).
- VBAT: 1.8–3.6 V. When VDD and VBAT are both powered, VBAT must be at most VDD + 0.3 V (ds p. 43).
- NRST: 10 kΩ pull-up to VDD, 100 nF to GND and a push-button across the capacitor (Figure 4-4). An external reset circuit must pull NRST below VIL(NRST) (ds p. 57).
- Boot: BOOT0 (pad 44) and BOOT1 on PB2 (pad 20) select main flash, system memory or SRAM; main flash is the default (ds p. 31, ds p. 34). The system-memory loader reprograms flash over USART0 on PA9/PA10 (pads 30/31); USART1 boot is listed only for CG/IG/KG devices (ds p. 34).
- Debug: PA13/PA14 (pads 34/37) are JTMS-SWDIO and JTCK-SWCLK after reset; PA15, PB3 and PB4 are JTDI, JTDO and NJTRST until remapped (ds p. 32).
- USB device: PA11 = USBDM (pad 32), PA12 = USBDP (pad 33). The 48 MHz USB clock comes either from the PLL with an HXTAL crystal, or from the IRC48M in automatic trimming mode without a crystal (ds p. 40). Keep VDD at 3.0–3.6 V for USB (ds p. 69).
- CAN: CAN0 is on PA11/PA12 or, by remap, PB8/PB9 (ds p. 31, ds p. 32). It needs an external transceiver.
- HXTAL: 4–32 MHz on pads 5/6 with CHXTAL1 = CHXTAL2 = 2 × (CLOAD − CS) (ds p. 52). LXTAL: 32.768 kHz on PC14/PC15 (pads 3/4), 15 pF typical matching capacitance (ds p. 53).

## Layout notes

- Place every decoupling capacitor as close as possible to its pin (ds p. 43).
- GigaDevice's recommended land (Figure 5-6) is 1.20 × 0.30 mm pads at 0.50 mm pitch with a 9.70 mm outside span, 7.30 mm inside span and 5.80 mm tangential span (ds p. 80). The library land `qfp::QFP50P900X900X160_48N` matches: 1.2 × 0.3 mm pads with row centres at ±4.25 mm, and pad 1 at the left end of one row with counterclockwise numbering from the top, as in Figure 5-6 rotated without mirroring.
- Package (Table 5-3): body 6.90–7.10 mm, lead span 8.80–9.20 mm, lead width b 0.18–0.26 mm, foot L 0.45–0.75 mm, height A 1.60 mm max (ds p. 79). The LQFP48 has no exposed pad.

## Pitfalls

- 5 V tolerance is pin-specific. PA0–PA7, PB0, PB1, PB5, PC13–PC15, the oscillator pins, NRST and BOOT0 are not 5VT (ds pp. 30–32) and are limited to 3.6 V absolute (ds p. 42). 5VT pins must still stay at or below 5.5 V (ds p. 42).
- Pads 5/6 are OSCIN/OSCOUT after reset. Using them as PD0/PD1 needs a remap, and PD0/PD1 cannot be EXTI sources in this package (ds p. 30, ds p. 32). The library names them PD0/PD1.
- PA10's internal pull-up and pull-down are about 10 kΩ, not the 40 kΩ of the other pins (ds p. 57, ds p. 58). This loads a USART0 RX line or a sensitive input.
- PC13–PC15 are supplied through the backup power switch: about 3 mA source in total, at most 2 MHz and 30 pF in output mode (ds p. 59).
- PA15, PB3 and PB4 are JTAG pins after reset (ds p. 32). Peripherals attached to them see JTAG levels until firmware releases them.
- Several alternate functions in Table 2-5 exist only on GD32F303CG (note 3) (ds p. 32), and the bootloader's USART1 option applies only to CG/IG/KG devices (ds p. 34).
- The start-up time from NRST release to main() is 40–110 ms, and so is the wakeup from Standby (ds p. 44). A supervisor or host that expects a fast reset may time out.
- USB is not available below VDD = 3.0 V even though the core runs from 2.6 V (ds p. 43, ds p. 69).

## Not established by the sources

- **USB D+ pull-up.** The datasheet does not say whether the USBD has an internal 1.5 kΩ pull-up on USBDP, and gives no external circuit (ds pp. 40, 69).
- **The BOOT0/BOOT1 truth table** is not in this datasheet; it lists three boot sources only (ds p. 34). No BOOT0 or BOOT1 resistor value is given.
- **VBAT when no backup battery is used.** `lib/@gigadevice/gd32/docs/gd32f303.md` says to connect VBAT "to the main supply when backup operation is unused". The datasheet gives only the 1.8–3.6 V range, the VBAT ≤ VDD + 0.3 V rule and a 100 nF capacitor (ds p. 43); it does not state the tie to VDD.
- **POR threshold.** Section 3.3 says POR/PDR ensure operation "starting from/down to 2.6 V" (ds p. 34), but Table 4-10 gives VPOR = 2.4 V and VPDR = 1.8 V typical (ds p. 51). Between 1.8 V and 2.6 V on a falling supply the part may be out of its operating range but not yet in reset; use the LVD (ds p. 51) if that matters.
- **ADC input range text.** Section 3.6 says the input range is "VREFN to VREFP" and also "between 2.6 V and 3.6 V" (ds p. 35); on the LQFP48 VREFP/VREFN are VDDA/VSSA (ds p. 43). The ADC electrical table should be used (ds p. 60).
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- **No crystal ESR or drive-level limit** is given for HXTAL or LXTAL (ds pp. 52–53). The datasheet refers to AN053 for hardware design (ds p. 43), which the package does not pin.
- **Thermal figures are simulations** (ds p. 84).
- **Other ordering codes are not modeled.** GD32F303CBT7 (-40 to +105 °C) and GD32F303CBU6 (QFN48) exist (ds p. 85); the library has only CBT6.
