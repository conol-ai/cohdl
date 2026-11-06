# Run for passive 0.2.6, 2026-11-07

This directory holds the B2 run repeated over the library with `passive` 0.2.6. That version adds 37 Samsung alternates that JLCPCB stocks as Basic or Preferred (see `lib/passive/docs/README.md`, "Stocked Samsung alternates"). Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:**

```sh
python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-07 --max-requests 2
```

**One request.** Every page came from the fetch cache (12,574 hits) except one: `CL21A106KAYNNNE` has a second candidate, C2922480, that no earlier run had read. A choice is made only over every candidate page, so the run fetched that page once, under the usual policy (identified user agent, robots.txt honoured, no API). `requests.json` records it. The page reads Extended, so the Basic C15850 is chosen.

**Inputs:** those of `../2026-11-06-full-spellings/`, apart from the library tree. The alias table is unchanged.

**Result:**
- The run covers 21,054 AVL entries and 21,052 identities, and maps 12,321 of them.
- Those are the 12,284 of the previous run, each with the same part number, plus the 37 new Samsung alternates.
- The 8,731 identities for review are those of the previous run.

**Promotion:** `promote_lcsc_map.py` re-checked the run and wrote 12,321 rows. Every row now cites this run's confirmed pages and date.

**Catalog:** the catalog built from this map is unchanged: 13,604 rows, SHA-256 `3307b6b8…`. Every new part number was already a B1 snapshot row, and the B1 and map rows overlap in 51 part numbers.
