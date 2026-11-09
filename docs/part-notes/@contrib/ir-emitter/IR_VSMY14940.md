# VSMY14940 — 940 nm high-speed IR emitting diode, side-view SMD 3.0 × 2.51 × 1.2 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/ir-emitter::VSMY14940",
  "parts": ["@contrib/ir-emitter::IR_VSMY14940"],
  "identities": [
    {"mfr": "Vishay", "mpn": "VSMY14940", "lcsc": "C511069"}
  ],
  "sources": [
    {"id": "ds", "title": "VSMY14940 High Speed Infrared Emitting Diodes, 940 nm, Surface Emitter Technology", "revision": "Document 84209, Rev. 1.6, 22-Sep-17, 7 pages",
     "url": "https://www.vishay.com", "sha256": "359c31c84d30efe8df1cef5caf0fa43c14ddb3f52dc8dd3811892d17b5f039eb", "local": "lib/@contrib/ir-emitter/docs/vsmy14940.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices; they equal the printed page numbers on pp. 1–6. Page 7 is Vishay's legal disclaimer (document 91000).

The source URL is only the manufacturer's home page. Neither `lib/@contrib/ir-emitter/docs/README.md` nor the PDF gives a document URL; the PDF names only document number 84209.

## Summary

The VSMY14940 is a GaAlAs surface-emitter infrared LED at 940 nm in a clear side-looking SMD package with a lens, 3.0 × 2.51 × 1.2 mm (ds p. 1). It is meant for remote controls (38 kHz), learning remotes, photo-interrupters and optical switches (ds p. 1). The beam is narrow, ±9° half-intensity angle, with 25 mW/sr typical at 20 mA and 90 mW/sr at 70 mA (ds p. 2). It switches fast (5 ns rise, 6 ns fall) and takes 0.7 A pulses of 100 µs (ds p. 1, p. 2). It is not designed for reverse operation (ds p. 2).

## Key parameters

Tamb = 25 °C unless stated (ds p. 2).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Forward voltage VF | 1.1 | 1.3 | 1.5 | V | IF = 20 mA, tp = 20 ms | ds p. 2 |
| Forward voltage VF | — | 1.5 | — | V | IF = 70 mA, tp = 20 ms | ds p. 2 |
| Forward voltage VF | — | 3.0 | — | V | IF = 0.7 A, tp = 100 µs | ds p. 2 |
| VF temperature coefficient | — | −0.9 | — | mV/K | IF = 20 mA | ds p. 2 |
| Junction capacitance CJ | — | 30 | — | pF | VR = 0 V, f = 1 MHz | ds p. 2 |
| Radiant intensity Ie | 15 | 25 | — | mW/sr | IF = 20 mA, tp = 20 ms | ds p. 2 |
| Radiant intensity Ie | — | 90 | — | mW/sr | IF = 70 mA, tp = 20 ms | ds p. 2 |
| Radiant intensity Ie | — | 560 | — | mW/sr | IF = 0.7 A, tp = 100 µs | ds p. 2 |
| Radiant power φe | — | 40 | — | mW | IF = 70 mA, tp = 20 ms | ds p. 2 |
| Radiant power temperature coefficient | — | −0.21 | — | %/K | IF = 70 mA | ds p. 2 |
| Angle of half intensity | — | ±9 | — | deg | — | ds p. 2 |
| Peak wavelength λp | 920 | 940 | 960 | nm | IF = 70 mA | ds p. 2 |
| Spectral bandwidth Δλ | — | 55 | — | nm | IF = 70 mA | ds p. 2 |
| λp temperature coefficient | — | 0.28 | — | nm/K | IF = 70 mA | ds p. 2 |
| Rise time tr | — | 5 | — | ns | IF = 70 mA, 10 % to 90 % | ds p. 2 |
| Fall time tf | — | 6 | — | ns | IF = 70 mA, 10 % to 90 % | ds p. 2 |
| Reverse light current Ira | — | 0.5 | — | µA | Ee = 1 mW/cm², λ = 950 nm, VR = 5 V | ds p. 2 |

## Absolute maximum ratings

Tamb = 25 °C unless stated (ds p. 1).

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Reverse voltage VR | 5 | V | ds p. 1 |
| Forward current IF | 70 | mA | ds p. 1 |
| Surge forward current IFSM, tp = 100 µs | 0.7 | A | ds p. 1 |
| Power dissipation PV | 119 | mW | ds p. 1 |
| Junction temperature Tj | 100 | °C | ds p. 1 |
| Storage temperature | −55 to +100 | °C | ds p. 1 |
| Soldering temperature (J-STD-020, Fig. 10) | 260 | °C | ds p. 1, p. 4 |
| Thermal resistance RthJA (J-STD-051, soldered on PCB) | 390 | K/W | ds p. 1 |

## Recommended operating conditions

The datasheet has no recommended-operating-conditions table. Vishay lists the operating temperature range among the absolute maximum ratings.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating ambient temperature | −40 | +85 | °C | ds p. 1 |
| Forward current, derated with ambient (RthJA = 390 K/W) | — | 70 | mA | ds p. 2 (Fig. 2) |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | Cathode | Cathode (−) | passive | Cathode terminal, marked by the cathode mask | ds p. 5 |
| 2 | Anode | Anode (+) | passive | Anode terminal | ds p. 5 |

The source marks polarity on the part (a "cathode mask" stripe beside one terminal, ds p. 5; "Anode mask" and "Cathode mask" in the tape drawing, ds p. 6) and with a diode symbol (ds p. 5). Its recommended solder pad drawing does not say which land is the cathode (ds p. 5). The library's pad-number-to-terminal assignment is its own convention ("repository diode convention") and cannot be checked against the land drawing; see "Not established".

## Typical application

- Drive with a series resistor or current source; the source gives no circuit. At 20 mA VF is 1.1–1.5 V (ds p. 2).
- Remote-control use at 38 kHz is named (ds p. 1). Pulse current up to 0.7 A for tp = 100 µs (ds p. 1); the source does not state a duty-cycle limit for pulses.
- Derate continuous IF and PV with ambient per Fig. 1 and Fig. 2 (ds p. 2).
- Do not apply reverse bias in use; IR is "not designed for reverse operation" (ds p. 2), although VR = 5 V is the absolute rating (ds p. 1).

## Layout notes

- Recommended solder pad (ds p. 5): two lands 0.90 mm wide × 1.40 mm, 1.80 mm gap between them, so 3.60 mm overall. `FP_VSMY14940_3P0X2P51mm` uses 0.9 × 1.4 mm pads at ±1.35 mm: inner gap 1.80 mm, overall 3.60 mm. It matches.
- The lens points sideways, parallel to the board, past one long edge of the land pair (ds p. 5, recommended solder pad drawing). Keep that edge clear of tall parts and the board edge free in the emission direction.
- Package (ds p. 5): length 3.00 mm, width including the lens 2.51 mm, height 1.20 mm, lens 1.60 mm across; tolerances ±0.1 mm where not shown.
- Reflow per J-STD-020 lead-free profile, peak 255–260 °C for ≤ 10 s (ds p. 4). MSL 3, floor life 168 h at < 30 °C / 60 % RH; bake 192 h at 40 °C, < 5 % RH if exceeded (ds p. 1, p. 4).

## Pitfalls

- Polarity: the cathode is the terminal under the cathode mask (ds p. 5). Check the library's pad 1 against the part's mask, not against the land drawing, which carries no polarity.
- 70 mA is the absolute maximum continuous current at 25 °C and must be derated with ambient (ds p. 1, p. 2). Design for less.
- RthJA = 390 K/W (ds p. 1): at 70 mA × 1.5 V ≈ 105 mW the junction rises about 41 K above ambient, close to the 100 °C Tj limit at 60 °C ambient.
- Junction temperature limit is only 100 °C and storage only to +100 °C (ds p. 1).
- Moisture sensitivity MSL 3 (ds p. 1, p. 4).

## Not established by the sources

Disagreements and gaps between the library and the source:

- **Which land is the cathode.** The recommended solder pad drawing (ds p. 5) shows two identical lands and the lens, without a polarity mark. The library makes pad 1 (at −1.35 mm) the cathode. Whether that pad sits under the cathode terminal for a given lens direction is not stated by the source, and the library footprint does not mark the lens (emission) side at all, so the footprint does not fix which way the light goes.
- **Courtyard contains the body.** The footprint comment says the 4.1 × 3.1 mm courtyard contains "3.00 x 2.51 mm body". The courtyard is centred on the pad row (±1.55 mm), while the land drawing places the lens on one side, outside the pad row (ds p. 5). The source gives no dimension from the lands to the body or lens. A 2.51 mm depth fits inside ±1.55 mm only if the body is centred within 0.295 mm of the land centreline; the drawing shows the lens well beyond the lands, so the claim is not established and probably false.
- **Pad naming.** "Cathode: 1, Anode: 2" is a repository convention; the source numbers no terminals.
- **Recommended operating conditions.** The source has none apart from the operating temperature in the absolute-maximum table (ds p. 1).
- Not given: a pulse duty-cycle rating for IFSM, minimum radiant intensity at 70 mA, limits over temperature, and a document URL.
