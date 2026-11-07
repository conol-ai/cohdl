# DMG1012T — 20 V N-channel enhancement MOSFET, ESD-protected gate, SOT523

## Identity

```json
{
  "schema": "part-note/1",
  "device": "mosfet::NMOS_SOT523",
  "parts": ["mosfet::FET_DMG1012T"],
  "identities": [
    {"mfr": "Diodes Incorporated", "mpn": "DMG1012T-7", "lcsc": "C20512"},
    {"mfr": "Diodes Incorporated", "mpn": "DMG1012T-13", "lcsc": "C3280264"}
  ],
  "sources": [
    {"id": "ds", "title": "DMG1012T N-Channel Enhancement Mode MOSFET", "revision": "DS31783 Rev. 8-2, February 2022",
     "url": "https://www.diodes.com/datasheet/download/DMG1012T.pdf", "sha256": "2f226aaee67a900773753a79c93d565820d01a300a4b1b7409fd07072da8aa40", "local": "lib/mosfet/docs/diodes-dmg1012t.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed "N of 7" page numbers.

## Summary

The DMG1012T is a 20 V, 0.63 A (at 25 °C, Note 6 mounting) N-channel MOSFET in SOT523. It has a low gate threshold (0.5–1.0 V) and an ESD gate-protection diode rated to 2 kV (ds p. 1, p. 2). It is specified with RDS(ON) at VGS = 1.8 V, 2.5 V and 4.5 V, so a 1.8 V to 3.3 V GPIO can drive it for low-side switching and level shifting (ds p. 2). The -7 and -13 suffixes differ only in reel quantity: 3000 and 10000 (ds p. 1). Do not use it where the gate can see more than ±6 V (ds p. 2), or above about 0.3 W dissipation (ds p. 2).

## Key parameters

All at TA = +25 °C unless noted (ds p. 2).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Drain-source breakdown BVDSS | 20 | — | — | V | VGS = 0 V, ID = 250 µA | ds p. 2 |
| Zero-gate-voltage drain current IDSS | — | — | 100 | nA | VDS = 20 V, VGS = 0 V, TJ = 25 °C | ds p. 2 |
| Gate-source leakage IGSS | — | — | ±1.0 | µA | VGS = ±4.5 V, VDS = 0 V | ds p. 2 |
| Gate threshold VGS(TH) | 0.5 | — | 1.0 | V | VDS = VGS, ID = 250 µA | ds p. 2 |
| RDS(ON) | — | 0.3 | 0.4 | Ω | VGS = 4.5 V, ID = 600 mA | ds p. 2 |
| RDS(ON) | — | 0.4 | 0.5 | Ω | VGS = 2.5 V, ID = 500 mA | ds p. 2 |
| RDS(ON) | — | 0.5 | 0.7 | Ω | VGS = 1.8 V, ID = 350 mA | ds p. 2 |
| Forward transfer admittance Yfs (magnitude) | — | 1.4 | — | S | VDS = 10 V, ID = 400 mA | ds p. 2 |
| Body-diode forward voltage VSD | — | 0.7 | 1.2 | V | VGS = 0 V, IS = 150 mA | ds p. 2 |
| Input capacitance Ciss | — | 60.67 | — | pF | VDS = 16 V, VGS = 0 V, f = 1.0 MHz | ds p. 2 |
| Output capacitance Coss | — | 9.68 | — | pF | VDS = 16 V, VGS = 0 V, f = 1.0 MHz | ds p. 2 |
| Reverse transfer capacitance Crss | — | 5.37 | — | pF | VDS = 16 V, VGS = 0 V, f = 1.0 MHz | ds p. 2 |
| Total gate charge Qg | — | 736.6 | — | pC | VGS = 4.5 V, VDS = 10 V, ID = 250 mA | ds p. 2 |
| Gate-source charge Qgs | — | 93.6 | — | pC | VGS = 4.5 V, VDS = 10 V, ID = 250 mA | ds p. 2 |
| Gate-drain charge Qgd | — | 116.6 | — | pC | VGS = 4.5 V, VDS = 10 V, ID = 250 mA | ds p. 2 |
| Turn-on delay tD(ON) | — | 5.1 | — | ns | VDD = 10 V, VGS = 4.5 V, RL = 47 Ω, RG = 10 Ω, ID = 200 mA | ds p. 2 |
| Rise time tR | — | 7.4 | — | ns | Same | ds p. 2 |
| Turn-off delay tD(OFF) | — | 26.7 | — | ns | Same | ds p. 2 |
| Fall time tF | — | 12.3 | — | ns | Same | ds p. 2 |
| Thermal resistance RθJA | — | 452 | — | °C/W | Single value; Note 6: FR-4, minimum recommended pad layout | ds p. 2 |

Off and on characteristics use a short pulse test (Note 7). The dynamic characteristics are guaranteed by design and not production tested (Note 8) (ds p. 2).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Drain-source voltage VDSS | 20 | V | ds p. 2 |
| Gate-source voltage VGSS | ±6 | V | ds p. 2 |
| Continuous drain current ID, steady state, TA = +25 °C (Note 6) | 0.63 | A | ds p. 2 |
| Continuous drain current ID, steady state, TA = +85 °C (Note 6) | 0.45 | A | ds p. 2 |
| Pulsed drain current IDM | 3 | A | ds p. 2 |
| Total power dissipation PD (Note 6) | 0.28 | W | ds p. 2 |
| ESD protection (gate) | 2 | kV | ds p. 1 |

## Recommended operating conditions

The datasheet has no recommended-operating-conditions table. The only operating row is the combined operating and storage range from the thermal characteristics table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating and storage temperature TJ, TSTG | -55 | +150 | °C | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | Gate | G | Input | Gate (ESD-protected) | ds p. 1 |
| 2 | Source | S | Passive | Source | ds p. 1 |
| 3 | Drain | D | Passive | Drain | ds p. 1 |

The source's top view (ds p. 1) shows G at the lower left, S at the lower right and D alone on the top edge. It does not print pin numbers. The library's footprint is a proper 90° rotation of that view, not a mirror: G (pad 1) and S (pad 2) are in the left column, and D (pad 3) is alone on the right.

## Typical application

- The source gives no application circuit and asks for no external parts (ds pp. 1–7).
- Low-side switch from a logic GPIO: RDS(ON) is specified at VGS = 1.8 V (≤ 0.7 Ω at 350 mA), 2.5 V (≤ 0.5 Ω at 500 mA) and 4.5 V (≤ 0.4 Ω at 600 mA) (ds p. 2).
- A gate pull-down sees up to ±1.0 µA of gate leakage at ±4.5 V (IGSS, ds p. 2) through the ESD diode. A very high-value pull-down can hold a measurable voltage.
- Thermal estimate: at 0.6 A and 0.4 Ω, P = 0.144 W, and the rise is about 65 °C at 452 °C/W (ds p. 2). The 0.28 W / 0.63 A ratings assume Note 6 mounting at 25 °C.

## Layout notes

- Suggested SOT523 pad layout (ds p. 6): X = 0.40 mm, Y = 0.51 mm, C = 1.29 mm (pad-row centre spacing), X1 = 0.70 mm (outer pad edge to package centre line), Y1 = 1.80 mm. That gives a 1.00 mm centre spacing between the two same-side pads.
- `SOT3P100X160X80N` is that layout rotated 90°: 0.51 × 0.40 mm pads, columns at x = ±0.645 mm and rows at y = ±0.5 mm. It matches the source.
- Package outline (ds p. 6): D 1.50–1.70 mm, E (over leads) 1.45–1.75 mm, E1 (body) 0.75–0.85 mm, A2 0.60–0.80 mm, b 0.15–0.30 mm, e1 0.90–1.10 mm. The library courtyard (2.3 × 2.2 mm) covers the maximum outline and the pads.
- MSL 1 per J-STD-020; matte tin finish (ds p. 1).

## Pitfalls

- VGSS is ±6 V (ds p. 2). Do not drive the gate from a 12 V or a 5 V-plus-overshoot source without a clamp. Fig. 1 plots a VGS = 8.0 V curve (ds p. 3), which is above the absolute maximum. Do not read it as permission.
- Note 5: the gate ESD diode only protects against ESD events; no VGS above VGSS may be applied (ds p. 1).
- The continuous ID falls from 0.63 A at 25 °C to 0.45 A at 85 °C. Both assume FR-4 with the minimum recommended pads (ds p. 2).
- The VGS = 1.8 V RDS(ON) figure is specified only at 350 mA (ds p. 2). At lower gate drive the part is not specified; the threshold can be as high as 1.0 V.
- Marking is "NA1" plus a date code, not the part number (ds p. 1).

## Not established by the sources

- **Pin numbers.** The library and `docs/README.md` ("Page 1 supplies ... pin 1 gate, pin 2 source, and pin 3 drain") assign G = 1, S = 2, D = 3. The source's p. 1 top view labels the terminals G, S and D by position only, with no numbers (ds p. 1). The numbering is the library's convention. The physical mapping is right because the footprint places each pad at the source's G, S and D positions (ds p. 1, p. 6).
- The library types Gate as `input` and Source and Drain as `passive`. The source states no direction; this is a library convention.
- No recommended operating conditions table (ds p. 2).
- RθJA and PD on any layout other than the "minimum recommended pad layout". No junction-to-case or junction-to-lead resistance is given (ds p. 2).
- Guaranteed RDS(ON) or VGS(TH) over temperature. Only typical curves are given (ds pp. 3–4, Fig. 5–7).
- The p. 6 outline and pad layout say "Please see http://www.diodes.com/package-outlines.html for the latest version". The pinned PDF may lag the current package drawing.
- Automotive (AEC-Q101) qualification: the -7 and -13 parts are "Commercial" (ds p. 1).
