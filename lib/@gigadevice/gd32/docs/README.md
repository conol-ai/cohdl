# GigaDevice GD32

Version 0.1.0 provides two exact parts at the package root:

| Part | Device | Package |
|---|---|---|
| `gigadevice_gd32::MCU_GD32E230F8P6TR` | `GD32E230F8_TSSOP20` | TSSOP20 |
| `gigadevice_gd32::MCU_GD32F303CBT6` | `GD32F303CB_LQFP48` | LQFP48 |

```toml
[dependencies]
"@gigadevice/gd32" = "0.1.0"
std = "0.3.0"
```

Read [GD32E230 integration](gd32e230.md) or [GD32F303 integration](gd32f303.md)
before connecting a part. Pin maps are package-specific and traced to the
retained manufacturer documents in [sources](sources.md). A similar STM32
name does not establish compatible silicon, peripherals, firmware, or pin
behavior. No cross-vendor AVL alternates are declared.

Power pins and BOOT0 require explicit connections. Other pins are optional:
use `nc:` for intentionally unused terminals. GPIO roles describe the physical
multifunction terminal; firmware still selects the peripheral and remap.
The compiler does not verify pin multiplexing, boot levels, per-domain voltage
limits, analog performance, crystal selection, or decoupling placement.

Reusable lands are dependencies of this package. F303 uses copper matching
the manufacturer's recommendation. E230 uses an attributed generic KiCad
land, with the difference documented explicitly. Assembly acceptance remains
specific to the selected component, PCB and stencil process.
