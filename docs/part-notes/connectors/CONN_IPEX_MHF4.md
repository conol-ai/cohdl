# CONMHF4-SMD-G-T — Linx (TE) MHF4-type 50 Ω SMT RF jack, 0 to 6 GHz

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::IPEX_MHF4",
  "parts": ["connectors::CONN_IPEX_MHF4"],
  "identities": [
    {"mfr": "Linx", "mpn": "CONMHF4-SMD-G-T", "lcsc": "C3173267"}
  ],
  "sources": [
    {"id": "ds", "title": "Linx / TE Connectivity CONMHF4-SMD-G, MHF4 jack PCB surface-mount connector", "revision": "09/22 Original, 6 pages",
     "url": "https://www.te.com/en/product-CONMHF4-SMD-G-T.html", "sha256": "63fedb5a916e68998c0ec4aafd5797065a19ffb404fb68fbbf44ad97910a7513", "local": "lib/connectors/docs/conmhf4-smd-g-t.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages; they equal the printed page numbers (p. 1 has none). **The URL is flagged.** It is the TE product page from `lib/connectors/docs/README-board.md`, not a link to the PDF, and it was not fetched again. The repository records no direct PDF URL.

## Summary

CONMHF4-SMD-G is an MHF4-type jack (male centre pin) for reflow soldering. It is 50 Ω, works from 0 Hz to 6 GHz, and is gold plated throughout (ds pp. 1–2). The -T suffix is tape and reel, 10,000 per reel (ds p. 2). The body is 2.0 × 2.0 mm (ds p. 2). It has one signal land and three ground lands (ds p. 3, Figure 2). It is rated 60 V RMS (ds p. 1) and only 30 mating cycles (ds p. 4), so it suits an internal antenna lead that is rarely unplugged, not a user-facing port. The datasheet gives no current rating.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Impedance | — | 50 | — | Ω | — | ds p. 1 |
| Frequency range | 0 | — | 6 | GHz | — | ds p. 1 |
| Voltage rating | — | — | 60 | V RMS | — | ds p. 1 |
| Contact resistance, centre / outer | — | — | 20.0 | mΩ | Each | ds p. 1 |
| Insulation resistance | 500 | — | — | MΩ | — | ds p. 1 |
| Insertion loss | — | — | 0.58 | dB | 2.4 GHz | ds p. 1 |
| Insertion loss | — | — | 1.57 | dB | 5 GHz | ds p. 1 |
| VSWR | — | — | 1.1 | — | 2.4 GHz | ds p. 1 |
| VSWR | — | — | 1.8 | — | 5 GHz | ds p. 1 |
| Insertion loss / VSWR | — | — | 0.33 / 1.1 | dB / — | 400–960 MHz ("Low-Band Cellular/ISM/LPWA") | ds p. 3 |
| Insertion loss / VSWR | — | — | 0.43 / 1.8 | dB / — | 1164–1609 MHz (GNSS) | ds p. 3 |
| Insertion loss / VSWR | — | — | 1.60 / 1.1 | dB / — | 1427–5000 MHz ("Midband Cellular") | ds p. 3 |
| Durability | 30 | — | — | cycles | Mating | ds p. 4 |
| Body | — | 2.0 × 2.0 | — | mm | Top view | ds p. 2 |
| Weight | — | 0.01 | — | g | — | ds p. 4 |

Materials (ds p. 2): body LCP, UL94V-0. Outer and centre contacts are brass, gold finish. Signal and ground contacts are phosphor bronze, gold finish.

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Reflow, maximum RoHS profile peak (Figure 3) | 255 | °C | ds p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Temperature range | −40 | +90 | °C | ds p. 4 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | RF1 | Signal | Bidirectional | Centre conductor, 0.60 × 0.60 mm land | ds p. 3 |
| 2 | GND | GND | Ground | Three ground lands (top 0.5 × 0.5 mm, two sides 0.58 × 1.58 mm) share pad 2 | ds p. 3 |

## Typical application

- Feed RF1 with a 50 Ω line from the radio or its matching network. The datasheet gives no feed-line geometry, stack-up or via fence (ds p. 3).
- Connect all three ground lands to the RF ground plane. Figure 2 labels all three "GND" (ds p. 3).

## Layout notes

- Figure 2, "Recommended PCB Dimensions" (ds p. 3), measured from a horizontal reference line:
  - side ground lands: outer span 2.5 mm, inner gap 1.34 mm, 1.58 mm tall;
  - top ground land: 0.5 mm wide, from 0.75 to 1.25 mm on one side of the line;
  - signal land: 0.60 mm wide, from 0.65 to 1.25 mm on the other side.
  The 1.5 mm dimension runs to the connector's ground legs drawn in the figure, not to copper.
- **Library land check, on ds p. 3, Figure 2.** Rendered at 400 dpi, the side lands are centred on the reference line. `FP_IPEX_MHF4` places the signal at (0, +0.95), 0.6 × 0.6; the top ground at (0, −1.0), 0.5 × 0.5; and the side grounds at (±0.96, 0), 0.58 × 1.58. These are the Figure 2 values: (0.65 + 1.25) / 2 = 0.95, (0.75 + 1.25) / 2 = 1.0, (2.5 + 1.34) / 4 = 0.96, (2.5 − 1.34) / 2 = 0.58. All match.
- Top or bottom view: Figure 2 is mirror-symmetric about its vertical axis, and signal and top ground both sit on that axis. A top view and a bottom view therefore give the same land, so this footprint cannot be mirrored.
- Reflow per Figure 3: the recommended RoHS profile and a maximum RoHS profile with a 255 °C peak (ds p. 4).

## Pitfalls

- Mating cycles: 30 minimum (ds p. 4). Do not use it as a frequently reconnected test port.
- The signal land (0.6 mm wide) is wider than the top ground land (0.5 mm). Do not swap them when hand-drawing a land (ds p. 3).
- MHF4 is the mating interface named by the datasheet (ds p. 1). The datasheet does not say which other mini-coax families (other makers' "MHF4", U.FL, W.FL and so on) mate with it.

## Not established by the sources

- Current rating and power handling.
- Solder-mask and paste openings, and keep-outs under the connector.
- The mating plug part numbers, and the mating and unmating forces.
- Table 1 gives VSWR 1.8 at 5 GHz (ds p. 1). Table 3 gives 1.1 for the 1427–5000 MHz band (ds p. 3). The two tables disagree for 5 GHz; use the worse figure.
- A direct official PDF URL (see Identity).
