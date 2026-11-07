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
