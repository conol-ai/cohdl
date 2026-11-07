# NS4150B — 3 W mono filterless class-D audio amplifier, analog differential input, MSOP-8

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/audio-amp::NS4150B",
  "parts": ["@contrib/audio-amp::AMP_NS4150B"],
  "identities": [
    {"mfr": "NSIWAY", "mpn": "NS4150B", "lcsc": "C189961"}
  ],
  "sources": [
    {"id": "ds", "title": "NS4150 用户手册 (NS4150 user manual), 超低 EMI、无需滤波器、3W 单声道 D 类音频功放", "revision": "V1.0, May 2011",
     "url": "https://www.nsiway.com.cn", "sha256": "61812f56feefc64f799d60b1807b5411ffe2c338687163f3b81ba1ccf6ced2f1", "local": "lib/@contrib/audio-amp/docs/ns4150b.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed page numbers in the footer (1–14). The source is in Chinese; the English in this note is a translation. The recorded URL is only the Nsiway home page (package README). The README also names Nsiway's NS4150B product page, `https://nsiway.com.cn/list_38/82.html`, which hosts the V1.4 NS4150B pages as images; those images are not in the repository and are not a source of this note.

**The pinned PDF is the legacy NS4150 manual, not an NS4150B document.** Every number below is for NS4150. The package README says the V1.4 NS4150B pages agree on the pin map and MSOP-8 dimensions; this note did not check them.

## Summary

NS4150 is a mono, filterless class-D amplifier with an analog (differential or single-ended) input. It drives a speaker bridge-tied between VoP and VoN (ds p. 5). It gives 2.8 W into 4 Ω at 5 V and 10 % THD, runs from 3.0 V to 5.25 V, and has overcurrent, overtemperature and undervoltage protection with automatic recovery (ds p. 5, p. 6, p. 12). Gain is set by the external input resistors: Av = 300 kΩ / Rin (ds p. 12). A single CTRL pin turns it on (high) or shuts it down (low) (ds p. 11). Use it behind an analog source such as a DAC or codec output; for an I²S source use a part with a digital input.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output power PO | — | 2.8 | — | W | THD = 10 %, 1 kHz, RL = 4 Ω, VDD = 5 V | ds p. 6 |
| Output power PO | — | 2.0 | — | W | THD = 1 %, 1 kHz, RL = 4 Ω, VDD = 5 V | ds p. 6 |
| Output power PO | — | 1.7 | — | W | THD = 10 %, 1 kHz, RL = 8 Ω, VDD = 5 V | ds p. 6 |
| Output power PO | — | 1.3 | — | W | THD = 1 %, 1 kHz, RL = 8 Ω, VDD = 5 V | ds p. 6 |
| THD | — | 0.1 | — | % | VDD = 3.6 V, PO = 0.5 W, RL = 4 Ω, 1 kHz | ds p. 6 |
| Efficiency η | — | 90 | — | % | PO = 0.6 W, RL = 8 Ω, VDD = 3.6 V, 1 kHz | ds p. 7 |
| Output offset, magnitude of VOS | — | 5 | 20 | mV | VIN = 0 V, VDD = 3.0–5.25 V | ds p. 6 |
| Quiescent current IQ | — | 3.0 | — | mA | VDD = 3.6 V, no load | ds p. 6 |
| Shutdown current ISD | — | 0.1 | 10 | µA | VDD = 3.6 V, CTRL = 0 V | ds p. 6 |
| PSRR | — | -80 / -72 | — | dB | 217 Hz / 20 kHz | ds p. 6 |
| CMRR | — | -70 | — | dB | | ds p. 6 |
| Switching frequency fSW | — | 400 | — | kHz | VDD = 3.0–5.25 V | ds p. 6 |
| CTRL input high VIH | 1.2 | — | VDD | V | | ds p. 7 |
| CTRL input low VIL | 0 | — | 0.2 | V | | ds p. 7 |
| Start-up time tST | — | 30 | — | ms | | ds p. 7 |
| Wake-up time tWK | — | 35 | — | ms | value printed in the Min column | ds p. 7 |
| Shutdown time tSD | — | 80 | — | ms | value printed in the Min column | ds p. 7 |
| Internal feedback resistor | — | 300 | — | kΩ | Av = 300 kΩ / Rin | ds p. 12 |
| Thermal resistance θJA, MSOP8 | — | 190 | — | °C/W | listed with the limits | ds p. 6 |

All Table 2 values are at TA = 25 °C (ds p. 6).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VDD | -0.3 to 6.0 | V | ds p. 6 |
| INP, INN, CTRL pin voltage | -0.3 to VDD + 0.3 | V | ds p. 6 |
| Maximum junction temperature | 150 | °C | ds p. 6 |
| Storage temperature | -65 to 150 | °C | ds p. 6 |
| Lead temperature (soldering, 10 s) | 260 | °C | ds p. 6 |
| ESD protection voltage (model not stated) | ±4000 | V | ds p. 6 |
| Latch-up, +IT / −IT | 150 / -150 | mA | ds p. 6 |

## Recommended operating conditions

The source has no separate table; the operating temperature is printed in Table 1 (limits) and the supply range in the features list.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD (工作电压范围) | 3.0 | 5.25 | V | ds p. 5 |
| Operating temperature | -40 | 85 | °C | ds p. 5, p. 6 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CTRL | CTRL | input | Mode control: high = on, low = shutdown | ds p. 7, p. 11 |
| 2 | BYPASS | Bypass | passive | Internal common-mode voltage bypass capacitor | ds p. 7 |
| 3 | INP | INP | input | Non-inverting audio input | ds p. 7 |
| 4 | INN | INN | input | Inverting audio input | ds p. 7 |
| 5 | VON | VoN | output | Inverting audio output | ds p. 7 |
| 6 | VCC | VCC | power | Supply input | ds p. 7 |
| 7 | GND | GND | ground | Ground | ds p. 7 |
| 8 | VOP | VoP | output | Non-inverting audio output | ds p. 7 |

Figure 2 (top view, ds p. 7): pins 1–4 down the left side, 5–8 up the right side, pin 8 opposite pin 1. The library footprint `SOP8P65X490X110N` numbers its pads the same way in its +y-down frame.

## Typical application

From Figure 1 (ds p. 5):
- 1 µF from VDD to GND. Section 9.9 asks for a 1 µF low-ESR ceramic as close to VDD as possible, plus 10 µF or more if low-frequency noise must be filtered (ds p. 12).
- Each input through 0.1 µF and 30 kΩ in series (Cin, Rin). Gain Av = 300 kΩ / Rin (ds p. 12); with 30 kΩ that is 10 (computed, not stated). The input high-pass corner is fc = 1/(2π·Rin·Cin) (ds p. 12). Differential and single-ended input wiring is in Figure 6 (ds p. 12).
- CTRL pulled up to VDD through a resistor, with a switch to GND for shutdown (Figure 1, ds p. 5). The resistor value is not printed.
- 1 µF from Bypass to GND (Figure 1, ds p. 5).
- Speaker between VoN and VoP; no output filter (ds p. 5, p. 11).

## Layout notes

- Place the VDD decoupling capacitor as close to the VDD pin as possible (ds p. 12).
- If the speaker leads are long or the amplifier is near EMI-sensitive parts, add ferrite beads and capacitors on the outputs, close to the chip (Figure 7, ds p. 12). Without them the source claims FCC compliance with 60 cm of speaker cable (ds p. 12).
- MSOP-8 package (Figure 8, ds p. 13): pitch e 0.65 mm BSC; lead span E 4.70–5.10 mm; body D and E1 2.90–3.10 mm; lead length L 0.40–0.70 mm; lead width b 0.29–0.38 mm; height A 1.10 mm max.
- Library land: 1.25 × 0.45 mm pads at x = ±2.175 mm and y = ±0.325, ±0.975 mm, so pitch 0.65 mm, outer span 5.60 mm and inner gap 3.10 mm. The source gives no recommended land; the library's derivation is in `lib/@contrib/audio-amp/docs/footprint-derivations.md`. The pads cover the full terminal envelope above.

## Pitfalls

- The output is bridge-tied. Do not connect VoN or VoP to ground; protection shuts the chip down on output shorts (ds p. 12).
- CTRL has no stated internal pull. Drive it or pull it up (Figure 1, ds p. 5); a floating CTRL is undefined.
- θJA is 190 °C/W in MSOP8 (ds p. 6). Typical dissipation curves are given only at 3.6 V and 4.2 V (ds p. 8); there is no curve at 5 V, where the 2.8 W rating applies.
- The SOP8 version (θJA 150 °C/W, Figure 9, ds p. 6, p. 14) has the same pin order but a different land. The library models only MSOP-8.

## Not established by the sources

- **Exact product:** the pinned PDF covers NS4150, not NS4150B (ds p. 1). Whether NS4150B differs electrically is not established here. The library comment cites "the current NS4150B V1.4 pages", which are not in the repository.
- **Supply range:** the library comment on `VCC` says "recommended supply 3.0-5.0V" and the device header says "3.0-5.0V". The source gives 3.0–5.25 V (ds p. 5, test conditions on ds p. 6).
- **CTRL default:** the library comment says CTRL has "no internal default". The source states nothing about an internal pull either way (ds p. 7, p. 11); the comment is an inference.
- **Bypass pin:** the library declares `BYPASS` optional. Figure 1 always fits 1 µF there (ds p. 5); the source does not say it may be left open.
- **Timing columns:** tWK = 35 ms and tSD = 80 ms are printed in the Min column (ds p. 7); whether they are minimums or typicals is unclear.
- **ESD model** (HBM or other) for the ±4000 V figure is not stated (ds p. 6).
- **Recommended PCB land, thermal guidance and maximum power dissipation** are not given.
