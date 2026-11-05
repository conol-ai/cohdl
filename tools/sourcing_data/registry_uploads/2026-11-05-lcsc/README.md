# LCSC supplier catalog for the registry (plan D2), 2026-11-05

Status: **uploaded 2026-11-05 and verified** (see "Upload record" below). Alex signed off on the snapshot SHA-256 below, and Tony Huang accepted the three aliases in #62, #63 and #64 (evidence in `tools/sourcing_data/alias_reviews/2026-11-05-mfr-aliases.md`).

`report.json` and `report.md` are the pre-upload report from `tools/upload_supplier_snapshot.py build`. The request body itself is not kept, because it rebuilds from the repository alone:

```sh
python3 -I tools/upload_supplier_snapshot.py build -o OUT --dates tools/sourcing_data/registry_uploads/2026-11-05-lcsc/report.json
```

The rebuild gives the same `snapshot.json`, SHA-256 `dced42839f77895ec7324e8c2207218b17352b51426ec213083fd40439066982`, and the same report. `tools/tests/test_upload_supplier_snapshot.py` checks both.

## What it holds

The catalog has 1,442 rows (350 basic, 984 preferred, 108 extended) and its body is 384,544 bytes. The registry's limits are 10,000 rows and 4 MiB, and one request replaces the whole catalog. The rows come from two sources, which share no part number:
- the 1,334 pages of the B1 snapshot (`jlc_catalog/2026-10-30/`);
- the 108 rows of `lcsc_map.json`, read from the confirmed pages of `lcsc_backfill/2026-10-30-sample200-aliases/`.

All six `supplier lcsc` part numbers in `lib/` are in it. Each row carries the JLCPCB part page as its one extra link, and its `fetched_at` is when that page was fetched. Five B1 pages were read from the fetch cache during the run, so its request log has no entry for them. Their times come from the cache's metadata, after checking each cached page's URL and body hash, and the report lists them.

The three aliases go up with the catalog, as `PUT /api/admin/manufacturer-aliases/{alias}`.

## Checks before upload (2026-11-05)

- The registry's own `parseSnapshot` (`registry/src/worker/suppliers.ts`, bundled with esbuild) accepts the body: 1,442 rows, with the classes in order.
- On a throwaway local registry (`wrangler dev --local` with its own D1 and KV under a scratch directory, and a local official session), the upload works end to end:
  - a wrong `--expect-sha256` is refused before anything is sent;
  - the three aliases and the 1,442 rows are accepted;
  - `verify` finds all six part numbers known, each equal to its snapshot row;
  - `cohdl suppliers` on the `lcsc-batch-1` board reports `ok` for all six bindings, with no `unknown` or `mismatch`, and exits 0.
- Against the production registry, the same audit reports today that the registry has no catalog for `lcsc`.

## Upload (after sign-off)

The admin endpoints take an official account's web session. The person runs the upload with their own session (the `__Host-session` cookie value), which the tool sends but never prints or writes:

```sh
python3 -I tools/upload_supplier_snapshot.py build -o OUT --dates tools/sourcing_data/registry_uploads/2026-11-05-lcsc/report.json
COHDL_ADMIN_SESSION=… python3 -I tools/upload_supplier_snapshot.py upload OUT \
  --expect-sha256 dced42839f77895ec7324e8c2207218b17352b51426ec213083fd40439066982
python3 -I tools/upload_supplier_snapshot.py verify OUT
```

Then record here the time, the account, and the outputs of `upload` and `verify`.

## Upload record (2026-11-05)

- Account: Alex's registry account, official since 2026-11-05 (`/api/me` reports `official: true`).
- Prerequisites: #62, #63 and #64 accepted and closed (03:53–03:54 UTC). Alex signed off on the snapshot SHA-256.
- Body: rebuilt at `90974dd` with `build --dates`, which gave SHA-256 `dced4283…` and a `report.json` identical to this one.
- Method: the same requests `upload` makes, in the same order, sent as same-origin `fetch` calls from Alex's signed-in registry tab. The browser attached its own session, so the session never left the browser. The page recomputed the body's SHA-256 before sending and would have refused a different one.
- Results, about 05:39 UTC:
  - `PUT /api/admin/manufacturer-aliases/GigaDevice%20Semicon%20Beijing`: 200, `alias_key` `gigadevice semicon beijing`.
  - `PUT /api/admin/manufacturer-aliases/Vishay%20Intertech`: 200, `vishay intertech`.
  - `PUT /api/admin/manufacturer-aliases/YAGEO`: 200, `yageo`.
  - `PUT /api/admin/suppliers/lcsc/snapshot`: 200, `rows` 1442.
- `verify` (05:40 UTC) exits 0. C117815, C129276, C529436, C717701, C730103 and C1857444 are all known, and each equals its snapshot row. The snapshot date is `2026-10-30T05:24:02Z`.
- `cohdl suppliers --supplier lcsc` on the `lcsc-batch-1` board, against the production registry, reports `ok` for all six bindings, with no `unknown` or `mismatch`, and exits 0.
