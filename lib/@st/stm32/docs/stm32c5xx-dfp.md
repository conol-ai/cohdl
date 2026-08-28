# STM32C5 device-family-pack provenance

This reference covers generated STM32C5 device declarations at the
`st_stm32` package root.

- Publisher: STMicroelectronics
- Repository: <https://github.com/STMicroelectronics/stm32c5xx-dfp>
- Release tag: `2.1.0`
- Commit: `a5f65bc64535cfa723e9d25f58d7ce23d0937aed`
- Commit date: 2026-06-15
- Retrieved: 2026-08-27
- License: BSD-3-Clause; the upstream notice is stored in
  `STM32C5XX_DFP_LICENSE.md`.
- Frozen normalized snapshot: `tools/stm32_data/pin_data.json` at repository
  root, shared with the open-pin-data import and identified in
  `stm32-open-pin-data.md`.

ST describes the pack as its IDE/toolchain device support for STM32C5. The
generator reads exact `Dvariant` identities and their inherited pinout
descriptors from `STMicroelectronics.stm32c5xx_dfp.pdsc`, then reads package
positions and die-pad functions from the referenced JSON files. Duplicate
`TR` packaging records collapse to the same electrical device because tape
and reel does not change its pin map.

Pinouts marked `package_type=4-edges-internal` are excluded. Their bond lists
enumerate only the numbered perimeter positions and omit the internal exposed
pad, so they cannot support a complete CoHDL device declaration without an
audited datasheet injection.

Pin roles and obligations use the same closed, fail-closed semantic policy
documented in `stm32-open-pin-data.md`; the DFP's GPIO/function classes are
not treated as a substitute for product-specific board requirements.

The pack supplies exact device identities and package pinouts, but it does not
provide a certified CoHDL land pattern. These declarations therefore remain
devices rather than fabrication-facing parts. A future `pub part` must add a
local current datasheet, exact MPN, and source-backed complete footprint under
the ordinary `LIBRARY.md` review rules.
