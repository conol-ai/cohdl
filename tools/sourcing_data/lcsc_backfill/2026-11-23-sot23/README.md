# Run for the SOT-23 parts boards 15 and 16 read, 2026-11-23

This directory holds the B2 run repeated over the library with the SOT-23 parts of `transistor` 0.2.2, `mosfet` 0.1.1, `diode` 0.1.4 and `esd` 0.1.5. The fetch was approved for the 71 part numbers preview boards 15 and 16 (`../../jlc_previews/2026-11-23-board-15/`, `-board-16/`) read with their pin 1 at our pad 1: every uncached candidate page of their MPNs, 1,533 pages, listed in an approved URL list (SHA-256 `6ff6bcb9b70a6edbeb6bb7b0ea5b4e955166799ad97593bfa6e4162b976d3c19`). Its `lcsc_map.json` was promoted into `tools/sourcing_data/lcsc_map.json`.

**Command:**

```sh
python3 -I tools/lcsc_backfill.py --allowed-host jlcpcb.com --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir … --date 2026-11-23 --max-requests 1535
```

The tool ran unchanged, behind the wrapper of `../2026-11-21-hongjiacheng-zener/`: an uncached page outside the approved list is not requested, and the wrapper first checked that the uncached candidate pages of the 71 were still exactly the approved list.

**1,534 requests:** `robots.txt` and the approved `partdetail` pages, on 2026-11-23 from 15:05 to 16:09 UTC, under the shared client's rules: an identified user agent (`cohdl-library-bot/0.1`), at least 2 s between requests, `robots.txt` obeyed, no `/api` path. No 403 or 429 response, the budget was not exhausted and the run was not stopped. Every other page came from the fetch cache (14,996 hits). `requests.json` records each one. Raw pages stay in the local cache; none is stored here.

**Result:**
- The run covers 21,471 AVL entries and 21,469 identities, and maps 12,541 of them: the 12,455 of the previous run, each with the same part number, and 86 more.
- 70 of the 71 map to the part number the B1 snapshot gives (table below). MMBT4401 (Zhuhai Hongjiacheng) stays `fetch_failed`: one of its 46 candidate pages, C466634 (another manufacturer's MMBT4401), was on the approved list and was requested, but the request failed (it is the one request `requests.json` does not list), so the tool makes no choice for it. A later run can retry it.
- The other 16 new rows are SOT-23 parts of board 17 whose candidate pages were all in the cache already: 12 Zhuhai Hongjiacheng TVS arrays, Nexperia BAT54C,215 and BAV99,215, and two Hongjiacheng MOSFETs (HL2302, HL3407A) that board 18 re-read with `rotate 90`.
- Identities outside the approved list whose candidate pages were not cached are `fetch_failed` (`PolicyError`), refused before any request, as in the earlier runs.
- `alias_proposals.json` holds the run's proposals; no alias was added.

**Not a binding.** A map row is the part number's identity. Each SOT-23 part binds only with its own preview reading, through a frame row `tools/frame_boards/sot23_frames.py` writes from the board records.

## The 71 part numbers

| Manufacturer | MPN | Part number | Rule |
|---|---|---|---|
| Jiangsu Changjing Electronics Technology | 2N7002 | C8545 | alias |
| Jiangsu Changjing Electronics Technology | MMBT2222A 1P | C8512 | alias |
| Jiangsu Changjing Electronics Technology | MMBT3904(RANGE:100-300) | C20526 | alias |
| Jiangsu Changjing Electronics Technology | MMBT5401(RANGE:200-300) | C8326 | alias |
| Jiangsu Changjing Electronics Technology | MMBT5551(RANGE:200-300) | C2145 | alias |
| Jiangsu Changjing Electronics Technology | S8550(RANGE:120-200) | C105432 | alias |
| Jiangsu Changjing Electronics Technology | S9012 2T1(RANGE:200-350) | C8543 | alias |
| Jiangsu Changjing Electronics Technology | S9013 J3(RANGE:200-350) | C6749 | alias |
| Jiangsu Changjing Electronics Technology | S9015(RANGE:300-400) | C2149 | alias |
| Jiangsu Changjing Electronics Technology | SS8050(RANGE:200-350) | C2150 | alias |
| Jiangsu Changjing Electronics Technology | SS8550 Y2(RANGE:200-350) | C8542 | alias |
| Vishay | SI2301CDS-T1-GE3 | C10487 | alias |
| Zhuhai Hongjiacheng Technology | 2N7002 | C7420321 | alias |
| Zhuhai Hongjiacheng Technology | 2N7002K | C22379475 | alias |
| Zhuhai Hongjiacheng Technology | 2SA812 | C7420323 | alias |
| Zhuhai Hongjiacheng Technology | 2SC1623 | C7420324 | alias |
| Zhuhai Hongjiacheng Technology | 2SC1815 | C20069143 | alias |
| Zhuhai Hongjiacheng Technology | 2SC3356 | C28646266 | alias |
| Zhuhai Hongjiacheng Technology | 2SK3018 | C20069148 | alias |
| Zhuhai Hongjiacheng Technology | BC807-40 | C20069142 | alias |
| Zhuhai Hongjiacheng Technology | BC817-16 | C20069132 | alias |
| Zhuhai Hongjiacheng Technology | BC817-25 | C20069154 | alias |
| Zhuhai Hongjiacheng Technology | BC817-40 | C20069155 | alias |
| Zhuhai Hongjiacheng Technology | BC846B | C20069134 | alias |
| Zhuhai Hongjiacheng Technology | BC847B | C20069135 | alias |
| Zhuhai Hongjiacheng Technology | BC847C | C7420337 | alias |
| Zhuhai Hongjiacheng Technology | BC856B | C20069136 | alias |
| Zhuhai Hongjiacheng Technology | BC857C | C20069137 | alias |
| Zhuhai Hongjiacheng Technology | BSS123 | C7420338 | alias |
| Zhuhai Hongjiacheng Technology | BSS138 | C7420339 | alias |
| Zhuhai Hongjiacheng Technology | BSS84 | C7420340 | alias |
| Zhuhai Hongjiacheng Technology | FDV301N | C20069151 | alias |
| Zhuhai Hongjiacheng Technology | FMMT493 | C28646275 | alias |
| Zhuhai Hongjiacheng Technology | HL2300 | C7420343 | alias |
| Zhuhai Hongjiacheng Technology | HL2301A | C7420344 | alias |
| Zhuhai Hongjiacheng Technology | HL2303 | C7420345 | alias |
| Zhuhai Hongjiacheng Technology | HL2304 | C7420346 | alias |
| Zhuhai Hongjiacheng Technology | HL2305 | C28646261 | alias |
| Zhuhai Hongjiacheng Technology | HL2306 | C28646262 | alias |
| Zhuhai Hongjiacheng Technology | HL2307 | C28646274 | alias |
| Zhuhai Hongjiacheng Technology | HL2309 | C28646263 | alias |
| Zhuhai Hongjiacheng Technology | HL2310A | C7420347 | alias |
| Zhuhai Hongjiacheng Technology | HL2312 | C28646272 | alias |
| Zhuhai Hongjiacheng Technology | HL3400 | C22469448 | alias |
| Zhuhai Hongjiacheng Technology | HL3400A | C7420348 | alias |
| Zhuhai Hongjiacheng Technology | HL3401 | C22469449 | alias |
| Zhuhai Hongjiacheng Technology | HL3401A | C7420349 | alias |
| Zhuhai Hongjiacheng Technology | HL3402 | C28646271 | alias |
| Zhuhai Hongjiacheng Technology | HL3415A | C28646270 | alias |
| Zhuhai Hongjiacheng Technology | HL3416 | C28646273 | alias |
| Zhuhai Hongjiacheng Technology | HL6042 | C28646264 | alias |
| Zhuhai Hongjiacheng Technology | MMBT2222A | C7420351 | alias |
| Zhuhai Hongjiacheng Technology | MMBT2907A | C7420352 | alias |
| Zhuhai Hongjiacheng Technology | MMBT3904 | C7420353 | alias |
| Zhuhai Hongjiacheng Technology | MMBT3906 | C7420354 | alias |
| Zhuhai Hongjiacheng Technology | MMBT4401 | — | fetch_failed |
| Zhuhai Hongjiacheng Technology | MMBT4403 | C20069140 | alias |
| Zhuhai Hongjiacheng Technology | MMBT5401 | C7420356 | alias |
| Zhuhai Hongjiacheng Technology | MMBT5551 | C7420357 | alias |
| Zhuhai Hongjiacheng Technology | MMBTA06 | C20069139 | alias |
| Zhuhai Hongjiacheng Technology | MMBTA42 | C20069138 | alias |
| Zhuhai Hongjiacheng Technology | MMBTA44 | C20069141 | alias |
| Zhuhai Hongjiacheng Technology | MMBTA92 | C20069146 | alias |
| Zhuhai Hongjiacheng Technology | S8050 | C20069125 | alias |
| Zhuhai Hongjiacheng Technology | S8550 | C20069126 | alias |
| Zhuhai Hongjiacheng Technology | S9012 | C20069129 | alias |
| Zhuhai Hongjiacheng Technology | S9013 | C20069127 | alias |
| Zhuhai Hongjiacheng Technology | S9014 | C20069130 | alias |
| Zhuhai Hongjiacheng Technology | S9015 | C20069131 | alias |
| Zhuhai Hongjiacheng Technology | SS8050 | C7420370 | alias |
| Zhuhai Hongjiacheng Technology | SS8550 | C7420371 | alias |
