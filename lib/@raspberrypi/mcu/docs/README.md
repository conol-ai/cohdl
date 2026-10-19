# Official documentation

Manufacturer documents are retained unchanged. The package's MIT license
applies to its original code and application note, not to Raspberry Pi PDFs.
The PDFs retain their own copyright and CC BY-ND 4.0 notices.

## `rp2350-datasheet.pdf`

- Retrieved: 2026-07-30.
- Covers: `RP2350A` / `RP2350A` in the 7 x 7 mm QFN-60 package.
- Document identity: build date 2025-07-29, build version
  `d126e9e-clean`.
- Official URL:
  <https://datasheets.raspberrypi.com/rp2350/rp2350-datasheet.pdf>
- SHA-256:
  `2877d0f270fb6d6a57943bee58aaad536aa027bea1e5b1c4ce2541a3230d4be8`
- Footprint authority: Figure 144, "Recommended PCB Footprint for the
  RP2350 QFN-60 package."

The pin map follows Figure 148. Table 1432's IOVDD row incorrectly substitutes
pin 54 for pin 1 relative to that figure; pin 54 is QSPI_IOVDD. No pin binding
was changed by the application-note update. The package
drawing and recommended land pattern were visually inspected on PDF pages
1328-1329.

## `rp2350-hardware-design.pdf`

- Retrieved: 2026-10-16 from Raspberry Pi's official document service.
- Title: Hardware design with RP2350.
- Identity: Release 3, build 20/08/2026, `06a7f75e58a1`; release-history
  date 24 August 2026. The URL retains `DS-2` although the PDF is Release 3.
- Official URL:
  <https://pip-assets.raspberrypi.com/categories/1214-rp2350/documents/RP-008280-DS-2-hardware-design-with-rp2350.pdf>
- SHA-256:
  `cef88bc7d87e67b4262ee15f2b94b03566667143f4e8daf6931828bef6408059`
- Size: 8,543,080 bytes, 25 pages.
- Copyright Raspberry Pi 2022-2026; CC BY-ND 4.0, retained unmodified.

## `rp2350-application.md`

Original summary for `raspberrypi_mcu::RP2350A` and
`raspberrypi_mcu::RP2350A_QFN60`, attached to both API symbols so a registry
document lookup can return the support-circuit information directly.

Reviewed against the included manufacturer sources on 2026-10-16. Regulator,
decoupling, flash, crystal and USB diagrams were visually inspected, along
with the QFN-60 pinout, supply/current tables and GPIO erratum. References use
1-based PDF page numbers. The note distinguishes manufacturer requirements,
reference examples, typical measurements and application-level obligations.
It does not certify a PCB, substitute part, firmware implementation or
passive USB PD analyzer design.
