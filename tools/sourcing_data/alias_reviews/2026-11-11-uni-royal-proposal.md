# Manufacturer alias proposal: `UNI-ROYAL(Uniroyal Elec)`

**PROPOSAL. NOT REVIEWED AND NOT APPLIED.** `tools/sourcing_data/mfr_aliases.json` has no entry for this supplier name, and this record does not add one. It is the draft of a review record, prepared so a reviewer has the evidence in one place. Nothing reads this file.

An alias lets a supplier's manufacturer name stand for the library's. It is applied by `lcsc_backfill.py` and `promote_lcsc_map.py`, uploaded to the registry's `manufacturer_aliases` table (plan D2), and used by `cohdl suppliers` when it compares a binding with the catalog. A wrong alias would let a binding to another company's part pass the audit, so it needs a human decision before it is added.

## Why there is no alias yet

The earlier records review aliases that a backfill run proposed for names the library already uses. This one differs: **the library has no part by this manufacturer**, so there is no library name to map to. Two decisions are open, in this order:

1. The library's name for the manufacturer, used as `mfr:` when a UNI-ROYAL alternate is first emitted.
2. The alias from the supplier's spelling to that name.

## Proposal

| # | Supplier name (page) | Library name (proposed) | Catalog rows | Library packages | Examples |
|---|---|---|---|---|---|
| 1 | `UNI-ROYAL(Uniroyal Elec)` | `UNI-ROYAL` | 295 (B1 snapshot) | none yet; `passive` if the alternates of PR #109 are emitted | C25744 `0402WGF1002TCE`, C17168 `0402WGF0000TCE` |

`UNI-ROYAL` is how the manufacturer's datasheet names itself in running text ("manufactured by UNI-ROYAL", p.2 §1.1; "UNI-ROYAL recommend…", p.11 §13.1). Other spellings a reviewer may prefer: `Uniroyal` (the company's legal names all start "Uniroyal Electronics") and `UniOhm` (a mark on the datasheet's header).

## Evidence

- The supplier name: every one of the 295 rows of `tools/sourcing_data/jlc_catalog/2026-10-30/catalog.json` whose manufacturer contains "ROYAL" reads exactly `UNI-ROYAL(Uniroyal Elec)`; no other spelling occurs. The rows' datasheet file names read `UNI-ROYAL-Uniroyal-Elec-<MPN>`.
- The manufacturer's own document: `tools/passive_data/uniroyal_chip_series.evidence.md` (datasheet SMD-SP-001 V.11, from `www.uni-royal.cn`). Its cover names Uniroyal Electronics Global Co., Ltd. and the plant Uniroyal Electronics Industry Co., Ltd.; its headers carry the UNI-ROYAL, ROYALOHM and uniohm marks.
- The part numbers are that document's: 289 of the 295 rows decode under its part-number system and agree with their JLCPCB page on resistance, tolerance, rated power and package (`tools/uniroyal_alternates.py --validate-pages`). The other 6 are 5 resistor arrays, which the document does not cover, and 1 number with a suffix.

What a reviewer checks:
1. `UNI-ROYAL(Uniroyal Elec)` and the chosen library name are the same company.
2. No other manufacturer could be meant by the supplier name. (Royalohm and UniOhm are marks of the same group on the same datasheet. The US tyre brand Uniroyal is unrelated and makes no electronic parts.)
3. The examples are that company's products.

The alias would apply to every part number whose page shows that supplier name, the arrays included. It joins names; it never joins different MPNs.

## Review

Not opened. No issue exists and no decision has been made.

| Row | Issue | Decision |
|---|---|---|
| 1 | — | Pending |
