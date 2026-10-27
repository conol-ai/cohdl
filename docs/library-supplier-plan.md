# Official library supplier program (LCSC / JLCPCB)

Status: plan, 2026-10-23; revised 2026-10-27 against main `329af05`. This is library work, not language design: it uses the supplier binding defined in [RFC-034](design/rfc-034-supplier-binding.md) and the authoring rules in [`LIBRARY.md`](../LIBRARY.md). Where it adds a rule for official libraries, that rule moves into `LIBRARY.md` when the corresponding workstream lands.

## Goal

The official library is built around JLCPCB assembly and LCSC sourcing. When this program is done:

1. **Bound.** Every official part that LCSC stocks carries `supplier lcsc "<C-number>"` on the AVL entry that orders it.
2. **Aligned.** Every official footprint sits in JLCPCB's assembly frame, so every `lcsc` mapping in the official library is `(0, 0, 0°)`, and the clause needs no `at`/`rotate`. The only non-zero mappings are listed exceptions (§W3).
3. **Covered.** The library covers JLCPCB's Basic and Preferred catalog (1586 parts on 2026-10-06; 20 covered then), with Extended parts added on demand.
4. **Uploadable.** A design built from official parts produces an LCSC BOM and, through `tools/smt_pos.py --map`, a CPL that JLCPCB accepts with no manual matching or rotation fixes.

Pad numbers, land geometry and pin maps stay authored from manufacturer documents. Alignment moves a whole footprint; it never edits a land.

## Ground rules

- **No data fetched from EasyEDA by any tool.** (Unrelated to the compiler's `--emit easyeda` netlist emitter, which writes a file for EasyEDA/LCEDA Pro import and fetches nothing.) EasyEDA's Terms (effective 2026-09-21) restrict bulk or systematic automated access and limit derivative use of its content, subject to the licences that apply to individual content (open-source content keeps its own licence). This project's rule is stricter than the Terms: no tool fetches from EasyEDA at all. Its footprints are not an authority: land sizes differ from manufacturer lands by 0.1–0.35 mm, and SK6812MINI-E is drawn for normal mount. A person may look at a supplier's footprint by hand to settle a question; nothing is copied.
- **No request to a path a site's robots.txt disallows.** For JLCPCB, that is `/api`.
- **Fetch policy for every tool, through one shared HTTP client:**
  - At most 1 request per 2 s per host, and a per-run request cap.
  - URL-keyed cache; outputs are dated snapshots.
  - User-Agent `cohdl-library-bot/<v> (+https://github.com/conol-ai/cohdl)`.
  - On 403/429, back off; after two in a row, stop the run. No IP rotation, no proxies.
  - Keep factual fields only (C-number, MPN, brand, package name, class, stock, datasheet URL, product-page path). Never redistribute raw pages.
- **Fail closed.** A tool that cannot decide (ambiguous match, conflicting manufacturer, missing evidence) writes the case to a review list and changes nothing.

## Dependencies

| Needs | For | Status |
|---|---|---|
| Geometry checks in `tests/library.rs` for every shipped package (today only `@contrib/` and `@sifli/`) | Any footprint change in W3/W4 | W0 |
| RFC-034 phase 1: compiler clause, supplier outputs, registry tables, `cohdl suppliers` | Publishing bindings (W2/W4); registry display (W5) | Draft RFC |
| Nothing | Catalog snapshot (W1), frame evidence and alignment (W3) | Can start now |

## Workstreams

### W0 — Prerequisites

- Extend the `tests/library.rs` geometry checks (pad overlap, courtyard escape, placeholder footprints) to every shipped package, and fix what they expose.
- ~~Fix [#47](https://github.com/conol-ai/cohdl/issues/47) (DRV8231A pads overlap the GND pad) and [#48](https://github.com/conol-ai/cohdl/issues/48) (S8050 SOT-23 mirrored).~~ Done: both closed 2026-10-15 by PR #50 (`5e4dc44`), which also fixed [#51](https://github.com/conol-ai/cohdl/issues/51) (`FP_SPK_2P` pads overlap).
- `lib/connectors/src/fpc/fpc.cohdl` has placeholder lands. Proposed treatment (awaiting confirmation):
  - Draw real lands for FH12-24S, FH12-10S, CONMHF4 and YZP0048 from the PDFs already in `lib/connectors/docs/`. FH12 can follow the audited FH12-26S.
  - `CONN_MIC_2P` (`MEMS MIC 2P`) and `CONN_SPK_2P` (`SPK 2P 1.25mm`) carry made-up MPNs under `mfr: "Generic"`. Two options:
    - quarantine them by removing their `pub part`; or
    - keep them, with `mpn: ""` under the new `#[source_unspecified("mpn")]` marker (`f1f7acc`). That states "unknown" instead of a fake MPN. They then never get a supplier binding from W2, because there is no MPN to match.

### W1 — Catalog snapshot

`tools/jlc_catalog.py` maintains a dated, hashed snapshot of the Basic/Preferred catalog. Each row holds: C-number, MPN, brand, package, class, stock, datasheet URL, and the full `partdetail` path taken from the sitemap. Tools never rely on the short `partdetail/C<n>` form. An earlier draft cited it as showing a wrong part; that did not reproduce on 2026-10-27, so the rule rests on using the canonical path, not on that example.

- **Baseline:** the 2026-10-06 capture counted 351 Basic and 1235 Preferred parts, but the raw capture was kept only in temporary session storage and was not preserved. The first committed snapshot is therefore regenerated: candidates from the sources below, each confirmed on its `partdetail` page. It is committed with its source list, date and hash.
- **Quarterly refresh (automated):** re-read the `partdetail` page of every C-number in the snapshot. These pages are robots-allowed and server-render the class. Write promotions, demotions and stock changes to a dated diff. The request budget is about 53 minutes for 1586 parts (one request per 2 s). Actual run time, including transfer and retries, is logged with each snapshot.
- **Quarterly discovery (human-assisted):** new Basic/Preferred parts cannot be enumerated from server-rendered pages; list pages load everything past their first page through `/api`.
  - A maintainer pages through the Basic and Preferred filters on jlcpcb.com in their own browser (about 32 pages) and exports the visible C-numbers with a page-side copy helper in `tools/`.
  - The refresh verifies each new C-number on its `partdetail` page before admitting it.
- **Official list:** ask JLCPCB (account manager or support@jlcpcb.com) for the Basic/Preferred list. If it arrives, or Components API access is granted, it replaces discovery.
- **jlcparts as a candidate source (re-evaluated 2026-10-27).** Its current snapshot (created 2026-10-27; `data/manifest.json`) holds 354,819 parts, 235,266 of them below C6374508, so the 2026-10-06 finding of "nothing below C6374508" no longer holds. Each record carries the class (`Basic`, `Preferred` or `Extended`). A full count over all 1089 component shards on 2026-10-27 found **350 Basic and 984 Preferred**. The site counted 351 and 1235 on 2026-10-06. So jlcparts covers Basic almost completely but is short of Preferred. Why is not established: the dates differ, and it may hold only some parts. It is therefore a **candidate source** for the regenerated baseline and for W2, never the catalog itself. Each candidate is confirmed on its `partdetail` page, and human-assisted discovery covers the gap.
- **Preserved candidate seed:** the 1,334 C-number/MPN pairs (350 Basic, 984 Preferred) are archived in [`basic_preferred.json`](../tools/sourcing_data/jlcparts/2026-10-27/basic_preferred.json), with the original [run log](../tools/sourcing_data/jlcparts/2026-10-27/run.log) and [provenance, counts and SHA-256 hashes](../tools/sourcing_data/jlcparts/2026-10-27/provenance.json). This is input for rebuilding W1, pending individual `partdetail` confirmation, not the verified catalog baseline.
- **Full `partdetail` paths:** JLCPCB's parts sitemaps are declared in its robots.txt as `https://jlcpcb.com/sitemap-index.xml`: 146 files of at most 50,000 URLs, so at most about 7.3 million URLs. They are not a parts count. Spot checks on 2026-10-27 (files 1 and 146) found:
  - non-detail links: 2,547 category pages in file 1;
  - a placeholder `…/C0/C0`;
  - 11,210 of file 146's 38,993 detail links using 10-digit `C99…` numbers, often with an empty slug prefix. Whether these are LCSC part numbers is not established.

  The ingest therefore keeps only `partdetail/<slug>/<C>/<C>` links whose two C-numbers are equal and non-zero, treats 10-digit numbers as a separate population until verified, de-duplicates by C-number, and reports the filtered total. A C-number is resolvable through the sitemap only if it appears in that filtered set. Identity (part number, MPN, manufacturer) is always confirmed on the `partdetail` page itself. The sitemaps carry no class, so they do not replace discovery.
- **Not usable as sources (checked 2026-10-22):**
  - jlcsearch: ignores filters and paging.
  - ole00 `basic_parts.csv`: from 2023, GPL-2.0.
  - jlcparts derivatives (CDFER, kicad-jlcpcb-tools) were not re-evaluated after jlcparts' format change and are not used.

### W2 — Backfill existing parts

`tools/lcsc_backfill.py` finds the LCSC C-number for every official AVL entry.

- **Scale (recounted at `329af05`):** 21,018 distinct MPNs (17,431 in `lib/passive`, 3,587 elsewhere).
- **Entry point (checked 2026-10-27).** A per-MPN supplier search is not usable under the ground rules:
  - JLCPCB search goes through `/api`.
  - LCSC's `/search?q=` is robots-allowed but rendered client-side; the server HTML holds no results.
  - LCSC's sitemaps list only C-numbers (`product-detail/C….html`), about 7.9 million of them.

  So the backfill works from JLCPCB's parts sitemaps instead:
  1. **Index.** Download the 146 sitemap files once per run (about 1 GB; a request budget of about 5 minutes at one request per 2 s, with actual time logged), apply the W1 filters, and add jlcparts records as a second candidate source. Build an offline index from normalized MPN to (C-number, full `partdetail` path). The slug's MPN is lossy: punctuation becomes `_` or is dropped (for example `MP1482DS-LF-Z` → `MP1482DS_LFZ`), and the prefix is sometimes a manufacturer and sometimes a number. So both sides are compared as upper-case alphanumerics only, and every hit is only a **candidate**.
  2. **Confirm.** Fetch each candidate's `partdetail` page (robots-allowed, server-rendered) to read the exact MPN, manufacturer and class. Where one MPN has several candidates, each is fetched. The request budget is therefore at least one page per matched MPN, about 12 hours at one request per 2 s for the whole library; the actual time is logged.
  3. **Re-measure.** The 82% exact-match rate from the 2026-10-06 186-MPN sample was measured with a different lookup, and its raw sample was not preserved. Re-measure it on the new index, with the sample and method committed, before committing the estimate.
- **Match:** exact MPN after normalizing case and whitespace, and manufacturer equality through a reviewed alias table. LCSC brand strings carry local names, for example `ST(…)` → `STMicroelectronics`.
- **Choice among several C-numbers:** Basic > Preferred > Extended, then in stock, then higher stock. A remaining tie goes to review.
- **Output:** `tools/sourcing_data/lcsc_map.json`, committed and dated. Each row holds MPN, manufacturer, C-number, rule applied and snapshot hash.
  - Generators (`gen_passive.py`, `gen_stm32.py`, `gen_esp32.py`) read it.
  - Hand-written packages are rewritten by a script, then `cohdl fmt`.
- **A binding ships only with a verified mapping.** An entry gets its `supplier lcsc` clause in the same release as that binding's frame evidence (W3). Until then the mapping is unknown, and writing `(0, 0, 0°)` would be an unverified claim. So W2 produces the map file early, and bindings land package by package as W3 covers their footprints.

### W3 — Align footprints to JLCPCB's frame

**Target:** for every official footprint F and every LCSC part bound to it, JLCPCB's footprint for that part sits at `(0, 0, 0°)` relative to F (RFC-034 §2).

**Unit of evidence.** RFC-034 defines a mapping for the pair (our footprint, the supplier's footprint for one part number). So proving one reference part number on F says nothing by itself about another part number bound to F. Evidence, exceptions and CI are therefore keyed by **(footprint, package version, supplier, part number)**.

- **Scope (recounted at `329af05`):** 300 distinct footprints (285 outside `lib/passive`, 15 in it). Most belong to standard families: CHIP, SOT, SOD, SOIC/TSSOP, QFP, QFN, BGA.
- **Evidence:** `tools/sourcing_data/jlc_frames.json`, machine-readable for CI, with a generated `docs/jlc-frames.md` view. One row per (footprint, package version, supplier, part number), holding:
  - the recorded mapping (target `(0, 0, 0°)`);
  - the mapping found before alignment;
  - the evidence: method, date and artifact (preview screenshot or test-board record);
  - **or** a reference to another row's evidence, which is allowed only with a recorded reason. Acceptable reasons, in order:
    1. the supplier shows the same footprint identifier for both part numbers (whether its part pages expose one is to be checked during W1);
    2. both part numbers were placed on the same frame test board and checked in the same preview.

  Without such a reason, a part number needs its own evidence. A cluster that shares evidence also needs a sample from it checked directly: at least 3 part numbers or 5%, whichever is larger.
- **Method:** a frame test board per batch.
  - One instance per part number at 0°, with its BOM and CPL uploaded to JLCPCB's assembly flow. The preview can show many parts at once without placing an order.
  - The placement-preview screenshot is archived with the batch.
  - Manual inspection of the JLCPCB part page is the fallback for a single part.
- **Change:** a rigid rotation and translation of the whole footprint: pads, courtyard and silkscreen together. Pad numbers and land dimensions never change.
- **Pin 1 and polarity:** after alignment, pin 1 or the polarity mark must sit where JLCPCB's preview expects it. If it does not, check the manufacturer document: either we are wrong (fix it, as in #48) or the part is an exception.
- **Exceptions:** part numbers whose supplier footprint legitimately differs from ours keep a non-zero mapping. An exception is recorded per (footprint, version, supplier, part number) with its reason, because two part numbers on one footprint can differ. Known case: SK6812MINI-E, a reverse-mount part validated in production.
- **Versioning:** every package with a moved footprint gets a new version, with no compatibility layer.
  - **Why it is breaking.** Moving a footprint's frame silently moves the copper of every consumer that places it with `place … at … rotate …`. No diagnostic fires, because the design still checks.
  - **Version bump.** So the bump is breaking: a major version, or a minor version while the package is `0.x`. Never a patch.
  - Old versions stay immutable on the registry, and existing locks keep resolving them.
  - Release notes list each moved footprint's delta as an RFC-034 §2 triple `(dx, dy, r)`, old frame to new.
  - **`tools/realign_place.py`.** Takes that delta table and rewrites a consumer's `place` statements, so the copper stays where it was. The new placement is the old one composed with the delta through the same `compose_place` rule as §2. Designs that do not `place` the moved footprints are untouched.
  - In-repo designs are updated in the same change with that tool, and their emitted boards are diffed to show the copper did not move.
- **Order:** start with the families that the 14-of-33 measurement of 2026-10-06 showed rotated. That comparison's raw data was not preserved, so the first frame test board re-measures those 33 parts and commits the result. Then bound and frequently used parts, then the rest.

### W4 — Catalog intake

Priority follows the snapshot: Basic (351) → Preferred (1235) → Extended on demand (registry `component_requests`, then search misses once they are logged). Each new part follows `LIBRARY.md`:

1. **Datasheet:** from the manufacturer's site. If the site blocks retrieval, use the manufacturer-authored document served by LCSC, recording both URLs and the SHA-256.
2. **Device and land pattern** from that document.
3. **Footprint frame:** aligned to JLCPCB (W3) from the start.
4. **Binding:** `supplier lcsc "<C-number>"` with the default `(0, 0, 0°)`, in the same change.

| Batch | Content | Approach | Estimate |
|---|---|---|---|
| Passives | 293 UNI-ROYAL resistors; MLCCs (68 Samsung, 60 Fenghua, 7 Yageo); ferrite beads, inductors, arrays, tantalum | `alt` entries on existing `passive` parts where parameters are equivalent, new parts where values are missing; generator decoders for UNI-ROYAL and Fenghua | 2–3 person-weeks |
| Package families | New `lib/sot`, `lib/sod`, `lib/do214` (about 12 lands) for new parts. Existing lands stay where they are: CoHDL has no re-export (`pub use` is rejected, RFC-016 first pass), so moving a footprint would change its fully-qualified path and break every `use` of it. Consolidation waits for re-exports, or for a breaking release with a path-rewrite list. | Hand-audited, JLCPCB-aligned | 1.5–2 person-weeks |
| Discretes | 1046 parts: 984 Hongjiacheng (about 130 series, 10 packages cover 93%) and about 60 others | Series generator with an explicit pin map per series (SOT-23 BEC/EBC, MOSFET GSD, diode cathode) | 5–8 person-weeks |
| ICs | 71 parts (AMS1117, LM358, 74HC595, SP3485, TP4056, CP2102, …) | One at a time, reusing `lib/soic` / `lib/sot` | 7–11 person-weeks |

### W5 — Registry data

- **Supplier table:** an `lcsc` row with display name `LCSC`, URL template `https://www.lcsc.com/product-detail/{pn}.html`, and class vocabulary `basic`, `preferred`, `extended` in that order.
- **Catalog uploads:** each W1 snapshot, plus every official-library C-number, uploaded into `supplier_catalog`. Each row carries the JLCPCB `partdetail` link as an extra link.
- **Manufacturer alias table:** shared with W2.

## Library CI (official packages)

Added to `tests/library.rs` as each workstream lands:

- Every `supplier lcsc` part number appears in `lcsc_map.json` with the same MPN and manufacturer.
- Every `supplier lcsc` binding has a row in `jlc_frames.json` for its (footprint, package version, supplier, part number).
- That row's mapping equals the binding's `at`/`rotate`. A non-zero mapping must be a listed exception for that exact key.
- Every row has its own evidence, or a reference to another row's evidence with an accepted reason.
- A moved footprint (new package version) invalidates its rows until they are re-verified or re-referenced with a reason.

## Milestones

| Order | Milestone | Depends on | Estimate |
|---|---|---|---|
| 1 | W0 done; W1 baseline committed | — | 1.5–2.5 person-weeks |
| 2 | W3 evidence for the 14 known-rotated families plus the first test board | 1 | 1 person-week |
| 3 | RFC-034 phase 1 merged | RFC review | (language work, tracked with the RFC) |
| 4 | W2 map file; first bindings published with aligned footprints | 2, 3 | 1–2 person-weeks, plus fetch time |
| 5 | W3 remaining families; packages republished | 2 | 2–4 person-weeks of alignment (per footprint), plus 1–4 person-weeks of evidence (per binding; see below) |
| 6 | W4 passives and package families | 3, 5 | 3.5–5 person-weeks |
| 7 | W4 discretes and ICs | 6 | 12–19 person-weeks |

**W3 evidence estimate.** Evidence scales with bindings, not footprints. The upper bound has every part number checked on its own: about 30 s of preview checking and recording each, for about 17,000 bindings, is about 3.5 person-weeks. The lower bound assumes most passives share evidence through a supplier footprint identifier, plus the required samples. Recompute this once W2 has counted bindings and clusters.

**Total:** about 22–37.5 person-weeks of library work, on top of RFC-034's implementation. Owner: the library maintainer assigned on 2026-10-22.

## Progress measures

- Share of the JLCPCB Basic/Preferred snapshot covered by official parts. The 2026-10-06 figure was 20 / 1586; it is re-measured against the regenerated W1 snapshot.
- Share of official AVL entries with a `supplier lcsc` binding.
- Share of official footprints with frame evidence; the number of listed non-zero exceptions.
- A reference design's LCSC BOM and CPL accepted by JLCPCB with no manual fixes.

## Decisions so far

| Date | Decision |
|---|---|
| 2026-10-22 | The library maintainer owns this program. |
| 2026-10-23 | The language feature is supplier-neutral (RFC-034). Aligning to JLCPCB's frame is library work: target `(0, 0, 0°)`. |
| 2026-10-23 | Tools never request JLCPCB `/api`. The catalog is a committed baseline, plus an automated `partdetail` refresh and human-assisted discovery. |
| 2026-10-23 | The LCSC BOM header is `LCSC Part #`. The value is the C-number, the same identifier JLCPCB calls "JLCPCB Part #". |
| 2026-10-27 | #47, #48 and #51 are fixed (PR #50); removed from W0. |
| 2026-10-27 | The W2 entry point is JLCPCB's parts sitemaps plus `partdetail` confirmation. Per-MPN supplier search is not usable under the ground rules. |
| 2026-10-27 | A moved footprint is a breaking version bump, shipped with a `(dx, dy, r)` delta table and `tools/realign_place.py`. |
| 2026-10-27 | Existing scattered lands are not migrated until CoHDL has re-exports. |
| 2026-10-27 | Frame evidence, exceptions and CI checks are keyed by (footprint, package version, supplier, part number). Shared evidence needs a recorded reason and a direct sample. |
| 2026-10-27 | jlcparts is re-admitted as a candidate source. Sitemap ingest is filtered and de-duplicated. Identity is always confirmed on `partdetail`. |
| 2026-10-27 | The 2026-10-06 raw captures (baseline, 82% sample, 14-of-33 comparison) were not preserved. Each figure is re-measured, and its data is committed. |
| pending | Connector placeholders: fix four; quarantine two, or keep them with `#[source_unspecified("mpn")]` (W0). |
