# FH12-10S-0.5SH(55) — Hirose FH12 10-contact 0.5 mm FPC/FFC connector, bottom contact, SMT horizontal

## Identity

```json
{
  "schema": "part-note/1",
  "device": "connectors::DEBUG_CONN_10P",
  "parts": ["connectors::CONN_DEBUG_10P"],
  "identities": [
    {"mfr": "Hirose Electric", "mpn": "FH12-10S-0.5SH(55)", "lcsc": "C506791"}
  ],
  "sources": [
    {"id": "ds", "title": "Hirose FH12 Series, 0.5 mm and 1 mm pitch FPC/FFC connectors (catalogue)", "revision": "Nov. 1, 2016 (page 1 also shows 2013.11), 20 pages",
     "url": "https://www.hirose.com/product/p/CL0586-0576-2-55", "sha256": "9f056f58ff000739633cc3cb7697ba185967083dcf1429b7ab10783b3f55bac5", "local": "lib/connectors/docs/fh12-series.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are PDF pages; in this catalogue they equal the printed page numbers. **The URL is flagged.** The repository records no official URL for this PDF or for this part. The URL above is the Hirose product page for the 26-contact sibling, from `hirose-fh12-26s-0.5sh.evidence.md`. It stands in only so that the source has an https link.

## Summary

FH12-10S-0.5SH(55) is a 10-contact, 0.5 mm pitch, bottom-contact FPC/FFC connector. It mounts SMT horizontally, has a rotating one-touch ZIF actuator, and in the (55) version is gold plated (ds pp. 1–4). The FPC goes in with its exposed conductors **facing down** (ds p. 17). The connector is 2 mm high (ds p. 4, note 5). It is rated 0.5 A DC per contact, 70 % of that when every contact carries current, and 50 V AC (ds p. 2). It takes a 0.3 ± 0.05 mm gold-plated FPC/FFC end (ds p. 2). Do not use it with a top-contact flex, or for supply currents above the per-contact rating. The library uses it as a 10-way debug port (SWD, UART, boot, reset and power). The pin functions come from the library, not from Hirose.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Current rating | — | — | 0.5 | A DC | Per contact; use 70 % when all contacts carry current (note 1) | ds p. 2 |
| Voltage rating | — | — | 50 | V AC | — | ds p. 2 |
| Insulation resistance | 500 | — | — | MΩ | 100 V DC | ds p. 2 |
| Withstanding voltage | 150 | — | — | V AC | 1 minute, no flashover or breakdown | ds p. 2 |
| Contact resistance | — | — | 50 | mΩ | 1 mA | ds p. 2 |
| Durability | 20 | — | — | cycles | Insertion/withdrawal, contact resistance ≤ 50 mΩ after | ds p. 2 |
| Pitch | — | 0.5 | — | mm | — | ds p. 4 |
| Dimension A (10 contacts) | — | 4.5 | — | mm | Contact 1 to contact 10 centres | ds p. 4 |
| Dimension B / C / D (10 contacts) | — | 8.1 / 9.1 / 5.57 | — | mm | B body width, C overall incl. fittings (top view); D front-view span | ds p. 4 |
| Height | — | 2 | — | mm | Standard actuator | ds p. 4 |
| Depth | — | 4.6 / 5.6 | — | mm | Body / with actuator, side view | ds p. 4 |
| FPC/FFC thickness | 0.25 | 0.3 | 0.35 | mm | Gold-plated contact end | ds p. 2, p. 11 |
| FPC/FFC width B (10 contacts) | — | 5.5 | — | mm | ±0.07 | ds p. 11 |

Materials (ds p. 2): insulator polyamide (beige) or PPS (dark brown), UL94V-0; contacts phosphor bronze, gold plated; metal fittings brass, tin plated. Plating code (55) = gold plated (ds p. 2).

## Absolute maximum ratings

The catalogue has no absolute-maximum table. The rows below are test and storage conditions, not damage thresholds.

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Resistance to soldering heat, manual soldering (test condition) | 350 ± 10 °C for 5 s | — | ds p. 2 |
| Reflow peak in Hirose's test profile | 250 | °C | ds p. 16 |
| Storage temperature, long-term storage before mounting (note 3) | −10 to +50 | °C | ds p. 2 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating temperature (includes temperature rise from current) | −40 | +85 | °C | ds p. 2 |
| Operating humidity (no condensation) | — | 90 | % RH | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | SWDIO | contact 1 | Bidirectional | Library: SWD data / JTAG TMS | ds p. 4 |
| 2 | SWCLK | contact 2 | Bidirectional | Library: SWD clock / JTAG TCK | ds p. 4 |
| 3 | UART_TX | contact 3 | Bidirectional | Library: SF32 DBG_UART TX | ds p. 4 |
| 4 | UART_RX | contact 4 | Bidirectional | Library: SF32 DBG_UART RX | ds p. 4 |
| 5 | BOOT | contact 5 | Input | Library: ESP32-S3 GPIO0 strap | ds p. 4 |
| 6 | NRST | contact 6 | Bidirectional | Library: ESP32-S3 EN | ds p. 4 |
| 7 | VCC | contact 7 | Power out | Library: 3.3 V reference for the probe | ds p. 4 |
| 8 | GND | contact 8 | Power in | Library: ground | ds p. 4 |
| 9 | GND2 | contact 9 | Power in | Library: ground | ds p. 4 |
| 10 | GND3 | contact 10 | Power in | Library: ground | ds p. 4 |
| MP | MP | metal fittings (hold-downs) | Passive | Two tin-plated brass hold-down lands; mechanical | ds p. 2, p. 11 |

## Typical application

- Insert the FPC/FFC fully and parallel to the board, conductors **facing down**, then rotate the actuator closed (ds p. 17). Do not load the FPC while locking (ds p. 18).
- Flex end (ds p. 11):
  - conductors 0.35 ± 0.03 mm wide (FPC) or 0.3 ± 0.03 mm (FFC), on 0.5 ± 0.05 mm pitch;
  - A = 4.5 mm between the outer conductors, B = 5.5 ± 0.07 mm overall width;
  - exposed length 3.5 mm min, stiffener region 4.5 mm min;
  - stiffener thickness 0.188 mm min, so that C = 0.3 ± 0.05 mm.
- The FH12 bottom-contact and top-contact (FH12A) versions take the flex with its conductive surface on opposite sides (ds p. 3, series configuration; pp. 17, 19). If a flex designed for one is used in the other, its conductors either face away from the contacts or, once turned over, land in reverse order. Which flex conductor meets contact 1 is the agent's inference from this; the catalogue does not draw it.

## Layout notes

- Recommended land (ds p. 11, "0.5mm Pitch" land figure; the same table serves bottom and top contact):
  - contact lands 0.3 ± 0.03 × 1.3 ± 0.1 mm on 0.5 ± 0.05 mm pitch, A = 4.5 ± 0.05 mm centre to centre;
  - hold-down lands 2.2 ± 0.1 mm high, B = 10.1 ± 0.1 mm outer and C = 6.5 ± 0.1 mm inner span;
  - 1.5 ± 0.1 mm between the hold-down and contact land edges;
  - metal mask 0.25 mm wide, 0.15 mm thick.
- **Library land check.** `FP_FPC10_0P5` gives contacts 0.3 × 1.3 mm at x = 0.5 · (n − 5.5), y = −1.85. Hold-downs are 1.8 × 2.2 mm at x = ±4.15, y = +1.40. From p. 11: hold-down width (B − C) / 2 = 1.8, centre ±(B + C) / 4 = ±4.15, row separation 0.65 + 1.5 + 1.1 = 3.25 = 1.40 + 1.85. All match.
- **Contact-1 end, checked on ds p. 4, top view of the 0.5 mm bottom-contact drawing.** The tails exit at the top of the view and the actuator is at the bottom. A ▽ "polarizing mark indicator" is at the left front corner, and a "Number of Contacts indicator" is at the right. The library puts the contact row at −y (top of the view in CoHDL's +y-down frame) and pad 1 at −x, the ▽ end. That is the same view with no rotation or reflection, **if** the ▽ marks contact 1. The catalogue does not say that it does; see below.
- Recommended reflow profile: peak 250 °C (ds p. 16).

## Pitfalls

- The device fixes a board-specific debug pinout: contact 1 SWDIO, 2 SWCLK, 3/4 UART, 5 BOOT, 6 NRST, 7 VCC, 8–10 GND (`lib/connectors/src/fpc/fpc.cohdl`). Hirose assigns no functions (ds p. 4). The probe adapter's flex must match it, including the conductors-down orientation, or contact 1 lands on contact 10.
- VCC is one contact rated 0.5 A, derated to 70 % with all contacts loaded (ds p. 2). It is a reference for the probe, not a supply path.
- Bottom contact: conductors face down (ds p. 17). The top-contact FH12A version needs them up (ds p. 19). Getting this wrong mirrors every signal.
- Derate to 70 % of 0.5 A when every contact carries current (ds p. 2, note 1). Use several contacts for a supply.
- Durability is only 20 insertion cycles (ds p. 2).
- The hold-downs are tin-plated brass fittings (ds p. 2). The catalogue gives them no electrical function; the library's `MP` pin is optional and passive.

## Not established by the sources

- **Which end is contact 1.** The catalogue's 0.5 mm bottom-contact drawing marks a ▽ "polarizing mark indicator" at one end (ds p. 4) but never says that it is contact 1. The only "No.1 contact" labels in the catalogue are on the vertical-mount land figures (ds p. 11). The library's pad 1 at the ▽ end agrees with the drawing's view, but the source does not confirm it. Check it against Hirose's per-part drawing before release.
- Whether the hold-downs may be tied to ground or another net.
- Per-contact derating with ambient temperature.
- An official URL for this PDF; see the note under Identity.
