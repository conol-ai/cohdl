---
name: lcsc-library
description: Add a batch of parts to the official CoHDL library with LCSC/JLCPCB supplier bindings, end to end - choose a part family, transcribe the part-number rule, generate the library source, check the transcription against the datasheet, backfill LCSC identities under the fetch policy, gather orientation evidence (the jlc-preview skill for polar parts), write frame rows and bindings, test, relock and open a draft PR. Use when asked to add missing parts or a part family to lib/, to bind parts to LCSC, to run a backfill, or to work the library supplier plan (docs/library-supplier-plan.md). Also the step list a scheduled library run follows.
---

# Library batch with LCSC bindings

One batch is one part family: for example the Hongjiacheng SOD-323/SOD-523 Zeners (#138) or the SOT-23 transistors (#137). The batch ends as one draft PR. It never merges, publishes or uploads to the registry: those are the maintainer's (Tony's).

The rules live in these documents:

- `LIBRARY.md`: how devices, parts, pin tables and footprints are modelled. Read section 3 ("Model devices and purchasable parts") and section 4 before step 3: a run that has not read them builds a family the wrong shape, or writes a rating no DRC can use.
- `docs/library-supplier-plan.md` (ground rules, W2–W4), `docs/library-supplier-impl-plan.md` (phases B–E, review checklist) and `docs/library-supplier-w3-sampling.md` (sampled inference for chip R/C).

This skill is the order of work and the gates between steps. Where it and those documents differ, the documents win; say so in the PR.

## Modes

- **Interactive** (a person is present): every step may run. Ask at each gate marked **Ask**.
- **Unattended** (a scheduled run; the schedule itself is the maintainer's, outside this skill): run only steps that need no one, and the gates a person approved in advance and in writing. Name each such approval in the run's report: who gave it, when, and what it covers. This skill grants none. At any other gate, stop: write what is ready and what waits on whom into the run's report and the plan doc's checklist, and end. Never drive the browser unattended: the preview may need a sign-in or a captcha.

## Ground rules

- **Never** merge, publish, upload to the registry, touch the production database, add to the assembler's cart, quote, order or pay.
- **Credentials:** never read, record or type them. Never capture the screen.
- **Fetching** goes through `tools/sourcing/http.py` only: its user agent, at least 2 s between requests to a host, `robots.txt` obeyed, no `/api` path, no EasyEDA host, stop after two 403/429 in a row, no IP change, no proxy. Every run that sends requests needs a person's approval of its exact page list (step 5). Raw supplier pages are never committed. A datasheet PDF is committed when the rule in step 2 says it ships, and only then.
- **Downloaded data** goes in its own empty directory; Python that reads it runs with `-I`, and scripts you write live elsewhere.
- **Fail closed:** a case a tool cannot decide (several candidates, an unreadable page, a conflicting manufacturer) goes to review. Nothing is guessed.
- **Aliases** (`tools/sourcing_data/mfr_aliases.json`) change only after a person's review that cites an issue. A run may propose one; it never adds one.
- **Private designs** never enter git or a PR, not even as a test fixture.
- **Git:** a new branch per batch, in its own worktree; never switch branches in a shared checkout. Merge commits only; no force-push; never a bare `git stash`. Push and open the PR only when the person asks (interactive) or the schedule's approval covers it (unattended). Commits end with the attribution line the session gives.
- **What not to commit:** KiCad's per-user `.kicad_prl` files (`.gitignore` excludes them; never force-add one) and the preview ZIPs you build. Check `git status` before each commit, since removing a file later means rewriting history. Keep any ignore rule narrow: the repository deliberately tracks ZIPs under `lib/cellular/docs/` and in each preview record's `package/` directory, so a blanket `*.zip` rule breaks it.
- **One open automated PR at a time.** If the previous batch's PR is not merged, an unattended run does not start a new batch; it only refreshes its report.

## Steps

### 1. Choose the family

- Inputs:
  - the gap report (missing parts by class and frequency), or the person's request;
  - demand: what real designs reference that the library lacks (staging and the local development database), ranked by use. Demand finds parts nobody asked for by name. The query belongs to whoever runs the schedule; cite it in the report.
- Classify each gap first: a **name or search mismatch** (the part exists under another name, as `R_27R_F_0402` for a search of `R_27_F_0402`) is not a missing part: report it, build nothing. The others are a missing part, a missing land, or a missing binding.
- Take the most-requested family whose land is already published. A family whose land is not published needs a land first (W4 "Package families"): interactive only.
- Classify it for step 7: **chip R/C** (non-polar `passive` CHIP lands: no preview needed, see step 7), **non-polar two-terminal** (body reading per part), or **polar** (diodes, Zeners, SOT-23 parts, LEDs: a pin 1 or cathode reading per part number).
- Output: the family, its parts list from the B1 snapshot (`tools/sourcing_data/jlc_catalog/`), and the class.

### 2. Part-number rule and evidence

- Transcribe the manufacturer's ordering rule ("How to Order") and pin map from its own datasheet into `lib/<pkg>/docs/<mfr>-<family>.evidence.md`: the document's URL and SHA-256, the page, and what each field means. Follow an existing one, e.g. `lib/diode/docs/hongjiacheng-zener-sod323-sod523.evidence.md`.
- **Ask** before fetching a datasheet. Take the first of these that can be had, and record which it is (`docs/pdf-digest.md` section 6.5):
  1. the manufacturer's original PDF with a text layer;
  2. the manufacturer's original without one;
  3. the manufacturer-authored copy LCSC serves, only when the original cannot be obtained (no manufacturer-hosted copy found, or its site blocks retrieval). Record both URLs, or where you looked.
- **Check the file** before reading it as a source, as `LIBRARY.md` section 2 lists: `file`, `pdfinfo`, `pdftotext` (is there a text layer?) and `shasum -a 256`. An HTML or error page with a `.pdf` name is not a document.
- **Ship the PDF or keep it out.** `python3 -I tools/datasheet_ship.py lib/<pkg> PDF…`, with every PDF the batch adds to that package, applies Rule B of `docs/pdf-digest.md` (section 6.3):
  - "ships": save the file unmodified under `lib/<pkg>/docs/`, list it in `lib/<pkg>/docs/README.md` with what `LIBRARY.md` section 2 asks for, and attach it with `#[doc("docs/<file>.pdf")]` to the devices and parts it covers. Identical content is committed once.
  - "stays out (condition N)": commit no PDF. The evidence record gives the URL, revision, SHA-256, size, page count, the pages used, the condition and the numbers the tool printed.
  - The tool judges the package's candidates as one set and does not pick a subset that would fit. If shipping some of them is worth it, that is the maintainer's choice: say so in the PR.
  - Condition 1 (the vendor prohibits redistribution) is yours to read from the vendor's terms; it applies only if the record names where the prohibition is stated. It does not keep an LCSC-served copy out (the owner's decision of 2026-11-27, section 6.5).
  - A shipped PDF is content: it needs the new version step 3 describes, like any other change.
  - Records written before 2026-11-27 say "the PDFs are not committed". Those documents are not moved by a later batch: applying Rule B to them is the measurement in `docs/pdf-digest.md` section 10.5, the maintainer's to run.
- A digest of the document (`docs/pdf-digest.md` Rule A, section 7) is not a step of this skill yet: its checker does not exist. Do not write one by hand to this skill's steps.
- The pin map is read from the datasheet's numbered drawing, per part: for SOT-23 the B/E/C or G/S/D order differs by series.
- A sheet with no ordering rule (a table of type numbers, as the Zener sheets have) is recorded as that: the MPN is the table's type number, and the generator checks each against the series' numbering pattern. A sheet with no numbered drawing gets its polarity or pin order from what it does print (a "cathode band" note, a marked outline), with the page. Record a misprint as printed next to the MPN used (`BZT52B3VOS` for `BZT52B3V0S`).

### 3. Generate the library source

- Use the family's generator (`tools/gen_passive.py`, `tools/gen_zener.py`, `tools/gen_sot23.py`, `tools/gen_sot23_diodes.py`, `tools/gen_sot23_tvs.py`, …). A new series of a family that has a generator extends it (a data file of its own, a source file of its own); a new family gets a generator first. Generated files are never edited by hand. Each generator has `--check`.
- **Shape: devices and parts.** One `device` per shared pin table, one `part` per member the datasheet lists (each row of its table, each ordering code), whether or not the assembler stocks it: stock decides what binds (steps 5–8), not what exists. So the SOD-323/523 Zener batch has 173 parts, 71 in the snapshot and 35 bound. Members that differ in pin table, enable polarity, threshold or a rating are separate devices over one footprint. A part carries a device, a manufacturer, an MPN and a footprint, and no spec of its own (`PartDef` has no `specs`), so a rating that varies across the family forces one device per rating. Examples: TPS2552 (`EN_N`) and TPS2553 (`EN`) are two devices; a 60 V member of a 30 V TVS family is a new device over the same footprint, not a new part.
- **Specs a DRC reads** (`LIBRARY.md` section 3):
  - `voltage_rating` is the maximum **recommended operating** voltage, never the absolute maximum. D001 compares it with the net's annotated voltage, and the absolute maximum silences the rule across the band where the part is already out of specification. Cite the page, and note the margin at the `spec` block so no one "corrects" it to the bigger number.
  - Leaving `spec` out is not neutral: D001 has nothing to compare, so the device is invisible to it. Take the value from the recommended operating conditions; if the sheet gives none, leave it out and list the device in the PR as a known gap. Never guess a number that looks like the family's.
- **Conventions a pin-table check would otherwise report as defects:** an exposed pad the datasheet does not number takes the next number after the last numbered pin, in the datasheet's order; a pin is `output` only if it drives its net (the driver DRC's meaning), whatever the datasheet's type column says, so a pin that carries a programming resistor is `passive`.
- **Version.** Any change to a package's content needs a new version: a new part, a `spec`, a pin or role, a document, a land fix, not only a moved footprint. A locked version must never change; an example that pins it then fails with E1103 ("locked package … has changed on disk"). A new part or spec raises the last digit (`diode` 0.1.4 → 0.1.5), as every batch has; a moved footprint is breaking (W3 "Versioning"). Add the version's section to the package's README (`lib/<pkg>/docs/README.md`).
- **What else pins the version.** Find every place that names the old one and move it in the same commit: `grep -rn '<pkg>.*<old version>' examples tests tools/frame_boards tools/tests --include=cohdl.toml --include=manifest.json --include='*.json'`. So far these have been:
  - an example's `cohdl.toml` (`regen_locks.py` rewrites locks, never manifests);
  - the manifests of generated preview boards (`REGENERATE` in `tools/frame_boards/preview_package.py`): rerun the board's script, since a test compares the manifest with a fresh build. A board's archived upload record (under `tools/sourcing_data/jlc_previews/`) is never touched;
  - fixtures under `tests/fixtures/` and `tools/tests/fixtures/`;
  - the dated datasheet-links report: `python3 -I tools/datasheet_links.py -o tools/sourcing_data/datasheet_links/<date>`, keeping exactly one dated directory.
  Then relock in step 9.
- No `supplier` clause yet: a binding needs a map row (step 6) and evidence (step 7).
- `target/debug/cohdl fmt lib/<pkg> --check` must pass.

### 4. Check what was transcribed

No tool in the other steps compares the source with the datasheet. This step does, before any binding is built on it:

1. Make a neutral kit: every value that was transcribed, dumped as claims with no reasoning attached (each pin's number, name and role; pad and paste geometry; each spec value; the ordering rule; every number of each table row), plus planted blind controls: a few claims deliberately wrong, which the readers are not told about. Build the claims from the generator's data file as well as the source. "Neutral" means the readers do not see our notes on why a value is right; it does not mean dropping values. A generator that keeps a row's numbers only in a comment (the Zener files keep VZ min/max, IZT, ZZT, IR and the marking there) still has them checked: a kit made by stripping comments from the source would leave them unread.
2. Two readers check the kit against the datasheet independently, on separate machines or sessions, sharing no notes. A reader may be a person or an agent.
3. Settle every disagreement, and every control a reader missed, by measurement (the drawing's dimensions, or a pixel measurement of the drawing at 600 dpi), never by consensus: two independent readers have agreed on the same wrong number.
4. Fix what it found in the generator, regenerate, and repeat until a round finds nothing. Record the rounds, the findings and their measurements in the evidence file.

A land fix found here, or later, invalidates any preview package built from the old land: rebuild it, and re-read the parts on that land (step 7).

### 5. Backfill LCSC identities

The backfill maps each (manufacturer, MPN) to its part number by reading every candidate `partdetail` page. It runs in two halves around a person's approval:

```sh
# SUBSET: a directory holding the batch's parts to plan for (any layout; every *.cohdl under it is read).
# BATCH: the same for all the batch's new parts. With no request sent:
python3 -I tools/lcsc_backfill_approved.py plan --subset SUBSET --cache-dir ~/.cache/cohdl-sourcing/jlc -o URLS.json
```

- **What goes in SUBSET:** the batch's parts that the B1 snapshot lists, not every new part. A part the snapshot does not list has no snapshot part number to confirm, and its MPN's candidate pages are other makers' listings: planning for all 173 Zeners lists 582 pages where the 71 listed ones needed 1,206 once and none since. Parts left out are refused and reported `fetch_failed`; say how many in the run's README. Planning for more is the person's choice, with its own list.
- `plan` and `run` each take several minutes and print nothing until the end.
- `plan` prints the number of uncached candidate pages and the list's SHA-256. **Ask** for approval of that list, quoting both numbers (about 2 s per page). An unattended run may proceed only under a standing approval that covers the count.
- Then:

```sh
python3 -I tools/lcsc_backfill_approved.py run --approved URLS.json --sha256 SHA --subset SUBSET --batch BATCH \
  --cache-dir ~/.cache/cohdl-sourcing/jlc --output-dir tools/sourcing_data/lcsc_backfill/<date>-<family> --date <date>
```

- It refuses to start if the list's hash changed or an uncached candidate page is not on it, and refuses any uncached page off the list before a request. It stops by itself after two 403/429.
- A list of no pages (`[]`) is a valid plan: every candidate page is cached, nothing needs approval, and `run` sends no request.
- A run that stopped part way resumes with the same command and approval: the pages it fetched are cached, and it fetches only the rest of the list. After two 403/429, a person decides when to resume.
- The backfill runs over the whole `lib/`, as every run has, so it also re-reads the earlier identities from the cache. Identities with an uncached candidate page off the list are refused and reported `fetch_failed`; `run` prints how many are in the batch (planned for, and not planned for) and how many outside it. Only the batch's counts are a result of this run.
- **One snapshot.** The official map must cite a single confirmed snapshot (`tools/w3_groups.py` refuses two), so each run re-reads every identity the map holds. `run` does this by itself for identities that are in the map and not in the library (an order line of a real board, read by an earlier run): it prints how many it carried. Do not run `tools/lcsc_backfill.py` directly; it would leave those rows on the old snapshot.
- Write the run's `README.md` like `tools/sourcing_data/lcsc_backfill/2026-11-24-zener2/README.md`: the approved list's hash, the request count and times, the result (mapped, in review and why, with both `fetch_failed` counts), and "Not a binding".

### 6. Map

- `python3 -I tools/promote_lcsc_map.py tools/sourcing_data/lcsc_backfill/<run> --check`, read what it would change, then run it without `--check`.
- A promotion re-dates every row. It must add rows only for this batch, refresh every other row, and change no existing row's part number, class or rule. If it would do anything else (a row lost, a row left on the old snapshot, a part number changed), stop and report: do not promote.
- Because every row is re-dated, all the W3 group records go stale. Regenerate each one, then check:

  ```sh
  for r in tools/sourcing_data/w3_records/*.json; do
    g=$(python3 -I -c 'import json,sys; print(",".join(json.load(open(sys.argv[1]))["groups"][0][k] for k in ("footprint","type","mfr","series")))' "$r")
    python3 -I tools/w3_record.py "$r" --group "$g" && python3 -I tools/w3_record.py "$r" --group "$g" --check
  done
  ```

  Only each record's `stock_source` should change.
- Add the run to the replay chain in `tools/tests/test_promote_lcsc_map.py` and move the counts in `tools/tests/test_upload_supplier_snapshot.py` (map rows, overlap).
- Identities left in review stay out. Known cases: an empty "Part Detail" candidate page (`page_unparsed`) and several candidates with no rule to choose (Basic > Preferred > Extended, then stock). A rule change for them is the maintainer's decision; list them in the PR.

### 7. Orientation evidence

- **Chip R/C** on `passive` CHIP lands: no new preview boards. On 2026-11-25 the project decided that these lands were verified on real assembled boards, so preview misreadings (board 13/14's CC 0402 and 0201) do not block them. A part a W3 group record already covers binds through it (`tools/w3_record.py`). A group with no record needs that decision written into `docs/library-supplier-w3-sampling.md` first, as the evidence its frame rows cite: **Ask** the maintainer; until then the parts stay unbound.
- **Non-polar two-terminal** (inductors, bidirectional TVS): a body reading per part number on its own preview (W3 rules); never borrowed from another part.
- **Polar:** every part number needs its own reading.
  1. Write a preview board for the batch (follow `tools/frame_boards/zener_small_boards.py` and `preview-board-20/`): each part number that has a map row, four times (top 0° and 37°, bottom 0° and 37°), no mapping. A part with no map row cannot bind, so it is not placed. The board script enforces the stock a part number needs and names the ones it leaves out.
  2. Build the package in a clean worktree, with the KiCad CLI that leaves reference designators out of the silkscreen:
     `--kicad-cli tools/sourcing_data/jlc_previews/2026-11-09-reverify-2/kicad-cli-no-refdes`.
  3. Run it with the **jlc-preview** skill. The upload needs the approval that skill asks for; a standing approval of uploads counts only when the run's report names it (who, when, what it covers). The run stops at Component Placements and needs the signed-in person at the computer (sign-in, captcha): interactive only.
  4. Read each copy. A two-terminal polar part always goes to manual review: its one mark at pad 1 may be its "+" as well as its band. Map designators to part numbers from the package's BOM CSV, which is sorted by part name, not from the manifest order.
  5. A part read reversed on all four copies is a **candidate** `rotate 180`, not a mapping: a re-verify board places it again with that mapping (as `preview-board-21`), and it binds only when every copy then reads right.
- Write the record under `tools/sourcing_data/jlc_previews/<date>-board-<n>/` as the jlc-preview skill says. By-eye readings go in the record's `readings.json` (and `body.json` for body readings), as in `tools/sourcing_data/jlc_previews/2026-11-24-board-20/`.

### 8. Frame rows and bindings

- Add one `jlc_frames.json` row per bound (footprint, package version, supplier, part number), citing its record; a nonzero mapping needs an `exception` string with the reason. Keep the file's `indent=2` form and put new rows **before** the W3 sampled rows, then regenerate the W3 group records with the loop in step 6.
- Regenerate the library source so the bound parts carry their `supplier lcsc "<C-number>" [rotate N]` clauses. Only parts with both a map row and evidence are bound; the rest stay unbound and are listed in the package README.

### 9. Test, relock, compare

Build into the worktree's own `target/` (unset `CARGO_TARGET_DIR`, or set it to the worktree's `target`): the Python tests run `target/debug/cohdl`, and a target dir shared between worktrees tests another tree's binary. Use a fresh, empty `COHDL_HOME` per run:

```sh
cargo build && cargo test
COHDL_HOME=$(mktemp -d) COHDL_REQUIRE_BINARY=1 python3 -I -m unittest discover -s tools/tests -t tools/tests
target/debug/cohdl fmt lib --check
python3 -I tools/regen_locks.py . --label <batch>   # after any change to a package's content, docs included
sh tools/examples_golden.sh . <batch>               # every example must report "diff: NONE"
```

`regen_locks.py` and `examples_golden.sh` are what catch a package changed without a new version (step 3): an example that pins the old version stops checking. Count tests (bindings, frame rows, map rows) are updated to the new numbers, never loosened; the suite names them when it fails (about 8 minutes a run, so keep the whole log). `regen_locks.py` prints `check lib/std: exit 1 codes ['E1104']` on a healthy tree; that line is not a failure. A test of a new rule must fail on the tree before the change. When the batch stacks on unmerged PRs, also trial-merge it with the open PRs it touches and run the full suite there.

**Merging main (or another batch) into a batch.** When both sides promoted a backfill run, `lcsc_map.json` would cite two confirmed snapshots, and `tools/w3_groups.py` refuses that ("the map cites more than one confirmed snapshot"). Do not hand-merge the map:

1. Replay the promotions in `tools/tests/test_promote_lcsc_map.py` with both sides' runs, and take the merged frame rows (the other side's file plus this side's new rows, before the W3 sampled rows).
2. Rerun the backfill **offline** over the merged library: `run` with an empty approved list (`[]`) and an empty `--subset` sends no request, and refuses every uncached page. It carries the map's identities that the library lacks by itself (step 5; `tools/sourcing_data/lcsc_backfill/2026-11-26-merge-main/README.md` is the first such run). It must map every identity of both sides with the same part number, class and rule; if not, stop.
3. Promote it and do the rest of step 6 (replay test, W3 records, counts), and regenerate the dated datasheet-links report.

### 10. Draft PR

- Title the family and the counts ("173 SOD-323/523 Zeners, 35 bound"). The body: what was added, the evidence and where it is, each datasheet with whether its PDF ships or which condition keeps it out, the transcription check's rounds and findings, what stays unbound and why, devices left without a rating, decisions waiting on the maintainer, and the test results.
- Open it as a draft; a person marks it ready for review. The batch's record is the PR and the run READMEs: `docs/library-supplier-plan.md` keeps no per-batch log, so change it only when the batch changes a rule or a workstream's status.

## Stop and report

Stop the batch, keep what is done on its branch, and report when: a test fails and the cause is not in this batch; a fetch is refused or two 403/429 arrive; the page list changed after approval; a reading is ambiguous; a step needs a decision the documents do not settle. Report what ran, what is waiting, and on whom.

## Files

- `LIBRARY.md`, `docs/library-supplier-plan.md`, `docs/library-supplier-impl-plan.md`, `docs/library-supplier-w3-sampling.md`
- `tools/lcsc_backfill_approved.py` (plan and run behind an approved page list), `tools/lcsc_backfill.py`, `tools/promote_lcsc_map.py`, `tools/sourcing/http.py`
- `tools/w3_record.py`, `tools/w3_groups.py`, `tools/sourcing/w3_rule.py`, `tools/sourcing_data/jlc_frames.json`, `tools/sourcing_data/lcsc_map.json`
- `tools/frame_boards/preview_package.py`, `tools/frame_boards/jlc_preview.py`, `tools/frame_boards/measure_preview.py` (the jlc-preview skill)
- `tools/regen_locks.py`, `tools/examples_golden.sh`
