# XC6206P182MR-G — 1.8 V, 3-terminal CMOS LDO (no enable), SOT-23

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/ldo::XC6206",
  "variants": ["SOT23"],
  "parts": ["@contrib/ldo::LDO_XC6206P182MR"],
  "identities": [
    {"mfr": "Torex Semiconductor", "mpn": "XC6206P182MR-G", "lcsc": "C2831490"}
  ],
  "sources": [
    {"id": "ds", "title": "XC6206 Series Low ESR Cap. Compatible Positive Voltage Regulators", "revision": "ETR0305_004b (no date printed)",
     "url": "https://www.torexsemi.com", "sha256": "8fda1e40ea3e7c17178353aceba9c9a9794e56a1960d02b105c576429ce418ed", "local": "lib/@contrib/ldo/docs/xc6206.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "N/17". The recorded URL is only the Torex home page (package README). Neither the README nor the PDF gives a document URL.

## Summary

The XC6206 is a 3-terminal CMOS positive LDO (VIN, VSS, VOUT) with a foldback current limiter and no enable pin (ds p. 1, p. 3). It is stable with 1.0 µF ceramic capacitors (ds p. 1, p. 6). The XC6206P182MR-G decodes as 1.8 V output (18), ±2% accuracy (2), SOT-23 on 3,000/reel (MR), halogen- and antimony-free (-G) (ds p. 2). Choose it for a light 1.8 V rail where 1 µA typical supply current matters (ds p. 4). The 1.8 V type is guaranteed for only 80 mA (ds p. 5), the SOT-23 dissipation limit is 250 mW (ds p. 3), and the input maximum is 6.0 V (ds p. 4). Do not use it for loads above about 80 mA, or where the rail must be switched off.

## Key parameters

Values for VOUT(T) = 1.8 V, ±2% grade. Ta = 25 °C, and VIN = VOUT(T) + 1.0 V = 2.8 V unless stated (ds p. 4, Note 1). Rows citing p. 5 come from the per-voltage chart E-1 to E-6.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Output voltage VOUT(E) | 1.764 | 1.8 | 1.836 | V | IOUT = 30 mA | ds p. 4, p. 5 |
| Maximum output current IOUTMAX | 80 | — | — | mA | VOUT(T) = 1.8 V (chart E-2) | ds p. 4, p. 5 |
| Load regulation ΔVOUT | — | — | 45 | mV | 1 mA ≤ IOUT ≤ 50 mA (VOUT(T) ≤ 1.8 V condition) | ds p. 4, p. 5 |
| Dropout voltage Vdif1 | — | 150 | 390 | mV | IOUT = 30 mA | ds p. 4, p. 5 |
| Dropout voltage Vdif2 | — | 350 | 780 | mV | IOUT = 60 mA (VOUT(T) ≤ 1.8 V condition) | ds p. 4, p. 5 |
| Supply current IDD | — | 1.0 | 3.0 | µA | VOUT open (test circuit 2) | ds p. 4, p. 6 |
| Line regulation ΔVOUT/(ΔVIN·VOUT) | — | 0.05 | 0.25 | %/V | VOUT(T) + 1.0 V ≤ VIN ≤ 6.0 V, IOUT = 30 mA | ds p. 4 |
| Output voltage temperature characteristic | — | ±100 | — | ppm/°C | IOUT = 30 mA, -40 °C ≤ Topr ≤ 85 °C | ds p. 4 |
| Short-circuit current Ishort | — | 130 | — | mA | VIN = VOUT + 1.5 V, VOUT = VSS | ds p. 4, p. 5 |

Dropout is defined as VIN1 − VOUT1, where VOUT1 is 98% of the output with VOUT(T) + 1.0 V applied (ds p. 4, Note 5).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Input voltage VIN | 7.0 | V | ds p. 3 |
| Output current IOUT (footnote: IOUT = Pd / (VIN − VOUT)) | 500 | mA | ds p. 3 |
| Output voltage VOUT | VSS − 0.3 to VIN + 0.3 | V | ds p. 3 |
| Power dissipation Pd, SOT-23 | 250 | mW | ds p. 3 |
| Storage temperature Tstg | -55 to +125 | °C | ds p. 3 |

## Recommended operating conditions

The datasheet has no separate recommended-operating-conditions table. The input range comes from the electrical-characteristics table. The ambient range is listed both in Features and in the absolute maximum ratings table.

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Input voltage VIN | 1.8 | 6.0 | V | ds p. 4 |
| Operating ambient temperature Topr | -40 | +85 | °C | ds p. 1, p. 3 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VSS | VSS | ground | Ground | ds p. 2 |
| 2 | VOUT | VOUT | power output | Output | ds p. 2 |
| 3 | VIN | VIN | power input | Power input | ds p. 2 |

SOT-23 top view: pins 1 (VSS) and 2 (VOUT) are on one side, and pin 3 (VIN) is alone on the opposite side (ds p. 2). The other packages number differently: SOT-89 and TO-92 are VSS 1, VIN 2, VOUT 3, and USP-6B is VSS 2, VIN 4, VOUT 6 with 1/3/5 NC (ds p. 2).

## Typical application

- CIN = 1.0 µF ceramic from VIN to VSS and CL = 1.0 µF ceramic from VOUT to VSS (typical application circuit, ds p. 1; test circuit 1, ds p. 6).
- There are no configuration pins. The regulator is on whenever VIN is applied (3-terminal, ds p. 1, p. 2).
- Headroom: all limits are specified at VIN = 2.8 V (ds p. 4). Worst-case dropout is 780 mV at 60 mA (ds p. 5), so keep VIN ≥ 2.6 V at 60 mA. Above 6.0 V the part is out of specification (ds p. 4).
- Dissipation: (VIN − 1.8 V) × IOUT must stay within Pd = 250 mW for SOT-23 (ds p. 3). From 5 V this caps IOUT at about 78 mA at 25 °C. The source gives no derating or θJA.

## Layout notes

- The source gives no recommended SOT-23 land pattern. The only reference pattern is for USP-6B (ds p. 14).
- SOT-23 outline: lead pitch 1.9 ± 0.2 mm (0.95 mm to centre), lead width 0.4 +0.1/−0.05 mm, lead-tip span 2.8 ± 0.2 mm, body 1.6 +0.2/−0.1 mm x 2.9 ± 0.2 mm, height 1.1 ± 0.1 mm (ds p. 13).
- Library `SOT3P190X290X160N` uses 0.90 x 0.75 mm pads at (−1.15, ±0.95) and (+1.15, 0) mm. Its land is derived by the library, not taken from Torex (see "Not established").
- Place CIN and CL at the pins. The source states no other layout rule.

## Pitfalls

- The output current depends on the voltage code. The 1.8 V type guarantees 80 mA (ds p. 5). The 200 mA on the front page is for the 3.0 V type (ds p. 1), and 250 mA applies only to the 4.0–5.0 V codes (ds p. 5).
- The SOT-23 limit is Pd = 250 mW at 25 °C (ds p. 3). High input-to-output drops (e.g. 5 V to 1.8 V) limit current well below IOUTMAX.
- The part has no enable pin and no output discharge (3-terminal, ds p. 1–3). Use a load switch if the rail must be turned off.
- The block diagram shows a parasitic diode across the PMOS pass device from VOUT to VIN (ds p. 3). If VOUT is held above VIN, current flows back into the input.
- Do not reuse a SOT-89 or TO-92 pinout: on SOT-23, VIN is pin 3 and VOUT is pin 2 (ds p. 2).
- The ±1% grade does not exist below 2.0 V ("Not Available", ds p. 5). Do not look for an XC6206P181 part.
- The current limiter is foldback (ds p. 1): the short-circuit current is about 130 mA typical (ds p. 5). Heavy capacitive or constant-current loads at start-up may hold the output low. The source has no start-up data for this.

## Not established by the sources

- **Library comment vs datasheet ratings:** `xc6206.cohdl` describes the series as "250mA, 1.5V dropout". The front page gives 200 mA and 250 mV at 100 mA, both for the 3.0 V type (ds p. 1). 250 mA min applies only to VOUT(T) 4.0–5.0 V (ds p. 5). No "1.5 V dropout" figure appears anywhere. For the bound 1.8 V part, IOUTMAX is 80 mA min and Vdif2 is 780 mV max at 60 mA (ds p. 5).
- **SOT-23 land pattern:** Torex publishes none for SOT-23 (only USP-6B, ds p. 14). The library footprint comment derives the land from a "3.10mm maximum lead-tip span". The source's span is 2.8 ± 0.2 mm, i.e. 3.0 mm max (ds p. 13), so the stated toe goal of 0.05 mm is actually 0.10 mm. The library comment's other inputs match the source: 1.50 mm min body width, 1.70–2.10 mm pitch and 0.50 mm max lead width (ds p. 13). The land needs assembly validation.
- **Footprint name:** `SOT3P190X290X160N` does not match the source outline in IPC-style terms. The lead-tip span is 2.8 mm, not 2.9 mm (2.9 mm is the body length), and the height is 1.1 ± 0.1 mm, not 1.60 mm (1.6 mm is the body width) (ds p. 13).
- **Supply-current condition** "VCE = VIN" refers to a CE pin that this 3-terminal series does not have (ds p. 4).
- **Chart error:** for VOUT(T) = 2.1 V the ±2% maximum is printed as 2.042 V, below nominal (ds p. 5). Treat other chart cells with care.
- **Not given:** θJA, θJC, maximum junction temperature, Pd derating, thermal shutdown, PSRR (graph only, ds p. 12), output noise, minimum CL, CL ESR range, and any reverse-current or reverse-voltage rating. All electrical limits are at 25 °C only (ds p. 4).
- **Other variants:** the library's `pins[SOT89]` has no pad for the SOT-89 tab, and the source does not say which terminal the tab connects to (ds p. 2, p. 13). The library's `pins[USP6B]` has no pad for the USP-6B dissipation pad. The source says to connect that pad, if at all, to pin 4 (VIN), not to VSS (ds p. 2). No part binds these variants today.
- **Document date and URL:** ETR0305_004b prints no issue date. The package records only the Torex home page as its URL, with no direct document link.
