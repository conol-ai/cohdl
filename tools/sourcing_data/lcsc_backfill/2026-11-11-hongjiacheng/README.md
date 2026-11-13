# Run for the six Hongjiacheng diode and TVS parts, 2026-11-11

This directory holds the B2 run repeated over the library with `diode` 0.1.2 and `esd` 0.1.3, which add six Zhuhai Hongjiacheng parts: B5819WS, SS34, 1N4148WT, SMBJ6.0CA, SMCJ36CA and SMF15CA. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:**

```sh
python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-11 --max-requests 387
```

The tool ran unchanged, behind a wrapper that adds one refusal: an uncached page outside the approved list of 386 is not requested (see "One page not read").

**387 requests:** `robots.txt` and 386 `partdetail` pages, on 2026-11-11 from 10:01 to 10:17 UTC, under the shared client's rules: an identified user agent, at least 2 s between requests, `robots.txt` obeyed, no `/api` path. No 403 or 429 response, no failed request, and the run was not stopped. Every other page came from the fetch cache (12,583 hits, sitemaps included). `requests.json` records each one. Raw pages stay in the local cache; none is stored here.

**Why 386 pages.** A choice is made only over every candidate page for an MPN, and these six MPNs are generic: the index finds 51 (B5819WS), 81 (SS34), 53 (1N4148WT), 112 (SMBJ6.0CA), 52 (SMCJ36CA) and 46 (SMF15CA) part numbers whose slug or jlcparts MPN matches. Nine of those pages were already cached. The fetch was approved for exactly this list, for these six rows.

**Inputs:** those of `../2026-11-07-samsung-b1/`, apart from the library tree. The alias table is unchanged (SHA-256 `b69b272e…`); "hongjiacheng" was already a reviewed alias of "Zhuhai Hongjiacheng Technology".

**Result:**
- The run covers 21,060 AVL entries and 21,058 identities, and maps 12,327 of them.
- Those are the 12,321 of the previous run, each with the same part number, plus six new rows:

| MPN | Part number | Rule | Class | Candidate pages | Pages with this exact MPN |
|---|---|---|---|---|---|
| 1N4148WT | C7420320 | alias | preferred | 53 | 53 |
| B5819WS | C7420331 | alias | preferred | 51 | 51 |
| SMBJ6.0CA | C19077561 | alias | preferred | 112 | 70 |
| SMCJ36CA | C19077609 | alias | preferred | 52 | 52 |
| SMF15CA | C19077510 | alias | preferred | 46 | 46 |
| SS34 | C7420365 | alias | preferred | 81 | 79 |

  For each MPN exactly one confirmed page names the manufacturer "hongjiacheng"; every other candidate page is another manufacturer's part or another MPN (the index key ignores punctuation, so `SMBJ60CA` pages are candidates of `SMBJ6.0CA`), so no choice between part numbers was needed. Preferred comes from the jlcparts candidate list, since a page shows only Basic or Extended.
- 8,731 identities are for review, as before in number. One changed: see below.
- `alias_proposals.json` holds the one proposal the previous run had. No alias was added.

**One page not read.** Since the previous run the library's SGMICRO MPN changed from `SGM2554` (no candidate) to `SGM2554AYN5G/TR`, which has one candidate page, C183309. That page was outside the approved fetch, so the wrapper refused it before any request and the tool records the identity as `fetch_failed` (`PolicyError`) in `review.json`. It is not mapped and nothing else depends on it. A later run with that one page approved would settle it; a rerun of the plain command with `--max-requests 0` stops at that page, so use a budget of 1 or the same refusal.

**Promotion:** `promote_lcsc_map.py` re-checked the run and wrote 12,327 rows. Every row now cites this run's confirmed pages and date.
