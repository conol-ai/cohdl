# USB-to-UART source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `ch343.pdf` | `CH343P` logical device only; no part binding | WCH V1.6 | <https://www.wch-ic.com> | `9ba293770db3da78124238f3d5bc1cd9cb41d563f543c8d2cad8ae96d32a118d` |

SHA-256 checksums:

```text
9ba293770db3da78124238f3d5bc1cd9cb41d563f543c8d2cad8ae96d32a118d  ch343.pdf
```

## Package qualification blocker

The pin assignment was visually checked against the QFN column on datasheet
page 2. WCH's official [CH-series common package drawing, revision 2E](https://www.wch.cn/uploads/file/20200305/1583372123113134.pdf)
adds the QFN16-3x3 component geometry on page 44: 3.0 mm body, 0.5 mm nominal
pitch, 0.25 mm terminal width, 0.4 mm terminal length, 1.8 mm exposed pad, and
0.55 mm package height.

That document is a component drawing, not a recommended PCB land/paste
pattern. Its general note makes pin-center pitch nominal and permits up to
+/-0.5 mm error for other dimensions, which is not a complete tolerance set
for an IPC-7351/7352 derivation. The former 1.7 mm exposed-pad footprint is
also inconsistent with WCH's 1.8 mm component pad. `CH343P` therefore remains
a logical device only.
