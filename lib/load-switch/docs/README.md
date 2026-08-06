# Load switch source manifest

Audited on 2026-08-05. The PDF is stored byte-for-byte as downloaded; none
was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm2554.pdf` | `LOADSW_SGM2554` | SGMICRO SGM2554 power distribution switch, April 2022, REV. A.4, 13 pages | <https://www.sg-micro.com> (retrieved via LCSC C1156937) | `58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87` |
| `sgm2554.md` | summary (pinout / usage notes) | — | — | — |

SHA-256 checksums:

```text
58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87  sgm2554.pdf
```

Note: datasheet states supply input 2.2-5.5V; `sgm2554.md` previously said
1.0-5.5V. The cohdl part was not changed — verify the EN/input threshold
against the datasheet before tape-out.
