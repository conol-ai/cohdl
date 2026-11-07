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

**Warning: the library's pin numbers disagree with the source on every pad, and the footprint's pad order matches no orientation of the part.** However the part is turned on the footprint, the eUICC's I/O pin is not on the library's `IO` net, so the SIM cannot work. In one orientation the eUICC also gets no supply and its I/O is shorted to ground. See "Pins", "Placement as drawn" and the first items under "Not established by the sources". Do not use `ESIM_MFF2_TRUPHONE` on a board until the device and footprint are fixed.

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

The "Library pin" column is what `esim::ESIM_MFF2` declares for each pad number. The "Datasheet name" column is what Figure 1 (VFDFPN8 pinout, top view) and Table 1 assign to that pin number (ds p. 5). **No pad agrees.**

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | VCC | GND | Ground | Ground supply | ds p. 5 |
| 2 | RST | SWIO | Input, pull-up | Not used | ds p. 5 |
| 3 | CLK | I/O | Bidirectional | Data; pull-down, then pull-up after card activation | ds p. 5 |
| 4 | IO | NC | — | Not connected internally | ds p. 5 |
| 5 | GND | NC | — | Not connected internally | ds p. 5 |
| 6 | NC6 | CLK | Input, pull-down | External clock | ds p. 5 |
| 7 | NC7 | RESET (active low) | Input, pull-down | External reset | ds p. 5 |
| 8 | NC8 | VCC | Power | Power supply | ds p. 5 |

Pad positions do not follow the source either. Figure 1 numbers the contacts 1–4 down the left side and 5–8 up the right side, so pin 8 is opposite pin 1 and pin 5 opposite pin 4 (ds p. 5). `FP_ESIM_MFF2_5x6mm` puts pads 1–4 left to right in one row and pads 5–8 left to right in the other, so pad 5 is opposite pad 1. No rotation or mirror of the source's top view gives that order.

## Placement as drawn

What a board built from the library connects. The contact rows run along the footprint's x axis, so the part's two columns of four (Figure 1, ds p. 5) must lie along the rows. Turning the source's top view a quarter turn gives only two such placements, and neither puts pin 1 at the footprint's pin-1 dot (pad 1):

- **A:** part pins 8, 7, 6, 5 on pads 1–4 and pins 1, 2, 3, 4 on pads 5–8 (pin 1 on pad 5).
- **B:** part pins 4, 3, 2, 1 on pads 1–4 and pins 5, 6, 7, 8 on pads 5–8 (pin 1 on pad 4).

| Library pad and net | Placement A: part pin reached | Placement B: part pin reached |
|---|---|---|
| 1 VCC | 8 VCC | 4 NC: no supply to the eUICC |
| 2 RST | 7 RESET | 3 I/O: reset line drives data |
| 3 CLK | 6 CLK | 2 SWIO (unused) |
| 4 IO | 5 NC: modem data line goes nowhere | 1 GND: modem data line shorted to the eUICC's ground |
| 5 GND | 1 GND | 5 NC: eUICC ground open |
| 6 NC6 | 2 SWIO (unused) | 6 CLK: clock floating |
| 7 NC7 | 3 I/O: the eUICC's data pin is left open | 7 RESET: floating (internal pull-down) |
| 8 NC8 | 4 NC | 8 VCC: supply pin open |

Placement A gets supply, ground, reset and clock right; only I/O is wrong (library `IO` on pad 4, the part's I/O on pad 7). It also puts the part's pin 1 on pad 5, away from the silkscreen dot, so an assembler following the dot cannot reach it. Placement B is wrong on every functional pin.

## Typical application

- Figure 2 ("ST4SIM-200M PCB integration recommendations") wires GND, VCC, RST, CLK and I/O directly to the modem's SIM port, and leaves SWIO and both NC pins unconnected (ds p. 5).
- It shows a capacitor, marked (1), from VCC to GND at the eUICC, and a resistor, marked (2), from VCC to I/O (ds p. 5). The footnotes (1) and (2) are not on the page, and no values are given.
- SWIO is "Not used" and has an internal pull-up (ds p. 5). RESET and CLK have internal pull-downs (ds p. 5).
- The modem sets the supply class (A 5 V, B 3 V, C 1.8 V, ds p. 3).

## Layout notes

- The source gives only the package size (5 × 6 × 1.27 mm, ds p. 2), the pinout (ds p. 5), and that the package is compatible with the MFF2 package of ETSI 102 671 release 12 (ds p. 5). It calls the package VFDFPN8 (ds p. 5) and DFN8 (MFF2) (ds p. 3).
- It gives no contact size, pitch, row spacing, land pattern, exposed pad or paste data. The library's 0.8 × 0.8 mm pads, 1.3 mm pitch and 3.2 mm row spacing come from elsewhere.

## Pitfalls

- Do not wire to the library pin names; see the warning and "Pins". Wire from Figure 1 and Table 1 (ds p. 5).
- RESET is active low (overbar in Figure 1 and Table 1, ds p. 5). Its internal pull-down holds the card in reset if left open.
- The plain SKU has "Low Power Support: No" (ds p. 2). If the design relies on PSM/eDRX power saving at the SIM, check whether the -LP SKU is needed. The source lists "Power saving features (PSM, eDRX)" among its software features without saying which SKUs have them (ds p. 1, p. 4).
- The extended range −40 °C to +105 °C is stated for MFF2 (ds p. 3). The 4FF formats are −25 °C to +85 °C (ds p. 3).

## Not established by the sources

Disagreements between the library and the source, most serious first:

- **Pin map.** The device declares `VCC: 1`, `RST: 2`, `CLK: 3`, `IO: 4`, `GND: 5`, `NC6: 6`, `NC7: 7`, `NC8: 8`. The source gives 1 GND, 2 SWIO, 3 I/O, 4 NC, 5 NC, 6 CLK, 7 RESET (active low), 8 VCC (ds p. 5). By number, no pad matches.
- **Footprint pad numbering.** `FP_ESIM_MFF2_5x6mm` numbers the second row 5–8 in the same direction as the first, so pad 5 is opposite pad 1. The source's top view puts pin 8 opposite pin 1 (ds p. 5). No orientation of the part matches the footprint's numbering, and the pin-1 dot cannot coincide with the part's pin 1 (see "Placement as drawn").
- **I/O on the wrong pad in every placement.** In the closest placement (A), VCC, RST, CLK and GND reach the right part pins, but the library's `IO` (pad 4) reaches a no-connect pin and the part's I/O sits on pad 7, which the library calls `NC7`. In placement B the eUICC gets no supply and the data line is shorted to its ground. Either way the SIM does not work. Fixing the names alone is not enough; the footprint numbering and the pin-1 marker must change with them.
- **The README and part comment say the source cannot settle the pinout.** `docs/README.md` says "pin numbers above are this library's logical map; verify contact assignment against the bound eUICC's datasheet", and that the Truphone document "describes the SIM product..., not a mechanical MFF2 package drawing". The shipped PDF has a full pinout and pin table on its last page (ds p. 5).
- **Documentation link.** Both `#[doc]` attributes point at `docs/README.md`, not the PDF.
- **Land pattern.** Contact size (0.8 × 0.8 mm), pitch (1.3 mm), row spacing (3.2 mm) and the absence of a centre pad are not in the source (ds pp. 1–5). Which body side (5 mm or 6 mm) the contact rows run along is not in the source either.
- **Supply comment.** The library says "supply 1.8/3.0V". The source also lists class A, 5 V (ds p. 3).
- **Pin types.** `RST` is typed `input` and `IO` `bidirectional`, which fit the source's RESET and I/O, but on the wrong pads. SWIO has no library pin at all.
- **Figure 2's part name.** Figure 2 is titled "ST4SIM-200M PCB integration recommendations", an ST part name, not the Truphone SKU (ds p. 5). Its pin names match Figure 1.

Not stated by the source:

- Any absolute maximum rating, supply current, I/O levels, clock frequency range or ESD rating.
- The values of the capacitor (1) and pull-up (2) in Figure 2 (ds p. 5).
- A package drawing, land pattern, MSL or reflow profile.
- Which supply classes the MFF2 SKU supports in practice; the class list is given for "all formats" (ds p. 3).
