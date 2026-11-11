# Level shifter source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `rs0104.pdf` | `RS0104YTQF14`, `RS0104YTQE12`, `RS0104YUTQH12`, `RS0104YQ` | Run-IC REV A.5 | <https://www.run-ic.com> | `fffb1350d28a1b1d53c6047678955fce034f0ed11858a046a3adc8409440ff76` |

All four package-specific pin maps have public part bindings. `RS0104YQ` in
TSSOP-14 uses the shared qualified SOIC/SOP land; the three QFN order codes use
package-specific, independently derived nominal-density lands. The datasheet
supplies full component tolerances but not manufacturer PCB-land
recommendations, so each derivation and its toe/heel/side goals are recorded
beside the footprint rather than hidden behind a generic QFN name.

SHA-256 checksums:

```text
fffb1350d28a1b1d53c6047678955fce034f0ed11858a046a3adc8409440ff76  rs0104.pdf
```

## RS0104 QFN14 land correction in 0.2.0

Re-audited page 25 (QFN3.5x3.5-14L) on 2026-11-07. Earlier versions put
pads 1/14 and 7/8 of `QFN14N50P350X350_1EP200X150` (`LS_RS0104_QFN14`,
RS0104YTQF14) 0.5 mm apart at x = -/+0.25 mm. The bottom view dimensions
E1 = 1.500 TYP between the centres of leads 1 and 14 (and of 7 and 8), so
those lands now sit at x = -/+0.75 mm on the top and bottom rows; on the
old lands VCCA, VCCB, GND, and OE would not have met their leads. The other
ten lands, the pad sizes, the courtyard, and the footprint name are
unchanged. A board that placed this part sees four pads move after
`cohdl update`, with `check` and `build` still passing; re-route those
four nets.

Page 25 draws the thermal pad but does not dimension it: D1 (2.000) and E1
(1.500) are lead spans. The 2.0 x 1.5 mm exposed-pad land is kept as a
library choice, not a manufacturer value. It clears the corrected lands by
0.375 mm and the side lands by 0.125 mm. The device pin map, the part
identity, and the two QFN12 lands are unchanged.

## LCSC binding for RS0104YTQF14 (0.2.1)

`LS_RS0104_QFN14` binds its primary to LCSC C5372421 with a quarter-turn map: `supplier lcsc "C5372421" rotate 270`. The part number is in `tools/sourcing_data/lcsc_map.json` (alias rule, 2026-11-07). The other three RS0104 parts stay unbound: no preview has read them.

- **Why a map.** On the 2026-11-08 probe preview the assembler's pin 1 for C5372421 sat a quarter turn counter-clockwise from our pad 1 (86.1–86.2° on top, 273.5–273.8° from below) on all four copies. The land was re-audited against the datasheet in 0.2.0 and stays where it is; the binding carries the turn.
- **Evidence.** The frame row `rv2-qfn14n50p350x3501ep200x150-c5372421` cites the second re-verify board (`tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/`), which leaves the reference designators off the silkscreen so every edge of the model shows. With `rotate 270` applied, pin 1 read 357.2° and 356.7° on top (0° and 37°) and 3.2° and 3.1° from below, 0.27–0.29 mm from pad 1 (pad 2: 1.08–1.10 mm). In the top view 13 of the 14 model leads sit on our pads (5 left, 5 right, 2 bottom, 1 top), and the 14th lies under the pin 1 mark, on pad 1. The body is 3.42–3.47 mm square, with its edges within 0.6° of the land and its centre within 0.03 mm. The first re-verify board (`2026-11-08-reverify`) read 357.5°, 356.7°, 2.7° and 2.6°.
- **What changes in output.** No land changed from 0.2.0. A plain `build` writes the same BOM and CPL; a design that places `LS_RS0104_QFN14` gains the part's `LCSC` field in its netlist. `--supplier lcsc` writes the turn into the LCSC CPL.
