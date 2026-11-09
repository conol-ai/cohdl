# Official documentation

Retrieved from Winbond on the dates recorded below. The PDFs are stored
byte-for-byte as downloaded; none was regenerated or optimized.

## `w25q32rv-datasheet.pdf`

- Covers: `W25Q32RVXHJQ`, package code XH (8-pad XSON
  2 x 3 x 0.4 mm).
- Document identity: Revision E, 2025-11-13.
- Official URL:
  <https://www.winbond.com/resource-files/W25Q32RV_SPI_QPI%20RevE%2011132025%20Plus.pdf>
- SHA-256:
  `a863fb8b34fa72d70eb9ee0fa8ffe5ad911b57383e17b0cf8a1eef4118061d2a`
- Footprint authority: section 10.3 provides the official package-terminal
  dimensions, but Winbond does not publish a recommended PCB land pattern
  for XH in this datasheet.

The pin configuration was visually inspected on PDF page 6 and the XH
package drawing on PDF page 78. The part uses the package-local land
`FP_W25Q32RV_XSON8_2X3`, derived from that drawing; its dimensions and the
non-Winbond land margins are recorded beside the footprint source (see the
0.3.0 entry below).

## `w25q128jv.pdf`

- Retrieved and audited: 2026-08-04.
- Covers: `W25Q128JVSIQ`, package code S (8-pin SOIC 208-mil), and
  `W25Q128JVPIQ`, package code P (8-pad WSON 6 x 5 mm).
- Document identity: Revision G, 2019-04-08.
- Official source: <https://www.winbond.com>.
- SHA-256:
  `5e2f6a77e0225a5c61b8f4b5b9680b57e0e1fe68cfc011465e30f665fe282057`

The SOIC and WSON pin configurations and descriptions were visually inspected
on PDF pages 5-6; they identify CLK as pin 6 and HOLD/RESET (IO3) as pin 7.
The package drawings were inspected on PDF pages 68-69. Winbond specifies the
package dimensions but does not publish a recommended PCB land pattern.

## Secondary land-pattern references

The W25Q128JV part uses the reusable
`soic::SOIC8P127X790X216N` land pattern. Its dimensional derivation, secondary
KiCad reference, and assembly-validation limitation are recorded beside the
footprint in `lib/soic`.

The W25Q128JVPIQ part uses a package-local WSON 6 x 5 mm footprint. Its
perimeter copper and courtyard follow KiCad 9's generated generic
`WSON-8-1EP_6x5mm_P1.27mm_EP3.4x4.3mm`; the center metal dimensions come
directly from Winbond page 69. This secondary land-pattern choice still
requires assembly validation.

## Footprint frame change in 0.2.0

Version 0.2.0 follows soic 0.3.0, which turned `soic::SOIC8P127X790X216N` by a quarter turn (270) onto the
assembler's frame (library plan W3). Pad numbers and lands are unchanged, but
a design that places a part on one of these lands sees its copper turn after
`cohdl update`, with `check` and `build` still passing. Migrate the design's
`place` statements first, as the land package's README describes, with
`tools/realign_place.py`.

## W25Q32 land correction in 0.3.0 (2026-11-07)

Re-audited the XH drawing (PDF page 78) and Figure 1b (PDF page 6).
Through 0.2.0, `FLASH_W25Q32` used `qfn::QFN8N50P200X300_1EP61X220`,
KiCad's generic DFN-8 2 x 3 mm land. That package has its terminal rows
along the 3 mm edges, 2 mm apart. The Winbond XH package has them along the
2 mm (D) edges, 3 mm (E) apart: E/2 = E2/2 + K + L = 0.10 + 0.95 + 0.45 =
1.50 mm, so the terminals span 1.05-1.50 mm from the center (center
1.275 mm). The old lands, 0.87 x 0.25 mm at x = +/-0.94 mm (0.505-1.375 mm),
covered only 0.325 mm of each 0.45 mm terminal, and its 0.61 x 2.2 mm center
land was longer than the 2.0 mm body edge it runs along, against a
0.20 x 1.60 mm center metal. The old footprint comment also cited a
"rev.e 05152024" PDF; the stored datasheet is Revision E of 2025-11-13.

0.3.0 points the part at the new package-local land
`flash::FP_W25Q32RV_XSON8_2X3`:

| Feature | 0.2.0 (`qfn::QFN8N50P200X300_1EP61X220`) | 0.3.0 (`flash::FP_W25Q32RV_XSON8_2X3`) |
|---|---|---|
| Pads 1-4 / 5-8 centers | x = -/+0.94, y = -0.75 ... +0.75 | x = -/+1.375, y = -0.75 ... +0.75 |
| Perimeter land | 0.87 x 0.25 | 0.85 x 0.25 (x = 0.95-1.80 from center) |
| Pad 9 (center metal) | 0.61 x 2.2 | 0.20 x 1.60 |
| Courtyard | 3.3 x 3.5 | 4.1 x 2.5 |

Pin 1 is top-left with pads 1-4 down the left row in both, so the part's
pin numbering and orientation are unchanged. Winbond publishes no PCB land
pattern for XH; the 0.10 mm heel and 0.30 mm toe around the nominal
terminal are local choices, not Winbond's, and still require assembly
validation. The center land matches the nominal center metal; Winbond says
to avoid exposed PCB vias under it.

This is breaking: a design that instantiates `FLASH_W25Q32` gets different
copper after `cohdl update`, and the footprint name in its netlist and BOM
changes. The part has no `supplier lcsc` binding and no assembler frame
evidence yet. `flash` no longer depends on `qfn`;
`qfn::QFN8N50P200X300_1EP61X220` itself is unchanged.
