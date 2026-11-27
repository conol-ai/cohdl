# Run for the 128 Hongjiacheng SOD-123 Zeners, 2026-11-21

This directory holds the B2 run repeated over the library with `diode` 0.1.3, which adds 285 Zhuhai Hongjiacheng Zeners (`lib/diode/src/zener.cohdl`). The fetch was approved for the 128 of them that the B1 snapshot (`../../jlc_catalog/2026-10-30/`) lists on the SOD-123 package. Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:**

```sh
python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-21 --max-requests 1772
```

The tool ran unchanged, behind the same kind of wrapper as `../2026-11-11-hongjiacheng/`, which adds one refusal: an uncached page outside the approved list is not requested. The approved list is every uncached candidate page of the 128 MPNs, 1,770 of their 1,898 candidate pages (the other 128 were already cached).

**1,771 requests:** `robots.txt` and the 1,770 approved `partdetail` pages, on 2026-11-21 from 06:11 to 07:21 UTC, under the shared client's rules: an identified user agent (`cohdl-library-bot/0.1`), at least 2 s between requests, `robots.txt` obeyed, no `/api` path. No 403 or 429 response, the budget was not exhausted and the run was not stopped. Every other page came from the fetch cache (13,097 hits, sitemaps included). `requests.json` records each one. Raw pages stay in the local cache; none is stored here.

**Inputs:** those of `../2026-11-11-hongjiacheng/`, apart from the library tree. The alias table is unchanged (SHA-256 `b69b272e…`); "hongjiacheng" was already a reviewed alias of "Zhuhai Hongjiacheng Technology".

**Result:**
- The run covers 21,345 AVL entries and 21,343 identities (285 more than the previous run, all Zeners), and maps 12,455 of them.
- Those are the 12,327 of the previous run, each with the same part number, plus the 128 snapshot Zeners (table below). Each new row's part number is the one the snapshot lists for that MPN.
- For each of the 128 MPNs every candidate page was read and confirmed, and exactly one names the manufacturer "hongjiacheng" with that exact MPN; the others are other manufacturers' parts (six BZT52C MPNs also have a page for another MPN that the punctuation-free index key matches). So no choice between part numbers was needed. Every row's class is Preferred, from the jlcparts candidate list, since a page shows only Basic or Extended. All 128 rows have rule `alias`.
- 8,888 identities are for review, the 8,731 of the previous run unchanged plus 157 Zeners:
  - 140 are `fetch_failed` (`PolicyError`): Zeners outside the snapshot whose candidate pages were not on the approved list, so the wrapper refused them before any request. They are not mapped. A later run with their pages approved would settle them.
  - 17 have no candidate page: MM1W120–MM1W330 (12), MMSZ4710, MMSZ5224B, MMSZ5264B, MMSZ5265B and MMSZ5267B.
- The SGMICRO identity is still `fetch_failed`, as in the previous run.
- `alias_proposals.json` holds the one proposal the previous runs had. No alias was added.

**Not a binding.** A map row is the part number's identity, not placement evidence. A Zener is polarized and outside W3 sampled inference, so each part number binds only with its own preview reading (`../../jlc_previews/2026-11-21-board-8/` and `-board-9/`). Two of the 128, C22395534 (MM1Z2V0B) and C22395535 (MM1Z2V7B), were left out of those previews for low stock; they have map rows here but no reading.

**Promotion:** `promote_lcsc_map.py` re-checked the run and wrote 12,455 rows. Every row now cites this run's confirmed pages and date.

## The 128 rows

"Candidate pages" are the part numbers whose slug or jlcparts MPN matches the MPN's index key; "exact" are those whose page reads exactly this MPN.

| MPN | Part number | Candidate pages | Exact |
|---|---|---|---|
| BZT52B10 | C22395568 | 17 | 17 |
| BZT52B12 | C22395569 | 20 | 20 |
| BZT52B13 | C22399883 | 18 | 18 |
| BZT52B15 | C22395570 | 17 | 17 |
| BZT52B3V3 | C19077392 | 13 | 13 |
| BZT52B5V1 | C19077393 | 27 | 27 |
| BZT52B5V6 | C22399881 | 21 | 21 |
| BZT52B8V2 | C22399882 | 15 | 15 |
| BZT52C10 | C19077408 | 37 | 37 |
| BZT52C11 | C19077409 | 29 | 29 |
| BZT52C12 | C19077410 | 41 | 41 |
| BZT52C13 | C19077411 | 35 | 35 |
| BZT52C15 | C19077412 | 40 | 40 |
| BZT52C16 | C19077413 | 32 | 32 |
| BZT52C18 | C19077414 | 37 | 37 |
| BZT52C20 | C19077415 | 34 | 34 |
| BZT52C22 | C22375295 | 28 | 27 |
| BZT52C24 | C19077416 | 35 | 34 |
| BZT52C27 | C19077417 | 35 | 35 |
| BZT52C2V4 | C22375296 | 26 | 26 |
| BZT52C2V7 | C19077394 | 30 | 30 |
| BZT52C30 | C19077418 | 35 | 35 |
| BZT52C33 | C19077419 | 32 | 32 |
| BZT52C36 | C19077420 | 31 | 31 |
| BZT52C3V0 | C19077395 | 32 | 32 |
| BZT52C3V3 | C19077396 | 34 | 34 |
| BZT52C3V6 | C19077397 | 36 | 36 |
| BZT52C3V9 | C19077398 | 31 | 31 |
| BZT52C43 | C22395571 | 24 | 24 |
| BZT52C47 | C22399887 | 21 | 21 |
| BZT52C4V3 | C19077399 | 32 | 32 |
| BZT52C4V7 | C19077400 | 34 | 34 |
| BZT52C5V1 | C19077401 | 46 | 46 |
| BZT52C5V6 | C19077402 | 39 | 38 |
| BZT52C6V2 | C19077403 | 39 | 38 |
| BZT52C6V8 | C19077404 | 35 | 34 |
| BZT52C75 | C22375297 | 6 | 6 |
| BZT52C7V5 | C19077405 | 37 | 36 |
| BZT52C8V2 | C19077406 | 37 | 37 |
| BZT52C9V1 | C19077407 | 36 | 36 |
| MM1W10 | C22395527 | 4 | 4 |
| MM1W100 | C22435090 | 3 | 3 |
| MM1W11 | C22395528 | 3 | 3 |
| MM1W12 | C22435092 | 6 | 6 |
| MM1W15 | C22435095 | 6 | 6 |
| MM1W18 | C22395529 | 6 | 6 |
| MM1W20 | C22435093 | 4 | 4 |
| MM1W22 | C22395530 | 4 | 4 |
| MM1W24 | C22395531 | 5 | 5 |
| MM1W30 | C22395532 | 4 | 4 |
| MM1W36 | C22435089 | 3 | 3 |
| MM1W3V3 | C22395523 | 4 | 4 |
| MM1W3V6 | C22395524 | 4 | 4 |
| MM1W3V9 | C22395525 | 5 | 5 |
| MM1W4V7 | C22435096 | 5 | 5 |
| MM1W5V1 | C22395526 | 8 | 8 |
| MM1W5V6 | C22435086 | 5 | 5 |
| MM1W6V2 | C22435091 | 5 | 5 |
| MM1W6V8 | C22435087 | 4 | 4 |
| MM1W75 | C22395541 | 3 | 3 |
| MM1W7V5 | C22435097 | 4 | 4 |
| MM1W82 | C22395533 | 3 | 3 |
| MM1W8V2 | C22435098 | 5 | 5 |
| MM1W91 | C22435088 | 4 | 4 |
| MM1W9V1 | C22435094 | 4 | 4 |
| MM1Z10B | C22466341 | 4 | 4 |
| MM1Z12 | C22379459 | 10 | 10 |
| MM1Z12B | C22466342 | 4 | 4 |
| MM1Z15 | C22379462 | 11 | 11 |
| MM1Z16 | C22379461 | 9 | 9 |
| MM1Z18 | C22379466 | 10 | 10 |
| MM1Z18B | C22379455 | 4 | 4 |
| MM1Z20 | C22379467 | 8 | 8 |
| MM1Z24 | C22379465 | 9 | 9 |
| MM1Z27 | C22379469 | 10 | 10 |
| MM1Z2V0B | C22395534 | 2 | 2 |
| MM1Z2V7 | C22379468 | 8 | 8 |
| MM1Z2V7B | C22395535 | 2 | 2 |
| MM1Z30B | C22466339 | 3 | 3 |
| MM1Z3V0 | C22379463 | 8 | 8 |
| MM1Z3V3 | C22379456 | 12 | 12 |
| MM1Z3V3B | C22466340 | 4 | 4 |
| MM1Z3V6 | C22379457 | 10 | 10 |
| MM1Z47B | C22466345 | 3 | 3 |
| MM1Z4V7 | C22379458 | 9 | 9 |
| MM1Z5V1 | C22395538 | 14 | 14 |
| MM1Z5V1B | C22466343 | 6 | 6 |
| MM1Z5V6 | C22395539 | 13 | 13 |
| MM1Z5V6B | C22395536 | 5 | 5 |
| MM1Z62 | C22395540 | 6 | 6 |
| MM1Z62B | C22466338 | 2 | 2 |
| MM1Z6V2 | C22379464 | 11 | 11 |
| MM1Z6V2B | C22466344 | 5 | 5 |
| MM1Z6V8 | C22379460 | 8 | 8 |
| MM1Z7V5B | C22395537 | 3 | 3 |
| MMSZ4685 | C22395546 | 3 | 3 |
| MMSZ4688 | C22395547 | 3 | 3 |
| MMSZ4691 | C22395549 | 4 | 4 |
| MMSZ4692 | C22395550 | 6 | 6 |
| MMSZ4699 | C22395551 | 5 | 5 |
| MMSZ5225B | C22399894 | 7 | 7 |
| MMSZ5226B | C19077421 | 11 | 11 |
| MMSZ5227B | C22395553 | 10 | 10 |
| MMSZ5228B | C22399895 | 8 | 8 |
| MMSZ5229B | C22399896 | 9 | 9 |
| MMSZ5230B | C22395554 | 8 | 8 |
| MMSZ5231B | C19077422 | 24 | 24 |
| MMSZ5232B | C19077423 | 19 | 19 |
| MMSZ5234B | C22395555 | 11 | 11 |
| MMSZ5235B | C19077424 | 10 | 10 |
| MMSZ5236B | C22395556 | 10 | 10 |
| MMSZ5237B | C22395557 | 11 | 11 |
| MMSZ5238B | C22399897 | 6 | 6 |
| MMSZ5239B | C22395558 | 12 | 12 |
| MMSZ5240B | C19077425 | 19 | 19 |
| MMSZ5241B | C22395559 | 10 | 10 |
| MMSZ5242B | C19077426 | 22 | 22 |
| MMSZ5243B | C19077427 | 12 | 12 |
| MMSZ5245B | C19077428 | 19 | 19 |
| MMSZ5246B | C19077429 | 17 | 17 |
| MMSZ5248B | C19077430 | 20 | 20 |
| MMSZ5250B | C19077431 | 12 | 12 |
| MMSZ5251B | C22395560 | 9 | 9 |
| MMSZ5252B | C19077432 | 16 | 16 |
| MMSZ5254B | C22395561 | 14 | 14 |
| MMSZ5256B | C19077433 | 13 | 13 |
| MMSZ5258B | C22399899 | 6 | 6 |
| MMSZ5259B | C22399900 | 4 | 4 |
