# E1SB40E001G00E (library MPN, unverified) — 40 MHz Hosonic E1SB-series crystal, 2.0 × 1.6 mm, 4 pads

## Identity

```json
{
  "schema": "part-note/1",
  "device": "osc::Crystal40MHz",
  "parts": ["osc::XTAL_40MHZ"],
  "identities": [
    {"mfr": "Hosonic", "mpn": "E1SB40E001G00E", "lcsc": null}
  ],
  "sources": [
    {"id": "hos", "title": "Hosonic SMD Seam Sealing Crystals, E1SB Series 2.0x1.6x0.50mm", "revision": "No revision printed; PDF modification date 2026-07-20",
     "url": "https://www.hosonic.com/Upload/FTP/P-C-S-E1SB.pdf", "sha256": "af74ec52f0d175cd03ff64cd71a926d839fe79b023a21a3e2c2b6bceb7b7e62b", "local": "lib/osc/docs/hosonic-e1sb.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. The source is a single image-only page with no extractable text; it was read from a rendering. `lcsc_map.json` has no row for this MPN.

**Warning: the MPN is unverified, and osc ≤0.1.x binds this 2.0 × 1.6 mm part to a 3.2 × 2.5 mm footprint; the footprint is fixed in osc 0.2.0 (PR #92), which uses the E1SB land.** See "Not established by the sources".

## Summary

`XTAL_40MHZ` is intended as the 40 MHz main crystal for the ESP32-S3. Its MPN, `E1SB40E001G00E`, is in the Hosonic E1SB series: seam-sealed 2.0 × 1.6 mm crystals, 16–96 MHz fundamental (hos p. 1). The pinned source is the series sheet. It gives series-wide limits only: at 40 MHz, ESR up to 60 Ω, CL 6–16 pF or as specified, ±10 ppm tolerance and ±15 ppm stability or as specified (hos p. 1). The library's own comment says the MPN only "follows the Hosonic E1SB naming scheme" (osc ≤0.1.x) or is "patterned on the 48MHz part's E1SB48E001G00E" (osc 0.2.0), and must be confirmed before fitting. The part's identity, and every part-specific parameter, are therefore not established.

## Key parameters

Series values from the E1SB sheet (hos p. 1). They apply to a 40 MHz E1SB part only where the ordering code does not specify otherwise.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Frequency range (series) | 16 | — | 96 | MHz | Fundamental mode | hos p. 1 |
| Load capacitance CL | 6 | — | 16 | pF | "6 ~ 16pF, or specify" | hos p. 1 |
| Frequency tolerance | -10 | — | 10 | ppm | at 25 °C; "or specify" | hos p. 1 |
| Temperature stability | -15 | — | 15 | ppm | ref. 25 °C; "or specify" | hos p. 1 |
| ESR | — | — | 60 | Ω | 30.0–74.9 MHz band (covers 40 MHz) | hos p. 1 |
| Shunt capacitance C0 | — | — | 2 | pF | — | hos p. 1 |
| Drive level | 1 | 50 | 100 | µW | — | hos p. 1 |
| Aging | -3 | — | 3 | ppm | per year at 25 °C | hos p. 1 |
| Insulation resistance | 500 | — | — | MΩ | at DC 100 V | hos p. 1 |

## Absolute maximum ratings

The sheet has no absolute maximum ratings table and no reflow profile. The only limit of that kind is storage temperature.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Storage temperature | -40 to +85 | °C | hos p. 1 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature ("-30 ~ +85°C, or specify") | -30 | +85 | °C | hos p. 1 |
| Drive level | 1 | 100 | µW | hos p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | XIN | Crystal | passive | Crystal electrode; top view lower left | hos p. 1 |
| 2 | GND | GND | passive | Lid ground; top view lower right | hos p. 1 |
| 3 | XOUT | Crystal | passive | Crystal electrode; top view upper right | hos p. 1 |
| 4 | GND | GND | passive | Lid ground; top view upper left | hos p. 1 |

Hosonic's pin table: 1 and 3 Crystal, 2 and 4 GND, "#2 & #4 are connected to Lid" (hos p. 1). The library's pin numbering agrees. The osc ≤0.1.x footprint does not; see below.

## Typical application

- Pierce oscillator: pads 1 and 3 to the oscillator pins, each with a load capacitor to ground. Pads 2 and 4 to ground (hos p. 1).
- The source gives no load-capacitor values. The correct CL depends on the ordering code, which the sheet does not decode (hos p. 1).
- Keep drive level at or below 100 µW (hos p. 1).

## Layout notes

- E1SB recommended pad layout (hos p. 1): four 0.85 × 0.75 mm lands with a 0.50 mm horizontal and a 0.30 mm vertical gap. Pad centres are at ±0.675 mm, ±0.525 mm.
- Body 2.00 ± 0.10 × 1.60 ± 0.10 mm, height 0.45 ± 0.1 mm per the side view (hos p. 1).
- `osc::FP_Crystal_SMD_2016_4Pin_E1SB` (used by `XTAL_48MHZ`) implements this land. In osc ≤0.1.x `XTAL_40MHZ` does not use it; it uses `FP_Crystal_3225`, see below. osc 0.2.0 (PR #92) moves `XTAL_40MHZ` to `FP_Crystal_SMD_2016_4Pin_E1SB`.

## Pitfalls

- A board built from `XTAL_40MHZ` with osc ≤0.1.x gets a 3.2 × 2.5 mm land under a 2.0 × 1.6 mm part (hos p. 1). Expect skew, opens or tombstoning. Use osc 0.2.0 or later (PR #92), which uses the E1SB land, or substitute a 3225 crystal that has its own source.
- A 90° rotation puts the crystal on the ground pair (hos p. 1).
- Operating range is -30 °C to +85 °C unless the ordering code specifies otherwise (hos p. 1).

## Not established by the sources

- **osc ≤0.1.x: wrong footprint; fixed in osc 0.2.0 (PR #92).** In ≤0.1.x, `XTAL_40MHZ` uses `FP_Crystal_3225`, the Abracon ABM8 3.2 × 2.5 mm land: 1.3 × 1.05 mm pads at x = ±1.15 mm, y = ±0.875 mm. The E1SB package is 2.0 × 1.6 mm, and its recommended land is 0.85 × 0.75 mm pads at ±0.675 mm, ±0.525 mm (hos p. 1). The ≤0.1.x `crystal_40mhz.cohdl` comment ("3225 4-pad package") and `docs/README.md` heading ("3225 4-pad") disagree with the series the MPN names. The E1SB sheet title is "2.0x1.6x0.50mm" (hos p. 1). osc 0.2.0 binds `FP_Crystal_SMD_2016_4Pin_E1SB` and rewrites both texts; `FP_Crystal_3225` stays for `XTAL_8M`.
- **Unverified MPN (all versions).** osc ≤0.1.x says the MPN "follows the Hosonic E1SB naming scheme used by the 48MHz part ... confirm the exact vendor stock part against the distributor list before fitting", and its README says "confirm vendor stock before fitting". osc 0.2.0 marks it "MPN UNVERIFIED". The sheet has no ordering-code key (hos p. 1), so whether `E1SB40E001G00E` exists, and what CL, tolerance and stability it has, is not established.
- **Load-capacitance guidance.** The osc ≤0.1.x comments say "10pF load class" and "C1=C4 10pF load caps + 24nH series inductor on XTAL_N, trimmed by measurement", citing the ESP32-S3 hardware design guidelines. That document is not a pinned source. The E1SB sheet gives only CL 6–16 pF or as specified (hos p. 1) and no external-component values. osc 0.2.0 keeps the guidance but marks it as from an unpinned source, not source-checked.
- ESR at 40 MHz is up to 60 Ω for the series (hos p. 1). Whether that meets the ESP32-S3's crystal requirement is not established here.
- The sheet gives no reflow profile, MSL, motional parameters or index mark. The terminal at the pin 2 position is drawn with a cut corner; the sheet does not say whether that is an index (hos p. 1). The library's pin-1 dot is at pad 1, not at that corner; osc 0.2.0 says so in its comments.
- The sheet carries no revision or date. 2026-07-20 is the PDF modification date.
