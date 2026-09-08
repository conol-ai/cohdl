# CAN source manifest

Audited on 2026-09-08. The PDF is stored byte-for-byte as downloaded; it
was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sn65hvd230.pdf` | `CAN_SN65HVD230` / `XCVR_SN65HVD230` | Texas Instruments SN65HVD230/231/232, SLOS346O (March 2001, revised April 2018), SOIC-8 "D" | <https://www.ti.com/lit/ds/symlink/sn65hvd230.pdf> | `e98fc0c59168c035d7958111924dae7e21e46ab45bdfd6400aef048299d6d716` |

SHA-256 checksums:

```text
e98fc0c59168c035d7958111924dae7e21e46ab45bdfd6400aef048299d6d716  sn65hvd230.pdf
```

## Retrieval notes

Downloaded from TI's canonical symlink URL on 2026-09-08. The pinout in
`src/sn65hvd230.cohdl` transcribes the Section 7 pin table (1=D, 2=GND,
3=VCC, 4=R, 5=Vref, 6=CANL, 7=CANH, 8=RS); the D package is the JEDEC
MS-012 narrow SOIC-8, bound to the dependency-owned
`soic::KICAD_SOIC_8_3_9X4_9MM_P1_27MM` land.
