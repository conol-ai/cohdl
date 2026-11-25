---
name: lcsc-library
description: Add a batch of parts to the official CoHDL library with LCSC/JLCPCB supplier bindings, end to end - choose a part family, transcribe the part-number rule, generate the library source, backfill LCSC identities under the fetch policy, gather orientation evidence (the jlc-preview skill for polar parts), write frame rows and bindings, test, relock and open a draft PR. Use when asked to add missing parts or a part family to lib/, to bind parts to LCSC, to run a backfill, or to work the library supplier plan (docs/library-supplier-plan.md). Also the step list a scheduled library run follows.
---

# Library batch with LCSC bindings

One batch is one part family: for example the Hongjiacheng SOD-323/SOD-523 Zeners (#138) or the SOT-23 transistors (#137). The batch ends as one draft PR. It never merges, publishes or uploads to the registry: those are the maintainer's (Tony's).

The rules live in `docs/library-supplier-plan.md` (ground rules, W2–W4), `docs/library-supplier-impl-plan.md` (phases B–E, review checklist) and `docs/library-supplier-w3-sampling.md` (sampled inference for chip R/C). This skill is the order of work and the gates between steps. Where it and those documents differ, the documents win; say so in the PR.

## Modes

- **Interactive** (a person is present): every step may run. Ask at each gate marked **Ask**.
- **Unattended** (a scheduled run): run only steps that need no one, and the gates a person approved in advance and in writing (name the approval in the run's report). At any other gate, stop: write what is ready and what waits on whom into the run's report and the plan doc's checklist, and end. Never drive the browser unattended: the preview may need a sign-in or a captcha.

## Ground rules

- **Never** merge, publish, upload to the registry, touch the production database, add to the assembler's cart, quote, order or pay.
- **Credentials:** never read, record or type them. Never capture the screen.
- **Fetching** goes through `tools/sourcing/http.py` only: its user agent, at least 2 s between requests to a host, `robots.txt` obeyed, no `/api` path, no EasyEDA host, stop after two 403/429 in a row, no IP change, no proxy. Every run that sends requests needs a person's approval of its exact page list (step 4). Raw pages and PDFs are never committed.
- **Downloaded data** goes in its own empty directory; Python that reads it runs with `-I`, and scripts you write live elsewhere.
- **Fail closed:** a case a tool cannot decide (several candidates, an unreadable page, a conflicting manufacturer) goes to review. Nothing is guessed.
- **Aliases** (`tools/sourcing_data/mfr_aliases.json`) change only after a person's review that cites an issue. A run may propose one; it never adds one.
- **Private designs** never enter git or a PR, not even as a test fixture.
- **Git:** a new branch per batch, in its own worktree; never switch branches in a shared checkout. Merge commits only; no force-push; never a bare `git stash`. Push and open the PR only when the person asks (interactive) or the schedule's approval covers it (unattended). Commits end with the attribution line the session gives.
- **One open automated PR at a time.** If the previous batch's PR is not merged, an unattended run does not start a new batch; it only refreshes its report.

## Steps

### 1. Choose the family

- Input: the gap report (missing parts by class and frequency) or the person's request.
- Classify each gap first: a **name or search mismatch** (the part exists under another name, as `R_27R_F_0402` for a search of `R_27_F_0402`) is not a missing part: report it, build nothing. The others are a missing part, a missing land, or a missing binding.
- Take the most-requested family whose land is already published. A family whose land is not published needs a land first (W4 "Package families"): interactive only.
- Classify it for step 6: **chip R/C** (non-polar `passive` CHIP lands: no preview needed, see step 6), **non-polar two-terminal** (body reading per part), or **polar** (diodes, Zeners, SOT-23 parts, LEDs: a pin 1 or cathode reading per part number).
- Output: the family, its parts list from the B1 snapshot (`tools/sourcing_data/jlc_catalog/`), and the class.

### 2. Part-number rule and evidence

- Transcribe the manufacturer's ordering rule ("How to Order") and pin map from its own datasheet into `lib/<pkg>/docs/<mfr>-<family>.evidence.md`: the document's URL and SHA-256, the page, and what each field means. Follow an existing one, e.g. `lib/diode/docs/hongjiacheng-zener-sod323-sod523.evidence.md`.
- **Ask** before fetching a datasheet. The manufacturer's site first; if it blocks retrieval, the manufacturer-authored copy LCSC serves (record both URLs).
- The pin map is read from the datasheet's numbered drawing, per part: for SOT-23 the B/E/C or G/S/D order differs by series.

### 3. Generate the library source

- Use the family's generator (`tools/gen_passive.py`, `tools/gen_zener.py`, `tools/gen_sot23.py`, `tools/gen_sot23_diodes.py`, `tools/gen_sot23_tvs.py`, …). A new family without one gets a generator first; generated files are never edited by hand. Each generator has `--check`.
- Bump the package version (a new part is a minor bump while `0.x`; a moved footprint is breaking, see W3 "Versioning") and update the package README's version history.
- No `supplier` clause yet: a binding needs a map row (step 5) and evidence (step 6).
- `target/debug/cohdl fmt lib/<pkg> --check` must pass.

### 4. Backfill LCSC identities

The backfill maps each (manufacturer, MPN) to its part number by reading every candidate `partdetail` page. It runs in two halves around a person's approval:

```sh
# Copy only the batch's new source files into SUBSET (a lib-shaped directory), then, with no request sent:
python3 -I tools/lcsc_backfill_approved.py plan --subset SUBSET --cache-dir ~/.cache/cohdl-sourcing/jlc -o URLS.json
```

- `plan` prints the number of uncached candidate pages and the list's SHA-256. **Ask** for approval of that list, quoting both numbers (about 2 s per page). An unattended run may proceed only under a standing approval that covers the count.
- Then:

```sh
python3 -I tools/lcsc_backfill_approved.py run --approved URLS.json --sha256 SHA --subset SUBSET \
  --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir tools/sourcing_data/lcsc_backfill/<date>-<family> --date <date>
```

- It refuses to start if the list changed, and refuses any uncached page off the list before a request. It stops by itself after two 403/429.
- Write the run's `README.md` like `tools/sourcing_data/lcsc_backfill/2026-11-24-zener2/README.md`: the approved list's hash, the request count and times, the result (mapped, in review and why), and "Not a binding".

### 5. Map

- `python3 -I tools/promote_lcsc_map.py tools/sourcing_data/lcsc_backfill/<run> --check`, read what it would change, then run it without `--check`.
- Identities left in review stay out. Known cases: an empty "Part Detail" candidate page (`page_unparsed`) and several candidates with no rule to choose (Basic > Preferred > Extended, then stock). A rule change for them is the maintainer's decision; list them in the PR.

### 6. Orientation evidence

- **Chip R/C** on `passive` CHIP lands: no new preview boards. Alex decided on 2026-11-25 that these lands were verified on real assembled boards, so preview misreadings (board 13/14's CC 0402 and 0201) do not block them. A part a W3 group record already covers binds through it (`tools/w3_record.py`). A group with no record needs that decision written into `docs/library-supplier-w3-sampling.md` first, as the evidence its frame rows cite: **Ask** the maintainer; until then the parts stay unbound.
- **Non-polar two-terminal** (inductors, bidirectional TVS): a body reading per part number on its own preview (W3 rules); never borrowed from another part.
- **Polar:** every part number needs its own reading.
  1. Write a preview board for the batch (follow `tools/frame_boards/zener_small_boards.py` and `preview-board-20/`): each part four times (top 0° and 37°, bottom 0° and 37°), no mapping, at least 40 in stock.
  2. Build the package in a clean worktree, with the KiCad CLI that leaves reference designators out of the silkscreen:
     `--kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes`.
  3. Run it with the **jlc-preview** skill. Alex pre-approved preview uploads on 2026-11-25; the run still stops at Component Placements, and needs Alex at the computer (sign-in, captcha).
  4. Read each copy. A two-terminal polar part always goes to manual review: its one mark at pad 1 may be its "+" as well as its band. Map designators to part numbers from the package's BOM CSV, which is sorted by part name, not from the manifest order.
  5. A part read reversed on all four copies is a **candidate** `rotate 180`, not a mapping: a re-verify board places it again with that mapping (as `preview-board-21`), and it binds only when every copy then reads right.
- Write the record under `tools/sourcing_data/jlc_previews/<date>-board-<n>/` as the jlc-preview skill says.

### 7. Frame rows and bindings

- Add one `jlc_frames.json` row per bound (footprint, package version, supplier, part number), citing its record; a nonzero mapping needs an `exception` string with the reason. Keep the file's `indent=2` form and put new rows **before** the W3 sampled rows, then rerun `tools/w3_record.py` for the affected group records (with `--check` after).
- Regenerate the library source so the bound parts carry their `supplier lcsc "<C-number>" [rotate N]` clauses. Only parts with both a map row and evidence are bound; the rest stay unbound and are listed in the package README.

### 8. Test, relock, compare

Use one `CARGO_TARGET_DIR` per worktree and a fresh, empty `COHDL_HOME` per run:

```sh
cargo build && cargo test
COHDL_HOME=$(mktemp -d) COHDL_REQUIRE_BINARY=1 python3 -I -m unittest discover -s tools/tests -t tools/tests
target/debug/cohdl fmt lib --check
python3 -I tools/regen_locks.py . --label <batch>   # after any change to a package's content, docs included
sh tools/examples_golden.sh . <batch>               # every example must report "diff: NONE"
```

Count tests (bindings, frame rows, map rows) are updated to the new numbers, never loosened. A test of a new rule must fail on the tree before the change. When the batch stacks on unmerged PRs, also trial-merge it with the open PRs it touches and run the full suite there.

### 9. Draft PR

- Title the family and the counts ("173 SOD-323/523 Zeners, 35 bound"). The body: what was added, the evidence and where it is, what stays unbound and why, decisions waiting on the maintainer, and the test results.
- Open it as a draft; a person marks it ready for review. Then update the plan doc's log and checklist.

## Stop and report

Stop the batch, keep what is done on its branch, and report when: a test fails and the cause is not in this batch; a fetch is refused or two 403/429 arrive; the page list changed after approval; a reading is ambiguous; a step needs a decision the documents do not settle. Report what ran, what is waiting, and on whom.

## Files

- `tools/lcsc_backfill_approved.py` (plan and run behind an approved page list), `tools/lcsc_backfill.py`, `tools/promote_lcsc_map.py`, `tools/sourcing/http.py`
- `tools/w3_record.py`, `tools/w3_groups.py`, `tools/sourcing/w3_rule.py`, `tools/sourcing_data/jlc_frames.json`, `tools/sourcing_data/lcsc_map.json`
- `tools/frame_boards/preview_package.py`, `tools/frame_boards/jlc_preview.py`, `tools/frame_boards/measure_preview.py` (the jlc-preview skill)
- `tools/regen_locks.py`, `tools/examples_golden.sh`
