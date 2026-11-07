# Air780E — LTE Cat.1 module (China bands), LGA-109, 15.8 × 17.7 × 2.3 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "cellular::AIR780E",
  "parts": ["cellular::CELL_AIR780E"],
  "identities": [
    {"mfr": "Luat (Hezhou)", "mpn": "Air780E", "lcsc": "C5246451"}
  ],
  "sources": [
    {"id": "hw", "title": "合宙 Air780E 硬件手册 (Air780E hardware manual)", "revision": "V1.0 (no date printed)",
     "url": "https://docs.openluat.com/air780e/hardware/design/", "sha256": "d792cb1b1e380a7cac2fd3e4d1d361b1b5b2fcd356d7ca33cb19188acb6f15e9", "local": "lib/cellular/docs/air780e-hw.pdf"},
    {"id": "gpio", "title": "Air780E/EG/EX, Air700E GPIO function table", "revision": "no revision printed",
     "url": "https://docs.openluat.com/air780e/", "sha256": "4ec4d57a2e8193fc74a02353f2b733c80622c2bbe3ba7921e58bc63e1078e328", "local": "lib/cellular/docs/Air780E_GPIO_table.pdf"},
    {"id": "decal", "title": "Air780E AT PADS decal (Air780E_AT.asc, 109 terminals)", "revision": "V20241030",
     "url": "https://docs.openluat.com/air780e/", "sha256": "14c6bcd01fa8aa51e78fd5d66e8d6637aa5cde3e82613807a211dcb4d6cb50dc", "local": "lib/cellular/docs/Air780E_AT_PADS_Decal_V20241030.zip"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. In `hw` they equal the printed "第 N 页，共 22 页" (page N of 22). `gpio` has two pages and no printed numbers. `decal` is a ZIP of PADS files, cited as "decal p. 1" for the whole archive; its coordinates are PADS basic units (1.5 × 10⁶ per mm) in PADS's +y-up frame. The manuals are in Chinese; the English here is a translation. The URLs are the ones in `lib/cellular/docs/README.md`; they are documentation pages, not direct PDF links. The manual's own pointer for the footprint library is `https://docs.openluat.com/air780e/product/` (hw p. 20).

**The pinned manual is short on electrical data.** It gives the supply range, a few pin limits and the land drawing. It sends the reader to web pages for the reference design, power-on circuit, supply design, UART, SIM and antenna guidance (hw p. 15, p. 19); none of those pages is in the repository.

## Summary

Air780E is an LTE Cat.1 module for the three Chinese operators (LTE-FDD B1/B3/B5/B8, LTE-TDD B34/B38/B39/B40/B41), 10 Mbps down and 5 Mbps up (hw p. 5). It runs from a single 3.3–4.3 V VBAT (3.8 V typical) and is used either with AT commands over UART1 or with Luat's LuatOS scripting (hw p. 5–6). All I/O runs at 1.8 V or 3.3 V, chosen by strapping PIN100 (hw p. 14). It has two SIM interfaces (dual SIM, single standby), USB, one main antenna pad and, under LuatOS, UART2, SPI, I²C, two ADCs and PWM (hw p. 6, p. 14, p. 17–18). It has no VoLTE or TTS (hw p. 7), and its bands do not cover most markets outside China; Luat's Air780EEN/EEU/EEJ are the regional variants (hw p. 12).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| VBAT supply | 3.3 | 3.8 | 4.3 | V | | hw p. 5, p. 14 |
| Average current, normal mode | — | 4.7 | — | mA | VBAT 3.8 V, connected, live network, B3, DRX 2.56 s, 5 min heartbeat | hw p. 5, p. 8 |
| Average current, low-power mode | — | 0.47 | — | mA | same test | hw p. 5, p. 8 |
| Average current, PSM+ (flight) | — | 3 | — | µA | same test | hw p. 5, p. 8 |
| Max TX power, TDD | — | 23 | — | dBm | Class 3, +1/−3 dB | hw p. 6 |
| Max TX power, FDD | — | 23 | — | dBm | Class 3, ±2 dB | hw p. 6 |
| RX sensitivity, FDD B1/B3/B5/B8 | — | -99 | — | dBm | 10 MHz | hw p. 6 |
| RX sensitivity, TDD B34/B38/B39/B40/B41 | — | -100 | — | dBm | 10 MHz | hw p. 6 |
| Peak data rate down / up | — | 10 / 5 | — | Mbps | theoretical | hw p. 5 |
| PWRKEY low time to power on | — | 1.5 | — | s | | hw p. 14 |
| STATUS goes high after power-on | — | 400 | — | ms | | hw p. 14 |
| VDD_EXT output current | — | — | 30 | mA | normal mode only | hw p. 14 |
| RefVolt (PIN99) output current | — | — | 3 | mA | reference only | hw p. 14 |
| ADC input range | — | — | 1.1 | V | 12-bit, ADC0 and ADC1 | hw p. 17, p. 18 |
| Normal GPIO drive, per pin / total | — | — | 10 / 200 | mA | | gpio p. 1 |
| AON GPIO drive, per pin / total | — | — | 5 / 5 | mA | the first three AON GPIO that double as WAKEUP3–5: ≤ 30 µA | gpio p. 1 |
| WAKEUP pin level | — | 1.8 | — | V | fixed; internal pull measures about 1.1 V; pull < 30 µA | gpio p. 1 |
| Module outline | — | 15.80 × 17.70 | — | mm | recommended PCB footprint drawing | hw p. 20 |
| Module height | — | 2.3 | — | mm | stated as "16*18*2.3mm" | hw p. 7 |
| Flash / RAM (LuatOS) | — | 5 MB / 2 MB SRAM | — | — | > 1.6 MB flash for user scripts | hw p. 10 |

## Absolute maximum ratings

The pinned sources give no absolute maximum ratings. The only hard limits stated are below.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| ADC0, ADC1 input | must not exceed 1.1 | V | hw p. 17, p. 18 |
| VDD_EXT load | must not exceed 30 | mA | hw p. 14 |
| RefVolt load | must not exceed 3 | mA | hw p. 14 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VBAT | 3.3 | 4.3 | V | hw p. 5, p. 14 |
| Temperature ("industrial grade"; not labelled operating or storage) | -40 | +85 | °C | hw p. 5 |
| I/O level, PIN100 open | 1.8 | 1.8 | V | hw p. 14 |
| I/O level, PIN100 to GND | 3.3 | 3.3 | V | hw p. 14 |
| Sealed-bag storage: temperature / humidity / time | — | < 40 °C, < 90 %RH, 12 months | — | hw p. 21 |
| Floor life after opening: < 30 °C and < 60 %RH, place within | — | 72 | h | hw p. 21 |

## Pins

The library mostly uses the AT names; LuatOS names are given after the slash. Pads marked "AT: float" are left unconnected in AT use (hw p. 14).

| Pad | Library pin | Datasheet name (AT / LuatOS) | Direction | Function | Source |
|---|---|---|---|---|---|
| 1, 10, 27, 34, 36, 37, 40, 41, 45, 46, 47, 48, 70, 71, 72, 73, 88, 89, 90, 91, 92, 93, 94, 95 | GND | GND | ground | System ground (24 pads; 89–94 are the centre thermal pads) | hw p. 13, p. 14 |
| 2, 3, 4, 5, 6, 8, 44, 68, 69, 74, 75, 76, 77, 83, 84, 85, 86, 87, 97, 98, 103, 104, 105, 108, 109 | NC | 悬空 (float) | — | Leave floating, do not connect (both AT and LuatOS) | hw p. 14, p. 18 |
| 7 | PWRKEY | PWRKEY | input | Power-on key: pull low 1.5 s to power on; bring out a test point | hw p. 14, p. 17 |
| 9 | ADC0 | ADC0 (AT: float) | input | 12-bit ADC, input must not exceed 1.1 V | hw p. 14, p. 17 |
| 11 | USIM_DAT | USIM_DAT | bidirectional | SIM1 data; SIM1 adapts to 1.8 V / 3.3 V cards | hw p. 14 |
| 12 | USIM_RST | USIM_RST | output | SIM1 reset | hw p. 14 |
| 13 | USIM_CLK | USIM_CLK | output | SIM1 clock | hw p. 14 |
| 14 | USIM_VDD | USIM_VDD | power out | SIM1 supply | hw p. 14 |
| 15 | RESET_N | RESET_N | input | Forced shutdown, active low; cannot reboot, power on again with PWRKEY | hw p. 14, p. 17 |
| 16 | NET_STATUS | NET_STATUS / GPIO27 | output | Network status LED control; LuatOS GPIO27 (AON) | hw p. 14, p. 17 |
| 17 | MAIN_RXD | MAIN_RXD / GPIO18 | input | UART1 (AT port) RX | hw p. 14, p. 17 |
| 18 | MAIN_TXD | MAIN_TXD / GPIO19 | output | UART1 (AT port) TX | hw p. 14, p. 17 |
| 19 | MAIN_DTR | MAIN_DTR / GPIO22 | bidirectional | UART1 DTR; LuatOS AON GPIO or interrupt input; float if unused | hw p. 14, p. 17 |
| 20 | MAIN_RI | MAIN_RI / GPIO24 | bidirectional | UART1 RI; LuatOS AON GPIO; float if unused | hw p. 14, p. 17 |
| 21 | MAIN_DCD | MAIN_DCD / GPIO2 | bidirectional | UART1 DCD; float if unused | hw p. 14, p. 17 |
| 22 | MAIN_CTS | MAIN_CTS / GPIO16 | bidirectional | UART1 CTS; float if unused | hw p. 14, p. 17 |
| 23 | MAIN_RTS | MAIN_RTS / GPIO17 | bidirectional | UART1 RTS; float if unused | hw p. 14, p. 17 |
| 24 | VDD_EXT | VDD_EXT | power out | 1.8 V (PIN100 open) or 3.3 V (PIN100 to GND); ≤ 30 mA; only valid in normal mode | hw p. 14, p. 17 |
| 25 | STATUS | STATUS / GPIO26 | output | High 400 ms after power-on; run status; LuatOS AON GPIO | hw p. 14, p. 17 |
| 26 | PWM0 | PWM0 (AT: float) | output | PWM0; GPIO table: I2S_MCLK, alt I2S0_MCLK / GPIO19 / PWM0 | hw p. 17, gpio p. 1 |
| 28 | AUX_RXD | AUX_RXD (AT: float) | input | UART2 RX; same signal as pad 51, use only one | hw p. 17 |
| 29 | AUX_TXD | AUX_TXD (AT: float) | output | UART2 TX; same signal as pad 53, use only one | hw p. 17 |
| 30 | GPIO29 | GPIO29 (AT: float) | bidirectional | GPIO29 / PWM1; GPIO table: I2S_BCK | hw p. 17, gpio p. 1 |
| 31 | GPIO30 | GPIO30 (AT: float) | bidirectional | GPIO30 / PWM2; GPIO table: I2S_LRCK | hw p. 17, gpio p. 1 |
| 32 | GPIO31 | GPIO31 (AT: float) | bidirectional | GPIO31; GPIO table: I2S_DIN | hw p. 17, gpio p. 1 |
| 33 | PWM4 | PWM4 (AT: float) | output | PWM4; GPIO table: I2S_DOUT, alt GPIO18 | hw p. 17, gpio p. 1 |
| 35 | LTE_ANT | LTE_ANT | RF | 4G antenna | hw p. 14, p. 17 |
| 38 | DBG_RXD | DBG_RXD | input | Debug UART (trace) RX; bring out a test point | hw p. 14, p. 17 |
| 39 | DBG_TXD | DBG_TXD | output | Debug UART (trace) TX; bring out a test point | hw p. 14, p. 17 |
| 42, 43 | VBAT | VBAT | power in | Module supply, 3.3–4.3 V | hw p. 14, p. 17 |
| 49 | GPIO1 | GPIO1 (AT: float) | bidirectional | GPIO1; GPIO table: LCD_RST | hw p. 18, gpio p. 1 |
| 50 | SPI_MOSI | SPI_MOSI / GPIO9 (AT: float) | bidirectional | SPI MOSI | hw p. 18 |
| 51 | SPI_MISO | SPI_MISO / GPIO10 (AT: float) | bidirectional | SPI MISO; same signal as pad 28 | hw p. 18 |
| 52 | SPI_CS | SPI_CS / GPIO8 (AT: float) | bidirectional | SPI chip select | hw p. 18 |
| 53 | SPI_CLK | SPI_CLK / GPIO11 (AT: float) | bidirectional | SPI clock; same signal as pad 29 | hw p. 18 |
| 54 | GPIO3 | GPIO3 (AT: float) | bidirectional | GPIO3; GPIO table: CAM_MCLK | hw p. 18, gpio p. 1 |
| 55 | GPIO6 | GPIO6 (AT: float) | bidirectional | GPIO6; internally joined to SIM2 with a pull-up (GPIO table note 16) | hw p. 18, gpio p. 2 |
| 56 | GPIO7 | GPIO7 (AT: float) | bidirectional | GPIO7; GPIO table: CAM_SPI_D1 | hw p. 18, gpio p. 1 |
| 57 | GPIO13 | GPIO13 (AT: float) | bidirectional | GPIO13; GPIO table: CAM_I2C_SCL | hw p. 18, gpio p. 1 |
| 58 | GPIO12 | GPIO12 (AT: float) | bidirectional | GPIO12; GPIO table: CAM_I2C_SDA | hw p. 18, gpio p. 1 |
| 59 | USB_DP | USB_DP | bidirectional | USB D+; bring out a test point | hw p. 14, p. 18 |
| 60 | USB_DM | USB_DM | bidirectional | USB D− | hw p. 14, p. 18 |
| 61 | VBUS | VBUS | input | USB VBUS; GPIO table: WAKEUP1 interrupt | hw p. 14, gpio p. 1 |
| 62 | USIM2_CLK | USIM2_CLK | output | SIM2 clock | hw p. 14, p. 18 |
| 63 | USIM2_RST | USIM2_RST | output | SIM2 reset | hw p. 14, p. 18 |
| 64 | USIM2_DAT | USIM2_DAT | bidirectional | SIM2 data; SIM2 level follows PIN100, not adaptive | hw p. 14, p. 18 |
| 65 | USIM2_VDD | USIM2_VDD | power out | SIM2 supply | hw p. 14, p. 18 |
| 66 | I2C_SDA | I2C_SDA / GPIO15 (AT: float) | bidirectional | I2C data | hw p. 18 |
| 67 | I2C_SCL | I2C_SCL / GPIO14 (AT: float) | bidirectional | I2C clock | hw p. 18 |
| 78 | GPIO28 | GPIO28 (AT: float) | bidirectional | GPIO28, AON; GPIO table: LCD_TE | hw p. 18, gpio p. 1 |
| 79 | USIM_DET | USIM_DET / WAKEUP2 | input | SIM card detect, interrupt on either edge, normally high; interrupt input only | hw p. 14, p. 18 |
| 80 | GPIO4 | GPIO4 (AT: float) | bidirectional | GPIO4; internally joined to SIM2 with a pull-up (note 16) | hw p. 18, gpio p. 2 |
| 81 | GPIO5 | GPIO5 (AT: float) | bidirectional | GPIO5; internally joined to SIM2 with a pull-up (note 16) | hw p. 18, gpio p. 2 |
| 82 | USB_BOOT | USB_BOOT | input | Short to VDD_EXT for USB firmware download; bring out a test point | hw p. 14, p. 18 |
| 96 | ADC1 | ADC1 (AT: float) | input | 12-bit ADC, input must not exceed 1.1 V | hw p. 18 |
| 99 | RefVolt | RefVolt / GPIO23 | power out | Always-high reference in every mode; ≤ 3 mA; reference only, not a supply | hw p. 14, p. 18 |
| 100 | IO_Volt_Set | IO_Volt_Set | input | Open = 1.8 V, GND = 3.3 V for VDD_EXT and all I/O including UART | hw p. 14, p. 18 |
| 101 | WAKEUP0 | WAKEUP0 (AT: float) | input | Wake-up interrupt input only; fixed 1.8 V level | hw p. 18, gpio p. 1 |
| 102 | GPIO20 | GPIO20 (AT: float) | bidirectional | GPIO20, AON or interrupt input | hw p. 18 |
| 106 | GPIO25 | GPIO25 (AT: float) | bidirectional | GPIO25, AON | hw p. 18 |
| 107 | GPIO21 | GPIO21 (AT: float) | bidirectional | GPIO21, AON or interrupt input | hw p. 18 |

Pin view (hw p. 13 AT, hw p. 16 LuatOS; "管脚透视图", seen from the top through the module): pin 1 at the top-left next to the chamfered corner; pins 1–13 down the left side, 14–22 along the bottom left to right (95 at the bottom-left corner, 96 at the bottom-right), 23–35 up the right side (35 = LTE_ANT at the top-right), 36–44 along the top right to left (97 top-right, 98 top-left). **The library footprint is the mirror image of this; see "Not established".** The pad-to-pin assignments in the device agree with hw pp. 13–18.

## Typical application

- Supply: VBAT 3.3–4.3 V on pads 42 and 43; 3.8 V is suggested for an adapter supply, and a Li-ion cell can feed it directly (hw p. 5). The manual gives no peak current, bulk capacitance or decoupling; it links to the online "供电设计及选型推荐" (supply design) page instead (hw p. 15, p. 19).
- Power-on: pull PWRKEY low for 1.5 s (hw p. 14). RESET_N only forces a shutdown; the module must then be started again with PWRKEY (hw p. 14). Bring PWRKEY out to a test point for fixture programming (hw p. 14). The power-on circuit itself is on a web page (hw p. 15).
- I/O voltage: leave PIN100 open for 1.8 V or tie it to GND for 3.3 V; all I/O including the UARTs follow VDD_EXT (hw p. 14). Under LuatOS it can also be set in software (hw p. 18).
- AT UART: only MAIN_RXD and MAIN_TXD are needed; leave the other MAIN_* pins floating if unused (hw p. 14). To wake the module from low-power or PSM+ mode by AT command, send at 9600 bps first (hw p. 14).
- SIM: use SIM1 (pads 11–14) for a single-SIM product (hw p. 14). SIM1 adapts to 1.8 V and 3.3 V cards; SIM2 only supports cards at the PIN100 level (hw p. 14, p. 18). With a removable card on SIM1 and a soldered SIM on SIM2, wire USIM_DET so the module notices the SIM1 card (hw p. 14).
- USB download: short USB_BOOT (pad 82) to VDD_EXT during USB firmware update; bring it, USB_DP/DM and DBG_RXD/TXD out to test points (hw p. 14).
- Reference levels: VDD_EXT is only valid in normal mode; in low-power and PSM+ modes it neither stays on nor stays off. For a reference that holds in every mode use RefVolt (PIN99, ≤ 3 mA) or an AON GPIO driven high (hw p. 14, p. 17).
- Antenna: LTE_ANT on pad 35 (hw p. 14). The antenna circuit guidance is a web page (hw p. 15, p. 19).

## Layout notes

Recommended PCB footprint (hw p. 20, top view, mm), confirmed by the PADS decal:
- Outline 15.80 × 17.70 mm, chamfered at the top-left (pin-1) corner.
- Perimeter: 13 pads per long side (pads 1–13, 23–35), 1.30 × 0.60 mm on a 1.10 mm pitch, first pad centre 2.25 mm from the short edge; 11 pads per short side (95, 14–22, 96 at the bottom; 98, 44–36, 97 at the top), 0.60 × 1.30 mm on a 1.10 mm pitch, outermost centres 2.40 mm from the long edges (hw p. 20; decal p. 1).
- Second ring: columns of 1.00 × 0.70 mm pads at x = ±5.0 mm on a 1.20 mm pitch (99, 45–53, 100 on the left; 104, 67–59, 103 on the right, top to bottom). Rows of 0.70 × 1.00 mm pads at y = ±5.95 mm: the top row (106, 72–68, 105) at x = −3.6 to +3.6 mm, the bottom row (101, 54–58, 102) at x = −3.5 to +3.5 mm (hw p. 20; decal p. 1).
- Third ring: columns of 1.00 × 0.70 mm pads at x = ±3.2 mm (73–80 on the left, 88–81 on the right, top to bottom); a row of three 0.70 × 1.00 mm pads (107, 108, 109) at y = 4.15 mm below centre (hw p. 20; decal p. 1).
- Centre: six 1.30 mm **square** ground pads (89–94) on a 1.80 mm grid (hw p. 20; decal shape `S`, p. 1).
- Keep other components at least 3 mm from the module (hw p. 20).
- Stencil thickness on the module pads: 0.2 mm (hw p. 21).
- Moisture: if the bag has been open longer than 72 h, or the humidity card shows > 10 %, bake at 125 ± 5 °C for 48 h with the packaging removed (hw p. 21).

## Pitfalls

- **Do not use `FP_Luat_AIR780E_LGA109_177x158mm` as it is.** It is mirrored top to bottom (see "Not established"); the module cannot be placed on it in any rotation.
- Pads 28/51 and 29/53 are the same two signals on two pads each (UART2 RX/TX and SPI MISO/CLK); use only one pad of each pair (hw p. 17, p. 18).
- GPIO4/5/6 (pads 80, 81, 55) are tied inside to the SIM2 pins with pull-ups; prefer other GPIOs (gpio p. 2, note 16).
- Normal GPIOs lose power in sleep and pulse high when the module wakes briefly for the network (gpio p. 1, note 9). Only AON GPIOs hold their level in low-power and PSM+ modes (hw p. 8; gpio p. 1, note 1).
- WAKEUP pins are interrupt inputs only, at a fixed 1.8 V (gpio p. 1, notes 5–6).
- Do not reuse DBG_TXD/DBG_RXD for anything else; they are the only log port in low-power mode and when USB is unplugged (gpio p. 2, note 12).
- RESET_N is not a reboot (hw p. 14).
- ADC inputs above 1.1 V are out of range (hw p. 17, p. 18).

## Not established by the sources

- **The library land is mirrored (most serious).** `FP_Luat_AIR780E_LGA109_177x158mm` copies all 109 PADS decal coordinates unchanged (checked pad by pad: decal x, y ÷ 1.5 × 10⁶ equals the library x, y for every pad). The decal is in PADS's +y-up frame: pin 1 at (−7.25, +6.6) mm next to the outline chamfer at the +y, −x corner. CoHDL's footprint frame is +y down (`docs/kicad_pcb.md`, "Coordinate frame and placement"). So the library puts pin 1 at the lower-left, numbers the pins clockwise seen from the top, puts 107–109 and the ±3.5 mm inner row in the top half, and puts 99 at the bottom of the left inner column. The manual's pin view (hw p. 13, p. 16) and its recommended footprint (hw p. 20, "正视图", top view) both show pin 1 at the top-left by the chamfer, counter-clockwise numbering, 107–109 and the ±3.5 mm row in the bottom half, and 99 at the top of the left inner column. Negating every library y coordinate would match hw p. 20. `CELL_AIR780EG` uses the same footprint and so has the same fault. No example in the repository uses either part.
- **Pad shapes differ from the decal and from hw p. 20:**
  - Pads 89–94: the library uses 1.3 mm circles and its comment reads the decal's `S` as round; in PADS `S` is square, and hw p. 20 draws 1.30 mm squares.
  - Pads 54–58, 68–72, 101, 102 and 105–109: the decal stack is 0.7 × 1.0 mm rotated 90° (long axis along y), and hw p. 20 draws them upright; the library uses 1.0 (x) × 0.7 (y) mm, turned 90°.
  - Pads 95–98: the decal stack is 0.6 × 1.3 mm at 90°, like the rest of the short-side rows, and hw p. 20 draws them the same; the library uses the 1.0 × 0.7 mm inner pad.
- **The README's padstack warning is wrong in detail.** It says the decal uses "per-pin padstack lengths of ~1.1-2.1 mm"; the decal's copper (layer −2) is 1.3, 1.0 and 1.3 mm, and the 1.5–2.1 mm figures are on its mask and paste layers 121 and 123 (decal p. 1).
- **Supply design is not in the pinned sources.** No peak or burst current, no VBAT capacitance, no ripple or drop limit for VBAT, no PWRKEY circuit or timing beyond "low for 1.5 s" (hw p. 14), and no power-off sequence. All are delegated to web pages (hw p. 15, p. 19). The README's VBAT 100 µF polymer-tantalum requirement ("design note §13.2") and the `t520.cohdl` comments cite a design note that is not in the repository. The 1.5 A peak quoted in `air780eg-hw.evidence.md` is for Air780EG, not Air780E.
- **Whether PWRKEY may be held low permanently for auto power-on**, and whether it has an internal pull-up, is not stated (hw p. 14, p. 17).
- **Antenna matching and RF layout** for LTE_ANT are not in the pinned sources (hw p. 15, p. 19).
- **Library comment on GPIO7 (pad 56):** "internally tied to SIM2, has pull-up". The GPIO table's note 16 names only pads 80, 81 and 55 (gpio p. 2).
- **GPIO numbers in the source collide:** the GPIO table gives GPIO19 for both MAIN_TXD (pad 18) and an alternate of I2S_MCLK (pad 26), and GPIO18 for both MAIN_RXD (pad 17) and an alternate of I2S_DOUT (pad 33) (gpio p. 1). The library repeats these in comments on `PWM0` and `PWM4` without comment.
- **Temperature range:** "-40°C ~ +85°C" is given without saying whether it is operating or storage (hw p. 5).
- **Absolute maximum ratings, ESD ratings and RF conducted power limits** are not given.
- **Module size:** the core-information list says 16 × 18 × 2.3 mm (hw p. 7); the footprint drawing says 15.80 × 17.70 mm (hw p. 20). The library comment uses 17.7 × 15.8 × 2.3 mm.
