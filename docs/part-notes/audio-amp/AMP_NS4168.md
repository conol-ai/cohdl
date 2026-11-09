# NS4168 — 2.5 W mono class-D amplifier with I²S digital input, ESOP-8

## Identity

```json
{
  "schema": "part-note/1",
  "device": "audio-amp::NS4168",
  "parts": ["audio-amp::AMP_NS4168"],
  "identities": [
    {"mfr": "NSIWAY", "mpn": "NS4168", "lcsc": "C910588"}
  ],
  "sources": [
    {"id": "ds", "title": "NS4168 2.5W I2S digital-input mono class-D audio power amplifier (Chinese)", "revision": "V1.2, April 2023, 10 pages",
     "url": "https://www.lcsc.com/datasheet/C910588.pdf", "sha256": "2f8e302b0f69ccad330f8e467b173e6a1eb2ae04f3eeb6640bf3c7afd3c808d4", "local": "lib/audio-amp/docs/ns4168.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed page numbers on all ten pages. The data sheet is in Chinese; translations below are this note's.

**No manufacturer URL.** The URL is LCSC's mirror of the Nsiway document (from `lib/audio-amp/docs/README.md`). Neither the README nor the PDF gives a document URL on Nsiway's own site; the PDF carries only a "www.nsiway.com.cn" watermark. The LCSC part number comes from `lcsc_map.json`, where it is an `alias` match, not an exact one.

## Summary

The NS4168 is a mono, filterless class-D amplifier with a built-in DAC and an I²S slave input. It drives a speaker directly from an I²S stream without a codec (ds p. 1). It delivers 2.5 W into 4 Ω at 5 V and 10 % THD+N, and runs from 3.0 V to 5.5 V (ds p. 1, p. 4). One multi-level CTRL pin selects the right channel, the left channel or shutdown (ds p. 8). It has an anti-clipping (NCN) gain control, overcurrent, overtemperature and undervoltage protection, and sample rates from 8 kHz to 96 kHz (ds p. 1, p. 9). The absolute-maximum supply in the source (5.0 V) is below its own operating maximum (5.5 V) (ds p. 3, p. 4), so do not plan on running above 5.0 V without asking Nsiway.

## Key parameters

T = 25 °C, VDD = 5 V, fS = 48 kHz, default I²S format, unless noted (ds p. 4).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage VDD | 3 | — | 5.5 | V | — | ds p. 4 |
| Quiescent current IDD | — | 13 | — | mA | VDD = 5.0 V, VIN = 0 V, no load | ds p. 4 |
| Shutdown current ISD | — | 1 | — | µA | VCTRL = 0 V | ds p. 4 |
| Output offset VOS | — | 10 | 40 | mV | — | ds p. 4 |
| PSRR | — | — | −80 | dB | 217 Hz | ds p. 4 |
| PSRR | — | — | −72 | dB | 20 kHz | ds p. 4 |
| Switching frequency fSW | — | 430 | — | kHz | VDD = 3 V to 5.25 V | ds p. 4 |
| Efficiency | — | 80 | — | % | Po = 2.5 W, RL = 4 Ω, VDD = 5 V | ds p. 4 |
| Output power Po | — | 1.2 | — | W | VDD = 3.6 V, 1 kHz, RL = 4 Ω, THD+N = 10 % | ds p. 4 |
| Output power Po | — | 2.5 | — | W | VDD = 5.0 V, 1 kHz, RL = 4 Ω, THD+N = 10 % | ds p. 4 |
| THD+N | — | 0.2 | — | % | 1 kHz, RL = 4 Ω, Po = 1.0 W | ds p. 4 |
| SNR | — | 85 | — | dB | RL = 4 Ω, Po = 2.0 W | ds p. 4 |
| CTRL threshold, right channel | 1.5 | — | VDD | V | — | ds p. 4 |
| CTRL threshold, left channel | 0.9 | — | 1.15 | V | — | ds p. 4 |
| CTRL threshold, shutdown | 0 | — | 0.4 | V | — | ds p. 4 |
| CTRL shutdown time TOFF | 100 | — | — | µs | VCTRL = 0 V | ds p. 4 |
| NCN attack time | — | 10 | — | ms | VDD = 3.6 V | ds p. 4 |
| NCN release time | — | 1.1 | — | s | VDD = 3.6 V | ds p. 4 |
| Input high VIH | 0.7 × VDD | — | VDD | V | BCLK, LRCLK, SDATA, CTRL | ds p. 4 |
| Input low VIL | −0.3 | — | 0.3 × VDD | V | BCLK, LRCLK, SDATA, CTRL | ds p. 4 |
| BCLK low / high pulse width | 40 | — | — | ns | — | ds p. 7 |
| LRCLK / SDATA setup and hold to BCLK rising | 10 | — | — | ns | — | ds p. 7 |
| BCLK frequency | 0.128 | — | 6.144 | MHz | Table 2: 16 fS at 8 kHz to 64 fS at 96 kHz (the text calls this the LRCLK range) | ds p. 7 |
| θJC / θJA, ESOP8 | — | 20 / 80 | — | °C/W | Listed with the absolute maximums | ds p. 3 |

## Absolute maximum ratings

Section 7, "极限工作参数" (ds p. 3).

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage range | 2.8 to 5.0 | V | ds p. 3 |
| CTRL, LRCLK, BCLK, SDATA | −0.3 to VDD | V | ds p. 3 |
| ESD, HBM / MM | 4000 / 200 | V | ds p. 3 |
| Operating temperature (listed here) | −40 to +85 | °C | ds p. 3 |
| Storage temperature | −65 to +150 | °C | ds p. 3 |
| Maximum junction temperature | +150 | °C | ds p. 3 |
| Soldering temperature, within 10 s | +220 | °C | ds p. 3 |

## Recommended operating conditions

The source has no separate table. The supply row is from the electrical characteristics; the temperature row is listed among the absolute maximums.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD | 3.0 | 5.5 | V | ds p. 1, p. 4 |
| Operating temperature | −40 | +85 | °C | ds p. 3 |

## Pins

Checked pad by pad against the eSOP-8 pin diagram and pin table in section 5 (ds p. 2). The pin table lists pins 1–8 only. The exposed pad appears only in the package drawing (E2 × D1, ds p. 10), with no number or net.

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | CTRL | CTRL | Input | Left/right channel select and shutdown | ds p. 2 |
| 2 | LRCLK | LRCLK | Input | I²S frame clock (left/right) | ds p. 2 |
| 3 | BCLK | BCLK | Input | I²S bit clock | ds p. 2 |
| 4 | SDATA | SDATA | Input | I²S serial data input | ds p. 2 |
| 5 | VON | VoN | Output | Amplifier output, negative | ds p. 2 |
| 6 | VDD | VDD | Power | Supply input | ds p. 2 |
| 7 | GND | GND | Ground | Power ground | ds p. 2 |
| 8 | VOP | VoP | Output | Amplifier output, positive | ds p. 2 |
| 9 | EP | (exposed pad, unnamed) | — | Exposed pad E2 × D1; no net stated | ds p. 10 |

Pad order agrees: pins 1–4 down the left, 5–8 up the right, pin 1 at the top left by the dot (ds p. 2). The diagram does not say whether it is a top view. `FP_NS4168_ESOP8` places pad 1 at the top left with the same order.

## Typical application

- Supply: 100 µF in parallel with a 1 µF ceramic, as close to the VDD pin as possible (ds p. 9). The p. 1 circuit shows 100 µF and "105" (1 µF) on VDD (ds p. 1).
- Inputs: the p. 1 circuit puts a series resistor in each of CTRL, LRCLK, BCLK and SDATA, with no value given (ds p. 1).
- CTRL: 1.5 V to VDD selects the right channel; 0.9–1.15 V selects the left; 0–0.4 V shuts down (ds p. 8). The left-channel window needs a divider or reference; a GPIO high selects right, low shuts down.
- I²S: always slave; standard I²S with a one-BCLK delay, MSB first, two's complement (ds p. 6). LRCLK high = right-channel data, low = left (ds p. 6). Table 2 lists the supported BCLK rates (16, 24, 32, 48 or 64 × fS) for fS from 8 kHz to 96 kHz (ds p. 7).
- Outputs: drive the speaker directly between VoN and VoP (ds p. 1). For EMI, add a ferrite bead (1000 Ω shown) and 1 nF to ground on each output, close to the pins (ds p. 9).
- Protection: output short to supply, ground or across the outputs shuts the chip down; it restarts when the short is removed. Overtemperature and undervoltage also shut down and recover (ds p. 9).

## Layout notes

- Keep the speaker traces short and wide and away from sensitive signals; put the supply decoupling next to the pins; star-connect power and ground (ds p. 9).
- Package (ds p. 10): D 4.70–5.10 mm, E 3.80–4.00 mm, E1 (lead span) 5.80–6.20 mm, A 1.30–1.70 mm, lead width b 0.33–0.51 mm, foot length L 0.40–1.27 mm, pitch 1.27 mm BSC, exposed pad E2 2.313–2.513 mm × D1 3.202–3.402 mm.
- The source gives no recommended land pattern. `FP_NS4168_ESOP8` is the library's own: 1.5 × 0.55 mm lead pads at ±3.1 mm, and a 2.413 × 3.302 mm centre pad (the mid-point of E2 × D1).
- θJA is 80 °C/W with no board condition stated (ds p. 3).

## Pitfalls

- The absolute-maximum supply (2.8–5.0 V, ds p. 3) disagrees with the 3.0–5.5 V operating range and the 2.5 W at 5.0 V figure (ds p. 1, p. 4). A 5 V USB rail with overshoot exceeds the stated absolute maximum.
- The I²S and CTRL inputs are rated only to VDD (ds p. 3). Do not drive them from a higher-voltage MCU while the amplifier is unpowered or on a lower rail.
- CTRL is a multi-level input, not a logic pin. 1.15–1.5 V and 0.4–0.9 V are undefined bands (ds p. 8).
- Measuring output power needs an LC filter (two 33 µH inductors in series with the load); a plain resistor load reads worse than a speaker (ds p. 9).
- The only soldering limit is 220 °C for 10 s (ds p. 3). The source gives no reflow profile, so check the assembler's profile against that limit.

## Not established by the sources

Disagreements and gaps between the library and the source, most serious first:

- **Exposed pad net.** The package drawing shows an exposed pad (E2 × D1, ds p. 10) but the pin table and every circuit leave it out (ds pp. 1–2). The library models it as `optional EP: 9 [passive]` and leaves the connection to the design. The source does not say whether to solder it, or whether to connect it to GND, or leave it floating.
- **Land pattern.** The source gives only package dimensions (ds p. 10). The library's lead pads (1.5 × 0.55 mm at ±3.1 mm) and its exposed-pad land are not from the source. The comment's "lead-tip span E1 6.0mm" is the mid-point of 5.80–6.20 mm; the source gives no nominal.
- **Body size in the comments.** "5.0x4.0mm body" is not a source value: D is 4.70–5.10 mm and E 3.80–4.00 mm (ds p. 10).
- **Supply range.** The library's "supply 3.0-5.5V (p1)" matches the operating range (ds p. 1, p. 4), but the source's absolute maximum is 5.0 V (ds p. 3). The source contradicts itself; the library does not mention the 5.0 V limit.
- **CTRL logic levels.** The source lists CTRL among the inputs with VIH = 0.7 × VDD (ds p. 4) and also gives it a 1.5 V right-channel threshold (ds p. 4, p. 8). The library types CTRL `input` and follows the multi-level reading, which is the one sections 8 and 10.2.4 describe.
- **Output pin names.** The source writes VoN and VoP (ds p. 2); the library uses VON and VOP. Only the case differs.
- **Source origin.** The PDF is an LCSC mirror, not a document from Nsiway's site.

Not stated by the source:

- Which clock the 128 kHz–6.144 MHz range applies to. The text says LRCLK, but Table 2 lists those values as BCLK rates; it also gives an MCLK range (2.048–24.576 MHz) although the part has no MCLK pin (ds p. 7).
- An English version, a document number, or a revision history (section 12 is empty, ds p. 10).
- Supply current in shutdown over temperature, or any limit over temperature.
- Pop/click timing, start-up time after CTRL rises, and I²S word lengths supported.
- The values of the series resistors in the p. 1 circuit.
- Moisture sensitivity level and reflow profile.
