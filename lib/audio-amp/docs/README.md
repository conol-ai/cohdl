# Audio amplifier source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `ns4168.pdf` | `AMP_NS4168` | Nsiway NS4168 V1.2, Apr 2023 | <https://www.lcsc.com/datasheet/C910588.pdf> (LCSC C910588; Nsiway-branded mirror) | `2f8e302b0f69ccad330f8e467b173e6a1eb2ae04f3eeb6640bf3c7afd3c808d4` |

> **Resolved 2026-08-05.** The `NS4168` device was previously modeled with the
> NS4150-family analog pinout in a QFN 2x2mm package, contradicting the
> official V1.2 datasheet. It is now re-mapped to the datasheet's I2S ESOP8
> pinout (§5 pin table): CTRL / LRCLK / BCLK / SDATA / VON / VDD / GND / VOP,
> with footprint `FP_NS4168_ESOP8` (SOP-8, 1.27mm pitch, 5.0x4.0mm body,
> 6.0mm lead-tip span, **no exposed pad** per the §11.1 package drawing).

SHA-256 checksums:

```text
2f8e302b0f69ccad330f8e467b173e6a1eb2ae04f3eeb6640bf3c7afd3c808d4  ns4168.pdf
```
