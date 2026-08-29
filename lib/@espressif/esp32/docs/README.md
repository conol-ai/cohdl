# Espressif ESP32 reference documents

The generated catalog uses the local deterministic source index in
`esp32-part-catalog.md`; it accounts for all 318 exact MPNs in the frozen
Espressif Product Selector response, including an explicit reason for every
omitted row. The index records the official document URL, lifecycle, exact
device/pin source, concrete footprint, upstream revisions, retrieval date, and
snapshot hashes for every admitted part. This catalog-scale index avoids
duplicating hundreds of datasheet binaries in the immutable registry archive.

The two PDFs retained here are unmodified Espressif downloads used by the four
preserved, hand-audited ESP32-S3 declarations.

| File | Applies to | Official source | Version |
|---|---|---|---|
| `esp32-s3_datasheet_en.pdf` | Bare `ESP32-S3` and `ESP32-S3R8` SoCs | <https://www.espressif.com/sites/default/files/documentation/esp32-s3_datasheet_en.pdf> | v2.2 (2026-03-05) |
| `esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf` | `ESP32-S3-WROOM-1-N8` and `ESP32-S3-WROOM-1-N8R2` | <https://www.espressif.com/sites/default/files/documentation/esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf> | v1.8 (2026-03-02) |

SHA-256 checksums:

```text
2d5a7cb7fd559d8d972bd88db32669c0196d23f22d7afaafb0f63d099b589a3f  esp32-s3_datasheet_en.pdf
27d71971da07c280c6068d08c74720d1a25b8f20cf8494dc1765bdd28d40d435  esp32-s3-wroom-1_wroom-1u_datasheet_en.pdf
```

Footprint geometry was checked against the unmodified CAD sources linked by
these datasheets. Both unversioned files were retrieved from Espressif on
2026-07-30:

| CAD source | Footprint coverage | SHA-256 |
|---|---|---|
| [`ESP32-S3_Footprint.asc`](https://www.espressif.com/sites/default/files/chips-dxf/ESP32-S3_Footprint.asc) | `qfn::ESPRESSIF_QFN56_0P4_7B` | `2f0e059f9ecef4350413068280a14e6f1f3e3792f512116b27138d4a6810fb5c` |
| [`ESP32-S3-WROOM-1 PCB Footprint.dxf`](https://www.espressif.com/sites/default/files/modules-dxf/ESP32-S3-WROOM-1%20PCB%20Footprint.dxf) | `FP_ESP32_S3_WROOM_1` | `565cb080dc99eb49e6e5ccfa97da0295ae1f1dc8113ff83ae10575a1e64f77e4` |

The module footprint remains in this package; the bare SoC footprint is
exported by the dedicated `qfn` package.

The complete 55-footprint frozen geometry set and its projection limits are
documented in `generated-footprints.md` here and
`qfn/docs/esp32-generated-footprints.md` in the dependency. The generated
geometry is derived from Espressif's pinned KiCad 3.2.1 library and direct
manufacturer PADS evidence; the corresponding CC-BY-SA-4.0 notice and KiCad
library exception are shipped as `LICENSE.kicad.md`.

`ESP32-S3R8` uses the same QFN56 land as the base `ESP32-S3`, but its 8 MB
Octal SPI PSRAM is internal to the package. `SPICS1` and `GPIO33` through
`GPIO37` are therefore modeled as required reserved terminals rather than
optional board GPIOs. `VDD_SPI` is fixed at 3.3 V for this variant. These
variant constraints follow the comparison and flash/PSRAM pin-mapping tables
in the stored v2.2 datasheet.
