# Yageo RC_L series — general-purpose thick-film chip resistors, 0201 to 2512

## Identity

```json
{
  "schema": "part-note/1",
  "device": "passive::ChipResistor",
  "parts_pattern": "R_.*",
  "variants": ["R0201", "R0402", "R0603", "R0805", "R1206", "R1210", "R2010", "R2512"],
  "identities": [
    {"mfr": "Yageo", "mpn_pattern": "RC(0201|0402|0603|0805|1206|1210|2010|2512)[FJ][RK]-07\\d+[RKM]?\\d*L", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "General Purpose Chip Resistors RC_L series, sizes 0075 to 2512", "revision": "Product specification V.14, 2025-11-14",
     "url": "https://yageogroup.com/component-documentation/download/datasheet/RC0201FR-071RL?lang=en", "sha256": "7b6f0ae72708ca4b38126f84b5407a3e970a94cb60bc1a0afdaeb25a23d839bb", "local": "lib/passive/docs/yageo_rc_series_datasheet_en.pdf"}
  ],
  "status": "draft"
}
```

## Summary

RC_L is Yageo's general-purpose thick-film chip resistor series with lead-free (Ni / matte tin) terminations (ds p. 2, p. 4). It is the primary entry of every generated `passive` resistor `R_<value>_<F|J>_<size>`: ±1% (F) and ±5% (J) values from 1 Ω to 10 MΩ plus a ±5% zero-ohm jumper `R_0R_J_<size>`, all at the standard-power `-07` rating (ds p. 2). The choice between sizes is decided by rated power at 70 °C and maximum working voltage, which both rise with size (ds pp. 5–6). Do not use these parts where TCR better than ±100 ppm/°C or a pulse/surge rating is required (ds pp. 5–6); RC_L also has ±0.5% (D) and ±0.1% (B) codes (ds p. 2), but the library ships only F and J.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Rated power, RC0201 | — | — | 1/20 | W | at 70 °C, standard power | ds p. 5, p. 7 |
| Rated power, RC0402 | — | — | 1/16 | W | at 70 °C, standard power | ds p. 5, p. 7 |
| Rated power, RC0603 | — | — | 1/10 | W | at 70 °C, standard power | ds p. 5, p. 7 |
| Rated power, RC0805 | — | — | 1/8 | W | at 70 °C, standard power | ds p. 5, p. 7 |
| Rated power, RC1206 | — | — | 1/4 | W | at 70 °C, standard power | ds p. 6, p. 7 |
| Rated power, RC1210 | — | — | 1/2 | W | at 70 °C | ds p. 6, p. 7 |
| Rated power, RC2010 | — | — | 3/4 | W | at 70 °C | ds p. 6, p. 7 |
| Rated power, RC2512 | — | — | 1 | W | at 70 °C, standard power | ds p. 6, p. 7 |
| Tolerance code F / J | — | ±1.0 / ±5.0 | — | % | J is also the code for jumpers | ds p. 2 |
| Resistance range, ±1% (E24/E96) | 1 Ω | — | 10 MΩ | — | every size 0201 to 2512, standard power | ds pp. 5–6 |
| Resistance range, ±5% (E24) | 1 Ω | — | 100 MΩ | — | RC0201, RC0402, RC0805, RC1206 | ds pp. 5–6 |
| Resistance range, ±5% (E24) | 1 Ω | — | 22 MΩ | — | RC0603, RC1210, RC2010, RC2512 | ds pp. 5–6 |
| Jumper (0 Ω) resistance | — | — | < 50 | mΩ | every size | ds pp. 5–6 |
| TCR, RC0201 | -100 | — | +350 | ppm/°C | 1 Ω ≤ R ≤ 10 Ω | ds p. 5 |
| TCR, RC0201 | -200 | — | +200 | ppm/°C | 10 Ω < R ≤ 100 MΩ | ds p. 5 |
| TCR, RC0402 to RC2512 | -200 | — | +200 | ppm/°C | 1 Ω ≤ R ≤ 10 Ω | ds pp. 5–6 |
| TCR, RC0402 to RC2512 | -100 | — | +100 | ppm/°C | 10 Ω < R ≤ 10 MΩ | ds pp. 5–6 |
| TCR test temperatures | — | — | — | °C | +25/–55 and +25/+125, MIL-STD-202 method 304 | ds p. 8 |

## Absolute maximum ratings

The datasheet has no "absolute maximum ratings" table. The rows below are its maximum overload voltage, dielectric withstanding voltage and jumper maximum current columns (see "Not established by the sources").

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Maximum overload voltage, RC0201 | 50 | V | ds p. 5 |
| Maximum overload voltage, RC0402 | 100 | V | ds p. 5 |
| Maximum overload voltage, RC0603 | 150 | V | ds p. 5 |
| Maximum overload voltage, RC0805 | 300 | V | ds p. 5 |
| Maximum overload voltage, RC1206 | 400 | V | ds p. 6 |
| Maximum overload voltage, RC1210 / RC2010 / RC2512 (1 W) | 500 | V | ds p. 6 |
| Dielectric withstanding voltage, RC0201 / RC0402 / RC0603 / RC0805 | 50 / 100 / 150 / 300 | V | ds p. 5 |
| Dielectric withstanding voltage, RC1206 / RC1210 / RC2010 / RC2512 | 500 | V | ds p. 6 |
| Jumper maximum current, RC0201 | 1.0 | A | ds p. 5 |
| Jumper maximum current, RC0402 / RC0603 | 2.0 | A | ds p. 5 |
| Jumper maximum current, RC0805 | 5.0 | A | ds p. 5 |
| Jumper maximum current, RC1206 / RC1210 / RC2010 / RC2512 | 10.0 | A | ds p. 6 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature, RC0201 | -55 | +125 | °C | ds p. 5, p. 7 |
| Operating temperature, RC0402 to RC2512 | -55 | +155 | °C | ds pp. 5–7 |
| Ambient for full rated power (linear derating above, Fig. 10-1/10-2) | — | 70 | °C | ds p. 7 |
| Continuous working voltage | — | √(P × R) or the max working voltage, whichever is less | V | ds p. 7 |
| Maximum working voltage, RC0201 | — | 25 | V | ds p. 5 |
| Maximum working voltage, RC0402 | — | 50 | V | ds p. 5 |
| Maximum working voltage, RC0603 | — | 75 | V | ds p. 5 |
| Maximum working voltage, RC0805 | — | 150 | V | ds p. 5 |
| Maximum working voltage, RC1206 / RC1210 / RC2010 / RC2512 | — | 200 | V | ds p. 6 |
| Jumper rated current, RC0201 | — | 0.5 | A | ds p. 5 |
| Jumper rated current, RC0402 / RC0603 | — | 1.0 | A | ds p. 5 |
| Jumper rated current, RC0805 to RC2512 | — | 2.0 | A | ds pp. 5–6 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | termination (either end) | passive | Resistor terminal; not polarised | ds p. 4 |
| 2 | B | termination (other end) | passive | Resistor terminal; not polarised | ds p. 4 |

## Typical application

- No external parts are required. A or B may go to either net; the two terminations are identical (ds p. 4, Fig. 9).
- Reading a library MPN, e.g. `RC0402FR-0710KL` (ds p. 2): `RC` series; `0402` size; `F` tolerance (F = ±1%, J = ±5%); `R` packing (R = paper taping reel, K = embossed taping reel); `-` TCR "based on spec"; `07` = 7-inch reel and standard power; `10K` value code with R/K/M as the decimal point (`97R6` = 97.6 Ω, `9K76` = 9760 Ω, `1M` = 1 MΩ); `L` default ordering code.
- The datasheet's own example `RC0402JR-07100KL` is 100 kΩ ±5% in 0402 on a 7-inch reel of 10,000 (ds p. 2, p. 7).
- Jumpers are ordered with tolerance code J (ds p. 2); the library spells them `RC<size>J<R|K>-070RL`.
- RC2010 and RC2512 exist only on embossed taping reels, which is why the library uses `K` for those two sizes and `R` for 0201 to 1210 (ds p. 7).

## Layout notes

- Body dimensions L × W per size (mm): 0201 0.60 × 0.30, 0402 1.00 × 0.50, 0603 1.60 × 0.80, 0805 2.00 × 1.25, 1206 3.10 × 1.60, 1210 3.10 × 2.60, 2010 5.00 × 2.50, 2512 6.35 × 3.10; terminations I1/I2 are in the same table (ds p. 4, Table 1).
- The RC_L datasheet gives no land pattern; it refers to the separate "Chip resistors mounting" data sheet (ds p. 6). The library's `CHIP_<size>` lands are KiCad nominal lands audited against Vishay CRCW pages (`lib/passive/docs/README.md`, "Footprint audit"), not against Yageo.
- Power derates linearly from 100% at 70 °C to 0 at 155 °C (0402 to 2512, Fig. 10-1) or at 125 °C (0201, Fig. 10-2) (ds p. 7). Budget copper and placement for the derated power, not the 70 °C rating.
- Board-flex qualification bends 5 mm for 0201/0402, 3 mm for 0603/0805 and 2 mm for 1206 and larger (ds p. 9): keep the large sizes away from board edges and V-scores.
- MSL class 1 (ds p. 2).

## Pitfalls

- Rated power applies only at or below 70 °C ambient; above it, use Fig. 10-1/10-2 (ds p. 7).
- Small sizes are voltage-limited: 0201 is 25 V and 0402 50 V maximum working voltage (ds p. 5). For high resistance the √(P × R) limit exceeds that, so the working-voltage cap governs, not power (ds p. 7).
- The 0201 tops out at +125 °C, not +155 °C like the other sizes (ds p. 5, p. 7).
- A 0R jumper is rated by current: 0.5 A for 0201, 1.0 A for 0402/0603, 2.0 A for 0805 and larger (ds pp. 5–6). Do not place a 0201/0402 0R in a supply path above its rated current.
- The datasheet also lists double-power ratings (0402 1/8 W, 0603 1/5 W, 0805 1/4 W, 1206 1/2 W, 2512 2 W), ordered with reel codes `7W`/`3W` (ds p. 2, pp. 5–7). The library ships only `-07` standard-power parts; never assume the higher rating.
- Do not swap the packing letter: `RC2512FR-…` / `RC2010FR-…` are not orderable because those sizes have no paper reel (ds p. 7).
- The resistor is a passive pair: an ERC "direction" or polarity check on A/B means nothing physically (ds p. 4).

## Not established by the sources

- No absolute-maximum table exists (ds pp. 4–6). Maximum overload voltage is used in the short-time overload test at 2.5 × RCWV or that voltage, whichever is less, for 5 s (ds p. 9); the datasheet does not call it a damage threshold or give a pulse/surge rating.
- Whether the jumper maximum current is a short-time or continuous limit, and how jumper current derates with temperature, are not stated (ds pp. 5–6).
- Recommended land pattern and soldering profile are in "Chip resistors mounting" (ds p. 6), and tape/reel dimensions in "Chip resistors packing" (ds p. 7); neither document is in the repository.
- The datasheet does not number the terminations; pad 1 = A and pad 2 = B is the library's convention.
- Thermal resistance, current noise, voltage coefficient and inductance are not given.
- Which individual value codes Yageo actually produces is not listed; the library filters MPNs with an official-endpoint snapshot (`tools/passive_data/yageo_verified.json`), so some sizes ship fewer than 799 ±1% values.
