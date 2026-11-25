# `son`: SON and WSON land patterns

Reusable SON / WSON lands that parts of more than one package bind to.
This package ships lands only: no device, no part and no supplier binding.
Added 2026-11-25 (son 0.1.0).

The `qfn` package keeps the QFN, DFN and SON lands it already owns. A new
SON land that more than one package needs goes here, so that adding it
does not change `qfn`'s published content and every package that pins it.

## Frame convention

CoHDL board frame: +x right, **+y down**, top view, origin at the package
centre. A land keeps the manufacturer's top view as drawn, pin 1 at the
upper left and numbering counter-clockwise; drawing y is negated.

No part number has been read on these lands in an assembler preview. They
are not known to sit on the assembler's frame, and no part bound to them
carries a `supplier lcsc` clause.

## `TI_DRV0006A`

Texas Instruments package drawing DRV0006A, 6-pin WSON, 0.8 mm maximum
height, drawing number 4222173/D, 09/2026. The same drawing is printed in
three datasheets held by the consuming packages, and the land was read
from all three:

| Datasheet | Pages (outline, board layout, stencil) | Stored as | SHA-256 |
|---|---|---|---|
| TPS255x, SLVS841G | 43, 44, 45 | `lib/load-switch/docs/tps255x.pdf` | `b6a2d18ce02fb21eafcc7e1f1d8c4d69074cac38fad44d496682f3393f26445e` |
| LP5912, Rev. D | 35, 36, 37 | `lib/ldo/docs/ti-lp5912.pdf` | `4266eef05203d0d90b7824cd7445df5a857bf95922df68274aef4a6f8c122d88` |
| TLV755P, SBVS320D | 48, 49, 50 | `lib/ldo/docs/ti-tlv755p.pdf` | `f9e676918d1af7144e55835616108def097b48fb5784bd0084ae77a418c64fbf` |

The TPS255x sheet also prints DRV0006D (4225563/A, 12/2019, pages 40 to
42). Its board layout has the same values, so the land does not depend on
which of the two drawings an orderable code is built to.

- **Outline.** Body 1.9 to 2.1 mm square, 0.7 to 0.8 mm high; six
  terminals 0.2 to 0.3 mm wide and 0.25 to 0.35 mm long on 0.65 mm pitch;
  exposed pad 1.0 +/-0.1 by 1.6 +/-0.1 mm.
- **Land.** 6X 0.45 x 0.3 mm with R0.05 mm corners, 4X 0.65 mm pitch,
  1.95 mm between row centres; exposed-pad land 1.0 x 1.6 mm, pad 7.
- **Coordinates.** Pads 1 to 3 down the left column at x = -0.975 mm,
  y = -0.65, 0, +0.65 mm; pads 4 to 6 up the right column. The drawing's
  +y is up the page; it is negated, and numbering stays counter-clockwise
  seen from the top.
- **Paste.** Pad 7's copper carries no paste of its own; two 1.0 x 0.7 mm
  helper lands with the same number, centred at y = +/-0.45 mm, are the
  stencil apertures of the drawing (0.125 mm stencil, 88% coverage).
- **Mask.** TI's preferred non-solder-mask-defined opening, 0.07 mm
  maximum expansion.
- **Not in the land.** The two optional 0.2 mm vias in pad 7 (1.1 mm
  apart).
- **Courtyard.** 2.9 x 2.6 mm: 0.25 mm beyond the 2.40 mm copper in x and
  beyond the 2.1 mm maximum body in y.

Consumers: `load_switch::LOADSW_TPS2553DRVR` and its family,
`ldo::LDO_LP5912_*` and `ldo::LDO_TLV755*P_DRV`.
