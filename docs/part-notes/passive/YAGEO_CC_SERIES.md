# Yageo CC series — general-purpose MLCCs, NP0/C0G, X5R and X7R, 0201 to 1206

## Identity

```json
{
  "schema": "part-note/1",
  "device": "passive::MLCC",
  "parts_pattern": "C_.*_(C0G|X5R|X7R)_(0201|0402|0603|0805|1206)",
  "variants": ["C0201", "C0402", "C0603", "C0805", "C1206"],
  "identities": [
    {"mfr": "Yageo", "mpn_pattern": "CC(0201|0402|0603|0805|1206)(JRNPO[0789AY]BN|KRX5R[4-9]BB|KRX7R[05-9AY]BB)(\\d{3}|\\dR\\d)", "lcsc": null}
  ],
  "sources": [
    {"id": "c0g", "title": "Surface-Mount Ceramic Multilayer Capacitors, General purpose, Class 1, NP0, 16 V to 250 V", "revision": "Product specification V.21, 2025-05-14",
     "url": "https://yageogroup.com/component-documentation/download/datasheet/CC0402JRNPO0BN100?lang=en", "sha256": "d1c53cd79d37157ff17a277cfd323e305eac3fa99e01652faef28b808ee190b2", "local": "lib/passive/docs/yageo_cc_series_datasheet_en.pdf"},
    {"id": "x5r", "title": "Surface Mount Multilayer Ceramic Capacitors, General purpose & High capacitance, Class 2, X5R, 4 V to 50 V", "revision": "Product specification V.31, 2025-10-08",
     "url": "https://yageogroup.com/component-documentation/download/datasheet/CC0201KRX5R4BB224?lang=en", "sha256": "c0e9d28a0f3330167f57b232b8b159e4b3b22c46ec58fedaa12261af22099486", "local": "lib/passive/docs/yageo_x5r_4v_to_50v_datasheet_en.pdf"},
    {"id": "x7r", "title": "Surface-Mount Ceramic Multilayer Capacitors, General Purpose & High Capacitance, Class 2, X7R, 6.3 V to 250 V", "revision": "Product specification V.27, 2026-03-06",
     "url": "https://yageogroup.com/component-documentation/download/datasheet/CC0402KRX7R5BB101?lang=en", "sha256": "3aa10c5d01fbea28f638f1e4bca56a14b9b3342f593d68641830b1cbdb3720e6", "local": "lib/passive/docs/yageo_x7r_6v3_to_250v_datasheet_en.pdf"}
  ],
  "status": "draft"
}
```

## Summary

Yageo CC is a family of multilayer ceramic chip capacitors with lead-free NiSn terminations, specified in three documents by dielectric: class-1 NP0 (C0G), and class-2 X5R and X7R (c0g p. 3, x5r p. 3, x7r p. 3). It is the primary entry of every generated `passive` MLCC part `C_<value>_<voltage>_<C0G|X5R|X7R>_<size>`: C0G at ±5% (J), X5R and X7R at ±10% (K), all on 7-inch paper/PE reels (c0g p. 2, x5r p. 2, x7r p. 2). The choice is decided by dielectric: C0G changes at most ±30 ppm/°C, while X5R and X7R may change ±15% over temperature, and X5R is rated only to +85 °C (c0g p. 17, x5r p. 9, x7r p. 14). None of the three documents quantifies capacitance loss under DC bias for X5R or X7R, so do not use the nominal value of a class-2 part where the effective capacitance at the working voltage matters without another source.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Capacitance range, NP0/C0G document | 0.22 pF | — | 100 nF | — | all sizes of the document | c0g p. 17 |
| Capacitance range, X5R document | 100 pF | — | 220 µF | — | all sizes of the document | x5r p. 9 |
| Capacitance range, X7R document | 100 pF | — | 47 µF | — | all sizes of the document | x7r p. 14 |
| Tolerance, C0G (code J) | — | ±5 | — | % | J is listed for C ≥ 10 pF; C < 10 pF lists ±0.1 / ±0.25 / ±0.5 pF | c0g p. 2, p. 17 |
| Tolerance, X5R / X7R (code K) | — | ±10 | — | % | X5R also M = ±20%; X7R also J = ±5% (not full range) and M = ±20% | x5r p. 2, x7r p. 2 |
| Rated voltage codes, C0G | 16 | — | 250 | V | 7 = 16, 8 = 25, 9 = 50, 0 = 100, A = 200, Y = 250 | c0g p. 2 |
| Rated voltage codes, X5R | 4 | — | 50 | V | 4 = 4, 5 = 6.3, 6 = 10, 7 = 16, 8 = 25, 9 = 50 | x5r p. 2 |
| Rated voltage codes, X7R | 6.3 | — | 250 | V | 5 = 6.3, 6 = 10, 7 = 16, 8 = 25, 9 = 50, 0 = 100, A = 200, Y = 250 | x7r p. 2 |
| Rated voltages by size, C0G | — | — | — | V | 0201: 25/50/100; 0402: 16/25/50/100; 0603, 0805, 1206: 25 to 250 | c0g p. 4, p. 5, p. 8 |
| Rated voltages by size, X5R | — | — | — | V | 0201, 0402, 0603, 0805: 4 to 50; 1206: 6.3 to 50 | x5r p. 4, p. 6, p. 7 |
| Rated voltages by size, X7R | — | — | — | V | 0201: 6.3 to 50; 0402: 6.3 to 100; 0603, 0805, 1206: 6.3 to 250 | x7r pp. 4–7 |
| Temperature coefficient, C0G | -30 | — | +30 | ppm/°C | C at 125 °C vs 25 °C, ΔT = 100 °C | c0g p. 17, p. 19 |
| Capacitance change over temperature, X5R | -15 | — | +15 | % | over the operating range, vs 25 °C | x5r p. 9, p. 15 |
| Capacitance change over temperature, X7R | -15 | — | +15 | % | over the operating range, vs 25 °C | x7r p. 14, p. 21 |
| Dissipation factor, C0G | — | — | 1 / (400 + 20C) | — | C < 30 pF (C in pF) | c0g p. 17 |
| Dissipation factor, C0G | — | — | 0.1 | % | C ≥ 30 pF | c0g p. 17 |
| Dissipation factor, X5R | — | — | 3.5 to 20 | % | by size, voltage and value (Table 11) | x5r p. 9 |
| Dissipation factor, X7R | — | — | 2.5 to 12.5 | % | by size, voltage and value (Table 12) | x7r p. 14 |
| Insulation resistance, C0G | 10 GΩ or 500 s | — | — | — | Rins or Rins × Cr, whichever is less; 1 min at Ur | c0g p. 17 |
| Insulation resistance, X5R | 10 GΩ or 500 / 100 / 50 / 20 Ω·F | — | — | — | whichever is less; R × C class by size/voltage/value | x5r p. 9, p. 10 |
| Insulation resistance, X7R | 10 GΩ or 500 / 100 / 50 Ω·F | — | — | — | whichever is less; R × C class by size/voltage/value | x7r p. 14, p. 15 |
| Measurement, C0G | — | 1 | — | Vrms | 1 MHz for C ≤ 1 nF, 1 kHz for C > 1 nF, 20 °C | c0g p. 19 |
| Measurement, X5R / X7R | — | 1 | — | Vrms | 1 kHz for C ≤ 1 µF; 0.5 Vrms if C > 1 µF and Ur ≤ 6.3 V; 120 Hz, 0.5 Vrms if C > 10 µF | x5r p. 14, x7r p. 20 |

## Absolute maximum ratings

The three documents have no absolute-maximum table. The rows below are qualification test stresses from their "Tests and requirements" tables, not damage thresholds or permitted operating levels (see "Not established by the sources").

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Voltage proof test, Ur ≤ 100 V (C0G: 1 min, 7.5 mA; X5R/X7R: 1 to 5 s, < 50 mA) | 2.5 × Ur | V | c0g p. 21, x5r p. 16, x7r p. 26 |
| Voltage proof test, 100 V < Ur ≤ 200 V | 1.5 × Ur + 100 | V | c0g p. 21, x7r p. 26 |
| Voltage proof test, 200 V < Ur ≤ 500 V | 1.3 × Ur + 100 | V | c0g p. 21, x7r p. 26 |
| Endurance test, C0G, 1,000 h at 125 °C | 2.0 × Ur (≤ 100 V); 1.5 × Ur (200/250 V) | V | c0g p. 21 |
| Endurance test, X7R, 1,000 h at 125 °C | 2.0 × Ur general purpose; 1.5 × Ur high capacitance and all 250 V | V | x7r p. 25 |
| Endurance test, X5R, 1,000 h at 85 °C | 2.0, 1.5 or 1.0 × Ur by size/voltage/value | V | x5r p. 18 |
| Resistance to soldering heat test, solder bath, 10 ± 0.5 s | 260 ± 5 | °C | c0g p. 20, x5r p. 16, x7r p. 22 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature, C0G | -55 | +125 | °C | c0g p. 17 |
| Operating temperature, X5R | -55 | +85 | °C | x5r p. 9 |
| Operating temperature, X7R | -55 | +125 | °C | x7r p. 14 |
| DC voltage across the part | — | Ur, the rated voltage of the part's voltage code | V | c0g p. 2, x5r p. 2, x7r p. 2 |
| Damp heat with load (qualification), 500 h at 40 °C, 90 to 95% RH | — | 1.0 × Ur | V | c0g p. 21, x5r p. 17, x7r p. 23 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | termination (either end) | passive | Capacitor terminal; not polarised | c0g p. 3, x5r p. 3, x7r p. 3 |
| 2 | B | termination (other end) | passive | Capacitor terminal; not polarised | c0g p. 3, x5r p. 3, x7r p. 3 |

## Typical application

- No external parts. The two end terminations are identical (Fig. 1 and Fig. 2, c0g p. 3, x5r p. 3, x7r p. 3), so A and B may go to either net.
- Reading a library MPN, e.g. `CC0402KRX7R7BB104` (x7r p. 2): `CC` series; `0402` size, inch code (1005 metric); `K` tolerance ±10%; `R` paper/PE taping reel, 7 inch; `X7R` dielectric (`NPO` for C0G, `X5R`); `7` rated voltage 16 V; `BB` process code (`BN` for NP0, c0g p. 2); `104` capacitance, two significant digits plus number of zeros in pF, so 10 × 10^4 pF = 100 nF.
- The letter R is the decimal point in the capacitance code (c0g p. 2): `CC0402JRNPO0BN3R9` is 3.9 pF; the datasheet example `121` is 120 pF (c0g p. 2), `103` is 10 nF (x5r p. 2).
- The library part name carries value, voltage, dielectric and size, not tolerance: `C_100n_16V_X7R_0402` is `CC0402KRX7R7BB104`, `C_27p_50V_C0G_0603` is `CC0603JRNPO9BN270`. Tolerance is fixed per dielectric by `tools/gen_passive.py` (`C_DIELECTRIC`).

## Layout notes

- Body L × W (mm): 0201 0.6 × 0.3, 0402 1.0 × 0.5, 0603 1.6 × 0.8, 0805 2.0 × 1.25, 1206 3.2 × 1.6; tolerances and thickness T depend on dielectric and value (c0g p. 3, x5r p. 3, x7r p. 3). Thickness codes per value are in the capacitance tables; a 1206 X7R reaches T = 1.6 mm (dimension codes FC/FD, x7r p. 3, p. 7).
- None of the three documents gives a land pattern. The library's `CHIP_<size>` lands are KiCad nominal lands audited against Vishay and Samsung pages (`lib/passive/docs/README.md`, "Footprint audit"), not against Yageo.
- Soldering method, C0G: reflow for all sizes, wave only for 0603, 0805 and 1206 (c0g p. 18). X5R: 0201 reflow only; wave is allowed only up to 100 nF (0402), 1 µF (0603), 2.2 µF (0805, 1206) (x5r p. 13). X7R: 0402 and smaller reflow only; wave up to 1.0 µF (0603), 2.2 µF (0805), 4.7 µF (1206) (x7r p. 15).
- Bending test: 1 mm bend on a 5 mm radius jig; class-2 limit ΔC/C ±10% general purpose, ±12.5% high capacitance (x5r p. 15, x7r p. 22). Keep class-2 parts away from board edges, V-scores and connectors.
- MSL level 1 (c0g p. 2, x5r p. 2, x7r p. 2).

## Pitfalls

- DC bias: the X5R and X7R documents give no capacitance-vs-DC-voltage curve or table (x5r pp. 9–13, x7r pp. 14–19). Do not assume a class-2 `C_10u_6V3_X5R_0603` delivers 10 µF at 5 V; get the derating from another source before sizing bulk decoupling or a regulator's minimum output capacitance.
- The X7R temperature-characteristic row adds "0201/22nF~100nF/16V: within 1/2 DC bias" without explaining it (x7r p. 21); the library ships those 0201 16 V X7R parts, so treat their ±15% as specified under a bias condition you cannot verify.
- Rated voltage is Ur from the voltage code (x7r p. 2). High-capacitance class-2 parts are endurance-tested at only 1.5 × Ur (X7R) or down to 1.0 × Ur (X5R) instead of 2.0 × Ur, with ±20% instead of ±15% ΔC/C after test (x7r p. 25, x5r p. 18). Which values count as high capacitance is listed per size and voltage (x5r p. 17, x7r p. 24); e.g. X5R 0603 10 µF at 6.3 V to 25 V is high capacitance (x5r p. 17).
- Use C0G for crystal load capacitors, RC timing and filters: ±30 ppm/°C against ±15% for X5R/X7R (c0g p. 17, x7r p. 14, x5r p. 9).
- X5R stops at +85 °C (x5r p. 9). Above that ambient use X7R or C0G (+125 °C, x7r p. 14, c0g p. 17).
- Size limits voltage: X5R tops out at 50 V in every size (x5r p. 2), X7R 0201 at 50 V and 0402 at 100 V (x7r p. 4); 200 V and 250 V exist only from 0603 up (c0g p. 4, p. 5, x7r p. 4, p. 5). C0G 0201 has no 16 V code (c0g p. 4).
- The library offers less than the tables. Only MPNs confirmed by Yageo's specsheet endpoint are emitted (`tools/passive_data/yageo_verified.json`): there is no X5R 1206 part, only four X5R 0805 and four X7R 1206 parts, though x5r p. 7 and x7r p. 7 list many more. Search `lib/passive/src/capacitors_<size>.cohdl` before choosing a value.
- Below 10 pF the C0G document lists only absolute tolerances (±0.1/±0.25/±0.5 pF, c0g p. 17), but `C_3p9_100V_C0G_0402` and `C_8p2_50V_C0G_0603` are spelled with J (±5%). Check the actual tolerance before using them in a tuned RF circuit.
- Alternates are different products: 716 parts carry a Samsung CL alternate (`samsung_mlcc_catalog_2025-12.pdf`) and `C_15p_50V_C0G_0402` / `C_100n_16V_X7R_0402` a Murata GRM alternate, each with its own datasheet in `lib/passive/docs/`. This note's numbers do not apply to them; with `--supplier lcsc`, `C_100n_16V_X7R_0402` is placed as the Samsung `CL05B104KO5NNNC`.

## Not established by the sources

- No absolute-maximum table exists. The voltage-proof, endurance and solder-bath values above are test conditions (c0g pp. 20–21, x5r pp. 16–18, x7r pp. 22–26); no surge, transient or reverse-voltage rating is given.
- DC-bias behaviour is not quantified. For C0G, Fig. 7 "Typical capacitance change ... as a function of DC voltage" shows only axes (±10%, 0 to 50 V) with no curve in this revision (c0g p. 17); X5R and X7R have no such figure.
- Capacitance ageing of the class-2 dielectrics is not stated in any of the three documents.
- ESR, impedance and self-resonance are only typical plots for a few values (x5r pp. 11–13, x7r pp. 16–19); there is no tabulated ESR or ripple-current rating.
- Recommended land pattern and reflow profile are not given; only the allowed soldering method per size (c0g p. 18, x5r p. 13, x7r p. 15).
- The documents do not number the terminations; pad 1 = A and pad 2 = B is the library's convention.
- Which exact part numbers Yageo produces is not derivable from the tables; the library relies on `tools/passive_data/yageo_mlcc.json` and the endpoint snapshot of 2026-07-25.
