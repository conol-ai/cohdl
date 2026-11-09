# USBLC6-2SC6 — two-line USB 2.0 ESD protection with VBUS clamp, SOT23-6L

## Identity

```json
{
  "schema": "part-note/1",
  "device": "esd::USBLC6",
  "parts": ["esd::ESD_USBLC6"],
  "identities": [
    {"mfr": "STMicroelectronics", "mpn": "USBLC6-2SC6", "lcsc": "C7519"}
  ],
  "sources": [
    {"id": "ds", "title": "USBLC6-2 Very low capacitance ESD protection", "revision": "DS4260 Rev 7, December 2021",
     "url": "https://www.st.com/resource/en/datasheet/usblc6-2.pdf", "sha256": "bc30154f310cd631043214ed52571daefe04270587ec55072d49a23bd18b9068", "local": "lib/esd/docs/st-usblc6-2-datasheet.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are physical PDF pages; they equal the printed "page N/20".

## Summary

USBLC6-2SC6 is a monolithic rail-to-rail ESD protector for two data lines plus VBUS in SOT23-6L. Steering diodes go from each I/O to GND and to VBUS, and a transil clamps VBUS to GND (ds p. 1, p. 4). It is meant for USB 2.0 up to 480 Mb/s, USB 1.1, 10/100 Ethernet, SIM and video lines (ds p. 1). Two properties decide the choice: low line capacitance (3.5 pF max I/O to GND, with 0.015 pF typical matching) and 150 nA max leakage (ds p. 2). It is not a surge (lightning) or overvoltage protector. Its datasheet gives no continuous working-voltage rating, and the clamp breakdown is only 6 V min (ds p. 2), so do not use it on lines or rails above 5 V.

## Key parameters

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Leakage current IRM | — | 10 | 150 | nA | VRM = 5.25 V, Tamb = 25 °C (pins not stated) | ds p. 2 |
| Breakdown voltage VBR, VBUS to GND | 6 | — | — | V | IR = 1 mA, Tamb = 25 °C | ds p. 2 |
| Forward voltage VF | — | — | 1.1 | V | IF = 10 mA | ds p. 2 |
| Clamping voltage VCL, any I/O to GND | — | — | 12 | V | IPP = 1 A, 8/20 µs | ds p. 2 |
| Clamping voltage VCL, any I/O to GND | — | — | 17 | V | IPP = 5 A, 8/20 µs | ds p. 2 |
| Capacitance I/O to GND, Ci/o-GND | — | 2.5 | 3.5 | pF | VR = 1.65 V | ds p. 2 |
| Capacitance matching I/O to GND, ΔCi/o-GND | — | 0.015 | — | pF | — | ds p. 2 |
| Capacitance I/O to I/O, Ci/o-i/o | — | 1.2 | 1.7 | pF | VR = 1.65 V | ds p. 2 |
| Capacitance matching I/O to I/O, ΔCi/o-i/o | — | 0.04 | — | pF | — | ds p. 2 |
| Analog crosstalk | — | < -55 | — | dB | Up to 240 MHz, Figure 11 test board (typical measurement) | ds p. 7 |
| IEC 61000-4-2 level 4, contact discharge (Features list) | — | — | 8 | kV | Disagrees with Table 1, see "Not established" | ds p. 1 |

All Table 2 values are at Tamb = 25 °C (ds p. 2). Leakage rises with junction temperature; Figure 3 gives only a typical relative curve at VBUS = 5 V (ds p. 3).

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Peak pulse voltage VPP, IEC 61000-4-2 air discharge | 15 | kV | ds p. 2 |
| Peak pulse voltage VPP, IEC 61000-4-2 contact discharge | 15 | kV | ds p. 2 |
| Peak pulse voltage VPP, MIL STD883G method 3015-7 | 25 | kV | ds p. 2 |
| Storage temperature Tstg | -55 to +150 | °C | ds p. 2 |
| Lead temperature for soldering, 10 s at 5 mm, TL | 260 | °C | ds p. 2 |

## Recommended operating conditions

The datasheet has no recommended-operating-conditions table. The operating junction range below is printed in Table 1, "Absolute ratings".

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Operating junction temperature Tj | -40 | +150 | °C | ds p. 2 |

## Pins

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | IO1_A | I/O1 | bidirectional | Data line 1, one end (D+in in Figure 17); same I/O1 node as pad 6 | ds p. 1, p. 11 |
| 2 | GND | GND | ground | Ground; anode side of the steering diodes and of the VBUS transil | ds p. 1 |
| 3 | IO2_A | I/O2 | bidirectional | Data line 2, one end (D-in in Figure 17); same I/O2 node as pad 4 | ds p. 1, p. 11 |
| 4 | IO2_B | I/O2 | bidirectional | Data line 2, other end (D-out in Figure 17) | ds p. 1, p. 11 |
| 5 | VBUS | VBUS | power (clamp rail) | Top rail of the steering diodes and the transil; connect to the bus supply with CBUS = 100 nF | ds p. 1, p. 11 |
| 6 | IO1_B | I/O1 | bidirectional | Data line 1, other end (D+out in Figure 17) | ds p. 1, p. 11 |

Pin numbering is the top view of the functional diagram: 1–3 down one side, 4–6 back up the other, counter-clockwise (ds p. 1).

## Typical application

- USB 2.0 port: D+ through pads 1/6, D- through pads 3/4, VBUS pad 5 to the connector VBUS, GND to the connector ground (Figure 13, ds p. 9; Figure 17, ds p. 11).
- Put CBUS = 100 nF from VBUS (pad 5) to GND (Figure 17, ds p. 11). The T1/E1/Ethernet example also ties VBUS to +VCC with 100 nF (Figure 14, ds p. 10).
- Each data line is meant to pass through the package, in at pad 1 (or 3) and out at pad 6 (or 4), which is the flow-through layout of Figure 17 (ds p. 11). The functional diagram draws pads 1 and 6 as one node and pads 3 and 4 as one node (ds p. 1).
- No other external parts are asked for.

## Layout notes

- Place the device as close as possible to the disturbance source, which is generally the connector (ds p. 6).
- Keep the I/O, VBUS-to-VCC and GND-to-plane tracks as short as possible (ds p. 6). In ST's example, 10 mm x 0.5 mm tracks (about 6 nH each) add 144 V per inductance during a 24 A, 1 ns IEC edge. They raise the clamp from +31.2 V to +319.2 V (ds p. 4).
- Recommended SOT23-6L land (Figure 19): 0.60 x 1.20 mm pads, 0.95 mm pitch, 2.30 mm between row centres, 3.50 mm overall, 1.10 mm gap between rows (ds p. 13). Library `FP_SOT_23_6` uses 1.20 x 0.60 mm pads at x = ±1.15 mm, y = 0, ±0.95 mm. That is Figure 19 rotated 90°, so it matches.
- Package body (Table 3): D 2.8–3.05 mm, E 1.5–1.75 mm, H 2.6–3.0 mm, A 0.9–1.45 mm, e 0.95 mm (ds p. 12).

## Pitfalls

- Do not tie VBUS (pad 5) to a rail above about 5 V. VBR VBUS-to-GND is only 6 V min, and leakage is specified only at 5.25 V (ds p. 2).
- The I/O clamp is referenced to VBUS. A positive strike clamps at VTRANSIL + VF, so the VBUS-to-GND path, CBUS and the GND return carry the ESD current (ds p. 4, p. 5). A long GND or VBUS trace undoes the protection (ds p. 4).
- Contact-discharge level: the Features list says 8 kV (ds p. 1), but Table 1 says 15 kV (ds p. 2). Claim 8 kV contact (IEC level 4) unless ST confirms otherwise.
- The marking code UL26 may be rotated 90° to show the assembly site (ds p. 18). Do not find pin 1 from the marking text. Use the pin-1 dot (Figure 22, ds p. 16).
- The PSpice model in Figures 15–16 is valid only at 27 °C ambient (ds p. 10). Do not use it for over-temperature leakage or capacitance.
- USBLC6-2P6 is the SOT-666 version, marking F (ds p. 18). It is not this part and has no library footprint.

## Not established by the sources

- **Contact-discharge rating conflict:** Features say IEC 61000-4-2 contact discharge 8 kV (ds p. 1). Table 1 gives 15 kV contact (ds p. 2). The source does not say which is right.
- **Maximum working (stand-off) voltage:** no VRWM or recommended-operating-voltage table for VBUS or the I/Os. VRM = 5.25 V appears only as the leakage test condition, and the leakage test pins are not stated (ds p. 2). There is no peak pulse current or peak pulse power rating beyond the two VCL test points (ds p. 2).
- **VBUS left open or tied to 3.3 V:** the library declares `VBUS` as a required `power_in` pin. Every application in the source ties it to the bus VBUS or a +VCC rail with 100 nF (ds pp. 9–11). The source does not say whether VBUS may float, or what changes when it is tied to a lower rail.
- **Pin pairs:** the library models pads 1/6 and 3/4 as separate pins (`IO1_A`/`IO1_B`, `IO2_A`/`IO2_B`), both required. The source draws each pair as one internal node (ds p. 1) and as in/out of a flow-through layout (ds p. 11). It gives no through-current rating, and no series resistance except the PSpice RI/O = 110 mΩ and LI/O = 750 pH per pin (model only, ds p. 11). It does not say whether one end of a pair may be left unconnected.
- **Pin 1 on the land:** Figure 19 does not number the pads (ds p. 13), and Figure 18 does not mark pin 1 (ds p. 12). The library's pad 1 position (upper-left, counter-clockwise numbering) rests on the functional-diagram top view (ds p. 1) and the unlabelled dot in Figure 22 (ds p. 16). The two are consistent but not dimensioned.
- **Clamping and leakage over temperature:** no guaranteed values. Table 2 is at 25 °C only (ds p. 2), and Figure 3 is typical (ds p. 3).
- **Datasheet errata:** the worked example gives VCL- = -13 V, then uses -13.1 V (ds p. 4). Section 2.2 ends with an empty cross-reference "(see )" (ds p. 4).
- **Scope:** only SOT23-6L (USBLC6-2SC6) is modelled. SOT-666 (USBLC6-2P6, Table 4 and Figure 21, ds pp. 14–15) has no device variant or footprint in `lib/esd`.
