# Load switch source manifest

Audited on 2026-08-05; `mt9700.evidence.md` added 2026-09-04. Entries ending
in `.evidence.md` are repository-authored evidence records; their SHA-256
values identify upstream manufacturer PDFs that are not redistributed here.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm2554.pdf` | `LOADSW_SGM2554` | SGMICRO SGM2554 power distribution switch, April 2022, REV. A.4, 13 pages | <https://www.sg-micro.com> (retrieved via LCSC C1156937) | `58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87` |
| `sgm2554.md` | summary (pinout / usage notes) | — | — | — |
| `tps22918.evidence.md` | `LOADSW_TPS22918DBVR` | Texas Instruments TPS22918, SLVSD76C; DBV package drawing 4214840/G | <https://www.ti.com/lit/ds/symlink/tps22918.pdf> | `1b28eb58003d28b18bf919a15734056f296a499bebee2bd493a1e4358c4dfb1c` |
| `mt9700.evidence.md` | `LOADSW_MT9700N` | Xi'an Aerosemi Technology MT9700 Rev2.1, 8 pages | <https://www.aerosemi.com> (retrieved via LCSC C42441843, productModel MT9700-N) | `b51be4566fc3e85022bbafd41874b29949e2bc4eada688ae912d361a19149ec8` |

SHA-256 checksums:

```text
58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87  sgm2554.pdf
1b28eb58003d28b18bf919a15734056f296a499bebee2bd493a1e4358c4dfb1c  tps22918.pdf (upstream, not vendored)
b51be4566fc3e85022bbafd41874b29949e2bc4eada688ae912d361a19149ec8  mt9700.pdf (upstream, not vendored)
```

## MT9700-N retrieval and land notes

- The accepted orderable model is `MT9700-N`; LCSC
  C42441843 lists `productModel` `MT9700-N`, brand `XI'AN Aerosemi Tech`,
  SOT-23-5, and serves this Rev2.1 document on that listing. The sibling
  SKU C89855 (`MT9700`, same SOT23-5) serves the older Rev1.0 document;
  only the document attached to the exact `MT9700-N` listing is used.
- The Rev2.1 order table lists the family part number `MT9700` with top
  mark `D00HAW`; pin configuration (VOUT 1, GND 2, SET 3, EN 4, VIN 5) and
  the SOT23-5 outline are on p2 and p7.
- Aerosemi publishes no recommended PCB land pattern (Figure 3 is
  topological only), so the part uses the nominal generic KiCad SOT-23-5
  land (JEDEC MO-178 Var AA) recorded beside `FP_Aerosemi_SOT23_5`. That
  secondary generic reference is not Aerosemi authority and keeps the
  assembly-validation requirement (the accepted `W25Q128JVPIQ` precedent).

Note: datasheet states supply input 2.2-5.5V; `sgm2554.md` previously said
1.0-5.5V. The cohdl part was not changed — verify the EN/input threshold
against the datasheet before tape-out.
