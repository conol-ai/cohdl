# LoRa source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sx1262.pdf` | `SX1262IMLTRT` | Semtech Rev. 1.2, June 2019 | <https://www.semtech.com> | `644b55f38e97309161ea0c952a3a093020d94268ca8276eca53e5aa933ae7068` |
| `sx1280.pdf` | `SX1280IMLTRT` | Semtech Rev 3.2, March 2020 | <https://www.semtech.com> | `ff541551f0d9ac7cb84c96d0ac0e364c68689fdf4ba3d38b0ad5562b45e1fa5c` |
| `hpd16a-tsx1262.pdf` | `HPD16A` logical interface only; no part binding | HPDTek T-SX1262 / HPD16A Series product specification | <https://www.lilygo.cn> | `8419b878b914d50e9f7ba5c90c34c093064417e27a914cd16fa1ff15af303de4` |
| `hpb16b3-provenance.txt` | `HPB16B3` logical interface only; no part binding | LilyGO T-Display-SF32 V1.0 schematic, p2 U3 | [official repository, commit `08961099`](https://github.com/Xinyuan-LilyGO/T-Display-SF32/blob/08961099a702b2044b9f3541fddb16d10281d8d2/hardware/T-Display%20SF32%20V1.0.PDF) | `b4eb5aa92a035be503978f543535e714c079675efeb51148735cb0474363840c` |

SHA-256 checksums:

```text
644b55f38e97309161ea0c952a3a093020d94268ca8276eca53e5aa933ae7068  sx1262.pdf
ff541551f0d9ac7cb84c96d0ac0e364c68689fdf4ba3d38b0ad5562b45e1fa5c  sx1280.pdf
8419b878b914d50e9f7ba5c90c34c093064417e27a914cd16fa1ff15af303de4  hpd16a-tsx1262.pdf
b4eb5aa92a035be503978f543535e714c079675efeb51148735cb0474363840c  hpb16b3-provenance.txt
```

## Module qualification blockers

The HPDTek product sheet explicitly labels its mechanical drawing and block
diagram `HPD16A Series`, and page 3 establishes the 16-pin assignment. Page 4
also gives a 16.0 +/-0.1 mm square body, eight castellations per side on 2.0
+/-0.1 mm pitch, and component-side castellation dimensions. It does not give
a recommended PCB land pattern or a frequency-specific orderable identity;
the prior footprint was an undocumented land-pattern inference and remains
removed.

HPB16B3 is a different 12-pin module. Its exact pin order is established by
the official LilyGO board schematic cited above, but that repository publishes
neither the board PCB nor a standalone module drawing. The source does not
establish package geometry, a land pattern, supply limits, or the module's
internal radio. HPD16A and HPB16B3 must not share a part or footprint.
