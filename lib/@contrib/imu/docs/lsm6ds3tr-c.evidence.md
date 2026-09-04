# LSM6DS3TR-C evidence record

- Library coverage: `LSM6DS3TR_C` and `IMU_LSM6DS3TR_C`.
- Manufacturer document: STMicroelectronics `DocID030071 Rev 3`, May 2017.
- Source: <https://www.st.com/resource/en/datasheet/lsm6ds3tr-c.pdf>.
- Original PDF SHA-256: `251737db70e3b6cdc217a5681dd4634a05faffd5288aff6a576078a6464a1b2b`.
- Facts used: Table 2 on page 20 for the full 14-pin map; Figure 17 on page 111 for the 3.0 x 2.5 mm LGA body, 0.50 mm pitch and 0.25 x 0.475 mm package terminals.
- Qualification boundary: ST refers PCB-land guidance to a separate MEMS soldering document. The library land is an independent nominal-density derivation cross-checked against KiCad's ST-specific LGA-14 generator output and requires assembly-process qualification.

The third-party PDF is not redistributed by this repository. Download it from the source above and verify the recorded SHA-256 before auditing the library model.
