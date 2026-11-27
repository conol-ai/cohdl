# Offline run for merging the Zener branch into the SOT-23 parts, 2026-11-26

This directory holds the B2 run repeated **offline** over the library after `lib/hongjiacheng-zener` (#130, with main merged in) was merged into `lib/sot23-bjt-mos` (#137). It has the same purpose as `../2026-11-26-merge-main/`: the two SOT-23 runs (`../2026-11-23-sot23/`, `../2026-11-24-sot23-2/`) did not read the UNI-ROYAL identities, so after their promotion the map would cite two confirmed snapshots, and `tools/w3_groups.py` requires one. This run reads every page from the fetch cache and maps the union under one snapshot. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command, wrapper and inputs:** as `../2026-11-26-merge-main/README.md`, over a temporary copy of this branch's merged library plus the UNI-ROYAL run's 289 input MPNs (of which only `0402WGF1803TCE`, C25760, is not in the library). The alias table is main's (SHA-256 `46070c3f…`).

**No request was sent.** `requests.json` lists 17,744 cache reads, sitemaps included. Raw pages stay in the local cache; none is stored here.

**Result:**
- The run covers 22,048 AVL entries and 21,758 identities, and maps 12,869: every identity of `../2026-11-26-merge-main/` and of the two SOT-23 runs, each with the same part number, class and match rule. No row was added or lost.
- 8,889 identities are for review. 142 are `fetch_failed` (`PolicyError`, refused before any request): the 141 of `../2026-11-26-merge-main/` and Hongjiacheng `1SS226`, which `../2026-11-24-sot23-2/` also left `fetch_failed`. The rest are the same cases as before.
- `alias_proposals.json` holds the one proposal the earlier runs had. No alias was added.

**Not a binding.** A map row is the part number's identity, not placement evidence; no binding or frame row changed because of this run.

**Promotion:** `promote_lcsc_map.py` re-checked the run and rewrote all 12,869 rows. Every row now cites this run's confirmed pages (SHA-256 `59bf978d…`) and date. The W3 group records were regenerated with `tools/w3_record.py`; only their `stock_source` changed.
