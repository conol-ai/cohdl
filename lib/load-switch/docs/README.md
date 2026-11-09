# Load switch source manifest

Audited on 2026-08-05; `mt9700.evidence.md` added 2026-09-04. Entries ending
in `.evidence.md` are repository-authored evidence records; their SHA-256
values identify upstream manufacturer PDFs that are not redistributed here.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm2554.pdf` | `LOADSW_SGM2554` | SGMICRO SGM2554 power distribution switch, April 2022, REV. A.4, 13 pages | <https://www.sg-micro.com> (retrieved via LCSC C1156937) | `58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87` |
| `sgm2554.md` | summary of `LOADSW_SGM2554` (SGM2554A pinout, land, usage notes) | — | — | — |
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

TPS22918 application boundaries updated 2026-10-17 against the same hashed
manufacturer PDF: output discharge does not establish reverse-current
blocking; see the attached evidence record for the specific sections.

## SGM2554 pin map and land correction in 0.2.0

Re-audited `sgm2554.pdf` on 2026-11-07 against the rendered p. 3 pin
configuration and the p. 11 recommended land pattern. Version 0.2.0 is a
breaking change to `SGM2554` / `LOADSW_SGM2554`:

- **Pin map.** Earlier versions declared 1=IN, 2=GND, 3=EN, 4=NC, 5=OUT.
  The datasheet's SGM2554A is 1=VOUT, 2=GND, 3=VIN, 4=EN, 5=VOUT (p. 3;
  the p. 1 typical application and p. 5 test circuit agree). A board wired
  to the old names put the supply on an output pad, drove the switch input
  from the enable net, and left the real EN unconnected. The device now
  declares `VOUT: 1, 5 [power_out]`, `GND: 2`, `VIN: 3 [power_in]` and
  `EN: 4 [input]` (logic high enables, p. 3). The pin names `IN`, `OUT` and
  `NC` are gone, so a design that wires them fails to check after
  `cohdl update` and must be rewired to `VIN`, `VOUT` and `EN`.
- **Part identity.** The primary MPN is now the A model's ordering code
  `SGM2554AYN5G/TR` (p. 2), not the family name `SGM2554`. Only the A model
  has EN; on `SGM2554BYN5G/TR` pin 4 is a second VIN (p. 3).
- **Input range.** The comments now give 2.2-5.5V (p. 1, p. 4), not
  1.0-5.5V.
- **Land.** `FP_SGM2554_SOT235` now uses the recommended land of p. 11:
  0.69 x 0.99mm pads (was 0.60 x 1.00mm) on 2.59mm row centres (was
  2.60mm), 0.95mm pitch unchanged. The courtyard grows from 4.1 x 3.0mm to
  4.1 x 3.6mm so it contains the 3.050mm maximum body length D (p. 11), and
  the reference designator moves from y = -2.2mm to -2.4mm. The footprint
  name, pad numbers and pad order are unchanged. The old comment's claim
  that the land matched `SOT5P95X290X145N` in `lib/@contrib/ldo` is removed:
  that package's RT9080 footprint is `SOT5P95X290X160N`, with 1.0 x 0.7mm
  pads.

The `sf32-miniboard` example had worked around the wrong map with a local
`SGM2554A_CORRECT` device on this land, under the same ordering code. It now
pins load-switch 0.2.0 and uses `LOADSW_SGM2554` directly; its connectivity
is unchanged.
