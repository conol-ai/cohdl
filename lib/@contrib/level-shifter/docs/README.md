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
