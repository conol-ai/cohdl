# Offline run for merging the SOT-23 branch into the SOD-323/523 Zeners, 2026-11-26

This directory holds the B2 run repeated **offline** over the library after `lib/sot23-bjt-mos` (#137, with #130 and main merged in) was merged into `lib/hongjiacheng-zener-sod323` (#138). It has the same purpose as `../2026-11-26-merge-main/` and `../2026-11-26-merge-sot23/`: the SOD-323/523 Zener run (`../2026-11-24-zener2/`) did not read the UNI-ROYAL identities, so after its promotion the map would cite two confirmed snapshots, and `tools/w3_groups.py` requires one. This run reads every page from the fetch cache and maps the union under one snapshot. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command, wrapper and inputs:** as `../2026-11-26-merge-main/README.md`, over a temporary copy of this branch's merged library plus the UNI-ROYAL run's 289 input MPNs (of which only `0402WGF1803TCE`, C25760, is not in the library). The alias table is main's (SHA-256 `46070c3f…`).

**No request was sent.** `requests.json` lists 19,021 cache reads, sitemaps included. Raw pages stay in the local cache; none is stored here.

**Result:**
- The run covers 22,221 AVL entries and 21,931 identities, and maps 12,904: every identity of `../2026-11-26-merge-sot23/` and of `../2026-11-24-zener2/`, each with the same part number, class and match rule. No row was added or lost.
- 9,027 identities are for review, the cases of those two runs: 242 `fetch_failed` (`PolicyError`, refused before any request), 38 `page_unparsed` (the 36 Zeners with an empty "Part Detail" candidate page and the two earlier ones), and the rest as before.
- `alias_proposals.json` holds the one proposal the earlier runs had. No alias was added.

**Not a binding.** A map row is the part number's identity, not placement evidence; no binding or frame row changed because of this run.

**Promotion:** `promote_lcsc_map.py` re-checked the run and rewrote all 12,904 rows. Every row now cites this run's confirmed pages (SHA-256 `2d335ade…`) and date. The W3 group records were regenerated with `tools/w3_record.py`; only their `stock_source` changed.
