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
