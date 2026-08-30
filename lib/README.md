# `lib/` — the libraries that ship with the compiler

Every bare-name child here is a package **family dir** (RFC-029): either a
package itself (`cohdl.toml` + `src/`) or a container of side-by-side
versions in arbitrarily-named subdirectories that each are one. Manufacturer
scopes use `lib/@scope/name/`. Nothing is path-privileged — `lib/std/`
resolves by exactly the rule `lib/passive/` does.

A dependency name resolves through `<project>/deps/<name>`, then
`<lib>/<name>`, then the RFC-030 cache (`~/.cohdl/registry`). The compiler
finds this directory by walking up from its own executable (then the current
directory) for a `lib/` that offers at least one readable package.

`std` is deliberately core-traits-only. Devices, purchasable parts, pads, and
footprints live in focused packages:

| Package | Representative source path | Owns |
|---|---|---|
| `std` | unqualified prelude traits such as `IC` and `Connector` | universal component contracts only |
| `passive` | `passive::MLCC` | generated chip resistors, MLCCs, chip inductors, lands, and passive helper circuits |
| `qfn` | `qfn::ESPRESSIF_QFN56_0P4_7B` | QFN, DFN, and SON lands; each part binding still requires package-specific qualification |
| `qfp` | `qfp::QFP50P900X900X160_48N` | audited QFP package land patterns, separated from manufacturer component libraries |
| `bga` | `bga::BGA64C50P8X8_500X500X60N` | audited BGA package land patterns with exact populated-ball maps |
| `csp` | `csp::KICAD_ST_WLCSP_49_DIE448` | attributed chip-scale package lands with exact populated-ball maps |
| `soic` | `soic::SOIC8P127X790X216N` | audited SOIC and TSSOP package land patterns |
| `connectors` | `connectors::headers::micro_fit_3::MOLEX_43045_0612` | general-purpose board connectors, keyed power/actuator harnesses, and headers |
| `usb` | `usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12` | USB connectors, including Type-C and Micro-B, and controllers |
| `esd` | `esd::ESD_USBLC6` | ESD protection devices |
| `diode` | `diode::D_1N4148W` | discrete diodes |
| `flash` | `flash::FLASH_W25Q32`, `flash::FLASH_W25Q128JVSIQ` | nonvolatile NOR flash memories |
| `ldo` | `ldo::LDO_AP2112K_3V3`, `ldo::LDO_RT9193_15PB` | low-dropout regulators |
| `led` | `led::LED_RED_0603` | discrete/addressable LEDs and their traits |
| `mosfet` | `mosfet::FET_DMG1012T` | discrete MOSFETs |
| `osc` | `osc::XTAL_8M`, `osc::XTAL_40MHZ` | crystals and oscillators |
| `mic` | `mic::MIC_ICS43434` | microphones and the `Microphone` trait |
| `audio-amp` | `audio_amp::AMP_NS4168` | audio power amplifiers with verified land patterns |
| `dcdc` | `dcdc::DCDC_JW5033S` | DC/DC step-down converters with verified land patterns |
| `logic` | `logic::BUFFER_SN74AHCT1G125` | logic gates and buffers with verified land patterns |
| `antenna` | `antenna::ANT_2450AT18B100E` | RF antennas with verified terminal lands and layout guidance |
| `misc` | `misc::TEST_POINT_ROUND_1MM`, `misc::MOUNTING_HOLE_M2_NPTH`, `misc::MOUNTING_HOLE_M2_PTH` | reusable PCB fabrication primitives with explicit electrical semantics |
| `cellular` | `cellular::CELL_AIR780E` | 4G/LTE cellular module devices with verified land patterns (Air780E, 109-pin LGA, official PADS decal land) |
| `esim` | `esim::ESIM_MFF2_TRUPHONE` | eUICC (eSIM) devices with verified land patterns |
| `load-switch` | `load_switch::LOADSW_SGM2554` | power-distribution load switches with verified land patterns |
| `@ti/dcdc` | `ti_dcdc::controllers::multiphase::CTRL_TPS59650` | TI DC/DC controllers |
| `@ti/logic` | `ti_logic::LS_SN74LVC8T245PWR` | TI logic and dual-supply level translators |
| `@ti/power-switch` | `ti_power_switch::EFUSE_TPS259823ONRGET` | TI protected power paths and eFuses |
| `@st/stm32` | `st_stm32::STM32F103C8Tx`, `st_stm32::MCU_STM32F103C8T6`, `st_stm32::MCU_STM32F072CBT6` | Generated ST-source-backed STM32 device catalog plus source-joined exact parts with attributed dependency-owned fabrication geometry |
| `@espressif/esp32` | `espressif_esp32::ESP32_C3`, `espressif_esp32::ESP32_C6_WROOM_1_N8`, `espressif_esp32::chips::s3::ESP32_S3R8` | Pinned-source generated ESP32-lineage SoCs and modules, exact MPN parts, and vendor module land patterns |
| `@contrib/imu` | `contrib_imu::IMU_BHI260AP` | community-contributed IMU devices and public, manufacturer-land-pattern part bindings |
| `@contrib/charger` | `contrib_charger::CHARGER_SGM41562B` | community-contributed battery charger / power-path devices and parts |
| `@contrib/env` | `contrib_env::ENV_BME280` | community-contributed environmental sensor devices and parts |
| `@sifli/sf32` | `sifli_sf32::MCU_SF32LB52EUB6` | SiFli SF32LB52X device and qualified EUB6 package binding |
| `@contrib/io-expander` | `contrib_io_expander::IOEXP_XL9555` | community-contributed I2C/SMBus I/O expander variants and a public TSSOP24 binding with an independently derived land pattern |
| `@raspberrypi/mcu` | `raspberrypi_mcu::RP2350A_QFN60` | Raspberry Pi microcontrollers |
| `@richtek/dcdc` | `richtek_dcdc::buck_boost::rt6150b::BUCKBOOST_RT6150B` | Richtek DC/DC converters |

Scoped manifest names are quoted (`"@ti/dcdc"`, `"@raspberrypi/mcu"`); their
CoHDL package namespace is normalized (`ti_dcdc`, `raspberrypi_mcu`).
Focused footprint dependencies such as `qfn`, `qfp`, and `bga` travel through
the transitive package closure. A root design repeats one of those pins only
when it intentionally uses RFC-029's root-pin authority to choose its version.

Existing projects can use the
[std 0.3 migration table](../docs/std-0.3-migration.md) to replace former
std paths with their new package-qualified paths.

To add an official library: create `lib/<name>/` (or
`lib/@scope/<name>/`) with a `cohdl.toml` whose `[package] name` matches the
path and whose `[package] version` is an exact `X.Y.Z`, put its sources under
`src/`, and publish that package directory with `cohdl publish`. Projects pin
it under `[dependencies]` like any other package. The resolver needs no
change.

Two rules the layout enforces by being what it is: a package's version comes
only from its manifest (a directory name that spells a version is
convention), and a released version's content is immutable — changing it
under the same version is a hard `E1103` against every `cohdl.lock` that
pinned it.

Additional community-contributed packages:

| Package | Representative source path | Owns |
|---|---|---|
| `@contrib/haptic` | `contrib_haptic::HAPTIC_AW86224` | community-contributed haptic / vibration driver devices and parts |
| `@contrib/analog-switch` | `contrib_analog_switch::SW_RS2257XH` | community-contributed analog switch / multiplexer devices and parts |
| `@contrib/led-driver` | `contrib_led_driver::LEDDRV_AW21009` | community-contributed LED driver devices and parts |
| `@contrib/keyscan` | `contrib_keyscan::KEYSCAN_TCA8418` | community-contributed keypad / keyboard scanner devices and parts |
| `@contrib/rtc` | `contrib_rtc::RTC_PCF85063AT` | community-contributed real-time clock devices and parts |
| `@contrib/gnss` | `contrib_gnss::GNSS_L76K`, `contrib_gnss::GNSS_MIA_M10Q` | community-contributed GNSS modules with public manufacturer-land-pattern bindings |
| `@contrib/ldo` | `contrib_ldo::LDO_RT9080_33_ZQFN`, `contrib_ldo::LDO_XC6206P182MR` | community-contributed low-dropout regulators with exact-manufacturer and independently derived lands, respectively |
| `@contrib/usb-uart` | `contrib_usb_uart::CH343P` | community-contributed USB-to-UART device model; CH343P part binding quarantined |
| `@contrib/audio-amp` | `contrib_audio_amp::AMP_MAX98357A`, `contrib_audio_amp::AMP_NS4150B` | community-contributed audio amplifiers with public exact-manufacturer and independently derived lands, respectively |
| `@contrib/pmu` | `contrib_pmu::AXP2101` | community-contributed PMU device model; AXP2101 part binding quarantined |
| `@contrib/lora` | `contrib_lora::LORA_SX1262` | community-contributed LoRa transceiver devices and parts |
| `@contrib/nfc` | `contrib_nfc::NFC_ST25R3916` | community-contributed NFC reader/IC devices and parts |
| `@contrib/display` | `contrib_display::H0216F002AM` | community-contributed display module interface; part binding quarantined |
| `@contrib/display` (CO5300) | `contrib_display::CO5300` | AMOLED driver IC (COF, device-only) |
| `@contrib/display` (CST9220) | `contrib_display::CST9220` | capacitive touch IC (device-only) |
| `@contrib/esd` | `contrib_esd::ESD_GBLC05C`, `contrib_esd::ESD_ULC0511C` | community-contributed ESD protection / TVS devices and public parts |
| `@contrib/level-shifter` | `contrib_level_shifter::LS_RS0104` | community-contributed voltage translators; all four RS0104 order codes have public package-specific bindings |
| `@contrib/ir-emitter` | `contrib_ir_emitter::IR_VSMY14940` | community-contributed infrared emitting diode devices and parts |
| `@contrib/mic` | `contrib_mic::MMICT3902_00_012` | TDK/InvenSense T3902 PDM microphone with manufacturer annular land and segmented stencil |
| `@contrib/sd-card` | `contrib_sd_card::CONN_MICROSD` | community-contributed SD/MicroSD card connector devices and parts |

## Audited part-binding status

Here, *quarantined* has a precise meaning: the logical `pub device` remains
available, but there is deliberately no `pub part`, so a design cannot select
an orderable component or emit an unqualified footprint by accident.

The current audit promoted these bindings using manufacturer-recommended PCB
lands directly. This list describes the new work, not every already-qualified
part in `lib/`.

| Public part | Exact orderable identity | Land-pattern evidence |
|---|---|---|
| `contrib_imu::IMU_BHI260AP` | Bosch Sensortec `BHI260AP` | Bosch Figure 37, including its eight paired inner-ring copper chamfers and 50 um mask frame |
| `sifli_sf32::MCU_SF32LB52EUB6` | SiFli `SF32LB52EUB6` | SiFli Figure 5-2 package envelope plus the pinned official footprint generator, including segmented paste, repeated exposed-pad copper, and 16 thermal vias |
| `contrib_gnss::GNSS_L76K` | Quectel `L76K` | Quectel hardware-manual recommended 18-LCC land |
| `contrib_gnss::GNSS_MIA_M10Q` | u-blox `MIA-M10Q-00B` | u-blox integration-manual copper, mask, and paste dimensions |
| `contrib_ldo::LDO_RT9080_33_ZQFN` | Richtek `RT9080-33GQZ` | Richtek section 20.2 asymmetric ZQFN land, including four inner chamfers and the isolated SGND diamond |
| `contrib_rtc::RTC_PCF85063ATL` | NXP `PCF85063ATL/1,118` | NXP Figure 37 reflow footprint, including mask clearance and the isolated-paddle paste deposit |
| `contrib_audio_amp::AMP_MAX98357A` | Analog Devices `MAX98357AETE+` | Analog Devices land pattern 90-0031 rev C |
| `passive::IND_4U7_WPN201610U` | Sunlord `WPN201610U4R7MT` | Sunlord WPN-series recommended 2016 land |
| `osc::XTAL_48MHZ` | Hosonic `E1SB48E001G00E` | Hosonic E1SB recommended land; SiFli-certified 22 ohm maximum ESR selection |
| `osc::XTAL_32K768` | Abracon `ABS07-32.768KHZ-9-T` | Abracon ABS07 recommended land |

The following are also public part bindings with exact manufacturer identity
and pin assignment, but their lands are independent nominal-density
derivations from complete component dimensions rather than manufacturer PCB
recommendations. A board fabricator and assembler must qualify these lands
for the intended copper, mask, stencil, placement, and reflow process.

| Public part | Derivation status and production caveat |
|---|---|
| `contrib_io_expander::IOEXP_XL9555` | TSSOP24 land derived from Xinluda's complete lead tolerances; 0.23 mm adjacent copper clearance |
| `contrib_ldo::LDO_XC6206P182MR` | SOT-23 land derived from Torex's component envelope; it is not a Torex-recommended PCB pattern |
| `contrib_audio_amp::AMP_NS4150B` | MSOP-8 land derived with stated IPC-7351 goals; requires support for its 0.20 mm minimum copper gap |
| `contrib_esd::ESD_ULC0511C` | DFN1006DN land derived with stated IPC-7351 goals; requires explicit fabrication approval for its 0.15 mm opposing-land gap |
| `contrib_level_shifter::LS_RS0104_QFN14` | QFN3.5x3.5-14L land independently derived from Run-IC package tolerances |
| `contrib_level_shifter::LS_RS0104_QFN12_2X2` | QFN2x2-12L land independently derived from Run-IC package tolerances |
| `contrib_level_shifter::LS_RS0104_QFN12_2X1P7` | QFN2x1.7-12L land independently derived from Run-IC package tolerances |

These devices remain quarantined:

| Device | Why no public part exists | Evidence needed to release it |
|---|---|---|
| `contrib_usb_uart::CH343P` | WCH publishes component dimensions, but not a PCB land/stencil pattern or a complete tolerance set suitable for a defensible derivation; the former inferred exposed pad was the wrong size. | Manufacturer land/paste data or complete package tolerances supporting an auditable derivation |
| `contrib_pmu::AXP2101` | The public data sheet gives only the QFN5x5 component outline, not the PCB copper and paste pattern. | Manufacturer land/stencil data or another authoritative package source |
| `contrib_lora::HPD16A` | The series sheet lacks both a frequency-specific orderable identity and a recommended PCB land. | Exact module order code plus its recommended host-board land |
| `contrib_lora::HPB16B3` | The official LilyGO schematic establishes the 12-pin order only; module geometry, land pattern, supply limits, and internal-radio specification are unpublished. It is not the 16-pin HPD16A. | Standalone module electrical/mechanical specification and recommended land |
| `contrib_display::H0216F002AM` | The module sheet ambiguously names two 31-contact connector codes and supplies no board-side land, anchors, mating orientation, or keep-out. | An official drawing that identifies the actual board-side mate and its recommended PCB land |
| `contrib_display::CO5300` | This is a chip-on-film display driver bonded inside the panel, not a PCB-mounted component. | None expected; board designs should instantiate a qualified display module instead |
| `contrib_display::CST9220` | Only the module-exposed touch interface is public; the bare-controller package data are NDA-only. | Public bare-device pinout, exact orderable identity, package drawing, and recommended land |
