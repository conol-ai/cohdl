# WCH CH32

Version 0.1.0 provides two package-specific RISC-V parts:

| Part | Device | Package |
|---|---|---|
| `wch_ch32::MCU_CH32V003F4P6` | `CH32V003F4_TSSOP20` | TSSOP20 |
| `wch_ch32::MCU_CH32V203C8T6` | `CH32V203C8_LQFP48` | LQFP48 |

```toml
[dependencies]
"@wch/ch32" = "0.1.0"
std = "0.3.0"
```

Read [CH32V003 integration](ch32v003.md) or [CH32V203 integration](ch32v203.md)
before use. [Sources](sources.md) records exact document versions and hashes.
The retained documents are WCH-authored datasheets retrieved from mirrors
because the manufacturer download endpoint returned a retry error.

Power pins require explicit connections; CH32V203 BOOT0 is also required.
Use `nc:` for unused optional pins. GPIO declarations represent physical
terminals, not independent pins for every alternate function. Firmware must
select the matching peripheral routing. No STM32 or GD32 AVL alternates are
declared. WCH debug signaling requires a compatible probe and software.

Both parts use attributed generic KiCad lands from the shared `soic`/`qfp`
packages. Pin numbering, nominal dimensions and emitted copper are checked;
these are not WCH-recommended PCB lands. WCH's broad package tolerance note
does not establish full tolerance-stack compatibility or assembly acceptance.
Confirm actual package tolerances, land/stencil geometry and placement with
the assembler before fabrication. The compiler also does not verify supply
sequencing, peripheral multiplexing, boot configuration or clock accuracy.
