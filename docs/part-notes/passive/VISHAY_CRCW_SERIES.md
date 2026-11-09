# Vishay CRCW e3 series: thick-film chip resistors, 0201 to 2512 (library alternate)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "passive::ChipResistor",
  "parts_pattern": "R_.*",
  "variants": ["R0201", "R0402", "R0603", "R0805", "R1206", "R1210", "R2010", "R2512"],
  "identities": [
    {"mfr": "Vishay", "mpn_pattern": "CRCW(0201|0402|0603|0805|1206|1210|2010|2512)(\\d{3}[RKM]|\\d{2}[RKM]\\d|\\d[RKM]\\d{2})(FKED|JNED)", "lcsc": null}
  ],
  "sources": [
    {"id": "crcw0201", "title": "CRCW0201 e3, Lead (Pb)-Free Commodity Thick Film Chip Resistors", "revision": "Document 20052, revision 21-Sep-2022",
     "url": "https://www.vishay.com/docs/20052/crcw0201e3.pdf", "sha256": "8b0c7e26a78b492136705ca23acdcbda247e7732037a365384ea88b29c953d5c", "local": "lib/passive/docs/vishay_crcw0201_e3_datasheet.pdf"},
    {"id": "dcrcw", "title": "D/CRCW e3, Standard Thick Film Chip Resistors", "revision": "Document 20035, revision 14-Apr-2026",
     "url": "https://www.vishay.com/docs/20035/dcrcwe3.pdf", "sha256": "efd5a87e03985e46a025634b66230116a4b2ca1081adf4236332253d841b00bd", "local": "lib/passive/docs/vishay_d_crcw_e3_datasheet.pdf"}
  ],
  "status": "draft"
}
```

## Summary

CRCW e3 is Vishay's thick-film chip resistor family with pure tin on nickel terminations (crcw0201 p. 1; dcrcw p. 4). The 0201 size is in a separate "commodity" datasheet; 0402 to 2512 are D/CRCW e3, which is AEC-Q200 qualified (crcw0201 p. 1; dcrcw p. 1). In the library it is only the alternate entry of each generated `R_<value>_<F|J>_<size>` part. The Yageo RC_L primary is described in `YAGEO_RC_SERIES.md`. The library uses `…FKED` for ±1% / ±100 ppm/K and `…JNED` for ±5% / ±200 ppm/K, from 1 Ω to 10 MΩ. The zero-ohm parts `R_0R_J_<size>` have no Vishay entry. Rated power at 70 °C matches the Yageo primary in the standard operation mode. Limiting voltages and the 0201 temperature range differ (see Pitfalls). Not every library MPN is covered by the datasheets' ordering tables (see Not established).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Rated dissipation P70, CRCW0201 | — | — | 0.05 | W | at 70 °C | crcw0201 p. 1 |
| Rated dissipation P70, CRCW0402 | — | — | 0.063 / 0.10 | W | standard / extended operation mode | dcrcw p. 1, p. 2 |
| Rated dissipation P70, CRCW0603 | — | — | 0.10 / 0.125 | W | standard / extended operation mode | dcrcw p. 1, p. 2 |
| Rated dissipation P70, CRCW0805 | — | — | 0.125 / 0.25 | W | standard / extended operation mode | dcrcw p. 1, p. 2 |
| Rated dissipation P70, CRCW1206 / 1210 | — | — | 0.25 / 0.5 | W | at 70 °C | dcrcw p. 1 |
| Rated dissipation P70, CRCW2010 / 2512 | — | — | 0.75 / 1.0 | W | at 70 °C | dcrcw p. 1 |
| Resistance, ±1%, TCR ±100 ppm/K (K), 0201 | 47.0 Ω | — | 1 MΩ | — | E24; E96 | crcw0201 p. 1 |
| Resistance, ±1%, TCR ±200 ppm/K (N), 0201 | 10.0 Ω | — | 10 MΩ | — | E24; E96 | crcw0201 p. 1 |
| Resistance, ±5%, TCR ±200 ppm/K (N), 0201 | 10.0 Ω | — | 10 MΩ | — | E24; below 10 Ω only -200/+400 ppm/K (X) | crcw0201 p. 1 |
| Resistance, ±1%, TCR ±100 ppm/K (K), 0402 to 2512 | 1 Ω | — | 10 MΩ | — | E24; E96; every size except 1218 | dcrcw p. 2 |
| Resistance, ±5%, TCR ±200 ppm/K (N), 0402 to 2512 | 1 Ω | — | 10 MΩ | — | E24; every size except 1218 | dcrcw p. 2 |
| TCR test temperatures, 0201 | — | — | — | °C | (20 / -55 / 20) and (20 / 125 / 20) | crcw0201 p. 3 |
| TCR test temperatures, 0402 to 2512 | — | — | — | °C | (20 / -55 / 20) and (20 / 155 / 20) | dcrcw p. 8 |
| Jumper resistance, 0201 / 0402 to 2512 | — | — | 50 / 20 | mΩ | jumpers are not used by the library | crcw0201 p. 1; dcrcw p. 2 |
| Resistance change, 1000 h at P70, 0402 to 0805 | — | — | 1 / 2 | % | standard / extended mode, stability class 1 | dcrcw p. 2 |

## Absolute maximum ratings

Neither datasheet has an "absolute maximum ratings" table. The rows below are the limiting element voltage, the insulation voltage and the jumper current. Each datasheet gives these as ratings.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Limiting element voltage Umax AC RMS/DC, 0201 | 30 | V | crcw0201 p. 1 |
| Umax, 0402 | 50 (standard) / 75 (extended) | V | dcrcw p. 2 |
| Umax, 0603 / 0805 | 75 / 150 | V | dcrcw p. 1, p. 2 |
| Umax, 1206 / 1210 | 200 / 200 | V | dcrcw p. 1 |
| Umax, 2010 / 2512 | 400 / 500 | V | dcrcw p. 1 |
| Insulation voltage Uins (1 min), 0201 | 50 | V | crcw0201 p. 1 |
| Insulation voltage Uins (1 min), 0402 / 0603 / 0805 | 100 / 100 / 200 | V | dcrcw p. 1 |
| Insulation voltage Uins (1 min), 1206 / 1210 / 2010 / 2512 | 300 | V | dcrcw p. 1 |
| Permissible film temperature | 155 | °C | crcw0201 p. 1; dcrcw p. 1 |
| Jumper Imax, 0201 | 1.0 (at 70 °C) | A | crcw0201 p. 1 |
| Jumper Imax, 0402 / 0603 / 0805 / 1206 | 1.5 / 2.0 / 2.5 / 3.5 | A | dcrcw p. 2 |
| Jumper Imax, 1210 / 2010 / 2512 | 5.0 / 6.0 / 7.0 | A | dcrcw p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature, 0201 | -55 | +155 | °C | crcw0201 p. 1 |
| Operating temperature, 0402 to 2512 | -55 | +155 | °C | dcrcw p. 1 |
| Ambient for full P70; linear derating above it to 0 at 155 °C | — | 70 | °C | crcw0201 p. 3; dcrcw p. 6 |
| Continuous voltage: U = √(P70 × R), but not above Umax | — | Umax | V | crcw0201 p. 3; dcrcw p. 9 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | termination (either end) | passive | Resistor terminal; not polarised | crcw0201 p. 2; dcrcw p. 11 |
| 2 | B | termination (other end) | passive | Resistor terminal; not polarised | crcw0201 p. 2; dcrcw p. 11 |

## Typical application

- No external parts are required. A and B are interchangeable because the two terminations are identical in the dimension drawings (crcw0201 p. 2; dcrcw p. 11).
- Reading a library MPN, e.g. `CRCW04021K58FKED`: `CRCW` model, `0402` size, `1K58` value, `F` tolerance, `K` TCR, `ED` packaging (dcrcw p. 3).
- The value is four characters, with R, K or M as the decimal point: `562R` = 562 Ω, `1K00` = 1 kΩ, `0000` = jumper (crcw0201 p. 2; dcrcw p. 3). The library's `vishay_code` always writes three significant digits, e.g. `1R00` and `10M0`.
- Tolerance: F = ±1%, J = ±5%, Z = jumper. TCR: K = ±100 ppm/K, N = ±200 ppm/K, 0 = jumper. X = -200/+400 ppm/K is defined for 0201 only (crcw0201 p. 2; dcrcw p. 3).
- So the library's `FKED` means ±1%, ±100 ppm/K, packaging ED, and `JNED` means ±5%, ±200 ppm/K, packaging ED.
- What ED means depends on the size:
  - 0201: ET7, 10 000 pieces, 180 mm / 7" reel, 8 mm paper tape, 2 mm pitch (crcw0201 p. 2).
  - 0402: ET7, 10 000 pieces, Ø 180 mm / 7" reel, 8 mm paper tape, 2 mm pitch (dcrcw p. 3).
  - 0603: ET3, 10 000 pieces, Ø 180 mm / 7" reel, 2 mm pitch (dcrcw p. 3).

## Layout notes

- Vishay reflow lands for 0402 to 2512 (G gap / Y pad length / X pad width / Z outer span, mm) (dcrcw p. 11):
  - 0402: 0.45 / 0.6 / 0.6 / 1.65
  - 0603: 0.75 / 0.75 / 1.00 / 2.25
  - 0805: 1.00 / 0.95 / 1.45 / 2.90
  - 1206: 1.50 / 1.05 / 1.80 / 3.60
  - 1210: 1.70 / 1.10 / 2.80 / 3.90
  - 2010: 3.50 / 1.45 / 2.80 / 6.40
  - 2512: 4.75 / 1.45 / 3.50 / 7.65
- The same page gives wave-soldering lands for 0603 and larger, and none for 0402 (dcrcw p. 11).
- 0201 solder pads: a = 0.28, b = 0.43, l = 0.23 mm (crcw0201 p. 2). In the drawing, a is the pad length, b the pad width and l the gap.
- The library's `CHIP_<size>` lands are KiCad nominal lands. They were checked against these pages (crcw0201 p. 2; dcrcw p. 11) but deliberately do not copy them (`lib/passive/docs/README.md`, "Footprint audit").
- Body L × W (mm), 0402 to 2512 (dcrcw p. 11): 1.0 × 0.5, 1.55 × 0.85, 2.0 × 1.25, 3.2 × 1.6, 3.2 × 2.5, 5.0 × 2.5 and 6.3 × 3.15.
- Body L × W, 0201: 0.6 × 0.3 mm (crcw0201 p. 2). The 0201 is not marked (crcw0201 p. 2).
- Rated power depends on the solder-point temperature, the placement density and the substrate (crcw0201 p. 1; dcrcw p. 1). Size copper for the derated power.
- The substrate-bending qualification uses a 2 mm bend depth (crcw0201 p. 3; dcrcw p. 10).

## Pitfalls

- P70 applies at or below 70 °C ambient. Above that, power derates linearly to zero at 155 °C (crcw0201 p. 3; dcrcw p. 6).
- The front-page ratings for 0402, 0603 and 0805 (0.10, 0.125 and 0.25 W) are extended-mode values (dcrcw p. 1, p. 2).
  - The standard-mode values are 0.063, 0.10 and 0.125 W.
  - Extended mode allows ≤ 2% drift after 1000 h, against ≤ 1% in standard mode (dcrcw p. 2).
  - The part number has no field for the mode (dcrcw p. 3), so design to the standard values.
- Small sizes are limited by voltage. Umax is 30 V for 0201, and 50 V for 0402 in standard mode (crcw0201 p. 1; dcrcw p. 2). At high resistance values, Umax rather than √(P70 × R) is the limit (dcrcw p. 9).
- Differences from the Yageo RC_L primary (`YAGEO_RC_SERIES.md`):
  - Umax for 2010 and 2512 is 400 / 500 V for Vishay, against 200 V for Yageo (dcrcw p. 1). Do not design to the Vishay figure: the primary may be placed.
  - The 0201 operates to +155 °C for Vishay, against +125 °C for Yageo (crcw0201 p. 1).
  - 0201 Umax is 30 V for Vishay, against 25 V for Yageo (crcw0201 p. 1).
  - Vishay's ±1% K grade is ±100 ppm/K down to 1 Ω in 0402 to 2512, where Yageo allows ±200 ppm/K at 10 Ω and below (dcrcw p. 2).
  - Vishay insulation voltages are lower than Yageo's withstanding voltages for 0603 and larger (dcrcw p. 1).
- 0201 TCR grades cover narrower ranges (crcw0201 p. 1):
  - ±100 ppm/K at ±1% covers only 47.0 Ω to 1 MΩ.
  - ±200 ppm/K at ±5% starts at 10.0 Ω.
  - Of the library's 0201 alternates, 304 `FKED` codes (1 Ω to 46.4 Ω and 1.02 MΩ to 10 MΩ) and 24 `JNED` codes (1 Ω to 9.1 Ω) fall outside the ranges the datasheet lists.
- Do not copy the 0201 example "CRCW02011K00FNED" without checking it: its header says N, but the boxed breakdown below it shows K (crcw0201 p. 2).

## Not established by the sources

- Packaging code `ED` is listed only for 0201, 0402 and 0603 (crcw0201 p. 2; dcrcw p. 3).
  - For 0805, 1206 and 1210 the D/CRCW table lists only EA/EC. For 2010 it lists only EF, and for 2512 only EG/EH (dcrcw p. 3).
  - So 4,832 library alternates (`CRCW0805…` to `CRCW2512…`, ending `ED`) use a packaging code these datasheets do not define. The generator applies `FKED`/`JNED` to every size (`tools/gen_passive.py`, `emit_resistors`).
- Whether Vishay will supply the 0201 codes outside the TCR ranges (see Pitfalls) is not established.
- No absolute-maximum table, thermal resistance, or pulse rating in numbers is given. The pulse load and pulse voltage limits are graphs only (dcrcw p. 5).
- The pulse curves cover 0402 to 2512; the 0201 datasheet has none (crcw0201 pp. 1–4).
- The D/CRCW jumper Imax has no stated temperature or duration (dcrcw p. 2). The library has no Vishay jumper.
- Neither datasheet numbers the terminations. Pad 1 = A and pad 2 = B is the library's convention.
- Which individual E24/E96 values Vishay produces is not checked against an endpoint, as it is for Yageo (`lib/passive/docs/README.md`).
