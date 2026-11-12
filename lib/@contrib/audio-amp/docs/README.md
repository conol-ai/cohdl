# Audio amplifier source manifest

Retrieved and audited on 2026-08-04. The PDFs are stored byte-for-byte as
downloaded; none was regenerated or optimized.

| Local file | Exact library coverage | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- | --- |
| `max98357a.pdf` | `MAX98357A` pins, `MAX98357AETE+` order code, and T1633+4 package outline | Analog Devices rev 16 | <https://www.analog.com> | `dea228ad619901d34c2f5f106243bdb9b9e95e38989d4d66fd6c6c489bfa6f1c` |
| `ns4150b.pdf` | Legacy `NS4150` pin map and MSOP-8 dimensions, corroborating the current exact `NS4150B` source below | Nsiway V1.0 | <https://www.nsiway.com.cn> | `61812f56feefc64f799d60b1807b5411ffe2c338687163f3b81ba1ccf6ced2f1` |

SHA-256 checksums:

```text
dea228ad619901d34c2f5f106243bdb9b9e95e38989d4d66fd6c6c489bfa6f1c  max98357a.pdf
61812f56feefc64f799d60b1807b5411ffe2c338687163f3b81ba1ccf6ced2f1  ns4150b.pdf
```

Additional footprint and exact-product evidence audited on 2026-08-05:

- Analog Devices land pattern 90-0031 rev C, official source
  <https://mds.analog.com/api/public/content/90-0031.pdf>, SHA-256
  `5661318725986f79e664adb5fd24d80c2b444c1d48d3aa4c1b0d51ebfa53f4c8`.
- Nsiway's official [`NS4150B` MSOP-8 product page](https://nsiway.com.cn/list_38/82.html)
  hosts the December 2022 V1.4 data-sheet pages. Relevant page-image SHA-256
  values are recorded in `footprint-derivations.md`; page 2 confirms the exact
  eight-pin assignment and page 9 confirms the MSOP-8 dimensions.
- `footprint-derivations.md` records the auditable IPC-7351 nominal-density
  derivation for `SOP8P65X490X110N`.

## Manufacturer name in 0.1.1

`AMP_NS4150B` now names its manufacturer `NSIWAY`, not `Nsiway`, the spelling of the official `audio-amp` package. One library name per company lets the reviewed supplier alias for Shenzhen Nsiway cover both packages (#73, rows 39 and 40). The part, its pins and its land are unchanged.

## MAX98357A SD_MODE obligation in 0.2.0

Re-audited pages 7, 15 and 17 of `max98357a.pdf` on 2026-11-07. `SD_MODE`
(pin 4) has an internal 100 kΩ pull-down (RPD 92–108 kΩ, p. 7), and Table 5
(p. 17) defines a level below the B0 trip point as shutdown, so an open pin
gives a silent amplifier. Earlier versions declared `SD_MODE` `optional`;
0.2.0 declares it `required`, and its comment now records the Table 5
channel select: high selects left, a pull-up through RSMALL selects right and
one through RLARGE selects (left + right)/2, with Table 6 values of
69.8 kΩ/300 kΩ at 1.8 V and 210.2 kΩ/634 kΩ at 3.3 V. This is breaking: a
design that left `SD_MODE` unconnected now fails `cohdl check` after
`cohdl update`; wire it to the logic level or pull-up for the channel it
needs. The pin number, name and role, the part identity and the land are
unchanged.

## LCSC binding for NS4150B (0.2.1)

`AMP_NS4150B` binds its primary to LCSC C189961 with a quarter-turn map: `supplier lcsc "C189961" rotate 270`. The part number is in `tools/sourcing_data/lcsc_map.json` (alias rule, 2026-11-07). `AMP_MAX98357A` stays unbound: no preview has read it.

- **Why a map.** On preview board 4 (`tools/sourcing_data/jlc_previews/2026-11-11-board-4/`, section A), placed with no mapping, the assembler's pin 1 for C189961 lay a quarter turn from our pad 1 on all four copies, top and bottom, at 0° and 37°: 94.8°, 95.2°, 95.1° and 94.7°. The model's leads ran above and below where our pads are left and right. `SOP8P65X490X110N` follows the datasheet (`footprint-derivations.md`) and stays where it is; the binding carries the turn.
- **Confirmed with the map applied.** The frame row `owtc-sop8p65x490x110n-c189961` cites a real board, OpenWalkieTalkie Rev C (`tools/sourcing_data/jlc_previews/2026-11-12-owt-rev-c/`). With `rotate 270` applied, U1's pin 1 mark read 5.9° from pad 1, on pad 1's lead, with the model's eight leads on our eight pads. That board has one copy of this part; board 4 has four, without the map. Both boards pinned the same `@contrib/audio-amp` 0.2.0 content.
- **The map is this part number's.** It says how the assembler drew C189961. Another part number on this land needs its own preview reading.
- **What changes in output.** No land, pin or part identity changed from 0.2.0. A plain `build` writes the same BOM and CPL; a design that places `AMP_NS4150B` gains the part's `LCSC` field in its netlist. `--supplier lcsc` writes the turn into the LCSC CPL.
