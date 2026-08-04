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
| `qfn` | `qfn::QFN56N40P700X700_1EP400X400` | verified QFN, DFN, and SON package land patterns |
| `connectors` | `connectors::headers::smd_254::SOCKET_2X3_254_SMD` | general-purpose board connectors and headers |
| `usb` | `usb::connectors::type_c::USB_C_HRO_TYPE_C_31_M_12` | USB connectors, including Type-C and Micro-B, and controllers |
| `esd` | `esd::ESD_USBLC6` | ESD protection devices |
| `diode` | `diode::D_1N4148W` | discrete diodes |
| `flash` | `flash::FLASH_W25Q32` | nonvolatile flash memories |
| `ldo` | `ldo::LDO_AP2112K_3V3` | low-dropout regulators |
| `led` | `led::LED_RED_0603` | discrete/addressable LEDs and their traits |
| `mosfet` | `mosfet::FET_DMG1012T` | discrete MOSFETs |
| `osc` | `osc::XTAL_8M` | crystals and oscillators |
| `mic` | `mic::MIC_ICS43434` | microphones and the `Microphone` trait |
| `@ti/dcdc` | `ti_dcdc::controllers::multiphase::CTRL_TPS59650` | TI DC/DC controllers |
| `@st/stm32` | `st_stm32::f0::stm32f072cb::MCU_STM32F072` | STMicroelectronics STM32 MCUs |
| `@espressif/esp32` | `espressif_esp32::chips::s3::ESP32_S3` | Espressif SoCs and modules |
| `@contrib/imu` | `contrib_imu::IMU_BHI260AP` | community-contributed IMU devices and parts |
| `@contrib/charger` | `contrib_charger::CHARGER_SGM41562B` | community-contributed battery charger / power-path devices and parts |
| `@contrib/env` | `contrib_env::ENV_BME280` | community-contributed environmental sensor devices and parts |
| `@contrib/sf32` | `contrib_sf32::MCU_SF32LB52EUB6` | community-contributed SiFli SF32LB52X MCU devices and parts |
| `@contrib/io-expander` | `contrib_io_expander::IOX_XL9555_QFN24` | community-contributed I2C/SMBus I/O expander devices and parts |
| `@raspberrypi/mcu` | `raspberrypi_mcu::RP2350A_QFN60` | Raspberry Pi microcontrollers |
| `@richtek/dcdc` | `richtek_dcdc::buck_boost::rt6150b::BUCKBOOST_RT6150B` | Richtek DC/DC converters |

Scoped manifest names are quoted (`"@ti/dcdc"`, `"@raspberrypi/mcu"`); their
CoHDL package namespace is normalized (`ti_dcdc`, `raspberrypi_mcu`).
Because dependency loading is intentionally direct-only, a design that
instantiates a part whose qualified footprint lives in `qfn` also lists
`qfn = "0.1.0"` in its own manifest.

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
| `@contrib/haptic` | `contrib_haptic::HAPTIC_AW86224` | community-contributed haptic / vibration driver devices and parts |
| `@contrib/analog-switch` | `contrib_analog_switch::SW_RS2257XH` | community-contributed analog switch / multiplexer devices and parts |
| `@contrib/led-driver` | `contrib_led_driver::LEDDRV_AW21009` | community-contributed LED driver devices and parts |
| `@contrib/keyscan` | `contrib_keyscan::KEYSCAN_TCA8418` | community-contributed keypad / keyboard scanner devices and parts |
| `@contrib/rtc` | `contrib_rtc::RTC_PCF85063AT` | community-contributed real-time clock devices and parts |
| `@contrib/gnss` | `contrib_gnss::GNSS_L76KB_A58` | community-contributed GNSS / GPS module devices and parts |
