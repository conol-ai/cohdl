# CH32V003F4P6 — QingKe V2A RISC-V MCU, 48 MHz, 16 KB flash, 2 KB SRAM, 2.7–5.5 V, TSSOP20

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@wch/ch32::CH32V003F4_TSSOP20",
  "parts": ["@wch/ch32::MCU_CH32V003F4P6"],
  "identities": [
    {"mfr": "WCH", "mpn": "CH32V003F4P6", "lcsc": "C5187096"}
  ],
  "sources": [
    {"id": "ds", "title": "CH32V003 Datasheet (CH32V003DS0)",
     "revision": "V1.8",
     "url": "https://www.wch-ic.com/downloads/CH32V003DS0_PDF.html",
     "sha256": "186210d35123325ea49305e3a0152cbfa6e377d8e6efc2e610e3497daabe17f8",
     "local": "lib/@wch/ch32/docs/ch32v003-v1.8.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are one-based PDF page indices, cover included (`pdftotext -f N -l N`). The printed page number is the PDF page minus two (the pinout in section 2.1 is PDF p. 12, printed 10). The URL is WCH's download page recorded in `lib/@wch/ch32/docs/sources.md`; it is an HTML page, not the PDF itself. The retained PDF came from the mirror `https://ch32-riscv-ug.github.io/CH32V003/datasheet_en/CH32V003DS0.PDF` because WCH's endpoint returned an error (`lib/@wch/ch32/docs/README.md`), so the SHA-256 has not been checked against a WCH-served copy.

## Summary

The CH32V003F4P6 is WCH's QingKe V2A (RV32EC) microcontroller at up to 48 MHz with 16 KB flash and 2 KB SRAM in a 4.4 × 6.5 mm TSSOP20, rated -40 to 85 °C (ds p. 1, ds p. 2, ds p. 32). It runs directly from 2.7–5.5 V, so it can sit on a 3.3 V or a 5 V rail, and has 18 GPIOs, one USART, one I2C, one SPI, a 10-bit ADC on 8 external channels and an op-amp/comparator (ds p. 1, ds p. 2, ds p. 7). It has no BOOT0 pin and no USB, and it is programmed and debugged over a one-wire SDI interface on PD1 that needs a WCH-compatible probe (ds p. 11, ds p. 14). The library declares no STM32 or GD32 alternates. Do not choose it when you need USB, more than 16 KB of code, or standard SWD/JTAG tooling.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| System clock | | | 48 | MHz | HCLK; Table 3-2 allows FHCLK up to 50 MHz | ds p. 1, ds p. 6, ds p. 17 |
| Flash / SRAM | | 16 / 2 | | KB | | ds p. 2 |
| GPIO count / ADC external channels | | 18 / 8 | | | CH32V003F4P6 | ds p. 2 |
| HSE crystal frequency | 4 | 24 | 25 | MHz | | ds p. 21 |
| HSE recommended load capacitance | | 20 | | pF | crystal RS = 60 Ω | ds p. 21 |
| HSE crystal ESR at 25 MHz | | | 60 | Ω | "recommended", may be relaxed below 25 MHz | ds p. 21 |
| HSE startup time | | 2 | | ms | 24 MHz crystal, VDD stable | ds p. 21 |
| HSI frequency (after calibration) | | 24 | | MHz | | ds p. 22 |
| HSI accuracy (after calibration) | -2.2 | | 2.2 | % | TA = -40 to 85 °C | ds p. 22 |
| LSI frequency | 100 | 128 | 150 | kHz | | ds p. 22 |
| POR/PDR threshold, rising | 2.32 | 2.5 | 2.68 | V | | ds p. 18 |
| POR/PDR threshold, falling | 2.3 | 2.48 | 2.66 | V | | ds p. 18 |
| Power-on reset time tRSTTEMPO | 1 | 1.5 | 21 | ms | RST_MODE can lengthen it | ds p. 18 |
| NRST input low level VIL | -0.3 | | 0.28 × (VDD − 1.8) + 0.6 | V | | ds p. 24 |
| NRST input high level VIH | 0.41 × (VDD − 1.8) + 1.3 | | VDD + 0.3 | V | | ds p. 24 |
| NRST internal pull-up | 35 | 45 | 55 | kΩ | | ds p. 24 |
| GPIO internal pull-up / pull-down | 35 | 45 | 55 | kΩ | | ds p. 23 |
| GPIO VOL, 8 pins sinking 8 mA | | | 0.4 | V | TTL or CMOS port | ds p. 23 |
| Run current | | 6.4 | | mA | 48 MHz from HSI, all peripherals on, VDD = 3.3 V | ds p. 19 |
| Run current | | 7.4 | | mA | 48 MHz from HSI, all peripherals on, VDD = 5 V | ds p. 19 |
| Standby current | | 9.1 | | µA | LSI on, VDD = 3.3 V | ds p. 20 |
| ESD, HBM | 4000 | | | V | | ds p. 17 |
| Flash endurance / retention | 10K / 10 | | | cycles / years | TA = 25 °C for endurance | ds p. 22 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD − VSS | -0.3 to 5.5 | V | ds p. 16 |
| VIN on FT (5 V-tolerant) pins | VSS − 0.3 to 5.5 | V | ds p. 16 |
| VIN on other pins | VSS − 0.3 to VDD + 0.3 | V | ds p. 16 |
| Variation between supply pins / between ground pins | 50 / 50 | mV | ds p. 16 |
| Total current into VDD / out of VSS | 100 / 80 | mA | ds p. 17 |
| Sink / source current on any I/O or control pin | 20 / -20 | mA | ds p. 17 |
| Injected current per pin (OSC_IN and other pins) / total | ±4 / ±20 | mA | ds p. 17 |
| Ambient temperature during operation | -40 to 85 | °C | ds p. 16 |
| Storage temperature | -40 to 125 | °C | ds p. 16 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VDD, ADC not used | 2.7 | 5.5 | V | ds p. 17 |
| VDD, ADC used (recommended) | 2.8 | 5.5 | V | ds p. 17 |
| VDD for flash programming | 2.8 | 5.5 | V | ds p. 22 |
| VDD rise time rate | 0 | ∞ | µs/V | ds p. 17 |
| VDD fall time rate | 20 | ∞ | µs/V | ds p. 17 |
| Internal AHB clock FHCLK | | 50 | MHz | ds p. 17 |
| Ambient temperature | -40 | 85 | °C | ds p. 17 |
| Junction temperature | -40 | 105 | °C | ds p. 17 |

## Pins

Pin map from the CH32V003F4P6 drawing in section 2.1 (ds p. 12, rendered and checked) and Table 2-1 (ds pp. 13–14). "Default alternate" functions need no remap; "remap" functions need the AFIO setting shown in Table 2-1. FT = 5 V tolerant, A = analog (ds p. 14).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | PD4 | PD4 | I/O/A | UCK, T2CH1ETR, A7 (ADC_IN7), OPO | ds p. 13 |
| 2 | PD5 | PD5 | I/O/A | UTX (USART TX), A5; remap URX | ds p. 13 |
| 3 | PD6 | PD6 | I/O/A | URX (USART RX), A6; remap UTX | ds p. 13 |
| 4 | PD7 | PD7 | I/O/A | GPIO after reset; NRST, T2CH4, OPP1 | ds p. 13 |
| 5 | PA1 | PA1 | I/O/A | T1CH2, A1, OPN0; OSCI by remap | ds p. 13 |
| 6 | PA2 | PA2 | I/O/A | T1CH2N, A0, OPP0; OSCO by remap | ds p. 13 |
| 7 | VSS | VSS | Power | Ground | ds p. 13 |
| 8 | PD0 | PD0 | I/O/A | T1CH1N, OPN1; remap SDA, UTX | ds p. 13 |
| 9 | VDD | VDD | Power | Supply, 2.7–5.5 V | ds p. 13 |
| 10 | PC0 | PC0 | I/O | T2CH3; remap NSS, UTX | ds p. 14 |
| 11 | PC1 | PC1 | I/O/FT | SDA (I2C), NSS (SPI) | ds p. 14 |
| 12 | PC2 | PC2 | I/O/FT | SCL (I2C), URTS, T1BKIN | ds p. 14 |
| 13 | PC3 | PC3 | I/O | T1CH3 | ds p. 14 |
| 14 | PC4 | PC4 | I/O/A | T1CH4, MCO, A2 | ds p. 14 |
| 15 | PC5 | PC5 | I/O/FT | SCK (SPI), T1ETR | ds p. 14 |
| 16 | PC6 | PC6 | I/O/FT | MOSI (SPI) | ds p. 14 |
| 17 | PC7 | PC7 | I/O | MISO (SPI) | ds p. 14 |
| 18 | PD1 | PD1 | I/O/A | SWIO (one-wire SDI debug), T1CH3N, AETR2 | ds p. 14 |
| 19 | PD2 | PD2 | I/O/A | T1CH1, A3 | ds p. 14 |
| 20 | PD3 | PD3 | I/O/A | A4, T2CH2, AETR, UCTS | ds p. 14 |

## Typical application

- Decoupling: one 0.1 µF from VDD to VSS (Figure 3-1 and Figure 3-11) (ds p. 16, ds p. 30). No bulk capacitor is shown.
- Reset: the typical NRST circuit is a 0.1 µF capacitor from NRST to ground with the internal pull-up (Figure 3-5) (ds p. 24). On this package NRST is a function of PD7, pad 4 (ds p. 13).
- Debug and programming: PD1 (pad 18) is SWIO, the one-wire SDI interface. It is enabled after power-on or reset, and HSI must stay on while it is used (ds p. 11). Bring out PD1, VDD and VSS.
- Clock: the part runs from the 24 MHz HSI by default (ds p. 9). An optional 4–25 MHz crystal goes on PA1/PA2 (pads 5/6) with CL1 = CL2 as the crystal maker recommends (Figure 3-4) (ds p. 21).
- Default peripheral pins: USART TX/RX on PD5/PD6 (pads 2/3), I2C SDA/SCL on PC1/PC2 (pads 11/12), SPI SCK/MOSI/MISO/NSS on PC5/PC6/PC7/PC1 (pads 15/16/17/11) (ds pp. 13–14).

## Layout notes

- Package (section 4.1): 4.4 × 6.5 mm body, 6.4 mm lead span, 0.65 mm pitch, 0.25 mm lead width, 0.6 mm foot, 1.1 mm height. All dimensions except pitch carry a tolerance of "the greater of ±0.2 mm or 10%" (ds p. 33). There is no exposed pad on the TSSOP20; only the QFN20 has pad 0 = VSS (ds p. 12, ds p. 13).
- The datasheet gives no recommended land pattern. The library uses KiCad's generic TSSOP-20 (`soic::KICAD_TSSOP_20_4_4X6_5MM_P0_65MM`: 1.475 × 0.40 mm pads, row centres ±2.8625 mm). Its numbering matches the drawing: pin 1 top left, 1–10 down the left side and 11–20 up the right side (ds p. 33), rotated without mirroring so pad 1 is at the left end of one row.
- Keep the 0.1 µF capacitor at the VDD/VSS pins (pads 9/7).

## Pitfalls

- Only PC1, PC2, PC5 and PC6 are FT (5 V tolerant) (ds p. 14). Every other pin is limited to VDD + 0.3 V (ds p. 16). On a 3.3 V supply, a 5 V signal on PD5/PD6 (USART) is outside the absolute maximum.
- The VDD absolute maximum (5.5 V) equals the operating maximum (ds p. 16, ds p. 17). A 5 V rail with overshoot has no margin.
- I/O levels follow VDD (ds p. 11): on a 5 V supply the outputs swing to 5 V and will overdrive 3.3 V parts.
- The POR threshold is 2.3–2.68 V (ds p. 18), close to the 2.7 V minimum supply; the ADC is recommended only from 2.8 V and its performance degrades below 2.9 V (ds p. 7, ds p. 17).
- OSCI/OSCO are remap functions of PA1/PA2, not their defaults (ds p. 13, ds p. 15).
- Do not route SWIO (PD1) through anything that loads it if you need to reprogram the part; it is the only debug pin (ds p. 11).

## Not established by the sources

- **NRST default on PD7.** Table 2-1 gives PD7 as the main function after reset and NRST as its default alternate function (ds p. 13). The datasheet does not say whether the NRST function is active out of reset or how to enable/disable it; it mentions only that the RST_MODE configuration bit can lengthen the power-on reset delay (ds p. 18). `lib/@wch/ch32/docs/ch32v003.md` says "reset behavior depends on the option-byte configuration"; this datasheet does not state that.
- **Bootloader interface.** The part has a 1920-byte bootloader and "supports Boot and user code jumping to each other" (ds p. 7), but this datasheet does not name the interface, the entry method or any pins. Without a BOOT0 pin, recovery relies on the SDI port.
- **Maximum clock.** The overview and clock tree give 48 MHz (ds p. 1, ds p. 6); Table 3-2 gives FHCLK max 50 MHz and the block diagram "AHB Fmax = 50MHz" (ds p. 3, ds p. 17). The library intent uses 48 MHz.
- **Land pattern and tolerances.** No land pattern is given, and the "±0.2 mm or 10%" note (ds p. 33) is broad: 10% of the 6.4 mm lead span is ±0.64 mm. The library's generic land is checked against nominal dimensions only, as `ch32v003.md` says.
- **Source revision labels.** The cover and chapter 4 say V1.8, but the page footers of pp. 15–31 say V1.4 or V1.7 (ds p. 15, ds p. 16, ds p. 31). The retained PDF was taken from a mirror, and its identity with WCH's own download is not established.
- **No crystal drive level and no decoupling beyond one 0.1 µF** are given (ds p. 16, ds p. 21).
- **The 5 V-tolerant pins are not marked in the library.** All GPIOs are typed `bidirectional`.
- **No thermal resistance** is given for any package.
- **Other packages are not modeled.** CH32V003F4U6 (QFN20), A4M6 (SOP16) and J4M6 (SOP8) have other pin maps (ds p. 12, ds p. 13, ds p. 32); the library has only the TSSOP20.
