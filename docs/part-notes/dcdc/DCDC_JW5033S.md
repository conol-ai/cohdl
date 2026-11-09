# JW5033S — 18 V, 2 A synchronous step-down converter, TSOT23-6

## Identity

```json
{
  "schema": "part-note/1",
  "device": "dcdc::JW5033S",
  "parts": ["dcdc::DCDC_JW5033S"],
  "identities": [
    {"mfr": "Joulwatt", "mpn": "JW5033S", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "JW5033S 18V/2A Sync. Step-Down Converter", "revision": "Rev. 0.88 (package README: 2019-03-19; the PDF prints no date)",
     "url": "https://www.lcsc.com/product-detail/C324577.html", "sha256": "a391630b450176f2280eb662e152ba28ecd1ee9ed39da7fa5b0270b467b3e3f0", "local": "lib/dcdc/docs/jw5033s.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. On pages 1–15 they equal the printed page numbers.

**No official URL.** Neither the package README nor the PDF gives a Joulwatt link; the README says the PDF came from the distributor SZLCSC, part C324577. The recorded URL is that distributor's product page, built from the README's part number and not checked (no network). `lcsc` is null because `tools/sourcing_data/lcsc_map.json` has no row for `JW5033S`.

## Summary

The JW5033S is a current-mode synchronous buck with two integrated N-channel MOSFETs, 3.7–18 V input, 2 A continuous output, 800 kHz switching and PFM at light load (ds p. 1, p. 9). The output is set by a divider to a 0.8 V feedback reference (ds p. 4, p. 10). It comes in TSOT23-6 and SOT563, which have **different pinouts** (ds p. 2, p. 5); the library models only TSOT23-6. Choose it for a 2 A rail from a 5–18 V input. Check the recommended output range first: the source limits VOUT to VIN − 3 V (ds p. 2). Do not use it above 18 V input (20 V absolute, ds p. 2).

## Key parameters

VIN = 12 V, TA = 25 °C unless stated (ds p. 4). No limits over temperature are given.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output current | — | 2 | — | A | Continuous (Features) | ds p. 1 |
| VIN UVLO threshold | 3.2 | 3.4 | 3.7 | V | VIN rising | ds p. 4 |
| VIN UVLO hysteresis | — | 300 | — | mV | Guaranteed by design | ds p. 4 |
| Shutdown supply current ISD | — | 0.1 | 1 | µA | VEN = 0 V | ds p. 4 |
| Supply current IQ | — | 120 | 150 | µA | VEN = 5 V, VFB = 1.2 V | ds p. 4 |
| Feedback voltage VFB | 776 | 800 | 824 | mV | 3.7 V < VIN < 18 V | ds p. 4 |
| Top switch RDS(ON) | — | 130 | — | mΩ | Guaranteed by design | ds p. 4 |
| Bottom switch RDS(ON) | — | 70 | — | mΩ | Guaranteed by design | ds p. 4 |
| Top / bottom switch leakage | — | 0.1 | 1 | µA | VIN = 18 V, VEN = 0 V | ds p. 4 |
| Top switch current limit | 3.2 | 3.9 | — | A | Minimum duty cycle | ds p. 4 |
| Switching frequency FSW | 600 | 800 | 1000 | kHz | — | ds p. 4 |
| Minimum on time | — | 100 | — | ns | Guaranteed by design | ds p. 4 |
| Minimum off time | — | 150 | — | ns | VFB = 0.4 V, guaranteed by design | ds p. 4 |
| EN input high voltage VEN_H | 2.4 | — | — | V | — | ds p. 4 |
| EN input low voltage VEN_L | — | — | 1.8 | V | — | ds p. 4 |
| Thermal shutdown / hysteresis | — | 145 / 20 | — | °C | Guaranteed by design; restarts below 125 °C | ds p. 4, p. 9 |
| RθJA / RθJC / RθJB, TSOT23-6 | — | 110 / 55 / 14.7 | — | °C/W | JESD51-7 4-layer PCB | ds p. 3 |
| ψJT / ψJB, TSOT23-6 | — | 1.2 / 14.7 | — | °C/W | JESD51-7 4-layer PCB | ds p. 3 |
| Recommended max power loss PD, TSOT23-6 | — | — | 0.9 | W | 25 °C | ds p. 3 |
| Recommended max case temperature TC, TSOT23-6 | — | — | 119 | °C | — | ds p. 3 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VIN pin | -0.3 to 20 | V | ds p. 2 |
| EN pin | -0.3 to 20 | V | ds p. 2 |
| SW | -0.3 (-4.5 for 10 ns) to 20 (22 for 10 ns) | V | ds p. 2 |
| BST pin | SW - 0.3 to SW + 5.5 | V | ds p. 2 |
| All other pins (GND, FB) | -0.3 to 6 | V | ds p. 2 |
| Junction temperature | 150 | °C | ds p. 2 |
| Lead temperature | 260 | °C | ds p. 2 |
| Storage temperature | -65 to 150 | °C | ds p. 3 |
| ESD, human body model | 2 | kV | ds p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Input voltage VIN | 3.7 | 18 | V | ds p. 2 |
| Output voltage VOUT | 0.8 | VIN - 3 | V | ds p. 2 |
| Operating junction temperature | -40 | 125 | °C | ds p. 3 |

## Pins

TSOT23-6, order code JW5033STSOTB#TRPBF (ds p. 2). Top view: pins 1–3 (GND, SW, VIN) down the left, pins 4–6 (FB, EN, BST) up the right, so 6 is across from 1 (ds p. 2).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | GND | GND | ground | Ground | ds p. 2, p. 5 |
| 2 | SW | SW | power output | Switching node; connect the output LC filter | ds p. 2, p. 5 |
| 3 | IN | VIN | power input | Supply input, 3.7–18 V; bypass to GND | ds p. 2, p. 5 |
| 4 | FB | FB | input | Feedback, regulated to 0.8 V; connect the divider | ds p. 2, p. 5 |
| 5 | EN | EN | input | Enable: high on, low off | ds p. 2, p. 5 |
| 6 | BS | BST | passive | Bootstrap: ≥ 0.1 µF from BST to SW | ds p. 2, p. 5 |

## Pins, SOT563 package (not modelled by the library)

The library has no SOT563 variant. This table records the other package so that it is never bought for the TSOT footprint. Order code JW5033SSOTI#TRPBF (ds p. 2).

| Pad | Datasheet name | Source |
|---|---|---|
| 1 | VIN | ds p. 2, p. 5 |
| 2 | SW | ds p. 2, p. 5 |
| 3 | GND | ds p. 2, p. 5 |
| 4 | BST | ds p. 2, p. 5 |
| 5 | EN | ds p. 2, p. 5 |
| 6 | FB | ds p. 2, p. 5 |

## Typical application

- Feedback divider: VFB = VOUT × R2 / (R2 + R3), with R2 from FB to GND and R3 from VOUT to FB. Keep R2 below 30 kΩ, choose it near 15 kΩ, and set R3 = R2 × (VOUT / 0.8 V − 1) (ds p. 10).
- Recommended dividers (R2 / R3): 2.5 V: 22.1 kΩ / 47 kΩ; 3.3 V: 16 kΩ / 49.9 kΩ; 5 V: 20 kΩ / 105 kΩ (ds p. 10).
- CIN: a 22 µF ceramic is recommended. With an electrolytic input capacitor, add a small X5R or X7R ceramic, e.g. 0.1 µF, close to the IC (ds p. 10).
- COUT: a 22 µF ceramic is recommended. Low-ESR electrolytic or tantalum also work (ds p. 10).
- Inductor: L = VOUT / (fs × ΔIL) × (1 − VOUT / VIN), with ΔIL typically 40% of the maximum switch current limit (ds p. 11). The reference designs use 3.3 µH for 3.3 V and 4.7 µH for 5 V (ds p. 1, p. 12).
- Bootstrap: a 0.1 µF low-ESR ceramic from BST to SW (ds p. 5, p. 11).
- EN: the reference designs pull EN up to VIN through 100 kΩ (ds p. 1, p. 12). For an input UVLO above the internal one, use a divider from VIN to EN to GND; EN below 1.8 V turns the part off (ds p. 4, p. 9).
- Reference designs: 3.3 V / 2 A from 4.7–18 V, and 5 V / 2 A from 6–18 V (ds p. 12). See Pitfalls for the conflict with the VOUT limit.

## Layout notes

- Place the input decoupling capacitor as close as possible to VIN and ground, and minimise the loop it forms with GND (ds p. 11).
- Route the feedback trace as far as possible from the inductor and noisy power traces (ds p. 11).
- Make the ground plane as large as possible for heat dissipation (ds p. 11). θJA = 110 °C/W is for a JESD51-7 4-layer board (ds p. 3).
- The source has a package outline (ds p. 13) but **no recommended land pattern**. Lead pitch 0.95 mm, lead width b 0.30–0.50 mm, overall width E 2.60–3.00 mm, body E1 1.50–1.70 mm, foot length L 0.30–0.60 mm (ds p. 13).

## Pitfalls

- **Package and pinout.** TSOT23-6 and SOT563 have different pin maps: pin 1 is GND on TSOT23-6 and VIN on SOT563, and FB/BST swap between pins 4 and 6 (ds p. 2, p. 5). The library MPN `JW5033S` names neither order code. Order JW5033STSOTB#TRPBF for the library footprint (ds p. 2).
- **Output voltage headroom.** The recommended output range is 0.8 V to VIN − 3 V (ds p. 2). A 3.3 V output then needs VIN ≥ 6.3 V, and 3.8 V needs ≥ 6.8 V. The source's own 3.3 V reference design starts at 4.7 V input (ds p. 1, p. 12); see "Not established".
- BST may rise only 5.5 V above SW (ds p. 2). Do not feed BST from an external rail; use the 0.1 µF capacitor to SW (ds p. 11).
- FB is an "all other pins" pin, rated -0.3 to 6 V (ds p. 2). Do not tie it to VOUT above 6 V.
- The source states no internal pull on EN. Do not leave it floating; tie it to VIN through a resistor or drive it (ds p. 1, p. 4).
- At light load the part runs in PFM and the switching frequency drops (ds p. 9). There is no forced-PWM option.
- Over-current is hiccup mode, not constant-current limiting (ds p. 9).

## Not established by the sources

- **Official source URL.** No Joulwatt URL is in the README or the PDF. The recorded URL is a distributor page built from the README's SZLCSC part number C324577. It was not opened.
- **LCSC part number.** `lcsc_map.json` has no row for `JW5033S`. The README's C324577 is not checked against the package code.
- **Which package the MPN means.** `JW5033S` is the device name, not an order code (ds p. 2). Which package an `JW5033S` purchase delivers is not established.
- **Pin-1 identification on the part.** The TSOT23-6 pin configuration has no pin-1 dot, only the SOT563 drawing has one (ds p. 2). The package outline marks no pin 1 (ds p. 13). The library's pin-1 marker at the upper-left pad follows the numbering on ds p. 2.
- **Land pattern.** The source gives none (ds p. 13). Library `FP_JW5033S_SOT236` uses 1.0 × 0.55 mm pads with centres at x = ±0.95 mm, so each pad spans 0.45–1.45 mm from the centre line. By the outline the nominal lead foot spans about 0.95–1.40 mm (E/2 = 1.40 mm, L = 0.45 mm), so the pads reach 0.05 mm past the nominal lead tip and 0.35 mm under the body (E1/2 = 0.80 mm). That fit is not checked against any IPC rule here.
- **Library comment on the land.** `jw5033s.cohdl` says the land is the "same geometry as … rs2257.cohdl (SOT6P95X160X290N)". It is not: RS2257 uses 0.99 × 0.69 mm pads at x = ±1.295 mm.
- **Library pin names.** The library calls pin 3 `IN` and pin 6 `BS`; the datasheet calls them VIN and BST (ds p. 2, p. 5).
- **EN comment.** The library comment says EN is valid "up to VIN". The source rates EN at -0.3 to 20 V regardless of VIN (ds p. 2) and gives no VIN-relative limit.
- **Header comment on the use case.** `jw5033s.cohdl` says the part supplies a 3.8 V Air780E VBAT rail on "the three-watch project". That project is not in this repository's sources. A 3.8 V output needs VIN ≥ 6.8 V by ds p. 2.
- **README description.** `lib/dcdc/docs/README.md` calls the document a "3.8V/2A sync buck". The datasheet title is 18V/2A (ds p. 1).
- **Datasheet conflict on VOUT headroom.** The recommended VOUT ≤ VIN − 3 V (ds p. 2) disagrees with the 3.3 V reference design from 4.7 V (ds p. 1, p. 12). The source does not say which governs or give a maximum duty cycle. Minimum off time is 150 ns typical (ds p. 4).
- **Datasheet text.** The layout guide says to decouple "VIN pin and PGND" (ds p. 11). The package has no PGND pin; GND is pin 1 (ds p. 5).
- **Not given:** limits over temperature (all at 25 °C, ds p. 4), soft-start time, inductor saturation current, capacitor voltage ratings, maximum EN input current, and moisture sensitivity level.
