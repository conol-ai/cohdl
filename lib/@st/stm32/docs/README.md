# Official documentation

## Generated device catalog

The broad device catalog is generated from two pinned, BSD-3-Clause-licensed
STMicroelectronics repositories. The checked-in normalized snapshot and
generator make the import reproducible without network access:

- [`stm32-open-pin-data.md`](stm32-open-pin-data.md) records the provenance,
  transformation rules, and limits for STM32CubeMX 6.18.0 pin data.
- [`stm32c5xx-dfp.md`](stm32c5xx-dfp.md) records the equivalent information
  for the STM32C5 device-family pack 2.1.0.
- [`STM32_OPEN_PIN_DATA_LICENSE.txt`](STM32_OPEN_PIN_DATA_LICENSE.txt) and
  [`STM32C5XX_DFP_LICENSE.md`](STM32C5XX_DFP_LICENSE.md) preserve the upstream
  license notices.

These sources establish package-specific physical pin identity. They do not
establish recommended land geometry or turn wildcard product patterns into
purchasable order codes. The generator emits `pub device` models broadly, but
emits an exact `pub part` only from the separate audited-part inventory after
checking its exact order code, local datasheet checksum, generated device, and
qualified dependency-owned footprint. Unsupported order codes remain devices;
they are never promoted to parts by guessing from a broad package label.

The emitted families are C0/C5, F0/F1/F2/F3/F4/F7, G0/G4, H5/H7,
L0/L1/L4/L4+/L5, N6, U0/U3/U5, WB/WB0/WBA, and WL. Families or package
variants with no complete source-backed physical pinout remain visible in the
generated coverage ledger rather than disappearing silently. STM32MP
application processors are an explicit product-category boundary: they are
not imported into this MCU package and require a separate interface and
package policy.

The audited overlay currently adds 14 F072 parts covering 22 exact order-code
rows in LQFP48, LQFP64, LQFP100, UFBGA64, and UFBGA100 packages. Its reusable
copper geometry lives in `qfp` and `bga`. DS9826's UFQFPN parts are not emitted
because the broad pin source omits their required exposed pad; its WLCSP parts
remain devices because the document gives bump mechanics but no recommended
PCB land diameter.

The 2,284-device catalog produces a large API-docs schema-v1 sidecar. Registry
uploads remove insignificant JSON whitespace for transport, and the registry's
200 MB application limit accommodates the complete catalog without silently
dropping device coverage. The browser fetches this document only when the API
tab or an item deep link is selected. Its compact sidecar exceeds the Worker's
16,000,000-byte in-memory indexing threshold, so it is streamed and remains
fully browsable but does not contribute part rows to registry search.

## `stm32f072cb-datasheet.pdf`

- Covers: `STM32F072CBT6` in the LQFP-48 7 x 7 mm package.
- Document identity: DS9826 Rev 6, 2019-09-03.
- Official URL:
  <https://www.st.com/resource/en/datasheet/stm32f072cb.pdf>
- Retrieval mirror:
  <https://www.mouser.cn/datasheet/2/389/stm32f072c8-1851113.pdf>
- Retrieved: 2026-07-30.
- SHA-256:
  `660bc16b5cf99b649da4ebcd2d1bb8d232edd3fb1dc275977b3c6389a85d2453`
- Footprint authority: Figure 50, "Recommended footprint for LQFP48
  package."
- Exact order-code authority: the ordering-information scheme identifies
  `STM32F072CBT6` as the tray code and the `TR` suffix as tape-and-reel
  packing. Both codes resolve to one generated part identity.
- Geometry ownership: `qfp::QFP50P900X900X160_48N` in the focused `qfp`
  package. The STM32 package owns the device/part binding and datasheet, not
  the reusable copper geometry.

The official global and China ST endpoints were available for browser review
on 2026-07-30 but timed out during binary retrieval. The stored file is the
same ST-authored DS9826 Rev 6 production datasheet served by authorized
distributor Mouser; its cover, document identity, pin tables, and LQFP-48
recommended-land page were independently checked after download.
