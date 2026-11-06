# B2 full run, 2026-11-06

This is the full run of library plan B2: every primary and alternate AVL entry under `lib/`, looked up against the LCSC catalog through JLCPCB's public `partdetail` pages. The 200-MPN sample of 2026-10-30 (`../2026-10-30-sample200-aliases/`) was its trial. Nothing here changes `tools/sourcing_data/lcsc_map.json`: rows are promoted from this run only after review.

## The run

- **Tree:** `origin/main` at `455f6c7`, with the reviewed `mfr_aliases.json` (SHA-256 `86e89089…`, three aliases, issues #62–#64) and the candidate list `jlcparts/2026-10-27/basic_preferred.json`.
- **Command:** `python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-06 --max-requests 12400`.
- **Fetching:** 2026-11-06, 02:38 to 09:35 (UTC+8), under the shared client's rules: an honest user agent, at least 2 s between requests, `robots.txt` obeyed, no `/api` path. 12,268 requests; 269 pages came from the cache of earlier runs. No 403 or 429 response, and the run was not stopped. Raw pages stay in the local cache; none is stored here.
- **One failed page.** The overnight run had one fetch failure: the only candidate of Yageo `RC0805FR-07165KL`. It was fetched again at 09:45 (one request), and the outputs here were then written from the cache. That second pass is what `provenance.json` describes (`requests: 1`, `cache_hits: 12536`). A third pass with `--max-requests 0` reproduces all six files byte for byte.

The snapshot dates in `requests.json` run from 2026-10-30 (pages cached by the sample) to 2026-11-06.

## Results

| | Count |
|---|---|
| AVL entries | 21,017 |
| Unique (mfr, mpn) | 21,015 |
| Partdetail pages fetched and parsed | 12,387 of 12,389 |
| Mapped (`lcsc_map.json`) | **12,106 (57.6%)** |
| — manufacturer name equal | 2,480 |
| — through a reviewed alias | 9,626 |
| For review (`review.json`) | 8,909 |

Mapped rows by catalog class: 2 basic, 7 preferred, 12,097 extended. No part number maps to more than one identity. Every one of the 108 rows of the current `lcsc_map.json` is here with the same part number.

Review reasons:

| Reason | Identities | What it means |
|---|---|---|
| `no_candidate` | 8,716 | No catalog part number carries the MPN. Mostly passive alternates and parts the catalog does not stock: Vishay 6,130, Yageo 950, STMicroelectronics 879, Samsung 712 |
| `manufacturer_not_matched` | 184 | The page's MPN matches, but its manufacturer name is neither the library's nor a reviewed alias. These give the alias proposals below |
| `no_confirmed_match` | 5 | Candidates found by the index, none confirmed by its page: JAE `SF72S006VBA(R2500)`, Molex `105017-0001`, `43045-0212`, `43045-0412`, `43045-0612` |
| `tied_after_priority` | 2 | Two confirmed part numbers and no rule to choose: STMicroelectronics `STM32C071C8T6` (C45481773, C49191142), `STM32L475VGT6` (C165886, C469418) |
| `page_unparsed` | 2 | Yageo `RC1206FR-0719R6L`, `RC1206FR-071K02L` |

## Alias proposals

`alias_proposals.json` lists 40 (supplier name, library name) pairs over the 184 identities. They are proposals only. The review record is `../../alias_reviews/2026-11-06-b2-full-aliases.md`; an alias takes effect only when the review accepts it and it is added to `mfr_aliases.json`.
