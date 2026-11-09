# Charger source manifest

Retrieved and audited on 2026-08-04. Entries ending in `.evidence.md` are
repository-authored evidence records; their SHA-256 values identify the
upstream manufacturer PDF, which is not redistributed here.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm41562a-sgm41562b.pdf` | `SGM41562BXG/TR` | SG Micro SGM41562A/B datasheet REV. A.1, January 2022 | <https://www.sg-micro.com> | `e3fcb1f1662400261dfe60b6376fcc269c49ca0839f4d64a0cebb596835008fe` |
| `bq25185.evidence.md` | `BQ25185DLHR` | Texas Instruments BQ25185 datasheet SLUSF65B, revised August 2026 | <https://www.ti.com/lit/ds/symlink/bq25185.pdf> | `c73ed7d63e6532bb05e26df30edd37c6ac2de310c1222a84a77625e15e133684` |

SHA-256 checksums:

```text
e3fcb1f1662400261dfe60b6376fcc269c49ca0839f4d64a0cebb596835008fe  sgm41562a-sgm41562b.pdf
c73ed7d63e6532bb05e26df30edd37c6ac2de310c1222a84a77625e15e133684  bq25185.pdf (upstream, not vendored)
```

## Retrieval notes

The manufacturer endpoint blocked automated binary retrieval. The manufacturer-authored document was retrieved from authorized distributor LCSC (part C5153801).

## SGM41562B I2C pins required in 0.2.0

Re-audited pages 13, 17, 20, 22, 25, 27 and 31 on 2026-11-07. Earlier
versions marked `SDA` and `SCL` of `SGM41562B` `optional`, with a comment
that the charger runs on standalone defaults without a host. It does not.
After a power-on reset the device is in default mode with `CEB = 1`, charge
disabled, and "by default the battery will not be charged after a reset"
(p. 17; REG01 D[3] power-on value 1, p. 20). With input power and no I2C
write, the watchdog, set to 40 s when the input is detected, power-cycles
`SYS` and returns the device to default mode (p. 13, p. 22, Figure 9 on
p. 27). A host at I2C address 03H (p. 20, p. 25) must write `CEB = 0`.

0.2.0 marks `SDA` and `SCL` `required`, so a design that leaves either open
now fails `check`. Wire both to the host with a 10 kΩ pull-up each (p. 3).
The comment on `VDD` now also gives the 1 µF capacitor of the typical
application and its BOM (p. 1, p. 31) next to the 0.1 µF minimum (p. 3).
Pin names, ball numbers, the part identity and the land are unchanged.
