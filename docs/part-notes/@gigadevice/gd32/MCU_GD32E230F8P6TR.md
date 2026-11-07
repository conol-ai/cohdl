# GD32E230F8P6TR — Arm Cortex-M23 MCU, 72 MHz, 64 KB flash, 8 KB SRAM, TSSOP20

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@gigadevice/gd32::GD32E230F8_TSSOP20",
  "parts": ["@gigadevice/gd32::MCU_GD32E230F8P6TR"],
  "identities": [
    {"mfr": "GigaDevice", "mpn": "GD32E230F8P6TR", "lcsc": "C380785"}
  ],
  "sources": [
    {"id": "ds", "title": "GD32E230xx Datasheet, Arm Cortex-M23 32-bit MCU",
     "revision": "Revision 3.1, Feb. 2026",
     "url": "https://www.gd32mcu.com/data/documents/datasheet/GD32E230xx%20Datasheet_Rev3.1.pdf",
     "sha256": "aa564218bdad841f82027ae85559d66301004c204001109e9b646c86f0d172c2",
     "local": "lib/@gigadevice/gd32/docs/gd32e230-rev3.1.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are one-based PDF page indices, cover included (`pdftotext -f N -l N`). The printed page number is the PDF page minus one (the pinout Figure 2-8 is PDF p. 15, printed 14). The URL and SHA-256 are the ones recorded in `lib/@gigadevice/gd32/docs/sources.md`; the local file hashes to that SHA-256. Nothing was downloaded to confirm the URL still serves the same bytes.

## Summary

The GD32E230F8P6TR is GigaDevice's Arm Cortex-M23 microcontroller at up to 72 MHz with 64 KB flash and 8 KB SRAM in a 4.4 × 6.5 mm TSSOP20, grade 6 (-40 to +85 °C), shipped on tape and reel (ds p. 8, ds p. 10, ds p. 96). The TSSOP20 brings out 15 GPIOs and 9 external ADC channels; it has two USARTs, two I2Cs, two SPIs, one I2S, a comparator and no USB (ds p. 10). It runs from a single 1.8–3.6 V supply (ds p. 51). Its pins share one VSS for digital and analog ground, it has no VBAT pin and no LXTAL pins, and only six of its pins are 5 V tolerant (ds p. 15, ds p. 35, ds p. 36). It is not a pin- or firmware-compatible STM32 replacement; choose it for a small, low-cost 3.3 V controller and not when you need USB, an RTC crystal or more than 15 I/Os.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| AHB / APB clock | 0 | | 72 | MHz | fHCLK1, fHCLK2, fAPB1, fAPB2 | ds p. 51 |
| Flash / SRAM | | 64 / 8 | | KB | GD32E230F8P6 | ds p. 10 |
| GPIO count | | 15 | | | GD32E230FxP6, TSSOP20 | ds p. 10 |
| ADC external channels | | 9 | | | TSSOP20; 12-bit, up to 2 MSPS | ds p. 10, ds p. 43 |
| HXTAL crystal frequency | 4 | 8 | 32 | MHz | VDD = 3.3 V | ds p. 60 |
| HXTAL recommended matching capacitance (each pin) | | 20 | 30 | pF | CHXTAL1 = CHXTAL2 = 2 × (CLOAD − CS) | ds p. 60 |
| HXTAL startup time | | 1.8 | | ms | VDD = 3.3 V | ds p. 60 |
| IRC8M accuracy, factory trimmed | -1.0 | | +1.0 | % | VDD = VDDA = 3.3 V, TA = 25 °C | ds p. 62 |
| IRC28M accuracy, factory trimmed | -2.0 | | +2.0 | % | VDD = VDDA = 3.3 V, TA = 25 °C | ds p. 63 |
| Power-on reset threshold VPOR | | 1.71 | | V | | ds p. 59 |
| Power-down reset threshold VPDR | | 1.67 | | V | | ds p. 59 |
| Reset temporization tRSTTEMPO | | 2 | | ms | | ds p. 59 |
| Start-up time, NRST high to first instruction | | 76 | | µs | IRC8M, PLL off | ds p. 52 |
| NRST input low level VIL | -0.5 | | 0.35 VDD | V | 1.8 V ≤ VDD = VDDA ≤ 3.6 V | ds p. 65 |
| NRST input high level VIH | 0.65 VDD | | VDD + 0.5 | V | 1.8 V ≤ VDD = VDDA ≤ 3.6 V | ds p. 65 |
| NRST internal pull-up | | 40 | | kΩ | | ds p. 65 |
| GPIO internal pull-up / pull-down | | 40 | | kΩ | | ds p. 66 |
| GPIO VOL, 20 mA sink | | | 0.40 | V | VDD = 3.3 V | ds p. 66 |
| GPIO VOH, 8 mA source | 3.10 | | | V | VDD = 3.3 V | ds p. 66 |
| Run current IDD + IDDA | | 8.5 | | mA | 72 MHz from 8 MHz HXTAL, all peripherals on, 3.3 V | ds p. 52 |
| ESD, HBM / CDM | | | 6000 / 2000 | V | TA = 25 °C | ds p. 60 |
| θJA, TSSOP20 | | 67.24 | | °C/W | natural convection, 2S2P, simulated | ds p. 93 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD | VSS − 0.3 to VSS + 3.6 | V | ds p. 50 |
| VDDA | VSSA − 0.3 to VSSA + 3.6 | V | ds p. 50 |
| VIN on 5 V-tolerant pins | VSS − 0.3 to VDD + 3.6, never above 5.5 | V | ds p. 50 |
| VIN on other I/O | VSS − 0.3 to 3.6 | V | ds p. 50 |
| Difference between VDD and VDDA, power-up and operation | 300 max | mV | ds p. 50 |
| Variation between VDD pins / between ground pins | 50 / 50 | mV | ds p. 50 |
| Current per GPIO pin | ±25 | mA | ds p. 50 |
| Power dissipation, TSSOP20, TA = 85 °C (grade 6) | 595 | mW | ds p. 50 |
| Junction temperature | 125 | °C | ds p. 50 |
| Storage temperature | -65 to +150 | °C | ds p. 50 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 1.8 | 3.6 | V | ds p. 51 |
| VDDA, ADC not used | 1.8 | 3.6 | V | ds p. 51 |
| VDDA, ADC used | 2.4 | 3.6 | V | ds p. 51, ds p. 67 |
| VDD rise time rate | 0 | ∞ | µs/V | ds p. 51 |
| VDD fall time rate | 20 | ∞ | µs/V | ds p. 51 |
| Ambient temperature, grade 6 (suffix 6) | -40 | 85 | °C | ds p. 49, ds p. 50 |
| ADC input voltage | 0 | VDDA | V | ds p. 67 |

## Pins

Pin map from Figure 2-8 (ds p. 15) and Table 2-11 (ds pp. 35–36). Functions list the GD32E230F8 subset: USART0_CTS/RTS on PA0/PA1 and the USART0 functions on PA2–PA4 are F4-only and are omitted (ds p. 36, notes 3–5). "5VT" is the datasheet's 5 V-tolerant level.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | BOOT0 | BOOT0 | Input | Boot selection | ds p. 36 |
| 2 | PF0 | PF0/OSCIN | I/O, 5VT | GPIO PF0; I2C0_SDA; HXTAL OSCIN | ds p. 35 |
| 3 | PF1 | PF1/OSCOUT | I/O, 5VT | GPIO PF1; I2C0_SCL; HXTAL OSCOUT | ds p. 35 |
| 4 | NRST | NRST | I/O | Reset, active low, internal pull-up | ds p. 35 |
| 5 | VDDA | VDDA | Power | Analog supply | ds p. 35 |
| 6 | PA0 | PA0-WKUP | I/O | USART1_CTS, CMP_OUT, I2C1_SCL; ADC_IN0, CMP_IM6, RTC_TAMP1, WKUP0 | ds p. 35 |
| 7 | PA1 | PA1 | I/O | USART1_RTS/DE, I2C1_SDA, EVENTOUT; ADC_IN1, CMP_IP | ds p. 35 |
| 8 | PA2 | PA2 | I/O | USART1_TX; ADC_IN2, CMP_IM7 | ds p. 35 |
| 9 | PA3 | PA3 | I/O | USART1_RX; ADC_IN3 | ds p. 35 |
| 10 | PA4 | PA4 | I/O | SPI0_NSS, I2S0_WS, USART1_CK, TIMER13_CH0, SPI1_NSS; ADC_IN4, CMP_IM4 | ds p. 35 |
| 11 | PA5 | PA5 | I/O | SPI0_SCK, I2S0_CK; ADC_IN5, CMP_IM5 | ds p. 35 |
| 12 | PA6 | PA6 | I/O | SPI0_MISO, I2S0_MCK, TIMER2_CH0, TIMER0_BRKIN, TIMER15_CH0, CMP_OUT; ADC_IN6 | ds p. 35 |
| 13 | PA7 | PA7 | I/O | SPI0_MOSI, I2S0_SD, TIMER2_CH1, TIMER0_CH0_ON, TIMER16_CH0; ADC_IN7 | ds p. 35 |
| 14 | PB1 | PB1 | I/O | TIMER2_CH3, TIMER13_CH0, TIMER0_CH2_ON, SPI1_SCK; ADC_IN9 | ds p. 36 |
| 15 | VSS | VSS | Power | Ground (the only ground pin) | ds p. 36 |
| 16 | VDD | VDD | Power | Digital and I/O supply | ds p. 36 |
| 17 | PA9 | PA9 (PA11 by remap) | I/O, 5VT | USART0_TX, TIMER0_CH1, I2C0_SCL, CK_OUT | ds p. 36 |
| 18 | PA10 | PA10 (PA12 by remap) | I/O, 5VT | USART0_RX, TIMER0_CH2, TIMER16_BRKIN, I2C0_SDA | ds p. 36 |
| 19 | PA13 | PA13 | I/O, 5VT | SWDIO after reset; IFRP_OUT, SPI1_MISO | ds p. 36 |
| 20 | PA14 | PA14 | I/O, 5VT | SWCLK after reset; USART1_TX, SPI1_MOSI | ds p. 36 |

## Typical application

- Decoupling (Figure 4-1): one 4.7 µF plus 100 nF per VDD pin on VDD, and 1 µF plus 10 nF on VDDA, each as close to its pin as possible (ds p. 51). The TSSOP20 has one VDD pin, so this is 4.7 µF + 100 nF on pad 16 and 1 µF + 10 nF on pad 5; both return to VSS pad 15.
- Power VDD and VDDA from the same source: they may differ by at most 300 mV during power-up and operation (ds p. 50). VDDA and VSSA must be connected to VDD and VSS (ds p. 43).
- NRST: the recommended circuit is a 10 kΩ pull-up to VDD, 100 nF to GND and a push-button across the capacitor (Figure 4-4). The internal pull-up is 40 kΩ typical. An external reset circuit must pull NRST below VIL(NRST) (ds p. 65).
- BOOT0 (pad 1) selects one of three boot sources (main flash, system memory, SRAM); main flash is the default (ds p. 43). Strap it deliberately; the library declares it `required`.
- Bootloader: the system-memory loader reprograms flash over USART0 on PA9/PA10 (pads 17/18), and on GD32E230x8/6 also over USART1 on PA2/PA3 (pads 8/9) (ds p. 43).
- SWD: bring out PA13 (SWDIO, pad 19), PA14 (SWCLK, pad 20), NRST, VDD and VSS (ds p. 36).
- Optional HXTAL: a 4–32 MHz crystal on PF0/PF1 (pads 2/3) with CHXTAL1 = CHXTAL2 = 2 × (CLOAD − CS), where CLOAD is the crystal's specified load and CS the PCB plus pin stray capacitance (ds p. 60). Without a crystal the part runs from IRC8M (ds p. 42).

## Layout notes

- Place every decoupling capacitor as close as possible to its pin (ds p. 51).
- GigaDevice's recommended land (Figure 5-14) is twenty 1.20 × 0.33 mm pads at 0.65 mm pitch, with 5.90 mm between the two pad-row centres (ds p. 90). Its pad rows therefore span 4.70–7.10 mm.
- Package (Table 5-7): body E1 4.30–4.50 mm × D 6.40–6.60 mm, lead span E 6.20–6.60 mm (6.40 nominal), lead width b 0.20–0.28 mm, foot length L 0.45–0.75 mm, height A 1.20 mm max (ds p. 89). There is no exposed pad (ds p. 15, ds p. 89).
- The library land (`soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM`) is KiCad's generic TSSOP-20, not GigaDevice's: 1.475 × 0.40 mm pads with row centres at ±2.8625 mm, so the copper spans 4.25–7.20 mm. Its pin numbering is correct: pad 1 at the left end of one row, counterclockwise from the top, as in Figure 5-14 (ds p. 90).

## Pitfalls

- Only PF0, PF1, PA9, PA10, PA13 and PA14 are 5 V tolerant (5VT) (ds pp. 35–36). PA0–PA7 and PB1 are limited to 3.6 V absolute (ds p. 50). Even on 5VT pins the input may not exceed 5.5 V (ds p. 50).
- PA9/PA10 can be remapped to PA11/PA12 in SYSCFG_CFG0; they are the same two pads, not extra pins, and the part has no USB (ds p. 10, ds p. 36).
- When the part boots from system memory, the USART RX pins (PA3, PA10) are in input level detection mode and must be held at a stable level, or the loader may be falsely triggered (ds p. 43). Do not leave PA3 and PA10 floating if BOOT0 can be high.
- The TSSOP20 has no PC14/PC15 and no VBAT, so there is no LXTAL or backup domain on this package, although the series description lists them (ds p. 15, ds p. 43).
- The VDD absolute maximum is VSS + 3.6 V, the same as the operating maximum (ds p. 50, ds p. 51); do not run it from a rail that can overshoot 3.6 V.
- PA13/PA14 are SWD after reset (ds p. 36). Repurposing them as GPIO removes the debug port until the next reset.

## Not established by the sources

- **Land pattern.** The library uses KiCad's generic TSSOP-20 land, not GigaDevice's Figure 5-14 land (ds p. 90). Its copper reaches 0.225 mm further under the body (inner edge at ±2.125 mm against ±2.35 mm), extends 0.05 mm further out (±3.60 against ±3.55 mm) and is 0.07 mm wider (0.40 against 0.33 mm). `lib/@gigadevice/gd32/docs/gd32e230.md` documents this. The datasheet gives no criterion for accepting another land.
- **The BOOT0 truth table** and the option bytes that also affect boot are not in this datasheet; it names three boot sources and "boot pin" only (ds p. 43). No BOOT0 resistor value is given.
- **VSSA in the TSSOP20.** Figure 4-1 draws a VSSA pin (ds p. 51) and VSSA must be connected to VSS (ds p. 43), but the TSSOP20 has no VSSA pad (ds p. 15). The datasheet does not say that VSSA is bonded to VSS inside this package.
- **The application note's decoupling wording.** `gd32e230.md` says "4.7 uF plus 100 nF per VDD"; Figure 4-1 reads "4.7 μF + N * 100 nF", which is one 4.7 µF in total plus 100 nF per VDD pin (ds p. 51). On this one-VDD package the two readings agree.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`; nothing tells an agent that PA0–PA7 and PB1 must stay at or below 3.6 V.
- **No crystal ESR limit or drive-level guidance** is given; only the matching-capacitance formula (ds p. 60). The datasheet refers to AN074 for hardware design (ds p. 51), which the package does not pin.
- **IRC8M accuracy over temperature** is given only as a typical -0.525 to +0.275 % (ds p. 62), narrower than the ±1.0 % stated at 25 °C on the same page. The table does not explain this.
- **Other ordering codes are not modeled.** GD32E230F8P7TR (grade 7, -40 to +105 °C), F6P6TR, F6P7TR, F4P6TR and F4P7TR share this pinout (ds p. 96) but the library has only F8P6TR.
