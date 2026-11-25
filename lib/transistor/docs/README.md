# Official source documents

`jscj_s8050.evidence.md` records the authorized manufacturer document, exact
ordering identity, pinout, electrical limits, SOT-23 dimensions and suggested
land used by `transistor::BJT_JSCJ_S8050_J3Y`.

## LCSC binding (transistor 0.2.1)

`BJT_JSCJ_S8050_J3Y` binds its primary to LCSC C2146 with a zero map: `supplier lcsc "C2146"`. The part number is in `tools/sourcing_data/lcsc_map.json` since the B2 full run, and its frame row `c2b1-sot23_s8050-c2146` cites C2 frame board 1 (`tools/sourcing_data/jlc_previews/2026-10-30-c2-board-1/`). That preview read a quarter turn on the land as it was before 0.2.0, which `FP_JSCJ_SOT23` has since been moved onto. No land changed from 0.2.0, so a plain `build` writes the same BOM and CPL. The three `transistor` frame rows named 0.2.1; since 0.2.2 they name 0.2.2 (no land of this package changed).

## BSS138 family (transistor 0.2.2)

Added 2026-11-25. Until 0.2.2 this package held bipolar transistors only;
it now also holds a small-signal MOSFET, with its own marker trait
`FieldEffectTransistor` (designator "Q"). The package gains the dependency
`sot` 0.1.2 for the land.

| Local file | Covered exact MPN | Document version/date | Official source | SHA-256 |
|---|---|---|---|---|
| `onsemi-bss138.pdf` | onsemi `BSS138` | BSS138/D, Rev. 7, April 2024, 8 pages; case 318 issue AU, 14 Aug 2024 | <https://www.onsemi.com/pdf/datasheet/bss138-d.pdf> (retrieved 2026-11-25) | `e12090e28c1734d037e7809a5fdefcdabbce27a4f92ade591b8e30ceb00b76b1` |

A family is modelled as one device per pin table and one part per stocked
member.

| Family | Device | Pins | Members included (part: MPN, LCSC, stock) |
|---|---|---|---|
| onsemi BSS138, SOT-23-3 | `BSS138` | `Gate` 1, `Source` 2, `Drain` 3 | `FET_BSS138`: `BSS138`, C52895, 88,700 |

- **Pins read.** p1 marking diagram: 1 Gate, 2 Source, 3 Drain, which is
  Style 21 of case 318 on p7. Checked from the sheet to the model and from
  the model back to the sheet.
- **Values used (page).** VDSS 50 V, VGSS +/-20 V, ID 0.22 A continuous,
  0.88 A pulsed, PD 0.36 W (p2); VGS(th) 0.8 V min, 1.3 V typ, 1.5 V max at
  1 mA (p2); RDS(on) 3.5 Ohm max at VGS 10 V and 6.0 Ohm max at 4.5 V (p1).
- **Stock** is the value on the JLCPCB part page on 2026-11-25. The
  requester's jlcparts snapshot of the same date gave 52k.
- **Left out, and why.**

| MPN | LCSC | Why it is not here |
|---|---|---|
| onsemi `BSS138-G` | none confirmed | Same pin table (p1 ordering table); no LCSC part number was confirmed for it. Add it as a second `part` on `BSS138` once one is. |
| onsemi `BSS138L`, `BSS138LT1G` and other "BSS138" types; other makers' BSS138 | none confirmed | Not on this sheet. This sheet has no series table and names no sibling, so nothing can be said about their pin tables from it. Each needs its own document read first. |

- **Adding a member.** A type joins `BSS138` only when its own datasheet
  gives 1 Gate, 2 Source, 3 Drain in SOT-23-3. A different pin order,
  package or channel type is a new device. The ratings in the device's
  `spec` are this sheet's; a member with other ratings is a new device as
  well.

### Land

`FET_BSS138` uses `sot::SOT3P190X290X130N` (`sot` 0.1.2, unchanged), not
this package's `FP_JSCJ_SOT23`: the `sot` land is the shared one for new
SOT-23 parts (`lib/README.md`).

| Source | Land along the row x across | Pin 1 to 2 centres | Across the rows |
|---|---|---|---|
| `sot::SOT3P190X290X130N` | 0.80 x 0.90 mm | 1.90 mm | 2.00 mm centres, 2.90 mm overall |
| onsemi case 318 issue AU, p6, "Recommended Mounting Footprint" | 0.56 x 0.95 mm | 1.90 mm (0.95 pitch) | 2.90 mm overall, so 1.95 mm centres |

- **Difference.** Same pad numbers, same 1.90 mm spacing and the same
  2.90 mm overall extent. The `sot` lands are 0.24 mm wider along the row
  (0.12 mm a side, leaving a 1.10 mm gap between pads 1 and 2) and 0.05 mm
  shorter, all of it at the inner end.
- **Fit to the p6 outline.** Lead width b 0.37 to 0.50 mm on 0.80 mm
  lands. Lead span HE 2.10 to 2.64 mm puts the tips 1.05 to 1.32 mm from
  the centre line, 0.13 to 0.40 mm inside the 1.45 mm outer land edge.
  Body D 2.80 to 3.04 mm by E 1.20 to 1.40 mm.
- **Courtyard.** 3.4 x 3.6 mm: 0.25 mm beyond the 2.90 mm copper across
  the rows, and 0.28 mm beyond the 3.04 mm maximum body length.
- **Coordinates.** onsemi's top view has pins 1 and 2 on the lower edge
  (1 left, 2 right) and pin 3 above. The `sot` frame is that view turned a
  quarter turn so the pin 1-2 edge faces +x: pad 1 at (1.0, 0.95), pad 2 at
  (1.0, -0.95), pad 3 at (-1.0, 0) mm, +y down. Numbering stays
  counter-clockwise 1-2-3 seen from the top, so gate and source are not
  swapped. Checked on the emitted `.kicad_mod` of a temporary design.

### LCSC binding

`supplier lcsc "C52895"` with no `at` or `rotate`.

- **Identity.** Read on the JLCPCB part page on 2026-11-25
  (`https://jlcpcb.com/partdetail/C52895`, through the shared fetch
  client): manufacturer onsemi, MPN `BSS138`, package SOT-23, class
  Extended.
- **Not yet in the sourcing tables.** C52895 has no row in
  `tools/sourcing_data/lcsc_map.json` and no row for
  `sot::SOT3P190X290X130N` in `tools/sourcing_data/jlc_frames.json`, so
  the supplier audit in `tests/library/supplier.rs` rejects it until a
  backfill run and an assembler preview add them.
- **Rotation is unverified.** No assembler preview has placed C52895 on
  this land. The zero map is a placeholder, not a reading. Do not order
  assembly with `--supplier lcsc` from this binding before a preview.
