# B2 full run with the reviewed aliases, 2026-11-06

This directory holds the B2 full run (`../2026-11-06-full/`) repeated offline with the alias table that #73's review produced. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json` with `promote_lcsc_map.py`.

## The run

- **Tree:** the full run's inputs, apart from `mfr_aliases.json`, which now holds 37 aliases: the three of #62–#64 and the 34 #73 accepted (rows 1–32, 38, 39 of `../../alias_reviews/2026-11-06-b2-full-aliases.md`). Its SHA-256 is `b69b272e…`, as `provenance.json` records.
- **Command:** `python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-06 --max-requests 0`.
- **No request.** Every page came from the fetch cache that the full run filled (12,536 cache hits). `confirmed.json` is byte for byte the full run's, so every row here cites the same confirmed pages, SHA-256 `861ff2ba…`.

## Results

| | First pass (`../2026-11-06-full/`) | This run |
|---|---|---|
| Mapped (`lcsc_map.json`) | 12,106 (57.6%) | **12,280 (58.4%)** |
| — manufacturer name equal | 2,480 | 2,480 |
| — through a reviewed alias | 9,626 | 9,800 |
| For review (`review.json`) | 8,909 | 8,735 |

Every identity the first pass mapped maps here to the same part number. The 174 new identities are the ones the accepted aliases cover. By class the map holds 6 basic, 8 preferred and 12,266 extended rows.

Review reasons now:

| Reason | Identities |
|---|---|
| `no_candidate` | 8,716 |
| `tied_after_priority` | 7: 5 Espressif modules, each with two equally ranked part numbers, `STM32C071C8T6`, `STM32L475VGT6` |
| `manufacturer_not_matched` | 5: `MAX16054AZT+T`, `HRS` `FH12-10S-0.5SH(55)` and `FH12-24S-0.5SH(55)`, Johanson `2450AT18B100E`, Nsiway `NS4150B` |
| `no_confirmed_match` | 5, as in the first pass |
| `page_unparsed` | 2, as in the first pass |

`alias_proposals.json` lists the four pairs still proposed: rows 35–37 and 40 of the review, which #73 rejected. Four of the five identities behind them wait for the library fixes #73 asked for (`switches`, `connectors`, `@contrib/audio-amp`), not for aliases. Johanson `2450AT18B100E` stays in review: its only page names Johanson Dielectrics, a different company.

## Promotion

`python3 -I tools/promote_lcsc_map.py tools/sourcing_data/lcsc_backfill/2026-11-06-full-aliases` re-checked the run against the current `mfr_aliases.json` and its confirmed pages, then replaced the 108 rows of the sample (`../2026-10-30-sample200-aliases/`) and added the rest: 12,280 rows. Each of the 108 keeps its part number. The first pass (`../2026-11-06-full/`) cannot be promoted, because it used the old alias table, and `promote_lcsc_map.py` refuses it.

## Since the run

The library then took the spellings #73 settled for the four identities waiting on it, and the run was repeated offline again in `../2026-11-06-full-spellings/`. That run maps those four as well (12,284), and its rows were promoted over this one's.
