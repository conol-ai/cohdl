# YZP0048-20048-04025-03 — Xinyangze 4-contact magnetic pogo-pin connector, 2.50 mm pitch

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::MAG_CONN_4P",
  "parts": ["connectors::CONN_MAG_4P"],
  "identities": [
    {"mfr": "Xinyangze", "mpn": "YZP0048-20048-04025-03", "lcsc": "C5126845"}
  ],
  "sources": [
    {"id": "ds", "title": "Shenzhen Xinyangze (XYZ Pogo) product specification and customer drawing YZP0048-20048-04025-03", "revision": "Specification A.0, 2022-08-08 (pp. 1–2); drawing A.1, 2022-03-24 (p. 3); packaging spec A.1 (p. 4)",
     "url": "https://www.lcsc.com/product-detail/C5126845.html", "sha256": "d2c26de7b1294bc46f5294b357ab1d7b9584fca8ca2aef185b156c550e0cdb34", "local": "lib/connectors/docs/yzp0048-20048-04025-03.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages; pp. 1, 2 and 4 print "第N页 共4页" with the same numbers. Page 3 is a rotated vector drawing with no text layer, so the agent read it from 600 dpi renders. **The URL is flagged.** It is the LCSC distributor page from `lib/connectors/docs/README-board.md`, not a manufacturer URL, and it was not fetched again. The repository records no manufacturer URL.

## Summary

YZP0048-20048-04025-03 is a four-contact spring-loaded pogo connector with gold-plated brass plungers. Two NdFeB magnets in a PA9T housing pull the mating half into place (ds pp. 1, 3). It is rated DC 12 V, 1 A (ds p. 1, p. 3). The plungers are rated for 10,000 cycles at 30 gf ± 20 % per pin at the 3.10 mm working height (ds pp. 1–3). It solders to the board through four Ø0.60 mm tails on 2.50 mm pitch (ds p. 3). The source does not number the contacts, so P1 to P4 and their functions are the library's.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Rated voltage | — | — | 12 | V | "12VAC (rms)/DC" on p. 1; "DC 12V" on the drawing | ds p. 1, p. 3 |
| Rated current | — | — | 1 | A | Per pin is not stated; tested at 12 V, 1 A static (EIA-364-70) | ds p. 1, p. 2 |
| Contact resistance | — | — | 50 | mΩ | At working height; 20 mV, 10 mA (EIA-364-23) | ds p. 2, p. 3 |
| Insulation resistance | 1000 | — | — | MΩ | 500 V DC between adjacent pins | ds p. 2 |
| Dielectric withstanding voltage | 500 | — | — | V AC | 1 minute, 60 Hz, adjacent pins | ds p. 1, p. 2 |
| Spring force | 24 | 30 | 36 | gf | Per pin at working height, 30 gf ± 20 % | ds p. 1, p. 3 |
| Durability | 10000 | — | — | cycles | 500 ± 50 cycles per hour (EIA-364-09) | ds p. 2 |
| Free height | 3.95 | 4.10 | 4.25 | mm | 4.10 ± 0.15, mounting face to plunger tip | ds p. 3 |
| Working height | — | 3.10 | — | mm | — | ds p. 3 |
| Compression limit height | — | 2.90 | — | mm | — | ds p. 3 |
| Contact pitch | — | 2.50 | — | mm | — | ds p. 3 |
| Housing | — | 21.20 × 9.17 | — | mm | Oval 6.86 high (R3.43) plus a 5.03 mm wide tab | ds p. 3 |
| Housing thickness | — | 2.80 | — | mm | Side view; tail-side boss 4.83 high | ds p. 3 |
| Magnet centres | — | 15.41 | — | mm | — | ds p. 3 |
| Tails | — | Ø0.60 × 0.70 | — | mm | Four, on the mounting face | ds p. 3 |

Materials and plating (ds pp. 1–3):
- plunger and barrel: brass C3604, 3 µ" Au over 50–80 µ" Ni;
- spring: SUS304, gold plated;
- magnet: NdFeB, zinc plated;
- housing: PA9T.

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Withstanding high temperature | 280 | °C | ds p. 1, p. 3 |
| Plunger compression (do not compress below) | 2.90 | mm height | ds p. 3 |
| Reflow test profile peak (preheat 150–180 °C for 120 s, about 40 s above 230 °C) | 250 ± 5 °C, 3 s max | — | ds p. 2 |
| Solderability test (Sn-Ag-Cu dip) | 265 ± 5 °C, 4.0–5.0 s | — | ds p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature | −20 | +70 | °C | ds p. 1 |
| Thermal shock qualification (EIA-364-32, 5 cycles) | −40 | +105 | °C | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | P1 | (unnumbered contact) | Power | Library: charge positive | ds p. 3 |
| 2 | P2 | (unnumbered contact) | Power | Library: charge negative | ds p. 3 |
| 3 | P3 | (unnumbered contact) | Passive | Library: data / ID | ds p. 3 |
| 4 | P4 | (unnumbered contact) | Passive | Library: data / ID | ds p. 3 |

## Typical application

- A charge or data dock interface. The mating half must hold four flat targets on 2.50 mm pitch, plus magnets of opposite polarity at 15.41 mm centres (ds p. 3). The source names no mating part.
- The 1 A rating is tested at 12 V static (ds p. 2). The source does not say whether it applies per pin or to the connector.

## Layout notes

- "RECOMMENDED PCB LAYOUT" (ds p. 3): four Ø2.20 mm lands with Ø0.80 mm holes at 2.50 mm pitch. The holes take the Ø0.60 × 0.70 mm tails (ds p. 3, front view).
- **Library land check.** `FP_MAG_4P` has four plated through-holes with 2.2 mm round copper and a 0.8 mm drill, at x = −3.75, −1.25, 1.25, 3.75. This matches the recommended layout.
- The courtyard is 21.7 × 9.67 mm at (0, +1.155). It is the 21.20 × 9.17 mm housing plus 0.25 mm per side, with the tab towards +y. The oval is centred on the contact row, ±3.43 mm (R3.43), and the tab reaches 9.17 − 3.43 = 5.74 mm (ds p. 3). This matches.
- The magnets need no PCB features (ds p. 3).

## Pitfalls

- **Contact order relative to the magnets.** The drawing marks the magnets "S" (left) and "N" (right) in its main view (ds p. 3). By the agent's reading, that view looks at the **tail (board) side**, not the plunger face:
  - the 4.83 mm tail-side boss and the Ø0.60 tails are drawn solid, and the magnets dashed;
  - in the side view placed to its right (third-angle projection, per the title-block symbol), the tail faces the main view.
  Seen from above the board, the plunger face is the mirror image: the "S" magnet is on the right. So the library's P1 (−x, tab towards +y) is the contact next to the **N** magnet. `yzp0048-20048-04025-03.evidence.md` says the opposite (that the main view is the contact face). The land itself is symmetric, so only the contact functions and the magnet polarity depend on this. Check it against a physical part before fixing the dock's pinout.
- Do not compress the plungers below 2.90 mm (ds p. 3).
- Operating range is only −20 to +70 °C (ds p. 1). The −40 to +105 °C figures are a thermal-shock test, not an operating range (ds p. 2).

## Not established by the sources

- **Contact numbering.** The drawing numbers no contacts (ds p. 3). P1–P4 and their functions (charge +, charge −, data/ID) are library conventions.
- **Library comments disagree with each other.** The device comment says P3 and P4 are both data/ID. The evidence record says P4 is not connected.
- **View of the main drawing**, and so which physical contact is P1. See Pitfalls; this is the agent's reading of ds p. 3 and has not been checked against a physical part.
- Whether the 1 A rating is per pin.
- The mating half, its target pad size, and the magnet pull force.
- Whether the reflow test profile on p. 2 (a qualification test: 250 ± 5 °C peak, 3 s max) is a recommended production profile. The source presents it only as a test.
- An official manufacturer URL.
