# GBLC05C-LF-T7 — single-line bidirectional low-capacitance TVS array, 5 V stand-off, SOD-323

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/esd::GBLC05C",
  "parts": ["@contrib/esd::ESD_GBLC05C"],
  "identities": [
    {"mfr": "ProTek", "mpn": "GBLC05C-LF-T7", "lcsc": "C85355"}
  ],
  "sources": [
    {"id": "ds", "title": "GBLC03 - GBLC24C Ultra Low Capacitance TVS Array", "revision": "05126.R22, 6/19",
     "url": "https://www.protekdevices.com", "sha256": "dbeb3a585f49a0b376d9ebf243c0caa6123dfe8c46f380995146252986c7ff97", "local": "lib/@contrib/esd/docs/gblc05c.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "Page N" (1–8). The recorded URL is only the ProTek home page, which the PDF prints as `www.protekdevices.com` (ds p. 8). The package README calls the PDF a "manufacturer-supplied copy" and gives no document link, so no document URL is recorded.

## Summary

GBLC05C is the 5 V, bidirectional member of ProTek's GBLC03–GBLC24C family: one TVS array in a SOD-323 package that protects one power or I/O line (ds p. 1, p. 2). The "C" suffix means bidirectional (ds p. 2). It is rated 350 W peak pulse power at 8/20 µs, with 3 pF typical capacitance at 0 V and 9.8 V maximum clamping at 1 A (ds p. 1, p. 2). Choose it for a single line that idles between about −5 V and +5 V and needs IEC 61000-4-2 protection. For USB 2.0 high-speed pairs a lower-capacitance part may be needed: the source gives only a typical 3 pF and no maximum (ds p. 2).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Rated stand-off voltage VWM | — | 5.0 | — | V | GBLC05C row, per line, 25 °C | ds p. 2 |
| Breakdown voltage V(BR) | 6.0 | — | — | V | @ 1 mA, 25 °C | ds p. 2 |
| Clamping voltage VC | — | — | 9.8 | V | @ IP = 1 A (8/20 µs, Fig. 2) | ds p. 2 |
| Leakage current ID | — | — | 5 | µA | @ VWM, 25 °C | ds p. 2 |
| Capacitance C | — | 3 | — | pF | @ 0 V, 1 MHz | ds p. 2 |
| Marking code | — | 5C | — | — | GBLC05C | ds p. 2 |
| IEC 61000-4-2 (Features) | — | — | ±15 air / ±8 contact | kV | "Exceeds Level 4: handles 20 kV contact & 25 kV air" | ds p. 1 |
| IEC 61000-4-4 EFT | — | — | 40 | A | 5/50 ns | ds p. 1 |
| ISO 10605 ESD | — | — | ±20 | kV | contact, 330 pF / 2 kΩ | ds p. 1 |
| Reel quantity | — | 3000 | — | pcs | suffix -T7, 7" reel, 8 mm tape | ds p. 7 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Peak pulse power PPP (tp = 8/20 µs) | 350 | W | ds p. 2 |
| Storage temperature TSTG | -55 to 150 | °C | ds p. 2 |

The Mechanical Characteristics list gives "Solder Reflow Temperature: Pure-Tin - Sn, 100: 260-270°C" (ds p. 1). It does not say whether that is a peak limit or a process window, so it is not listed as a rating here.

## Recommended operating conditions

The source has no recommended-operating-conditions table. The operating temperature below is printed in "Maximum ratings".

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature TA | -55 | 150 | °C | ds p. 2 |
| Working voltage (stand-off VWM, either polarity for the bidirectional part) | — | 5.0 | V | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | 1 | passive | One terminal of the bidirectional array; line or ground | ds p. 1 |
| 2 | B | 2 | passive | Other terminal; ground or line | ds p. 1 |

The bidirectional pin configuration (ds p. 1) is symmetric, so either pad may go to the line. The source prints a polarity band only on unidirectional parts (ds p. 7), so a GBLC05C body carries no pin-1 mark.

## Typical application

- Connect one pad to the protected line and the other to ground. The source shows no application circuit; it lists Ethernet 10/100/1000 Base-T, smart phones, handheld wireless and USB interfaces (ds p. 1).
- No external parts are asked for.

## Layout notes

- Recommended SOD-323 pad layout (ds p. 6): overall span A 2.87–3.12 mm, pad width B 0.66–0.91 mm, pad height C 0.66–0.91 mm.
- Library `FP_SOD323_2P5X1P25mm`: 0.80 × 0.80 mm pads at x = ±1.10 mm, so the overall span is 3.00 mm. Both values are inside the ranges on ds p. 6.
- Package outline (ds p. 6): body A 1.60–1.90 mm long, B 1.15–1.45 mm wide; lead-tip span C 2.39–2.70 mm; lead width E 0.25–0.40 mm; height D 0.80–1.10 mm.
- Place it close to the connector; the source gives no layout advice beyond the land.

## Pitfalls

- Do not order the unidirectional GBLC05: it has the same SOD-323 package and marking "5" instead of "5C" (ds p. 2), and its clamp is asymmetric (ds p. 1, p. 5).
- Capacitance is typical only (3 pF at 0 V) and falls with bias (Fig. 3, ds p. 4). There is no maximum to budget against.
- Leakage rises with junction temperature (Fig. 4, typical, ds p. 4); the 5 µA limit is at 25 °C only (ds p. 2).

## Not established by the sources

- **Contact-discharge level:** the Features list says IEC 61000-4-2 contact ±8 kV and also "handles 20 kV contact & 25 kV air" (ds p. 1). Neither is in the ratings table on ds p. 2.
- **Peak pulse current:** no IPP rating. VC is given only at 1 A (ds p. 2). PPP = 350 W is the only surge rating (ds p. 2).
- **Maximum capacitance and capacitance at the working voltage:** not given (ds p. 2, p. 4).
- **Library "TwoTerminal" pin names:** the library names the pins `A` and `B` with comments "I/O line" and "I/O line / ground reference". The source numbers them 1 and 2 only (ds p. 1); that is consistent, not a conflict.
- **Ordering code:** the library MPN `GBLC05C-LF-T7` follows the ordering scheme (base + `-LF` + `-T7`, ds p. 7). The source does not print that full string.
- **Document URL:** none; see the note under Identity.
