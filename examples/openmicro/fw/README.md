# OpenMicro firmware

Rust/[embassy](https://embassy.dev) firmware for the OpenMicro macropad
(STM32F072CBT6). Async, no RTOS, no unsafe outside the vendored HAL.

Build stats (release, LTO, `opt-level = "s"`): ~20 KiB flash of 128 KiB,
~5 KiB static RAM of 16 KiB.

## Pin map — where it comes from

The pin assignment is **generated from the board design**, not chosen here:
`../src/openmicro_parts.cohdl` (the `STM32F072CBT6` device block) is the
single source of truth — it is the position-aware GPIO map that made the
board routable. If the `.cohdl` changes, update the table at the top of
`src/main.rs` to match.

| Function | Pins |
|---|---|
| Matrix rows (out, drive-high) | ROW0 `PA9` · ROW1 `PB3` · ROW2 `PB6` · ROW3 `PB5` |
| Matrix cols (in, pull-down) | COL0 `PB8` · COL1 `PB7` · COL2 `PA15` · COL3 `PA10` |
| Rotary encoder | A `PC13` · B `PC14` · push `PC15` |
| Joystick | X `PB1`/ADC_IN9 · Y `PB0`/ADC_IN8 · push `PA8` |
| Touch pad | `PB9` (RC charge-time sensing) |
| RGB data | per-key chain (13× SK6812MINI-E) `PB4` · underglow ring (16×) `PA0` |
| USB FS | DM `PA11` · DP `PA12` |
| SWD (J2) | SWDIO `PA13` · SWCLK `PA14` |

Clocking: HSI48 with CRS sync from USB SOF drives both the core (48 MHz —
the WS2812 bit-bang cycle counts assume it) and the USB peripheral. The
8 MHz HSE crystal on the board is fitted belt-and-braces but not required.

## What it does

A composite USB HID device (VID `0x1209` pid.codes, keyboard + consumer
control):

- **13 keys → F13…F24** — 1 kHz matrix scan, 5 ms debounce, COL2ROW diodes.
  The two switches under the 2U keycap (sw10/sw11) both send F23.
- **Encoder → volume** up/down, push → mute.
- **Touch pad → play/pause.**
- **Joystick → arrow keys** (ADC thresholds), push → Enter.
- **LEDs**: pressed keys light white over an idle rainbow; the underglow
  ring rotates hue. Brightness is capped in `ws2812.rs` (`scaled(n/64)`)
  to keep all 29 LEDs inside the 500 mA VBUS budget.

## Building

```sh
rustup target add thumbv6m-none-eabi
cargo build --release
```

The workspace/CI at the repo root does not build this crate (it needs the
thumb target); it lives here as part of the example board's deliverables.

## Flashing

**SWD (J2 header), recommended** — with a probe (ST-Link, CMSIS-DAP, …)
attached to J2:

```sh
cargo install probe-rs-tools
cargo run --release        # runner = probe-rs run --chip STM32F072CBTx
```

**USB DFU, no probe needed** — the F072's ROM bootloader enumerates on the
same USB port. Hold BOOT0 high while plugging in, then:

```sh
cargo install cargo-binutils && rustup component add llvm-tools
cargo objcopy --release -- -O binary openmicro.bin
dfu-util -a 0 -s 0x08000000:leave -D openmicro.bin
```
