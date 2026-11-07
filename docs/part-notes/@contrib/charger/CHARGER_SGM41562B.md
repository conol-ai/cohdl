# SGM41562B — 500 mA single-cell Li-ion charger with power path, I²C, WLCSP-1.52×1.52-9B

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/charger::SGM41562B",
  "parts": ["@contrib/charger::CHARGER_SGM41562B"],
  "identities": [
    {"mfr": "SG Micro", "mpn": "SGM41562BXG/TR", "lcsc": "C5153801"}
  ],
  "sources": [
    {"id": "ds", "title": "SGM41562A/SGM41562B 500mA Single-Cell Li-Ion Battery Charger with Power Path Management", "revision": "REV. A.1, January 2022, 34 pages",
     "url": "https://www.sg-micro.com", "sha256": "e3fcb1f1662400261dfe60b6376fcc269c49ca0839f4d64a0cebb596835008fe", "local": "lib/@contrib/charger/docs/sgm41562a-sgm41562b.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed page numbers on pp. 2–31; p. 1 and the package pages 32–34 carry no printed number.

The source URL is only the manufacturer's home page. Neither `lib/@contrib/charger/docs/README.md` nor the PDF gives a document URL; the README says the PDF was retrieved through LCSC C5153801.

The datasheet covers SGM41562A and SGM41562B. The library models only the B model; this note gives B values and marks A-only values where they appear.

## Summary

The SGM41562B is an I²C-programmable charger for one Li-ion or Li-polymer cell with power-path management: the system rail SYS runs from IN when input power is present and from BAT otherwise (ds p. 1). Charge current is set in register ICC from 8 mA to 456 mA; there is no current-set resistor (ds p. 21). The B model stops charging and enters over-voltage protection above about 6 V input, so it suits a 5 V USB input only (ds p. 1, p. 4); the A model charges up to 18 V. **It does not charge without a host: after power-on and after every watchdog expiry the device is in default mode with CEB = 1, charge disabled** (ds p. 17, p. 20). Its I²C address is 03H and cannot be changed by pins (ds p. 20, p. 25). Charging parameters are guaranteed only from 0 °C to +55 °C (ds p. 1).

## Key parameters

TA = +25 °C, VIN = 5 V, VBAT = 3.5 V unless stated (ds p. 4). The source gives no limits over temperature.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Input UVLO threshold VIN_UVLO | 3.44 | 3.65 | 3.88 | V | Input falling | ds p. 4 |
| VIN_UVLO hysteresis | — | 105 | — | mV | Input rising | ds p. 4 |
| Input OVP threshold VIN_OVLO, SGM41562B | 5.75 | 6 | 6.27 | V | Input rising | ds p. 4 |
| Input OVP threshold VIN_OVLO, SGM41562A | 18 | 19 | 21 | V | Input rising (A model only) | ds p. 4 |
| Input clamp voltage | 19.5 | 21 | — | V | 1.5 mA clamp current | ds p. 4 |
| BAT pin input voltage VBAT | — | — | 4.5 | V | — | ds p. 4 |
| Regulated SYS voltage | 4.15 | 4.20 | 4.25 | V | VIN = 5.5 V, VSYS_REG[3:0] = 0000 | ds p. 4 |
| Input current limit IIN_LIM | 320 | 500 | 620 | mA | IIN_LIM[3:0] = 1111 (default) | ds p. 4, p. 20 |
| Input current limit setting | 50 | — | 500 | mA | 50 mA + 30 mA × n, n = IIN_LIM[3:0] | ds p. 20 |
| Input minimum voltage regulation VIN_MIN | 4.27 | 4.60 | 4.96 | V | VIN_MIN[3:0] = 1001 (default) | ds p. 4, p. 20 |
| IN to SYS switch on-resistance | — | 235 | — | mΩ | VIN = 4.5 V, ISYS = 100 mA | ds p. 4 |
| Battery FET on-resistance | — | 100 | — | mΩ | VIN < 2 V, ISYS = 100 mA | ds p. 5 |
| Input quiescent current | — | 80 | 100 | µA | VIN = 5.5 V, charge enabled or disabled, no load | ds p. 4 |
| Battery quiescent current, shipping mode | — | 0.7 | 1.2 | µA | VBAT = 4.5 V, IN open or grounded | ds p. 5 |
| Battery quiescent current, discharge | — | 10 | 70 | µA | VIN = GND, NTC drive disabled, conditions on ds p. 5 | ds p. 5 |
| Charge regulation voltage VBAT_REG | 4.175 | 4.200 | 4.225 | V | VBAT_REG[5:0] = 101000 (default) | ds p. 5, p. 22 |
| VBAT_REG setting | 3.60 | — | 4.545 | V | 3.6 V + 15 mV × n, n = VBAT_REG[5:0] | ds p. 22 |
| Charge current ICC | 80 | 96 | 110 | mA | ICC[5:0] = 001100 | ds p. 5 |
| Charge current ICC | 375 | 456 | 530 | mA | ICC[5:0] = 111000 (maximum code) | ds p. 5 |
| Charge current setting | 8 | — | 456 | mA | 8 mA + 8 mA × n, n = ICC[5:0] ≤ 56; default 128 mA; CC_FINE = 1 scales by ¼ | ds p. 21, p. 25 |
| Termination current ITERM | 1.8 | 3 | 4 | mA | ITERM[3:0] = 0001 (default); IPRE = ITERM | ds p. 6, p. 14, p. 21 |
| Battery discharge current limit IDSCHG | 400 | — | 3200 | mA | 200 mA + 200 mA × n, n ≠ 0; default 2000 mA | ds p. 21 |
| Pre-charge to fast-charge threshold | 2.9 | 3 | 3.1 | V | VBAT rising | ds p. 6 |
| Thermal regulation TJ_REG setting | 60 | — | 120 | °C | I²C programmable range; default 120 °C | ds p. 5, p. 16 |
| Thermal shutdown | — | 150 | — | °C | Hysteresis 20 °C | ds p. 6 |
| NTC cold threshold VCOLD | 63 | 65 | 67 | % of VDD | Rising | ds p. 6 |
| NTC hot threshold VHOT | 31 | 33 | 35 | % of VDD | Falling | ds p. 6 |
| I²C input low VIL | — | — | 0.4 | V | SDA, SCL | ds p. 6 |
| I²C input high VIH | 1.4 | — | — | V | SDA, SCL | ds p. 6 |
| SDA VOL | — | — | 0.2 | V | ISINK = 5 mA | ds p. 6 |
| I²C clock fSCL | — | — | 400 | kHz | — | ds p. 6 |
| nINT output pulse | — | 250 | — | µs | Per event | ds p. 4 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| IN | −0.3 to 21 | V | ds p. 2 |
| SYS | −0.3 to 5.3 (5.5 for 500 µs) | V | ds p. 2 |
| All other pins to GND | −0.3 to 6 | V | ds p. 2 |
| IINCLAMP | 5 | mA | ds p. 2 |
| θJA, WLCSP-1.52×1.52-9B | 95 | °C/W | ds p. 2 |
| Junction temperature | +150 | °C | ds p. 2 |
| Storage temperature | −65 to +150 | °C | ds p. 2 |
| Lead temperature (soldering, 10 s) | +260 | °C | ds p. 2 |
| ESD, HBM | 3000 | V | ds p. 2 |
| ESD, CDM | 1000 | V | ds p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| VIN, SGM41562B, charging | 4.35 | 5.5 | V | ds p. 2 |
| VIN, SGM41562A, charging (other model) | 4.35 | 18 | V | ds p. 2 |
| VIN, over-voltage protection state, continuous | — | 19 | V | ds p. 2 |
| IIN | — | 500 | mA | ds p. 2 |
| IBAT | — | 3.2 | A | ds p. 2 |
| ICHG | — | 456 | mA | ds p. 2 |
| VBAT_REG | — | 4.545 | V | ds p. 2 |
| Operating junction temperature | −40 | +125 | °C | ds p. 2 |
| Ambient for guaranteed charging parameters | 0 | +55 | °C | ds p. 1 |

## Pins

Top view, ball A1 at the upper left; row A along the top, column 1 at the left (ds p. 3, Pin Configuration).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | IN | IN | power in | Input power; ≥ 2.2 µF ceramic to GND, close | ds p. 3 |
| A2 | SYS | SYS | power out | System supply output; ceramic capacitor to GND | ds p. 3 |
| A3 | BAT | BAT | power in/out | Battery positive terminal; ceramic capacitor to GND | ds p. 3 |
| B1 | NTC | NTC | analog I/O | Battery thermistor to GND, RT1/RT2 divider from VDD | ds p. 3, p. 30 |
| B2 | NINT | nINT | digital I/O | Interrupt pulse output; pull low > tRST_DGL to recycle SYS or to leave shipping mode | ds p. 3, p. 18, p. 19 |
| B3 | VDD | VDD | power out | Internal supply; ≥ 0.1 µF ceramic; external load ≤ 1 mA | ds p. 3 |
| C1 | SDA | SDA | digital I/O | I²C data; 10 kΩ pull-up | ds p. 3 |
| C2 | SCL | SCL | digital in | I²C clock; 10 kΩ pull-up | ds p. 3 |
| C3 | GND | GND | ground | Device ground | ds p. 3 |

Checked by hand against the rendered Pin Configuration (ds p. 3): each library ball carries the signal shown. `BGA9C50P3X3_152X152X60N` puts A1 at (−0.5, −0.5) mm with row A along the top and column 1 on the left, which is the p. 3 top view. The p. 32 bottom view shows column 1 on the right, the expected mirror.

## Typical application

- Figure 13 / Table 16 BOM (ds p. 31): C1 = 4.7 µF on IN, C2 = 10 µF on SYS, C3 = 1 µF on VDD, C4 = 4.7 µF on BAT, all 16 V X5R or X7R 0603. Figure 1 shows the same values (ds p. 1).
- Minimums from the text: IN ≥ 2.2 µF; SYS ceramic > 2.2 µF; BAT ≥ 2.2 µF; VDD 0.1 µF (ds p. 3, p. 30).
- I²C: 10 kΩ pull-ups on SDA and SCL (ds p. 3). Address 03H (ds p. 20); REG0A ADDR[2:0] is read-only and reads 011 = 03H by default (ds p. 25).
- Charge current is a register value, not a resistor: ICC = 8 mA + 8 mA × ICC[5:0], codes above 56 clamp to 456 mA, default 128 mA (ds p. 21). Input current limit 50 mA + 30 mA × IIN_LIM[3:0], default 500 mA (ds p. 20).
- To charge, the host must write CEB = 0 (REG01 D[3]) and keep writing within the watchdog period, or set WATCHDOG[1:0] = 00 (ds p. 17, p. 20, p. 22). Defaults: WATCHDOG 160 s, safety timer 5 h, termination enabled (ds p. 22).
- NTC: thermistor from NTC to GND, RT1 from VDD to NTC and RT2 from NTC to GND; RT1 and RT2 from Equations 1 and 2. The example is RT1 = 7.6 kΩ, RT2 = 29.33 kΩ for a 10 kΩ, β = 3260 thermistor and a 0 °C to 50 °C window (ds p. 30). The NTC function can be disabled with EN_NTC = 0 (ds p. 17).
- nINT: Figure 13 ties it to the host and to a push button to GND (ds p. 31). Holding it low for tRST_DGL (16 s default) recycles SYS for tRST_DUR (4 s default) (ds p. 3, p. 20); a 2 s low exits shipping mode (ds p. 5, p. 19).

## Layout notes

- Recommended land (ds p. 32): nine round lands, Φ 0.250–0.275 mm, 0.5 mm pitch both ways. `BGA9C50P3X3_152X152X60N` uses Φ 0.26 mm at ±0.5 mm, inside that range. Body D = E = 1.500–1.550 mm, height A 0.562–0.638 mm, ball Φd 0.296–0.336 mm (ds p. 32).
- Place all capacitors as close as possible to the device; route the I²C ground straight to the GND ball, apart from high-current returns (ds p. 30).
- SYS capacitor between SYS and GND with thick traces and a small loop (ds p. 30). VDD capacitor close with thick traces (ds p. 30).

## Pitfalls

- **No host, no charging.** Default mode has CEB = 1; "by default the battery will not be charged after a reset" (ds p. 17; REG01 default, ds p. 20). A watchdog expiry also returns to this state (ds p. 17).
- With a valid input the watchdog is set to 01 (40 s) and, if the host does not write, SYS is power-cycled once (ds p. 13, p. 27 Figure 9). A host that boots slowly from SYS can be reset by the charger.
- I²C address 03H (ds p. 20). The source does not discuss that this value lies in the range the I²C specification reserves; check the host driver accepts it.
- The B model enters OVP at 5.75–6.27 V (ds p. 4); do not feed it from a 9 V or 12 V adapter. IN tolerates 21 V absolute (ds p. 2).
- SYS absolute maximum is 5.3 V (ds p. 2). Do not back-drive SYS from another 5 V rail.
- Charge current accuracy is wide: 375–530 mA at the 456 mA code (ds p. 5).
- Ambient for guaranteed charging parameters is 0 °C to +55 °C (ds p. 1).
- Shipping mode: after FET_DIS = 1 the battery is disconnected and only VIN or a 2 s nINT low wakes the device (ds p. 19). If nINT is used for wake-up, EN_SHIP_DGL[1:0] must stay at its default (ds p. 19).

## Not established by the sources

Disagreements between the library and the source:

- **"Standalone defaults" comment.** The device comment on SDA/SCL says "Optional: the charger runs on standalone defaults without a host (p1 feature set)". The source says the power-on default is CEB = 1, charge disabled, and that the battery is not charged after a reset (ds p. 17, p. 20). The p. 1 feature "Fully Autonomous Charger" does not override this. SDA and SCL are marked `optional`, so a design without I²C passes the library's checks but never charges the battery.
- **nINT output type.** The source types nINT as DIO and does not say whether the output is open-drain or push-pull, or whether it needs a pull-up (ds p. 3, p. 18). Figure 13 shows no pull-up (ds p. 31). The library types it `bidirectional`, which is consistent but does not settle the question.
- **VDD capacitor.** The library comment cites the 0.1 µF minimum (ds p. 3); the application circuit and BOM use 1 µF (ds p. 1, p. 31).
- **Land diameter.** The library uses a 0.26 mm midpoint of the 0.250–0.275 mm range; the source gives no nominal (ds p. 32). The comment's "vector-measured drawn pads: 0.267 mm" is not a value the source states.
- **Courtyard.** The 2.0 mm square courtyard is the library's choice; the source prints none.

Not stated by the source:

- A document URL; only the manufacturer home page.
- Limits over temperature for any electrical parameter (every table is at +25 °C, ds pp. 4–6).
- Whether nINT needs an external pull-up, and its sink capability.
- The physical meaning of the I²C address (03H is given as the "I2C Slave Address", ds p. 20, without saying 7-bit or 8-bit form).
- Thermal derating or the test board behind θJA = 95 °C/W (ds p. 2).
