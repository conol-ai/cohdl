# Manufacturer alias review: the 40 proposals of the B2 full run

This is the review record for the manufacturer aliases proposed by the B2 full run of 2026-11-06 (`lcsc_backfill/2026-11-06-full/alias_proposals.json`). None of them is in `tools/sourcing_data/mfr_aliases.json` yet. An alias lets a supplier's manufacturer name stand for the library's; it is applied by `lcsc_backfill.py` and `promote_lcsc_map.py`, uploaded to the registry's `manufacturer_aliases` table (plan D2), and used by `cohdl suppliers` when it compares a binding with the catalog. The three aliases already accepted are recorded in `2026-11-05-mfr-aliases.md`.

## What an alias does, and what it does not

- It applies to every part number whose page shows that supplier name, not only to the rows below.
- It joins names; it never joins different MPNs. A row is mapped only when the page's MPN equals the library's.
- `mfr_aliases.json` maps each supplier name to exactly one library name. A supplier name cannot stand for two library names.

A wrong alias would let a binding to another company's part pass the audit, so each one needs a human decision before it is added.

## Evidence

All of it comes from the repository's own data:
- the proposals, `lcsc_backfill/2026-11-06-full/alias_proposals.json`;
- the identities behind them, the `manufacturer_not_matched` rows of `lcsc_backfill/2026-11-06-full/review.json`, each with its candidate part numbers and their pages' manufacturer names;
- the confirmed part pages, `lcsc_backfill/2026-11-06-full/confirmed.json`;
- the library's AVL, for the packages listed.

"Identities" counts the library (mfr, mpn) pairs that would map through the alias.

What a reviewer checks, for each row:
1. The two names are the same company.
2. No other manufacturer could be meant by the supplier name.
3. The examples are that company's products.

## Same company by name

These read as one company's name in two forms: a legal or regional suffix, a case change, or a short form. Notes:
- **19 and 20** are two supplier names for Infineon (Cypress is part of Infineon), each on its own part number for `CYPD6227-96BZXI`. With both accepted, that identity has two confirmed part numbers and may end as `tied_after_priority`.
- **7**: `RT9080-33GJ5` also has a `TECH PUBLIC` page (row 33). Accepting 7 and rejecting 33 maps it to the Richtek part number only.
- **8**: the library's `Samsung` on the CL MLCCs means Samsung Electro-Mechanics, the maker of that series.

| # | Supplier name (page) | Library name | Identities | Library packages | Examples |
|---|---|---|---|---|---|
| 1 | `Espressif Systems` | `Espressif` | 135 | `@espressif/esp32` | C2838500 `ESP32-C3`, C2934569 `ESP32-C3-MINI-1-H4` |
| 2 | `Jiangsu RUNIC Tech` | `Run-IC` | 6 | `@contrib/analog-switch`, `@contrib/level-shifter` | C2762288 `RS0104YQ`, C3515913 `RS0104YTQE12` |
| 3 | `NXP Semicon` | `NXP` | 6 | `@contrib/rtc`, `usb` | C5151540 `PCF85063AT/AY`, C404360 `PCF85063ATL/1,118` |
| 4 | `Winbond Elec` | `Winbond` | 6 | `flash` | C190862 `W25Q128JVPIQ`, C2803379 `W25Q128JVPIQ` |
| 5 | `Murata Electronics` | `Murata` | 5 | `passive` | C76891 `BLM15PD121SN1D`, C337893 `DFE201612E-2R2M=P2` |
| 6 | `MICRONE(Nanjing Micro One Elec)` | `Microne` | 4 | `@contrib/ldo` | C53100 `ME6211C15M5G-N`, C487906 `ME6211C15M5G-N` |
| 7 | `Richtek Tech` | `Richtek` | 4 | `@contrib/ldo`, `@richtek/dcdc`, `ldo` | C3187669 `RT6150B-33GQW`, C841192 `RT9080-33GJ5` |
| 8 | `Samsung Electro-Mechanics` | `Samsung` | 4 | `passive` | C14445 `CL05A105KP5NNNC`, C1525 `CL05B104KO5NNNC` |
| 9 | `Abracon LLC` | `Abracon` | 3 | `osc`, `passive` | C20625731 `ABM8-272-T3`, C179635 `ABS07-32.768KHZ-9-T` |
| 10 | `AWINIC(Shanghai Awinic Tech)` | `Awinic` | 2 | `@contrib/haptic`, `@contrib/led-driver` | C5162564 `AW21009QNR`, C5162563 `AW86224AFCR` |
| 11 | `Luat` | `Luat (Hezhou)` | 2 | `cellular` | C5246451 `Air780E`, C5360889 `Air780EG` |
| 12 | `SEMTECH` | `Semtech` | 2 | `@contrib/lora` | C191341 `SX1262IMLTRT`, C125969 `SX1280IMLTRT` |
| 13 | `TDK InvenSense` | `TDK/InvenSense` | 2 | `@contrib/mic`, `mic` | C3171773 `ICS-40300`, C3032242 `MMICT3902-00-012` |
| 14 | `WCH(Jiangsu Qin Heng)` | `WCH` | 2 | `@wch/ch32` | C5187096 `CH32V003F4P6`, C3001172 `CH32V203C8T6` |
| 15 | `Cellwise` | `CellWise Microelectronics` | 1 | `@contrib/fuel-gauge` | C881838 `CW2015CHBD` |
| 16 | `Diodes Incorporated` | `Diodes Inc` | 1 | `diode` | C83528 `1N4148W-7-F` |
| 17 | `Foshan NationStar Optoelectronics` | `NATIONSTAR` | 1 | `led` | C130719 `NCD0402R1` |
| 18 | `Global Connector Technology` | `GCT` | 1 | `usb` | C3020560 `USB4105-GF-A` |
| 19 | `Infineon Technologies` | `Infineon` | 1 | `usb` | C17495453 `CYPD6227-96BZXI` |
| 20 | `Infineon/Cypress Semicon` | `Infineon` | 1 | `usb` | C22453211 `CYPD6227-96BZXI` |
| 21 | `Jiangsu Changjing Electronics Technology Co., Ltd.` | `Jiangsu Changjing Electronics Technology` | 1 | `transistor` | C2146 `S8050 J3Y(RANGE:200-350)` |
| 22 | `Korean Hroparts Elec` | `HRO` | 1 | `usb` | C165948 `TYPE-C-31-M-12` |
| 23 | `OPSCO Optoelectronics` | `Dongguan OPSCO Optoelectronics` | 1 | `led` | C22466970 `SK6812MINI-E-012` |
| 24 | `PANASONIC` | `Panasonic` | 1 | `passive` | C2162098 `6SVPC220M` |
| 25 | `ProTek Devices` | `ProTek` | 1 | `@contrib/esd` | C85355 `GBLC05C-LF-T7` |
| 26 | `SGMICRO` | `SG Micro` | 1 | `@contrib/charger` | C5153801 `SGM41562BXG/TR` |
| 27 | `Torex Semicon` | `Torex Semiconductor` | 1 | `@contrib/ldo` | C2831490 `XC6206P182MR-G` |
| 28 | `U-BLOX` | `u-blox` | 1 | `@contrib/gnss` | C6990279 `MIA-M10Q-00B` |
| 29 | `XI'AN Aerosemi Tech` | `Xi'an Aerosemi Tech` | 1 | `load-switch` | C42441843 `MT9700-N` |
| 30 | `XINLUDA` | `Xinluda` | 1 | `@contrib/io-expander` | C609791 `XL9555` |
| 31 | `etasolution` | `ETA Solutions` | 1 | `@contrib/ldo` | C7465551 `ETA5060V330S8F` |
| 32 | `hongjiacheng` | `Zhuhai Hongjiacheng Technology` | 1 | `esd` | C7420372 `H5VL10B` |

## Needing attention

| # | Supplier name (page) | Library name | Identities | Library packages | Examples |
|---|---|---|---|---|---|
| 33 | `TECH PUBLIC` | `Richtek` | 1 | `@contrib/ldo` | C20617299 `RT9080-33GJ5` |
| 34 | `Yongfukang` | `NSIWAY` | 1 | `audio-amp` | C5246378 `NS4168` |
| 35 | `Johanson Dielectrics` | `Johanson Technology` | 1 | `antenna` | C2917717 `2450AT18B100E` |
| 36 | `Analog Devices` | `Analog Devices/Maxim Integrated` | 1 | `switches` | C79401 `MAX16054AZT+T` |
| 37 | `HRS(Hirose)` | `HRS` | 2 | `connectors` | C506791 `FH12-10S-0.5SH(55)`, C202112 `FH12-24S-0.5SH(55)` |
| 38 | `HRS(Hirose)` | `Hirose Electric` | 2 | `connectors` | C202113 `FH12-26S-0.5SH(55)`, C88373 `U.FL-R-SMT-1(10)` |
| 39 | `Shenzhen Nsiway Tech` | `NSIWAY` | 1 | `audio-amp` | C910588 `NS4168` |
| 40 | `Shenzhen Nsiway Tech` | `Nsiway` | 1 | `@contrib/audio-amp` | C189961 `NS4150B` |

- **33, `TECH PUBLIC` → `Richtek`: not the same company.** TECH PUBLIC sells its own parts under other makers' part names. The same identity has a Richtek page (row 7). Recommended: reject.
- **34, `Yongfukang` → `NSIWAY`: not the same company.** The same identity has a Shenzhen Nsiway page (row 39). Recommended: reject.
- **35, `Johanson Dielectrics` → `Johanson Technology`: two companies.** Johanson Dielectrics makes capacitors, Johanson Technology RF components such as this chip antenna. The alias would join every Johanson Dielectrics page to Johanson Technology. Recommended: reject.
- **36, `Analog Devices` → `Analog Devices/Maxim Integrated`.** The library name is a combined form, used on this one part. The alias would join every Analog Devices page to it. Recommended: reject, and settle the library's name for this part instead.
- **37 and 38, `HRS(Hirose)`; 39 and 40, `Shenzhen Nsiway Tech`.** The library spells each of these companies two ways (`HRS` and `Hirose Electric`; `NSIWAY` and `Nsiway`), and one supplier name can stand for only one library name. Recommended: choose one library spelling per company, accept that row, and reject the other until the library uses one name.

## Review

The proposals are reviewed in one issue, assigned to Tony Huang (`conol-tony`), by row number. The reviewer's comment there, accept or reject for each row, is the review: it names the reviewer, the date and the decisions.

| Rows | Issue | Decision |
|---|---|---|
| 1–40 | (to be opened) | Pending |

An accepted alias is added to `mfr_aliases.json`, and its rows are promoted through `promote_lcsc_map.py`. A rejected one stays out, and its identities stay in review.
