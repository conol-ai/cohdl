# Samsung Electro-Mechanics CL series — commercial MLCCs, C0G / X5R / X7R, 0201 to 1206 (library alternate)

## Identity

```json
{
  "schema": "part-note/1",
  "device": "passive::MLCC",
  "parts_pattern": "C_.*_(C0G|X5R|X7R)_(0201|0402|0603|0805|1206)",
  "variants": ["C0201", "C0402", "C0603", "C0805", "C1206"],
  "identities": [
    {"mfr": "Samsung", "mpn_pattern": "CL(03|05|10|21|31)[ABC]\\d{3}[JK][A-EOPQ][0-9A-Z]{4}C?", "lcsc": null}
  ],
  "sources": [
    {"id": "cat", "title": "Multilayer Ceramic Capacitors, Part I. Commercial/Industrial (Samsung Electro-Mechanics MLCC catalog)", "revision": "December 2025",
     "url": "https://weblib.samsungsem.com/resources/file/product-catalog/MLCC_2512.pdf", "sha256": "99f87a705209ab4e097df75e0bdaf61cb7abd162b60e88fdb376d121f952e0b7", "local": "lib/passive/docs/samsung_mlcc_catalog_2025-12.pdf"}
  ],
  "status": "draft"
}
```

## Summary

CL is Samsung Electro-Mechanics' multilayer ceramic capacitor series; the "Normal Capacitors_Standard" level is the general-purpose line for consumer equipment, with Ni/Cu electrodes, Cu (or Cu + metal epoxy) terminations and Ni/Sn plating (cat p. 9, p. 10). In the library a CL number is only ever an `alt` entry of a generated `C_<value>_<volts>_<dielectric>_<size>` part whose primary is Yageo CC: 716 parts from 0201 to 1206 carry one Samsung number each (109 / 188 / 226 / 117 / 76 by size), none in 1210. All are C0G with tolerance J (±5%) or X5R / X7R with tolerance K (±10%), rated 6.3 V to 250 V (cat pp. 6–7). The number comes from an exact-MPN product-database snapshot (`tools/passive_data/samsung_cl.json`) or, for four parts, from the hand-verified list in `tools/gen_passive.py`, because the thickness and design codes cannot be derived from value, voltage and dielectric. Choose C0G where linearity and low loss matter (cat p. 30); do not expect X5R / X7R to hold their nominal capacitance under DC bias (cat p. 30, p. 47).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Temperature coefficient, C0G (code C) | -30 | 0 | +30 | ppm/°C | -55 to +125 °C | cat p. 6, p. 19 |
| Capacitance change, X5R (code A) | -15 | — | +15 | % | -55 to +85 °C, without bias | cat p. 6, p. 19 |
| Capacitance change, X7R (code B) | -15 | — | +15 | % | -55 to +125 °C, without bias | cat p. 6, p. 19 |
| Capacitance tolerance, code J / K | — | ±5 / ±10 | — | % | J on every library C0G number, K on every X5R / X7R number | cat p. 7 |
| Q, C0G | 1,000 | — | — | — | C ≥ 30 pF | cat p. 18 |
| Q, C0G | 400 + 20 × C | — | — | — | C < 30 pF, C in pF | cat p. 18 |
| Tan δ, X5R, 50 V / 35 V | — | — | 0.025 (0.05*) | — | * = parts with an individual specification | cat p. 18 |
| Tan δ, X5R, 25 V | — | — | 0.025 (0.05* / 0.10*) | — | as above | cat p. 18 |
| Tan δ, X5R, 16 V and ≤ 10 V | — | — | 0.035 (0.10*) | — | as above | cat p. 18 |
| Tan δ, X7R, ≥ 50 V / 35 V / 25 V | — | — | 0.025 (0.05* / 0.10*) | — | as above | cat p. 18 |
| Tan δ, X7R, 16 V | — | — | 0.035 (0.10*) | — | as above | cat p. 18 |
| Tan δ, X7R, ≤ 10 V | — | — | 0.05 (0.10*) | — | as above | cat p. 18 |
| Insulation resistance, Vr > 16 V | 10,000 MΩ or 500 MΩ·µF (or 100 MΩ·µF) | — | — | — | whichever is smaller; rated voltage applied 60 to 120 s | cat p. 18 |
| Insulation resistance, Vr ≤ 16 V | 10,000 MΩ or 100 MΩ·µF | — | — | — | whichever is smaller; rated voltage applied 60 to 120 s | cat p. 18 |
| Measurement, class I (C0G) | — | — | — | — | ≤ 1,000 pF at 1 MHz, > 1,000 pF at 1 kHz (±10%), 0.5 to 5 Vrms | cat p. 18 |
| Measurement, class II (X5R / X7R) | — | — | — | — | ≤ 10 µF: 1 kHz, 1.0 ± 0.2 Vrms; > 10 µF: 120 Hz, 0.5 ± 0.1 Vrms; listed exceptions 1 kHz, 0.5 ± 0.1 Vrms | cat p. 18, p. 30 |
| Capacitance change after 1 mm board bend | — | — | ±5% or ±0.5 pF / ±10% | — | class I (larger of the two) / class II | cat p. 19 |

## Absolute maximum ratings

The catalog has no absolute-maximum table. The first three rows are its withstanding-voltage test levels (1 to 5 s, 50 mA charge current maximum), not ratings for continuous use; the others are its stated limits for use and soldering.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Withstanding voltage test, class I, Vr < 100 V | 300% of Vr | V | cat p. 18 |
| Withstanding voltage test, class II, Vr < 100 V | 250% of Vr | V | cat p. 18 |
| Withstanding voltage test, 100 V ≤ Vr < 500 V | 250% of Vr | V | cat p. 18 |
| DC, or peak of DC + AC, applied voltage | ≤ Vr | V | cat p. 35 |
| Peak-to-peak AC or pulse voltage; surge and static electricity | ≤ Vr | V | cat p. 35 |
| Reflow peak temperature | 240 to 260 | °C | cat p. 39 |
| Soldering iron tip, 0201 to 1206 (≥ 60 s preheat, ≤ 3 s contact, ΔT ≤ 190 °C) | 350 | °C | cat p. 42 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature, C0G | -55 | +125 | °C | cat p. 6 |
| Operating temperature, X5R | -55 | +85 | °C | cat p. 6 |
| Operating temperature, X7R | -55 | +125 | °C | cat p. 6 |
| Self-heating temperature rise (AC or pulse) | — | 20 | °C | cat p. 32 |
| Storage temperature | 0 | 40 | °C | cat p. 49 |
| Storage humidity | 0 | 70 | %RH | cat p. 49 |
| Shelf life from delivery (then check solderability) | — | 6 | months | cat p. 49 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | A | termination (either end) | passive | Capacitor terminal; not polarised | cat p. 48 |
| 2 | B | termination (other end) | passive | Capacitor terminal; not polarised | cat p. 48 |

## Typical application

- No external parts. A and B may go to either net; the two terminations are identical (cat p. 10, p. 48).
- Reading `CL05B104KO5NNNC` (cat pp. 6–8): `CL` series; `05` size 0402/1005 (library also uses `03` 0201/0603, `10` 0603/1608, `21` 0805/2012, `31` 1206/3216); `B` X7R (`C` C0G, `A` X5R); `104` = 10 × 10⁴ pF = 100 nF; `K` ±10% (`J` ±5%); `O` 16 V (library also uses `Q` 6.3, `P` 10, `A` 25, `B` 50, `C` 100, `D` 200, `E` 250 V); `5` thickness 0.50 ± 0.05 mm; `N` design: Ni electrode, Cu termination, Ni/Sn plating; `N` product code Normal; `N` control code Standard; `C` packaging: paper tape, 7" reel.
- Thickness codes the library uses, as the catalog defines them (cat p. 7): 0201 `3` 0.30 ± 0.03; 0402 `5` 0.50 ± 0.05 and `3` 0.30 (low-profile design L only, e.g. `CL05A105KQ3LNN`); 0603 `8` 0.80 ± 0.10 and `5` 0.50 +0/-0.1 (design L only, e.g. `CL10A225KP5LNN`); 0805 `A` 0.65, `C` 0.85, `F` 1.25 mm (each ±0.10); 1206 `C` 0.85 ± 0.15, `F` 1.25 ± 0.15, `H` 1.6 ± 0.20 mm.
- Codes ⑨⑩ name the reliability level: `NN` Standard, `NW` High Level I, `4N` High Level II, `PN` / `PJ` AEC-Q200 (cat p. 9). Library counts: `NN` 141, `NW` 28, `4N` 15, `PN` / `PJ` 260; 272 carry pairs p. 9 does not list (`IN` 203, `FN` 38, `PE` 14, others 17).
- Reel quantities (cat p. 24): 0201 and 0402 on 2 mm paper, 10K per 7" reel; 0603 0.8 mm on paper, 4K; 0805 and 1206 at T ≤ 0.85 mm on paper, 4K, and at T ≥ 1.0 mm on embossed tape, 2K per 7" reel.

## Layout notes

- Samsung reflow lands, a = gap, b = land length, c = land width, at the tightest chip tolerance listed (cat p. 48): 0201 (0603 mm, ±0.03) a 0.22–0.28, b 0.31–0.37, c 0.30–0.36; 0402 (1005, ±0.05) a 0.36–0.44, b 0.49–0.57, c 0.51–0.59; 0603 (1608, ±0.10) a 0.63–0.73, b 0.71–0.81, c 0.80–0.90; 0805 (2012, ±0.10) a 0.79–0.89, b 0.88–0.98, c 1.25–1.35; 1206 (3216, ±0.20) a 1.64–1.76, b 1.19–1.31, c 1.74–1.86 mm. The catalog says the land must be confirmed on the actual set and board (cat p. 48).
- The library's `CHIP_<size>` lands are KiCad nominal lands, not Samsung's: they were compared with cat p. 48 but not copied (`lib/passive/docs/README.md`, "Footprint audit"). Most library pads are wider than the Samsung c range; an alternate does not change the land.
- Reflow: preheat 150 to 200 °C for 60 to 120 s, 30 to 150 s above 217 °C, peak 240 to 260 °C, ΔT ≤ 150 °C up to 3216; fewer than three reflows (cat p. 39). Too much solder raises crack risk, too little weakens the joint (cat p. 40).
- Height: the tallest library numbers are 1206 code `H`, 1.6 ± 0.20 mm (cat p. 7).
- Board bending cracks MLCCs and a cracked part may short; the catalog asks for a fuse or other protection where a short would be dangerous (cat p. 47).

## Pitfalls

- DC bias: class II (X5R / X7R) capacitance changes with applied AC and DC voltage, and the TCC limits (±15%) are measured without bias (cat p. 19, p. 30). The catalog shows only an example bias TCC curve (10 µF, 6.3 V, X5R, at 3.15 Vdc) and asks the designer to evaluate the real system (cat p. 32, p. 47). Do not size a decoupling or filter capacitor on nominal value; use C0G where the value must hold (cat p. 30).
- Class II capacitance also falls with time (aging), linearly with log time, and recovers after 150 °C for 1 h (cat p. 31).
- Same value, different Samsung number: the thickness code is a free field per size, e.g. 0402 has `3` and `5`, 0805 has six codes (cat p. 7). `samsung_cl.json` records one number per combination, and `tools/gen_passive.py` overrides it where a number was checked by hand: for `C_100n_16V_X7R_0402` the database pick `CL05B104KO3LNN` (0.30 mm) is replaced by `CL05B104KO5NNNC` (0.50 mm). Two Samsung numbers for one library part are both the same value, voltage, dielectric and tolerance; they differ in thickness / design / level codes. Do not treat them as a difference in capacitance or rating.
- Tolerance `M` (±20%) exists in the catalog, and its own example `CL10A106MQ8NNNC` uses it (cat pp. 6–7). The library never uses an `M` number: a C0G part is ±5% (`J`), an X5R / X7R part ±10% (`K`). Do not substitute an `M` number for a library alternate.
- Do not read every library number as Standard level: many carry `PN` / `PJ`, `NW` or `4N`, which p. 9 assigns to AEC-Q200, High Level I or II (cat p. 9). Standard-level test conditions (40 °C, 95%RH, 1 Vr, 500 h; 1 mm board flex) apply as such only to `NN` (cat p. 9).
- An X5R part is only rated to +85 °C; an X7R or C0G part of the same value goes to +125 °C (cat p. 6).

## Not established by the sources

- 712 of 716 library numbers have 14 characters and stop after the control code, with no packaging code; only the four hand-verified numbers end in `C`. The catalog shows 11 fields (cat p. 6), so the reel and tape type of the 14-character numbers is not fixed by the library.
- 522 of 716 library numbers use at least one code that pp. 7–8 do not define for that position or size: design `1`, `3`, `4`, `6`, `P`, `V`, `W`, `X`, `I`; product `P`, `I`, `F`, `C`, `3`; control `J`, `E`; thickness 0201 `S`, 0805 `6` and `9`, 1206 `D` (cat pp. 7–8). Those numbers can only be decoded from Samsung's product database, which is not in the repository.
- Per-part capacitance under DC bias, ESR, self-resonance, ripple current and which numbers are "derated MLCC" (cat p. 35) are not in the catalog; it refers to individual specifications (cat p. 18).
- Body length and width tolerances per number are not given; p. 48 lands are indexed by chip tolerance, which the size-control code only sets for some numbers (cat p. 8).
- The catalog does not number the terminations; pad 1 = A and pad 2 = B is the library's convention.
