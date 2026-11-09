# ABM8-8.000MHZ-10-1-U-T — 8 MHz, 10 pF ceramic SMD crystal, 3.2 × 2.5 mm, 4 pads

## Identity

```json
{
  "schema": "part-note/1",
  "device": "osc::Crystal8MHz",
  "parts": ["osc::XTAL_8M"],
  "identities": [
    {"mfr": "Abracon", "mpn": "ABM8-8.000MHZ-10-1-U-T", "lcsc": null}
  ],
  "sources": [
    {"id": "fam", "title": "Abracon ABM8 Ceramic SMD Crystal family datasheet", "revision": "Revised 07-29-20",
     "url": "https://abracon.com/Resonators/abm8.pdf", "sha256": "564ac331d8bf38b4ae1222444b0d9cfea3a6ba29148a8be906b2e0ffe40471d8", "local": "lib/osc/docs/abracon-abm8.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. The two-page PDF prints no page numbers. `lcsc_map.json` has no row for this MPN.

## Summary

ABM8 is Abracon's seam-sealed 3.2 × 2.5 × 0.8 mm ceramic SMD crystal family (fam p. 1). The library part is the 8.000 MHz fundamental-mode configuration. Decoded from the p. 1 option table, it has 10 pF load capacitance, ±10 ppm tolerance at 25 °C, ±10 ppm stability, standard operating temperature (-10 °C to +60 °C) and 1k-piece tape and reel (fam p. 1). It is the HSE crystal for STM32-class MCUs in this library. Before choosing it, check the 400 Ω maximum ESR at 8 MHz against the oscillator's gain margin, and check the narrow -10 °C to +60 °C operating range (fam p. 1).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Nominal frequency | — | 8.000 | — | MHz | From the MPN frequency field | fam p. 1 |
| Family frequency range | 8.000 | — | 125.000 | MHz | Fundamental 8–54 MHz; "Contact Abracon for available F<10MHz" | fam p. 1 |
| Load capacitance CL | — | 10 | — | pF | MPN load-capacitance field "10" (family standard is 18.0 pF) | fam p. 1 |
| Frequency tolerance at +25 °C | -10 | — | +10 | ppm | MPN tolerance option "1" | fam p. 1 |
| Frequency stability over operating temperature (ref. +25 °C) | -10 | — | +10 | ppm | MPN stability option "U"; U is available only with temperature option E, B or standard -10 °C to +60 °C | fam p. 1 |
| ESR (R1) | — | — | 400 | Ω | 8.000–9.999 MHz fundamental, Table 1 | fam p. 1 |
| Shunt capacitance C0 | — | — | 3.0 | pF | — | fam p. 1 |
| Drive level | — | 10 | 100 | µW | — | fam p. 1 |
| Aging | -2 | — | +2 | ppm | First year, 25 °C ± 3 °C | fam p. 1 |
| Insulation resistance | 500 | — | — | MΩ | 100 Vdc ± 15 V | fam p. 1 |
| Package height | — | — | 0.80 | mm | Height option blank (ABM8) | fam p. 1, p. 2 |

## Absolute maximum ratings

The datasheet has no absolute maximum ratings table. The rows below are its storage range and reflow limits.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Storage temperature | -55 to +125 | °C | fam p. 1 |
| Reflow peak (TP) | 260 ± 5, 10 s max | °C | fam p. 2 |
| Reflow (TL) | 230, 30–40 s | °C | fam p. 2 |
| Preheat | 150–180, 60–120 s | °C | fam p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature (MPN has no temperature option, so the standard range applies) | -10 | +60 | °C | fam p. 1 |
| Drive level | — | 100 | µW | fam p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | XIN | (unnamed crystal terminal) | passive | Crystal electrode; top view lower left | fam p. 2 |
| 2 | GND | GND | passive | Case ground; top view lower right | fam p. 2 |
| 3 | XOUT | (unnamed crystal terminal) | passive | Crystal electrode; top view upper right | fam p. 2 |
| 4 | GND | GND | passive | Case ground; top view upper left | fam p. 2 |

The top view (fam p. 2) draws the crystal element between terminals 1 and 3 and labels 2 and 4 "GND". The bottom view is its mirror image. The crystal is symmetric; XIN and XOUT are library names.

## Typical application

- Pierce oscillator: pads 1 and 3 to the MCU's oscillator pins, each with a load capacitor to ground. Pads 2 and 4 to ground.
- The source gives only CL = 10 pF (fam p. 1). It does not give load-capacitor values, board stray capacitance or a drive-level resistor; derive them from the MCU's oscillator documentation.
- Keep the drive level at or below 100 µW (fam p. 1).

## Layout notes

- Recommended land (fam p. 2): four 1.3 × 1.05 mm lands with a 1.0 mm horizontal and a 0.7 mm vertical inner gap (reference dimensions in parentheses). `FP_Crystal_3225` matches: pads at x = ±1.15 mm, y = ±0.875 mm.
- Body 3.2 ± 0.20 × 2.5 ± 0.20 mm; terminals 0.9 ± 0.10 wide, 0.80 ± 0.10 tall, 1.2 ± 0.10 and 0.70 ± 0.10 apart (fam p. 2).
- Chamfer: "the chamfer could be located on pin #1, 3, or 4" and has no electrical effect (fam p. 2). Do not use the chamfer to orient the part. A 180° error is harmless (1 and 3 swap, 2 and 4 swap); a 90° error puts the crystal on the ground net.
- Hermetically sealed, MSL not applicable (fam p. 1).

## Pitfalls

- Operating range is only -10 °C to +60 °C (fam p. 1). The ±10 ppm "U" stability exists only for the standard, E or B temperature options (fam p. 1). For an industrial range a different MPN with a different stability code is needed.
- ESR can be as high as 400 Ω at 8 MHz (fam p. 1, Table 1). This is much higher than for crystals above 10 MHz (200 Ω at 10–12 MHz, 120 Ω at 12–16 MHz). Check the oscillator's critical transconductance against it.
- Do not reuse `FP_Crystal_3225` for a 2.0 × 1.6 mm crystal: `osc::XTAL_40MHZ` does this (see its note).

## Not established by the sources

- **Load-capacitor guidance in the library.** The part comment says "AN2867: C_load = 2*(CL - Cstray) = 2*(10 - 2.5) = 15pF". The pinned source gives only CL = 10 pF (fam p. 1). The formula, the 2.5 pF stray value and the 15 pF result come from ST AN2867, which is not a pinned source of this package. The source gives no stray value or external capacitor values.
- **Exact MPN string.** The source does not print `ABM8-8.000MHZ-10-1-U-T`. Its option diagram shows the fields (height, frequency, CL, custom ESR, then operating temperature, tolerance and stability in one group, then packaging) (fam p. 1). The decoding above (CL 10 pF, tolerance 1, stability U, packaging T, other fields blank) follows the field tables. The exact dash placement of the grouped fields is not shown in the source.
- **8 MHz availability.** The source notes "Contact Abracon for available F<10MHz" (fam p. 1). It does not confirm that this configuration is stocked.
- The device comment's claims about the STM32F072 (crystal-less USB, HSI48+CRS fallback) are outside this source.
- Pin 1 marker: the library's silkscreen dot at pad 1 shows nominal orientation. The source says the package chamfer may be at pin 1, 3 or 4 (fam p. 2), so the dot need not match a visible mark on the part. `docs/README.md` already says so.
- Gain margin, oscillator transconductance requirements, motional parameters (C1, L1) and frequency pulling sensitivity are not given.
