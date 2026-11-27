# Offline run for merging main into the SOD-123 Zeners, 2026-11-26

This directory holds the B2 run repeated **offline** over the library after main was merged into `lib/hongjiacheng-zener` (#130). It exists because each side had promoted a run the other did not cover: main the UNI-ROYAL run (`../2026-11-18-uniroyal/`, 12,616 rows), this branch the Zener run (`../2026-11-21-hongjiacheng-zener/`, 12,455 rows). Replaying both gives the 12,744 rows of the union, but they would cite two confirmed snapshots, and `tools/w3_groups.py` requires the map to cite one. This run reads every page from the fetch cache and maps the union under one snapshot. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:**

```sh
python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-26 --max-requests 0 --lib-dir LIB
```

The tool ran unchanged, behind a wrapper that refuses every uncached page with `PolicyError` before a request (as `tools/lcsc_backfill_approved.py run` does for a page off its list). With `--max-requests 0` alone, the first uncached page would ask for `robots.txt`, exhaust the budget and end the loop, leaving every later page unread.

**No request was sent.** `requests.json` lists 15,166 cache reads, sitemaps included. Raw pages stay in the local cache; none is stored here.

**Inputs:**
- LIB is a temporary copy of the merged library plus one AVL block per MPN of the UNI-ROYAL run's inputs, as `../2026-11-18-uniroyal/README.md` reconstructs them (the distinct MPNs of the `alternates`, `zero_ohm` and `openwalkietalkie_rev_b` arrays in `tools/passive_data/uniroyal_dry_run.json`, manufacturer `UNI-ROYAL`). Only one of those 289 is not in the library: `0402WGF1803TCE` (C25760), the Rev B order line. Without it the run maps 12,743 rows, and C25760's row would keep the older snapshot.
- The alias table is main's (SHA-256 `46070c3f…`, with the reviewed UNI-ROYAL alias). The candidate list is `tools/sourcing_data/jlcparts/2026-10-27/basic_preferred.json`, as before.

**Result:**
- The run covers 21,922 AVL entries and 21,632 identities, and maps 12,744: every identity of the two runs above, each with the same part number, class and match rule. No row was added or lost.
- 8,888 identities are for review, the same cases as before:
  - 141 are `fetch_failed` (`PolicyError`, refused before any request): the 140 Zeners outside the B1 snapshot whose pages were never approved, and the SGMICRO identity.
  - The rest are unchanged: no candidate page, ties after the priority rule, no confirmed match, unparsed pages, a manufacturer not matched.
- `alias_proposals.json` holds the one proposal the earlier runs had. No alias was added.

**Not a binding.** A map row is the part number's identity, not placement evidence; no binding or frame row changed because of this run.

**Promotion:** `promote_lcsc_map.py` re-checked the run against the current alias table and its confirmed pages, and rewrote all 12,744 rows. Every row now cites this run's confirmed pages (SHA-256 `4829cece…`) and date. The W3 group records were regenerated with `tools/w3_record.py`; only their `stock_source` changed.
