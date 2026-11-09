# CH32V203C8T6 — QingKe V4B RISC-V MCU, 144 MHz, 64 KB zero-wait flash, 20 KB SRAM, two USB FS ports and CAN, LQFP48 7 × 7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@wch/ch32::CH32V203C8_LQFP48",
  "parts": ["@wch/ch32::MCU_CH32V203C8T6"],
  "identities": [
    {"mfr": "WCH", "mpn": "CH32V203C8T6", "lcsc": "C3001172"}
  ],
  "sources": [
    {"id": "ds", "title": "CH32V203 Datasheet (CH32V203DS0)",
     "revision": "V3.0",
     "url": "https://www.wch-ic.com/downloads/CH32V203DS0_PDF.html",
     "sha256": "10af5de2779ce1e985162dceae2e829800b2d633d34491333b1ce90f4ffcbdd2",
     "local": "lib/@wch/ch32/docs/ch32v203-v3.0.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are one-based PDF page indices, cover included (`pdftotext -f N -l N`). The printed numbers are not a fixed offset: they are the PDF page minus two up to p. 35, but pp. 36–37 print 31–32 again, so from p. 36 on the printed number is the PDF page minus five (Table 4-1 is PDF p. 38, printed 33). The URL is WCH's download page recorded in `lib/@wch/ch32/docs/sources.md`; it is an HTML page, not the PDF. The retained PDF came from the mirror `https://raw.githubusercontent.com/ch32-riscv-ug/CH32V20x/main/datasheet_en/CH32V203DS0.PDF` because WCH's endpoint returned an error (`lib/@wch/ch32/docs/README.md`), so the SHA-256 has not been checked against a WCH-served copy.

## Summary

The CH32V203C8T6 is WCH's QingKe V4B (RV32IMAC) microcontroller at up to 144 MHz in a 7 × 7 mm LQFP48, rated -40 to 85 °C (ds p. 7, ds p. 39, ds p. 59). It has 64 KB of zero-wait flash, 20 KB SRAM, 37 GPIOs, a USB 2.0 full-speed device (USBD), a second USB full-speed host/device port (USBFS), one CAN, four USARTs, two I2Cs, two SPIs and two 12-bit ADCs on 10 external channels (ds p. 7). It needs a 2.4–3.6 V main supply (3.0–3.6 V when USB is used), a separate I/O supply VIO that may not exceed VDD, VDDA equal to VIO, and VBAT for the backup domain (ds pp. 13–14, ds p. 39). Debug uses WCH's two-wire SDI on PA13/PA14 and needs a WCH-compatible probe (ds p. 20). The library declares no STM32 or GD32 alternates. Do not choose it if you need a supply above 3.6 V or standard Arm SWD tooling.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| HB / PB1 / PB2 clock | | | 144 | MHz | | ds p. 39 |
| Zero-wait flash / SRAM | | 64 / 20 | | KB | C8x6; non-zero-wait area is 224K minus the zero-wait area | ds p. 7 |
| GPIO count / ADC external channels | | 37 / 10 | | | C8x6, 2 ADC units | ds p. 7 |
| HSE crystal frequency | 3 | 8 | 25 | MHz | | ds p. 46 |
| HSE recommended load capacitance | | 20 | | pF | crystal RS = 60 Ω | ds p. 46 |
| HSE crystal ESR | | | 80 | Ω | "recommended" | ds p. 46 |
| HSE startup time | | 2.5 | | ms | 8 MHz crystal, HSEON to HSERDY | ds p. 46 |
| LSE load capacitance | | | 15 | pF | 32.768 kHz, RS < 70 kΩ | ds p. 46 |
| LSE startup time | | 800 | | ms | VDD stable | ds p. 47 |
| HSI frequency (after calibration) | | 8 | | MHz | | ds p. 47 |
| HSI accuracy (after calibration) | -2.2 | | 2.2 | % | full operating temperature range | ds p. 47 |
| PLL output | 18 | | 144 | MHz | input 3–25 MHz | ds p. 48 |
| POR/PDR threshold, rising and falling | 1.9 | 2.2 | 2.4 | V | | ds p. 40 |
| Power-on reset time tRSTTEMPO | 24 | 28 | 30 | ms | | ds p. 40 |
| NRST input low level VIL | -0.3 | | 0.28 × (VIO − 1.8) + 0.6 | V | | ds p. 51 |
| NRST input high level VIH | 0.41 × (VIO − 1.8) + 1.3 | | VIO + 0.3 | V | | ds p. 51 |
| NRST internal pull-up | 30 | 40 | 50 | kΩ | | ds p. 51 |
| NRST filtered / not-filtered pulse width | | | 100 / — | ns | not filtered: 300 ns min | ds p. 51 |
| GPIO internal pull-up / pull-down | 30 | 40 | 50 | kΩ | | ds p. 50 |
| FT pin input high level VIH | 0.42 × (V* − 1.8) + 1 | | 5.0 | V | | ds p. 49 |
| USB operating VDD | 3.0 | | 3.6 | V | | ds p. 55 |
| Run current | | 11.8 | | mA | 144 MHz from external clock, all peripherals on, VDD = 3.3 V | ds p. 41 |
| ESD, HBM: all pins / USB pins PA11, PA12 | 4000 / 3000 | | | V | | ds p. 39 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD − VSS, including VDDA | -0.3 to 4.0 | V | ds p. 38 |
| VIO − VSS | -0.3 to 4.0 | V | ds p. 39 |
| VIN on FT (5 V-tolerant) pins | VSS − 0.3 to 5.5 | V | ds p. 39 |
| VIN on other pins | VSS − 0.3 to VDD + 0.3 | V | ds p. 39 |
| Variation between VDD pins / VIO pins / ground pins | 50 / 50 / 50 | mV | ds p. 39 |
| Total current into VDD/VDDA/VIO / out of VSS | 150 / 150 | mA | ds p. 39 |
| Sink / source current on any I/O or control pin | 25 / -25 | mA | ds p. 39 |
| Injected current per pin (NRST, OSC_IN, others) / total | ±5 / ±25 | mA | ds p. 39 |
| Ambient temperature during operation, suffix 6 | -40 to 85 | °C | ds p. 38 |
| Storage temperature | -40 to 125 | °C | ds p. 38 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD | 2.4 | 3.6 | V | ds p. 39 |
| VDD when USB is used | 3.0 | 3.6 | V | ds p. 39, ds p. 55 |
| VIO (not higher than VDD) | 2.4 | 3.6 | V | ds p. 39 |
| VDDA (equal to VIO; VREF+ not above VDDA) | 2.4 | 3.6 | V | ds p. 39 |
| VBAT | 1.8 | 3.6 | V | ds p. 39 |
| VDD rise time rate | 0 | ∞ | µs/V | ds p. 40 |
| VDD fall time rate | 20 | ∞ | µs/V | ds p. 40 |
| HB clock FHCLK | | 144 | MHz | ds p. 39 |
| Ambient temperature, suffix 6 | -40 | 85 | °C | ds p. 39 |
| Junction temperature, suffix 6 | -40 | 105 | °C | ds p. 39 |

## Pins

Pin map from the CH32V203CxT6 drawing in section 3.1 (ds p. 21, rendered and checked) and Table 3-1-1, LQFP48/QFN48X7 column (ds pp. 23–26). Remap functions need AFIO configuration (ds p. 35, note 8). FT = 5 V tolerant (ds p. 34). The drawing labels pads 24 and 48 "VDD_VIO_1" and "VDD_VIO_3"; the table and the library use VDD_IO_1 and VDD_IO_3. Pads 1–4 are in the VDD/VBAT power domain (ds p. 21).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VBAT | VBAT | Power | Backup-domain supply | ds p. 23 |
| 2 | PC13 | PC13-TAMPER-RTC | I/O | PC13; TAMPER-RTC | ds p. 23 |
| 3 | PC14 | PC14-OSC32_IN | I/O/A | PC14; LSE OSC32_IN | ds p. 23 |
| 4 | PC15 | PC15-OSC32_OUT | I/O/A | PC15; LSE OSC32_OUT | ds p. 23 |
| 5 | PD0 | OSC_IN | I/A | HSE OSC_IN after reset; PD0 by software | ds p. 23, ds p. 35 |
| 6 | PD1 | OSC_OUT | O/A | HSE OSC_OUT after reset; PD1 by software | ds p. 23, ds p. 35 |
| 7 | NRST | NRST | I | Reset, active low, internal pull-up | ds p. 23 |
| 8 | VSSA | VSSA | Power | Analog ground | ds p. 23 |
| 9 | VDDA | VDDA | Power | Analog supply | ds p. 23 |
| 10 | PA0 | PA0-WKUP | I/O/A | WKUP, USART2_CTS, ADC_IN0, TIM2_CH1/ETR | ds p. 23 |
| 11 | PA1 | PA1 | I/O/A | USART2_RTS, ADC_IN1, TIM2_CH2 | ds p. 23 |
| 12 | PA2 | PA2 | I/O/A | USART2_TX, ADC_IN2, TIM2_CH3, OPA2_OUT0 | ds p. 23 |
| 13 | PA3 | PA3 | I/O/A | USART2_RX, ADC_IN3, TIM2_CH4, OPA1_OUT0 | ds p. 23 |
| 14 | PA4 | PA4 | I/O/A | SPI1_NSS, USART2_CK, ADC_IN4, OPA2_OUT1 | ds p. 24 |
| 15 | PA5 | PA5 | I/O/A | SPI1_SCK, ADC_IN5, OPA2_CH1N | ds p. 24 |
| 16 | PA6 | PA6 | I/O/A | SPI1_MISO, ADC_IN6, TIM3_CH1, OPA1_CH1N | ds p. 24 |
| 17 | PA7 | PA7 | I/O/A | SPI1_MOSI, ADC_IN7, TIM3_CH2, OPA2_CH1P | ds p. 24 |
| 18 | PB0 | PB0 | I/O/A | ADC_IN8, TIM3_CH3, OPA1_CH1P, USART4_TX | ds p. 24 |
| 19 | PB1 | PB1 | I/O/A | ADC_IN9, TIM3_CH4, OPA1_OUT1, USART4_RX | ds p. 24 |
| 20 | PB2 | PB2 | I/O, FT | PB2; BOOT1; USART4_CK | ds p. 24 |
| 21 | PB10 | PB10 | I/O/A, FT | I2C2_SCL, USART3_TX, OPA2_CH0N | ds p. 24 |
| 22 | PB11 | PB11 | I/O/A, FT | I2C2_SDA, USART3_RX, OPA1_CH0N | ds p. 24 |
| 23 | VSS_1 | VSS_1 | Power | Ground | ds p. 24 |
| 24 | VDD_IO_1 | VDD_IO_1 | Power | I/O supply VIO | ds p. 24 |
| 25 | PB12 | PB12 | I/O/A, FT | SPI2_NSS, I2C2_SMBA, USART3_CK, TIM1_BKIN | ds p. 24 |
| 26 | PB13 | PB13 | I/O/A, FT | SPI2_SCK, USART3_CTS, TIM1_CH1N | ds p. 25 |
| 27 | PB14 | PB14 | I/O/A, FT | SPI2_MISO, TIM1_CH2N, USART3_RTS, OPA2_CH0P | ds p. 25 |
| 28 | PB15 | PB15 | I/O/A, FT | SPI2_MOSI, TIM1_CH3N, OPA1_CH0P | ds p. 25 |
| 29 | PA8 | PA8 | I/O, FT | USART1_CK, TIM1_CH1, MCO | ds p. 25 |
| 30 | PA9 | PA9 | I/O, FT | USART1_TX, TIM1_CH2 | ds p. 25 |
| 31 | PA10 | PA10 | I/O, FT | USART1_RX, TIM1_CH3 | ds p. 25 |
| 32 | PA11 | PA11 | I/O/A, FT | USBDM (USBD), CAN1_RX, USART1_CTS, TIM1_CH4 | ds p. 25 |
| 33 | PA12 | PA12 | I/O/A, FT | USBDP (USBD), CAN1_TX, USART1_RTS, TIM1_ETR | ds p. 25 |
| 34 | PA13 | PA13 | I/O, FT | SWDIO (SDI) after reset; PA13 by remap | ds p. 25 |
| 35 | VSS_2 | VSS_2 | Power | Ground | ds p. 25 |
| 36 | VDD_2 | VDD_2 | Power | Main supply VDD | ds p. 25 |
| 37 | PA14 | PA14 | I/O, FT | SWCLK (SDI) after reset; PA14 by remap | ds p. 25 |
| 38 | PA15 | PA15 | I/O, FT | PA15; remap TIM2_CH1/ETR, SPI1_NSS, USART4_RTS | ds p. 25 |
| 39 | PB3 | PB3 | I/O, FT | USART4_CTS; remap TIM2_CH2, SPI1_SCK | ds p. 25 |
| 40 | PB4 | PB4 | I/O, FT | USART4_RTS; remap TIM3_CH1, SPI1_MISO | ds p. 25 |
| 41 | PB5 | PB5 | I/O, FT | I2C1_SMBA; remap TIM3_CH2, SPI1_MOSI, USART4_RX | ds p. 26 |
| 42 | PB6 | PB6 | I/O, FT | I2C1_SCL, TIM4_CH1, USBFS_DM; remap USART1_TX | ds p. 26 |
| 43 | PB7 | PB7 | I/O, FT | I2C1_SDA, TIM4_CH2, USBFS_DP; remap USART1_RX | ds p. 26 |
| 44 | BOOT0 | BOOT0 | I | Boot selection | ds p. 26 |
| 45 | PB8 | PB8 | I/O/A, FT | TIM4_CH3; remap I2C1_SCL, CAN1_RX | ds p. 26 |
| 46 | PB9 | PB9 | I/O/A, FT | TIM4_CH4; remap I2C1_SDA, CAN1_TX | ds p. 26 |
| 47 | VSS_3 | VSS_3 | Power | Ground | ds p. 26 |
| 48 | VDD_IO_3 | VDD_IO_3 | Power | I/O supply VIO | ds p. 26 |

## Typical application

- Decoupling (Figure 4-1): 0.1 µF on each VDDx/VIOx pin to its VSSx, and 0.1 µF from VDDA to VSSA (ds p. 38). No bulk capacitor is shown.
- Supplies: VDD (pad 36) 2.4–3.6 V; VIO (pads 24, 48) 2.4–3.6 V and never above VDD; VDDA (pad 9) equal to VIO, and kept live with VIO if VDD is off (ds pp. 13–14, ds p. 39). One common 3.3 V rail meets all three rules; with USB, VDD must be 3.0–3.6 V (ds p. 39).
- VBAT (pad 1): 1.8–3.6 V optional backup supply for the RTC, LSE and backup registers; keep the battery trace short (ds p. 14, ds p. 39). Figure 4-1 shows a 1.8–3.6 V source on VBAT (ds p. 38).
- NRST (pad 7): 0.1 µF from NRST to ground with the internal pull-up (Figure 4-7) (ds p. 52).
- Boot: BOOT0 (pad 44) and BOOT1 on PB2 (pad 20) select program flash, system memory or SRAM. The system-memory bootloader reprograms flash over USART1 and USB (ds p. 13).
- Debug: two-wire SDI on PA13 = SWDIO (pad 34) and PA14 = SWCLK (pad 37), enabled after power-on or reset (ds p. 20, ds p. 25).
- USB: USBD on PA11/PA12 (pads 32/33) and USBFS on PB6/PB7 (pads 42/43) (ds pp. 25–26). The 48 MHz USB clock is divided from the PLL, which must run at 144, 96 or 48 MHz for USBFS (ds p. 19).
- HSE: 3–25 MHz crystal on pads 5/6 with CL1 = CL2 per the crystal maker (Figure 4-5); ESR at most 80 Ω recommended (ds p. 46).
- LSE: 32.768 kHz crystal on PC14/PC15 (pads 3/4); CL1 = CL2, usually about 12 pF, with CL = CL1·CL2/(CL1 + CL2) + Cstray and Cstray typically 2–7 pF (ds p. 47).
- CAN1 on PA11/PA12 or, by remap, PB8/PB9 (ds p. 25, ds p. 26) needs an external transceiver.

## Layout notes

- Package (section 5.7): 7.00 mm body, 9.00 mm lead span, 0.5 mm pitch, 0.2 mm lead width, 0.6 mm foot, 0.1 mm standoff (ds p. 63). All dimensions except pitch carry a tolerance of "the greater of ±0.2 mm or 10%" (ds p. 60). There is no exposed pad on the LQFP48.
- The datasheet gives no recommended land pattern. The library uses KiCad's generic LQFP-48 (`qfp::KICAD_LQFP_48_7X7MM_P0_5MM`: 1.475 × 0.3 mm pads at row centres ±4.1625 mm). The section 5.7 drawing already has pin 1 at the left end of the bottom row with counterclockwise numbering (ds p. 63), the same as the library land.
- Keep the battery-to-VBAT connection as short as possible (ds p. 39).

## Pitfalls

- Not every pin is 5 V tolerant. PC13–PC15, the oscillator pins, NRST, BOOT0, PA0–PA7, PB0 and PB1 are not FT (ds pp. 23–26) and are limited to VDD + 0.3 V (ds p. 39). On FT pins the operating VIH maximum is 5.0 V (ds p. 49), below the 5.5 V absolute maximum (ds p. 39).
- VIO must never exceed VDD in operation, and VDDA must follow VIO (ds pp. 13–14). Do not feed VDD_IO_1/VDD_IO_3 from a different rail than VDD_2 unless the sequencing guarantees this.
- PC13–PC15 are supplied through a 3 mA analog switch: as outputs they work only in 2 MHz mode with at most 30 pF, and they cannot drive LEDs. On VBAT power, PC14/PC15 are LSE-only (ds pp. 34–35). Their state after reset is held by the backup domain (ds p. 35).
- Pads 5/6 are OSC_IN/OSC_OUT after reset; using them as PD0/PD1 needs software reconfiguration (ds p. 35).
- USBD and CAN share a 512-byte SRAM; when both are used, USBD may use only the lower 384 bytes (ds p. 19).
- The power-on reset delay is 24–30 ms (ds p. 40).
- PA11/PA12 and PB6/PB7 are both USB pairs; the library comments label PA11/PA12 as USBDM/USBDP and PB6/PB7 as USBFS_DM/USBFS_DP, matching Table 3-1-1 (ds pp. 25–26). The pinout drawing calls them USB1 and USB2 (ds p. 21).

## Not established by the sources

- **Supply range on the first page.** The feature list says "System power supply VDD range: 2.4~6V" (ds p. 1). The absolute maximum is 4.0 V and the operating range 2.4–3.6 V (ds p. 38, ds p. 39). An agent that reads only page 1 would destroy the part on a 5 V rail. Use 3.6 V.
- **USB without a crystal.** The USB clock is divided from the PLL (ds p. 19), and the PLL can run from HSI (ds p. 41), whose accuracy is ±2.2 % over temperature (ds p. 47). The datasheet does not say whether USB meets its timing from HSI or needs the HSE crystal. It also does not say whether the USB ports have internal D+ pull-ups.
- **USB supply in the application note.** `lib/@wch/ch32/docs/ch32v203.md` gives "VDD and VIO operate at 2.4-3.6 V" without the 3.0 V minimum that applies when USB is used (ds p. 39, ds p. 55).
- **The BOOT0/BOOT1 truth table** and any BOOT0 resistor value for the LQFP48 are not given; the 4.7 kΩ pull-down in note 6 applies only to packages where BOOT0 shares a pin with PB8/PB9 (ds p. 13, ds p. 35).
- **VBAT with no battery.** VBAT is called "optional" (ds p. 14), but the datasheet does not say what to connect when no battery is fitted. The library declares VBAT `required`; tying it to VDD is a common choice but is not stated here.
- **Flash size in the library.** The device intent says "64KB flash". That is the zero-wait area; the C8x6 also has a non-zero-wait area of 224 KB minus the zero-wait area (ds p. 7, note 1).
- **Land pattern and tolerances.** No land pattern is given, and the "±0.2 mm or 10%" tolerance note (ds p. 60) is broad. The library's generic land is checked against nominal dimensions only, as `ch32v203.md` says.
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- **No thermal resistance** and no maximum power dissipation are given.
- **"Other resets" tRSTTEMPO** is given as 8/10/30 with the unit "mS" (ds p. 40). The source does not confirm whether this is milliseconds.
- **Source identity.** The retained PDF was taken from a mirror and its identity with WCH's own download is not established. Printed page numbers 31–33 appear twice: on PDF pp. 33–35 and again on pp. 36–38 (ds p. 33, ds p. 36).
- **Other parts are not modeled.** CH32V203C6T6 (32 KB, 10 KB SRAM) shares this LQFP48 (ds p. 7, ds p. 59) but the library has only C8T6.
