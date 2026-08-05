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
