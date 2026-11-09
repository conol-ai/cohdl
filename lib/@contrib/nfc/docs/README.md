# NFC reader source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `st25r3916.pdf` | `ST25R3916-AQET` | STMicroelectronics DS12484 Rev 8, May 2023 | <https://www.st.com> | `1170f4d74ff501917262ae42bf81c60c38c11d121e2ea2af6b1e845433befef1` |

SHA-256 checksums:

```text
1170f4d74ff501917262ae42bf81c60c38c11d121e2ea2af6b1e845433befef1  st25r3916.pdf
```

## ST25R3916 land and supply pins (nfc 0.2.0, 2026-11-07)

0.2.0 corrects `QFN32N50P500X500_1EP345X345` to DS12484 Rev 8 Figure 39
(p. 153), rendered at 450 and 600 dpi and measured along the land rows.
The figure dimensions the lands as 0.60 mm long (pad 25) and 0.30 mm wide
(pad 8) at 0.50 mm pitch, with a 5.30 mm outer span and a 3.45 mm exposed
land. Its three "3.80" dimensions are the length of one 8-land row (outer
edge of pad 32 to outer edge of pad 25, pad 24 to pad 17, pad 9 to pad 16),
and its "0.75" is the step from the end of a row (pad 17) to the outer edge
(pad 16), that is (5.30 − 3.80) / 2; it is not a land length.

| | 0.1.0 | 0.2.0 |
|---|---|---|
| Land | 0.75 × 0.30 mm | 0.60 × 0.30 mm |
| Land centre | ±2.275 mm | ±2.35 mm |
| Outer edge | 2.65 mm | 2.65 mm |
| Inner edge | 1.90 mm | 2.05 mm |
| Gap to exposed land | 0.175 mm | 0.325 mm |

Pitch, land width, exposed land, pad numbers, courtyard and silkscreen are
unchanged. Only the inner 0.15 mm of each signal land is removed.

The same version makes seven `ST25R3916` pins `required`, where the data
sheet requires a connection:

- `VDD_TX`: VDD and VDD_TX must be connected to the same supply (p. 39),
  within 0.2 V (p. 137).
- `VDD_DR`: the antenna-driver supply input (p. 21), tied to `VDD_RF` in
  the minimum configuration (Figure 1, p. 13), or with `VDD_RF` to
  `VDD_TX` above 350 mArms (p. 41).
- `VDD_D`, `VDD_A`, `VDD_RF`, `VDD_AM`: regulator outputs, each with a
  2.2 µF ∥ 10 nF blocking capacitor (p. 40; Figure 1, p. 13); VDD_AM
  "requires" 2.2 µF + 1 nF (p. 41).
- `I2C_EN`: selects the host interface, GND for SPI or VDD_D for I²C
  (p. 48, p. 49, p. 55; Figure 1, p. 13). It is a 3 V-domain pin, 0–3.6 V
  (p. 137).

This is a breaking change: a design that left one of these pins
unconnected now fails `check`, and the land copper differs. No package or
example in this repository depends on `@contrib/nfc`.
