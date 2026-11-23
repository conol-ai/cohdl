# Second run for the SOT-23 parts, 2026-11-24

This directory holds the B2 run repeated over the library for the 39 SOT-23 part numbers that preview boards 15 to 19 (`../../jlc_previews/2026-11-23-board-*`) read at our pad 1 and that `../2026-11-23-sot23/` did not map: 19 diodes and the TVS array PSM712-LF-T7 of board 17, MMBTA56 of board 18, the 18 Zhuhai Hongjiacheng diodes of board 19, and MMBT4401, retried. The fetch was approved for every uncached candidate page of their MPNs, 917 pages, listed in an approved URL list (SHA-256 `417853e289e8a03fb3d4a0540be1e62b002c7ccae773c55e2478115a4cf02177`). Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:** as `../2026-11-23-sot23/`, with `--date 2026-11-24 --max-requests 919`, behind the same wrapper (no uncached page outside the approved list is requested; the uncached candidate pages were checked to be exactly that list first).

**917 requests:** the approved `partdetail` pages (`robots.txt` came from the cache), from 17:36 to 18:10 UTC on 2026-11-23, under the shared client's rules: an identified user agent (`cohdl-library-bot/0.1`), at least 2 s between requests, `robots.txt` obeyed, no `/api` path. No 403 or 429 response, the budget was not exhausted and the run was not stopped. Every other page came from the fetch cache (16,529 hits). `requests.json` records each one. Raw pages stay in the local cache; none is stored here.

**Result:**
- The run maps 12,580 identities: the 12,541 of the previous run, each with the same part number, and the 39, each to the part number the B1 snapshot gives (table below).
- Identities outside the approved list whose candidate pages were not cached stay `fetch_failed` (`PolicyError`), refused before any request.
- `alias_proposals.json` holds the run's proposals; no alias was added.

**Not a binding.** A map row is the part number's identity. Each SOT-23 part binds only with its own preview reading, through a frame row `tools/frame_boards/sot23_frames.py` writes from the board records.

## The 39 part numbers

| Manufacturer | MPN | Part number | Rule |
|---|---|---|---|
| Jiangsu Changjing Electronics Technology | BAV70 | C68978 | alias |
| ProTek | PSM712-LF-T7 | C32677 | alias |
| Zhuhai Hongjiacheng Technology | 1SS181 | C7502730 | alias |
| Zhuhai Hongjiacheng Technology | BAS21 | C7502725 | alias |
| Zhuhai Hongjiacheng Technology | BAS21S | C7502726 | alias |
| Zhuhai Hongjiacheng Technology | BAS40 | C7502685 | alias |
| Zhuhai Hongjiacheng Technology | BAS40-04 | C7502688 | alias |
| Zhuhai Hongjiacheng Technology | BAS40-05 | C7502689 | alias |
| Zhuhai Hongjiacheng Technology | BAS70 | C7502686 | alias |
| Zhuhai Hongjiacheng Technology | BAS70-04 | C7502687 | alias |
| Zhuhai Hongjiacheng Technology | BAS70-06 | C22466364 | alias |
| Zhuhai Hongjiacheng Technology | BAT54 | C22466368 | alias |
| Zhuhai Hongjiacheng Technology | BAT54A | C7502690 | alias |
| Zhuhai Hongjiacheng Technology | BAT54C | C22466350 | alias |
| Zhuhai Hongjiacheng Technology | BAT54S | C7420333 | alias |
| Zhuhai Hongjiacheng Technology | BAT720 | C22466371 | alias |
| Zhuhai Hongjiacheng Technology | BAV70 | C7420335 | alias |
| Zhuhai Hongjiacheng Technology | BAV99 | C7420336 | alias |
| Zhuhai Hongjiacheng Technology | BAW56 | C7502729 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C10 | C19077470 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C12 | C19077471 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C15 | C19077472 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C16 | C22379471 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C18 | C19077473 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C30 | C22379472 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C33 | C22379474 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C3V3 | C19077467 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C4V7 | C22395521 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C5V1 | C19077468 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C5V6 | C19077469 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C6V8 | C22379473 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C7V5 | C22379470 | alias |
| Zhuhai Hongjiacheng Technology | BZX84C8V2 | C22395522 | alias |
| Zhuhai Hongjiacheng Technology | MMBD4148 | C7502723 | alias |
| Zhuhai Hongjiacheng Technology | MMBD4148CC | C17179589 | alias |
| Zhuhai Hongjiacheng Technology | MMBD4148SE | C17179590 | alias |
| Zhuhai Hongjiacheng Technology | MMBT4401 | C7420355 | alias |
| Zhuhai Hongjiacheng Technology | MMBTA56 | C20069147 | alias |
| Zhuhai Hongjiacheng Technology | MMBZ5226B | C22395543 | alias |
