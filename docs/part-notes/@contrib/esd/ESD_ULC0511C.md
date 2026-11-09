# ULC0511C — single-line bidirectional ultra-low-capacitance TVS diode, 5 V, DFN1006DN

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/esd::ULC0511C",
  "parts": ["@contrib/esd::ESD_ULC0511C"],
  "identities": [
    {"mfr": "Tergy", "mpn": "ULC0511C", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "ULC0511C Ultra Low Capacitance Bidirectional TVS Diode", "revision": "Rev 01.06.2014",
     "url": "https://www.tergy.com", "sha256": "3eb2f4902e3aa429a39116e4d3257f8e13e645ddf93eea7790572afa507b7e4d", "local": "lib/@contrib/esd/docs/ulc0511c.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "N/3". The recorded URL is only the Tergy home page, which the PDF footer prints as `www.tergy.com` (ds p. 1). The package README calls the PDF a "manufacturer-supplied copy"; no document URL is recorded. `lcsc_map.json` has no row for this part.

## Summary

ULC0511C is a bidirectional TVS diode for one line in a 2-terminal DFN1006DN (about 1.0 × 0.6 mm) package (ds p. 1, p. 3). It is aimed at USB 2.0 power and data lines, SIM ports, DVI and 10/100 Ethernet (ds p. 1). Its point is 0.35 pF typical capacitance with a 5 V working voltage (ds p. 2). It is rated 90 W peak pulse power at 8/20 µs (ds p. 1), much less than a SOD-323 TVS such as GBLC05C. The datasheet contradicts itself on its ESD rating (see "Not established").

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Reverse working voltage VRWM | — | — | 5 | V | TA = 25 °C | ds p. 2 |
| Breakdown voltage VBR | 6 | — | — | V | IT = 1 mA; table does not say min/typ (see "Not established") | ds p. 2 |
| Clamping voltage VC | — | — | 15 | V | @ 1 A | ds p. 2 |
| Clamping voltage VC | — | — | 30 | V | @ 3 A | ds p. 2 |
| Reverse leakage IR | — | — | 1 | µA | at VRWM (condition not printed) | ds p. 2 |
| Capacitance C | — | 0.35 | — | pF | conditions not printed | ds p. 2 |
| IEC 61000-4-2 (Features list) | — | — | ±30 air / ±30 contact | kV | conflicts with ratings table | ds p. 1 |
| IEC 61000-4-4 EFT | — | — | 40 | A | 5/50 ns | ds p. 1 |
| IEC 61000-4-5 surge | — | — | 12 | A | 8/20 µs | ds p. 1 |
| Reel quantity | — | 10,000 | — | pcs | 7 inch reel | ds p. 1 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Peak pulse power PPP (8/20 µs) | 90 | W | ds p. 1 |
| ESD per IEC 61000-4-2, air | ±8 | kV | ds p. 1 |
| ESD per IEC 61000-4-2, contact | ±15 | kV | ds p. 1 |
| Storage temperature TSTJ | -55 to +150 | °C | ds p. 1 |

## Recommended operating conditions

The source has no recommended-operating-conditions table. The temperature range below is printed in the absolute maximum ratings table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature TJ | -55 | +150 | °C | ds p. 1 |
| Working voltage VRWM (Features: "Operating voltage: 5V") | — | 5 | V | ds p. 1, p. 2 |

Power derating: Fig. 3 derates rated power from 100 % at 25 °C to 0 at 150 °C ambient (ds p. 2).

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | 1 | passive | Terminal 1, at the notched end of the bottom view (PIN 1 ID) | ds p. 1, p. 3 |
| 2 | B | 2 | passive | Terminal 2 | ds p. 1 |

The pin configuration shows a symmetric bidirectional diode pair between pins 1 and 2 (ds p. 1), so either pad may go to the line.

## Typical application

- One pad to the protected line, the other to ground. The source shows no circuit and asks for no external parts (ds pp. 1–3).

## Layout notes

- Package (ds p. 3): E 0.95–1.05 mm long, D 0.55–0.65 mm wide, A 0.40–0.50 mm high. Terminals: b 0.40–0.60 mm wide, L 0.15–0.35 mm long, terminal pitch e 0.65 mm typical, L1 0.05 mm reference pull-back from the body end.
- Library `QFN2N65P60X100`: two 0.65 × 0.50 mm lands at y = ±0.325 mm. The pitch equals e (ds p. 3), and the land covers the maximum terminal width b (0.60 mm) and the terminal length L. The source gives no recommended land; the derivation is the library's own (`lib/@contrib/esd/docs/footprint-derivations.md`).
- The two lands are 0.15 mm apart (0.65 − 0.50). The source does not address that gap.

## Pitfalls

- Peak pulse power is only 90 W (ds p. 1). Do not use it where a surge (IEC 61000-4-5) above 12 A is expected (ds p. 1).
- The clamp at 3 A is 30 V maximum (ds p. 2), so the protected IC must survive a short 30 V transient.
- At 0.35 pF typical it suits USB 2.0 data lines, but the source gives no maximum capacitance (ds p. 2).

## Not established by the sources

- **ESD rating conflict:** Features say IEC 61000-4-2 air ±30 kV and contact ±30 kV (ds p. 1). The absolute-maximum table on the same page says air ±8 kV and contact ±15 kV (ds p. 1). The library comment repeats neither. Use ±8 kV air / ±15 kV contact unless Tergy confirms otherwise.
- **VBR qualifier:** the electrical table prints "VBR (V) 6" at IT = 1 mA without saying min, typ or max (ds p. 2).
- **Test conditions for C and IR:** the table gives C = 0.35 pF "Typ." and IR = 1 µA "Max" without bias or frequency (ds p. 2).
- **Recommended land pattern:** none in the source; see Layout notes.
- **Library comment "Protects USB 2.0 / SIM / Ethernet lines"** matches the application list (ds p. 1).
- **Document URL:** none; see the note under Identity.
