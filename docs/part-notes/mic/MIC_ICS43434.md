# ICS-43434 — bottom-port MEMS microphone with 24-bit I²S output, 3.50 × 2.65 mm LGA

## Identity

```json
{
  "schema": "part-note/1",
  "device": "mic::ICS43434",
  "parts": ["mic::MIC_ICS43434"],
  "identities": [
    {"mfr": "TDK InvenSense", "mpn": "ICS-43434", "lcsc": "C5656610"}
  ],
  "sources": [
    {"id": "ds", "title": "ICS-43434 Multi-Mode Microphone with I2S Digital Output", "revision": "DS-000069 Revision 1.2, released 2016-08-29; 22-page Mouser copy (21 pages plus a distributor page)",
     "url": "https://invensense.tdk.com/wp-content/uploads/2016/02/DS-000069-ICS-43434-v1.2.pdf", "sha256": "5f940de70ac517f541528d1ad5bb9a5cb4326fe6dfc1228e000cd1b3aed9d927", "local": "lib/mic/docs/tdk-invensense-ics-43434.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed "Page N of 21" on pp. 2–21. Page 1 (cover) has no printed number. Page 22 is a Mouser page appended to the manufacturer document.

**Source URL caveat.** The URL is TDK's, from `lib/mic/docs/README.md`. The README says that endpoint redirected to HTML during retrieval and the local file came from Mouser (`https://www.mouser.cn/datasheet/2/400/ds_000069_ics_43434_v1_2-2581173.pdf`). Page 22 confirms the Mouser origin. The SHA-256 is of the Mouser copy, so a file fetched from the TDK URL is not expected to hash the same.

The ICS-40300 in the same package has no shipped PDF (only `ics-40300.evidence.md`), so it has no note.

## Summary

The ICS-43434 is an omnidirectional, bottom-port MEMS microphone with an ADC, decimation filters and a 24-bit I²S slave output, so it connects to an MCU or DSP without a codec (ds p. 1, p. 4). It runs from 1.65 V to 3.63 V (ds p. 4). Its mode follows the clock: high-performance at fS 23–51.6 kHz (65 dBA SNR, 490 µA typ), low-power at 6.25–18.75 kHz (64 dBA, 230 µA typ), and sleep below 3.125 kHz (12 µA typ) (ds pp. 4–6). Sensitivity is −26 dB FS ±1 dB, and the acoustic overload point is 120 dB SPL (ds p. 4). The LR pin puts the microphone in the left or right I²S slot, so two can share one data line (ds p. 10, p. 12). It needs an acoustic hole in the PCB under the port (ds p. 17).

## Key parameters

Tables 1–3: TA = 25 °C, VDD = 1.8–3.3 V, CLOAD = 30 pF; typical values are not guaranteed (ds pp. 4–5). Table 5: −40 °C < TA < +85 °C, 1.8 V < VDD < 3.3 V (ds p. 6).

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage VDD | 1.65 | — | 3.63 | V | — | ds p. 4 |
| Sleep-mode current | — | 12 | 20 | µA | fS < 3.125 kHz | ds p. 4 |
| Supply current, high-performance mode | — | 490 | 550 | µA | VDD = 1.8 V, no load, fS = 48 kHz | ds p. 4 |
| Supply current, low-power mode | — | 230 | 300 | µA | VDD = 1.8 V, no load, fS = 16 kHz | ds p. 5 |
| Sensitivity | −27 | −26 | −25 | dB FS | 1 kHz, 94 dB SPL, both modes | ds p. 4 |
| SNR, high-performance mode | — | 65 | — | dBA | 20 kHz bandwidth, A-weighted | ds p. 4 |
| SNR, low-power mode | — | 64 | — | dBA | 20 kHz bandwidth, A-weighted | ds p. 5 |
| THD | — | 0.2 | 1 | % | 105 dB SPL | ds p. 4 |
| Acoustic overload point | — | 120 | — | dB SPL | 10 % THD | ds p. 4 |
| PSR | — | −99 | — | dB FS | 217 Hz, 100 mV p-p square on VDD = 1.8 V, HP mode | ds p. 4 |
| Sampling frequency fS, high-performance mode | 23 | — | 51.6 | kHz | — | ds p. 6 |
| Sampling frequency fS, low-power mode | 6.25 | — | 18.75 | kHz | — | ds p. 6 |
| Sampling frequency fS, sleep mode | — | — | 3.125 | kHz | — | ds p. 6 |
| SCK period tSCP | 303 | — | 2500 | ns | — | ds p. 6 |
| SCK high / low time | 50 | — | — | ns | — | ds p. 6 |
| WS setup / hold | 0 / 20 | — | — | ns | — | ds p. 6 |
| SD data valid tSDV | — | — | 75 | ns | From SCK falling | ds p. 6 |
| SD data disable tSDD | — | — | 76 | ns | From SCK falling to tristate | ds p. 6 |
| Input low VIL | 0 | — | 0.3 × VDD | V | — | ds p. 6 |
| Input high VIH | 0.7 × VDD | — | VDD | V | — | ds p. 6 |
| SD output low VOL | 0 | — | 0.35 × VDD | V | — | ds p. 6 |
| SD output high VOH | 0.65 × VDD | — | VDD | V | — | ds p. 6 |
| SD maximum load | — | — | 85 | pF | fS = 48 kHz | ds p. 6 |
| Wake-up time from sleep | — | — | 20 | ms | Output within 1 dB of final sensitivity | ds p. 6 |
| High-pass filter −3 dB corner | — | 24 | — | Hz | Does not scale with fS | ds p. 14 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VDD | −0.3 to +3.63 | V | ds p. 8 |
| Digital pin input voltage | −0.3 to VDD + 0.3, or 3.63 if less | V | ds p. 8 |
| Sound pressure level | 160 | dB | ds p. 8 |
| Mechanical shock | 10,000 | g | ds p. 8 |
| Temperature, biased | −40 to +85 | °C | ds p. 8 |
| Temperature, storage | −55 to +150 | °C | ds p. 8 |

## Recommended operating conditions

The data sheet has no separate recommended-operating-conditions table. The rows are the supply range from Table 1 and the temperature range from the ordering information.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VDD | 1.65 | 3.63 | V | ds p. 4 |
| Temperature range | −40 | +85 | °C | ds p. 1 |

## Pins

Checked pad by pad against Figure 3 (top view, terminal side down) and Table 8 (ds p. 10).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | WS | WS | Input | I²S word select | ds p. 10 |
| 2 | LR | LR | Input | Channel select: low = left, high = right | ds p. 10 |
| 3 | GND | GND | Ground | Ground; the annular pad round the sound port | ds p. 10 |
| 4 | SCK | SCK | Input | I²S serial clock | ds p. 10 |
| 5 | VDD | VDD | Power | 1.65–3.63 V; decouple with 0.1 µF | ds p. 10 |
| 6 | SD | SD | Output | I²S serial data; tristates outside its slot | ds p. 10 |

Pad positions also agree. In Figure 3 the ground ring is at the top centre, pads 4 and 2 are left and right in the middle row, and pads 5, 6, 1 run left to right along the bottom (ds p. 10). `FP_InvenSense_ICS_43434_6_3_5x2_65mm` places them the same way, with y increasing downward.

## Typical application

- VDD: 0.1 µF ceramic, X7R or better, between pin 5 and ground, as close to pin 3 as possible, with short traces on one layer and no vias (ds p. 15, Figure 12).
- SD: 100 kΩ pull-down on the shared SD line to discharge it while every microphone is tristated (ds p. 10, p. 12). Use a smaller value such as 10 kΩ if the line must discharge faster (ds p. 12).
- LR: tie low for the left channel (data after WS falls) or high for the right channel (data after WS rises) (ds p. 10, p. 12). For stereo, tie the two SD pins together and give the microphones opposite LR levels (ds pp. 12–13, Figure 8).
- Frame: I²S, 24-bit two's complement, MSB first, 64 SCK cycles per WS frame (ds p. 12).
- Mode is set only by the clock rate; there is no mode pin (ds p. 6, p. 15). Standby starts when SCK falls below about 200 kHz (ds p. 12).
- Startup takes less than 20 ms (ds p. 12). Wake from sleep takes 32,768 SCK cycles before data starts (ds p. 15).

## Layout notes

- Land pattern, 1:1 with the package pads (Figure 13, ds p. 17): five 0.600 × 0.522 mm pads, 0.900 mm pitch along the three-pad row, 0.822 mm between the two pad rows, 1.252 mm from the inner row to the port centre, and a ground annulus 1.625 mm outer / 1.025 mm inner diameter. The library footprint matches every one of these values.
- Paste stencil (Figure 14, ds p. 17): 0.500 × 0.422 mm on the five pads, and a four-segment annulus 1.625 / 1.125 mm with 0.10 mm gaps. The library `paste:` overrides match.
- PCB sound hole: larger than the microphone's port, at least 0.5 mm diameter (ds p. 17). The port is ø0.375 mm (Figure 15, ds p. 19). The library's 0.5 mm NPTH meets the minimum.
- Keep solder paste out of the sound hole (ds p. 17). Do not wash with water reaching the port, blow off or use ultrasonic cleaning (ds p. 18).
- Package: 3.50 × 2.65 × 0.98 mm (ds p. 19). Pick up from the lid only; never vacuum on the port side (ds p. 18).
- Reflow: Table 7, Pb-free peak 260 °C +0/−5 °C (ds p. 9).

## Pitfalls

- SD is tristated outside its slot. Without the pull-down, the line floats while every microphone is idle (ds p. 10, p. 12).
- Do not clock WS and SCK while VDD is off. It forward-biases the ESD diodes, which may affect long-term reliability (ds p. 12).
- To stop the microphone, stop both SCK and WS and hold them low, so no current flows through WS's internal pull-down (ds p. 12).
- The SD output is rated for 85 pF, including about 6 pF per other tristated microphone on the line (ds p. 15).
- Sensitivity is −26 dB FS peak, not RMS (ds p. 14).
- VDD's absolute maximum (3.63 V) equals its operating maximum (ds p. 4, p. 8). A 3.3 V rail with overshoot leaves little margin.

## Not established by the sources

The library agrees with the source on every pin number and name, LR function, ground annulus, sound-port hole and land dimension checked above. Remaining gaps and differences:

- **SD pin type.** The library types SD as `output`. The source says it tristates when not driving and needs a bus pull-down (ds p. 10). The library cannot express a tristate output, and neither the device comment nor the package says the 100 kΩ pull-down is required.
- **WS internal pull-down.** Section "Standby Mode" refers to "the WS pin's internal pull-down resistor" (ds p. 12). Table 8's text layer has a WS sentence giving it as 100 kΩ, but that sentence does not appear on the rendered page (ds p. 10). The visible document does not give the value.
- **LR has no stated internal pull.** The library comment "tie to GND or VDD" is correct (ds p. 10). The source does not say what LR does if left open.
- **Sound hole at the minimum.** The library uses a 0.5 mm hole, the smallest the source recommends (ds p. 17). The source gives no maximum and no acoustic effect of a larger hole.
- **Lifecycle.** The package README says TDK's product page marks the part EOL. The shipped data sheet does not say so.
- **ICS-40300** has no shipped PDF, so its pin map and land in `src/ics40300.cohdl` are not checked here.

Not stated by the source:

- An ESD rating (only an ESD caution, ds p. 8) or moisture sensitivity level.
- Electrical limits over temperature for Tables 1–3; they are 25 °C values (ds pp. 4–5).
- Package-to-port mechanical tolerances beyond Figure 15 (ds p. 19).
