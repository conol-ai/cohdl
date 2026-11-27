# Littelfuse 451/453 series evidence record

- Library coverage: `Fuse_451_0A5`, `Fuse_451_1A`, `Fuse_451_2A`,
  `Fuse_451_3A`, `Fuse_451_4A`, their parts and `FP_LITTELFUSE_NANO2`.
- Document identity: Littelfuse, "Surface Mount Fuses, NANO2 > Very
  Fast-Acting Fuse > 451/453 Series", revised 09/10/15, 4 pages.
- Manufacturer source: <https://www.littelfuse.com>. Its document
  endpoints answered 403 to automated retrieval on 2026-11-25, so the
  copy used is the one LCSC serves for C3099.
- JLCPCB part page read on 2026-11-25: C3099, manufacturer Littelfuse,
  MPN `0451001.MRL`, package 2410, class Extended, stock 11,642.
- PDF as served by LCSC for C3099:
  <https://datasheet.lcsc.com/datasheet/pdf/00c9748756ca418792b5b22a2bd74996.pdf>,
  879,459 bytes.
- Retrieved: 2026-11-25.
- SHA-256: `ade583d66dc303591035fa92a9f3874828f567f59a3cb4f5d24984c3de96ea87`.

## Facts used

- Part numbering (page 4): `0451` series (gold-plated caps), amp code,
  `M` 1000 pieces, `R` tape and reel, `L` RoHS and halogen free. The amp
  codes of page 2 give `0451.500MRL`, `0451001.MRL`, `0451002.MRL`,
  `0451003.MRL` and `0451004.MRL`.
- Ratings (page 2, "Electrical Specifications by Item"), all 125 V:

| Ampere rating | Amp code | Interrupting rating | Nominal cold resistance | Nominal melting I2t |
|---|---|---|---|---|
| 0.500 A | `.500` | 50 A at 125 VAC/VDC | 0.4200 Ohm | 0.0824 A2s |
| 1.00 A | `001.` | 50 A at 125 VAC/VDC | 0.1530 Ohm | 0.6029 A2s |
| 2.00 A | `002.` | 50 A at 125 VAC/VDC | 0.0367 Ohm | 0.530 A2s |
| 3.00 A | `003.` | 50 A at 125 VAC/VDC | 0.0227 Ohm | 1.650 A2s |
| 4.00 A | `004.` | 50 A at 125 VAC/VDC | 0.0160 Ohm | 3.152 A2s |

  The interrupting-rating cell is printed once per group of rows; the
  value above is the first line of the cell that spans these rows.
- Dimensions (page 4): body 6.10 +/-0.20 mm long, 2.69 +/-0.25 mm square,
  end caps 1.45 mm.
- Recommended pad layout (page 4): 1.96 mm land length, 3.15 mm land
  width, 2.95 mm between lands, 6.86 mm overall.
- The sheet numbers no terminal and marks no polarity.

The PDF is not redistributed by this repository. Download it from the
source above and verify the recorded SHA-256 before auditing the model.
