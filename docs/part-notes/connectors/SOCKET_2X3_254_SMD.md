# SSW-103-22-SM-D-VS — Samtec 2 × 3 vertical SMT socket strip, 2.54 mm pitch

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::Socket_2x3",
  "parts": ["connectors::SOCKET_2X3_254_SMD"],
  "identities": [
    {"mfr": "Samtec", "mpn": "SSW-103-22-SM-D-VS", "lcsc": "C7370854"}
  ],
  "sources": [
    {"id": "asm", "title": "Samtec SSW-1XX-22-XXX-X-VS-XX-X-X-XX, .100 [2.54] surface mount assembly (marketing print)", "revision": "Revision BC, 4 sheets",
     "url": "https://suddendocs.samtec.com/prints/ssw-1xx-22-xxx-x-vs-xx-x-x-xx-mkt.pdf", "sha256": "b058e14102bd07ea190bdc2904126003354b68f092e492c4fe77b4545dc83061", "local": "lib/connectors/docs/samtec-ssw-1xx-22-surface-mount-drawing.pdf"},
    {"id": "fp", "title": "Samtec recommended P.C.B. layout for SSW-DVS", "revision": "Rev. A, one sheet (PDF created 2000-06-21)",
     "url": "https://suddendocs.samtec.com/prints/ssw-dvs.pdf", "sha256": "a8e837ed7fac38b784c95c2bcada75ed5d751a339400c7c85f204d258769d58c", "local": "lib/connectors/docs/samtec-ssw-double-row-smt-footprint.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages, which equal the drawing sheet numbers (`asm` sheet 1 of 4 is PDF page 1). The footprint drawing `fp` has no text layer; the agent read it from a rendered image. The URLs are the ones in `lib/connectors/docs/README.md` and were not fetched again.

## Summary

SSW-103-22-SM-D-VS is a Samtec SSW socket strip in the 0.100 in (2.54 mm) family (asm p. 1). Its part number decodes as:
- `103`: 3 positions per row;
- `-22`: lead style;
- `-SM`: selective plating, 30 µ" Au over 50 µ" Ni on the contacts, matte tin on the tails;
- `-D`: double row;
- `-VS`: vertical surface mount.

Its six SMT tails solder to six 1.27 mm wide lands in two rows (fp p. 1). The insulator is LCP, UL 94 V-0, black, and the terminals are phosphor bronze (asm p. 1). **Neither print gives a current or voltage rating.** The library binds it as a generic 2 × 3 female header with odd/even numbering; the source confirms that numbering.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Column pitch | — | 2.54 | — | mm | .100 in | fp p. 1 |
| Body length | — | 8.13 | — | mm | (positions × .10) + .02 in = .32 in for 3 positions; the print's own mm rounding (× 2.5 + 0.5) gives 8.0 | asm p. 1 |
| Body width, double row | — | 4.95 | — | mm | REF | asm p. 1 |
| Body height above PCB, double row | — | 8.51 | — | mm | REF | asm p. 1 |
| Tail span across both rows | — | 7.87 | — | mm | .31 in, after shearing (note 7) | asm p. 1 |
| Tail section | — | 0.79 × 0.41 | — | mm | REF, TYP | asm p. 1 |
| Coplanarity | — | — | 0.10 | mm | .004 in, control-plan critical | asm p. 1 |
| Minimum contact push-out force | 1.5 | — | — | lbf | Note 10 | asm p. 1 |
| Contact plating (-SM) | 30 | — | — | µ" Au | Over 50 µ" Ni min, on the contact; matte tin tail | asm p. 1 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| None stated on either print | — | — | asm p. 1 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| None stated on either print | — | — | — | asm p. 1 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | P1 | 01 | Passive | Odd row, end position | asm p. 1 |
| 2 | P2 | 02 | Passive | Even row, opposite 01 | asm p. 1 |
| 3 | P3 | 03 | Passive | Odd row, next to 01 | asm p. 1 |
| 4 | P4 | 04 | Passive | Even row, opposite 03 | asm p. 1 |
| 5 | P5 | 05 | Passive | Odd row, far end | asm p. 1 |
| 6 | P6 | 06 | Passive | Even row, opposite 05 | asm p. 1 |

## Typical application

- A board-to-board or module socket. The board assigns the six contacts. The prints name no mating header.
- `-SM` plating keeps gold on the contact area and tin on the tails for reflow (asm p. 1).

## Layout notes

- Recommended layout (fp p. 1):
  - lands 1.27 mm (.050) wide, on a 2.54 mm (.100) column pitch;
  - outer columns (positions − 1) × 2.54 = 5.08 mm apart;
  - rows 8.64 mm (.340) overall, with a 1.78 mm (.070) gap between the rows' inner edges;
  - so each land is (8.64 − 1.78) / 2 = 3.43 mm long, and the row centres are ±2.605 mm from the centreline.
- **Library land check.** `FP_Socket_2x3_254_SMD` uses 1.27 × 3.43 mm lands at x = −2.54, 0, +2.54 and y = ±2.605. This matches fp p. 1 exactly. The 7.87 mm tail span (asm p. 1) ends 0.385 mm inside the 8.64 mm land envelope on each side.
- **Numbering and view, checked on asm p. 1, Fig. 1, "-D: DOUBLE ROW" top view.** The view looks into the contact openings. 01 is at the lower right and 02 directly above it; 03 is left of 01 and 04 left of 02. Odd positions are in one row and even in the other, and numbers increase right to left. `FP_Socket_2x3_254_SMD` has pad 1 at (−2.54, −2.605), with 3 and 5 along +x in the same row and 2/4/6 opposite. That is the Fig. 1 top view rotated 180° in CoHDL's +y-down top-view frame, with no reflection. The library comment says the same.
- The layout print `fp` itself does not number the lands (fp p. 1). Pin numbering comes only from `asm` Fig. 1.
- Courtyard 8.7 × 9.2 mm. It contains the 8.13 × 4.95 mm body and the 8.64 mm land span (asm p. 1; fp p. 1).

## Pitfalls

- The numbering is odd/even across the rows (01/02 opposite each other), not row-major like the Molex Micro-Fit headers in this package (asm p. 1).
- The 2.54 mm in the footprint print is the column pitch, not the land length; the land is 3.43 mm long (fp p. 1). The library comment records an earlier misreading of this.
- Pick-and-place pads (-P) and polarization (-K, -N) are options that this MPN does not include (asm p. 1). Plan for a vacuum nozzle that fits the bare socket, or a manual place.

## Not established by the sources

- Current rating, voltage rating, operating temperature, mating cycles and mating-post size. Neither print gives them.
- Solder-paste and mask openings, and a reflow profile.
- A pin-1 marking on the body. Note 11 ties the "polarized corner" to the PPP-12 pick-and-place pad, which this MPN does not have (asm p. 1).
- Whether `fp` Rev. A (2000) is still Samtec's current recommended layout for the Rev BC assembly. The prints do not cross-reference each other.
