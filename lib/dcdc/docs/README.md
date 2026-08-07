# DCDC source manifest

Audited on 2026-08-05. The PDF is stored byte-for-byte as downloaded; it
was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `jw5033s.pdf` | `JW5033S` / `DCDC_JW5033S` | Joulwatt JW5033S, SOT-23-6, 3.8V/2A sync buck — datasheet from Joulwatt | Joulwatt (via authorized distributor SZLCSC, part C324577) | `a391630b450176f2280eb662e152ba28ecd1ee9ed39da7fa5b0270b467b3e3f0` |
| `sgm6029.pdf` | `SGM6029` / `DCDC_SGM6029C` | SGMICRO SGM6029, WLCSP-0.74x1.09-6B, Rev. A.1 (November 2022) | <https://www.sg-micro.com/product/SGM6029> | `79d7e25abc3acfd18ab9c394e082663e07dc169151b4bfe0314c13510ced63e0` |

SHA-256 checksums:

```text
a391630b450176f2280eb662e152ba28ecd1ee9ed39da7fa5b0270b467b3e3f0  jw5033s.pdf
79d7e25abc3acfd18ab9c394e082663e07dc169151b4bfe0314c13510ced63e0  sgm6029.pdf
```

## Retrieval notes

Joulwatt's site does not serve the datasheet at a stable public URL. The
manufacturer-authored document (R0.88, 2019-03-19) was retrieved from the
authorized distributor SZLCSC (part C324577).

The SGM6029 document was downloaded from the official SGMICRO product-page
asset on 2026-08-07:
<https://www.sg-micro.com/rect/assets/3323f5dc-718f-4b81-b820-398ddb702989/SGM6029.pdf>.
The access token appended by the product page is intentionally omitted because
it is transient; the asset UUID and checksum identify the retrieved document.
Its package page specifies the 2x3, 0.35 mm-pitch WLCSP and a 0.217 mm nominal
solder-ball diameter. The recommended PCB land is 0.17 to 0.19 mm; the public
footprint uses a 0.19 mm drawn land. The datasheet top view was preserved
without mirroring: row A is above row C and A1 is the upper-left land.
