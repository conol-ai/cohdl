# Manufacturer alias review: the three aliases of `mfr_aliases.json`

This is the review record for the three manufacturer aliases in `tools/sourcing_data/mfr_aliases.json` (SHA-256 `86e89089cbe1592f07dc0cc319a8a2c97d349363dc7f13a3c0fca9f1c1d429c6`). An alias lets a supplier's manufacturer name stand for the library's. It is applied by `lcsc_backfill.py` and `promote_lcsc_map.py`, uploaded to the registry's `manufacturer_aliases` table (plan D2), and used by `cohdl suppliers` when it compares a binding with the catalog.

## History

Commit `4da2f86` (2026-11-02) added the three aliases with the message "The maintainer reviewed and accepted the proposals". It names no reviewer and cites no evidence, so it is not taken as a review here. This record is a new review, made on the evidence below, and only its own sign-off counts.

## Evidence

All of it comes from the repository's own data:
- the proposals, `lcsc_backfill/2026-10-30-sample200/alias_proposals.json`;
- the confirmed part pages of the run that uses the aliases, `lcsc_backfill/2026-10-30-sample200-aliases/confirmed.json`;
- the library's AVL.

| Supplier name (page) | Library name | Map rows | MPNs on those rows | Library AVL | Page evidence |
|---|---|---|---|---|---|
| `YAGEO` | `Yageo` | 64 | 56 `RC…` thick-film resistors, 8 `CC…` MLCCs | `lib/passive` lists these MPNs with `mfr: "Yageo"`. They follow Yageo's own part-number schemes, which `tools/gen_passive.py` derives from Yageo's datasheets, and `tools/verify_passive_mpns.py` checks them against Yageo's specsheet endpoint | Every page reads `YAGEO`, and the datasheet file names read `YAGEO-<MPN>`. The two names differ only in case |
| `Vishay Intertech` | `Vishay` | 15 | 15 `CRCW…` thick-film resistors | `lib/passive` lists them as `alt { mfr: "Vishay" }` alternates (the CRCW e3 series) | Every page reads `Vishay Intertech`, and the datasheet file names read `Vishay-Intertech-<MPN>` |
| `GigaDevice Semicon Beijing` | `GigaDevice` | 1 | `GD32E230F8P6TR` (C380785, TSSOP-20) | `@gigadevice/gd32` declares it with `mfr: "GigaDevice"` | The page reads `GigaDevice Semicon Beijing` |

What a reviewer checks:
1. The two names on each row are the same company. For example, "Vishay Intertech" is short for Vishay Intertechnology, Inc., and "GigaDevice Semicon Beijing" for GigaDevice Semiconductor (Beijing) Inc.
2. No other manufacturer could be meant by the supplier name.
3. The sample rows (`alias_proposals.json`, `examples`) are that company's products.

The alias applies to every part number from the supplier that shows that manufacturer name, not only to the rows above. It joins names; it never joins different MPNs.

## Review

To be completed by the reviewing person. An alias that is not accepted here must be removed from `mfr_aliases.json`, and the 80 map rows that rely on it re-promoted, before the aliases are uploaded to the registry.

| Alias | Decision (accept / reject) | Note |
|---|---|---|
| `YAGEO` → `Yageo` | | |
| `Vishay Intertech` → `Vishay` | | |
| `GigaDevice Semicon Beijing` → `GigaDevice` | | |

- Reviewer:
- Date:
- Evidence checked (beyond the above, if any):
