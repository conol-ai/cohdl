# SGM2554 — 2.2 V to 5.5 V, 1.1 A power distribution switch, SOT-23-5

## Identity

```json
{
  "schema": "part-note/1",
  "device": "load-switch::SGM2554",
  "parts": ["load-switch::LOADSW_SGM2554"],
  "identities": [
    {"mfr": "SGMICRO", "mpn": "SGM2554", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "SGM2554 Power Distribution Switch", "revision": "REV. A.4, April 2022, 13 pages",
     "url": "https://www.sg-micro.com", "sha256": "58d6e47c130c08cdf6f49f1b7844eb9079b0255018a846df5ec0516e2c002a87", "local": "lib/load-switch/docs/sgm2554.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed page numbers on pp. 2–10; pp. 1 and 11–13 carry no printed number.

The source URL is only the manufacturer's home page. Neither the package README nor the PDF gives a document URL. The README says the PDF was retrieved through LCSC C1156937; `lcsc_map.json` has no SGM2554 row, so that listing is not checked here.

**Warning: the library's pin map is wrong for this part.** See "Pins" and the first item under "Not established by the sources". Do not use `LOADSW_SGM2554` on a board until the device is fixed.

## Summary

The SGM2554 is a single-channel power distribution switch with an integrated 95 mΩ N-MOSFET. It runs from 2.2 V to 5.5 V and is rated for 1.1 A continuous (ds p. 1). It has soft-start, a 1.85 A typical current limit, input UVLO, thermal shutdown and reverse-leakage blocking (ds p. 1, p. 4). Two models exist. SGM2554A has an active-high EN on pin 4. SGM2554B has no enable: pin 4 is a second VIN (ds p. 2, p. 3). Use the A model to gate a 3.3 V or 5 V rail from a GPIO. Do not use the part above 5.5 V input or 6 V absolute (ds p. 2, p. 4). Its current limit has only a typical value, so do not use it where a guaranteed trip point is needed (ds p. 4).

## Key parameters

All values at TA = +25 °C, VIN = 5 V, CIN = COUT = 1 µF unless noted (ds p. 4). The source gives no limits over temperature.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Input voltage range VIN | 2.2 | — | 5.5 | V | — | ds p. 4 |
| Continuous current | — | — | 1.1 | A | Features list; no conditions given | ds p. 1 |
| Quiescent current IQ | — | 18 | 27 | µA | VIN = 3 V, VEN = 3 V | ds p. 4 |
| Quiescent current IQ | — | 19 | 30 | µA | VIN = 5 V, VEN = 5 V | ds p. 4 |
| Shutdown current ISD (SGM2554A) | — | 0.1 | 1 | µA | VEN = 0 V | ds p. 4 |
| Output leakage current (SGM2554A) | — | 0.1 | 1 | µA | VEN = 0 V, VOUT = 0 V | ds p. 4 |
| EN input high threshold VIH (SGM2554A) | 1.5 | — | — | V | — | ds p. 4 |
| EN input low threshold VIL (SGM2554A) | — | — | 0.4 | V | — | ds p. 4 |
| Output turn-on delay tON | — | 1.1 | — | ms | RL = 30 Ω, COUT = 1 µF | ds p. 4 |
| Switch resistance RDS(ON) | — | 95 | 150 | mΩ | IL = 1 A | ds p. 4 |
| Current limit threshold ILIM, SGM2554A | — | 1.85 | — | A | RL = 2 Ω | ds p. 4 |
| Current limit threshold ILIM, SGM2554B | — | 1.75 | — | A | RL = 2 Ω | ds p. 4 |
| Short-circuit output current, SGM2554A | — | 1.2 | — | A | VOUT = 0 V, VIN = 3 V | ds p. 4 |
| Short-circuit output current, SGM2554B | — | 1.1 | — | A | VOUT = 0 V, VIN = 3 V | ds p. 4 |
| UVLO threshold | — | 1.65 | — | V | VIN rising | ds p. 4 |
| UVLO hysteresis | — | 50 | — | mV | — | ds p. 4 |
| Thermal shutdown threshold | — | 125 | — | °C | TJ increasing | ds p. 4 |
| Thermal shutdown hysteresis | — | 20 | — | °C | — | ds p. 4 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Input supply voltage | 6 | V | ds p. 2 |
| EN pin | -0.3 to 6 | V | ds p. 2 |
| Package thermal resistance θJA, SOT-23-5 | 250 | °C/W | ds p. 2 |
| Junction temperature | +150 | °C | ds p. 2 |
| Storage temperature | -65 to +150 | °C | ds p. 2 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 2 |
| ESD, HBM | 2000 | V | ds p. 2 |
| ESD, MM | 300 | V | ds p. 2 |

## Recommended operating conditions

The datasheet has no separate recommended-operating-conditions table. The rows below are the input range from the electrical characteristics table and the operating temperature range. The datasheet lists that range among its absolute maximum ratings, and as the specified range in the ordering table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Input voltage VIN | 2.2 | 5.5 | V | ds p. 4 |
| Operating temperature | -40 | +85 | °C | ds p. 2 |

## Pins

The "Library pin" column is what `load-switch::SGM2554` declares. The "Datasheet name" column is what the source assigns to that pad. **Three of the five pads disagree** (pads 1, 3, 4).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | IN | VOUT (both models) | Power output | Switch output. The library calls this pad the supply input | ds p. 3 |
| 2 | GND | GND | Ground | Ground | ds p. 3 |
| 3 | EN | VIN (both models) | Power input | Switch input. The library calls this pad the enable | ds p. 3 |
| 4 | NC | EN (SGM2554A) / VIN (SGM2554B) | Input (A) / power input (B) | A: chip enable, logic high enables. B: second switch input. The library calls this pad "no connect" | ds p. 3 |
| 5 | OUT | VOUT (both models) | Power output | Switch output; tied to pad 1 by name | ds p. 3 |

Pad positions agree with the source: top view, pads 1–3 down the left side, pad 4 at the lower right, pad 5 at the upper right (ds p. 3). The library footprint places pad 1 at the upper left with the same arrangement.

## Typical application

- SGM2554A connection per the source: VIN on pin 3, EN on pin 4, VOUT on pins 1 and 5, GND on pin 2 (ds p. 1). CIN = 1 µF from VIN to GND and COUT = 1 µF from VOUT to GND (ds p. 1). Test circuit 1 uses the same parts (ds p. 5).
- SGM2554B has no EN. Pins 3 and 4 are both VIN, so the switch is on whenever VIN is above UVLO (ds p. 3, p. 5 test circuit 2).
- EN: logic high enables the device (ds p. 3). Drive it at ≥ 1.5 V on and ≤ 0.4 V off (ds p. 4); stay within -0.3 V to 6 V (ds p. 2).
- Turn-on takes about 1.1 ms with a 30 Ω load and 1 µF output capacitor (ds p. 4). The typical inrush curves for COUT = 1 µF, 33 µF and 100 µF are on ds p. 6.
- Dissipation at 1.1 A and the 150 mΩ maximum RDS(ON): 1.1² × 0.15 ≈ 0.18 W. At 250 °C/W that is about a 45 °C rise (ds p. 2, p. 4). The source does not state the board for its θJA figure.

## Layout notes

- Recommended land (ds p. 11): 0.69 mm × 0.99 mm pads, 0.95 mm pitch, 1.90 mm between the two pads on the short side, and 2.59 mm between pad-row centres.
- The library's `FP_SGM2554_SOT235` uses 0.60 mm × 1.00 mm pads with rows 2.60 mm apart. The pads are 0.09 mm narrower than the recommended land; see "Not established".
- Package body (ds p. 11): D 2.750–3.050 mm, E 1.450–1.750 mm, E1 (span over leads) 2.600–3.000 mm, A 1.450 mm max. JEDEC MO-178 reference.
- Place CIN and COUT next to the VIN and VOUT pins (ds p. 1 shows them at the pins). The source gives no other layout guidance and no copper-area condition for θJA (ds p. 2).
- Tape pin-1 quadrant Q3 on a 7-inch reel, 8 mm tape (ds p. 12).

## Pitfalls

- **Do not wire to the library pin names.** Library `IN` is VOUT, library `EN` is VIN, and library `NC` is EN (ds p. 3). A board built from the library symbol puts the supply on an output pin, drives the switch input from the GPIO, and leaves the real EN floating. The output then follows the input through the pad 1–pad 5 VOUT connection whatever the GPIO does.
- The library MPN `SGM2554` names no model. Only the A model has EN; on the B model pin 4 is VIN (ds p. 2, p. 3). Order `SGM2554AYN5G/TR` for an enable (ds p. 2).
- Input absolute maximum is 6 V and the operating range ends at 5.5 V (ds p. 2, p. 4). A 5 V USB rail with overshoot leaves little margin.
- Operating temperature is -40 °C to +85 °C only (ds p. 2).
- The current limit (1.85 A) and short-circuit current (1.2 A) are typical values at 25 °C with no minimum (ds p. 4). Do not count on them for a protection budget.
- The EN thresholds and shutdown current are specified for SGM2554A only, at 25 °C (ds p. 4).

## Not established by the sources

Disagreements between the library and the source, most serious first:

- **Pin map (pads 1, 3, 4).** The device declares `IN: 1`, `EN: 3`, `NC: 4`, `OUT: 5`, and its comments repeat this. `docs/sgm2554.md` says "Pinout (SOT-23-5): 1=IN, 2=GND, 3=EN, 4=NC, 5=OUT" and "Pinout follows the standard SOT-23-5 load switch layout". The source gives SGM2554A as 1 VOUT, 2 GND, 3 VIN, 4 EN, 5 VOUT, and SGM2554B as 1 VOUT, 2 GND, 3 VIN, 4 VIN, 5 VOUT (ds p. 3; same assignment in the p. 1 typical application and the p. 5 test circuits). Pad 2 matches. Pad 5 matches in function. Pads 1, 3 and 4 do not. The pin types are wrong with them: pad 1 is typed `power_in` but is an output, and pad 4 is `optional ... [passive]` but is the A model's enable input.
- **Ambiguous MPN.** The library entry `SGM2554` is not an ordering number. The source lists `SGM2554AYN5G/TR` (marking SI4XX) and `SGM2554BYN5G/TR` (marking SI5XX), 3000 per reel (ds p. 2). The library comments assume an enable, which only the A model has (ds p. 3).
- **Input voltage.** The device comment ("1.0-5.5V in") and the `IN` pin comment ("supply input 1.0-5.5V") disagree with the source's 2.2 V to 5.5 V (ds p. 1, p. 4). `docs/README.md` and `docs/sgm2554.md` note the correction, but the `.cohdl` comments are unchanged.
- **Referenced footprint does not exist.** The footprint comment says "Same geometry as SOT5P95X290X145N (lib/@contrib/ldo/src/rt9080.cohdl)", and `docs/sgm2554.md` says "matches SOT5P95X290X145N in lib/@contrib/ldo". `lib/@contrib/ldo` declares no `SOT5P95X290X145N`. Its RT9080 TSOT-23-5 footprint is `SOT5P95X290X160N`. Commit 2228bdc (2026-08-04, "fix: audit contrib pin maps and footprints") renamed it from `SOT5P95X290X145N` and widened its pads from 1.0 × 0.6 mm to 1.0 × 0.7 mm. `FP_SGM2554_SOT235` keeps 1.0 × 0.6 mm pads, so it does not have the same geometry as the current `SOT5P95X290X160N`. A footprint named `SOT5P95X290X145N` does exist, in a different package (`lib/ldo/src/rt9193.cohdl`), but it also has 1.0 × 0.7 mm pads.
- **Land pattern.** The library pads are 0.60 × 1.00 mm on 2.60 mm row centres. The source's recommended land is 0.69 × 0.99 mm on 2.59 mm row centres (ds p. 11). Pitch (0.95 mm) and pad positions otherwise agree.
- **Footprint comment dimensions.** The comment "body 2.9 x 2.8, pitch 0.95, e1 2.6" does not match the source. The source gives e1 = 1.900 BSC; 2.6 is its E1 minimum and about its 2.59 mm land row pitch. It gives D 2.750–3.050 and E1 2.600–3.000 with no nominal (MOD) values (ds p. 11).
- **Courtyard.** The library courtyard is 3.0 mm in the pitch direction, which is less than the body length D max of 3.050 mm (ds p. 11).
- **Documentation link.** Both `#[doc]` attributes point at `docs/sgm2554.md`, the repository summary, not the manufacturer PDF.
- `docs/sgm2554.md` says the rated current "depends on package/PCB". The source states 1.1 A continuous with no package or PCB condition (ds p. 1). It also gives no derating.

Not stated by the source:

- EN pull-up or pull-down, and what the A model does with EN floating.
- Limits over temperature for any parameter. Every table value is at 25 °C (ds p. 4).
- Minimum current limit, current-limit response time, and whether the part latches or retries after thermal shutdown.
- Conditions for "No Reversed Leakage Current" and the reverse-voltage function (ds p. 1): threshold, response time and maximum reverse voltage.
- Output discharge when disabled.
- The test board behind θJA = 250 °C/W (ds p. 2).
- Turn-off delay limits. Only a typical-versus-temperature curve is given (ds p. 7).
