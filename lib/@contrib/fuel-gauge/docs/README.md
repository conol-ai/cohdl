# Fuel gauge source manifest

Retrieved and audited on 2026-09-03. The `.evidence.md` file is a
repository-authored evidence record; its SHA-256 identifies the upstream
manufacturer PDF, which is not redistributed here.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `cw2015chbd.evidence.md` | `CW2015CHBD` device and `FGAUGE_CW2015CHBD` public part | CellWise CW2015CHBD-DS V1.3 (15 pages) | <https://datasheet.lcsc.com/datasheet/pdf/79d345fcfd2ee12112c42f3bd7f90599.pdf?productCode=C881838> | `d29d16e303586e8a56105a30606689762dcf63af822c7c8b92af39087ee85f48` |

SHA-256 checksums:

```text
d29d16e303586e8a56105a30606689762dcf63af822c7c8b92af39087ee85f48  cw2015chbd.pdf (upstream, not vendored)
```

## Retrieval notes

The CellWise official brief (en.cellwise-semi.com, 2 pages, V1.3 header
`CW2015CHBD-DS`) carries no pin map or package drawing. The full 15-page
manufacturer-authored datasheet with the p3 pin configuration and the p14
TDFN 2x3-8L outline was retrieved from the authorized distributor LCSC
(part C881838); its identity header and revision table match the official
brief.
