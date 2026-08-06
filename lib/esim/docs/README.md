# eSIM — eUICC (eSIM) devices and parts

eUICC (eSIM) devices and purchasable parts, per the MFF2 standard form
factor (JEDEC MFF2 / GSMA SGP.22, ETSI TS 102 671).

## ESIM_MFF2 — 8-contact MFF2 eUICC

Molded SMD package, 5 x 6 mm body, 8 bottom contacts in two rows of four
(1.3 mm pitch, 3.2 mm row spacing). 4-wire USIM interface, directly
compatible with a modem USIM port such as the Air780E's USIM_VDD /
USIM_DAT / USIM_CLK / USIM_RST pins. Contacts 6-8 are reserved no-connects.

| Pin | Name | Role | Notes |
| --- | --- | --- | --- |
| 1 | VCC | power_in | supply, 1.8 / 3.0 V (auto-detected by modem) |
| 2 | RST | input | reset, active low |
| 3 | CLK | input | USIM clock |
| 4 | IO | bidirectional | USIM data |
| 5 | GND | power_in | ground |
| 6-8 | NC | passive | reserved contact, no connect |

NOTE: pin numbers above are this library's logical map; verify contact
assignment against the bound eUICC's datasheet (vendor eUICCs commonly
number contacts per ISO 7816-2) before fabrication.

## Parts

- `ESIM_MFF2_TRUPHONE` — Truphone SIM-S-IO3-MFF2-2 (LCSC C5122390),
  in-stock MFF2 eUICC, footprint `FP_ESIM_MFF2_5x6mm`
  (8 contacts 0.8 x 0.8 mm in two rows of four, body 5 x 6 mm).

## Datasheet

| Local file | Manufacturer document | Official source | SHA-256 |
| --- | --- | --- | --- |
| `truphone-sim-s-io3-mff2-2.pdf` | Truphone "For Things SIM" product datasheet (eSIM as standard, single SKU, global network), 5 pages | <https://www.lcsc.com/datasheet/C5122390.pdf> (LCSC C5122390) | `3e8a48d3e17191ae87b8f25f60fa195fb43dbfd3bca97f01c5794cce181542b0` |

The Truphone document describes the SIM product (eSIM capability, logistics,
global connectivity), not a mechanical MFF2 package drawing. Mechanical
verification against the official package drawing is still required before
board fabrication; contact assignments remain to be verified per the note
above.
