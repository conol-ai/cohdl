# PMEG6010ELR — 60 V, 1 A low-leakage Schottky rectifier, CFP3 (SOD123W)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "diode::Schottky_SOD123",
  "parts": ["diode::SCHOTTKY_PMEG6010"],
  "identities": [
    {"mfr": "Nexperia", "mpn": "PMEG6010ELR", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "PMEG6010ELR 60 V, 1 A low leakage current Schottky barrier rectifier, product data sheet", "revision": "v.5, 1 January 2023, 13 pages",
     "url": "https://assets.nexperia.com/documents/data-sheet/PMEG6010ELR.pdf", "sha256": "c0eb3503d558ac894117e0dc520c33b0fe4e832f98602b6f56dfc4fc9538f112", "local": "lib/diode/docs/nexperia-pmeg6010elr-datasheet.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed "N / 13" page numbers on every page.

The URL is the one in `lib/diode/docs/README.md`. `lcsc_map.json` has no PMEG6010ELR row; the 2026-11-06 LCSC backfill review lists it as `no_candidate`. So `lcsc` is null.

## Summary

The PMEG6010ELR is a planar Schottky barrier rectifier with an integrated guard ring in the flat-lead CFP3 (SOD123W) package (ds p. 1). It is rated for 60 V reverse and 1 A average forward current, with 660 mV maximum forward drop at 1 A and 300 nA maximum leakage at 60 V and 25 °C (ds p. 1, p. 4). Choose it for low-voltage rectification, DC-DC catch or OR-ing, and reverse-polarity protection (ds p. 1). Its leakage rises to about 120 µA typical at 60 V and 125 °C (ds p. 4), and the source warns of thermal runaway from reverse losses (ds p. 3). The 1 A and power ratings depend strongly on mounting (ds p. 2, p. 3). The current data sheet is non-automotive; Nexperia points to a -Q product for automotive use (ds p. 11).

## Key parameters

All at Tj = 25 °C unless noted.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Reverse breakdown voltage V(BR)R | 60 | — | — | V | IR = 1 mA | ds p. 4 |
| Forward voltage VF | — | 475 | 540 | mV | IF = 0.1 A | ds p. 4 |
| Forward voltage VF | — | 550 | 605 | mV | IF = 0.5 A | ds p. 4 |
| Forward voltage VF | — | 575 | 625 | mV | IF = 0.7 A | ds p. 4 |
| Forward voltage VF | — | 605 | 660 | mV | IF = 1 A | ds p. 4 |
| Reverse current IR | — | 5 | — | nA | VR = 5 V, pulsed (tp ≤ 300 µs, δ ≤ 0.02) | ds p. 4 |
| Reverse current IR | — | 25 | 50 | nA | VR = 40 V, pulsed | ds p. 4 |
| Reverse current IR | — | 90 | 300 | nA | VR = 60 V, pulsed | ds p. 4 |
| Reverse current IR | — | 25 | — | µA | VR = 10 V, Tj = 125 °C, pulsed | ds p. 4 |
| Reverse current IR | — | 120 | — | µA | VR = 60 V, Tj = 125 °C, pulsed | ds p. 4 |
| Diode capacitance Cd | — | 110 | — | pF | VR = 1 V, f = 1 MHz | ds p. 4 |
| Diode capacitance Cd | — | 45 | — | pF | VR = 10 V, f = 1 MHz | ds p. 4 |
| Reverse recovery time trr | — | 4.5 | — | ns | IF = IR = 0.5 A, IR(meas) = 0.1 A | ds p. 4 |
| Peak forward recovery voltage VFRM | — | 580 | — | mV | IF = 0.5 A, dIF/dt = 20 A/µs | ds p. 4 |
| Rth(j-a), FR4, standard footprint | — | — | 220 | K/W | In free air | ds p. 3 |
| Rth(j-a), FR4, 1 cm² cathode pad | — | — | 130 | K/W | In free air | ds p. 3 |
| Rth(j-a), ceramic Al2O3, standard footprint | — | — | 70 | K/W | In free air | ds p. 3 |
| Rth(j-sp), junction to solder point | — | — | 18 | K/W | Solder point of cathode tab | ds p. 3 |

## Absolute maximum ratings

Nexperia's "Limiting values", per IEC 60134 (ds p. 2).

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Reverse voltage VR, Tj = 25 °C | 60 | V | ds p. 2 |
| Forward current IF, δ = 1, Tsp = 165 °C | 1.41 | A | ds p. 2 |
| Average forward current IF(AV), δ = 0.5, 20 kHz square wave, Tsp ≤ 170 °C | 1 | A | ds p. 2 |
| Average forward current IF(AV), same waveform, Tamb ≤ 140 °C, ceramic PCB | 1 | A | ds p. 2 |
| Non-repetitive peak forward current IFSM, 8.3 ms half sine, Tj(init) = 25 °C | 50 | A | ds p. 2 |
| Ptot, Tamb ≤ 25 °C, FR4 standard footprint | 680 | mW | ds p. 2 |
| Ptot, Tamb ≤ 25 °C, FR4 with 1 cm² cathode pad | 1.15 | W | ds p. 2 |
| Ptot, Tamb ≤ 25 °C, ceramic Al2O3 standard footprint | 2.14 | W | ds p. 2 |
| Junction temperature Tj | 175 | °C | ds p. 2 |
| Storage temperature Tstg | -65 to 175 | °C | ds p. 2 |

## Recommended operating conditions

The data sheet has no recommended-operating-conditions table. The only operating range is the ambient-temperature row of the limiting values.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Ambient temperature Tamb (from the limiting values) | -55 | 175 | °C | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | Cathode | K (cathode) | passive | Cathode; the marking bar is on this end | ds p. 1 |
| 2 | Anode | A (anode) | passive | Anode | ds p. 1 |

Both pads agree with Table 2 (ds p. 1). The package outline numbers the terminals and draws the marking band at the pin 1 end (ds p. 8).

## Typical application

- The source gives no application circuit and asks for no external parts (ds pp. 1–11).
- Listed uses: low-voltage rectification, high-efficiency DC-DC conversion, switch-mode supplies and reverse-polarity protection (ds p. 1).
- Forward drop budget: 475 mV typ at 0.1 A up to 660 mV max at 1 A (ds p. 4).
- Derating: Figs. 9–12 give typical IF(AV) versus Tamb or Tsp for each mounting (ds pp. 6–7). They are typical curves, not limits.

## Layout notes

- Reflow footprint (Fig. 17, ds p. 9): solder lands 1.2 × 1.2 mm on 2.8 mm centres; solder paste 1.1 × 1.1 mm; solder resist opening 1.4 × 1.4 mm; occupied area 4.4 mm overall; recommended stencil 0.1 mm thick.
- `FP_D_SOD_123W` has 1.2 × 1.2 mm pads at ±1.4 mm, matching the lands and centre spacing. Its 4.9 × 2.6 mm courtyard encloses the 4.4 × 2.1 mm occupied-area envelope.
- A separate wave-soldering footprint exists (Fig. 18, ds p. 10). The library models reflow only.
- Package (Fig. 16, ds p. 8): body length 2.4–2.8 mm, overall length 3.3–3.7 mm, width 1.5–1.9 mm, height 0.9–1.1 mm, terminal width 0.75–1.05 mm. Nominal body 2.6 × 1.7 × 1 mm (ds p. 2).
- Heat leaves through the cathode tab (Rth(j-sp) is to the cathode tab, ds p. 3). A larger cathode pad cuts Rth(j-a) from 220 to 130 K/W (ds p. 3).

## Pitfalls

- Polarity: pin 1 is the cathode and the marking bar marks it (ds p. 1). Do not reuse another vendor's SOD-123 pin numbering.
- The 1 A IF(AV) rating holds for a 50 % duty 20 kHz square wave with the solder point at or below 170 °C, or Tamb ≤ 140 °C only on a ceramic board (ds p. 2). On FR4 with a standard footprint, Ptot is only 680 mW (ds p. 2).
- Leakage is strongly temperature dependent: 90 nA typ at 60 V and 25 °C, 120 µA typ at 60 V and 125 °C (ds p. 4). The source warns that reverse losses can cause thermal runaway (ds p. 3).
- Capacitance is 110 pF typ at 1 V (ds p. 4). Account for it in fast switching or signal paths.
- The IFSM condition changed in v.5 from a square wave to an 8.3 ms half sine (ds p. 11). Older copies of the data sheet quote a different surge condition.

## Not established by the sources

Disagreements between the library and the source:

- **Paste and mask apertures.** Fig. 17 asks for 1.1 × 1.1 mm paste and a 1.4 × 1.4 mm solder-resist opening on the 1.2 × 1.2 mm lands (ds p. 9). `P_SOD123W_Term` sets no `paste:` override, so the paste aperture is whatever the default is for a 1.2 mm pad, not the 1.1 mm the source asks for.
- **Device name.** The device is `Schottky_SOD123`, but the package is SOD123W / CFP3 (ds p. 1, p. 2), a different outline from SOD-123. The comment, footprint name and land are correct for SOD123W; only the device name is misleading.
- **Footprint comment.** It says Figure 17 "declares a 4.4 x 2.1mm occupied area". The occupied area in Fig. 17 is not a rectangle: 4.4 mm long at the land rows (1.6 mm tall) and 2.1 mm tall only over the 2.9 mm centre (ds p. 9). The courtyard drawn round the 4.4 × 2.1 mm bounding box is still conservative.
- **LCSC.** No LCSC part number is recorded, so the part cannot be bound to a JLC/LCSC stock item.

Pin numbering, anode/cathode assignment and pad positions agree with the source (ds p. 1, p. 8, p. 9).

Not stated by the source:

- Any ESD rating, moisture sensitivity level or reflow profile.
- Limits (min/max) over temperature for leakage, capacitance or recovery. The 125 °C leakage values are typical only (ds p. 4).
- A recommended operating range for reverse voltage or current separate from the limiting values.
- Rth(j-a) for the exact library land (1.2 × 1.2 mm on FR4). The "standard footprint" of Note [2] is not dimensioned (ds p. 2, p. 3).
