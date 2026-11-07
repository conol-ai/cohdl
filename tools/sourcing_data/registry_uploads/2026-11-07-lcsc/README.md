# LCSC supplier catalog for the registry, 2026-11-07

Status: **uploaded 2026-11-07 and verified** (see "Upload record" below). This upload replaces the 2026-11-05 catalog (`../2026-11-05-lcsc/`) with the one built after the B2 full run was promoted (#75) and three library spellings were settled (#76).

`report.json` and `report.md` are the pre-upload report from `tools/upload_supplier_snapshot.py build`. The request body itself is not kept, because it rebuilds from the repository alone. `lcsc_map.json` and `mfr_aliases.json` here are byte-for-byte copies of the map and alias table it was built from; their SHA-256 values are in the report. `upload.txt` and `verify.txt` are the tool's output, verbatim. Neither contains a credential.

```sh
R=tools/sourcing_data/registry_uploads/2026-11-07-lcsc
python3 -I tools/upload_supplier_snapshot.py build -o OUT --dates tools/sourcing_data/registry_uploads/2026-11-05-lcsc/report.json \
  --map $R/lcsc_map.json --aliases $R/mfr_aliases.json
```

This gives `snapshot.json` with SHA-256 `3307b6b8d811445eb6070a56c04534b9dc4eea5fdcb4d536e136d95dded84529` and this `report.json`.

## What it holds

The catalog has 13,604 rows (350 basic, 984 preferred, 12,270 extended), and its body is 3,583,901 bytes. The registry's limits are 15,000 rows and 4 MiB (#74), and one request replaces the whole catalog. The rows come from two sources:
- the 1,334 pages of the B1 snapshot (`jlc_catalog/2026-10-30/`);
- the 12,284 rows of `lcsc_map.json`, read from the confirmed pages of `lcsc_backfill/2026-11-06-full-spellings/`.

14 part numbers are in both, merged. All 46 `supplier lcsc` part numbers in `lib/` are in the body.

The 37 aliases go up with the catalog, as `PUT /api/admin/manufacturer-aliases/{alias}`: the three of #62–#64, and the 34 of #73 (rows 1–32, 38 and 39).

## Prerequisites

- #73: Tony Huang's review accepted rows 1–32, 38 and 39 and rejected 33–37 and 40.
- #74 (row cap 15,000), #75 (aliases and promotion; C1338995 `rotate 270` accepted by Tony) and #76 (spellings) merged.
- Migration 0006 applied to production, then #77 merged and deployed (Workers Builds, 2026-11-07 00:13 UTC).
- `passive` 0.2.5, `@st/stm32` 0.3.1, `switches` 0.1.3, `connectors` 0.2.1 and `@contrib/audio-amp` 0.1.1 published (2026-11-06) before this upload. Until it ran, the part numbers new in this catalog read `unknown`.
- Sign-off: Tony Huang asked for this catalog, SHA-256 `3307b6b8…` above, to be uploaded.

## Upload record (2026-11-07)

- Account: Tony Huang's official registry account (`tony@conol.ai`).
- Credential: an `admin`-scope CLI token (#77), minted by Tony on the account page and read by the tool from `~/.cohdl/admin-token` (mode 600). It was sent as `Authorization: Bearer` and never printed or written. The publish token was not used: the admin routes refuse it.
- Body: rebuilt from `main` at `71fd1cc` with `build --dates ../2026-11-05-lcsc/report.json`, giving SHA-256 `3307b6b8…` and this `report.json`.
- Command: `upload OUT --expect-sha256 3307b6b8d811445eb6070a56c04534b9dc4eea5fdcb4d536e136d95dded84529`, exit 0, 00:59:04–00:59:08 UTC.
  - All 37 alias `PUT`s returned 200; `upload.txt` lists each alias with its normalized key.
  - `PUT /api/admin/suppliers/lcsc/snapshot`: 200, `rows` 13604.
- `verify` exits 0. All 46 library part numbers are known, and each equals its snapshot row (`verify.txt`).
- Spot checks against production:
  - C2838500 (`ESP32-C3`, mapped through the `Espressif Systems` alias) and C79401 (`MAX16054AZT+T`, unlocked by #76), both `unknown` before this upload, now resolve.
  - `/manufacturers/normalize?mfr=Espressif%20Systems` gives key `espressif`; `HRS(Hirose)` gives `hirose electric` (#73 row 38).
  - `cohdl suppliers examples/joint-motor-controller` (cohdl 0.10.0) reports its binding `ok` (C1525, through the new Samsung Electro-Mechanics alias) and exits 0.

## Full check (2026-11-07)

`verify.txt` above is the check `main` had at the upload: the library's 46 part numbers. The old catalog holds all 46, so that check alone cannot tell the two catalogs apart (`../2026-11-05-lcsc/README.md`, "Full check"). The full check of #78 was run read-only against production at about 03:30 UTC, from `tools/verify-full-catalog` at `690d421`, on the body rebuilt with this record's own files (`build --dates $R/report.json --map $R/lcsc_map.json --aliases $R/mfr_aliases.json`, SHA-256 `3307b6b8…`, `report.json` byte-identical):

- **This catalog: exit 0.** It asked for all 13,604 rows, 100 per request: 13,604 of 13,604 known and equal, no row missing from the answers or answered twice. The registry's snapshot date, `2026-11-06T01:45:04Z`, is the body's newest `fetched_at`. All 37 aliases were checked; 31 normalize to their canonical key, and 6 differ from their canonical name only in case or spacing, so the public lookup answers the same whether or not the registry holds them. `verify-full.txt` is its output, verbatim.
- **The 2026-11-05 catalog (`dced4283…`): exit 1.** Its 1,442 rows are all still served and equal, because the new catalog keeps them. What refuses it is the date: the registry's `2026-11-06T01:45:04Z` against that body's newest `fetched_at`, `2026-10-30T05:24:02Z`.

The public API cannot list rows, so a row the registry holds beyond the body would go unseen. The registry returning the stored body's SHA-256 and row count would close that.
