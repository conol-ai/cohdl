# Würth 693070010811 — WR-CRD microSD card connector, push-pull, SMT, 8 contacts

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/sd-card::MICROSD_SOCKET",
  "parts": ["@contrib/sd-card::CONN_MICROSD"],
  "identities": [
    {"mfr": "Wurth Elektronik", "mpn": "693070010811", "lcsc": null}
  ],
  "sources": [
    {"id": "ds", "title": "ConCARD WR-CRD MicroSD Card Connector SMT, order code 693070010811", "revision": "Revision 001.000, 2025-03-05",
     "url": "https://www.we-online.com", "sha256": "e6226d72366353e84da353df89b196cb5b6e9bd164d19e0a56b4290694a3a27d", "local": "lib/@contrib/sd-card/docs/microsd-wurth.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "PAGE N/8". The recorded URL is only the Würth Elektronik home page, which the package README and the PDF title block give; no document link is recorded. `lcsc_map.json` has no row for this MPN.

## Summary

693070010811 is a surface-mount microSD (TransFlash) receptacle with 8 contacts and two shell solder tabs, rated 0.5 A per contact and 10,000 mating cycles (ds p. 1, p. 3). The source calls it "Push-Pull MicroSD" and "Type: Push & Pull" (ds p. 1, p. 3): the card is pushed in and pulled out by hand, with no push-push ejector. It has **no card-detect or write-protect switch**; the only card-detect means is CD/DAT3 on contact 2 (ds p. 3). Choose it for a compact, low-cost slot where the card is reachable by fingers.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Rated current IR | — | 0.5 | — | A | per contact (stated without qualifier) | ds p. 3 |
| Working voltage | — | 50 | — | V (DC) | | ds p. 3 |
| Withstanding voltage | — | 500 | — | V (AC) | | ds p. 3 |
| Contact resistance R | — | 40 | — | mΩ | footnote "1) Pin"; min/max not stated | ds p. 3 |
| Insulation resistance RISO | — | 1000 | — | MΩ | min/max not stated | ds p. 3 |
| Durability | — | 10,000 | — | mating cycles | | ds p. 3 |
| Contact pitch | — | 1.1 | — | mm | ±0.03 | ds p. 1 |
| Reel quantity | — | 1500 | — | pcs | 24 mm tape | ds p. 4 |

## Absolute maximum ratings

The source has no absolute-maximum table. The reflow limits below are from the classification profile (J-STD-020E).

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Peak package body temperature Tp, Pb-free, by package volume and thickness | 245–260 (Tc table) | °C | ds p. 6 |
| Time within 5 °C of peak | 20–30 | s | ds p. 6 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature | -25 | +85 | °C | ds p. 3 |
| Working voltage | — | 50 | V (DC) | ds p. 3 |
| Current per contact | — | 0.5 | A | ds p. 3 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | DAT2 | DAT2 | bidirectional | Data line 2 | ds p. 3 |
| 2 | DAT3 | CD/DAT3 | bidirectional | Card detection / data line 3 (SPI CS) | ds p. 3 |
| 3 | CMD | CMD | bidirectional | Command / answer | ds p. 3 |
| 4 | VDD | Vdd | power | Card power supply | ds p. 3 |
| 5 | CLK | CLK | input (card) | Clock | ds p. 3 |
| 6 | GND | Vss 2 | ground | Ground | ds p. 3 |
| 7 | DAT0 | DAT0 | bidirectional | Data line 0 | ds p. 3 |
| 8 | DAT1 | DAT1 | bidirectional | Data line 1 | ds p. 3 |
| G1 | SHIELD1 | (shell land, unnumbered) | mechanical | Shell solder tab; electrical role not stated | ds p. 1 |
| G2 | SHIELD2 | (shell land, unnumbered) | mechanical | Shell solder tab; electrical role not stated | ds p. 1 |

Recommended land (ds p. 1): the eight signal lands are numbered 8, 7, …, 1 from left to right, with the two shell lands on the far side of the body (above the signal row in the drawing). The library footprint `FP_MicroSD_Wurth_693070010811` has the same arrangement in its +y-down frame: pad 8 at x = −3.85 mm, pad 1 at x = +3.85 mm, shell lands at y = −9.25 mm.

## Typical application

- The source gives no application circuit. It names the contacts only (ds p. 3); pull-ups, series resistors and supply switching are set by the SD specification and the host, not by this source.
- Card detect: the only card-detect function in the source is on CD/DAT3 (contact 2, ds p. 3). There is no mechanical detect or write-protect switch to wire.

## Layout notes

Recommended land pattern (ds p. 1, general tolerance ±0.05 mm):
- Signal lands 0.8 × 1.5 mm on a 1.1 mm pitch; 7.7 mm between the centres of lands 8 and 1.
- Shell lands 3.1 × 1.5 mm, 6.06 mm apart centre to centre, symmetric about the signal row's centre.
- 9.25 mm from the signal-row centre line to the shell-land centre line.
- The shell lands reach 0.35 mm beyond the body outline.

Library `FP_MicroSD_Wurth_693070010811` matches every one of those values: signal lands at x = ±0.55, ±1.65, ±2.75, ±3.85 mm, y = 0; shell lands 3.1 × 1.5 mm at (±3.03, −9.25) mm.

Orientation check: the land drawing is a top view. Its body outline is narrower at the shell end and steps out (chamfer) on the pin-1 side, which matches the 3D view and the lower outline drawing on the same page (ds p. 1). The upper outline drawing on ds p. 1 is the mirror image of the land drawing (contacts 8…1 still read left to right but the shell tabs are at the other end), so it appears to be a view from below (the source labels none of the views); do not lay out from it.

Package (ds p. 1): 11.9 ± 0.1 mm long overall (upper drawing), 10.86 mm wide at the shell end and 12.16 mm wide overall (lower drawing); contact solder tails 0.35 mm wide; height dimensions 1.8, 1.95 and 2 mm are drawn on the side views without saying which surface each refers to.

## Pitfalls

- It is a **push-pull** socket, not push-push (ds p. 1, p. 3): leave finger room at the card slot.
- Do not expect a card-detect switch: pull CD/DAT3 per the SD specification if you need detection (ds p. 3).
- Washing agents and brushes may damage the contacts (ds p. 7).
- Würth recommends not storing the parts longer than 12 months before use (ds p. 7).

## Not established by the sources

- **Push-push vs push-pull:** the library header comment says "push-push card connector". The source says "Push-Pull MicroSD" (ds p. 1) and "Type: Push & Pull" (ds p. 3).
- **Shell lands as ground:** the library declares `SHIELD1` and `SHIELD2` as required pins and comments "Shell / mechanical solder tabs (GND, not signal)". The source lists only contacts 1–8 (ds p. 3) and draws the shell lands without a function (ds p. 1). It does not say the shell is grounded or electrically connected.
- **Courtyard:** the library courtyard is 12.7 × 12.5 mm centred at (0, −4.625) mm, so it spans x = ±6.35 mm and y = −10.875 to +1.625 mm. Scaled from the land drawing (not dimensioned), the body outline reaches about +6.7 mm on the pin-1 side and about +2.2 mm below the signal-row centre line, so the body appears to overhang the courtyard by roughly 0.4 mm and 0.6 mm there. The source does not dimension the body position relative to the lands.
- **Library comment "Pinout per MicroSD spec"**: the pin names agree with the source (ds p. 3); the SD specification itself is not a source here.
- **Electrical maximums:** the 40 mΩ contact resistance and 1000 MΩ insulation resistance carry no min/max (ds p. 3).
- **Card insertion force, retention and the meaning of "Polarity 8"** (ds p. 1) are not explained.
