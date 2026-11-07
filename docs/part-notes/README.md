# Part notes

Part notes are short Markdown summaries of a library part's datasheet, written for agents that design boards with the library (plan 5b). A note gives the facts an agent needs to use a part correctly, and cites the page of a pinned source for each one. It does not replace the datasheet.

**Location is provisional.** Whether notes ship inside each package's `docs/` (and so change its package hash) or live in a separate knowledge base is still open. Until that is decided, they live here, outside every package, so no package version or lock depends on them.

## Writing a note

1. Copy `TEMPLATE.md` to `<package>/<PART_NAME>.md`. A series note may cover a generated family instead (see `passive/YAGEO_RC_SERIES.md`).
2. Work from a pinned source. That is the manufacturer PDF the package already ships or records, with its SHA-256; `tools/sourcing_data/datasheet_links/` lists the links per identity. Do not use memory or another part's datasheet.
3. Cite a page for every number, keep absolute maximums apart from operating conditions, and say what the source does not establish.
4. Run `python3 -I tools/part_notes.py check` and fix what it reports. It checks:
   - the identity against the library;
   - the LCSC part number against `lcsc_map.json`;
   - each local source against its SHA-256;
   - the pin table against the device's pins;
   - that every table row cites a page.
5. A note starts as `"status": "draft"`. It becomes `"reviewed"` only after a person has checked it against the source.

## Pilots (2026-11-07)

| Note | Kind | Source |
|---|---|---|
| `@st/stm32/MCU_STM32F072CBT6.md` | MCU | ST datasheet, shipped in `lib/@st/stm32/docs/` |
| `ldo/LDO_AP2112K_3V3.md` | Power IC | Diodes DS39724 |
| `usb/FUSB302B_MLP.md` | Interface IC, four variants | onsemi FUSB302B/D Rev. 5 |
| `diode/D_1N4148W.md` | Discrete | Diodes DS30086 |
| `passive/YAGEO_RC_SERIES.md` | Passive series | Yageo RC_L V.14 |

The plan named STM32F103C8T6 as the MCU pilot. Its datasheet is not in the repository, so the pilot uses STM32F072CB, whose datasheet is.

## First 5c batch: series behind the bound parts (2026-11-07)

Every `passive` binding is on a Yageo RC, Yageo CC, Vishay CRCW or Samsung CL entry. These four series notes cover them:

| Note | Covers | Sources |
|---|---|---|
| `passive/YAGEO_RC_SERIES.md` | 7,744 Yageo RC_L primaries | Yageo RC_L V.14 |
| `passive/YAGEO_CC_SERIES.md` | 1,225 Yageo CC primaries (C0G, X5R, X7R) | Yageo NP0/C0G, X5R and X7R datasheets |
| `passive/VISHAY_CRCW_SERIES.md` | 7,736 Vishay CRCW alternates | Vishay documents 20052 and 20035 |
| `passive/SAMSUNG_CL_SERIES.md` | 716 Samsung CL alternates | Samsung CL catalog, December 2025 |

Writing them turned up where the library's MPNs and the sources disagree. Each note records the details under "Not established by the sources":
- 4,832 Vishay alternates (0805–2512) end in packaging code `ED`, which document 20035 lists only for 0402 and 0603.
- Two C0G parts below 10 pF use ±5%, which Yageo does not list there.
- Most Samsung numbers use design, product or control codes the catalog does not define.

## Second 5c batch: hand-written packages with a shipped datasheet (2026-11-07)

| Note | Covers | Source |
|---|---|---|
| `@espressif/esp32/ESP32_S3_WROOM_1_SERIES.md` | 13 ESP32-S3-WROOM-1 and -1U modules | Espressif WROOM-1/1U datasheet v1.8 |
| `@espressif/esp32/ESP32_S3.md`, `ESP32_S3R8.md` | The bare ESP32-S3 and ESP32-S3R8 | Espressif ESP32-S3 series datasheet v2.2 |
| `esd/ESD_USBLC6.md` | USBLC6-2SC6 | ST DS4260 Rev 7 |
| `@contrib/ldo/LDO_XC6206P182MR.md`, `RT9080.md` | XC6206P182MR, RT9080-33 (TSOT-23-5 and ZQFN) | Torex ETR0305_004b, Richtek DS9080-09 |
| `led/LED_RED_0603.md`, `LED_GREEN.md`, `RGB_SK6812.md` | Lite-On LTST-C193KRKT-5A, Würth 150060GS75000, SK6812MINI-E | Their manufacturer datasheets |
| `flash/FLASH_W25Q32.md` | W25Q32RVXHJQ | Winbond W25Q32RV Rev. E |
| `can/XCVR_SN65HVD230.md` | SN65HVD230 | TI SLOS346O |

The checker now accepts a part that names its device by path (`espressif_esp32::modules::wroom_s3::ESP32_S3_WROOM_1`). It still compares one pin table with every listed variant, so RT9080's ZQFN pad map, which differs from the TSOT one, is in its own section and was checked by hand.

Each note lists, under "Not established by the sources", where the library and its source disagree. The ones that need a decision:
- **`flash::FLASH_W25Q32` land.** The library uses KiCad's generic DFN-8 2x3, `qfn::QFN8N50P200X300_1EP61X220`, but the W25Q32RV XH package differs (Winbond p. 78):
  - Its terminals sit about ±1.275 mm from centre; the library's pad rows are at ±0.94 mm.
  - Its centre metal is 0.20 × 1.60 mm; the library's is 0.61 × 2.2 mm.
  - `examples/rpi-pico2` uses this part. It has no supplier binding.
- **`esp32` WROOM-1U footprint.** `FP_ESP32_S3_WROOM_1U` has no thermal vias, although Espressif draws them for the -1U as for the -1 (pp. 45–46).
- **`led::RGB_SK6812` land.** It is the mirror image of the source's recommended land (pp. 4–5). It is correct only for die-down mounting, which the source does not state.
- **`can` alternate SN65HVD230D.** TI lists it as Obsolete; only DR, DR.A and DRG4 are Active (p. 36).
- **Comments that go beyond the sources:**
  - XC6206: "250mA, 1.5V dropout" (p. 1 gives 200 mA, 250 mV).
  - RT9080 TSOT: "EP must solder"; the TSOT package has no EP.
  - `load-switch/src/sgm2554.cohdl` names a footprint that does not exist.

## Third 5c batch (2026-11-07)

| Note | Covers | Source |
|---|---|---|
| `usb/STUSB4500_QFN24.md`, `STUSB1602A_QFN24.md` | STUSB4500QTR, STUSB1602AQTR | ST DS12499 Rev 7, DS11254 Rev 5 |
| `usb/TUSB422_YFP9.md` | TUSB422IYFPR and -IYFPT | TI SLLSEW6C |
| `usb/PTN5110_HX2QFN16.md` | the six PTN5110 parts | NXP PTN5110 Rev. 1.6 |
| `usb/FUSB307B_QFN16.md` | FUSB307BMPX and BVMPX | onsemi FUSB307B/D Rev. 7 |
| `usb/CCG6DF.md` | CYPD6227-96BZXI | Infineon 002-27161 Rev. *K |
| `osc/XTAL_8M.md`, `CRYSTAL_4PAD.md`, `XTAL_32K768.md`, `XTAL_40MHZ.md` | The four crystal devices | Abracon ABM8, ABM8-272-T3 and ABS07; Hosonic E1SB |
| `mosfet/FET_DMG1012T.md` | DMG1012T-7 and -13 | Diodes DMG1012T |
| `load-switch/LOADSW_SGM2554.md` | SGM2554 | SG Micro SGM2554 |
| `logic/BUFFER_SN74AHCT1G125.md` | SN74AHCT1G125 | TI Rev. P |

The library-vs-source disagreements that need a decision, most serious first:
- **`load-switch::LOADSW_SGM2554` pins are wrong on three of five pads.**
  - The library has 1 IN, 2 GND, 3 EN, 4 NC, 5 OUT.
  - SGM2554A is 1 VOUT, 2 GND, 3 VIN, 4 EN, 5 VOUT; SGM2554B has VIN on pad 4 too (p. 3).
  - A board built from the library puts the supply on an output pad and leaves EN floating.
  - The MPN `SGM2554` names no model; only the A model has EN.
  - The comments give the input range as 1.0–5.5 V, where the source says 2.2–5.5 V.
  - No example uses the part.
- **`osc::XTAL_40MHZ` footprint.** It uses the 3.2 × 2.5 mm ABM8 land, but its MPN is a Hosonic E1SB part, which is 2.0 × 1.6 mm (p. 1). The MPN itself is unverified.
- **FUSB307B is not available.** onsemi's document is discontinued and marked "THIS DEVICE IS NOT AVAILABLE". Both parts are still mapped to LCSC part numbers, though neither is bound.
- **CCG6DF is rated only to 55 °C ambient** (pp. 40–41). Also:
  - every pin is `required`, although Infineon leaves many open (p. 22);
  - its oval pad length is not given by the source (pp. 24–25).
- **`lib/usb/docs/stusb1602-application.md` misstates the default mode.** The factory default is dual role with accessory support, CC_POWER_MODE 011b (Table 14, pp. 20–21), not source mode.
- **STUSB4500's application note cites Rev 8**, which is not in the repository.
- **Pin types the library cannot express.** Open-drain outputs are typed `output` or `bidirectional`, and decoupling-only regulator pins are typed `power_out`. This affects the ST, TI, NXP, onsemi and Infineon parts.

## Fourth 5c batch (2026-11-07)

| Note | Covers | Source |
|---|---|---|
| `dcdc/DCDC_JW5033S.md`, `DCDC_SGM6029C.md` | JW5033S (TSOT23-6), SGM6029C | Joulwatt Rev 0.88, SG Micro Rev A.1 |
| `@richtek/dcdc/BUCKBOOST_RT6150B.md` | RT6150B | Richtek DS6150A/B-06 and its footprint drawing |
| `@ti/dcdc/CTRL_TPS59650.md` | TPS59650 | TI SLUSAV7 |
| `@ti/power-switch/EFUSE_TPS259823ONRGET.md` | TPS259823ONRGET | TI SLVSEI3D |
| `@contrib/rtc/PCF85063A.md` | The three PCF85063A parts | NXP Rev. 7.3 |
| `@contrib/level-shifter/RS0104.md` | The four RS0104 parts | Run-IC REV A.5 |
| `@contrib/analog-switch/RS2257.md`, `SW_TMUX1072DGSR.md`, `SW_TMUX1072RUTR.md` | RS2257, TMUX1072 | Run-IC RevC.1, TI SCDS382C |
| `@contrib/comparator/COMP_TLV9022DR.md`, `COMP_TLV9032DR.md` | TLV9022, TLV9032 | TI SNOSDA3H |
| `@contrib/io-expander/IOEXP_XL9555.md` | XL9555 | Xinluda Rev 2.4 |
| `diode/SCHOTTKY_PMEG6010.md` | PMEG6010ELR | Nexperia v.5 |
| `mic/MIC_ICS43434.md` | ICS-43434 | InvenSense DS-000069 Rev 1.2 |
| `motor-driver/MOTOR_DRV8231A.md` | DRV8231A | TI SLVSFZ8A |
| `audio-amp/AMP_NS4168.md` | NS4168 | Nsiway V1.2 |
| `esim/ESIM_MFF2_TRUPHONE.md` | SIM-S-IO3-MFF2-2 | Truphone, Oct 2020 |

The library-vs-source disagreements that need a decision, most serious first:
- **`esim::ESIM_MFF2_TRUPHONE` pin map is wrong.**
  - The library has 1 VCC, 2 RST, 3 CLK, 4 IO, 5 GND.
  - The source's Figure 1 has 1 GND, 2 SWIO, 3 I/O, 4 NC, 5 NC, 6 CLK, 7 RESET, 8 VCC (PDF p. 5).
  - The footprint numbers both rows left to right, where the source puts pin 8 opposite pin 1.
  - No placement of the part works.
- **`@contrib/level-shifter` RS0104 QFN3.5x3.5-14 land: pads 1, 7, 8 and 14 sit where the package has no leads.**
  - The library has them at ±0.25 mm. E1 = 1.500 mm is the centre distance of leads 1 and 14, so they belong at ±0.75 mm (p. 25).
  - VCCA, VCCB, GND and OE would be open.
- **`qfn::QFN48N40P600X600_1EP405X360` (TPS59650) thermal land is turned 90°.** TI's 4.05 mm side runs parallel to pins 1–12 (p. 51); the library's runs across them. The agent also reads a 0.225 mm offset from the drawing.
- **`examples/joint-motor-controller` drives DRV8231A IPROPI above its limit.**
  - IPROPI is limited to 3 mA peak (p. 4).
  - The example's 3.7 A peak at 1500 µA/A is 5.55 mA.
- **JW5033S.**
  - The MPN does not fix the package, and the SOT563 version has another pinout (pp. 2, 5).
  - The output range is 0.8 V to VIN − 3 V (p. 2), so the example's 3.3 V rail needs 6.3 V in.
- **TPS25982 thermal vias** are 0.075 mm off TI's grid (p. 56).
- **TPS59650 thermal pad is analog ground**, not to be tied straight to PGND (pp. 12, 40, 43). The library calls it `EP` and gives no warning.
- **RS0104's QFN2x1.7-12 land** (used by `examples/sf32-miniboard`) leaves 0.05 mm between end and side lands (p. 28).
- **Open-drain outputs typed `output` again**: PCF85063A INT, XL9555 INT, TMUX1072 FLT_N, TLV9022 OUT, TPS59650 and TPS25982 status pins.

## Fifth 5c batch (2026-11-07)

21 notes for `@contrib` packages and two RF modules:
- `@contrib/charger/CHARGER_SGM41562B.md`;
- `@contrib/led-driver/LEDDRV_AW21009.md`;
- `@contrib/haptic/HAPTIC_AW86224.md`, `HAPTIC_DRV2605.md`;
- `@contrib/ir-emitter/IR_VSMY14940.md`;
- `@contrib/keyscan/KEYSCAN_TCA8418.md`;
- `@contrib/lora/SX1262.md`, `SX1280.md`;
- `@contrib/gnss/L76K.md`, `MIA_M10Q.md`;
- `@contrib/nfc/ST25R3916.md`;
- `@contrib/env/BME280.md`;
- `@contrib/imu/BHI260AP.md`;
- `@contrib/esd/ESD_GBLC05C.md`, `ESD_ULC0511C.md`;
- `@contrib/audio-amp/AMP_MAX98357A.md`, `AMP_NS4150B.md`;
- `@contrib/mic/MMICT3902_00_012.md`;
- `@contrib/sd-card/CONN_MICROSD.md`;
- `antenna/ANT_2450AT18B100E.md`;
- `cellular/CELL_AIR780E.md`.

**Devices with a shipped datasheet but no part.** The checker requires a note to cover a part, so these have no note yet: AXP2101, CO5300, CST9220, H0216F002AM, CH343P and HPD16A. Their pin maps were compared with their datasheets and match, except that CO5300 was not checked (226 pages).

The library-vs-source disagreements that need a decision, most serious first:
- **`cellular` Air780E footprint is mirrored top to bottom.** `FP_Luat_AIR780E_LGA109_177x158mm` copies the PADS decal's +y-up coordinates into CoHDL's +y-down frame.
  - The library has pin 1 at the lower left, with pin 2 above it. The manual's pin view has pin 1 at the top left, with pin 2 below it (p. 13).
  - Inner pad 99 is mirrored the same way.
  - No rotation of the module fits.
  - `CELL_AIR780EG` uses the same footprint.
  - Some pad shapes are also wrong: pads 89–94 should be square, and several rows are turned 90° (p. 20).
- **`@contrib/charger` SGM41562B does not charge without a host.** The library comment says it "runs on standalone defaults without a host". The power-on default is CEB = 1, battery discharge mode (p. 17).
- **`@contrib/audio-amp` MAX98357A `SD_MODE` is `optional`.** Its internal pull-down makes an open pin shutdown (pp. 7, 17).
- **`@contrib/nfc` ST25R3916 land (`QFN32N50P500X500_1EP345X345`).** The leads reach 0.15 mm further in than ST's 0.60 mm lands at ±2.35 mm (p. 153). The agent read this from Figure 39 rendered at 400 dpi.
- **`@contrib/imu` BHI260AP land (`LGA44P40_360X410`), to be reviewed by a person.** The library's 0.45 × 0.35 mm copper may be Bosch's copper plus its 50 µm mask frame. The agent reads Bosch's copper as 0.35 × 0.20 mm (Figure 37, p. 162), but the figure's dimensions are ambiguous. `examples/sf32-miniboard` uses it.
- **`@contrib/haptic` AW86224 and DRV2605 cannot share an I2C bus.** DRV2605 also answers 0x58 (drv2605 p. 23; aw86224 p. 20).
- **NS4150B's shipped PDF is the NS4150 V1.0 manual**, not an NS4150B document.
- **Pins typed `required` that the source lets float, or `optional` that it requires.**
  - ST25R3916: VDD_TX and VDD_DR are `optional` but required.
  - TCA8418: unused ROW/COL pins need pull-ups.
  - MIA-M10Q: RTC_O must go to GND if unused.
- **Open-drain or configurable outputs typed `output`.** This is a recurring library limitation.

## Sixth 5c batch (2026-11-07)

17 notes for the MCU and connector packages:
- `@raspberrypi/mcu/RP2350A_QFN60.md`;
- `@sifli/sf32/SF32LB52X_SERIES.md`;
- `@gigadevice/gd32/MCU_GD32E230F8P6TR.md`, `MCU_GD32F303CBT6.md`;
- `@wch/ch32/MCU_CH32V003F4P6.md`, `MCU_CH32V203C8T6.md`;
- `connectors/` (11 notes): JST XH battery, Molex 43045 ×3, Samtec SSW, Hirose FH12 ×3, I-PEX MHF4, a magnetic 4-pin, and the Pico SWD header.

Every MCU pad was compared with its package's pinout, and none is wrong. The checker now reads a connector's mechanical pads (`MP`).

The library-vs-source disagreements that need a decision, most serious first:
- **`connectors::CONN_BAT_2P` cannot take its part.**
  - `FP_BAT_2P_2P5` is two SMD pads with no holes.
  - The part, S2B-XH-A, is a through-hole header with φ0.9 mm holes at 2.5 mm (pp. 2, 5). JST makes no 2-circuit SMT XH (p. 6).
  - No example uses it.
- **RP2350 `RUN` is `optional`, and its comment says it "need not be routed".** The datasheet says to tie it high if unused (p. 498).
- **RP2350 `XIN`/`XOUT` are `optional`**, but BOOTSEL needs a crystal or a clock (p. 376).
- **`MCU_SF32LB52GUC6` has no source in the repository.** It is not in DS0052's ordering table (p. 60). Its basis, DS5202, is not shipped.
- **Lands not from the vendor:**
  - The SF32 land is 0.25 mm wide against a 0.20 mm maximum lead width (p. 58).
  - The GD32E230 uses KiCad's generic TSSOP-20 instead of GigaDevice's land (p. 90).
- **The CH32V203 datasheet's page 1 says "VDD 2.4~6V".** The absolute maximum is 4.0 V and the operating range 2.4–3.6 V (pp. 38–39). The library is right; the note warns agents.
- **The package application notes overstate their sources:**
  - CH32V203: with USB, VDD must be 3.0–3.6 V.
  - GD32F303: tying VBAT to VDD is not in the datasheet.
  - CH32V003: NRST behaviour.
- **`connectors` README gives the Pico SWD order as SWCLK/SWDIO/GND.** The Pico schematic has SWCLK, GND, SWDIO (p. 26). The library and `examples/rpi-pico2` are right.
- **Contact 1 is a library convention on two connectors.**
  - The magnetic 4-pin's P1 may sit next to the N magnet rather than S (p. 3). It should be checked on a physical part.
  - On the FH12, contact 1 rests on an unlabelled ▽ mark (p. 4).
- **No 5 V-tolerance marking on any MCU GPIO.** This is a library limitation.
