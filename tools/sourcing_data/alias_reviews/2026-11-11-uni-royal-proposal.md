# Manufacturer alias review: `UNI-ROYAL(Uniroyal Elec)`

**Decided by the maintainer on 2026-11-13:** use `UNI-ROYAL` in the library. On 2026-11-18 Tony accepted the supplier spelling in [issue #125](https://github.com/conol-ai/cohdl/issues/125). This decoder PR records the decision. A separate mapping change applies it.

An alias lets a supplier's manufacturer name stand for the library's. It is applied by `lcsc_backfill.py` and `promote_lcsc_map.py`, uploaded to the registry's `manufacturer_aliases` table (plan D2), and used by `cohdl suppliers` when it compares a binding with the catalog. A wrong alias would let a binding to another company's part pass the audit, so it needs a human decision before it is added.

## The alias

| # | Supplier name (page) | Library name (decided 2026-11-13) | Catalog rows | Library packages | Examples |
|---|---|---|---|---|---|
| 1 | `UNI-ROYAL(Uniroyal Elec)` | `UNI-ROYAL` | 295 (B1 snapshot) | none yet; `passive` if the alternates of PR #109 are emitted | C25744 `0402WGF1002TCE`, C17168 `0402WGF0000TCE` |

`UNI-ROYAL` is how the manufacturer's datasheet names itself in running text ("manufactured by UNI-ROYAL", p.2 §1.1; "UNI-ROYAL recommend…", p.11 §13.1). The other spellings that were offered, and not chosen: `Uniroyal` (the company's legal names all start "Uniroyal Electronics") and `UniOhm` (a mark on the datasheet's header).

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

[Decision #125](https://github.com/conol-ai/cohdl/issues/125) covers this exact spelling and groups the chip resistors by size and rated-power prefix (`0402WG`, `0603WA`, `0805W8`, `1206W4`). Start with 0402. The other three groups need additional samples.
