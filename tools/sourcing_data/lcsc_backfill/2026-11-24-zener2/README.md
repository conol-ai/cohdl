# SOD-323 and SOD-523 Zeners, 2026-11-24

This directory holds the B2 run repeated over the library for the 71 Zhuhai Hongjiacheng SOD-323 and SOD-523 Zeners of the B1 snapshot, which `diode` 0.1.5 adds (`lib/diode/src/zener_sod323_sod523.cohdl`). The fetch was approved for every uncached candidate page of their MPNs, 1,206 pages, listed in an approved URL list (SHA-256 `148586ef2e79600a2cb4e89000ca6048976c11fed89b70bb40a747ac32b27258`). Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:** as `../2026-11-23-sot23/`, with `--date 2026-11-24 --max-requests 1208`, behind the same wrapper (no uncached page outside the approved list is requested; the uncached candidate pages were checked to be exactly that list first).

**1,206 requests:** the approved `partdetail` pages (`robots.txt` came from the cache) on 2026-11-24, under the shared client's rules: an identified user agent (`cohdl-library-bot/0.1`), at least 2 s between requests, `robots.txt` obeyed, no `/api` path. No 403 or 429 response, the budget was not exhausted and the run was not stopped. Every other page came from the fetch cache. `requests.json` records each one. Raw pages stay in the local cache; none is stored here.

**Result:**
- The run maps 12,615 identities: the 12,580 of the previous run, each with the same part number, and 35 of the 71 Zeners, each to the part number the B1 snapshot gives (first table).
- The other 36 stay in review as `page_unparsed` (second table). Each has its snapshot page confirmed, but one or more other candidate pages of the same MPN could not be read: 62 such pages, 60 of them fetched in this run, each an empty "Part Detail" page with no part data (a listing of another manufacturer's part number that no longer shows its details). The tool chooses only over every candidate page, so an unreadable one keeps the identity in review, as for the two Yageo identities of `../2026-11-06-full/`. Treating such an empty page as "not a candidate" would be a rule change for a maintainer to decide; nothing here assumes it.
- Identities outside the approved list whose candidate pages were not cached stay `fetch_failed` (`PolicyError`), refused before any request.
- `alias_proposals.json` holds the run's proposals; no alias was added.

**Not a binding.** A map row is the part number's identity. Each Zener binds only with its own preview reading (preview board 20, `tools/frame_boards/preview-board-20/`).

## The 35 part numbers mapped

All Zhuhai Hongjiacheng Technology.

| MPN | Part number | Rule |
|---|---|---|
| BZT52B12S | C22395572 | alias |
| BZT52C16S | C19077449 | alias |
| BZT52C20S | C19077451 | alias |
| BZT52C22S | C22375288 | alias |
| BZT52C36S | C22375293 | alias |
| BZT52C3V6S | C19077438 | alias |
| BZT52C3V9S | C22375299 | alias |
| BZX584C10 | C22395577 | alias |
| BZX584C13 | C22395580 | alias |
| BZX584C15 | C19077466 | alias |
| BZX584C18 | C22395581 | alias |
| BZX584C2V4 | C22395573 | alias |
| BZX584C39 | C22395582 | alias |
| BZX584C3V3 | C19077462 | alias |
| BZX584C3V6 | C22395574 | alias |
| BZX584C3V9 | C22395575 | alias |
| BZX584C4V3 | C22395576 | alias |
| BZX584C5V1 | C19077463 | alias |
| BZX584C9V1 | C19077465 | alias |
| MM3Z10 | C19077459 | alias |
| MM3Z15 | C22399890 | alias |
| MM3Z18 | C19077461 | alias |
| MM3Z3V0 | C19077452 | alias |
| MM3Z3V6 | C19077454 | alias |
| MM3Z4V7 | C22399891 | alias |
| MM3Z51 | C22395542 | alias |
| MM3Z5V1 | C19077455 | alias |
| MM3Z6V2 | C19077457 | alias |
| MM3Z6V8 | C19077458 | alias |
| MM3Z9V1 | C22399893 | alias |
| MMSZ5232BS | C22395563 | alias |
| MMSZ5239BS | C22395564 | alias |
| MMSZ5246BS | C22395565 | alias |
| MMSZ5248BS | C22395566 | alias |
| MMSZ5250BS | C22395567 | alias |

## The 36 left in review

Each MPN's snapshot page (`hongjiacheng-<MPN>`) parsed; these candidate pages did not.

| MPN | Unreadable candidate pages |
|---|---|
| BZT52B10S | C78857 |
| BZT52B5V1S | C499804, C541634, C78478 |
| BZT52B5V6S | C699215 |
| BZT52B8V2S | C18199550, C699131 |
| BZT52B9V1S | C699179 |
| BZT52C10S | C493185, C541637 |
| BZT52C11S | C541638, C56259351 |
| BZT52C12S | C271264, C541640 |
| BZT52C13S | C523834 |
| BZT52C15S | C2886375, C499800, C541642 |
| BZT52C18S | C266584, C493208 |
| BZT52C24S | C7544586 |
| BZT52C27S | C511894, C541644 |
| BZT52C2V4S | C2886309 |
| BZT52C2V7S | C699151 |
| BZT52C30S | C266586 |
| BZT52C33S | C266587, C541645 |
| BZT52C3V0S | C2886311, C523830, C541646 |
| BZT52C3V3S | C499797, C699135, C7502062, C7544565 |
| BZT52C4V3S | C266577 |
| BZT52C4V7S | C2886316, C5125709, C7544569 |
| BZT52C5V1S | C266578, C5249652, C699201, C7544570 |
| BZT52C5V6S | C493207 |
| BZT52C6V2S | C2886369, C699209 |
| BZT52C6V8S | C2886370 |
| BZT52C7V5S | C523831 |
| BZT52C8V2S | C699144 |
| BZT52C9V1S | C511897, C541654, C7544576 |
| BZX584C11 | C53892 |
| BZX584C12 | C51315213 |
| BZX584C5V6 | C48667 |
| MM3Z12 | C3015288 |
| MM3Z3V3 | C2978809 |
| MM3Z56 | C51315407 |
| MM3Z5V6 | C2832598 |
| MMSZ5231BS | C51315462 |
