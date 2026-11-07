# W25Q32RVXHJQ — 32 Mbit 3 V SPI/Dual/Quad/QPI NOR flash, XSON-8 2 x 3 x 0.4 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "flash::W25Q32",
  "parts": ["flash::FLASH_W25Q32"],
  "identities": [
    {"mfr": "Winbond", "mpn": "W25Q32RVXHJQ", "lcsc": "C32246052"}
  ],
  "sources": [
    {"id": "ds", "title": "W25Q32RV 3V 32M-bit Serial NOR Flash Memory with Dual/Quad SPI & QPI", "revision": "Revision E, 2025-11-13",
     "url": "https://www.winbond.com/resource-files/W25Q32RV_SPI_QPI%20RevE%2011132025%20Plus.pdf", "sha256": "a863fb8b34fa72d70eb9ee0fa8ffe5ad911b57383e17b0cf8a1eef4118061d2a", "local": "lib/flash/docs/w25q32rv-datasheet.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages (1-based); the printed page number is one lower.

## Summary

32 Mbit (4 MB) serial NOR flash with standard, Dual, Quad SPI and QPI, 256-byte pages and uniform
4 KB / 32 KB / 64 KB erase, on one 2.7 V to 3.6 V supply over –40 °C to +105 °C (ds p. 5, p. 68).
Clock up to 133 MHz for all instructions except 03h (ds p. 73). The XH package is a 2 x 3 x 0.4 mm
XSON with a non-connected centre strip (ds p. 78). The "Q" ordering option fixes QE = 1, so pads 3
and 7 are IO2/IO3 and /HOLD, /RESET are unavailable (ds p. 16, p. 81). XH is a special-order package
(ds p. 82). Choose another part for 1.8 V I/O.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Clock FR, all instructions except Read Data 03h | D.C. | — | 133 | MHz | Read start address 4-byte aligned, [A1,A0] = 00 (note 6) | ds p. 73 |
| Clock FR, all instructions except Read Data 03h | D.C. | — | 104 | MHz | No address-alignment restriction (note 7) | ds p. 73 |
| Clock fR, Read Data 03h | D.C. | — | 66 | MHz | 4-byte aligned start address (note 6) | ds p. 73 |
| Clock fR, Read Data 03h | D.C. | — | 50 | MHz | No alignment restriction (note 7) | ds p. 73 |
| Standby current ICC1 | — | 10 | 28 | µA | /CS = VCC, VIN = GND or VCC, ≤ 85 °C (max by design) | ds p. 71 |
| Standby current ICC1 | — | 10 | 60 | µA | /CS = VCC, VIN = GND or VCC, ≤ 105 °C | ds p. 71 |
| Power-down current ICC2 | — | 0.1 | 8 | µA | /CS = VCC, VIN = GND or VCC | ds p. 71 |
| Read current ICC3 | — | 6 / 7 / 9 / 11 | 12 / 15 / 18 / 20 | mA | 50 / 84 / 104 / 133 MHz, C = 0.1/0.9 VCC, DO open | ds p. 71 |
| Write-status / program / erase current ICC4–ICC7 | — | 8 | 15 | mA | /CS = VCC | ds p. 71 |
| Input low / high voltage VIL / VIH | –0.5 (VIL) / VCC x 0.7 (VIH) | — | VCC x 0.3 (VIL) / VCC + 0.4 (VIH) | V | — | ds p. 71 |
| Output low voltage VOL | — | — | 0.2 | V | IOL = 100 µA | ds p. 71 |
| Output high voltage VOH | VCC – 0.2 | — | — | V | IOH = –100 µA | ds p. 71 |
| Input / output capacitance | — | — | 6 / 8 | pF | VIN / VOUT = 0 V; sample-tested, 25 °C, 3.0 V | ds p. 71 |
| Input / I/O leakage | — | — | ±2 | µA | — | ds p. 71 |
| Write Status Register time tW | — | 1.5 | 15 | ms | — | ds p. 74 |
| Page program time tPP | — | 0.25 | 2 | ms | Up to 256 bytes | ds p. 74 |
| Sector erase tSE (4 KB) | — | 30 | 240 | ms | — | ds p. 74 |
| Block erase tBE1 (32 KB) / tBE2 (64 KB) | — | 80 / 120 | 800 / 1200 | ms | — | ds p. 74 |
| Chip erase tCE | — | 6 | 40 | s | — | ds p. 74 |
| Software reset time tRST | — | — | 30 | µs | After 66h + 99h | ds p. 74 |
| /CS high to power-down tDP / to standby tRES1 | — | — | 3 | µs | — | ds p. 74 |
| Clock low to output valid tCLQV | — | — | 4.5 | ns | CL = 30 pF | ds p. 72, p. 73 |
| VCC(min) to /CS low tVSL | 20 | — | — | µs | Characterized only | ds p. 69 |
| Delay before write instruction tPUW | 5 | — | — | ms | Characterized only | ds p. 69 |
| Write-inhibit threshold VWI | 1.0 | — | 2.0 | V | Characterized only | ds p. 69 |
| Power-cycle: VCC below VPWD for tPWD | 100 (tPWD) | — | 1.2 (VPWD) | µs / V | Characterized only | ds p. 69, p. 70 |
| Program/erase endurance | 100,000 | — | — | cycles | Feature list | ds p. 5 |
| Data retention | 20 | — | — | years | "More than 20-year", feature list | ds p. 5 |
| JEDEC ID (9Fh) | — | EFh 40h 16h | — | — | W25Q32RV-Q: MF = EFh, ID15–ID0 = 4016h | ds p. 20 |
| Device ID (ABh/90h/92h/94h) | — | 15h | — | — | W25Q32RV-Q | ds p. 20 |
| Output driver strength default | — | 50 | — | Ω | DRV1,DRV0 = 1,0 | ds p. 17 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | –0.6 to 4.6 | V | ds p. 68 |
| Voltage on any pin VIO (relative to ground) | –0.6 to VCC + 0.4 | V | ds p. 68 |
| Transient voltage on any pin VIOT (< 20 ns) | –2.0 to VCC + 2.0 | V | ds p. 68 |
| Storage temperature TSTG | –65 to +150 | °C | ds p. 68 |
| Lead temperature | J-STD-20C (note 2) | — | ds p. 68 |
| ESD, human body model | –2000 to +2000 | V | ds p. 68 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply VCC (FR = 133 MHz, fR = 66 MHz) | 2.7 | 3.6 | V | ds p. 68 |
| Ambient temperature TA, Industrial Plus ("J") | –40 | +105 | °C | ds p. 68, p. 81 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CS | /CS | Input | Chip select, active low; must track VCC at power-up/down | ds p. 6, p. 7 |
| 2 | IO1 | DO (IO1) | I/O | Data output; IO1 in Dual/Quad/QPI | ds p. 6 |
| 3 | IO2 | /WP (IO2) | I/O | Write protect, active low; IO2 when QE = 1 (fixed on -JQ) | ds p. 6, p. 7 |
| 4 | GND | VSS | Ground | Ground | ds p. 6 |
| 5 | IO0 | DI (IO0) | I/O | Data input; IO0 in Dual/Quad/QPI | ds p. 6 |
| 6 | CLK | CLK | Input | Serial clock; SPI mode 0 or 3 | ds p. 6, p. 9 |
| 7 | IO3 | /HOLD or /RESET (IO3) | I/O | IO3 when QE = 1 (fixed on -JQ); /HOLD or /RESET only when QE = 0 | ds p. 6, p. 17 |
| 8 | VCC | VCC | Power | Supply | ds p. 6 |
| 9 | EP | (centre metal pad, unnumbered) | — | Not connected internally; leave floating or tie to GND | ds p. 78 |

## Typical application

- Connect CLK, /CS and IO0–IO3 to a Quad SPI host (ds p. 6, p. 10). The -JQ part has QE = 1 fixed,
  so Quad instructions (6Bh, EBh, 32h) need no QE write (ds p. 81).
- /CS: add a pull-up to VCC if the host cannot hold /CS high while VCC ramps (ds p. 7, p. 12).
- Without a Quad host, pads 3 and 7 still have no /WP or /HOLD function on this part (ds p. 16,
  p. 17); tie them to a defined level (inference; the source gives no value and no internal pull-up).
- Centre pad 9: leave floating or connect to GND (ds p. 78).
- Firmware: wait tVSL ≥ 20 µs after VCC(min) before the first instruction and tPUW ≥ 5 ms before
  any write/program/erase (ds p. 69); issue 06h before each program, erase or status write (ds p. 12).
- Power cycling: let VCC fall below 1.2 V for at least 100 µs before ramping again (ds p. 69, p. 70).

## Layout notes

- XH package (ds p. 78): body D = 2.00 mm x E = 3.00 mm, A ≤ 0.40 mm; terminals b = 0.25 mm wide,
  L = 0.45 mm long, e = 0.50 mm pitch, four along each **2 mm** edge; centre metal D2 = 1.60 mm
  (parallel to the terminal rows) x E2 = 0.20 mm; K = 0.95 mm from centre metal to terminal.
- Avoid exposed PCB vias under the centre pad (ds p. 78).
- Winbond gives no land pattern and no decoupling guidance in this datasheet.
- AC limits assume CL = 30 pF and 5 ns input edges (ds p. 72).

## Pitfalls

- 133 MHz (and 66 MHz for 03h) holds only when a read starts at a 4-byte aligned address; with
  arbitrary start addresses the limits are 104 MHz and 50 MHz (ds p. 73 notes 6, 7). The operating
  range table quotes only the 133/66 MHz case (ds p. 68).
- /HOLD and /RESET are not available: the "Q" option fixes QE = 1 (ds p. 81), and with QE = 1 the
  HOLD/RST bit has no effect (ds p. 17). Reset with 66h then 99h and wait tRST 30 µs (ds p. 67, p. 74).
- Check BUSY and SUS before a software reset; reset during program/erase can corrupt data (ds p. 67).
- QPI needs QE = 1 before 38h (ds p. 16); on this part that is always true.
- Page Program wraps within a 256-byte page; more than 256 bytes overwrite earlier data (ds p. 40).
- Erase is slow: 64 KB block up to 1.2 s, chip erase up to 40 s (ds p. 74); poll BUSY.
- Write and erase instructions must end on a byte boundary or they are ignored (ds p. 20).
- In power-down only ABh is accepted (ds p. 12).
- The array ships erased (all FFh) (ds p. 82).

## Not established by the sources

- **Footprint does not match the XH drawing.** In the Winbond package the terminals sit on the
  2 mm edges and the two terminal rows are 3 mm apart: E/2 = E2/2 + K + L = 0.10 + 0.95 + 0.45 =
  1.50 mm, so terminal centres are ±1.275 mm from the centre line (ds p. 78). The library land
  `qfn::QFN8N50P200X300_1EP61X220` is KiCad's generic DFN-8 2 x 3 mm, whose pad rows are at
  ±0.94 mm (0.87 x 0.25 mm pads spanning 0.505–1.375 mm). Each 1.05–1.50 mm terminal overlaps
  its land by only 0.325 mm and overhangs its outer end by 0.125 mm. The library 0.61 x 2.2 mm
  centre land is longer than the 2.0 mm body edge it runs along, against a 0.20 x 1.60 mm
  centre metal. The README and footprint comments flag the land as generic, but not that the
  row spacing differs. This needs a footprint review before fabrication.
- Pin 1 location: Figure 1b (ds p. 6) is a shared top view for XSON, WSON and USON; the XH drawing
  marks a "PIN 1 INDENT" on top and on the centre strip (ds p. 78) but numbers no terminals.
- Footprint citation: the footprint comment in `lib/qfn/src/footprints.cohdl` cites
  `w25q32rv rev.e 05152024 plus.pdf`, while the package README and this PDF are Revision E of
  2025-11-13 (ds p. 1, p. 83). The revision history lists no 2024-05-15 release (ds p. 83).
- Hardware /RESET on 8-pin parts: the feature list says a hardware /RESET pin exists only on SOP-16
  and TFBGA (ds p. 5), but p. 7 and p. 17 describe IO3 as /HOLD or /RESET on 8-pin packages when
  QE = 0, and p. 10 names "WSON-8 and TFBGA". Irrelevant for -JQ (QE = 1), but contradictory.
- "Fixed" QE: the ordering code says QE = 1 is fixed (ds p. 81); the QE description calls it the
  factory default and a non-volatile read/write bit (ds p. 16). Whether firmware can clear it on
  -JQ is not established.
- Whether /WP still gates Status Register writes on -JQ (QE = 1); p. 13 ties /WP to Standard/Dual
  SPI operation.
- Decoupling value, a land pattern, thermal resistance and maximum junction temperature.
- Retention/endurance conditions behind the feature-list numbers (ds p. 5).
- Top-side marking for XH is given as "VAywH / JQxxxx", but the note attached to it describes USON
  (ds p. 82).
- Availability: XH is a special-order package; contact Winbond (ds p. 82 note 1, p. 5 note 2).
