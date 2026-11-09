# CON_PICO_3W — Raspberry Pi Pico three-pad SWD debug feature (J2), 2.54 mm pitch

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::Header_1x3",
  "parts": ["connectors::HEADER_SWD_3W"],
  "identities": [
    {"mfr": "Raspberry Pi", "mpn": "CON_PICO_3W", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "Raspberry Pi Pico Datasheet", "revision": "Release 21, build date 03/07/2026 (03 July 2026), build version d4e0f1799616",
     "url": "https://datasheets.raspberrypi.com/pico/pico-datasheet.pdf", "sha256": "757ff485227493b9fcc0c2c96c4dea9de020e1d8b2b11e2aa4f9ee8b25aa89eb", "local": "lib/connectors/docs/raspberry-pi-pico-datasheet.pdf"}
  ],
  "status": "draft"
}
```

**Page numbers are PDF pages.** The printed page number is one less, because the colophon is printed page 1 on PDF page 2. For example, Figure 3 is on PDF p. 7, printed "6". The URL is the one in `lib/connectors/docs/README.md` and was not fetched again.

## Summary

This is not an orderable connector. CON_PICO_3W is the value of J2 ("DEBUG") in the Raspberry Pi Pico Rev3 schematic (ds p. 26, Figure 19). It is the three-pad Serial Wire Debug port on the board's lower edge (ds p. 24). The library uses it so that a Pico-style board (`examples/rpi-pico2`) carries the same debug pads. Contact 1 (D1) is SWCLK, 2 (D2) GND and 3 (D3) SWDIO (ds p. 26). The device itself names them only P1–P3. The datasheet gives no electrical rating for the pads. The RP2040 has internal pull-ups of nominally 60 kΩ on SWDIO and SWCLK (ds p. 24).

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| SWDIO / SWCLK internal pull-up (RP2040) | — | 60 | — | kΩ | Nominal | ds p. 24 |
| SWD pad hole centres from the board's lower edge | — | 1.6 | — | mm | Agent's reading of Figure 3; see below | ds p. 7 |
| Middle SWD pad offset from the left mounting hole | — | 5.7 | — | mm | Mounting hole 4.8 mm from the left edge, so the middle pad is on the 21 mm board's centreline | ds p. 7 |
| SWD land outer span on the carrier (SMT module footprint) | — | 6.68 | — | mm | Figure 5, outer edge of D1 land to outer edge of D3 land | ds p. 10 |
| Main 40-pin grid (not the SWD pads) | — | 2.54 | — | mm | 1 mm holes | ds p. 7 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| None stated for the SWD pads | — | — | ds p. 24 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Pico board operating temperature (including self-heating) | −20 | +85 | °C | ds p. 11 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | P1 | D1 (J2 pin 1) | Passive | SWCLK | ds p. 26 |
| 2 | P2 | D2 (J2 pin 2) | Passive | GND | ds p. 26 |
| 3 | P3 | D3 (J2 pin 3) | Passive | SWDIO | ds p. 26 |

## Typical application

- Wire P1 to SWCLK, P2 to GND and P3 to SWDIO (ds p. 26, Figure 19). `examples/rpi-pico2/src/main.cohdl` does this: `SWCLK_H` to `swd.P1`, GND to `swd.P2`, `SWDIO_H` to `swd.P3`.
- No external pull-ups are needed for the RP2040 SWD pins; they have internal ~60 kΩ pull-ups (ds p. 24).

## Layout notes

- **Pad order, checked on ds p. 8, Figure 4** (top view, USB at the top, pin 1 at the upper left). D1, D2 and D3 run left to right along the lower edge. `FP_Pico_Castellated_3` has pads 1, 2, 3 at x = −2.54, 0, +2.54. That is the same order when the board edge is towards +y in CoHDL's +y-down top-view frame. Three pads in a row have no handedness of their own, so the order is right as long as the footprint is not mirrored.
- Figure 3 (ds p. 7) draws the SWD pads as oblong pads with a hole and a half-circle at the board edge. The agent reads the 1.6 mm dimension as running from the lower edge to the SWD hole centres. The figure does not dimension the SWD pitch, hole diameter or pad size. From the drawing scale the pitch measures about 2.5 mm, which is a measurement, not a dimension.
- Figure 5 (ds p. 10) gives the carrier-board SMT land for mounting a Pico: three SWD lands, 6.68 mm across their outer edges, the middle one square. This is a land for a carrier under a Pico, not the Pico's own pads.

## Pitfalls

- The schematic's order is SWCLK, GND, SWDIO (ds p. 26). Do not use the order in `lib/connectors/docs/README.md`, which says the schematic maps D1/D2/D3 to "SWCLK/SWDIO/GND". The README is wrong; the example board follows the schematic.
- This is a board feature, not a purchasable part (ds p. 26). A BOM line for CON_PICO_3W has nothing to buy.

## Not established by the sources

- **Hole diameter and copper of the SWD pads.** The library uses Ø1.0 mm drill and Ø1.7 mm copper, and its comment says "the datasheet mechanical drawing specifies Ø1mm holes". The Ø1 callouts in Figure 3 point to the 40 edge pins (ds p. 7); the SWD pads are not dimensioned. The library's values come from Raspberry Pi's design-file ZIP, which is not in the repository.
- **SWD pad pitch.** 2.54 mm is not dimensioned in the datasheet. The library and README take it from the design files.
- **Castellation.** The library models a plain plated through-hole. Figure 3 shows an oblong pad with a hole and an edge half-circle (ds p. 7). The datasheet does not say whether the SWD pads are castellated through the edge.
- **README error.** `lib/connectors/docs/README.md` gives the D1/D2/D3 mapping as SWCLK/SWDIO/GND. The schematic (ds p. 26) has SWCLK/GND/SWDIO.
- Current or voltage ratings for the pads.
