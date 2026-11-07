# SIM-S-IO3-MFF2-2 — Truphone IoT eUICC (eSIM) in MFF2 / DFN-8, 5 × 6 mm

## Identity

```json
{
  "schema": "part-note/1",
  "device": "esim::ESIM_MFF2",
  "parts": ["esim::ESIM_MFF2_TRUPHONE"],
  "identities": [
    {"mfr": "Truphone", "mpn": "SIM-S-IO3-MFF2-2", "lcsc": "C5122390"}
  ],
  "sources": [
    {"id": "ds", "title": "Truphone for Things SIM datasheet", "revision": "Dated Oct 27, 2020, 5 pages, no document number or revision",
     "url": "https://www.lcsc.com/datasheet/C5122390.pdf", "sha256": "3e8a48d3e17191ae87b8f25f60fa195fb43dbfd3bca97f01c5794cce181542b0", "local": "lib/esim/docs/truphone-sim-s-io3-mff2-2.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. The printed footer number is one less on pp. 2–5 (PDF p. 5 is printed "4"); p. 1 has none.

**No manufacturer URL.** The URL is LCSC's mirror, from `lib/esim/docs/README.md`. Neither the README nor the PDF gives a Truphone document URL. The document is marked "Confidential" (ds p. 2).

**Library history: esim ≤0.1.x had the wrong pin map and footprint numbering; fixed in esim 0.2.0 (PR #87).** In 0.1.x the device's pin numbers disagreed with the source on every pad and no orientation of the part fitted the footprint, so the eUICC's I/O never reached the library's `IO` net. 0.2.0 declares the source's pin map and numbers the footprint as the source's top view. This note describes 0.2.0. A board built with esim 0.1.x does not work; see "Library history".

## Summary

The SIM-S-IO3-MFF2-2 is Truphone's IoT SIM in the soldered MFF2 form factor. It is 5 × 6 × 1.27 mm and built on an ST33G1M2 secure element (ds p. 2). It is an eUICC: remote SIM provisioning per GSMA M2M, a bootstrap profile, and up to 10 operator profiles (ds p. 1). It works with 2G, 3G, LTE, CDMA, NB-IoT and Cat-M networks (ds p. 1). It connects to a modem's SIM port over five contacts: VCC, GND, RESET, CLK and I/O (ds p. 5). It supports supply classes A (5 V), B (3 V) and C (1.8 V) (ds p. 3). This SKU has no low-power support; that is the SIM-S-IO3-MFF2-2-LP (ds p. 2).

## Key parameters

The source is a product sheet; it gives no electrical characteristics table with limits.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply voltage classes | — | 5 / 3 / 1.8 | — | V | Class A / B / C, all formats | ds p. 3 |
| Package dimensions | — | 5 × 6 × 1.27 | — | mm | MFF2 | ds p. 2 |
| Operator profiles | — | — | 10 | — | — | ds p. 1 |
| Logical channels | — | 4 | — | — | — | ds p. 4 |
| Clock division factors F/D | 8 | — | 372 | — | 372, 64, 32, 16, 8 supported | ds p. 3 |
| Total flash | — | 1280 | — | KB | ST33G1M2M | ds p. 3 |
| Flash available to customer | — | 380 / 512 | — | KB | Standard / extended | ds p. 3 |
| NVM endurance (min) | 100k / 500k | — | — | cycles | At 25 °C, standard / extended | ds p. 3 |
| NVM data retention (min) | 10 / 25 | — | — | years | At 25 °C, standard / extended | ds p. 3 |
| Transmission protocols | — | T=0, T=1 | — | — | — | ds p. 4 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| None given. The source has no absolute maximum ratings. | — | — | ds pp. 1–5 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Temperature, MFF2 ("Extended Range MFF2") | −40 | +105 | °C | ds p. 3 |
| Supply voltage | Class C 1.8 V | Class A 5 V | V | ds p. 3 |

## Pins

The "Library pin" column is what `esim::ESIM_MFF2` (esim 0.2.0) declares for each pad number. The "Datasheet name" column is what Figure 1 (VFDFPN8 pinout, top view) and Table 1 assign to that pin number (ds p. 5).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | GND | GND | Ground | Ground supply | ds p. 5 |
| 2 | SWIO | SWIO | Input, pull-up | Not used | ds p. 5 |
| 3 | IO | I/O | Bidirectional | Data; pull-down, then pull-up after card activation | ds p. 5 |
| 4 | NC4 | NC | — | Not connected internally | ds p. 5 |
| 5 | NC5 | NC | — | Not connected internally | ds p. 5 |
| 6 | CLK | CLK | Input, pull-down | External clock | ds p. 5 |
| 7 | RST | RESET (active low) | Input, pull-down | External reset | ds p. 5 |
| 8 | VCC | VCC | Power | Power supply | ds p. 5 |

Figure 1 numbers the contacts 1–4 down the left side and 5–8 up the right side, so pin 8 is opposite pin 1 and pin 5 opposite pin 4 (ds p. 5). `FP_ESIM_MFF2_5x6mm` in 0.2.0 follows that top view: pad 1 at the top left with the pin-1 dot, pads 1–4 down the left column, pads 5–8 up the right column. `SWIO`, `NC4` and `NC5` are optional pins.

## Library history

esim ≤0.1.x declared `VCC: 1`, `RST: 2`, `CLK: 3`, `IO: 4`, `GND: 5`, `NC6: 6`, `NC7: 7`, `NC8: 8`, which matched no pad of the source's map. Its `FP_ESIM_MFF2_5x6mm` put pads 1–4 left to right in one row and pads 5–8 left to right in the other, so pad 5 was opposite pad 1, and no rotation or mirror of the source's top view gave that order (ds p. 5). In the closest placement (part pin 8 on pad 1) VCC, RST, CLK and GND reached the right part pins, but the library's `IO` (pad 4) reached a no-connect pin and the part's I/O sat on pad 7, which 0.1.x called `NC7`. In the other placement the eUICC got no supply and the modem's data line was shorted to its ground. Either way the SIM did not work.

Fixed in esim 0.2.0 (PR #87): the pin map in "Pins", counter-clockwise pad numbering, the land turned a quarter turn (columns along y, courtyard 5.6 × 6.6 mm), `#[doc]` pointing at the PDF, and the supply comment listing class A 5 V. A design on 0.1.x must update to 0.2.0, re-check its `place` rotation for the turned land, and regenerate the board.

## Typical application

- Figure 2 ("ST4SIM-200M PCB integration recommendations") wires GND, VCC, RST, CLK and I/O directly to the modem's SIM port, and leaves SWIO and both NC pins unconnected (ds p. 5).
- It shows a capacitor, marked (1), from VCC to GND at the eUICC, and a resistor, marked (2), from VCC to I/O (ds p. 5). The footnotes (1) and (2) are not on the page, and no values are given.
- SWIO is "Not used" and has an internal pull-up (ds p. 5). RESET and CLK have internal pull-downs (ds p. 5).
- The modem sets the supply class (A 5 V, B 3 V, C 1.8 V, ds p. 3).

## Layout notes

- The source gives only the package size (5 × 6 × 1.27 mm, ds p. 2), the pinout (ds p. 5), and that the package is compatible with the MFF2 package of ETSI 102 671 release 12 (ds p. 5). It calls the package VFDFPN8 (ds p. 5) and DFN8 (MFF2) (ds p. 3).
- It gives no contact size, pitch, row spacing, land pattern, exposed pad or paste data. The library's 0.8 × 0.8 mm pads, 1.3 mm pitch and 3.2 mm column spacing come from elsewhere; esim 0.2.0 carried them over from 0.1.x unchanged and its README says so.

## Pitfalls

- Use esim 0.2.0 or later. With esim ≤0.1.x the library pin names do not match the part; see "Library history".
- RESET is active low (overbar in Figure 1 and Table 1, ds p. 5). Its internal pull-down holds the card in reset if left open.
- The plain SKU has "Low Power Support: No" (ds p. 2). If the design relies on PSM/eDRX power saving at the SIM, check whether the -LP SKU is needed. The source lists "Power saving features (PSM, eDRX)" among its software features without saying which SKUs have them (ds p. 1, p. 4).
- The extended range −40 °C to +105 °C is stated for MFF2 (ds p. 3). The 4FF formats are −25 °C to +85 °C (ds p. 3).

## Not established by the sources

Open points between the library (esim 0.2.0) and the source:

- **Land pattern.** Contact size (0.8 × 0.8 mm), pitch (1.3 mm), column spacing (3.2 mm) and the absence of a centre pad are not in the source (ds pp. 1–5). Which body side (5 mm or 6 mm) the contact columns run along is not in the source either; 0.2.0 runs them along the 6 mm side. The library README says the land must be checked against the ETSI TS 102 671 MFF2 drawing or ST's VFDFPN8 drawing before fabrication.
- **Figure 2's part name.** Figure 2 is titled "ST4SIM-200M PCB integration recommendations", an ST part name, not the Truphone SKU (ds p. 5). Its pin names match Figure 1.

Not stated by the source:

- Any absolute maximum rating, supply current, I/O levels, clock frequency range or ESD rating.
- The values of the capacitor (1) and pull-up (2) in Figure 2 (ds p. 5).
- A package drawing, land pattern, MSL or reflow profile.
- Which supply classes the MFF2 SKU supports in practice; the class list is given for "all formats" (ds p. 3).
