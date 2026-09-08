# Motor-driver source manifest

Audited on 2026-09-08. The PDF is stored byte-for-byte as downloaded; it
was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `drv8231a.pdf` | `DRV8231A` / `MOTOR_DRV8231A` / `FP_TI_DRV8231A_DDA` | Texas Instruments DRV8231A, SLVSFZ8A (November 2021, revised January 2026), DDA 8-pin PowerPAD HSOP | <https://www.ti.com/lit/ds/symlink/drv8231a.pdf> | `96545c004b2fa920eb71e99287d27fd1803626814c448f291ef500dbc24749c9` |

SHA-256 checksums:

```text
96545c004b2fa920eb71e99287d27fd1803626814c448f291ef500dbc24749c9  drv8231a.pdf
```

## Retrieval notes

Downloaded from TI's canonical symlink URL on 2026-09-08. The pinout in
`src/drv8231a.cohdl` transcribes Table 5-1 (1=IPROPI, 2=IN2, 3=IN1,
4=VREF, 5=VM, 6=OUT1, 7=GND, 8=OUT2, thermal pad to ground). The
`FP_TI_DRV8231A_DDA` land transcribes TI drawing 4214849/B (the
datasheet's own DDA0008B land pattern example): eight 1.55 x 0.6 mm pads
on a 1.27 mm pitch at a 5.4 mm outer extent, and a 2.71 x 3.40 mm
exposed-pad land with 100% printed paste (the 0.125 mm stencil row);
thermal vias are optional per the drawing's note 10 and left to boards.
A_IPROPI is 1500 uA/A (Section 6.5).
