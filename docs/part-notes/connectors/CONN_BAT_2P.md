# S2B-XH-A(LF)(SN) — JST XH 2-circuit side-entry through-hole header, 2.5 mm pitch

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::BAT_CONN_2P",
  "parts": ["connectors::CONN_BAT_2P"],
  "identities": [
    {"mfr": "JST", "mpn": "S2B-XH-A(LF)(SN)", "lcsc": "C157931"}
  ],
  "sources": [
    {"id": "ds", "title": "JST XH connector catalogue (2.5 mm pitch wire-to-board, crimp style)", "revision": "Undated catalogue, 9 pages; no revision printed",
     "url": "https://www.jst-mfg.com/product/pdf/eng/eXH.pdf", "sha256": "9426b136902f11900825077535e5c65032b7fbc31ffb59c5e9e1f463bb20fb90", "local": "lib/connectors/docs/jst-xh.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages. In this PDF they equal the printed page numbers (‑1‑ to ‑9‑). The URL is the one in `lib/connectors/docs/README-board.md`. It was not fetched again, so the agent has not checked that it still serves the same bytes.

## Summary

S2B-XH-A is the 2-circuit **side-entry, through-hole** header of JST's XH wire-to-board series. It has two □0.64 mm posts on 2.5 mm pitch and mates with the XHP-2 housing (ds pp. 2, 4, 5). The series is rated 3 A (AWG #22) and 250 V AC/DC, from −25 to +85 °C (ds p. 1). The library binds it to a battery connector. **In connectors ≤0.2.x the library land `FP_BAT_2P_2P5` is two SMD pads with no holes, so this through-hole header cannot be mounted on it; fixed in connectors 0.3.0 (PR #89), which gives it two plated holes** (see Pitfalls). The XH SMT version exists only with 3, 4 and 6 circuits (ds p. 6), so there is no 2-circuit SMT XH that would fit the library land.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Current rating | — | — | 3 | A | AC/DC, AWG #22 | ds p. 1 |
| Voltage rating | — | — | 250 | V | AC/DC | ds p. 1 |
| Contact resistance | — | — | 10 | mΩ | Initial | ds p. 1 |
| Contact resistance | — | — | 20 | mΩ | After environmental tests | ds p. 1 |
| Insulation resistance | 1000 | — | — | MΩ | — | ds p. 1 |
| Withstanding voltage | 1000 | — | — | V AC | 1 minute, no breakdown or flashover | ds p. 1 |
| Applicable wire | AWG #30 | — | AWG #22 | — | Insulation O.D. 0.9 to 1.9 mm | ds p. 1 |
| Applicable PCB thickness | — | 1.6 | — | mm | — | ds p. 1 |
| Pitch | — | 2.5 | — | mm | Hole pitch tolerance ±0.05, non-accumulating | ds p. 2, p. 5 |
| Post section | — | 0.64 | — | mm | Square post | ds p. 5 |
| Dimension A / B (2 circuits) | — | 2.5 / 7.4 | — | mm | A = post span, B = body width | ds p. 5 |
| Dimension C (S2B-XH-A) | — | 9.2 | — | mm | Hole row to the "side edge of header on PCB" | ds p. 2, p. 5 |

Materials (ds p. 5): post brass, copper-undercoated, tin-plated; wafer PA 66, natural (white). Note 1 on p. 5: the product shows "(LF)(SN)" on its label, which is the suffix of the library MPN.

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| None stated; the catalogue gives only the ratings under Key parameters | — | — | ds p. 1 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Temperature range (including temperature rise from applied current) | −25 | +85 | °C | ds p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | P1 | No. 1 circuit | Power | Library: battery positive | ds p. 2, p. 5 |
| 2 | P2 | circuit 2 | Power | Library: battery negative | ds p. 2, p. 5 |

The catalogue numbers the circuits but gives them no function; "battery positive/negative" is the library's assignment (`lib/connectors/src/fpc/fpc.cohdl`).

## Typical application

- Mates with the XHP-2 housing (ds p. 4) and SXH-001T-P0.6, SXH-002T-P0.6 or SXH-001T-P0.6N crimp contacts. Choose the contact by wire: #26 to #22 for -001T-P0.6N, #30 to #26 for -002T-P0.6, #28 to #22 for -001T-P0.6 (ds p. 3).
- The 3 A rating is for AWG #22 (ds p. 1). Smaller wire is allowed (ds p. 1), but the catalogue gives no derating for it.

## Layout notes

- Side-entry PC board layout, viewed from the connector mounting surface (ds p. 2, note 1). It has two φ0.9 (+0.1/0) mm holes at 2.5 ± 0.05 mm. The header body runs C = 9.2 mm from the hole row to the "side edge of header on PCB" (ds p. 2; C from p. 5). With the body drawn above the holes, No. 1 circuit is the **right-hand** hole (ds p. 2, side-entry figure).
- JST calls the hole sizes reference values. It suggests a larger hole for hard glass-cloth board material (ds p. 2, note 4).
- The body is B = 7.4 mm wide (ds p. 5). The side view gives 4.5, 7 and (6.1) mm, and (14.3) mm with the housing mated (ds p. 2).

## Pitfalls

- **connectors ≤0.2.x: the library land does not fit the part; fixed in connectors 0.3.0 (PR #89).** In ≤0.2.x, `FP_BAT_2P_2P5` is two SMD rectangles, 1.2 × 1.6 mm at ±1.25 mm, with no drill (`lib/connectors/src/fpc/fpc.cohdl`). S2B-XH-A is a through-hole header with □0.64 posts that need φ0.9 holes (ds pp. 2, 5). A board built from ≤0.2.x has no holes for the posts and must be re-laid out. 0.3.0 keeps the footprint, pad and pin names and uses two plated holes, 1.0 mm drill on 1.7 mm round lands.
- connectors ≤0.2.x: the library courtyard is 3.9 × 3.0 mm. The body is 7.4 mm wide and runs 9.2 mm from the hole row (ds p. 5), so that courtyard does not reserve the body's space. 0.3.0 uses the body plus 0.5 mm, 8.4 × 12.5 mm, and outlines the body on the silkscreen.
- Pin 1 direction: the datasheet puts No. 1 circuit on the right when the body extends upward from the holes in the mounting-surface view (ds p. 2). Library pad 1 (−x) is correct only if the body extends towards +y. The ≤0.2.x land has no body outline; 0.3.0 draws the body towards +y.
- connectors ≤0.2.x: the device comment calls it a "2-pin battery spring clip". S2B-XH-A is a shrouded wire-to-board header (ds pp. 1, 5), not a spring clip. 0.3.0 describes it as the header.
- XH SMT headers (S*B-XH-SM4-TB) exist only with 3, 4 and 6 circuits (ds p. 6). Do not change the MPN to an SMT XH to fit the land.

## Not established by the sources

- **connectors ≤0.2.x, library vs. datasheet: land type; fixed in connectors 0.3.0 (PR #89).** The ≤0.2.x land is SMD (1.2 × 1.6 mm, no hole); the datasheet layout is two φ0.9 mm holes (ds p. 2). This is a fabrication-blocking mismatch.
- **connectors ≤0.2.x, library vs. datasheet: courtyard; fixed in connectors 0.3.0 (PR #89).** It is 3.9 × 3.0 mm; the body is 7.4 mm wide and C = 9.2 mm deep (ds p. 5).
- Copper pad diameter for the φ0.9 holes. The catalogue gives only hole sizes (ds p. 2). The 1.7 mm land and the 1.0 mm drill (the top of JST's 0.9 +0.1/−0 mm range) in connectors 0.3.0 are the library's choice.
- Per-contact current derating with temperature or for wire thinner than AWG #22.
- Storage conditions, soldering profile (wave or hand), mating cycles.
- Battery polarity: the catalogue assigns no function to the circuits. Which circuit is positive depends on the mating battery's harness.
