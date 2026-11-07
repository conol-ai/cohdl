# SN74AHCT1G125DBVR — single bus buffer, 3-state output, active-low OE, TTL inputs, SOT-23-5

## Identity

```json
{
  "schema": "part-note/1",
  "device": "logic::SN74AHCT1G125",
  "parts": ["logic::BUFFER_SN74AHCT1G125"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "SN74AHCT1G125DBVR", "lcsc": "C7484"}
  ],
  "sources": [
    {"id": "ds", "title": "SN74AHCT1G125 Single Bus Buffer Gate With 3-State Output", "revision": "SCLS378P, revised March 2024; package option addendum dated 8-Nov-2025; DBV0005A drawing 4214839/K 08/2024",
     "url": "https://www.ti.com/lit/ds/symlink/sn74ahct1g125.pdf", "sha256": "dbaf49b3af33690fc7f7356afe387e7815a56b8bb73fe1e88bf794f4fb8e0d2f", "local": "lib/logic/docs/sn74ahct1g125.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal TI's printed page numbers on pp. 1–12. The addendum and mechanical pages (pp. 13–26) are unnumbered.

## Summary

The SN74AHCT1G125 is a single non-inverting buffer with a 3-state output. Y follows A while OE is low, and Y is high-impedance while OE is high (ds p. 1, p. 8). It runs from 4.5 V to 5.5 V. Its TTL-level inputs (VIH 2 V, VIL 0.8 V) accept 3.3 V logic, so it is a 3.3 V to 5 V up-translator (ds p. 4, p. 9). The output drives ±8 mA (ds p. 4). Do not use it on a 3.3 V supply: VCC below 4.5 V is outside the recommended range (ds p. 4). Do not use it as a down-translator or where the output is pulled above VCC (ds p. 9).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| VOH | 4.4 | 4.5 | — | V | VCC = 4.5 V, IOH = -50 µA, TA = 25 °C (min 4.4 V also -40 °C to 125 °C) | ds p. 5 |
| VOH | 3.94 | — | — | V | VCC = 4.5 V, IOH = -8 mA, TA = 25 °C | ds p. 5 |
| VOH | 3.8 | — | — | V | VCC = 4.5 V, IOH = -8 mA, -40 °C to 125 °C | ds p. 5 |
| VOL | — | — | 0.1 | V | VCC = 4.5 V, IOL = 50 µA, all temperature columns | ds p. 5 |
| VOL | — | — | 0.36 | V | VCC = 4.5 V, IOL = 8 mA, TA = 25 °C | ds p. 5 |
| VOL | — | — | 0.44 | V | VCC = 4.5 V, IOL = 8 mA, -40 °C to 125 °C | ds p. 5 |
| Input leakage II | — | — | ±1 | µA | VI = 5.5 V or GND, VCC = 0 V to 5.5 V, -40 °C to 125 °C (±0.1 µA at 25 °C) | ds p. 5 |
| Off-state output current IOZ | — | — | ±2.5 | µA | VO = VCC or GND, VCC = 5.5 V, -40 °C to 125 °C (±0.25 µA at 25 °C) | ds p. 5 |
| Supply current ICC | — | — | 10 | µA | VI = VCC or GND, IO = 0, VCC = 5.5 V, -40 °C to 125 °C (1 µA at 25 °C) | ds p. 5 |
| Supply-current change ΔICC per TTL-level input | — | — | 1.5 | mA | One input at 3.4 V, other at VCC or GND, VCC = 5.5 V, -40 °C to 125 °C (1.35 mA at 25 °C) | ds p. 5 |
| Input capacitance Ci | — | 4 | 10 | pF | VI = VCC or GND, VCC = 5 V | ds p. 5 |
| Output capacitance Co | — | 10 | — | pF | VO = VCC or GND, VCC = 5 V | ds p. 5 |
| tPLH, tPHL (A to Y) | 1 | 3.8 | 7 | ns | CL = 15 pF, VCC = 5 V ± 0.5 V; typ at 25 °C, max over -40 °C to 125 °C (5.5 ns at 25 °C, 6.5 ns to 85 °C) | ds p. 6 |
| tPLH, tPHL (A to Y) | 1 | 5.3 | 9.5 | ns | CL = 50 pF; max over -40 °C to 125 °C (7.5 ns at 25 °C, 8.5 ns to 85 °C) | ds p. 6 |
| tPZH, tPZL (OE to Y) | 1 | 3.6 | 6.5 | ns | CL = 15 pF; max over -40 °C to 125 °C | ds p. 6 |
| tPHZ, tPLZ (OE to Y) | 1 | 4.6 | 8.5 | ns | CL = 15 pF; max over -40 °C to 125 °C | ds p. 6 |
| tPZH, tPZL (OE to Y) | 1 | 5.1 | 9 | ns | CL = 50 pF; max over -40 °C to 125 °C | ds p. 6 |
| tPHZ, tPLZ (OE to Y) | 1 | 6.1 | 11 | ns | CL = 50 pF; max over -40 °C to 125 °C | ds p. 6 |
| Power dissipation capacitance Cpd | — | 14 | — | pF | No load, f = 1 MHz, VCC = 5 V, TA = 25 °C | ds p. 6 |
| RθJA, DBV | — | 278 | — | °C/W | Thermal metric (single value) | ds p. 5 |

The 1 ns minimum propagation delays are specified in the -40 °C to 85 °C and -40 °C to 125 °C columns only (ds p. 6).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | -0.5 to 7 | V | ds p. 4 |
| Input voltage VI | -0.5 to 7 | V | ds p. 4 |
| Output voltage VO | -0.5 to VCC + 0.5 | V | ds p. 4 |
| Input clamp current IIK (VI < 0) | -20 | mA | ds p. 4 |
| Output clamp current IOK (VO < 0 or VO > VCC) | ±20 | mA | ds p. 4 |
| Continuous output current IO (VO = 0 to VCC) | ±25 | mA | ds p. 4 |
| Continuous current through VCC or GND | ±50 | mA | ds p. 4 |
| Storage temperature | -65 to 150 | °C | ds p. 4 |
| Junction temperature | 150 | °C | ds p. 4 |
| ESD, HBM (JS-001, all pins) | ±1000 | V | ds p. 4 |
| ESD, CDM (JESD22-C101, all pins) | ±1500 | V | ds p. 4 |

The input and output voltage ratings may be exceeded if the input and output current ratings are observed (ds p. 4, note 2).

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 4.5 | 5.5 | V | ds p. 4 |
| High-level input voltage VIH | 2 | — | V | ds p. 4 |
| Low-level input voltage VIL | — | 0.8 | V | ds p. 4 |
| Input voltage VI | 0 | 5.5 | V | ds p. 4 |
| Output voltage VO | 0 | VCC | V | ds p. 4 |
| High-level output current IOH | — | -8 | mA | ds p. 4 |
| Low-level output current IOL | — | 8 | mA | ds p. 4 |
| Input transition rise or fall rate Δt/Δv | — | 20 | ns/V | ds p. 4 |
| Operating free-air temperature TA | -40 | 125 | °C | ds p. 4 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | OE_N | OE (overbar, active low) | Input | Output enable: low passes A to Y, high puts Y in Z | ds p. 3, p. 8 |
| 2 | A | A | Input | Data input | ds p. 3 |
| 3 | GND | GND | Ground | Ground | ds p. 3 |
| 4 | Y | Y | Output (3-state) | Data output | ds p. 3, p. 8 |
| 5 | VCC | VCC | Power | Supply | ds p. 3 |

DBV top view: pads 1–3 down the left side, pad 4 lower right, pad 5 upper right (ds p. 3). The library footprint matches this.

## Typical application

- Function: OE = L, A = H gives Y = H; OE = L, A = L gives Y = L; OE = H gives Y = Z (ds p. 8).
- Bypass VCC with 0.1 µF placed as close to the power pin as possible (ds p. 10).
- To keep Y high-impedance during power-up and power-down, tie OE to VCC through a pull-up resistor. Its minimum value is set by the current-sinking capability of the driver (ds p. 8).
- Hold unused inputs at VCC or GND. Do not leave A or OE floating (ds p. 4 note 1, p. 10).
- 3.3 V to 5 V translation: drive A from 3.3 V logic and power VCC from 5 V (ds p. 9, p. 10 Figure 8-2). Each input held at a TTL level such as 3.4 V adds up to 1.5 mA of supply current (ΔICC, ds p. 5).
- Keep load current at or below 25 mA per output and 50 mA total, and do not pull the output above VCC (ds p. 9).

## Layout notes

- DBV land (ds p. 24, drawing 4214839/K): five 1.1 mm × 0.6 mm pads, 0.95 mm pitch, 1.9 mm between pads 4 and 5, and 2.6 mm between row centres. Non-solder-mask-defined pads are preferred, with 0.07 mm max mask clearance. `SOT5P95X300X145N` matches these values.
- DBV body (ds p. 23): 2.75–3.05 mm long, 1.45–1.75 mm wide, 2.6–3.0 mm across the leads, 1.45 mm max height.
- Note 5 on p. 23: the small body feature it marks (a "support pin") may differ or may not be present. The land pattern has no pad for it (ds p. 24).
- The source states no thermal-pad or copper requirement. RθJA is 278 °C/W (ds p. 5).

## Pitfalls

- OE is active low (ds p. 3, p. 8). Tie OE_N to GND for an always-on buffer, not to VCC.
- On a 3.3 V supply the part is outside its recommended range (4.5 V min, ds p. 4). For 3.3 V-only systems this is the wrong part.
- The output is 3-state, so a floating OE_N leaves the output state undefined (ds p. 4 note 1).
- TTL-level inputs held at 3.3 V increase ICC by up to 1.5 mA each (ds p. 5). Budget this in low-power designs.
- The DBVT (250-piece reel) ordering code is Obsolete; DBVR is Active (ds p. 13). The library uses DBVR.

## Not established by the sources

The library's pins, pin 1 location, land, footprint and comments agree with the source. Minor points:

- The library types Y as `output`. The source calls it a 3-state output (ds p. 8). An ERC that treats `output` as always driving may flag a shared bus that is legitimate when OE_N is high.
- `docs/README.md` records the source as "Rev. P (March 2024)". The pinned PDF also carries a package option addendum dated 8-Nov-2025 (ds p. 13). The same Rev. P datasheet may therefore be republished with a different addendum and a different SHA-256.
- The addendum lists the SN74AHCT1G125DBVR lead finish as "Call TI | Sn", with markings 37QH, 3BEF, B253, B25G, B25J, B25L and B25S (ds p. 13). Which marking a given reel carries is not established.

Not stated by the source:

- Input hysteresis.
- Behaviour with VCC below 4.5 V, including power-up with inputs high before VCC, apart from the OE pull-up advice (ds p. 8).
- Output drive or timing at loads other than CL = 15 pF and 50 pF.
- The board behind the RθJA value (ds p. 5).
