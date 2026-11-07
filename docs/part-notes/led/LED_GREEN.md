# 150060GS75000 — green InGaN chip LED, 0603 (WL-SMCW), water clear

## Identity

```json
{
  "schema": "part-note/1",
  "device": "led::ChipLED",
  "parts": ["led::LED_GREEN"],
  "identities": [
    {"mfr": "Wurth Elektronik", "mpn": "150060GS75000", "lcsc": "C5252984"}
  ],
  "sources": [
    {"id": "ds", "title": "WL-SMCW SMT Mono-color Chip LED Waterclear 150060GS75000", "revision": "Rev. 002.009, 2019-02-26",
     "url": "https://www.we-online.com/components/products/datasheet/150060GS75000.pdf", "sha256": "4e12ec7220fb4cb46c61b9f837bef372d82709fbba410caaf7aecae204abd60a", "local": "lib/led/docs/wurth-150060gs75000.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "PAGE N/8".

## Summary

Green (InGaN, 525 nm dominant) water-clear chip LED in a 1.6 x 0.8 mm "0603" body (ds p. 1,
p. 2). It is characterised at 20 mA: 220 mcd min, 430 mcd typ, VF 3.2 V typ / 3.5 V max
(ds p. 2), and is rated 30 mA continuous (ds p. 1). Choose it for a bright status LED. On a
3.3 V rail there is almost no headroom over VF (3.2 V typ, 3.5 V max at 20 mA, ds p. 2), so
the current there is set more by VF spread than by the resistor (inference). It is MSL 3
(ds p. 1).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Peak wavelength λPeak | — | 515 | — | nm | IF = 20 mA | ds p. 2 |
| Dominant wavelength λDom | — | 525 | — | nm | IF = 20 mA | ds p. 2 |
| Luminous intensity IV | 220 | 430 | — | mcd | IF = 20 mA | ds p. 2 |
| Forward voltage VF | — | 3.2 | 3.5 | V | IF = 20 mA | ds p. 2 |
| Spectral bandwidth Δλ | — | 30 | — | nm | IF = 20 mA | ds p. 2 |
| Reverse current IREV | — | — | 10 | µA | VR = 5 V | ds p. 2 |
| Viewing angle 2θ50% | — | 140 | — | ° | IF = 20 mA | ds p. 2 |
| Body length x width | — | 1.6 x 0.8 | — | mm | ±0.1 mm | ds p. 1 |
| Moisture sensitivity level | — | 3 | — | — | — | ds p. 1 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Power dissipation PDiss (ambient 25 °C) | 105 | mW | ds p. 1 |
| Peak forward current IF Peak (duty 1/10 at 1 kHz) | 100 | mA | ds p. 1 |
| Continuous forward current IF | 30 | mA | ds p. 1 |
| Reverse voltage VREV | 5 | V | ds p. 1 |
| ESD threshold, human body model | 1000 | V | ds p. 1 |
| Junction temperature TJ | 95 | °C | ds p. 1 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature (from "General Information") | -40 | +85 | °C | ds p. 1 |
| Storage, single parts | -40 | +85 | °C | ds p. 1 |
| Storage in original packaging | — | < 40 °C, < 90 % RH | — | ds p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | Cathode | 1 (–), polarity-mark end | passive | Cathode | ds p. 1 |
| 2 | Anode | 2 (+) | passive | Anode | ds p. 1 |

## Typical application

- A series current-limiting resistor is mandatory: "Resistors for protection are obligatory"
  (ds p. 7).
- Size it for VF up to 3.5 V at 20 mA (ds p. 2) and stay at or below 30 mA continuous (ds p. 1),
  derated per the curve on ds p. 4.

## Layout notes

- Recommended land: two 0.8 x 0.8 mm pads, 0.8 mm gap, 2.4 mm overall span; pad 1 on the left
  of the drawing (ds p. 1). `FP_LED_0603_1608Metric_WURTH` places 0.8 mm pads at ±0.8 mm, which
  matches.
- The Lite-On land (2.3 mm span) is a different footprint; do not swap parts between the two
  footprints without checking (ds p. 1; `lib/led/src/chip.cohdl`).
- Reflow per J-STD-020E: peak 260 °C for packages < 1.6 mm thick, 10 s max within 5 °C of
  peak, two cycles max (ds p. 6).

## Pitfalls

- MSL 3 (ds p. 1): handle per IPC/JEDEC J-STD-033 once the bag is opened (ds p. 7).
- Junction limit is only 95 °C (ds p. 1); the 30 mA rating is at 25 °C ambient and the
  derating curve on ds p. 4 reduces it with temperature.
- ESD threshold is only 1000 V HBM (ds p. 1); ESD prevention is required for manual and machine
  handling (ds p. 7).
- Only one bin per reel, and a specific bin cannot be ordered (ds p. 7); brightness and colour
  can differ between reels.

## Not established by the sources

- A numeric derating rule: the derating curve on ds p. 4 is a graph only; no breakpoint or
  slope is given in text.
- Minimum VF and maximum IV (ds p. 2 gives neither).
- Thermal resistance and package height. The drawing dimensions 0.7 ± 0.1 mm vertically in the
  end view and 0.3 ± 0.1 mm for the lens step (ds p. 1); neither is labelled as seated height.
- The library part name `LED_GREEN` does not say 0603 or Würth; the device `ChipLED` is shared
  with the Lite-On red LED and declares the same pins, which agree with this datasheet
  (pin 1 cathode, pin 2 anode, ds p. 1).
