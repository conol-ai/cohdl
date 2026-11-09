# TPS259823ONRGET — 2.7–24 V, 15 A, 2.7 mΩ eFuse, circuit breaker, 7.6 V OVLO, RGE0024M VQFN

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@ti/power-switch::TPS25982",
  "parts": ["@ti/power-switch::EFUSE_TPS259823ONRGET"],
  "identities": [
    {"mfr": "Texas Instruments", "mpn": "TPS259823ONRGET", "lcsc": "C2155764"}
  ],
  "sources": [
    {"id": "ds", "title": "TPS25982 2.7V to 24V, 2.7mΩ, 15A Smart eFuse - Integrated Hot-Swap Protection With 1.5% Accurate Load Current Monitoring and Adjustable Transient Fault Management", "revision": "SLVSEI3D, revised May 2026 (package option addendum 2-Jul-2026; RGE0024M drawing 4223975/B)",
     "url": "https://www.ti.com/lit/ds/symlink/tps25982.pdf", "sha256": "655c21dbf6b91a3c98b7ab26cd3bf0020d3eb7d765f411c71030d22fa4b6bc1f", "local": "lib/@ti/power-switch/docs/tps25982.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages. Pages 1–48 equal the printed page numbers. Pages 49–58 are the package option addendum, packing information and the RGE0024M drawings (no printed page numbers).

## Summary

The TPS25982 is a 15 A eFuse with an integrated 2.7 mΩ FET, 2.7–24 V operating input and 30 V absolute maximum (ds p. 1, p. 7). It has an adjustable current limit (2–15 A), a fast trip at 2.1 × ILIM, an adjustable overcurrent blanking timer, a dVdt slew control, a load-current monitor, power good, a load handshake input and configurable retry (ds p. 1, p. 23, p. 26). TPS259823O is the circuit-breaker variant with a fixed 7.6 V typical overvoltage lockout (ds p. 4). It is meant for 5 V-class rails. Do not use it on a 12 V rail: it switches off above 7.39–7.76 V (ds p. 8). RGET is the 250-piece small reel of the RGE package (ds p. 49).

## Key parameters

-40 °C ≤ TJ ≤ 125 °C, VIN = 5 V for TPS259823x, VEN/UVLO = 2 V, RILIM = 1650 Ω, CdVdt open, OUT open, unless stated (ds p. 8).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| IN quiescent current IQ | — | 800 | 1200 | µA | VEN ≥ VUVLO(R) | ds p. 8 |
| IN shutdown current ISD | — | 204 | 300 | µA | VSD < VEN < VUVLO | ds p. 8 |
| IN shutdown current ISD | — | 3.67 | 20 | µA | VEN < VSD | ds p. 8 |
| IN undervoltage protection VUVP | 2.46 | 2.53 | 2.6 | V | VIN rising | ds p. 8 |
| IN undervoltage protection VUVP | 2.35 | 2.42 | 2.49 | V | VIN falling | ds p. 8 |
| Overvoltage threshold VOVP(R), TPS259823x | 7.39 | 7.6 | 7.76 | V | VIN rising | ds p. 8 |
| Overvoltage threshold VOVP(F), TPS259823x | 7.22 | 7.4 | 7.55 | V | VIN falling | ds p. 8 |
| Overvoltage response time, TPS259823x | — | 5 | — | µs | VIN > VOVLO(R) to VOUT falling | ds p. 10 |
| Current monitor gain GIMON | 238.6 | 246 | 253.4 | µA/A | 3 A ≤ IOUT ≤ min(15 A, ILIM), -40 °C ≤ TA ≤ 75 °C | ds p. 8 |
| Current limit ILIM | 4.75 | 4.98 | 5.23 | A | RILIM = 300 Ω, TJ = 25 °C | ds p. 9 |
| Current limit ILIM | 4.36 | 4.98 | 5.66 | A | RILIM = 300 Ω, TJ = -40 to 125 °C | ds p. 9 |
| Current limit ILIM | 7.23 | 8.13 | 9.07 | A | RILIM = 182 Ω, TJ = -40 to 125 °C | ds p. 9 |
| Current limit ILIM | 12.85 | 14.71 | 15.99 | A | RILIM = 100 Ω, TJ = -40 to 125 °C | ds p. 9 |
| Current limit ILIM | 1.53 | 2 | 2.43 | A | RILIM = 773 Ω, TJ = -40 to 125 °C | ds p. 9 |
| Current limit, ILIM pin open | — | 0 | — | A | RILIM open | ds p. 9 |
| Circuit-breaker threshold with ILIM shorted to GND | — | — | 20 | A | TJ = 25 °C, single-point failure | ds p. 9 |
| Fast-trip threshold ISC | — | 210 | — | % ILIM | — | ds p. 9 |
| Short-circuit response time | — | 400 | — | ns | IOUT > 3 × ILIM to VOUT off | ds p. 10 |
| ON resistance RON | — | 2.7 | 3.2 | mΩ | TJ = 25 °C, IOUT = 2 A | ds p. 9 |
| ON resistance RON | — | — | 4.5 | mΩ | TJ = -40 to 125 °C, IOUT = 2 A | ds p. 9 |
| EN/UVLO threshold VUVLO(R) | 1.18 | 1.2 | 1.23 | V | VEN rising | ds p. 9 |
| EN/UVLO threshold VUVLO(F) | 1.08 | 1.1 | 1.13 | V | VEN falling | ds p. 9 |
| EN/UVLO threshold for lowest shutdown current VSD | 0.59 | 0.8 | — | V | VEN falling | ds p. 9 |
| EN/UVLO leakage | — | — | 0.1 | µA | — | ds p. 9 |
| PG low voltage | — | 100 | — | mV | VIN ≥ 5 V, IPG ≤ 5 mA | ds p. 9 |
| PG leakage (asserted) | — | — | 1.7 | µA | Pulled up to 5 V through 10 kΩ | ds p. 9 |
| VIN − VOUT for PG de-assertion VPGTHD | 0.224 | 0.326 | 0.450 | V | — | ds p. 9 |
| PG de-glitch | — | 120 | — | µs | Assertion and de-assertion | ds p. 10 |
| RETRY_DLY / NRETRY bias current | 1.7 | 2.05 | 2.5 | µA | — | ds p. 9 |
| RETRY_DLY / NRETRY oscillator hysteresis | 0.65 | 0.75 | 0.85 | V | — | ds p. 9 |
| ITIMER discharge current IITIMER | 1.4 | 2.1 | 2.8 | µA | ISC > IOUT > ILIM | ds p. 9 |
| ITIMER comparator delta ΔVITIMER | 0.7 | 0.98 | 1.3 | V | ISC > IOUT > ILIM | ds p. 10 |
| ITIMER default voltage / pull-up | — | 2.5 / 23 | — | V / kΩ | IOUT < ILIM | ds p. 9 |
| LDSTRT rising threshold | 1.1 | 1.21 | 1.3 | V | — | ds p. 10 |
| LDSTRT charging current | 1.7 | 2.05 | 2.4 | µA | PG asserted | ds p. 10 |
| dVdt charging current | 2 | 4.6 | 6.33 | µA | — | ds p. 10 |
| Output slew rate, dVdt open / 3300 pF / 6800 pF | — | 7.35 / 1.4 / 0.68 | — | V/ms | VIN = 12 V, TJ = 25 °C, RL = 3.6 Ω, COUT = 1 mF | ds p. 10 |
| Thermal shutdown / hysteresis | — | 150 / 10 | — | °C | TJ rising / falling | ds p. 10 |
| RθJA / RθJB / RθJC(bot) | — | 34.6 / 11.2 / 1.6 | — | °C/W | Simulated, JEDEC 2s2p, minimum pad, 3×2 via array | ds p. 8 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| IN | -0.3 to 30 | V | ds p. 7 |
| OUT | -0.8 to min(30, VIN + 0.3) | V | ds p. 7 |
| EN/UVLO | -0.3 to 7 | V | ds p. 7 |
| LDSTRT | 7 (max) | V | ds p. 7 |
| PG | -0.3 to 7 | V | ds p. 7 |
| dVdt, ITIMER, NRETRY, RETRY_DLY | Internally limited | V | ds p. 7 |
| Continuous switch current, IN to OUT | Internally limited | A | ds p. 7 |
| Junction temperature | Internally limited | °C | ds p. 7 |
| Soldering temperature | 300 | °C | ds p. 7 |
| Storage temperature | -65 to 150 | °C | ds p. 7 |
| ESD, HBM (all pins) | ±2000 | V | ds p. 7 |
| ESD, CDM (all pins) | ±1000 | V | ds p. 7 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Input voltage VIN | 2.7 | 24 | V | ds p. 7 |
| Output voltage VOUT | — | VIN + 0.3 | V | ds p. 7 |
| EN/UVLO voltage | — | 6 | V | ds p. 7 |
| PG voltage | — | 6 | V | ds p. 7 |
| LDSTRT, ITIMER, NRETRY, RETRY_DLY capacitor voltage rating | 4 | — | V | ds p. 7 |
| dVdt capacitor voltage rating | VIN + 4 | — | V | ds p. 7 |
| ILIM resistor RILIM | 82 | 1650 | Ω | ds p. 7 |
| Continuous switch current, IN to OUT | — | 15 | A | ds p. 7 |
| Junction temperature | -15 | 125 | °C | ds p. 8 |

## Pins

Top view: pins 1–6 down the left side, 7–12 along the bottom, 13–18 up the right side, 19–24 along the top, pin-1 dot at the upper-left. Thermal pad 1 (IN) is the upper exposed pad, towards the OUT pins 19–24, and thermal pad 2 (GND) the lower one (ds p. 5). The library numbers thermal pad 1 as pad 25 and thermal pad 2 as pad 26, matching the drawing's labels (ds p. 55, p. 56).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1, 2, 3, 16 | IN | IN | power input | Power input | ds p. 5 |
| 25 | IN | IN (thermal pad 1) | power input | Exposed pad; solder uniformly to the input plane for heat and current sharing | ds p. 5, p. 55 |
| 4, 5, 14 | GND | GND | ground | Device ground | ds p. 5 |
| 26 | GND | GND (thermal pad 2) | ground | Exposed pad; device ground | ds p. 5, p. 55 |
| 6 | EN_UVLO | EN/UVLO | input | Active-high enable; divider from IN sets UVLO; do not leave floating | ds p. 5 |
| 7 | ITIMER | ITIMER | analog output | Capacitor to GND sets the overcurrent blanking time; open = fastest | ds p. 5 |
| 8 | ILIM | ILIM | analog output | Resistor to GND sets current limit and fast trip; do not leave floating | ds p. 5 |
| 9 | IMON | IMON | analog output | Current proportional to load current; resistor to GND gives a voltage | ds p. 5 |
| 10 | RETRY_DLY | RETRY_DLY | analog output | Capacitor to GND sets retry delay; GND = latch off | ds p. 5 |
| 11 | NRETRY | NRETRY | analog output | Capacitor to GND sets number of retries; GND = retry indefinitely | ds p. 5 |
| 12 | LDSTRT | LDSTRT | analog input | Load handshake; GND if unused; do not leave floating | ds p. 5 |
| 13 | PG | PG | digital output | Power good, active high, open drain; needs a pull-up | ds p. 6 |
| 15 | dVdt | dVdt | analog output | Capacitor to GND sets the turn-on slew rate; open = fastest | ds p. 6 |
| 17, 18, 19, 20, 21, 22, 23, 24 | OUT | OUT | power output | Power output | ds p. 5 |

## Typical application

- Current limit: RILIM (Ω) = 1460 / (ILIM (A) − 0.11), within 82–1650 Ω (ds p. 7, p. 23). Check the minimum ILIM over temperature against the load: 300 Ω gives 4.36 A minimum, 100 Ω gives 12.85 A minimum (ds p. 9, p. 35). UL 2367 recognition requires RILIM ≥ 182 Ω (ds p. 1).
- UVLO: VIN(UVLO) = VUVLO(R) × (RVL1 + RVL2) / RVL2 with VUVLO(R) = 1.2 V, and the divider current at least 20 × the EN leakage (ds p. 21, p. 35). Below 6 V supply EN may be tied straight to IN; above 6 V use a divider (ds p. 8).
- Slew rate: SR (V/ms) = IINRUSH (mA) / COUT (µF), and CdVdt (pF) = 4600 / SR (V/ms) (ds p. 22). The dVdt capacitor sees about VIN + 4 V, so allow for DC-bias loss (ds p. 7, p. 37). Check start-up dissipation against the thermal-shutdown plot (ds p. 36, p. 37).
- ITIMER (circuit breaker): tITIMER (ms) = CITIMER (nF) × ΔVITIMER (V) / IITIMER (µA) (ds p. 25; Eq. 5 on ds p. 24 misprints the denominator, see "Not established"); the design example uses CITIMER (nF) = tITIMER (ms) / 0.47, and 4.7 nF for about 2 ms (ds p. 38). TI recommends an ITIMER interval above 100 µs (ds p. 27). Limit CITIMER < 53000 × tGHI / (VIN + 3.6 V) (ds p. 24).
- Retry: RETRY_DLY to GND = latch off; both open = 4 retries at minimum delay; NRETRY to GND = retry indefinitely (ds p. 31). tRETRY_DLY (µs) = 128 × (CRETRY_DLY (pF) + 4 pF) × VRETRY_DLY_HYS (V) / IRETRY_DLY (µA) (ds p. 31). Example capacitor pairs are in Table 7-5, e.g. 2.2 nF on RETRY_DLY for 91.7 ms (ds p. 31).
- IMON: VIMON = GIMON × IOUT × RIMON. Keep VIMON(MAX) at 1 V for VIN = 2.7 V, 1.8 V for 3.3 V and 3.3 V above 5 V (ds p. 27, p. 28). A series RC filter of ≥ 10 kΩ is recommended (ds p. 28).
- LDSTRT: tie to GND if the handshake is not used (ds p. 5, p. 29). Otherwise tLDSTRT (ms) = CLDSTRT (nF) × VLDSTRT (V) / ILDSTRT (µA) (ds p. 29).
- PG: pull up through a resistor; to IN/OUT only when the supply is below 6 V, otherwise to a lower rail (ds p. 8). The application uses 100 kΩ to 3.3 V (ds p. 45).
- Input: a ceramic bypass of ≥ 0.01 µF between IN and GND (ds p. 46), larger than 0.1 µF if the source is more than a few inches away (ds p. 44). 0.001–0.1 µF damps transients (ds p. 45). Add a TVS at IN and a Schottky diode from GND to OUT where line inductance requires them (ds p. 45, p. 46).

## Layout notes

- RGE0024M land (ds p. 56): 24 lands 0.575 × 0.24 mm, R0.05 mm, 0.5 mm pitch, 3.825 mm between opposite land-row centres. Thermal land 25: 2.7 × 1.45 mm, centre 0.625 mm from the package centre towards pins 19–24. Thermal land 26: 2.7 × 0.85 mm, centre 0.925 mm towards pins 7–12. Non-solder-mask-defined openings with 0.07 mm maximum expansion are preferred.
- Thermal vias (ds p. 56): Ø0.2 mm at x = 0, ±1.1 mm. On land 25 they sit in two rows, 1.1 mm and 0.15 mm from the package centre towards pins 19–24. On land 26 one row lies 0.925 mm towards pins 7–12. Fill, plug or tent vias under paste (ds p. 56).
- Stencil, 0.125 mm thick (ds p. 57): two 1.188 × 1.3 mm apertures on land 25 and two 1.188 × 0.76 mm on land 26, centred at x = ±0.694 mm. That gives 78% coverage on pad 25.
- Connect the IN thermal pad to as much copper as possible with a via array. This also evens out the current through the FET and improves current-monitor accuracy (ds p. 46).
- Connect device GND to a signal-ground island joined to the system power ground at one point (ds p. 46).
- Keep the OUT path short and at least twice the full-load current rating; at least 50 mil wide (ds p. 46).
- Place the ILIM, IMON, ITIMER, RETRY_DLY, NRETRY and dVdt parts close to their pins. Keep ILIM parasitic capacitance below 30 pF (ds p. 46).
- Avoid a steady thermal gradient between the two thermal pads; the overtemperature protection relies on it (ds p. 46).

## Pitfalls

- **7.6 V OVLO.** The TPS259823O switches off at 7.39–7.76 V rising (ds p. 8). It cannot protect a 12 V rail; TPS259824x (16.9 V) or -27x (no OVLO) are the other options (ds p. 4).
- **Circuit breaker, not current limiter.** An overload beyond ILIM for longer than tITIMER turns the FET off; then the RETRY_DLY/NRETRY settings decide what happens (ds p. 23, p. 30, p. 31).
- **ILIM open = zero current limit.** The FET turns off as soon as any load current flows; ILIM shorted to GND is a fault (ds p. 23).
- **ITIMER shorted to GND is a fault** and shuts the part down (ds p. 24). Leave it open, or fit a capacitor.
- **EN/UVLO above 6 V.** Tying EN to IN is allowed only below 6 V (ds p. 8). With the 7.6 V OVLO the input can reach 7.39 V while the part is on (ds p. 8), above the 6 V EN limit (ds p. 7).
- **LDSTRT must not float.** Floating, it charges up and trips the LDSTRT fault after PG asserts (ds p. 5, p. 29, p. 30).
- **NRETRY quantisation.** The retry count is quantised to 4, 16, 64, 256 or 1024 (ds p. 31). The design example's 2.2 nF on both NRETRY and RETRY_DLY computes to N ≈ 3.99 by Eq. 15 (ds p. 31, p. 38), just below the 4/16 boundary, so capacitor tolerance can move it to 16 (inference from the equation).
- **PG with no input.** With no supply, PG has no active pull-down; a pull-up to another rail can make it read high (ds p. 29).
- **Fault latch reset** needs IN below VUVP(F) or EN/UVLO below VSD, or the auto-retry logic (ds p. 30).

## Not established by the sources

- **Thermal-via positions on pad 25 (library footprint).** `TI_RGE0024M` places the pad-25 vias at y = -1.175 mm and -0.075 mm, 1.1 mm apart and centred on the pad. The drawing dimensions them 1.1 mm and 0.15 mm from the package centre line, i.e. y = -1.1 mm and -0.15 mm (ds p. 56, checked on the rendered page). The library's Ø0.4 mm via lands therefore reach 0.025 mm past both long edges of the 1.45 mm IN land (y -1.35 to +0.10 mm). The package README calls these "TI's exact 1.10mm grid". The x positions and the pad-26 row match.
- **Ground connection.** `lib/@ti/power-switch/docs/README.md` says to join pins 4, 5, 14 and pad 26 "to the ground plane". The datasheet recommends a signal-ground island joined to the system power ground at one point (ds p. 46).
- **IMON open.** The library declares IMON `optional`. The source describes only a resistor to GND (ds p. 5, p. 27) and does not say IMON may be left open.
- **Pin types the library cannot express.** PG is an open-drain output (ds p. 6); the library types it `output`. ITIMER, ILIM, RETRY_DLY, NRETRY and dVdt are "Analog Output" pins that take a resistor or capacitor to GND (ds p. 5, p. 6).
- **Junction temperature range.** Rev. D changed the recommended minimum TJ to -15 °C (ds p. 8, p. 48). The electrical table is still specified from -40 °C (ds p. 8). The addendum gives -40 to 125 °C for TPS259823ONRGET and -15 to 125 °C for TPS259823ONRGET.A (ds p. 49).
- **Equation 5.** The circuit-breaker blanking equation prints the denominator as "RILIM(Ω)" (ds p. 24, checked on the rendered page). The matching current-limiter equation uses IITIMER in µA (ds p. 25), and the design example's 2 ms with 4.7 nF agrees with IITIMER (ds p. 38). The note uses the IITIMER form.
- **Retry delay, equation against table.** Eq. 14 with typical values gives about 103 ms for 2.2 nF (128 × 2204 pF × 0.75 V / 2.05 µA, ds p. 9, p. 31), and Eq. 31 in the design example gives the same (ds p. 38). Table 7-5 lists 91.7 ms for 2.2 nF (ds p. 31). The source does not explain the difference.
- **Thermal data.** θJA is from simulation on a JEDEC 2s2p board with the minimum pad and a 3 × 2 via array (ds p. 8), not from a measured board.
- **Via annulus and courtyard.** TI gives only the Ø0.2 mm drill (ds p. 56). The library's 0.4 mm via land and 4.9 mm courtyard are its own choices, as its comments say.
- **Not given:** any reverse-current behaviour (the source does not mention it), current-limit accuracy below 2 A, and IMON accuracy below 3 A.
