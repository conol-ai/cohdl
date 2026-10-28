# Library supplier plan: implementation plan

Status: draft, 2026-10-29. This breaks [`library-supplier-plan.md`](library-supplier-plan.md) (the "library plan") into work items that can be assigned one at a time. It gives their order, deliverables and acceptance checks, and marks the steps that need a person. The figures come from a read-only survey of `feat/rfc-034` on 2026-10-29.

Roles: the **implementer** writes the change; the **reviewer** re-runs the checks independently (section 6) before it merges. "Maintainer" is the library maintainer named in the library plan.

## 1. Where things stand

| Item | State |
|---|---|
| `supplier` bindings in the official library | 0 |
| AVL entries | 21,018 distinct MPNs (`passive` 17,431, the rest 3,587), matching the library plan |
| Footprints | 304 declared; about 285 referenced by parts (the library plan says 300; to be recounted) |
| Part-number data | Only the 1,334 candidate pairs (350 Basic, 984 Preferred) in `tools/sourcing_data/jlcparts/2026-10-27/`, not yet confirmed one by one |
| W1/W2/W3 tools | `jlc_catalog.py`, `lcsc_backfill.py`, `realign_place.py`, `lcsc_map.json` and `jlc_frames.json` do not exist |
| HTTP fetching | No shared client. Existing scripts call `curl` directly, with no User-Agent, rate limit or cache |
| RFC-034 in the compiler | Implemented on the local branch `feat/rfc-034`, including the pcbnew projection tests (round trip and a hand flip in each direction). Not merged or released |
| Bottom-side convention | A test board with top and bottom copies of four parts, bound to LCSC part numbers, is ready for an assembler preview |

### W0 findings that size the work

- `tests/library.rs` has one test function with three tiers:
  - all 62 packages: declarations, documents, E807, and a pin-aware pad-overlap check;
  - `@contrib/` and `@sifli/` only: a rectangular courtyard that contains every pad, no overlap between differently numbered pads, a non-empty MPN and manufacturer, and the QFN quarantine list;
  - a 46-entry allowlist keyed per part entry.
- Extending the strict tier to every package runs into four problems:
  - The allowlist is keyed per entry. `passive` has 17,431 entries and `@st/stm32` 3,303, so per-entry keys do not scale.
  - `lib/misc` has 3 circular courtyards that fail the rectangle-only assertion.
  - The overlap check skips 19 pads at 45°/315° (generated ESP32 footprints, `@contrib/ldo` RT9080).
  - The checks iterate over `pub part`s, so footprint-only packages (`bga`, `csp`, `qfn`, `qfp`, `soic`) are never checked.
- The `fpc.cohdl` "placeholders" all have pads. They are generic lands, not empty footprints:
  - FH12-24S lacks its two hold-down pads.
  - FH12-10S has one row of pads, although its comment says two.
  - CONMHF4 is a single pad.
  - YZP0048 is four round pads.
  
  `lib/connectors/docs/` holds the FH12, CONMHF4, YZP0048 and JST-XH PDFs. No design instantiates these parts. `CONN_MIC_2P` and `CONN_SPK_2P` are the only `mfr: "Generic"` entries in `lib/`, and both use invented MPNs.
- No generator has an entry point for extra AVL clauses:
  - The `gen_passive.py` test copies only `tools/passive_data/`.
  - `gen_stm32.py` validates AVL row keys strictly and rejects an extra key.
  - `gen_esp32.py` emits primaries only.
- E813 checks `(supplier, PN)` uniqueness only within one compile closure. The library test compiles each package separately, so a conflict between two packages in `lib/` goes unseen.
- The registry has `PUT /api/admin/suppliers/{id}/snapshot`:
  - it replaces the whole catalog;
  - one request takes at most 10,000 rows or 4 MiB;
  - it needs an official account's session;
  - no client calls it yet.

## 2. Order

```
Phase A (offline; can start now)
  A1 geometry audit for every package ─┐
  A2 fpc connectors                    ├─→ W0 done
  A3 shared fetch client               │
  A4 library CI scaffold               │
  A5 generator entry points            │
  A6 W3 tools                          ┘
Phase B (network; needs approval)       needs A3
  B1 W1 baseline snapshot
  B2 W2 part-number map
Phase C (needs a person: assembler previews)   needs A6, B1
  C1 bottom-side convention + first alignment board (ready)
  C2 the 14 footprint families with a known orientation difference
  C3 remaining footprint families
Phase D (release)                       needs RFC-034 merged and released, B2, C
  D1 bindings released package by package
  D2 registry: lcsc supplier and catalog upload
Phase E (W4 new parts; long-running)    needs D
```

Two gates sit outside the library work but hold up release:

1. **RFC-034 merged and released.** A package with `supplier` clauses needs a compiler that accepts them.
2. **Assembler previews need a person.** Every W3 alignment board is uploaded and checked by hand. This is the slowest step in the plan; see section 5.

## 3. Work items

Estimates are implementer time plus review, not time spent waiting for a person.

### Phase A: offline

**A1 · Geometry audit for every package (most of W0).** About 3–4 days.

- Audit per footprint instead of per part entry: walk every `footprint` each package declares, including footprint-only packages.
- Make the overlap check handle pads at any angle (separating-axis test for rectangles and rounded rectangles).
- Courtyards: either accept circles or redraw the three in `lib/misc` as rectangles. The implementer compares both and recommends one; the reviewer decides.
- Key the allowlist by `(package, footprint, version)`, each entry with a written reason.
- Fix what the audit exposes. Anything that cannot be fixed goes on the quarantine list with an issue.
- Acceptance:
  - every one of the 62 packages passes the strict tier or is on the quarantine list;
  - each new check has a negative test (a built overlap, a built courtyard overflow) that must fail;
  - `cargo test` passes; `git diff --stat` touches only `tests/` and the fixed packages.
- Three items:
  - A1a: the audit and its negative tests;
  - A1b: `lib/misc`, the rotated pads, and `@contrib/ldo`;
  - A1c: batch fixes for the remaining packages.

**A2 · fpc connectors.** About 1.5 days, after one decision.

- Draw the real lands of FH12-24S and FH12-10S (following the reviewed FH12-26S), CONMHF4 and YZP0048 from the PDFs in the repository.
- Acceptance: every pad within 0.01 mm of the PDF's recommended land, with a pad-by-pad table in the PR description; the A1 audit passes.
- **Decision needed:** quarantine `CONN_MIC_2P`/`CONN_SPK_2P` by removing `pub`, or keep them with `#[source_unspecified("mpn")]` and `mpn: ""`. Recommended: keep them with `source_unspecified`. No design uses them, the change is small, and it states "unknown" honestly.

**A3 · Shared fetch client.** About 1 day; sends no real request.

- Create `tools/sourcing/http.py`, implementing every rule in the library plan's "Ground rules":
  - at most 1 request per 2 s per host, and a request cap per run;
  - a cache keyed by URL, and output as dated snapshots;
  - the fixed User-Agent;
  - back off on 403/429, and stop the run after two in a row;
  - refuse robots-disallowed paths, `/api` included, **before** any request is sent;
  - refuse every EasyEDA host.
- Store only factual fields; raw pages never enter the repository.
- Acceptance: unit tests against a local fake server cover rate limiting, caching, the two-strike stop, robots refusal and EasyEDA refusal. No test touches the network.

**A4 · Library CI scaffold.** About 1.5 days.

- Add the five checks from the library plan's "Library CI" section to `tests/library.rs`. With empty data they pass vacuously.
- Fix the formats of `lcsc_map.json` and `jlc_frames.json` (with `schema_version`) and validate them.
- Add a library-wide uniqueness check: across all of `lib/`, one `(supplier, PN)` names one `(mfr, mpn)`. This closes the per-closure gap in E813.
- Acceptance: each check has a deliberately broken fixture that must fail, and passes on the real library.

**A5 · Generator entry points.** About 2 days.

- `gen_passive.py`, `gen_stm32.py` and `gen_esp32.py` read `lcsc_map.json` and the W3 evidence, and emit `supplier lcsc` clauses only for aligned footprints.
- Widen the strict key validation in `gen_stm32.py`. Make the `gen_passive.py` test copy the map file too.
- With an empty map the output is byte-identical to today's. That is the regression test.
- Acceptance:
  - with an empty map, all three generators' `--check` passes and `git diff` is empty;
  - with one fake mapping, exactly that one clause appears;
  - the generated clauses compile and pass E813.

**A6 · W3 tools.** About 2–3 days.

- `tools/jlc_frame_board.py`: from a list of `(footprint, part number)`, generate one alignment test board. It produces:
  - a CoHDL project;
  - Gerber and drill files, using the outline corner as the origin;
  - the BOM and CPL;
  - a checklist giving each part's pin-1 direction.
  
  Each part number gets one top 0° copy. A bottom copy is added as a control where the side matters.
- `tools/realign_place.py`: read the `(dx, dy, r)` delta table and rewrite users' `place` statements with RFC-034 §2's `compose_place`. Rewriting the repository's designs must leave every copper coordinate of the generated boards unchanged.
- Acceptance:
  - `realign_place.py` applied forward and then inverse to both examples is the identity;
  - the board generator reproduces the bottom-side test board byte for byte.

### Phase B: network (each run approved; supervised)

**B1 · W1 baseline snapshot**

- Confirm the 1,334 candidates on their `partdetail` pages: about 45 minutes at 1 request per 2 s.
- Download and filter the 146 parts sitemap files following the library plan's rules: about 5 minutes and roughly 1 GB.
- Output: `tools/jlc_catalog.py` and the first dated, hashed snapshot, with its source list and the filtered count.
- About 250 Preferred parts are missing from the candidates. The maintainer fills that gap by paging the filter view in a browser, about 32 pages. Whether to also ask the supplier for its official list is an open decision.

**B2 · W2 part-number map**

- Build the index offline from the sitemap and jlcparts, then confirm each candidate on its `partdetail` page. For the whole library that is about 12 hours at 1 request per 2 s, run over several nights.
- First re-measure the match rate on a 200-MPN sample. Commit the sample and the method, then decide whether to run the full set.
- Output: `tools/lcsc_backfill.py`, `tools/sourcing_data/lcsc_map.json`, and a human review list.
- Manufacturer aliases: the tool proposes them; they take effect only after a person reviews them.

### Phase C: alignment evidence (assembler previews)

**C1 · Bottom-side convention and the first alignment board.** The test board is ready. It holds four part types (SOT-23, SOIC-8, 0603, 0805), each at top 0°, top 37°, bottom 0° and bottom 37°. Its compiler CPL matches both `smt_pos.py --map` and KiCad's own position export. A top-side error that repeats identically on the bottom copy is a footprint-frame offset (W3). An error on the bottom copies only is a bottom-side convention error (RFC-034).

**C2 · The 14 footprint families with a known orientation difference.**

- The first full alignment board also re-measures the 33 parts compared on 2026-10-06, whose raw data was not kept.
- Each board carries 30–60 part numbers checked in one preview.
- Expect 3–5 boards, each needing about 15 minutes from a person: upload, screenshots, report back.

**C3 · Remaining footprint families.**

- Use the library plan's shared-evidence rule: group chip parts by the supplier's footprint identifier, and spot-check at least 3 or 5% of each group, whichever is larger.
- Recount the number of boards once B2 has counted the bindings. A rough estimate is 15–40.

Flow:
1. A6 generates a board.
2. A person uploads it and takes screenshots.
3. The reviewer records the measurements and screenshots in `jlc_frames.json`.
4. The implementer applies the rigid transform to each footprint that needs it and bumps the package version.
5. `realign_place.py` rewrites the repository's designs.
6. The A1 and A4 checks pass.

### Phase D: release

- **D1:** Release package by package, and only when the mapping is confirmed, the alignment evidence is complete and CI passes. Order follows the library plan: the 14 families first, then the common parts.
- **D2:** Add the `lcsc` supplier row. Add `tools/upload_supplier_snapshot.py`: it reads a snapshot and sends at most 10,000 rows per request; above that, split the upload or change the endpoint first. It authenticates as an official account. Also upload the alias table. Every upload to the production registry needs explicit sign-off.

### Phase E: W4 new parts

The library plan's four batches:

| Batch | Estimate |
|---|---|
| Chip resistors and capacitors | 2–3 person-weeks |
| Footprint-family packages | 1.5–2 person-weeks |
| Discretes | 5–8 person-weeks |
| Common ICs | 7–11 person-weeks |

Every new part arrives with its datasheet, land, alignment and binding, so phase E moves at the same preview pace as phase C. Split it into items once D1 has run end to end.

## 4. Timeline (rough)

| Week | Implementation | Review / people |
|---|---|---|
| 1 | A1a–c, A2, A3 | Review. Decide on MIC/SPK. Upload the bottom-side test board for a preview |
| 2 | A4, A5, A6 | Review. Approve and run B1 |
| 3 | B2 sample; C2 boards 1–2 | Upload 2 boards. Decide whether to run B2 in full |
| 4–5 | B2 full run (nights); finish C2; D1 first batch | RFC-034 merged and released. D2 upload signed off |
| Later | C3, phase E | 1–3 boards a week |

Phase A takes about two weeks. After that the pace depends on how many previews a person can do and on when RFC-034 is released.

## 5. Risks

| Risk | Response |
|---|---|
| Preview throughput (W3 evidence for up to about 17,000 bindings) | Fill every board; share evidence by supplier footprint identifier, with spot checks; common parts first |
| Supplier pages change or rate-limit | A3 stops after two consecutive 403/429; dated snapshots let a run resume; never change IP or use a proxy |
| Moving a footprint silently moves users' copper | Breaking version bump, `realign_place.py`, and a "copper does not move" regression test |
| Tests pass because they share the implementation's mistake | Every item names an independent reference: the PDF's recommended land, KiCad's own export, a negative fixture |
| The allowlist turns into a blanket pass | Every entry keyed by `(package, footprint, version)` with a reason; the reviewer reads each one |

## 6. Review checklist (every item)

- Re-run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`, and `python3 -I -m unittest` for Python tools.
- Generator items: `--check` shows no diff, or exactly the diff the item predicts.
- Footprint items: compare each pad with the PDF; open a generated board in KiCad and run DRC; `smt_pos.py --map` passes.
- Fetch items: read the request log and confirm the rate limit, User-Agent and request cap, and that no `/api` or EasyEDA URL appears.
- Judge scope from `git diff --stat`, not from the item's own report.

## 7. Open decisions

1. `CONN_MIC_2P`/`CONN_SPK_2P`: quarantine, or `source_unspecified` (recommended).
2. Whether to run B1/B2, and at what hours (nights recommended).
3. Whether to ask the supplier for the official Basic/Preferred list or Components API access.
4. When RFC-034 is merged and released. Phase D cannot start before that.
5. How many preview boards a week someone can check (about 15 minutes each).
