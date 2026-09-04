# LDO source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `xc6206.pdf` | `XC6206P182MR-G` public SOT-23 part with documented independent land derivation | Torex ETR0305_004b | <https://www.torexsemi.com> | `8fda1e40ea3e7c17178353aceba9c9a9794e56a1960d02b105c576429ce418ed` |
| `rt9080.pdf` | `RT9080-33GJ5`; `RT9080-33GQZ` | Richtek DS9080-09, December 2024 | <https://www.richtek.com> | `52d09967ef32672003df93d1382bc249715365406f59d110d9ebad6d618017d4` |
| `eta5060.pdf` | `ETA5060V330S8F` | ETA Solutions ETA5060-V2.6 | <https://datasheet.lcsc.com/datasheet/pdf/f06e40bfaf9d0ccdda290ccd8eaa059b.pdf?productCode=C7465551> | `dc7f99f539e1fbe21ed445c670a2d2f47f5731229220f33dd071d1b6994ba359` |
| `me6211c15.pdf` | `ME6211C15M5G-N` | Microne ME6211 Ver07 | <https://datasheet.lcsc.com/datasheet/pdf/21ba39e3bdd741eeb3319f5d3b04354d.pdf?productCode=C53100> | `e5c37419befbce53fa0c3dc630b1de6fa9812fa41f1c580667c53cd9efa7f900` |
| `me6211c18.pdf` | `ME6211C18M5G-N` | Microne ME6211 V20 | <https://datasheet.lcsc.com/datasheet/pdf/2079c21a565bb18f68c1d4daad5e823a.pdf?productCode=C236671> | `23084e9d0d0fd762c7419509ee656856be36f08ffc39f184e22a2acf7cfae5e6` |
| `me6211c28.pdf` | `ME6211C28M5G-N` | Microne ME6211 Ver07 | <https://datasheet.lcsc.com/datasheet/pdf/9d2921049f60470683cf40648b87ef79.pdf?productCode=C53099> | `1d8126fbc091b11411756780f845a8773cd4be657d00d080bfbced20550e6965` |

SHA-256 checksums:

```text
8fda1e40ea3e7c17178353aceba9c9a9794e56a1960d02b105c576429ce418ed  xc6206.pdf
52d09967ef32672003df93d1382bc249715365406f59d110d9ebad6d618017d4  rt9080.pdf
dc7f99f539e1fbe21ed445c670a2d2f47f5731229220f33dd071d1b6994ba359  eta5060.pdf
e5c37419befbce53fa0c3dc630b1de6fa9812fa41f1c580667c53cd9efa7f900  me6211c15.pdf
23084e9d0d0fd762c7419509ee656856be36f08ffc39f184e22a2acf7cfae5e6  me6211c18.pdf
1d8126fbc091b11411756780f845a8773cd4be657d00d080bfbced20550e6965  me6211c28.pdf
```

## Retrieval notes

The ETA Solutions manufacturer endpoint (eta-semi.com) was unreachable during
retrieval. The manufacturer-authored ETA5060-V2.6 document was retrieved from
authorized distributor LCSC (part C7465551, byte-identical document header
and revision history).

The ME6211 documents were retrieved from the exact LCSC product listings
C53100, C236671 and C53099. Microne's official product page identifies the
same family; the V20 document supplies the SOT-23-5 pin assignment and
package outline. No recommended PCB land is published, so the library records
a nominal KiCad SOT-23-5 derivation that requires assembly validation.
