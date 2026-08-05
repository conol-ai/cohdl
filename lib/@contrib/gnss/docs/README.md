# GNSS source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `l76k.pdf` | `L76K`, `GNSS_L76K`, and the manufacturer-recommended 18-LCC land | Quectel L76K hardware design manual v1.1, 2021-08-23 | <https://www.quectel.com> | `58909bfbdac0cab507482b07a170b95f9327ac12dc9b3d5fe6ca56e5c9771a5d` |
| `mia-m10q.pdf` | `MIA_M10Q` pin map and exact `MIA-M10Q-00B` ordering code | u-blox UBX-22015849 R08 | <https://www.u-blox.com> | `bd8e04d2520251d70565a5187369db84b4e979fc14cf0a0b47c1adfdc2131a43` |

SHA-256 checksums:

```text
58909bfbdac0cab507482b07a170b95f9327ac12dc9b3d5fe6ca56e5c9771a5d  l76k.pdf
bd8e04d2520251d70565a5187369db84b4e979fc14cf0a0b47c1adfdc2131a43  mia-m10q.pdf
```

The MIA-M10Q copper land is from u-blox Integration Manual UBX-21028173 R05,
section 4.5.1 and Figure 35, official source
<https://content.u-blox.com/sites/default/files/documents/MIA-M10Q_IntegrationManual_UBX-21028173.pdf>.
The copy audited on 2026-08-05 has SHA-256
`679a1d0ce1859922dfbee5f75b5565be82149e7294e862aafb0a63ddff5597f0`.
It specifies 0.27 mm copper lands, 0.37 mm non-solder-mask-defined openings,
paste equal to copper, and a 100 um stencil.
