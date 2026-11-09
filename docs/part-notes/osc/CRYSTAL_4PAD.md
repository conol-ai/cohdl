# Crystal_4pad — ABM8-272-T3 (12 MHz, 3.2 × 2.5 mm) and Hosonic E1SB48E001G00E (48 MHz, 2.0 × 1.6 mm)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "osc::Crystal_4pad",
  "parts": ["osc::XTAL_12MHZ", "osc::XTAL_48MHZ"],
  "identities": [
    {"mfr": "Abracon", "mpn": "ABM8-272-T3", "lcsc": "C20625731"},
    {"mfr": "Hosonic", "mpn": "E1SB48E001G00E", "lcsc": null}
  ],
  "sources": [
    {"id": "scd", "title": "Abracon ABM8-272-T3 Source Control Drawing # 456603", "revision": "Revision B, issued 2024-09-16 (local file holds drawing pages 2, 4 and 6 of 9)",
     "url": "https://abracon.com/datasheets/ABM8-272-T3.pdf", "sha256": "aead22b6bd9d6f8ad4472352f70fce3ade633e90b9772ba80fabe8fd0856ae91", "local": "lib/osc/docs/abracon-abm8-272-t3.pdf"},
    {"id": "fam", "title": "Abracon ABM8 Ceramic SMD Crystal family datasheet", "revision": "Revised 07-29-20",
     "url": "https://abracon.com/Resonators/abm8.pdf", "sha256": "564ac331d8bf38b4ae1222444b0d9cfea3a6ba29148a8be906b2e0ffe40471d8", "local": "lib/osc/docs/abracon-abm8.pdf"},
    {"id": "hos", "title": "Hosonic SMD Seam Sealing Crystals, E1SB Series 2.0x1.6x0.50mm", "revision": "No revision printed; PDF modification date 2026-07-20",
     "url": "https://www.hosonic.com/Upload/FTP/P-C-S-E1SB.pdf", "sha256": "af74ec52f0d175cd03ff64cd71a926d839fe79b023a21a3e2c2b6bceb7b7e62b", "local": "lib/osc/docs/hosonic-e1sb.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices:
- `scd` has three PDF pages. They carry the drawing's own page numbers (2), (4) and (6) of (9). So scd p. 1 = drawing page 2, p. 2 = page 4, p. 3 = page 6.
- `fam` has two unnumbered pages.
- `hos` is a single image-only page with no extractable text; it was read from a rendering.

## Summary

`osc::Crystal_4pad` is a four-terminal crystal: two crystal electrodes on the 1–3 diagonal and two grounded case terminals on 2 and 4. Two parts use it, in different packages and footprints:
- `XTAL_12MHZ` is Abracon ABM8-272-T3: 12.000 MHz fundamental AT-cut, 10 pF load, ±30 ppm tolerance and ±30 ppm stability over -40 °C to +85 °C, 50 Ω maximum ESR, 3.2 × 2.5 × 0.8 mm (scd p. 1). Abracon states it is approved for Raspberry Pi RP2040 and RP235x microcontrollers (scd p. 1).
- `XTAL_48MHZ` is Hosonic E1SB48E001G00E, a 2.0 × 1.6 mm E1SB-series crystal (hos p. 1). The pinned series sheet gives only series-wide limits. It does not establish the part-specific load capacitance, tolerance or ESR that the library quotes; see "Not established by the sources".

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| ABM8-272-T3 centre frequency | — | 12.000 | — | MHz | Fundamental AT | scd p. 1 |
| ABM8-272-T3 frequency tolerance | -30 | — | 30 | ppm | at +25 °C | scd p. 1 |
| ABM8-272-T3 frequency stability | -30 | — | 30 | ppm | over -40 °C to +85 °C, ref. +25 °C | scd p. 1 |
| ABM8-272-T3 ESR (R1) | — | — | 50 | Ω | — | scd p. 1 |
| ABM8-272-T3 shunt capacitance C0 | — | — | 3.0 | pF | — | scd p. 1 |
| ABM8-272-T3 load capacitance CL | — | 10 | — | pF | — | scd p. 1 |
| ABM8-272-T3 drive level | — | 10 | 200 | µW | — | scd p. 1 |
| ABM8-272-T3 aging | -5 | — | 5 | ppm | 25 ± 3 °C, first year | scd p. 1 |
| ABM8-272-T3 insulation resistance | 500 | — | — | MΩ | 100 Vdc ± 15 V | scd p. 1 |
| E1SB series frequency range | 16 | — | 96 | MHz | Fundamental | hos p. 1 |
| E1SB series load capacitance CL | 6 | — | 16 | pF | "6 ~ 16pF, or specify" | hos p. 1 |
| E1SB series frequency tolerance | -10 | — | 10 | ppm | at 25 °C; "±10ppm, or specify" | hos p. 1 |
| E1SB series temperature stability | -15 | — | 15 | ppm | ref. 25 °C; "±15ppm, or specify" | hos p. 1 |
| E1SB series ESR | — | — | 60 | Ω | 30.0–74.9 MHz band (covers 48 MHz) | hos p. 1 |
| E1SB series shunt capacitance C0 | — | — | 2 | pF | — | hos p. 1 |
| E1SB series drive level | 1 | 50 | 100 | µW | — | hos p. 1 |
| E1SB series aging | -3 | — | 3 | ppm | per year, at 25 °C | hos p. 1 |
| E1SB series insulation resistance | 500 | — | — | MΩ | at DC 100 V | hos p. 1 |

## Absolute maximum ratings

Neither source has an absolute maximum ratings table. The rows below are storage ranges and reflow limits. The ABM8-272-T3 reflow page is not in the local SCD, so the ABM8 family profile is shown.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| ABM8-272-T3 storage temperature | -55 to 125 | °C | scd p. 1 |
| ABM8 family reflow peak TP | 260 ± 5, 10 s max | °C | fam p. 2 |
| E1SB series storage temperature | -40 to +85 | °C | hos p. 1 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| ABM8-272-T3 operating temperature | -40 | 85 | °C | scd p. 1 |
| ABM8-272-T3 drive level | — | 200 | µW | scd p. 1 |
| E1SB series operating temperature ("-30 ~ +85°C, or specify") | -30 | +85 | °C | hos p. 1 |
| E1SB series drive level | 1 | 100 | µW | hos p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | XIN | ABM8-272-T3: unnamed crystal terminal; E1SB: "Crystal" | passive | Crystal electrode; top view lower left (both packages) | scd p. 2, hos p. 1 |
| 2 | GND | GND | passive | Case / lid ground; top view lower right | scd p. 2, hos p. 1 |
| 3 | XOUT | ABM8-272-T3: unnamed crystal terminal; E1SB: "Crystal" | passive | Crystal electrode; top view upper right | scd p. 2, hos p. 1 |
| 4 | GND | GND | passive | Case / lid ground; top view upper left | scd p. 2, hos p. 1 |

Hosonic's pin table gives 1 and 3 as Crystal and 2 and 4 as GND, with the note "#2 & #4 are connected to Lid" (hos p. 1). Both libraries' footprints put pad 1 at the lower left in top view (y down) and agree with these drawings. XIN and XOUT are library names for a symmetric crystal.

## Typical application

- Pierce oscillator: pads 1 and 3 to the oscillator pins, each with a load capacitor to ground. Pads 2 and 4 to ground.
- ABM8-272-T3: CL = 10 pF (scd p. 1). The source gives no external capacitor values or stray capacitance.
- E1SB48E001G00E: the pinned sheet gives only the series CL range, 6–16 pF or as specified (hos p. 1). Take the exact CL from a part-specific document before choosing load capacitors.
- Keep drive level at or below 200 µW for ABM8-272-T3 (scd p. 1) and 100 µW for E1SB (hos p. 1).

## Layout notes

- XTAL_12MHZ, `FP_Crystal_SMD_3225_4Pin_3_2x2_5mm`: the local SCD pages show only the package (3.2 ± 0.20 × 2.5 ± 0.20 mm; terminals 0.9 ± 0.10 wide, 0.80 ± 0.10 tall) and no land (scd p. 2). The footprint uses the ABM8 family land, 1.3 × 1.05 mm with 1.0 mm and 0.7 mm inner gaps (fam p. 2). It matches that land.
- XTAL_48MHZ, `FP_Crystal_SMD_2016_4Pin_E1SB`: recommended pad layout 0.85 × 0.75 mm, 0.50 mm horizontal and 0.30 mm vertical gaps (hos p. 1). The footprint matches: pads at ±0.675, ±0.525 mm. Body 2.00 ± 0.10 × 1.60 ± 0.10 mm; the side view gives 0.45 ± 0.1 mm height (hos p. 1).
- ABM8 chamfer: "could be located on pin #1, 3, or 4" (scd p. 2, fam p. 2). Do not orient by it.
- ABM8-272-T3 ships on 3,000-piece reels (scd p. 3).

## Pitfalls

- A 90° rotation puts the crystal on the ground pair. A 180° rotation is harmless. Neither source gives a reliable visible pin-1 index (scd p. 2, hos p. 1).
- The two parts are not interchangeable on a board: same pins, different packages and lands (scd p. 2, hos p. 1).
- E1SB's series operating range is -30 °C to +85 °C unless specified (hos p. 1). That is narrower than ABM8-272-T3's -40 °C to +85 °C (scd p. 1).
- E1SB storage range is only -40 °C to +85 °C (hos p. 1).

## Not established by the sources

- **E1SB48E001G00E specifics.** The library comment says the part "guarantees 8.8pF load, -6/+8ppm frequency tolerance, and 22ohm maximum ESR". `docs/README.md` attributes these figures to SiFli's SF32LB52X hardware guide, which is not a pinned source. The Hosonic sheet has no ordering-code key, so it does not decode `E1SB48E001G00E` or confirm that the MPN exists. It gives ESR 60 Ω max for 30.0–74.9 MHz (hos p. 1), which exceeds the "<=30ohm" MCU limit the comment cites. On the pinned evidence, whether the part meets that limit is not established.
- **Incomplete SCD.** The local `abracon-abm8-272-t3.pdf` holds only drawing pages 2, 4 and 6 of 9 (scd pp. 1–3). The "See Table 2" height reference on the mechanical page (scd p. 2) has no Table 2 in the local file. Any land pattern, reflow profile or test conditions on the missing pages cannot be checked. The footprint therefore relies on the ABM8 family land (fam p. 2), as `docs/README.md` says.
- MSL wording differs. The SCD says "Hermetically Sealed and not Moisture Sensitive - MSL = 1" (scd p. 1). The family sheet says "MSL = N/A" (fam p. 1).
- The Hosonic header says 0.50 mm thick; its side view gives 0.45 ± 0.1 mm (hos p. 1).
- **E1SB orientation mark.** In both the top view and the unlabelled terminal-side view, the terminal at the pin 2 position is drawn with a cut corner (hos p. 1). The sheet does not say whether that is an index mark. The library's pin-1 dot is at pad 1. An assembler who reads the cut corner as pin 1 would rotate the part 90°.
- The Hosonic sheet carries no revision or date. The 2026-07-20 date in `docs/README.md` is the PDF modification date.
- Neither source gives load-capacitor values, motional parameters or oscillator gain-margin data.
