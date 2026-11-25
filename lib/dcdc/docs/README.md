# DCDC source manifest

Audited on 2026-08-05. The PDF is stored byte-for-byte as downloaded; it
was not regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `jw5033s.pdf` | `JW5033S` / `DCDC_JW5033S` | Joulwatt JW5033S, SOT-23-6, 3.8V/2A sync buck — datasheet from Joulwatt | Joulwatt (via authorized distributor SZLCSC, part C324577) | `a391630b450176f2280eb662e152ba28ecd1ee9ed39da7fa5b0270b467b3e3f0` |
| `sgm6029.pdf` | `SGM6029` / `DCDC_SGM6029C` | SGMICRO SGM6029, WLCSP-0.74x1.09-6B, Rev. A.1 (November 2022) | <https://www.sg-micro.com/product/SGM6029> | `79d7e25abc3acfd18ab9c394e082663e07dc169151b4bfe0314c13510ced63e0` |

SHA-256 checksums:

```text
a391630b450176f2280eb662e152ba28ecd1ee9ed39da7fa5b0270b467b3e3f0  jw5033s.pdf
79d7e25abc3acfd18ab9c394e082663e07dc169151b4bfe0314c13510ced63e0  sgm6029.pdf
```

## Retrieval notes

Joulwatt's site does not serve the datasheet at a stable public URL. The
manufacturer-authored document (R0.88, 2019-03-19) was retrieved from the
authorized distributor SZLCSC (part C324577).

The SGM6029 document was downloaded from the official SGMICRO product-page
asset on 2026-08-07:
<https://www.sg-micro.com/rect/assets/3323f5dc-718f-4b81-b820-398ddb702989/SGM6029.pdf>.
The access token appended by the product page is intentionally omitted because
it is transient; the asset UUID and checksum identify the retrieved document.
Its package page specifies the 2x3, 0.35 mm-pitch WLCSP and a 0.217 mm nominal
solder-ball diameter. The recommended PCB land is 0.17 to 0.19 mm; the public
footprint uses a 0.19 mm drawn land. The datasheet top view was preserved
without mirroring: row A is above row C and A1 is the upper-left land.

## LMR36520 family (dcdc 0.1.2)

Added 2026-11-25 from `ti-lmr36520.pdf`: Texas Instruments LMR36520,
SNVSBF0C, September 2019, revised November 2025, 41 pages, retrieved from
<https://www.ti.com/lit/ds/symlink/lmr36520.pdf> on 2026-11-25, SHA-256
`c5be6bd4f8a480f1dc9a6bea9469e7ab8867a0c83341c1a1f9c153bc015db09b`.

| Family | Device | Members included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|
| LMR36520, HSOIC-8 (DDA) | `LMR36520` | `DCDC_LMR36520ADDAR`: `LMR36520ADDAR`, C2879422, 2,519; `DCDC_LMR36520FADDAR`: `LMR36520FADDAR`, C1858395, 7,970 |

- **Membership.** The Device Comparison Table (p3) lists exactly these
  two orderable members: both 2 A and 400 kHz, the "F" member with forced
  PWM. The pin table is the same and no pin changes function, so they are
  two parts of one device.
- **Pins read.** p4, Figure 5-1 and Table 5-1: PGND 1, VIN 2, EN 3, PG 4,
  FB 5, VCC 6, BOOT 7, SW 8, thermal pad. EN must not float; PG may be
  left open (`optional`); FB must not float or be grounded; VCC takes a
  1 uF capacitor and no load; BOOT takes 100 nF to SW. The thermal pad
  must go to a ground plane and is not an electrical interconnect: it is
  the `required` pin `PAD` on pad 9. Checked from the table to the model
  and back.
- **Values used.** VIN 4.2 V to 65 V (p5).
- **Land.** `TI_DDA0008B`, in this package: TI drawing 4214849/B
  (09/2025), outline p38, board layout p39, stencil p40. 8X 1.55 x 0.6 mm
  lands, 1.27 mm pitch, 5.4 mm between row centres. Pin 1 is at the upper
  left, (-2.7, -1.905) mm; the drawing's +y is negated.
- **Thermal pad land, recorded deviation.** TI draws a solder-mask-defined
  pad: 2.95 x 4.9 mm metal (note 9: the metal size may vary) under a
  2.71 x 3.4 mm mask opening. The land is the 2.71 x 3.4 mm opening as
  copper with no mask expansion, so the exposed metal and the paste (the
  2.71 x 3.4 mm aperture, 100% at 0.125 mm) are TI's, and the copper under
  the mask is left to layout. The optional 0.2 mm vias are not in the
  land.
- **Courtyard.** 7.5 x 5.5 mm: 0.25 mm beyond the 6.95 mm copper across
  the rows and beyond the 5.0 mm maximum body length.
- **Left out.** `LMR33640ADDAR` (C2836966) and `TPS62933DRLR` (C3200405)
  were on the same demand list at one design each and are not modelled
  yet. The LMR33640 sheet prints two DDA drawings (DDA0008J and
  DDA0008B), which must be told apart before a land is chosen.

### LCSC part numbers (not bound)

No part here carries a `supplier lcsc` clause: a binding needs a row in
`tools/sourcing_data/lcsc_map.json` and a frame row with reviewed evidence
in `tools/sourcing_data/jlc_frames.json`, and neither exists. The candidate
numbers and stock are from the local jlcparts snapshot of 2026-11-25, which
has no manufacturer field; the part pages were not read. The preview board
input and build for these candidates is `tools/frame_boards/demand-batch-2/`;
it has not been previewed.
