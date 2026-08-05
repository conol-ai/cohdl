# Audio amplifier source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `max98357a.pdf` | `MAX98357A` pins, `MAX98357AETE+` order code, and T1633+4 package outline | Analog Devices rev 16 | <https://www.analog.com> | `dea228ad619901d34c2f5f106243bdb9b9e95e38989d4d66fd6c6c489bfa6f1c` |
| `ns4150b.pdf` | Legacy `NS4150` pin map and MSOP-8 dimensions, corroborating the current exact `NS4150B` source below | Nsiway V1.0 | <https://www.nsiway.com.cn> | `61812f56feefc64f799d60b1807b5411ffe2c338687163f3b81ba1ccf6ced2f1` |

SHA-256 checksums:

```text
dea228ad619901d34c2f5f106243bdb9b9e95e38989d4d66fd6c6c489bfa6f1c  max98357a.pdf
61812f56feefc64f799d60b1807b5411ffe2c338687163f3b81ba1ccf6ced2f1  ns4150b.pdf
```

Additional footprint and exact-product evidence audited on 2026-08-05:

- Analog Devices land pattern 90-0031 rev C, official source
  <https://mds.analog.com/api/public/content/90-0031.pdf>, SHA-256
  `5661318725986f79e664adb5fd24d80c2b444c1d48d3aa4c1b0d51ebfa53f4c8`.
- Nsiway's official [`NS4150B` MSOP-8 product page](https://nsiway.com.cn/list_38/82.html)
  hosts the December 2022 V1.4 data-sheet pages. Relevant page-image SHA-256
  values are recorded in `footprint-derivations.md`; page 2 confirms the exact
  eight-pin assignment and page 9 confirms the MSOP-8 dimensions.
- `footprint-derivations.md` records the auditable IPC-7351 nominal-density
  derivation for `SOP8P65X490X110N`.
