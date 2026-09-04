# IMU source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `bst-bhi260ap-ds000.pdf` | `BHI260AP` device and `IMU_BHI260AP` public part | Bosch Sensortec BST-BHI260AP-DS000-02 Rev. 1.1, 2021-04-15 | <https://www.bosch-sensortec.com/media/boschsensortec/downloads/datasheets/bst-bhi260ap-ds000.pdf> | `1db636a1dcdc6cf2c5da3bd4b324aa210043d6e340327f2ccaa068f39b97b5c9` |
| `mmc5603nj.pdf` | `MMC5603NJ` device and `MAG_MMC5603NJ` public part | MEMSIC MMC5603NJ Rev. B, 2022-01-17 | <https://www.memsic.com/Public/Uploads/uploadfile/files/20220119/MMC5603NJDatasheetRev.B.pdf> | `3b64ea625a9928363f847805b88406ffbee7b4b55f3ad7421cd73b5d9d816518` |
| `lsm6ds3tr-c.pdf` | `LSM6DS3TR_C` device and `IMU_LSM6DS3TR_C` public part | STMicroelectronics DocID030071 Rev 3, May 2017 | <https://www.st.com/resource/en/datasheet/lsm6ds3tr-c.pdf> | `251737db70e3b6cdc217a5681dd4634a05faffd5288aff6a576078a6464a1b2b` |

SHA-256 checksums:

```text
1db636a1dcdc6cf2c5da3bd4b324aa210043d6e340327f2ccaa068f39b97b5c9  bst-bhi260ap-ds000.pdf
3b64ea625a9928363f847805b88406ffbee7b4b55f3ad7421cd73b5d9d816518  mmc5603nj.pdf
251737db70e3b6cdc217a5681dd4634a05faffd5288aff6a576078a6464a1b2b  lsm6ds3tr-c.pdf
```

## LSM6DS3TR-C geometry qualification

Table 2 defines the complete 14-pin electrical map. Figure 17 defines the
3.0 x 2.5mm package, 0.5mm pitch, and 0.25 x 0.475mm package terminals.
The data sheet does not contain a PCB land pattern; it points to the ST MEMS
soldering recommendations. `LGA14P50_300X250X86N` is therefore an independent
nominal-density derivation, cross-checked against KiCad 9's
`LGA-14_3x2.5mm_P0.5mm_LayoutBorder3x4y`, whose generator cites this exact ST
data sheet. Production use requires assembly-process qualification.
