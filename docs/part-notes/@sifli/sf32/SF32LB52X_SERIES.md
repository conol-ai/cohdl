# SF32LB52X series — dual Cortex-M33 (192 MHz + 24 MHz) BT/BLE 5.3 MCU with 2.5D GPU and audio codec, QFN68L 7 × 7 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@sifli/sf32::SF32LB52X",
  "parts": ["@sifli/sf32::MCU_SF32LB52EUB6", "@sifli/sf32::MCU_SF32LB52GUC6"],
  "identities": [
    {"mfr": "SiFli Technologies", "mpn": "SF32LB52EUB6", "lcsc": null},
    {"mfr": "SiFli Technologies", "mpn": "SF32LB52GUC6", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "SF32LB52X 芯片技术规格书 (DS0052-SF32LB52X-CN, Chinese)",
     "revision": "V0.1, 2024-11-20 (draft, 初稿)",
     "url": "https://www.sifli.com",
     "sha256": "68ad87f5846c47908614e980eaa56b0eea5e4ae036d0a5cee9357f80db7357c2",
     "local": "lib/@sifli/sf32/docs/ds0052-sf32lb52x-cn.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices (`pdftotext -f N -l N`). The PDF has eight roman-numbered front pages, so the PDF page is the printed arabic page + 10 (Figure 5-1 is printed page 38/51, PDF page 48). The source is in Chinese; translations below are the note author's.

**Source URL flag.** No official document URL is known for this file. `lib/@sifli/sf32/docs/README.md` records only the home page `https://www.sifli.com` and says the PDF came "from an authorized source" because SiFli's server blocked automated download. The URL above is that home page, not a link to this PDF; the SHA-256 is of the shipped file.

**Coverage flag.** Only SF32LB52EUB6 is in this source's ordering table (Table 5-7, ds p. 60). SF32LB52GUC6 is not, so nothing in this note is established for it beyond the shared pinout that the library assumes (see "Not established by the sources"). No LCSC number is mapped for either part in `lcsc_map.json`.

## Summary

SF32LB52X is SiFli's low-power AIoT MCU family: a 192 MHz Cortex-M33 STAR-MC1 "big" core with 512 KB SRAM and a 24 MHz "little" core with 64 KB SRAM, dual-mode Bluetooth 5.3, a 2D/2.5D GPU and LCD controller, and a 24-bit audio ADC and DAC (ds p. 3, ds p. 4). All variants share one QFN68L 7 × 7 × 0.85 mm package with 0.35 mm pitch and 45 GPIOs, and differ in supply voltage and in-package memory (ds p. 5, ds p. 60). SF32LB52EUB6 runs PVDD at 3.3 V and has 32 Mb pSRAM in the package; this source lists no in-package flash for it (ds p. 60). An on-chip PMU with a buck converter and LDOs makes the internal rails, so the board must supply the buck inductor and decouple the LDO output pins (ds p. 19). This V0.1 datasheet is a preliminary draft without any application circuit or land pattern (ds p. 2), so a board needs SiFli's hardware design guide, which the package does not ship.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| HCPU / LCPU clock | | | 192 / 24 | MHz | Cortex-M33 STAR-MC1 | ds p. 4 |
| HCPU / LCPU SRAM | | 512 / 64 | | KB | All retention SRAM | ds p. 4 |
| GPIOs | | | 45 | | | ds p. 4, ds p. 25 |
| In-package memory, SF32LB52EUB6 | | 32 Mb pSRAM | | | PVDD 3.3 V | ds p. 60 |
| Supply range (feature list) | 1.71 | | 3.63 | V | | ds p. 5 |
| VBAT measurement input range | 0 | | 4.7 | V | VBATS | ds p. 5 |
| Shutdown current | | 2.0 | | µA | Key wake-up, @3.3 V | ds p. 40 |
| CoreMark current, HCPU | | 7360 | | µA | 192 MHz, @3.3 V | ds p. 40 |
| BT/BLE sleep current | | 18 | | µA | @3.3 V | ds p. 40 |
| BLE sensitivity / max TX power | | -100 / 19 | | dBm | 1 Mbps; TX max BR/BLE (13 dBm EDR2/3) | ds p. 4 |
| VIH / VIL | 0.7 × VDDIOA | | 0.3 × VDDIOA | V | VIH min / VIL max; 3.3 V, 25 °C | ds p. 38 |
| VOH / VOL | 0.8 × VDDIOA | | 0.2 × VDDIOA | V | VOH min / VOL max; high-impedance load | ds p. 38 |
| IOH / IOL | | 12 | | mA | Max drive setting | ds p. 38 |
| Drive strength settings | | 2 / 4 / 8 / 12 | | mA | @3.3 V; 1 / 2 / 4 / 6 mA @1.8 V | ds p. 46, ds p. 47 |
| Internal pull-up / pull-down | | 10 | | kΩ | | ds p. 38 |
| Pin capacitance / input current | | 3 / 10 | 40 | pF / nA | IIH, IIL max 40 nA | ds p. 38 |
| GPADC resolution / max rate | | 12 | 4 | bit / Msps | | ds p. 23, ds p. 46 |
| GPADC input range, single-ended / differential | 0 / -2.1 | | 3.3 / 2.1 | V | 7 single-ended inputs + battery, or 3 differential pairs | ds p. 23 |
| GPADC ENOB, single-ended | | 10 | | bit | Vin = -3 dBFS, no averaging | ds p. 46 |
| MIC_BIAS output | 1.4 | | 2.8 | V | 0–2 mA | ds p. 4 |
| Audio DAC output swing | | 1.1 | | Vrms | | ds p. 45 |
| Temperature sensor range / accuracy | -40 / -3 | | 125 / 3 | °C | | ds p. 23 |
| ESD HBM / CDM | | ±4000 / ±1000 | | V | JS-001-2017 / JS-002-2018 | ds p. 39 |
| Moisture sensitivity | | MSL3 | | | Reflow 260 °C, 3 times | ds p. 39 |
| Package body D × E | | 7 × 7 | | mm | BSC | ds p. 58 |
| Exposed pad D2 = E2 | 5.39 | 5.49 | 5.59 | mm | | ds p. 58 |
| Lead width b / pitch e | 0.10 | 0.15 / 0.35 | 0.20 | mm | | ds p. 58 |
| Lead length L / L1 | 0.35 / 0.30 | 0.40 / 0.40 | 0.45 / 0.50 | mm | | ds p. 58 |
| Lead tip to exposed pad edge K | | 0.355 | | mm | REF | ds p. 58 |
| Package height A | 0.80 | 0.85 | 0.90 | mm | | ds p. 58 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| PVDD, AVDD33, AVDD33_AUD, AVDD_BRF | -0.3 to +3.63 | V | ds p. 38 |
| VDDIOA | -0.3 to +3.63 | V | ds p. 38 |
| RF input (VLNA) | 0 max | dBm | ds p. 38 |
| Storage temperature | -40 to +125 | °C | ds p. 38 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| PVDD (typ 1.8 / 3.3 V; depends on the part number) | 1.71 | 3.63 | V | ds p. 38 |
| PVDD, SF32LB52EUB6 (supply voltage in ordering table) | 3.3 | 3.3 | V | ds p. 60 |
| AVDD33, AVDD33_AUD, AVDD_BRF (typ 1.8 / 3.3 V) | 1.71 | 3.63 | V | ds p. 38 |
| Operating ambient temperature | -40 | 85 | °C | ds p. 38 |

## Pins

Pin map read from Figure 5-1, the top-view pin diagram (ds p. 48), rendered and inspected, and cross-checked with Table 5-2 (PA pins, ds pp. 50–56) and Table 5-3 (dedicated pins, ds p. 57). Pin 1 (PA32) is at the top left; numbers run anticlockwise: 1–17 down the left side, 18–34 along the bottom, 35–51 up the right side, 52–68 along the top, 69 the ground pad. The bottom view of Figure 5-2 agrees (ds p. 58). Every PA pin has functions 0 (GPIO), 4 (PA_I2C_UART) and 5 (PA_TIM); the table lists the others. Functions marked `#` in the source are analog or special functions.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | PA32 | PA32 | I/O | GPIO_A32, PA_I2C_UART, PA_TIM; GPADC_CH4 | ds p. 48, ds p. 54 |
| 2 | PA31 | PA31 | I/O | GPIO_A31, PA_I2C_UART, PA_TIM; GPADC_CH3 | ds p. 48, ds p. 54 |
| 3 | PA30 | PA30 | I/O | GPIO_A30, PA_I2C_UART, PA_TIM; EFUSE_PWR, I2S1_LRCK, GPADC_CH2 | ds p. 48, ds p. 54 |
| 4 | PA29 | PA29 | I/O | GPIO_A29, PA_I2C_UART, PA_TIM; SPI1_CS, I2S1_BCK, GPADC_CH1 | ds p. 48, ds p. 54 |
| 5 | PA28 | PA28 | I/O | GPIO_A28, PA_I2C_UART, PA_TIM; SPI1_CLK, I2S1_SDI, GPADC_CH0 | ds p. 48, ds p. 54 |
| 6 | PA27 | PA27 | I/O | GPIO_A27, PA_I2C_UART, PA_TIM; WKUP_PIN3 | ds p. 48, ds p. 54 |
| 7 | PA26 | PA26 | I/O | GPIO_A26, PA_I2C_UART, PA_TIM; WKUP_PIN2 | ds p. 48, ds p. 53 |
| 8 | PA25 | PA25 | I/O | GPIO_A25, PA_I2C_UART, PA_TIM; SPI1_DI, I2S1_SDO, XTAL32K_EXT, WKUP_PIN1 | ds p. 48, ds p. 53 |
| 9 | PA24 | PA24 | I/O | GPIO_A24, PA_I2C_UART, PA_TIM; SPI1_DIO, I2S1_MCLK, WKUP_PIN0 | ds p. 48, ds p. 53 |
| 10 | PA23 | PA23 | I/O | GPIO_A23, PA_I2C_UART, PA_TIM; PDM1_DATA, XTAL32K_XO | ds p. 48, ds p. 53 |
| 11 | PA22 | PA22 | I/O | GPIO_A22, PA_I2C_UART, PA_TIM; PDM1_CLK, XTAL32K_XI | ds p. 48, ds p. 53 |
| 12 | VDD_RTC | VDD_RTC | PWR (output) | RTC LDO output | ds p. 48, ds p. 19, ds p. 57 |
| 13 | VDD_RET | VDD_RET | PWR (output) | RET LDO output | ds p. 48, ds p. 19, ds p. 57 |
| 14 | AVDD33 | AVDD33 | PWR | Analog supply input | ds p. 48, ds p. 57 |
| 15 | VDD_VOUT2 | VDD_VOUT2 | not in Table 5-3 | LPSYS LDO output node (Figure 1-3); absent from Table 5-3 | ds p. 48, ds p. 19 |
| 16 | VDD_VOUT1 | VDD_VOUT1 | not in Table 5-3 | HPSYS LDO output node (Figure 1-3); absent from Table 5-3 | ds p. 48, ds p. 19 |
| 17 | BUCK_FB | BUCK_FB | PWR | Buck output | ds p. 48, ds p. 19, ds p. 57 |
| 18 | BUCK_LX | BUCK_LX | A, I/O | Buck inductor switch node | ds p. 48, ds p. 19, ds p. 57 |
| 19 | PVDD | PVDD | PWR | System supply input | ds p. 48, ds p. 57 |
| 20 | VBATS | VBATS | A, I | VBAT voltage measurement input | ds p. 48, ds p. 57 |
| 21 | VDDIOA | VDDIOA | PWR | GPIO supply input | ds p. 48, ds p. 57 |
| 22 | NC | NC | - | No function | ds p. 48, ds p. 57 |
| 23 | VDD_SIP | VDD_SIP | PWR | SiP supply input (Table 5-3); VDD18_LDO output node in Figure 1-3 | ds p. 48, ds p. 19, ds p. 57 |
| 24 | PA21 | PA21 | I/O | GPIO_A21, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 53 |
| 25 | PA20 | PA20 | I/O | GPIO_A20, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 53 |
| 26 | PA19 | PA19 | I/O | GPIO_A19, PA_I2C_UART, PA_TIM; SWCLK | ds p. 48, ds p. 52 |
| 27 | PA18 | PA18 | I/O | GPIO_A18, PA_I2C_UART, PA_TIM; SWDIO | ds p. 48, ds p. 52 |
| 28 | PA17 | PA17 | I/O | GPIO_A17, PA_I2C_UART, PA_TIM; MPI2_DIO3, SD1_DIO1 | ds p. 48, ds p. 52 |
| 29 | PA16 | PA16 | I/O | GPIO_A16, PA_I2C_UART, PA_TIM; MPI2_CLK, SD1_DIO0 | ds p. 48, ds p. 52 |
| 30 | PA15 | PA15 | I/O | GPIO_A15, PA_I2C_UART, PA_TIM; MPI2_DIO0, SD1_CMD | ds p. 48, ds p. 52 |
| 31 | PA14 | PA14 | I/O | GPIO_A14, PA_I2C_UART, PA_TIM; MPI2_DIO2, SD1_CLK | ds p. 48, ds p. 52 |
| 32 | PA13 | PA13 | I/O | GPIO_A13, PA_I2C_UART, PA_TIM; MPI2_DIO1, SD1_DIO3 | ds p. 48, ds p. 52 |
| 33 | PA12 | PA12 | I/O | GPIO_A12, PA_I2C_UART, PA_TIM; MPI2_CS, SD1_DIO2 | ds p. 48, ds p. 51 |
| 34 | PA11 | PA11 | I/O | GPIO_A11, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 51 |
| 35 | PA10 | PA10 | I/O | GPIO_A10, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 51 |
| 36 | DACN | DACN | A, O | Audio DAC output | ds p. 48, ds p. 57 |
| 37 | DACP | DACP | A, O | Audio DAC output | ds p. 48, ds p. 57 |
| 38 | AVDD33_AUD | AVDD33_AUD | PWR | Audio supply input | ds p. 48, ds p. 57 |
| 39 | AUD_VREF | AUD_VREF | A, I | Audio reference voltage input | ds p. 48, ds p. 57 |
| 40 | MIC_BIAS | MIC_BIAS | PWR (output) | Microphone bias supply output | ds p. 48, ds p. 57 |
| 41 | ADCP | ADCP | A, I | Audio ADC input | ds p. 48, ds p. 57 |
| 42 | PA09 | PA09 | I/O | GPIO_A9, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 51 |
| 43 | PA08 | PA08 | I/O | GPIO_A8, PA_I2C_UART, PA_TIM; LCDC1_SPI_DIO3, PDM1_DATA, LCDC1_JDI_VST, LCDC1_8080_DIO1 | ds p. 48, ds p. 51 |
| 44 | PA07 | PA07 | I/O | GPIO_A7, PA_I2C_UART, PA_TIM; LCDC1_SPI_DIO2, PDM1_CLK, LCDC1_JDI_ENB, LCDC1_8080_DIO0 | ds p. 48, ds p. 51 |
| 45 | PA06 | PA06 | I/O | GPIO_A6, PA_I2C_UART, PA_TIM; LCDC1_SPI_DIO1, I2S1_LRCK, LCDC1_JDI_HST, LCDC1_8080_DC | ds p. 48, ds p. 51 |
| 46 | PA05 | PA05 | I/O | GPIO_A5, PA_I2C_UART, PA_TIM; LCDC1_SPI_DIO0, I2S1_BCK, LCDC1_JDI_R1, LCDC1_8080_RD | ds p. 48, ds p. 50 |
| 47 | PA04 | PA04 | I/O | GPIO_A4, PA_I2C_UART, PA_TIM; LCDC1_SPI_CLK, I2S1_SDI, LCDC1_JDI_G1, LCDC1_8080_WR | ds p. 48, ds p. 50 |
| 48 | PA03 | PA03 | I/O | GPIO_A3, PA_I2C_UART, PA_TIM; LCDC1_SPI_CS, I2S1_SDO, LCDC1_JDI_B1, LCDC1_8080_CS | ds p. 48, ds p. 50 |
| 49 | PA02 | PA02 | I/O | GPIO_A2, PA_I2C_UART, PA_TIM; LCDC1_SPI_TE, I2S1_MCLK, LCDC1_JDI_B2, LCDC1_8080_TE | ds p. 48, ds p. 50 |
| 50 | PA01 | PA01 | I/O | GPIO_A1, PA_I2C_UART, PA_TIM | ds p. 48, ds p. 50 |
| 51 | PA00 | PA00 | I/O | GPIO_A0, PA_I2C_UART, PA_TIM; LCDC1_SPI_RSTB, LCDC1_8080_RSTB | ds p. 48, ds p. 50 |
| 52 | AVDD_BRF | AVDD_BRF | PWR | RF supply input | ds p. 48, ds p. 57 |
| 53 | BRF_ANT | BRF_ANT (ANT in Figure 5-1) | A, I/O | Antenna interface | ds p. 48, ds p. 57 |
| 54 | XTAL48M_XO | XTAL48M_XO | A, O | 48 MHz crystal interface | ds p. 48, ds p. 57 |
| 55 | XTAL48M_XI | XTAL48M_XI | A, I | 48 MHz crystal interface | ds p. 48, ds p. 57 |
| 56 | PA44 | PA44 | I/O | GPIO_A44, PA_I2C_UART, PA_TIM; WKUP_PIN20 | ds p. 48, ds p. 56 |
| 57 | PA43 | PA43 | I/O | GPIO_A43, PA_I2C_UART, PA_TIM; LCDC1_JDI_G2, LCDC1_8080_DIO7, WKUP_PIN19 | ds p. 48, ds p. 56 |
| 58 | PA42 | PA42 | I/O | GPIO_A42, PA_I2C_UART, PA_TIM; LCDC1_JDI_R2, LCDC1_8080_DIO6, WKUP_PIN18 | ds p. 48, ds p. 56 |
| 59 | PA41 | PA41 | I/O | GPIO_A41, PA_I2C_UART, PA_TIM; LCDC1_JDI_HCK, LCDC1_8080_DIO5, WKUP_PIN17 | ds p. 48, ds p. 56 |
| 60 | PA40 | PA40 | I/O | GPIO_A40, PA_I2C_UART, PA_TIM; SPI2_CS, LCDC1_JDI_XRST, LCDC1_8080_DIO4, WKUP_PIN16 | ds p. 48, ds p. 56 |
| 61 | PA39 | PA39 | I/O | GPIO_A39, PA_I2C_UART, PA_TIM; SPI2_CLK, LCDC1_JDI_VCK, LCDC1_8080_DIO3, WKUP_PIN15 | ds p. 48, ds p. 56 |
| 62 | PA38 | PA38 | I/O | GPIO_A38, PA_I2C_UART, PA_TIM; SPI2_DI, WKUP_PIN14 | ds p. 48, ds p. 55 |
| 63 | PA37 | PA37 | I/O | GPIO_A37, PA_I2C_UART, PA_TIM; SPI2_DIO, LCDC1_8080_DIO2, WKUP_PIN13 | ds p. 48, ds p. 55 |
| 64 | PA36 | PA36 | I/O | GPIO_A36, PA_I2C_UART, PA_TIM; USB11_DM, WKUP_PIN12 | ds p. 48, ds p. 55 |
| 65 | PA35 | PA35 | I/O | GPIO_A35, PA_I2C_UART, PA_TIM; USB11_DP, WKUP_PIN11 | ds p. 48, ds p. 55 |
| 66 | PA34 | PA34 | I/O | GPIO_A34, PA_I2C_UART, PA_TIM; GPADC_CH6, WKUP_PIN10 | ds p. 48, ds p. 55 |
| 67 | PA33 | PA33 | I/O | GPIO_A33, PA_I2C_UART, PA_TIM; GPADC_CH5 | ds p. 48, ds p. 55 |
| 68 | GPADC_VREF | GPADC_VREF | A, I | GPADC VREF input | ds p. 48, ds p. 57 |
| 69 | EPAD | EPAD (GND PAD in Figure 5-1) | GND | Ground | ds p. 48, ds p. 57 |

## Typical application

This datasheet contains no application circuit and no external component values. What it does establish:

- **Supplies.** PVDD (system), AVDD33 (analog), AVDD33_AUD (audio), AVDD_BRF (RF) and VDDIOA (GPIO) are supply inputs (ds p. 57). For SF32LB52EUB6, PVDD is 3.3 V (ds p. 60).
- **PMU pins.** PVDD feeds the buck and the VDD18 and RTC LDOs. The buck's switch node is BUCK_LX and its output is BUCK_FB, which feeds the HPSYS, LPSYS, RET and RF LDOs. Their outputs appear at VDD_VOUT1 (HPSYS), VDD_VOUT2 (LPSYS), VDD_RET and VDD_RTC; VDD_SIP sits on the VDD18 LDO output (Figure 1-3, ds p. 19). So the buck inductor goes from BUCK_LX to BUCK_FB, and the LDO outputs are decoupling nodes. The source gives no inductor or capacitor values.
- **Clocks.** A 48 MHz crystal on XTAL48M_XI/XTAL48M_XO (ds p. 57). An optional 32 kHz crystal on PA22 (XTAL32K_XI) and PA23 (XTAL32K_XO), or an external 32 kHz input on PA25 (XTAL32K_EXT) (ds p. 53). No crystal specification or load capacitors are given.
- **Debug.** SWDIO is PA18 (pin 27) and SWCLK is PA19 (pin 26), function 2 (ds p. 52).
- **USB 2.0 FS.** USB11_DP is PA35 (pin 65) and USB11_DM is PA36 (pin 64), function 2, with an on-chip PHY (ds p. 30, ds p. 55).
- **External memory.** MPI2 (QSPI) is on PA12–PA17: CS PA12, CLK PA16, DIO0 PA15, DIO1 PA13, DIO2 PA14, DIO3 PA17 (ds p. 51, ds p. 52).
- **GPADC.** Channels 0–6 are PA28–PA34 (ds p. 54, ds p. 55). VBATS (pin 20) measures the battery (ds p. 57). Keep source resistance within Table 4-18's limit for the chosen sample time, e.g. 50 kΩ at 6250 ns (ds p. 46).
- **Unused inputs.** In low-power use, a floating input with its input buffer enabled raises current; tie such pins to a defined level through another chip's output or a pull resistor (ds p. 49).
- **Ground.** Pin 69 (EPAD) is ground (ds p. 57).

## Layout notes

- QFN68L, 7.00 × 7.00 mm body, 0.35 mm pitch, 5.49 mm nominal exposed pad, leads 0.40 mm long, 0.15 mm wide nominal (ds p. 58). From these, lead centres sit 3.30 mm from the package centre, and 17 leads per side span ±2.80 mm.
- The source gives no recommended land pattern, stencil, thermal-via or RF layout guidance.
- The library footprint `FP_SiFli_SF32LB52X_QFN68L_7X7` places 0.55 × 0.25 mm lands at ±3.30 mm, ±2.80 mm along each edge, with pad 1 at top left and numbering anticlockwise. This matches the package drawing's lead positions and pin order (ds p. 48, ds p. 58). Its 5.49 mm square ground pad equals D2/E2 nominal (ds p. 58). Its 0.25 mm land width exceeds the 0.20 mm maximum lead width and leaves 0.10 mm between lands; the source does not say whether that is intended.

## Pitfalls

- **Table 5-3 omits pins 15 and 16.** VDD_VOUT2 and VDD_VOUT1 appear only in Figure 5-1 and Figure 1-3 (ds p. 48, ds p. 19, ds p. 57). An agent building a pin list from the table alone gets 67 pins.
- The variants differ in supply: SF32LB52DUB6 needs PVDD = 1.8 V, the others in Table 5-7 need 3.3 V (ds p. 60). Do not substitute across variants on one board.
- SF32LB52EUB6 has 32 Mb pSRAM in the package and no flash listed (ds p. 60); firmware storage needs an external memory, which this source does not describe.
- PA22/PA23 double as the 32 kHz crystal pins and PA18/PA19 as SWD (ds p. 52, ds p. 53); using them as plain GPIO gives those up.
- The USB pins are PA35/PA36, which also carry wake-up functions (ds p. 55). They are not dedicated USB pins in the library; nothing marks them as USB.
- GPADC single-ended range is 0–3.3 V (ds p. 23), so VBAT up to 4.7 V must go to VBATS, not to a PA ADC pin (ds p. 5).
- VLNA (the RF input) is rated 0 dBm maximum (ds p. 38).
- This is a V0.1 draft whose values may change without notice (ds p. 2).

## Not established by the sources

Library and package-doc claims compared with the shipped source, most important first:

- **`MCU_SF32LB52GUC6` has no source in the package.** Table 5-7 lists SF32LB52BU36, SF32LB52BU56, SF32LB52DUB6 and SF32LB52EUB6 only (ds p. 60). The library attaches `#[doc("docs/ds0052-sf32lb52x-cn.pdf")]` to GUC6 and the README calls it "the official 3.3 V / 64 Mb pSRAM ordering variant recorded by DS5202 V0.2.5", a document not in the repository. Its supply voltage, in-package memory and pin compatibility are not established here.
- **Pins 15 and 16 are not in the pin table.** The library declares `VDD_VOUT2: 15` and `VDD_VOUT1: 16` as `required [power_out]` with "add 4.7 uF to ground". Figure 5-1 shows both names at those pins (ds p. 48), and Figure 1-3 shows them as LPSYS and HPSYS LDO output nodes (ds p. 19). Table 5-3 does not list them (ds p. 57). The 4.7 µF value is not in this source.
- **`VDD_SIP` direction and the variant rule.** The library types pin 23 `passive` and says it is "external input on B/D variants, internal-LDO output on E/G/J variants". Table 5-3 calls VDD_SIP "SIP 电源输入" (SiP supply input) for all variants (ds p. 57). Figure 1-3 draws it on the output of the VDD18 LDO fed from PVDD (ds p. 19). The variant rule is not in this source; `examples/sf32-miniboard` treats VDD_SIP as a decoupling-only node for EUB6, which this source neither confirms nor rules out.
- **Land pattern: not checkable here, and the package README cites the wrong figure.** `docs/README.md` says "SiFli Figure 5-3 specifies 0.20 x 0.55 mm perimeter copper, 0.30 x 0.65 mm solder-mask openings…". In this PDF, Figure 5-3 is the carrier-tape drawing (ds p. 58); the README means a figure in the hardware design guide, which is not in the repository. The README's 0.20 mm copper width also disagrees with the library's own 0.25 mm lands (`P_SF32LB52X_QFN68_LEAD`, 0.55 × 0.25 mm). Against the package drawing, the land centres, pitch, pin order and 5.49 mm centre pad match (ds p. 48, ds p. 58). The paste windows, 16 thermal vias and bottom copper come from SiFli's KiCad generator, which is not pinned locally.
- **The source URL is a home page** (see the flag under Identity). The document's provenance ("an authorized source") is not stated more precisely.
- **Support circuit missing from the source.** The datasheet gives no:
  - buck inductor value or rating, or capacitor values for BUCK_FB, VDD_VOUT1/2, VDD_RET, VDD_RTC, VDD_SIP or the supply pins;
  - reset pin, boot-mode pin, or boot/download procedure (the 68 signal pins in Figure 5-1 include no reset; ds p. 48);
  - 48 MHz crystal specification or load capacitors;
  - antenna matching network for BRF_ANT;
  - recommended operating range for VDDIOA (Table 4-1 omits it; Table 4-2 has its absolute maximum; ds p. 38);
  - thermal resistance or maximum junction temperature (HTOL was run at Tj = 125 °C; ds p. 39).
- **Naming inconsistencies in the source.** Pin 53 is "ANT" in Figure 5-1 and "BRF_ANT" in Table 5-3 (ds p. 48, ds p. 57). The UART maximum is 6 Mbps in Section 3.5.2 but 3 Mbaud in Table 3-1 (ds p. 25, ds p. 37). Table 5-4 gives the Sn-Pb preheat target Tsmax as 100 °C, equal to Tsmin (ds p. 60).
- **Library comment claims checked against this source:** "pin layout Fig. 5-1, pin tables Table 5-2/5-3, package Fig. 5-2, ordering Table 5-7" and "Pin 1 is upper-left in top view and numbering proceeds counter-clockwise" are correct (ds p. 48, ds p. 57, ds p. 58, ds p. 60). "HCPU 192MHz + LCPU 24MHz, 576KB SRAM, BT/BLE 5.3" matches (ds p. 2, ds p. 4). The cross-check against DS5202 V0.2.5 and SiliconSchema cannot be repeated from shipped PDFs. `docs/siliconschema-sf32lb52x-chip.yaml` is a SiFli YAML file, not a datasheet.
- **Pin types.** `BUCK_FB` is `power_out`, matching "Buck 输出" (Buck output) (ds p. 57). `BUCK_LX` is `bidirectional` for an analog switch node (A, I/O). `XTAL48M_XO`/`XI` are `required` even though the source gives no rule for operating without the 48 MHz crystal. The source types `MIC_BIAS`, `VDD_RTC` and `VDD_RET` as plain "PWR".
