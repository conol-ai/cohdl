# Board connector source manifest

Board-level connectors bound to real, in-stock MPNs with manufacturer
documents. The PDFs are stored byte-for-byte as downloaded; none was
regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `fh12-series.pdf` | `CONN_FPC24_0P5` = HRS `FH12-24S-0.5SH(55)` (LCSC C202112) | Hirose FH12 series, 0.5/1.0 mm FPC/FFC connectors | <https://www.hirose.com> (via LCSC C202112) | `9f056f58ff000739633cc3cb7697ba185967083dcf1429b7ab10783b3f55bac5` |
| `conmhf4-smd-g-t.pdf` | `CONN_IPEX_MHF4` = Linx/TE `CONMHF4-SMD-G-T` (LCSC C3173267) | Linx (TE Connectivity) CONMHF4-SMD-G-T, MHF4-class jack, 50 ohm, 6 GHz | <https://www.te.com/en/product-CONMHF4-SMD-G-T.html> | `63fedb5a916e68998c0ec4aafd5797065a19ffb404fb68fbbf44ad97910a7513` |
| `jst-xh.pdf` | `CONN_BAT_2P` = JST `S2B-XH-A(LF)(SN)` (LCSC C157931) | JST XH series, 2.5 mm pitch wire-to-board connector | <https://www.jst-mfg.com/product/pdf/eng/eXH.pdf> | `9426b136902f11900825077535e5c65032b7fbc31ffb59c5e9e1f463bb20fb90` |
| `yzp0048-20048-04025-03.pdf` | `CONN_MAG_4P` = Xinyangze `YZP0048-20048-04025-03` (LCSC C5126845) | Xinyangze product spec 2022-08-08: 4-pin magnetic pogo, 1 A, 3.1 mm working height | <https://www.lcsc.com/product-detail/C5126845.html> | `d2c26de7b1294bc46f5294b357ab1d7b9584fca8ca2aef185b156c550e0cdb34` |

`CONN_MIC_2P`, `CONN_DEBUG_4P`, `CONN_SPK_2P` remain generic (2-pin mic
module terminal, 4-pin 1.27 mm debug port, 2-pin speaker terminal) — no
manufacturer documents stored.

| Device | Part | Bound MPN | Type | Spec basis |
| --- | --- | --- | --- | --- |
| `FPC24_0P5` | `CONN_FPC24_0P5` | HRS FH12-24S-0.5SH(55) | 24-pin FPC socket, 0.5mm pitch, bottom contact | Hirose FH12 series datasheet (`fh12-series.pdf`) |
| `IPEX_MHF4` | `CONN_IPEX_MHF4` | Linx CONMHF4-SMD-G-T | 1-pin RF coax jack, MHF4 class | Linx/TE datasheet (`conmhf4-smd-g-t.pdf`) |
| `BAT_CONN_2P` | `CONN_BAT_2P` | JST S2B-XH-A(LF)(SN) | 2-pin 2.5mm pitch SMD wire-to-board (battery) | JST XH series datasheet (`jst-xh.pdf`) |
| `MAG_CONN_4P` | `CONN_MAG_4P` | Xinyangze YZP0048-20048-04025-03 | 4-pin magnetic pogo charge connector | Xinyangze spec (`yzp0048-20048-04025-03.pdf`) |

Footprints are generic land patterns sized for the stated pitches; verify
geometry against the bound manufacturer drawing before fabrication.

SHA-256 checksums:

```text
9f056f58ff000739633cc3cb7697ba185967083dcf1429b7ab10783b3f55bac5  fh12-series.pdf
63fedb5a916e68998c0ec4aafd5797065a19ffb404fb68fbbf44ad97910a7513  conmhf4-smd-g-t.pdf
9426b136902f11900825077535e5c65032b7fbc31ffb59c5e9e1f463bb20fb90  jst-xh.pdf
d2c26de7b1294bc46f5294b357ab1d7b9584fca8ca2aef185b156c550e0cdb34  yzp0048-20048-04025-03.pdf
```

## Generic (customer-selected) parts

- `CONN_MIC_2P`: MEMS mic is a board-mounted part (note §11 U6, 3.5×2.65mm
  class); the 2-pin interface pad is customer-selected per the chosen mic —
  no fixed MPN.
- `CONN_SPK_2P`: speaker is case-integrated (note §11: 1×2mm rear-wall port
  with waterproof membrane); the 2-pin terminal is per chosen speaker.
- `XTAL_*`/`IND_24N` (watch design): clock components are generic placeholders
  bound at build time; the design note §15.1 table specifies package/CL.
