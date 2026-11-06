# B2 full run after the library's spelling fixes, 2026-11-06

This directory holds the B2 full run repeated offline once more, after the library took the manufacturer spellings #73 settled. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json` with `promote_lcsc_map.py`.

## The run

- **Tree:** that of `../2026-11-06-full-aliases/`, with the same 37-alias `mfr_aliases.json` (SHA-256 `b69b272e…`), apart from three library spellings:

  | Package | Part | MPN | Was | Now |
  |---|---|---|---|---|
  | `switches` 0.1.3 | `PUSHBUTTON_CTRL_MAX16054AZT_T` | `MAX16054AZT+T` | `Analog Devices/Maxim Integrated` | `Analog Devices` |
  | `connectors` 0.2.1 | `CONN_FPC24_0P5`, `CONN_DEBUG_10P` | `FH12-24S-0.5SH(55)`, `FH12-10S-0.5SH(55)` | `HRS` | `Hirose Electric` |
  | `@contrib/audio-amp` 0.1.1 | `AMP_NS4150B` | `NS4150B` | `Nsiway` | `NSIWAY` |

- **Command:** `python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-06 --max-requests 0`.
- **No request.** Every page came from the fetch cache (12,536 cache hits). `confirmed.json` is byte for byte that of the earlier runs, SHA-256 `861ff2ba…`.

## Results

12,284 identities map, 4 more than `../2026-11-06-full-aliases/`. Every identity mapped there maps here to the same row. The four new ones are the identities the spellings fixed, with the part numbers #73's review table named:

| Identity | LCSC | Rule |
|---|---|---|
| Analog Devices `MAX16054AZT+T` | C79401 | exact |
| Hirose Electric `FH12-10S-0.5SH(55)` | C506791 | alias (row 38) |
| Hirose Electric `FH12-24S-0.5SH(55)` | C202112 | alias (row 38) |
| NSIWAY `NS4150B` | C189961 | alias (row 39) |

All four are extended parts. 8,731 identities stay in review: 8,716 `no_candidate`, 7 `tied_after_priority`, 5 `no_confirmed_match`, 2 `page_unparsed`, and 1 `manufacturer_not_matched`, Johanson Technology `2450AT18B100E`. `alias_proposals.json` now proposes only its pair, row 35 of the review, which #73 rejected.

## Promotion

`python3 -I tools/promote_lcsc_map.py tools/sourcing_data/lcsc_backfill/2026-11-06-full-spellings` reported "12284 rows, 4 changed": the four rows above were added, and every other row was already there with the same content.
