# MAX98357AETE+ — I²S/TDM-input mono class-D amplifier, 3.2 W, 2.5–5.5 V, 16-pin 3 × 3 mm TQFN

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/audio-amp::MAX98357A",
  "parts": ["@contrib/audio-amp::AMP_MAX98357A"],
  "identities": [
    {"mfr": "Analog Devices", "mpn": "MAX98357AETE+", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "MAX98357A/MAX98357B Tiny, Low-Cost, PCM Class D Amplifier with Class AB Performance", "revision": "19-6779 Rev 16, 2/26",
     "url": "https://www.analog.com", "sha256": "dea228ad619901d34c2f5f106243bdb9b9e95e38989d4d66fd6c6c489bfa6f1c", "local": "lib/@contrib/audio-amp/docs/max98357a.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed page numbers ("Analog Devices │ N", 1–38). The recorded URL is only the Analog Devices home page, which is what the package README gives; neither the README nor the PDF gives a document link for the datasheet. `lcsc_map.json` has no row for this MPN.

The library land comes from ADI land pattern 90-0031 rev C, which the README records by URL and SHA-256 but does not ship. It is not a source of this note; the land is checked here only against the package outline on ds pp. 36–37.

## Summary

MAX98357A is a mono class-D amplifier with a digital PCM input: I²S (or 8-slot TDM) on BCLK, LRCLK and DIN, no MCLK and no I²C (ds p. 1). It delivers 3.2 W into 4 Ω at 5 V and 10 % THD+N, and runs from a single 2.5–5.5 V supply (ds p. 1, p. 4, p. 5). One pin, SD_MODE, selects shutdown and the channel (left, right or (L+R)/2); another, GAIN_SLOT, selects one of five gains (ds p. 15, p. 17, p. 28). The output is filterless and needs a speaker with more than 10 µH series inductance (ds p. 33). The library models the TQFN only; the MAX98357B is the left-justified variant and is not this part (ds p. 1).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output power POUT | — | 3.2 | — | W | THD+N 10 %, gain 12 dB, 4 Ω + 33 µH, VDD = 5 V | ds p. 5 |
| Output power POUT | — | 1.8 | — | W | THD+N 10 %, 8 Ω + 68 µH, VDD = 5 V | ds p. 5 |
| Output power POUT | — | 0.93 | — | W | THD+N 10 %, 8 Ω + 68 µH, VDD = 3.7 V | ds p. 5 |
| Output power POUT | — | 2.5 / 1.4 | — | W | THD+N 1 %, 4 Ω / 8 Ω, VDD = 5 V | ds p. 5 |
| THD+N | — | 0.02 | — | % | 1 kHz, POUT = 1 W, 4 Ω + 33 µH, TQFN | ds p. 5 |
| Undervoltage lockout | 1.5 | 1.8 | 2.3 | V | | ds p. 4 |
| Quiescent current IDD | — | 2.75 | 3.35 | mA | TA = 25 °C, VDD = 5 V | ds p. 4 |
| Quiescent current IDD | — | 2.4 | 2.85 | mA | TA = 25 °C, VDD = 3.7 V | ds p. 4 |
| Shutdown current | — | 0.6 | 2 | µA | SD_MODE = 0 V, 25 °C | ds p. 4 |
| Standby current (no BCLK) | — | 340 | 400 | µA | SD_MODE = 1.8 V, 25 °C | ds p. 4 |
| Turn-on time | — | 7 | 7.5 | ms | | ds p. 4 |
| Gain, GAIN_SLOT to GND via 100 kΩ | 14.4 | 15 | 15.6 | dB | relative to 2.1 dBV | ds p. 5 |
| Gain, GAIN_SLOT to GND | 11.4 | 12 | 12.6 | dB | | ds p. 5 |
| Gain, GAIN_SLOT unconnected | 8.4 | 9 | 9.6 | dB | | ds p. 5 |
| Gain, GAIN_SLOT to VDD | 5.4 | 6 | 6.6 | dB | | ds p. 5 |
| Gain, GAIN_SLOT to VDD via 100 kΩ | 2.4 | 3 | 3.6 | dB | | ds p. 5 |
| Current limit ILIM | — | 2.8 | — | A | outputs off for about 100 µs, then retry | ds p. 5, p. 28 |
| Switching frequency | — | 300 | — | kHz | spread ±20 kHz | ds p. 5 |
| Efficiency | — | 92 | — | % | 8 Ω + 68 µH, THD+N 10 %, 1 kHz, 12 dB | ds p. 5 |
| LRCLK (sample rate) ranges | 7.6 | 8 / 16 / 48 / 96 | 100.8 | kHz | four ranges, see table | ds p. 6 |
| BCLK frequency | 0.2432 | — | 25.804 | MHz | BCLK = 32, 48 or 64 × LRCLK | ds p. 6 |
| Digital input VIH / VIL | 1.3 | — | 0.6 | V | VIH min / VIL max | ds p. 6, p. 7 |
| SD_MODE trip points B0 / B1 / B2 | — | 0.16 / 0.77 / 1.4 | — | V | see Table 5 | ds p. 7 |
| SD_MODE pull-down RPD | 92 | 100 | 108 | kΩ | internal | ds p. 7 |
| θJA / θJC, TQFN | — | 48 / 7 | — | °C/W | JESD51-7, four-layer board | ds p. 4 |

Electrical characteristics are at VDD = 5 V, GAIN_SLOT = VDD, BCLK = 3.072 MHz, LRCLK = 48 kHz unless noted, typicals at 25 °C (ds p. 4).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| VDD, LRCLK, BCLK, DIN to GND | -0.3 to +6 | V | ds p. 4 |
| All other pins (SD_MODE, GAIN_SLOT, outputs) to GND | -0.3 to VDD + 0.3 | V | ds p. 4 |
| Continuous current in/out of VDD, GND, OUT_ | ±1.6 | A | ds p. 4 |
| Continuous input current, all other pins | ±20 | mA | ds p. 4 |
| Continuous power dissipation, TQFN, TA = +70 °C (derate 20.8 mW/°C above) | 1666 | mW | ds p. 4 |
| Junction temperature | +150 | °C | ds p. 4 |
| Storage temperature | -65 to +150 | °C | ds p. 4 |
| Soldering temperature (reflow) | +260 | °C | ds p. 4 |
| Lead temperature (soldering, 10 s, TQFN) | +300 | °C | ds p. 4 |

Output short to GND, to VDD or OUTP to OUTN: continuous (ds p. 4).

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD (guaranteed by PSRR test) | 2.5 | 5.5 | V | ds p. 4 |
| Operating temperature (listed under absolute maximum ratings; also the MAX98357AETE+ ordering range) | -40 | +85 | °C | ds p. 4, p. 34 |
| Speaker series inductance | 10 | — | µH | ds p. 33 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | DIN | DIN | input | Digital audio data | ds p. 15 |
| 2 | GAIN_SLOT | GAIN_SLOT | input | Gain select (I²S/LJ); TDM slot select with SD_MODE | ds p. 15 |
| 3 | GND3 | GND | ground | Ground | ds p. 15 |
| 4 | SD_MODE | SD_MODE | input | Shutdown (low) and channel select | ds p. 15 |
| 5 | NC5 | N.C. | — | No connection | ds p. 15 |
| 6 | NC6 | N.C. | — | No connection | ds p. 15 |
| 7 | VDD | VDD | power | Supply input | ds p. 15 |
| 8 | VDD | VDD | power | Supply input | ds p. 15 |
| 9 | OUTP | OUTP | output | Positive speaker output | ds p. 15 |
| 10 | OUTN | OUTN | output | Negative speaker output | ds p. 15 |
| 11 | GND11 | GND | ground | Ground | ds p. 15 |
| 12 | NC12 | N.C. | — | No connection | ds p. 15 |
| 13 | NC13 | N.C. | — | No connection | ds p. 15 |
| 14 | LRCLK | LRCLK | input | Frame clock (I²S/LJ left/right; TDM sync) | ds p. 15 |
| 15 | GND15 | GND | ground | Ground | ds p. 15 |
| 16 | BCLK | BCLK | input | Bit clock | ds p. 15 |
| EP | EP | EP | — | Exposed pad, not internally connected; connect to a solid ground plane for heat | ds p. 15 |

TQFN top view (ds p. 15): pins 1–4 along the bottom left to right, 5–8 up the right side, 9–12 along the top right to left, 13–16 down the left side; pin 1 at the "+" mark at the lower-left corner. The library footprint `QFN16N50P300X300_1EP123X123` places pads the same way in its +y-down frame (pad 1 at (−0.75, +1.425) mm, i.e. lower left).

## Typical application

- VDD bypass: 0.1 µF and 10 µF to GND; some designs need only the 10 µF. Add bulk at the IC if the supply trace is long (ds p. 33; Figures 16–17, ds p. 29).
- SD_MODE selects the output (Table 5, ds p. 17):
  - driven high (above B2, 1.4 V typ) → left channel;
  - pulled up through RSMALL → right channel;
  - pulled up through RLARGE → (left + right)/2;
  - low (below B0) → shutdown.
  RSMALL (kΩ) = 94.0 × VDDIO − 100 and RLARGE (kΩ) = 222.2 × VDDIO − 100; Table 6 gives 69.8 kΩ / 300 kΩ at 1.8 V and 210.2 kΩ / 634 kΩ at 3.3 V (ds p. 17).
- If SD_MODE is driven high from a logic rail above VDD (for example VDD < 3.0 V with 3.3 V logic), add about 2 kΩ in series with SD_MODE (ds p. 17).
- GAIN_SLOT sets the I²S gain (Table 8, ds p. 28): 100 kΩ to GND = 15 dB, GND = 12 dB, open = 9 dB, VDD = 6 dB, 100 kΩ to VDD = 3 dB (100 kΩ ±5 %). In TDM mode the gain is fixed at 12 dB and GAIN_SLOT with SD_MODE selects the slot (Table 7, ds pp. 19–20).
- LRCLK must be one of 8, 16, 32, 44.1, 48, 88.2 or 96 kHz; 11.025, 12, 22.05 and 24 kHz are not supported (ds p. 16). LRCLK must start within half an LRCLK period of BCLK (ds p. 18). BCLK = 256 kHz needs the extra start-up sequence on ds pp. 18–19.

## Layout notes

- Solder the exposed pad to a solid ground plane for heat (ds p. 15).
- Use wide, low-resistance output, supply and ground traces; output trace capacitance adds VDD × 300 kHz × C to the quiescent current (ds p. 33).
- Add ground fill around signal traces on the outer layers for RF immunity (ds p. 33).
- Package T1633+4, outline 21-0136, land pattern 90-0031 (ds p. 35). Outline for 16L 3 × 3 (ds p. 37): D and E 2.90–3.10 mm; e 0.50 mm; b 0.20–0.30 mm; L 0.30–0.50 mm; A 0.70–0.80 mm; exposed pad D2 and E2 0.95–1.25 mm (T1633-4, 1.10 nominal).
- Library land: 0.80 × 0.30 mm leads on a 0.50 mm pitch, row centres 2.85 mm apart (±1.425 mm), 1.23 mm square centre land. Against the outline above: the lead lands span 1.025–1.825 mm from centre. The terminal outer edge is at 1.45–1.55 mm and its inner edge at 0.95–1.25 mm (computed from D and L), so a worst-case long terminal reaches 0.075 mm inside the land's inner edge. The centre land is inside D2/E2 max (1.25 mm); the 0.30 mm lead land equals b max, with no side margin.

## Pitfalls

- **SD_MODE left open means shutdown.** SD_MODE has an internal 100 kΩ pull-down (ds p. 7) and below B0 the part is shut down (Table 5, ds p. 17). Always drive or bias it. @contrib/audio-amp ≤0.1.x declares `SD_MODE` `optional`, so a design can leave it open and get no sound; fixed in 0.2.0 (PR #96), which declares it `required`.
- GAIN_SLOT open is a valid setting (9 dB), but its trip band is 0.4–0.6 × VDD (ds p. 7); leakage or noise on a long open trace can shift it.
- The left channel is LRCLK low on MAX98357A (Table 3, ds p. 16). Do not swap in MAX98357B, which uses left-justified data and LRCLK high for left (ds p. 16).
- Do not remove LRCLK while BCLK runs: the output can show a large DC voltage (ds p. 16).
- There is no volume ramp on entering shutdown; ramp the digital data down first (ds p. 28).

## Not established by the sources

- **@contrib/audio-amp ≤0.1.x: SD_MODE optionality; fixed in 0.2.0 (PR #96).** See Pitfalls. The ≤0.1.x comment "shutdown + channel select" is correct, but `optional` lets the pin float, which the source defines as shutdown (ds p. 7, p. 17). 0.2.0 makes the pin `required` and gives the Table 5/6 channel select in its comment.
- **Land pattern:** the shipped datasheet points to 90-0031 for the land (ds p. 35) but does not reproduce it. The library's 0.80 × 0.30 mm leads, 2.85 mm row pitch and 1.23 mm centre land are taken from 90-0031, which is not in the repository; only their consistency with the outline (ds p. 37) is checked here.
- **Operating temperature placement:** −40 to +85 °C is listed among the absolute maximum ratings (ds p. 4); the source has no separate recommended-conditions table.
- **Thermal vias** in the exposed pad: not specified.
- **Ordering status:** the ordering table lists MAX98357AETE+ and MAX98357AETE+T (tape and reel) with top mark +AKK (ds p. 34). The library MPN is the non-tape version; the source does not say whether either is preferred.
