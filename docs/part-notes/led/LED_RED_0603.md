# LTST-C193KRKT-5A — red AlInGaP chip LED, 1.6 x 0.8 mm, 0.35 mm high

## Identity

```json
{
  "schema": "part-note/1",
  "device": "led::ChipLED",
  "parts": ["led::LED_RED_0603"],
  "identities": [
    {"mfr": "Lite-On", "mpn": "LTST-C193KRKT-5A", "lcsc": "C157741"}
  ],
  "sources": [
    {"id": "ds", "title": "SMD LED Product Data Sheet LTST-C193KRKT-5A", "revision": "Spec No. DS22-2005-077 Rev. D, effective 2021-10-22",
     "url": "https://optoelectronics.liteon.com/upload/download/DS22-2005-077/LTST-C193KRKT-5A.PDF", "sha256": "a735509f0638262d6d43851e13f42ca05bf37c70e9afedbf3173f71591e392ac", "local": "lib/led/docs/liteon-ltst-c193krkt-5a.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages (1-based). PDF p. 1 is a cover, so the printed "Page N of 12"
is PDF page N + 1. PDF p. 14 is a Mouser distributor page, not Lite-On content.

## Summary

Water-clear, red (AlInGaP) chip LED in a 1.60 x 0.80 mm EIA package only 0.35 mm high (ds p. 2).
It is characterised at 5 mA: 7.1 to 45.0 mcd and VF 1.70 to 2.30 V (ds p. 5), with a 20 mA DC
maximum (ds p. 3). Use it as a low-current status indicator. It is not a high-brightness part, and
its 5 V reverse rating (ds p. 3) means it must not see reverse bias in a matrix or AC drive.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Luminous intensity IV | 7.1 | — | 45.0 | mcd | IF = 5 mA, TA = 25 °C | ds p. 5 |
| Viewing angle 2θ1/2 | — | 130 | — | deg | TA = 25 °C | ds p. 5 |
| Peak emission wavelength λP | — | 639 | — | nm | TA = 25 °C | ds p. 5 |
| Dominant wavelength λd | — | 631 | — | nm | IF = 5 mA | ds p. 5 |
| Spectral line half-width Δλ | — | 20 | — | nm | TA = 25 °C | ds p. 5 |
| Forward voltage VF | 1.70 | — | 2.30 | V | IF = 5 mA | ds p. 5 |
| Reverse current IR | — | — | 10 | µA | VR = 5 V | ds p. 5 |
| VF bins E2 / E3 / E4 | 1.70 / 1.90 / 2.10 | — | 1.90 / 2.10 / 2.30 | V | IF = 5 mA, ±0.1 V per bin | ds p. 6 |
| IV bins K / L / M / N | 7.1 / 11.2 / 18.0 / 28.0 | — | 11.2 / 18.0 / 28.0 / 45.0 | mcd | IF = 5 mA, ±15 % per bin | ds p. 6 |
| Package length x width x height | — | 1.60 x 0.80 x 0.35 | — | mm | ±0.10 mm unless noted | ds p. 2 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Power dissipation (TA = 25 °C) | 50 | mW | ds p. 3 |
| Peak forward current (1/10 duty, 0.1 ms pulse) | 40 | mA | ds p. 3 |
| DC forward current | 20 | mA | ds p. 3 |
| Reverse voltage | 5 | V | ds p. 3 |
| Storage temperature | -40 to +85 | °C | ds p. 3 |
| Infrared soldering | 260 °C for 10 s | — | ds p. 3 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature (listed in the absolute maximum table) | -30 | +85 | °C | ds p. 3 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | Cathode | CATHODE (marked terminal) | passive | Cathode; the drawing marks one soldering terminal "CATHODE" | ds p. 2 |
| 2 | Anode | (unmarked terminal) | passive | Anode | ds p. 2 |

## Typical application

- Drive from a voltage source through a series resistor, one resistor per LED; Lite-On calls this
  the recommended circuit and warns that parallel LEDs sharing a resistor differ in brightness
  (ds p. 11).
- Size the resistor for the 5 mA characterisation point and VF up to 2.30 V (ds p. 5); stay
  at or below 20 mA DC (ds p. 3).

## Layout notes

- Suggested lands: two 0.8 x 0.8 mm pads, 2.3 mm overall span (0.7 mm gap); stencil at most
  0.10 mm thick (ds p. 8). `FP_LED_0603_1608Metric` places 0.8 mm pads at ±0.75 mm, which gives
  the same 2.3 mm span.
- The suggested land drawing does not say which pad is the cathode (ds p. 8); polarity comes only
  from the package drawing (ds p. 2) and the tape drawing (ds p. 8).
- Reflow: preheat 150–200 °C up to 120 s, peak 260 °C max, 10 s max, at most two passes; hand
  soldering 300 °C max for 3 s, once (ds p. 10).

## Pitfalls

- 20 mA is the DC maximum at 25 °C (ds p. 3); the source gives no derating figure in text, so do
  not run at 20 mA in a hot enclosure without checking the curves on ds p. 7.
- Moisture: once the bag is opened, reflow within 672 h at ≤ 30 °C / 60 % RH, or bake at about
  60 °C for at least 20 h (ds p. 10).
- ESD-sensitive: handle with grounding (ds p. 5, p. 11). A damaged AlInGaP LED shows VF below
  1.4 V at 0.1 mA (ds p. 11).
- Reels hold 5,000 pieces (ds p. 9).

## Not established by the sources

- Pad numbering. The datasheet never numbers the terminals (ds p. 2, p. 8). "Pad 1 = Cathode"
  is the library's chip-LED convention (`lib/led/src/chip.cohdl`), not a Lite-On statement. The
  library's polarity agrees with the drawing only if pad 1 is placed on the marked terminal.
- Which face the p. 2 package drawing shows: the left view shows the soldering terminals and the
  cathode mark, so it reads as a bottom view, but it is not labelled (ds p. 2).
- "0603": the source says "EIA STD package" and gives 1.60 x 0.80 mm (ds p. 2); it does not use the
  name 0603, which is the library's part name.
- A recommended operating current, a derating rule in text, thermal resistance and maximum
  junction temperature. Only the absolute maximum table and typical curves exist (ds pp. 3, 7).
- Typical VF and typical IV: the table gives only min and max (ds p. 5).
- Source URL: the repository records the official Lite-On URL, but the local file was retrieved from
  Mouser (`lib/led/docs/README.md`) and includes a Mouser page (ds p. 14). The SHA-256 above is of
  the Mouser copy, not proven equal to the file at the Lite-On URL.
