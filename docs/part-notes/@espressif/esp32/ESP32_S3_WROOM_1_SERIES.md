# ESP32-S3-WROOM-1 / WROOM-1U series — 2.4 GHz Wi-Fi + Bluetooth LE module, ESP32-S3, 4–16 MB flash, optional 2 MB PSRAM

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@espressif/esp32::ESP32_S3_WROOM_1",
  "parts_pattern": "ESP32_S3_WROOM_1U?_[NH]\\d+(R2)?",
  "identities": [
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N4", "lcsc": "C2913197"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N8", "lcsc": "C2913198"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N16", "lcsc": "C2913199"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-H4", "lcsc": null},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N4R2", "lcsc": "C2913203"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N8R2", "lcsc": "C2913204"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1-N16R2", "lcsc": "C2913205"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N4", "lcsc": "C2980296"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N8", "lcsc": "C2980297"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N16", "lcsc": "C2980298"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N4R2", "lcsc": "C3013942"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N8R2", "lcsc": "C3013944"},
    {"mfr": "Espressif", "mpn": "ESP32-S3-WROOM-1U-N16R2", "lcsc": "C3013945"}
  ],
  "sources": [
    {"id": "ds", "title": "ESP32-S3-WROOM-1 / ESP32-S3-WROOM-1U Datasheet", "revision": "v1.8, 2026-03-02",
     "url": "https://www.espressif.com/sites/default/files/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf",
     "sha256": "27d71971da07c280c6068d08c74720d1a25b8f20cf8494dc1765bdd28d40d435",
     "local": "lib/@espressif/esp32/docs/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf"},
    {"id": "soc", "title": "ESP32-S3 Series Datasheet", "revision": "v2.2, 2026-03-05",
     "url": "https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf",
     "sha256": "2d5a7cb7fd559d8d972bd88db32669c0196d23f22d7afaafb0f63d099b589a3f",
     "local": "lib/@espressif/esp32/docs/esp32-s3_datasheet_en.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF page indices (`pdftotext -f N -l N`). In both PDFs they equal the printed page numbers. The URLs are the ones recorded in `lib/@espressif/esp32/docs/README.md`; the PDFs themselves print the shorter `https://www.espressif.com/documentation/<file>.pdf` form (ds p. 2, soc p. 7).

## Summary

ESP32-S3-WROOM-1 and ESP32-S3-WROOM-1U are 41-pad Wi-Fi (802.11b/g/n) + Bluetooth LE 5 modules built on the ESP32-S3 SoC (dual-core Xtensa LX7, up to 240 MHz). Each carries a 40 MHz crystal, Quad SPI flash and RF matching (ds p. 2, ds p. 9). WROOM-1 has a PCB antenna and is 18.0 × 25.5 × 3.1 mm; WROOM-1U has an external-antenna connector and is 18.0 × 19.2 × 3.2 mm. Both share one pinout (ds p. 3, ds p. 10). In the library one device, `ESP32_S3_WROOM_1`, serves 13 parts: seven WROOM-1 (N4, N8, N16, H4, N4R2, N8R2, N16R2) and six WROOM-1U (the same without H4). WROOM-1 parts use `FP_ESP32_S3_WROOM_1` and WROOM-1U parts `FP_ESP32_S3_WROOM_1U`. Nn = n MB Quad SPI flash, R2 = 2 MB Quad SPI PSRAM in the chip package, and H4 = 4 MB flash with a -40 to 105 °C rating (ds p. 3). Choose it when you want a certified radio with flash on board and need only a 3.3 V supply, EN and decoupling. Do not use this device for the R8 / R16VA modules: their Octal PSRAM takes IO35–IO37, which this device offers as free GPIOs (ds p. 12).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| CPU clock | | | 240 | MHz | Xtensa dual-core 32-bit LX7 | ds p. 2 |
| ROM / SRAM / RTC SRAM | | 384 / 512 / 16 | | KB | | ds p. 2 |
| Flash (Quad SPI) | | 4 / 8 / 16 | | MB | N4, H4 / N8 / N16 suffix | ds p. 3 |
| PSRAM (Quad SPI, in chip package) | | 2 | | MB | R2 suffix only; none on N and H4 parts | ds p. 3, ds p. 4 |
| Flash clock | | | 80 | MHz | default; 120 MHz only on request | ds p. 4, ds p. 30 |
| On-module crystal | | 40 | | MHz | | ds p. 2 |
| GPIOs on pads | | 36 | | | of which 4 are strapping pins | ds p. 2 |
| Wi-Fi channel centre frequency | 2412 | | 2484 | MHz | | ds p. 32 |
| Wi-Fi TX power, 802.11b 1 Mbps | | 20.5 | | dBm | spectral mask and EVM met | ds p. 32 |
| Wi-Fi TX power, 802.11n HT40 MCS7 | | 17.0 | | dBm | spectral mask and EVM met | ds p. 32 |
| Wi-Fi RX sensitivity, 802.11b 1 Mbps | | -98.2 | | dBm | PER 8 % | ds p. 33 |
| Bluetooth LE TX power range | -24.0 | | 20.0 | dBm | | ds p. 34 |
| Supply current, Wi-Fi TX peak | | 355 | | mA | 802.11b 1 Mbps @ 20.5 dBm, 100 % duty, 3.3 V, 25 °C | ds p. 28 |
| Supply current, Wi-Fi RX peak | | 95 / 97 | | mA | HT20 / HT40 | ds p. 28 |
| Supply current, Bluetooth LE TX peak | | 344 | | mA | @ 20.0 dBm | ds p. 28 |
| Modem-sleep current | | 91.7 / 107.9 | | mA | 240 MHz, both cores 128-bit access; peripheral clocks off / on | ds p. 29 |
| Light-sleep current | | 240 | | µA | add 40 µA for 2 MB 4-line PSRAM (R2) | ds p. 30 |
| Deep-sleep current | | 7 | | µA | RTC memory on, RTC peripherals off | ds p. 30 |
| Power-off current | | 1 | | µA | EN low | ds p. 30 |
| VIH / VIL | 0.75 × VDD | | 0.25 × VDD | V | VIH min / VIL max; 3.3 V, 25 °C | ds p. 27 |
| VOH / VOL | 0.8 × VDD | | 0.1 × VDD | V | VOH min / VOL max, high-impedance load | ds p. 27 |
| IOH (source) / IOL (sink) | | 40 / 28 | | mA | VDD = 3.3 V, PAD_DRIVER = 3 | ds p. 27 |
| Internal weak pull-up / pull-down | | 45 | | kΩ | | ds p. 27 |
| EN reset release voltage VIH_nRST | 0.75 × VDD | | VDD + 0.3 | V | | ds p. 28 |
| EN reset voltage VIL_nRST | -0.3 | | 0.25 × VDD | V | | ds p. 28 |
| Rail stabilisation before EN high, tSTBL | 50 | | | µs | | ds p. 16 |
| EN low time to reset, tRST | 50 | | | µs | | ds p. 16 |
| Strapping hold time after EN high, tH | 3 | | | ms | setup time tSU = 0 ms min | ds p. 14 |
| ESD, HBM / CDM | | ±2000 / ±500 | | V | | ds p. 47 |
| Moisture sensitivity level | | 3 | | | solder within 168 h of opening at 25 ± 5 °C, 60 %RH | ds p. 47 |
| Module size, WROOM-1 | | 18.0 × 25.5 × 3.1 | | mm | ±0.2 (L, W), ±0.15 (H) | ds p. 42 |
| Module size, WROOM-1U | | 18.0 × 19.2 × 3.2 | | mm | ±0.2 (L, W), ±0.15 (H) | ds p. 42 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VDD33 (pad 2) | -0.3 to 3.6 | V | ds p. 27 |
| Storage temperature | -40 to 105 | °C | ds p. 27 |
| Reflow peak temperature (single reflow, SAC305) | 235 to 250, 30 to 70 s | °C | ds p. 47 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD33 (typ 3.3 V) | 3.0 | 3.6 | V | ds p. 27 |
| Current the external supply can deliver | 0.5 | | A | ds p. 27 |
| Ambient temperature, N and R2 parts (85 °C version) | -40 | 85 | °C | ds p. 3, ds p. 27 |
| Ambient temperature, H4 (105 °C version) | -40 | 105 | °C | ds p. 3, ds p. 27 |
| Storage in sealed moisture barrier bag | | < 40 °C, 90 %RH | — | ds p. 47 |

## Pins

The library pin `VDD` is the datasheet's `3V3`. Pads 1, 40 and 41 (EPAD) all map to the library pin `GND`.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | GND | GND | Power | Ground | ds p. 11 |
| 2 | VDD | 3V3 | Power | 3.3 V supply | ds p. 11 |
| 3 | EN | EN | Input | High enables the chip, low powers it off; do not leave floating | ds p. 11 |
| 4 | IO4 | IO4 | I/O/T | RTC_GPIO4, GPIO4, TOUCH4, ADC1_CH3 | ds p. 11 |
| 5 | IO5 | IO5 | I/O/T | RTC_GPIO5, GPIO5, TOUCH5, ADC1_CH4 | ds p. 11 |
| 6 | IO6 | IO6 | I/O/T | RTC_GPIO6, GPIO6, TOUCH6, ADC1_CH5 | ds p. 11 |
| 7 | IO7 | IO7 | I/O/T | RTC_GPIO7, GPIO7, TOUCH7, ADC1_CH6 | ds p. 11 |
| 8 | IO15 | IO15 | I/O/T | RTC_GPIO15, GPIO15, U0RTS, ADC2_CH4, XTAL_32K_P | ds p. 11 |
| 9 | IO16 | IO16 | I/O/T | RTC_GPIO16, GPIO16, U0CTS, ADC2_CH5, XTAL_32K_N | ds p. 11 |
| 10 | IO17 | IO17 | I/O/T | RTC_GPIO17, GPIO17, U1TXD, ADC2_CH6 | ds p. 11 |
| 11 | IO18 | IO18 | I/O/T | RTC_GPIO18, GPIO18, U1RXD, ADC2_CH7, CLK_OUT3 | ds p. 11 |
| 12 | IO8 | IO8 | I/O/T | RTC_GPIO8, GPIO8, TOUCH8, ADC1_CH7, SUBSPICS1 | ds p. 11 |
| 13 | IO19 | IO19 | I/O/T | RTC_GPIO19, GPIO19, U1RTS, ADC2_CH8, CLK_OUT2, USB_D- | ds p. 11 |
| 14 | IO20 | IO20 | I/O/T | RTC_GPIO20, GPIO20, U1CTS, ADC2_CH9, CLK_OUT1, USB_D+ | ds p. 11 |
| 15 | IO3 | IO3 | I/O/T | RTC_GPIO3, GPIO3, TOUCH3, ADC1_CH2; strapping (JTAG source) | ds p. 11, ds p. 13 |
| 16 | IO46 | IO46 | I/O/T | GPIO46; strapping (boot mode, ROM messages) | ds p. 11, ds p. 13 |
| 17 | IO9 | IO9 | I/O/T | RTC_GPIO9, GPIO9, TOUCH9, ADC1_CH8, FSPIHD, SUBSPIHD | ds p. 11 |
| 18 | IO10 | IO10 | I/O/T | RTC_GPIO10, GPIO10, TOUCH10, ADC1_CH9, FSPICS0, FSPIIO4, SUBSPICS0 | ds p. 11 |
| 19 | IO11 | IO11 | I/O/T | RTC_GPIO11, GPIO11, TOUCH11, ADC2_CH0, FSPID, FSPIIO5, SUBSPID | ds p. 11 |
| 20 | IO12 | IO12 | I/O/T | RTC_GPIO12, GPIO12, TOUCH12, ADC2_CH1, FSPICLK, FSPIIO6, SUBSPICLK | ds p. 11 |
| 21 | IO13 | IO13 | I/O/T | RTC_GPIO13, GPIO13, TOUCH13, ADC2_CH2, FSPIQ, FSPIIO7, SUBSPIQ | ds p. 11 |
| 22 | IO14 | IO14 | I/O/T | RTC_GPIO14, GPIO14, TOUCH14, ADC2_CH3, FSPIWP, FSPIDQS, SUBSPIWP | ds p. 11 |
| 23 | IO21 | IO21 | I/O/T | RTC_GPIO21, GPIO21 | ds p. 11 |
| 24 | IO47 | IO47 | I/O/T | SPICLK_P_DIFF, GPIO47, SUBSPICLK_P_DIFF | ds p. 11 |
| 25 | IO48 | IO48 | I/O/T | SPICLK_N_DIFF, GPIO48, SUBSPICLK_N_DIFF | ds p. 11 |
| 26 | IO45 | IO45 | I/O/T | GPIO45; strapping (VDD_SPI voltage) | ds p. 11, ds p. 13 |
| 27 | IO0 | IO0 | I/O/T | RTC_GPIO0, GPIO0; strapping (boot mode) | ds p. 11, ds p. 13 |
| 28 | IO35 | IO35 | I/O/T | SPIIO6, GPIO35, FSPID, SUBSPID; default function set by eFuse | ds p. 11, ds p. 12 |
| 29 | IO36 | IO36 | I/O/T | SPIIO7, GPIO36, FSPICLK, SUBSPICLK; default function set by eFuse | ds p. 11, ds p. 12 |
| 30 | IO37 | IO37 | I/O/T | SPIDQS, GPIO37, FSPIQ, SUBSPIQ; default function set by eFuse | ds p. 11, ds p. 12 |
| 31 | IO38 | IO38 | I/O/T | GPIO38, FSPIWP, SUBSPIWP | ds p. 11 |
| 32 | IO39 | IO39 | I/O/T | MTCK, GPIO39, CLK_OUT3, SUBSPICS1 | ds p. 12 |
| 33 | IO40 | IO40 | I/O/T | MTDO, GPIO40, CLK_OUT2 | ds p. 12 |
| 34 | IO41 | IO41 | I/O/T | MTDI, GPIO41, CLK_OUT1 | ds p. 12 |
| 35 | IO42 | IO42 | I/O/T | MTMS, GPIO42 | ds p. 12 |
| 36 | RXD0 | RXD0 | I/O/T | U0RXD, GPIO44, CLK_OUT2 | ds p. 12 |
| 37 | TXD0 | TXD0 | I/O/T | U0TXD, GPIO43, CLK_OUT1 | ds p. 12 |
| 38 | IO2 | IO2 | I/O/T | RTC_GPIO2, GPIO2, TOUCH2, ADC1_CH1 | ds p. 12 |
| 39 | IO1 | IO1 | I/O/T | RTC_GPIO1, GPIO1, TOUCH1, ADC1_CH0 | ds p. 12 |
| 40 | GND | GND | Power | Ground | ds p. 12 |
| 41 | GND | EPAD | Power | Ground (exposed pad, 3 × 3 copper islands) | ds p. 12 |

## Typical application

- Supply: 3.0–3.6 V on pad 2 from a source that can deliver at least 0.5 A (ds p. 27). The reference circuit puts 22 µF and 0.1 µF on 3V3 (C1, C3 in Figure 9-1) (ds p. 41).
- EN: never leave it floating (ds p. 11). Add an RC delay, usually R = 10 kΩ to 3V3 and C = 1 µF to GND, adjusted to the supply's power-up timing. EN must stay low until the rails have been stable for tSTBL ≥ 50 µs (ds p. 16, ds p. 41). The reference circuit adds a reset button through 0 Ω with 0.1 µF (SW1, R7, C8) (ds p. 41).
- Boot mode: IO0 has a weak pull-up (1 = SPI boot) and IO46 a weak pull-down. IO0 = 0 with IO46 = 0 enters joint download boot (UART0 or USB) (ds p. 13, ds p. 14). A button or jumper from IO0 to GND is the reference "Boot Option" (ds p. 41).
- IO45 selects the VDD_SPI voltage when the eFuse is not burnt: 0 (default, weak pull-down) gives 3.3 V and 1 gives 1.8 V (ds p. 13, ds p. 15). On N and H4 parts (no PSRAM), make sure the external circuit does not pull IO45 high at power-up. On R2 parts the VDD_SPI voltage is fixed by eFuse (ds p. 39).
- IO3 has no internal pull. It selects the JTAG source only when EFUSE_STRAP_JTAG_SEL is burnt, and then the external circuit must drive it (ds p. 15, ds p. 16).
- Strapping levels must be held for tH ≥ 3 ms after EN goes high (ds p. 14).
- USB: IO19 = USB_D- and IO20 = USB_D+ (ds p. 11). The reference circuit connects them through 0 Ω resistors, with footprints for capacitors to GND whose values are TBD (R4, R6, C5, C6) (ds p. 41).
- Optional 32.768 kHz crystal on IO15 (XTAL_32K_P) and IO16 (XTAL_32K_N), shown not fitted with 12 pF load capacitors and ESR ≤ 70 kΩ (ds p. 11, ds p. 41).
- Debug: UART0 on TXD0/RXD0 (pads 37/36) and pad JTAG on IO39–IO42 (MTCK, MTDO, MTDI, MTMS) (ds p. 12, ds p. 41).
- WROOM-1U needs an external 2.4 GHz, 50 Ω antenna with a first-generation (U.FL / MHF I / AMC compatible) plug, gain ≤ 2.33 dBi to stay inside the certification. The module ships without one (ds p. 43, ds p. 44).

## Layout notes

- Land pattern (both variants): 40 castellation lands, 1.5 × 0.9 mm on a 1.27 mm pitch. The left and right columns are 17.5 mm apart, and the 12 bottom lands span 13.97 mm centre to centre (ds p. 42, ds p. 45, ds p. 46).
- Exposed pad 41: a 3.7 × 3.7 mm array of 3 × 3 0.9 mm copper islands with vias for the thermal pad. Its centre is 7.5 mm from the left module edge and 10.29 mm from the bottom edge (ds p. 45, ds p. 46).
- WROOM-1: the 6 mm antenna area at the top of the 18 × 25.5 mm outline must stay free. Pad 1 is 7.49 mm from the top edge (ds p. 45). Placement and further keep-out rules are in the ESP32-S3 Hardware Design Guidelines, which this note does not pin (ds p. 10, ds p. 46).
- WROOM-1U has no antenna keep-out zone (ds p. 10). Pad 1 is 1.19 mm from the top edge of the 18 × 19.2 mm outline (ds p. 46).
- Soldering the EPAD is optional but improves heat transfer. Too much paste there lifts the module and weakens the castellated joints (ds p. 41).
- One reflow only: peak 235–250 °C for 30–70 s, 60–90 s above 217 °C, SAC305 solder (ds p. 47). Keep the module away from ultrasonic welding or cleaning, which can damage the crystal (ds p. 48).

## Pitfalls

- IO35, IO36 and IO37 (pads 28–30) are free only because no library part has Octal PSRAM. On R8 / R16VA modules they belong to the PSRAM (ds p. 12), and those modules are rated -40 to 65 °C (ds p. 3). Do not add an R8 or R16VA MPN to this device.
- On ESP32-S3-WROOM-1(U)-N16R16VA, IO47/IO48 run at 1.8 V (ds p. 4, ds p. 12). No library part is that variant, but do not assume otherwise if one is added.
- IO19/IO20 come up as USB Serial/JTAG with the USB pull-up enabled and glitch high at power-up. Re-assign them in firmware before using them as GPIO (soc p. 17, soc p. 19, soc p. 25).
- IO0, IO3, IO45 and IO46 are strapping pins: nothing on the board may override their levels during reset (ds p. 13).
- TXD0/RXD0 print ROM messages by default (ds p. 15). A device on UART0 sees boot traffic.
- R2 parts may draw more current than the modem-sleep and low-power tables show, which were measured without PSRAM; add about 40 µA in light-sleep (ds p. 29, ds p. 30).
- The H4 part is rated to 105 °C ambient, but Espressif asks customers to contact them about it (ds p. 3, ds p. 4). Check it can be bought before you design it in.
- The library pin name is `VDD`; the datasheet calls pad 2 `3V3` (ds p. 11).

## Not established by the sources

- **R8 / R16VA modules are not modeled.** The datasheet lists WROOM-1(U)-N4R8, -N8R8, -N16R8 and -N16R16VA (ds p. 3). The library leaves them out (`docs/esp32-part-catalog.md`: "memory variant changes pin availability or I/O voltage"). Because `ESP32_S3_WROOM_1` declares IO35–IO37 as optional free GPIOs, an R8 MPN must not be added to this device.
- **ESP32-S3-WROOM-1U-H4 is missing.** The datasheet lists it with -40 to 105 °C (ds p. 3), but the library has no part for it. The package's frozen product-selector catalog does not list it either, not even among the omitted MPNs.
- **H4 has no LCSC number.** ESP32-S3-WROOM-1-H4 is the only part here that `lcsc_map.json` does not map. The other twelve map to the C numbers in Identity.
- **The WROOM-1U footprint has no thermal vias.** Figure 11-2 draws the "via for thermal pad" ring in the WROOM-1U EPAD the same way Figure 11-1 does for WROOM-1 (ds p. 45, ds p. 46). `FP_ESP32_S3_WROOM_1` has the twelve vias, but `FP_ESP32_S3_WROOM_1U` (generated from Espressif's KiCad library) has only the nine 0.9 mm islands. The 0.5 mm via lands and 0.25 mm drills in `FP_ESP32_S3_WROOM_1` come from the CAD file, not from the datasheet.
- **The datasheet does not require pad 41.** The library declares EPAD as part of the required `GND` pin, while the datasheet calls soldering it optional (ds p. 41). The library choice is the safe one; it is not a datasheet requirement.
- **The library comment omits the EN capacitor.** `wroom_s3.cohdl` says a design "needs only power, EN pull-up, and decoupling". The datasheet asks for an RC delay on EN, not a bare pull-up (ds p. 41).
- **The decoupling values come from a figure, not a stated rule.** 22 µF + 0.1 µF are the values shown in Figure 9-1 (ds p. 41). R1/C2 on EN, C5/C6 on USB and the antenna matching parts are "TBD" there (ds p. 39, ds p. 41).
- **Not in these two PDFs:** antenna keep-out size beyond the 6 mm antenna area, ground-plane rules, the board-edge position of the antenna, and RF layout. They are in the ESP32-S3 Hardware Design Guidelines, which the library does not pin (ds p. 10, ds p. 46).
- **Which chip is inside each part is not stated per MPN.** The block diagrams list ESP32-S3, ESP32-S3R2, ESP32-S3R8 and ESP32-S3R16V as possible chips (ds p. 9). The SoC datasheet lists ESP32-S3R2 as EOL, upgraded to ESP32-S3RH2 (soc p. 13).
- **The module datasheet gives no thermal resistance, no junction-temperature limit and no per-pin current limit.**
- **The datasheet contradicts itself on a part name.** Figure 9-1 still names "ESP32-S3-WROOM-1-N16R16V" (ds p. 41), although v1.5 renamed that part N16R16VA (ds p. 51).
- **ESP32-S3-MINI-1 / MINI-1U are not covered.** ESP32-S3-MINI-1-N8 (C2913206) and ESP32-S3-MINI-1U-N8 (C2980299) are also LCSC-mapped S3 modules in this package (`DEV_ESP32_S3_MINI_1`). The package does not ship their datasheet, so this note does not cover them.
