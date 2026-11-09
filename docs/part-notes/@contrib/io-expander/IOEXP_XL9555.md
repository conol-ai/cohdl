# XL9555 — 16-bit I²C/SMBus GPIO expander with interrupt and weak I/O pull-ups, TSSOP24

## Identity

```json
{
  "schema": "part-note/1",
  "device": "@contrib/io-expander::XL9555",
  "variants": ["SOP24", "SSOP24", "TSSOP24"],
  "parts": ["@contrib/io-expander::IOEXP_XL9555"],
  "identities": [
    {"mfr": "Xinluda", "mpn": "XL9555", "lcsc": "C609791"}
  ],
  "sources": [
    {"id": "ds", "title": "XL9535, XL9555 16-bit I2C-bus and SMBus I/O port Expander With Interrupt Output and Configuration Registers, Product datasheet", "revision": "Rev 2.4 in every page footer; the page-1 header reads Rev. 2.3 — 2018",
     "url": "https://www.xinluda.com", "sha256": "8d6805fbf1e452d674bc0d3868f5c047065e82317a9f6fc6d5bfd22fb6e58d78", "local": "lib/@contrib/io-expander/docs/xl9535-xl9555.pdf"}
  ],
  "status": "draft"
}
```

Page numbers are 1-based PDF page indices. They equal the printed "N / 25" page numbers. The recorded URL is only the Xinluda home page, which is what the package README and the PDF footer give. The README also says the PDF was not downloaded from Xinluda but "retrieved from an authorized source", which it does not name. No document URL is recorded.

The library declares one device with four package variants and one part, `IOEXP_XL9555` (TSSOP24). SOP24, SSOP24 and TSSOP24 share one pin map (ds p. 2, p. 3), so the checked `## Pins` table covers all three. QFN24 has a different map and is in its own section below, which `tools/part_notes.py` does not check. The datasheet also covers the XL9535, which has no I/O pull-ups; the library models only the XL9555.

## Summary

The XL9555 is a 16-bit GPIO expander on I²C/SMBus, 2.3–5.5 V, with polarity inversion and configuration registers and an open-drain, active-low interrupt output (ds p. 1). Three address pins select one of eight addresses, 0x20–0x27 (ds p. 11). All 16 I/Os are inputs at power-on; the outputs are push-pull (totem pole) when enabled (ds p. 3). Unlike the XL9535, each XL9555 I/O has a weak pull-up to VCC (ds p. 3, p. 14). That pull-up costs current: standby current is 1.1 mA typical, 1.5 mA max with the inputs held at GND, against 1 µA max with them at VCC (ds p. 6). Choose the XL9535 instead where inputs sit low for long periods on a battery. The parts are rated for −40 °C to +85 °C only (ds p. 2, p. 5).

## Key parameters

VCC = 2.3 V to 5.5 V, GND = 0 V, over the operating temperature range unless stated (ds p. 5). Rows are for the XL9555 where the source splits by part.

| Parameter | Min | Typ | Max | Unit | Conditions | Source |
|---|---|---|---|---|---|---|
| Supply current ICC, operating | — | 135 | 200 | µA | VCC = 5.5 V, I/Os inputs, fSCL = 100 kHz, no load | ds p. 5 |
| Standby current ISTB, inputs at GND | — | 1.1 | 1.5 | mA | VCC = 5.5 V, fSCL = 0 kHz, no load | ds p. 6 |
| Standby current ISTB, inputs at VCC | — | 0.25 | 1 | µA | VCC = 5.5 V, fSCL = 0 kHz, no load | ds p. 6 |
| Additional standby current ΔICC | — | — | 1.5 | mA | One input at VCC − 0.6 V, others at VCC or GND | ds p. 6 |
| Power-on reset, VCC rising VPORR | — | 1.7 | 2.2 | V | VIN = VCC or GND, no load | ds p. 6 |
| Power-on reset, VCC falling VPORF | 0.7 | 1 | — | V | VIN = VCC or GND, no load | ds p. 6 |
| SCL/SDA VIL / VIH | −0.5 / 0.7 × VCC | — | 0.3 × VCC / 5.5 | V | — | ds p. 6 |
| SDA sink current IOL | 3 | — | — | mA | VOL = 0.4 V | ds p. 6 |
| SCL/SDA input capacitance CI | — | 6 | 10 | pF | VIN = GND | ds p. 6 |
| I/O VIL / VIH | −0.5 / 0.7 × VCC | — | 0.3 × VCC / 5.5 | V | — | ds p. 6 |
| I/O sink current IOL | 8 | 8 to 20 | — | mA | VOL = 0.5 V | ds p. 6 |
| I/O sink current IOL | 10 | 10 to 24 | — | mA | VOL = 0.7 V | ds p. 6 |
| I/O VOH | 1.8 / 2.6 / 4.1 | — | — | V | IOH = −8 mA; VCC = 2.3 / 3 / 4.75 V | ds p. 6 |
| I/O VOH | 1.7 / 2.5 / 4 | — | — | V | IOH = −10 mA; VCC = 2.3 / 3 / 4.75 V | ds p. 6 |
| I/O high-level input leakage ILIH | — | — | 1 | µA | VIN = VCC = 5.5 V | ds p. 6 |
| I/O low-level input leakage ILIL (pull-up current) | — | — | −100 | µA | VIN = GND, VCC = 5.5 V | ds p. 6 |
| I/O capacitance CI / CO | — | 3.7 | 5 | pF | — | ds p. 6 |
| INT sink current IOL | 3 | — | — | mA | VOL = 0.4 V | ds p. 6 |
| A0–A2 VIL / VIH | −0.5 / 0.7 × VCC | — | 0.3 × VCC / 5.5 | V | — | ds p. 7 |
| A0–A2 leakage IL | −1 | — | 1 | µA | VIN = VCC or GND | ds p. 7 |
| SCL clock frequency fSCL | 0 | — | 400 | kHz | Fast mode (100 kHz standard mode) | ds p. 7 |
| Output data valid tpv(Q) | — | — | 200 | ns | From SCL to I/O | ds p. 7 |
| Interrupt valid / reset time tiv, tirst | — | — | 4 | µs | On pin INT | ds p. 7 |
| RθJA, TSSOP24 | — | 48.4 | — | °C/W | Minimum pad spacing, FR4, 10 mm × 1 inch 2 oz trace, no air flow | ds p. 5 |

## Absolute maximum ratings

| Parameter | Limit | Unit | Source |
|---|---|---|---|
| Supply voltage VCC | −0.5 to +6 | V | ds p. 4 |
| Input pin voltage VIN | −0.5 to +6 | V | ds p. 4 |
| Output pin voltage VO | −0.5 to +6 | V | ds p. 4 |
| Input clamp current IIK (VIN < 0) | −20 | mA | ds p. 4 |
| Output clamp current IOK (VO < 0) | −20 | mA | ds p. 4 |
| I/O clamp current IIOK (VO < 0 or VO > VCC) | ±20 | mA | ds p. 4 |
| Continuous output low current IOL (VO = 0 to VCC) | +50 | mA | ds p. 4 |
| Continuous output high current IOH (VO = 0 to VCC) | −50 | mA | ds p. 4 |
| Continuous current through GND | −250 | mA | ds p. 4 |
| Continuous current through VCC | +160 | mA | ds p. 4 |
| Total power dissipation Ptot | 200 | mW | ds p. 4 |
| Storage temperature | −65 to +150 | °C | ds p. 4 |
| ESD, HBM, all pins | 2000 | V | ds p. 5 |
| ESD, MM, all pins (printed as "Charged Device Model (MM)") | 200 | V | ds p. 5 |
| ESD, CDM, all pins | 1000 | V | ds p. 5 |

## Recommended operating conditions

| Parameter | Min | Max | Unit | Source |
|---|---|---|---|---|
| Supply voltage VCC | 2.3 | 5.5 | V | ds p. 5 |
| VIH, SCL, SDA, A0–A2, P00–P17 | 0.7 × VCC | VCC | V | ds p. 5 |
| VIL, SCL, SDA, A0–A2, P00–P17 | −0.5 | 0.3 × VCC | V | ds p. 5 |
| I/O high-level output current IOH, per pin | — | −10 | mA | ds p. 5 |
| I/O low-level output current IOL, per pin | — | 25 | mA | ds p. 5 |
| I/O sink current per octal (P00–P07 or P10–P17) | — | 100 | mA | ds p. 7 |
| I/O sink current, device total | — | 200 | mA | ds p. 7 |
| I/O source current, all I/Os (80 mA per octal) | — | 160 | mA | ds p. 7 |
| Operating free-air temperature TA | −40 | 85 | °C | ds p. 5 |

The recommended VIH maximum is VCC (ds p. 5), while the electrical characteristics allow inputs up to 5.5 V (ds p. 6) and the I/O description allows inputs above VCC up to 5.5 V (ds p. 14).

## Pins

SOP24, SSOP24 and TSSOP24 (variants `SOP24`, `SSOP24`, `TSSOP24`; the bound part is TSSOP24).

| Pad | Library pin | Datasheet name | Direction | Function | Source |
|---|---|---|---|---|---|
| 1 | INT | INT (overbar, active low) | output, open-drain | Interrupt; pull up to VCC | ds p. 2, p. 3 |
| 2 | A1 | A1 | input | Address bit 1; tie directly to VCC or GND | ds p. 3 |
| 3 | A2 | A2 | input | Address bit 2; tie directly to VCC or GND | ds p. 3 |
| 4 | P00 | P00 | I/O | Port 0 bit 0; input at power-on | ds p. 3 |
| 5 | P01 | P01 | I/O | Port 0 bit 1 | ds p. 3 |
| 6 | P02 | P02 | I/O | Port 0 bit 2 | ds p. 3 |
| 7 | P03 | P03 | I/O | Port 0 bit 3 | ds p. 3 |
| 8 | P04 | P04 | I/O | Port 0 bit 4 | ds p. 3 |
| 9 | P05 | P05 | I/O | Port 0 bit 5 | ds p. 3 |
| 10 | P06 | P06 | I/O | Port 0 bit 6 | ds p. 3 |
| 11 | P07 | P07 | I/O | Port 0 bit 7 | ds p. 3 |
| 12 | GND | GND | ground | Supply ground | ds p. 3 |
| 13 | P10 | P10 | I/O | Port 1 bit 0 | ds p. 3 |
| 14 | P11 | P11 | I/O | Port 1 bit 1 | ds p. 3 |
| 15 | P12 | P12 | I/O | Port 1 bit 2 | ds p. 3 |
| 16 | P13 | P13 | I/O | Port 1 bit 3 | ds p. 3 |
| 17 | P14 | P14 | I/O | Port 1 bit 4 | ds p. 3 |
| 18 | P15 | P15 | I/O | Port 1 bit 5 | ds p. 3 |
| 19 | P16 | P16 | I/O | Port 1 bit 6 | ds p. 3 |
| 20 | P17 | P17 | I/O | Port 1 bit 7 | ds p. 3 |
| 21 | A0 | A0 | input | Address bit 0; tie directly to VCC or GND | ds p. 3 |
| 22 | SCL | SCL | input | I²C clock; pull up to VCC | ds p. 3 |
| 23 | SDA | SDA | I/O | I²C data; pull up to VCC | ds p. 3 |
| 24 | VCC | VCC | power | Supply, 2.3–5.5 V | ds p. 3, p. 5 |

Top view: pads 1–12 down the left side, pads 13–24 up the right side, pin 24 opposite pin 1 (Figure 1, ds p. 2). The pin-1 mark on the TSSOP24 body is at the pin-1 corner (ds p. 24). `SOP24P65X640X120N` numbers its pads the same way.

## Pins, QFN24 variant (not checked by the tool)

Variant `QFN24`. No library part uses it. Pad names and numbers from Table 2 (ds p. 3); positions from Figure 2 (ds p. 2).

| Pad | Library pin | Datasheet name | Source |
|---|---|---|---|
| 1–8 | P00–P07 | P00–P07 | ds p. 3 |
| 9 | GND | GND | ds p. 3 |
| 10–17 | P10–P17 | P10–P17 | ds p. 3 |
| 18 | A0 | A0 | ds p. 3 |
| 19 | SCL | SCL | ds p. 3 |
| 20 | SDA | SDA | ds p. 3 |
| 21 | VCC | VCC | ds p. 3 |
| 22 | INT | INT | ds p. 3 |
| 23 | A1 | A1 | ds p. 3 |
| 24 | A2 | A2 | ds p. 3 |
| ePAD (library pad name `Y`) | EPAD | ePAD | ds p. 3 |

Top view: pads 1–6 down the left side, 7–12 along the bottom, 13–18 up the right side, 19–24 right to left along the top, pin-1 index at the top-left (ds p. 2). `pins[QFN24]` agrees with Table 2 for pads 1–24.

## Typical application

- I²C address: 0 1 0 0 A2 A1 A0, so 0x20 with all address pins low up to 0x27 with all high (ds p. 11). Tie A0, A1 and A2 directly to VCC or GND; never float them, and do not change them while powered (ds p. 3, p. 11).
- Pull-ups: SCL, SDA and INT to VCC (ds p. 3, p. 15). Figure 21 uses 10 kΩ (ds p. 19).
- Power-on state: all I/Os inputs (configuration registers 0xFF), output registers 0xFF, polarity inversion 0x00 (ds p. 12, p. 13).
- Reset: there is no reset pin. To reset, lower VCC below 0.2 V for at least 5 µs, then restore it (ds p. 7, note 9; p. 14).
- LED drive: sink LED current into an I/O. To avoid extra supply current when an LED is off, keep the I/O at or above VCC, e.g. with a high-value resistor across the LED or a lower VCC (ds p. 19, p. 20).
- Interrupt: INT goes low when an input changes and clears when the input returns or the port is read; Port 0 and Port 1 clear separately (ds p. 15).
- Decoupling: the source shows no VCC bypass capacitor and gives no value.

## Layout notes

- The source gives TSSOP24 package dimensions but no land pattern (ds p. 24): e = 0.65 mm, lead span E1 = 6.25–6.55 mm, body E = 4.30–4.50 mm, D = 7.70–7.90 mm, b = 0.19–0.30 mm, L = 0.50–0.70 mm, A ≤ 1.20 mm.
- `SOP24P65X640X120N` is the library's own derivation from those values: 1.78 × 0.42 mm pads on 5.60 mm row centres at 0.65 mm pitch. The source has nothing to compare it with.
- QFN24 (no library part): connect the exposed pad directly to ground, solder it to a matching board pad and add thermal vias (ds p. 3, note 2).

## Pitfalls

- Inputs held low draw current through the XL9555's internal pull-ups: up to 100 µA per pin (ILIL, ds p. 6) and up to 1.5 mA in standby (ds p. 6). Use the XL9535 or keep idle inputs high where standby current matters.
- The internal pull-up value is not given; the source only says "a high value resistor" (ds p. 3, p. 13). Do not rely on it as a defined bias for external circuits.
- An I/O input held a little below VCC (for example by an off LED) adds up to 1.5 mA (ΔICC, ds p. 6).
- Per-pin, per-octal and total current limits are separate: 25 mA per I/O, 100 mA per octal, 200 mA total sink, 160 mA total source (ds p. 5, p. 7).
- A 5 V I²C bus or 5 V I/O inputs are allowed (5 V tolerant, inputs up to 5.5 V, ds p. 1, p. 14), but an I/O configured as an output drives to VCC (ds p. 14).
- The device does not respond to the general call address (ds p. 9), so a general-call reset does not reset it.
- A port changed from output to input can raise a false interrupt (ds p. 15).

## Not established by the sources

- **INT typing:** the library types INT as `output` and names it `INT`. The datasheet marks it active low and open-drain (ds p. 2, p. 3, p. 15). An ERC that treats `output` as always driving will not ask for the pull-up or allow a wired-OR interrupt line. The library comment does say "open-drain, needs pull-up".
- **QFN24 exposed-pad pad name:** `pins[QFN24]` declares the exposed pad as pad `Y`. In Table 2, "Y" is the entry in the QFN column for ePAD, meaning the package has one; it is not a pad number (ds p. 3). No footprint or part uses the variant, and the checker cannot read pad `Y`.
- **Revision:** the library and README cite Rev 2.4, which every page footer carries. The page-1 header says "Rev. 2.3 — 2018" (ds p. 1). The PDF has no revision history.
- **Source provenance:** the README says the PDF came from an unnamed "authorized source" because the manufacturer site blocked automated download. The official URL is only the Xinluda home page.
- **TSSOP24 land:** no manufacturer land pattern exists in the source; the library land is a derivation (see Layout notes).
- **Datasheet errors:** the MM ESD row is labelled "Charged Device Model (MM)" (ds p. 5). The XL9555 typical IOL is printed as ranges "(8 to 20)" and "(10 to 24)" mA (ds p. 6). tpv(Q) is defined only for the XL9535 (ds p. 8, note 18). Several timing diagrams are images with no values (ds pp. 8–9).
- **Not given:** the internal pull-up resistance, a VCC bypass capacitor, I/O behaviour while VCC is below VPOR, the board for the RθJA values, and any land pattern.
