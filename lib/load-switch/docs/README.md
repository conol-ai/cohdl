# Load switch source manifest

Audited on 2026-08-05; `mt9700.evidence.md` added 2026-09-04. Entries ending
in `.evidence.md` are repository-authored evidence records; their SHA-256
values identify upstream manufacturer PDFs that are not redistributed here.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `sgm2554.pdf` | `LOADSW_SGM2554` | SGMICRO SGM2554 power distribution switch, April 2022, REV. A.4, 13 pages | <https://www.sg-micro.com> (retrieved via LCSC C1156937) | `58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87` |
| `sgm2554.md` | summary of `LOADSW_SGM2554` (SGM2554A pinout, land, usage notes) | — | — | — |
| `tps22918.evidence.md` | `LOADSW_TPS22918DBVR` | Texas Instruments TPS22918, SLVSD76C; DBV package drawing 4214840/G | <https://www.ti.com/lit/ds/symlink/tps22918.pdf> | `1b28eb58003d28b18bf919a15734056f296a499bebee2bd493a1e4358c4dfb1c` |
| `mt9700.evidence.md` | `LOADSW_MT9700N` | Xi'an Aerosemi Technology MT9700 Rev2.1, 8 pages | <https://www.aerosemi.com> (retrieved via LCSC C42441843, productModel MT9700-N) | `b51be4566fc3e85022bbafd41874b29949e2bc4eada688ae912d361a19149ec8` |
| `tps255x.pdf` | `LOADSW_TPS2553DBVR`, `LOADSW_TPS2553DBVR_1`, `LOADSW_TPS2552DBVR` | Texas Instruments TPS2552, TPS2552-1, TPS2553, TPS2553-1, SLVS841G, November 2008, revised October 2026, 46 pages; DBV package drawing 4214840/G | <https://www.ti.com/lit/ds/symlink/tps2553.pdf> (retrieved 2026-11-25) | `b6a2d18ce02fb21eafcc7e1f1d8c4d69074cac38fad44d496682f3393f26445e` |

SHA-256 checksums:

```text
58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87  sgm2554.pdf
1b28eb58003d28b18bf919a15734056f296a499bebee2bd493a1e4358c4dfb1c  tps22918.pdf (upstream, not vendored)
b51be4566fc3e85022bbafd41874b29949e2bc4eada688ae912d361a19149ec8  mt9700.pdf (upstream, not vendored)
b6a2d18ce02fb21eafcc7e1f1d8c4d69074cac38fad44d496682f3393f26445e  tps255x.pdf
```

## MT9700-N retrieval and land notes

- The accepted orderable model is `MT9700-N`; LCSC
  C42441843 lists `productModel` `MT9700-N`, brand `XI'AN Aerosemi Tech`,
  SOT-23-5, and serves this Rev2.1 document on that listing. The sibling
  SKU C89855 (`MT9700`, same SOT23-5) serves the older Rev1.0 document;
  only the document attached to the exact `MT9700-N` listing is used.
- The Rev2.1 order table lists the family part number `MT9700` with top
  mark `D00HAW`; pin configuration (VOUT 1, GND 2, SET 3, EN 4, VIN 5) and
  the SOT23-5 outline are on p2 and p7.
- Aerosemi publishes no recommended PCB land pattern (Figure 3 is
  topological only), so the part uses the nominal generic KiCad SOT-23-5
  land (JEDEC MO-178 Var AA) recorded beside `FP_Aerosemi_SOT23_5`. That
  secondary generic reference is not Aerosemi authority and keeps the
  assembly-validation requirement (the accepted `W25Q128JVPIQ` precedent).

TPS22918 application boundaries updated 2026-10-17 against the same hashed
manufacturer PDF: output discharge does not establish reverse-current
blocking; see the attached evidence record for the specific sections.

## SGM2554 pin map and land correction in 0.2.0

Re-audited `sgm2554.pdf` on 2026-11-07 against the rendered p. 3 pin
configuration and the p. 11 recommended land pattern. Version 0.2.0 is a
breaking change to `SGM2554` / `LOADSW_SGM2554`:

- **Pin map.** Earlier versions declared 1=IN, 2=GND, 3=EN, 4=NC, 5=OUT.
  The datasheet's SGM2554A is 1=VOUT, 2=GND, 3=VIN, 4=EN, 5=VOUT (p. 3;
  the p. 1 typical application and p. 5 test circuit agree). A board wired
  to the old names put the supply on an output pad, drove the switch input
  from the enable net, and left the real EN unconnected. The device now
  declares `VOUT: 1, 5 [power_out]`, `GND: 2`, `VIN: 3 [power_in]` and
  `EN: 4 [input]` (logic high enables, p. 3). The pin names `IN`, `OUT` and
  `NC` are gone, so a design that wires them fails to check after
  `cohdl update` and must be rewired to `VIN`, `VOUT` and `EN`.
- **Part identity.** The primary MPN is now the A model's ordering code
  `SGM2554AYN5G/TR` (p. 2), not the family name `SGM2554`. Only the A model
  has EN; on `SGM2554BYN5G/TR` pin 4 is a second VIN (p. 3).
- **Input range.** The comments now give 2.2-5.5V (p. 1, p. 4), not
  1.0-5.5V.
- **Land.** `FP_SGM2554_SOT235` now uses the recommended land of p. 11:
  0.69 x 0.99mm pads (was 0.60 x 1.00mm) on 2.59mm row centres (was
  2.60mm), 0.95mm pitch unchanged. The courtyard grows from 4.1 x 3.0mm to
  4.1 x 3.6mm so it contains the 3.050mm maximum body length D (p. 11), and
  the reference designator moves from y = -2.2mm to -2.4mm. The footprint
  name, pad numbers and pad order are unchanged. The old comment's claim
  that the land matched `SOT5P95X290X145N` in `lib/@contrib/ldo` is removed:
  that package's RT9080 footprint is `SOT5P95X290X160N`, with 1.0 x 0.7mm
  pads.

The `sf32-miniboard` example had worked around the wrong map with a local
`SGM2554A_CORRECT` device on this land, under the same ordering code. It now
pins load-switch 0.2.0 and uses `LOADSW_SGM2554` directly; its connectivity
is unchanged.

## LCSC binding for TPS22918 (load-switch 0.2.1)

`LOADSW_TPS22918DBVR` binds its primary to LCSC C131941 with a half-turn map: `supplier lcsc "C131941" rotate 180`. The part number is in `tools/sourcing_data/lcsc_map.json` (exact rule, 2026-11-07). No land changes from 0.2.0.

- **Why a map.** Preview board 4 placed C131941 with no mapping (`tools/sourcing_data/jlc_previews/2026-11-11-board-4/`): on all four copies the model's leads were on our pads with its pin 1 at our pad 4, half a turn: 183.0° and 183.0° (top, 0° and 37°) and 177.4° and 178.2° (bottom) counter-clockwise from our pad 1. `FP_TI_DBV0006A` follows TI's DBV0006A drawing and stays where it is; the binding carries the turn.
- **Evidence.** The frame row `pb5-fptidbv0006a-c131941` cites preview board 5 (`tools/sourcing_data/jlc_previews/2026-11-14-board-5/`), which placed the part with `rotate 180` applied. Its pin 1 mark read 1.8° and 1.2° (top) and 358.1° and 357.7° (bottom), 0.10–0.11 mm from pad 1. By eye, on all four copies the model's six leads lie on our pads, three a side, with its pin 1 dimple at the pad 1 corner.
- **Per part number, not per land.** C12044 on this same land read `rotate 270` on C2 frame board 1 (row `c2b1-sot236_dbv-c12044`, which binds nothing in this package), and C85364 on `esd::FP_SOT_23_6` read 180. A rotation proved for C131941 says nothing about another part number on `FP_TI_DBV0006A`.
- **What changes in output.** A plain `build` writes the same BOM and CPL; the netlist gains the part's `LCSC` field. `--supplier lcsc` writes the turn into the LCSC CPL.

## TPS255x family (load-switch 0.2.2)

Added 2026-11-25 from `tps255x.pdf`, stored as downloaded from ti.com. A
family is modelled as one device per pin table and one part per stocked
member.

| Family | Device | Pin 3 | Members included (MPN, LCSC candidate, page stock) |
|---|---|---|---|
| TPS2553, SOT-23-6 (DBV) | `TPS2553` | `EN`, logic high turns the switch on | `LOADSW_TPS2553DBVR`: `TPS2553DBVR`, C55266, 41,809; `LOADSW_TPS2553DBVR_1`: `TPS2553DBVR-1`, C111738, 9,519 |
| TPS2552, SOT-23-6 (DBV) | `TPS2552` | `EN_N`, logic low turns the switch on | `LOADSW_TPS2552DBVR`: `TPS2552DBVR`, C46506, 9,492 |

- **Membership rule.** Two orderable parts share a device only when the
  datasheet's pin table gives them the same pin numbers and the same pin
  functions. Table 5-1 (p4) gives the TPS2552 and the TPS2553 the same DBV
  pin numbers, but pin 3 is active low on the TPS2552 and active high on the
  TPS2553, so they are two devices on one land. The "-1" suffix changes the
  overcurrent response only (latch-off instead of constant current with
  auto-retry, Table 4-1 on p3 and section 8.3 on p14); the pin table is the
  same, so `TPS2553DBVR-1` is a second part on `TPS2553`.
- **Pins read.** p4, Figure 5-1 and Table 5-1: IN 1, GND 2, EN 3, FAULT 4,
  ILIM 5, OUT 6. Checked from the table to the model and from the model
  back to the table. `FAULT` is an active-low open-drain output and is
  named `FAULT_N`; it is `optional`. `ILIM` is typed "O" by TI and modelled
  `passive`: it is the current-limit programming node (15 kOhm to 232 kOhm
  to GND, p5) and may be tied to IN (p7).
- **Ratings used in comments.** VIN 2.5 V to 6.5 V and the 0.1 uF minimum
  input capacitor (p5, p4); up to 1.5 A continuous (p1, p5).
- **Stock** in the table above is the value on each JLCPCB part page on
  2026-11-25; the snapshot values are in "LCSC part numbers" below.
- **Left out, and why.**

| MPN | LCSC | Why it is not here |
|---|---|---|
| `TPS2553DDBVR` | C521201 (5,711 in stock) | The JLCPCB page lists it as SOT-23-6. It is the TPS2553D, which `tps255x.pdf` does not cover, so its pin table has not been read. |
| `TPS2552DBVR-1`, `TPS25530DRVR-1`, other reel and `G4` codes | none confirmed | No LCSC part number was confirmed for them. |

- **Adding a member.** Read its row in Table 4-1 and its column in Table
  5-1. Same pin numbers and functions as an existing device: add a `part`
  with the exact orderable MPN from the package addendum (pp29-30). Any
  different pin function or package: add a device.

### Land

All three parts use `FP_TI_DBV0006A`, already in this package for the
TPS22918. `tps255x.pdf` prints the same TI drawing, DBV0006A 4214840/G
(outline p36, example board layout p37): six 1.1 x 0.6 mm lands, 0.95 mm
pitch, 2.6 mm between row centres. The footprint is unchanged.

- **Coordinates.** TI's top view as drawn: pins 1 to 3 down the left
  column, 4 to 6 up the right. The drawing's +y is up and the board frame's
  +y is down, so pin 1 is at (-1.3, -0.95) mm and numbering is
  counter-clockwise seen from the top. Checked on the emitted
  `.kicad_mod` of a temporary design.
- **Courtyard not changed.** It is 4.1 x 3.0 mm: 0.20 mm beyond the
  3.70 mm copper in x, where the library rule asks for 0.25 mm, and
  0.025 mm inside the 3.05 mm maximum body length in y (p36). Growing it
  to 4.2 x 3.6 mm was tried and withdrawn on 2026-11-25: preview board 4
  packs its rows by courtyard size, so the change reorders a board whose
  preview is on record. It is left for a change that owns that board.
  Pad numbers, sizes and centres are untouched in 0.2.2, so the C131941
  reading on this land still applies; its frame row now names 0.2.2.

### LCSC part numbers (not bound)

No part here carries a `supplier lcsc` clause. The clauses added on
2026-11-25 were removed the same day: the library binds a part only with a
row in `tools/sourcing_data/lcsc_map.json` and a frame row with evidence in
`tools/sourcing_data/jlc_frames.json` (`tests/library/supplier.rs`), and
neither exists for these part numbers. The numbers below are candidates for
that work, not bindings.

| Part | Candidate | Stock, jlcparts snapshot 2026-11-25 | Stock, JLCPCB part page 2026-11-25 |
|---|---|---|---|
| `LOADSW_TPS2553DBVR` | C55266 | 54,233 | 41,809 |
| `LOADSW_TPS2553DBVR_1` | C111738 | 2,869 | 9,519 |
| `LOADSW_TPS2552DBVR` | C46506 | 8,940 | 9,492 |

- **Identity.** Each candidate was read on its JLCPCB part page on
  2026-11-25 (`https://jlcpcb.com/partdetail/<C-number>`, through the
  shared fetch client): manufacturer, exact MPN, package, class Extended.
- **The two stock columns differ, and both are kept.** The snapshot and
  the part page carry the same date and do not agree: C55266 reads 54,233 in the snapshot and
  41,809 on the page, 12,424 fewer. Neither
  is corrected to the other: stock is a reading of one source at one time.
- **Missing for a map row.** A reviewed `tools/lcsc_backfill.py` run that
  confirms every candidate page of the MPN, promoted with
  `tools/promote_lcsc_map.py`. The local jlcparts snapshot cannot stand in
  for it: it has no manufacturer field.
- **Missing for a frame row.** An assembler preview of the part number on
  `FP_TI_DBV0006A`, reviewed by a person. On this land C131941 needed `rotate 180` and C12044 read 270
  (see the TPS22918 section), so no rotation can be assumed.
- **Preview board input.** `tools/frame_boards/demand-batch-1/` holds the
  manifest for that preview. It has not been built or previewed.

## TPS255x WSON members (added to load-switch 0.2.2 on 2026-11-25)

The DRV (6-pin WSON) members of the same datasheet, on their own devices
and their own land. They were the first item left for the next batch above
and are no longer left out.

| Family | Device | Pin 4 | Members included (part: MPN, LCSC candidate, snapshot stock) |
|---|---|---|---|
| TPS2553, WSON-6 (DRV) | `TPS2553_DRV` | `EN`, logic high turns the switch on | `LOADSW_TPS2553DRVR`: `TPS2553DRVR`, C138719, 28,273; `LOADSW_TPS2553DRVR_1`: `TPS2553DRVR-1`, C411878, 1,050 |
| TPS2552, WSON-6 (DRV) | `TPS2552_DRV` | `EN_N`, logic low turns the switch on | `LOADSW_TPS2552DRVR`: `TPS2552DRVR`, C473916, 1,272 |

- **Pins read.** p4, Figure 5-2 and the WSON column of Table 5-1: OUT 1,
  ILIM 2, FAULT 3, EN 4, GND 5, IN 6, PowerPAD "internally connected to
  GND". Not the DBV numbering (IN 1 ... OUT 6), which is why these are
  separate devices. The pad is pad 7 of the land and is listed with pin 5
  under `GND`. Checked from the table to the model and back.
- **Same rules as the DBV devices**: enable polarity makes TPS2552 and
  TPS2553 two devices; the "-1" latch-off member is a second part.
- **Land.** `son::TI_DRV0006A` (`son` 0.1.0, a new dependency), drawing
  4222173/D on pp43-45; see `lib/son/docs/README.md`. The sheet also
  prints DRV0006D (pp40-42) with the same board layout values. The parts
  are never placed on `FP_TI_DBV0006A`.
- **Left out.** `TPS2553DRVT`, `TPS2553DRVT-1`, `TPS2552DRVR-1` and
  `TPS2552DRVT-1`: 6 to 101 pieces in the snapshot. `TPS2553DRVR` was
  read on its JLCPCB page on 2026-11-25 (21,362 in stock, WSON-6-EP(2x2));
  the other two candidates were not read on their pages.

### LCSC part numbers (not bound), WSON members

No part here carries a `supplier lcsc` clause: a binding needs a row in
`tools/sourcing_data/lcsc_map.json` and a frame row with reviewed evidence
in `tools/sourcing_data/jlc_frames.json`, and neither exists. The candidate
numbers and stock are from the local jlcparts snapshot of 2026-11-25, which
has no manufacturer field; the part pages were not read. The preview board
input and build for these candidates is `tools/frame_boards/demand-batch-2/`;
it has not been previewed.
