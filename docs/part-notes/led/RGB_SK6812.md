# SK6812MINI-E-012 — addressable RGB LED with integrated driver IC, 3.2 x 2.8 x 1.78 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "led::SK6812",
  "parts": ["led::RGB_SK6812"],
  "identities": [
    {"mfr": "Dongguan OPSCO Optoelectronics", "mpn": "SK6812MINI-E-012", "lcsc": "C22466970"}
  ],
  "sources": [
    {"id": "ds", "title": "OPSCO product specification SK6812MINI-E-012", "revision": "File SK6812MINI-E-012, header Rev. B/1, cover Version B/2, 2025-11-22",
     "url": "https://www.opscoled.com/en/download/index.html", "sha256": "d78c60d7a163c7fdb69db78a2fccf0faa8104532fdee0b0472736aaeac427cc7", "local": "lib/led/docs/opsco-sk6812mini-e.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "N / 13".

## Summary

RGB LED with a built-in driver IC, cascaded on a single-wire return-to-zero data line at up to
800 kbps, 24-bit GRB per pixel, 256 grey levels per colour (ds p. 3, p. 8). Supply is 3.7 V to
5.5 V, 5.0 V typ (ds p. 6); each colour channel sinks 12 mA typ (ds p. 6). The 12 mA version
may run one colour at 100 % grey but only 70 % grey with all three lit (ds p. 3). DIN switches at
0.65 x VDD (ds p. 6), so a 3.3 V MCU driving a 5 V string has no guaranteed margin. MSL 5a
(ds p. 3).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage VDD | 3.7 | 5.0 | 5.5 | V | TA = 25 °C | ds p. 6 |
| Input high threshold VIH | 0.65 x VDD | — | — | V | VDD = 5.0 V | ds p. 6 |
| Input low threshold VIL | — | — | 0.3 x VDD | V | VDD = 5.0 V | ds p. 6 |
| R/G/B output drive current IDOUT | 10 | 12 | 14.5 | mA | VDD = 5.0 V, VDS = 5.0 V | ds p. 6 |
| PWM frequency FPWM | — | 1.0 | — | kHz | IOUT = 12 mA, OUT via 200 Ω to VDD | ds p. 6 |
| Static current IDD | — | 0.6 | — | mA | VDD = 4.5 V, outputs off | ds p. 6 |
| Data rate FDIN | — | 800 | — | kbps | — | ds p. 6 |
| Symbol period T | 1.20 | — | — | µs | — | ds p. 7 |
| 0-code high time T0H | 0.25 | 0.30 | 0.40 | µs | — | ds p. 7 |
| 0-code low time T0L | 0.80 | 0.90 | — | µs | low time < 20 µs (note 3) | ds p. 7 |
| 1-code high time T1H | 0.70 | 0.90 | 1.00 | µs | — | ds p. 7 |
| 1-code low time T1L | 0.2 | 0.30 | — | µs | low time < 20 µs (note 3) | ds p. 7 |
| Reset (latch) low time | > 200 | — | — | µs | — | ds p. 7 |
| Red dominant wavelength / intensity | 620 / 160 | — | 625 / 320 | nm / mcd | 12 mA version; ±1.0 nm, ±10 % | ds p. 6 |
| Green dominant wavelength / intensity | 515 / 815 | — | 525 / 1275 | nm / mcd | 12 mA version | ds p. 6 |
| Blue dominant wavelength / intensity | 460 / 120 | — | 470 / 240 | nm / mcd | 12 mA version | ds p. 6 |
| Viewing angle (50 %) | — | 120 | — | ° | — | ds p. 3 |
| Package L x W x H | — | 3.2 x 2.8 x 1.78 | — | mm | ±0.1 mm | ds p. 4 |
| Lead span / lead length / lead width | — | 5.88 / 1.34 / 0.68 | — | mm | ±0.1 mm | ds p. 4 |
| Moisture sensitivity level | — | 5a | — | — | — | ds p. 3 |

## Absolute maximum ratings

The source has no table titled "absolute maximum". Its "IC limit parameter" table (Ta = 25 °C)
is the closest; its supply row is labelled "Working voltage" and repeats the operating range.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Logic supply VDD ("Working voltage" in the limit table) | +3.7 to +5.5 | V | ds p. 5 |
| Storage temperature Tstg | -40 to +80 | °C | ds p. 5 |
| ESD withstand VESD, human body model | 2000 | V | ds p. 5 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD | 3.7 | 5.5 | V | ds p. 6 |
| Operating temperature Topt | -40 | +80 | °C | ds p. 5 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | GND | GND | Ground | Power ground; chamfered ("C") corner | ds p. 4, p. 5 |
| 2 | DIN | DIN | Input | Control data input | ds p. 5 |
| 3 | VDD | VDD | Power | Supply | ds p. 5 |
| 4 | DOUT | DOUT | Output | Control data output, to next DIN | ds p. 5, p. 8 |

## Typical application

- Chain DOUT of each LED to DIN of the next; the MCU drives only the first DIN, and each stage
  reshapes and forwards the data (ds p. 7, p. 8).
- Send 24 bits per LED, MSB first, in G-R-B order; then hold the line low > 200 µs to latch
  (ds p. 7, p. 8).
- Put a series protection resistor at the signal input and at the output end, about 500 Ω each
  (R1, R2) for short strip-to-strip runs, tuned by test (ds p. 9).
- A decoupling capacitor C1 at every LED is "essential" (ds p. 9); no value is given.
- At 30 frames/s keep the chain under 1024 LEDs (ds p. 3).

## Layout notes

- Recommended lands (ds p. 5): four pads 1.8 x 0.82 mm, 0.68 mm gap between the two pads of a
  side; the drawing shows pads abutting a body outline but does not dimension the overall span.
- The library pad is 1.35 x 0.82 mm, pulled back from the aperture side, on a 6.80 mm outer span
  with a 3.4 x 3.0 mm routed window (`lib/led/src/sk6812.cohdl`); none of these three values is
  dimensioned in the source.
- No PCB aperture, keep-out or orientation of mounting is given in the source.

## Pitfalls

- Grey-level limit: for the 12 mA version, use at most 70 % grey when all three colours are lit;
  100 % is allowed only for a single colour (ds p. 3). The source gives no reason (thermal is a
  guess), so firmware must enforce it.
- Logic level: VIH = 0.65 x VDD = 3.25 V at VDD = 5.0 V (ds p. 6). A 3.3 V output has no
  guaranteed margin; either level-shift DIN or run VDD lower within 3.7–5.5 V (inference).
- Timing: the high time decides the bit; T0H must stay 0.25–0.40 µs and T1H 0.70–1.00 µs
  (ds p. 7). Any low time over 20 µs is outside the stated range for data, and > 200 µs is a
  reset (ds p. 7).
- MSL 5a (ds p. 3): bake or reflow promptly after opening; the source gives no floor-life time.
- Storage temperature max is only +80 °C (ds p. 5); reliability tests used 260 °C, 10 s, two
  passes for soldering heat (ds p. 12).
- No-light at power-up is intended: the built-in reset keeps LEDs off until data arrives (ds p. 3).

## Not established by the sources

- **Mounting direction and land-pattern orientation.** The source never says "reverse mount".
  Its p. 4 drawing has an unlabelled view showing the R/G/B dice (pads, clockwise from lower
  right: 1 GND, 4 DOUT at lower left, 3 VDD at upper left, 2 DIN at upper right) and a "BOTTOM
  VIEW" that is its mirror image. The p. 5 recommended land drawing uses the die-side
  arrangement. The library footprint (y-down: pads 1 and 2 east at y = –0.75/+0.75, pads 4 and 3
  west) is the p. 4 BOTTOM VIEW turned 180°, so it is the **mirror image of the p. 5 land
  drawing**. That is right only if the part is mounted die-down through a board cutout and p. 5
  is drawn for die-up mounting; the source states neither. The library cites hardware (built v23)
  for its choice; that evidence is not in the source.
- Land geometry: the library shortens the land to 1.35 mm (source: 1.8 mm, ds p. 5) and sets a
  6.80 mm outer span; the 6.80 mm figure is 3.2 mm body + 2 x 1.8 mm, read from the drawing, not
  a dimension. Against the 5.88 mm lead span (ds p. 4), the library land runs 0.46 mm past each
  lead tip and leaves the inner 0.45 mm of each 1.34 mm lead off copper (arithmetic from
  ds p. 4 and the library coordinates).
- The 3.4 x 3.0 mm board window has no source; the library says it is KiCad's convention.
- A true absolute maximum supply voltage (above 5.5 V), maximum junction temperature, thermal
  resistance, DIN/DOUT current limits and DOUT output levels.
- Decoupling capacitor value (ds p. 9 asks for one but gives none) and supply current with LEDs on.
- Floor life for MSL 5a and a reflow profile (only the reliability test condition, ds p. 12).
- Revision label: the page headers say "Rev. No.: B/1" while the cover says "Version B/2"
  and the revision record lists B/2, 2025-11-22 (ds p. 1, p. 13).
- Source URL: OPSCO's portal gives no direct file URL; the local copy came from LCSC record
  C5149201 (`lib/led/docs/README.md`), while `lcsc_map.json` maps this MPN to C22466970
  (rule "alias"). Which LCSC number ships this exact document is not established.
