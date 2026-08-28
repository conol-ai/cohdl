# Migrating from `std` 0.2 to 0.3

`std` 0.3 is a core-traits-only prelude. It exports `TwoTerminal`,
`Capacitor`, `Resistor`, `Polarized`, `Diode`, `IC`, and `Connector`; component
devices, parts, footprints, and pads now live with their domain libraries.

Update exact dependency pins and regenerate `cohdl.lock` with `cohdl update`.
The moved declarations have these new qualified paths:

| `std` 0.2 declaration family | `std` 0.3 replacement |
|---|---|
| HRO USB-C receptacle | `usb::connectors::type_c::*` |
| Infineon CCG6DF controller | `usb::pd::ccg6df::*` |
| AP2112K LDO | `ldo::*` |
| TPS59650 controller | `ti_dcdc::controllers::multiphase::*` |
| `ChipLED` and `LED_RED_0603` | `led::*` |
| `Microphone` and ICS43434 | `mic::*` |
| ESP32-S3-WROOM-1 | `espressif_esp32::modules::wroom_s3::*` |

The duplicated USBLC6 protection array formerly carried by the example boards
is now `esd::ESD_USBLC6`. OpenMicro's SK6812 slice is now
`led::RGB_SK6812`.

Scoped package names are quoted in TOML and normalize to underscore-separated
source roots:

```toml
[dependencies]
"@espressif/esp32" = "0.1.0"
"@raspberrypi/mcu" = "0.1.0"
"@richtek/dcdc" = "0.1.0"
"@st/stm32" = "0.2.0"
"@ti/dcdc" = "0.1.0"
connectors = "0.1.0"
diode = "0.1.0"
esd = "0.1.0"
flash = "0.1.0"
ldo = "0.1.0"
led = "0.1.0"
mic = "0.1.0"
mosfet = "0.1.0"
osc = "0.1.0"
passive = "0.2.0"
std = "0.3.0"
usb = "0.1.0"
```

Only the Raspberry Pi Pico 2 example's board-specific 40-pin edge and BOOTSEL
switch remain in its local part, pad, and footprint files.

This is an intentional breaking move. CoHDL has no re-export or deprecation
alias mechanism, and retaining duplicate purchasable parts in two loaded
packages would violate AVL identity checks.

## OpenMicro-local catalog follow-up

Reusable declarations that originally lived only in OpenMicro were moved into
the same package structure:

| Former OpenMicro declaration | Package-qualified replacement |
|---|---|
| `XTAL_8M` and its 3225 land pattern | `osc::*` |
| `D_1N4148W` and its SOD-123 land pattern | `diode::*` |
| `J_SWD` 2×3 socket | `connectors::headers::smd_254::SOCKET_2X3_254_SMD` |
| `MCU_STM32F072` | `st_stm32::MCU_STM32F072CBT6` |

The exact STM32F072CBT6 part now binds the generated family device and the
dependency-owned `qfp` land pattern. It exposes datasheet GPIO names instead
of OpenMicro-specific aliases. Board functions remain explicit in the design
nets—for example, OpenMicro's `ROW0` uses `mcu.PA9`, USB D− uses `mcu.PA11`,
and SWDIO uses `mcu.PA13`.

## Raspberry Pi Pico 2-local catalog follow-up

Reusable declarations that originally lived only in the Pico 2 example now
live in the same focused package structure:

| Former Pico 2 declaration | Package-qualified replacement |
|---|---|
| `RP2350A_QFN60` | `raspberrypi_mcu::RP2350A_QFN60` |
| `BUCKBOOST_RT6150B` | `richtek_dcdc::buck_boost::rt6150b::BUCKBOOST_RT6150B` |
| `FLASH_W25Q32` | `flash::FLASH_W25Q32` |
| `XTAL_12MHZ` | `osc::XTAL_12MHZ` |
| `USB_MICRO_B` | `usb::connectors::micro_b::USB_MICRO_B` |
| `HEADER_SWD_3W` | `connectors::headers::castellated_254::HEADER_SWD_3W` |
| `SCHOTTKY_PMEG6010` | `diode::SCHOTTKY_PMEG6010` |
| `FET_DMG1012T` | `mosfet::FET_DMG1012T` |
| `IND_2U2`, `IND_3U3` | `passive::IND_2U2`, `passive::IND_3U3` |
| `LED_GREEN` | `led::LED_GREEN` |

The flash API uses the device's generic `IO0`–`IO3` and `CLK` names instead
of the Pico-local `SD0`–`SD3` and `SCLK` aliases. The generic 1×3 connector
uses `P1`, `P2`, and `P3`; the Pico design maps those contacts to SWCLK, GND,
and SWDIO respectively.
