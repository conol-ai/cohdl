# TYPE-C-31-M-12 — HRO USB Type-C 2.0 receptacle, 16 contacts, SMT with plated shell pegs

## Identity

```json
{
  "schema": "part-note/1",
  "device": "usb::USB_C_Receptacle_2_0",
  "parts": ["usb::USB_C_HRO_TYPE_C_31_M_12"],
  "identities": [
    {"mfr": "HRO", "mpn": "TYPE-C-31-M-12", "lcsc": "C165948"}
  ],
  "sources": [
    {"id": "dwg", "title": "HRO TYPE-C-31-M-12 / -12A / -12B / -12C family drawing (exact part on PDF page 4)", "revision": "Unversioned family PDF; page 4 sheet Rev. A, drawn 2020-12-08",
     "url": "https://omo-oss-file110.thefastfile.com/portal-saas/new2023011311465136174/cms/file/type-c-31-m-12&12a&12b&12c.pdf", "sha256": "c5a4131fb0432bfc24dae4597173d9a3d242c8c778506e2b3032181c75aaedd4", "local": "lib/usb/docs/hro-type-c-31-m-12-family-drawing.pdf"}
  ],
  "status": "draft"
}
```

## Summary

Horizontal SMT USB Type-C receptacle carrying only the USB 2.0 contact set: 16 SMT
contacts in one row on the PCB (the four GND and four VBUS contacts share four 0.60 mm lands), four
plated shell slots and two Ø0.50 mm plastic locating bosses (dwg p. 4). The drawing rates it 5 A /
20 V with 10,000 mating cycles and −30 °C to +85 °C operation (dwg p. 4). Choose it for a USB 2.0
device or sink port; it has no SuperSpeed contacts, so do not use it for USB 3.x or alternate modes
that need the TX/RX pairs. Only PDF page 4 describes this exact MPN; pages 1–3 are the -12A/-12B/-12C
variants and were not used for any fact here.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Current rating | — | — | 5 | A | As stated; whether per contact or per port is not stated | dwg p. 4 |
| Voltage rating | — | — | 20 | V | As stated | dwg p. 4 |
| Insulation resistance | 100 | — | — | MΩ | Test voltage not stated | dwg p. 4 |
| Dielectric withstanding voltage | 100 | — | — | V AC | For 1 minute | dwg p. 4 |
| Durability | 10000 | — | — | cycles | Mating cycles | dwg p. 4 |
| Insertion force | 5 | — | 20 | N | — | dwg p. 4 |
| Removal force | 8 | — | 20 | N | — | dwg p. 4 |
| Height above PCB | — | 3.26 | — | mm | Shell top to seating plane | dwg p. 4 |
| Mating centreline height | — | 1.68 | — | mm | CL above seating plane, section A-A | dwg p. 4 |
| Body width / length | — | 8.94 / 7.35 | — | mm | Top view | dwg p. 4 |
| Shell peg length below seating plane | — | 1.01 | — | mm | 4 pegs | dwg p. 4 |

Materials (dwg p. 4): housing high-temperature plastic UL94 V-0; contacts copper alloy; mid plate and
shell stainless steel; contacts Ni 50 µ" min under-plate, Au on the contact area ("gold plating
thickness follow the P/N"), gold flash on the solder area; front shell Ni 30 µ" min.

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| None stated; the drawing gives only the ratings under Key parameters | — | — | dwg p. 4 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature | −30 | +85 | °C | dwg p. 4 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| A1 | GND | GND | Ground | Ground; shares a land with B12 | dwg p. 4 |
| A12 | GND | GND | Ground | Ground; shares a land with B1 | dwg p. 4 |
| B1 | GND | GND | Ground | Ground; shares a land with A12 | dwg p. 4 |
| B12 | GND | GND | Ground | Ground; shares a land with A1 | dwg p. 4 |
| A4 | VBUS | VBUS | Power | Bus power; shares a land with B9 | dwg p. 4 |
| A9 | VBUS | VBUS | Power | Bus power; shares a land with B4 | dwg p. 4 |
| B4 | VBUS | VBUS | Power | Bus power; shares a land with A9 | dwg p. 4 |
| B9 | VBUS | VBUS | Power | Bus power; shares a land with A4 | dwg p. 4 |
| A5 | CC1 | CC1 | Passive | Configuration channel 1 | dwg p. 4 |
| B5 | CC2 | CC2 | Passive | Configuration channel 2 | dwg p. 4 |
| A6 | DP | DP1 | Bidirectional | USB 2.0 D+, row A | dwg p. 4 |
| B6 | DP | DP2 | Bidirectional | USB 2.0 D+, row B | dwg p. 4 |
| A7 | DN | DN1 | Bidirectional | USB 2.0 D−, row A | dwg p. 4 |
| B7 | DN | DN2 | Bidirectional | USB 2.0 D−, row B | dwg p. 4 |
| A8 | SBU1 | SBU1 | Passive | Sideband use 1 (optional in the library) | dwg p. 4 |
| B8 | SBU2 | SBU2 | Passive | Sideband use 2 (optional in the library) | dwg p. 4 |
| SH1 | SHIELD | — (shell) | Passive | Rear-left plated shell slot | dwg p. 4 |
| SH2 | SHIELD | — (shell) | Passive | Rear-right plated shell slot | dwg p. 4 |
| SH3 | SHIELD | — (shell) | Passive | Front-left plated shell slot | dwg p. 4 |
| SH4 | SHIELD | — (shell) | Passive | Front-right plated shell slot | dwg p. 4 |

The drawing's pin table names the 16 contacts only; the shell slots carry no designator there, and
SH1..SH4 are the library's names (dwg p. 4; `lib/usb/src/connectors/type_c.cohdl`).

## Typical application

- The drawing contains no application circuit. It does not state the CC1/CC2 terminations; the
  5.1 kΩ Rd pull-downs a USB 2.0 sink or device needs on CC1 and CC2 come from the USB Type-C
  specification, which is outside this note's sources. Use a separate resistor per CC pin.
- Tie DP (A6+B6) and DN (A7+B7) together so either plug orientation works; the library device already
  merges them into one pin each (dwg p. 4 pin table: DP1/DP2, DN1/DN2).
- SBU1/SBU2 may be left unconnected for plain USB 2.0 (the library declares them optional).
- Connect SHIELD per the board's shield strategy; the drawing gives no guidance.

## Layout notes

- Recommended PCB layout (component side), tolerance ±0.05 mm (dwg p. 4): eight 0.30 mm and four
  0.60 mm wide contact lands in one row; land pitch 0.50 mm; centre spans 6.40 (A1/B12 to B1/A12),
  4.80 (A4/B9 to B4/A9), 3.50 (B8 to B5), 2.50 (A5 to A8), 1.50 (B7 to B6) mm.
- Land length: 1.64 mm is dimensioned from the land's outer edge to the locating-hole centreline and
  0.50 mm from the land's inner edge to the same line, so the drawn land is 1.14 mm long (dwg p. 4).
- Locating holes Ø0.60 on 5.78 mm centres for the Ø0.50 bosses (dwg p. 4).
- Shell slots on 8.65 mm centres: rear pair 0.60 × 1.70 hole in 0.90 × 2.00 copper; front pair
  0.60 × 1.40 hole in 0.90 × 1.70 copper; rear-to-front slot centres 4.18 mm (dwg p. 4).
- PCB edge is 5.79 mm in front of the locating-hole centreline (dwg p. 4); in the library footprint's
  frame (holes at y = −2.6 mm) that is y ≈ +3.19 mm.
- Assembly origin: LCSC C165948's placement model origin sits about 1.4–1.55 mm from the library
  footprint's along the mating axis. `tools/sourcing_data/jlc_frames.json` row `usbc-origin-c165948`
  maps it `at (0mm, -1.55mm)` (measured −1.42 mm); with that map the assembler preview put all four
  copies' pegs within 0.2 mm of the holes
  (`tools/sourcing_data/jlc_previews/2026-10-30-usbc-origin/README.md`). Export CPL with this map.

## Pitfalls

- The library's contact lands (`P_USBC_Sig_L`/`P_USBC_Sig_S`) are 1.64 mm long, centred 1.445 mm
  behind the locating-hole line (spanning 0.625–2.265 mm); the drawing's lands are 1.14 mm long,
  spanning 0.50–1.64 mm behind that line (dwg p. 4). The library comment reads 1.64 as the land
  length. Review before release.
- Library slot centres sit 0.53 mm behind / 3.65 mm in front of the hole line; the drawing implies
  0.50 / 3.68 mm (rear slot centres on the land inner-edge line, dwg p. 4), and 8.64 vs 8.65 mm
  across. Each difference is ≤ 0.03 mm, inside the ±0.05 mm layout tolerance.
- The 5 A rating is not stated per contact (dwg p. 4); route all four VBUS and four GND contacts.
- Without the supplier origin map above in the CPL export, the assembler places the part ~1.5 mm
  off along the mating axis (preview README cited under Layout notes).
- The page-4 title block reads "DETECTOR SWITCHS"; the part number field (TYPE-C-31-M-12) is the
  identity to trust (dwg p. 4). Do not take facts from pages 1–3 (other variants).

## Not established by the sources

- Contact resistance, per-contact current, insulation-resistance test voltage, absolute maximums.
- Storage temperature, reflow profile, moisture sensitivity, paste and solder-mask openings.
- Gold plating thickness for this exact P/N ("follow the P/N" is not decoded on page 4).
- CC termination values (USB Type-C specification, not this drawing); shield grounding.
