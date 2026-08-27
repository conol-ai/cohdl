# RFC / decision-record compliance report

**Exit criterion** (docs/design/09-mvp-definition.md): *"Every P0 RFC's
decision record accurately reflects what was actually built — no RFC claims a
mechanism that the implementation doesn't actually enforce."*

**Method** (2026-07-13): eight independent audit agents — one per Accepted RFC
(RFC-001…007) plus one for the Product Constitution's hard constraints — each
read its document in full, enumerated every normative claim, and classified it
as *verified* (with `file:line` evidence in the implementation or tests) or a
*violation*. Every reported violation was then re-examined by an independent
adversarial verifier with instructions to refute it (including running the
compiler on probe fixtures). Full per-claim evidence:
[`docs/compliance-audit.json`](compliance-audit.json).

## Summary

| Document | Claims verified | Violations confirmed | Outcome |
|---|---|---|---|
| RFC-001 units-as-types | 16 | 2 | **both fixed** (see below) |
| RFC-002 pin obligations | 16 | 0 | clean |
| RFC-003 trait satisfaction | 16 | 0 | clean |
| RFC-004 DRC reclassification | 11 | 3 | 2 fixed, 1 documented deviation |
| RFC-005 designator allocation | 18 | 1 confirmed (1 refuted) | fixed |
| RFC-006 nested fn calls | 12 | 1 | fixed |
| RFC-007 generics-over-specs | 12 | 1 | fixed |
| Product Constitution | 15 | 3 confirmed (1 refuted) | 2 fixed, 1 documented provisional |

Deduplicated across documents (three audits independently found the
net-annotation gap), the confirmed deviations were **seven distinct issues:
six fixed the same day, one retained as a documented deviation.**

## Fixed deviations (each with a regression test in `tests/exit_criteria.rs`)

1. **Net voltage annotations accepted any unit literal.** `net X [100nF]: …`
   compiled, then D001 compared capacitance against voltage — violating
   RFC-001's same-unit comparison discipline. Fixed: a non-`Voltage`
   annotation is now `E110` naming expected vs. actual
   (`src/parse.rs`, test `net_annotation_must_be_voltage_typed`).

2. **D001 compared raw magnitudes without a unit check.** A (legal but odd)
   non-Voltage `voltage_rating` spec was magnitude-compared against the net
   voltage. Fixed: D001 compares only Voltage-vs-Voltage
   (`src/drc.rs`, test `drc_d001_never_compares_across_unit_types`).

3. **RFC-005 reserved set omitted override-shadowed prior assignments.**
   Step 3 defines the reserved set over *all* prior assignments; the
   implementation used only live-and-not-overridden ones, so a fresh instance
   could take a number whose path had just been overridden. Fixed: reserved =
   all prior assignments ∪ overrides ∪ all tombstones
   (`src/lock.rs`, test `overridden_paths_prior_designator_stays_reserved`).

4. **`impl Trait` parameters used a second trait-bound-checking code path.**
   DR-016 mandates exactly one mechanism. Fixed: both named generic
   parameters and `impl Trait` value parameters now route through the single
   `check::generics::check_trait_bounds` (`src/check/generics.rs`,
   `src/check/expand.rs`; behavior covered by the existing E403 fixtures).

5. **The promised ambiguous-part-binding note was never emitted.**
   provisional §2 says the deterministic pick is "noted in the build output".
   Fixed: `cohdl build` now prints a note naming the candidates and the
   winner (test `ambiguous_part_binding_is_deterministic_and_noted`).

6. **The `__fn`/`__net` mangling namespace was user-forgeable.** RFC-006's
   collision-free naming guarantee requires the compiler-generated namespace
   to be reserved. Fixed: instance/net names beginning with `__` are `E206`
   (test `dunder_names_are_reserved`).

## Retained, documented deviations

- **D003 single-driver** — **deviation pending a note-side amendment**
  (classification corrected 2026-07-14, review 3; the earlier "retired /
  faithful to the text" wording here overstated it). What is implemented:
  D003 fires only when a net's lone connected pin is a *driver*
  (`output`/`power_out`). RFC-004's accepted sentence ("exactly one
  output-type (driver) pin connected") admits more than one reading; the
  implemented reading is the electrically sensible one (a literal
  lone-pin-of-any-role rule would flag every ordinary two-pin signal net),
  but implemented behavior is not accepted-contract compliance — the RFC
  text needs an amendment that picks this reading explicitly.

- **`pub` is parsed but not enforced.** The MVP's single flat scope has no
  visibility boundary; provisional-syntax.md §1 documents this openly. The
  adversarial verifier rejected the "correct-by-convention" reading: `pub`
  today has zero semantic weight (nothing behaves differently), so nothing is
  *claimed* that isn't enforced. Real visibility arrives with the module RFC.

- **RFC-005's "show the reserved-number set on request" debugging affordance**
  is a Tooling & operations item; MVP tooling beyond `check`/`build` is
  explicitly cut (09-mvp-definition.md). Tracked for post-MVP.

## Error-code housekeeping (resolved by RFC-011)

The two wrinkles the audits flagged in the informal registry are fixed by
RFC-011 (DR-009, docs/design/rfc-011-error-registry.md): unit-mismatch checks
at generic sites moved from `E402`/`E404` into the `E1xx` unit block as
`E112`/`E113`, and a standalone Unicode `Ω` now reports under its own `E107`
instead of the `E001` catch-all. The registry (docs/error-codes.md) is now the
formal v2 baseline, enforced in both directions by `tests/error_registry.rs`.

## External review 2026-07-13 (Codex, at 596acf8): dispositions

An independent review audited RFC-001–013 conformance adversarially. Every
high-confidence reproduction was verified; dispositions below. Fixed items
carry regression tests.

**Fixed in code:**

- F3 — fn-local `net_class` names now carry call-chain-scoped identity
  (RFC-006 style), so a layout-bearing fn is reusable across calls.
- F4 — `diff_pair`/`length_match` require *distinct* resolved nets, catching
  both direct repetition and electrically-merged aliases (E1003/E1004).
- F1 (partial) — `[tolerance: …]` accepts RFC-001 unit literals (`1ms`)
  alongside the quoted-string escape hatch; `fmt` canonicalizes unit-literal
  tolerances to the unquoted spelling.
- F5 — `build` removes a stale `<name>-layout.json` when layout metadata is
  removed from the source.
- F8 — a successful plain `build` renders warnings again (D003 was hidden; a
  regression from the RFC-010 refactor).
- F10 — *reclassified (review 2/3): not "fixed" —* D003's role-aware firing
  is a **deviation pending amendment**; see "Retained, documented
  deviations" above. The code change (role-awareness) shipped; the
  accepted-text conflict remains.
- F12.2 — *reclassified (review 2/3): partial —* `10kΩ` produces one
  targeted E101 (`write \`10kohm\``), not an E103+E107 cascade; the
  documented E401 parser-recovery follow-on remains (the pipeline's uniform
  recovery policy, pinned in tests/cli.rs).
- F6/F7/F11 — `fmt` keeps attribute spans (trailing comments on `#[intent]`/
  `#[placement_hint]`/`#[designator]` survive; comments between an attribute
  and its target stay between); comments inside `layout {}`, trait, impl, and
  part bodies are preserved in place (comment maps are consume-once, so
  double-emission is impossible by construction); pin buses, AVL entries,
  variants lists, and layout constraints wrap at the 100-column soft target;
  author blanks after `{` are preserved. The four >100-column std lines are
  rewrapped.
- Gradeability — the RFC-010 equivalence suite now *decodes* the JSON (local
  parser, no deps) and compares every field (secondary labels, help, end
  positions, build object); the RFC-012 non-impact suite compares verdict,
  diagnostics, `--json` document, netlist, BOM, and the designator lock, on
  clean and failing/warning fixtures.

**Documented contracts (deliberate dispositions, not code changes):**

- F9 — invocation-level failures (bad flags, missing project, design
  selection, nothing-to-build) are the `E000` class: exit 2, prose on stderr,
  never a JSON document; collected source diagnostics render to stderr first
  (they are never discarded). Machine consumers key on the exit code (the
  harness already does). NOTE (review 2): RFC-010's accepted text reserves
  stderr prose for failures *before* diagnostic collection begins, and design
  selection runs after — so this is a **deliberate deviation pending a
  note-side amendment** (or a v2 schema invocation-error envelope), not
  accepted-contract compliance. Documented in README and docs/error-codes.md;
  tested in tests/cli.rs.
- RFC-012 target set — the parser accepts `#[intent]` on `design` and call
  statements (a superset of the spec's list, matching the RFC's own "any
  top-level or body statement" heading). Zero-impact makes the superset
  harmless; kept pending note-side resolution of the RFC's internal conflict.
- RFC-013 zero-impact wording — the tested guarantee is precisely scoped in
  docs/layout-json.md: *valid* layout metadata never changes any observable
  output; an *invalid* block is an ordinary compile error (E1001–E1004).
- `layout.json` — the versioned output contract RFC-013 requires is now
  documented (docs/layout-json.md).

**Note-side items (need conol.ai amendments, not repo changes — the
`docs/design/` snapshot is extraction-only):**

- RFC-013's unquoted `[tolerance: 0.15mm]` example cannot lex: RFC-001's
  closed ten-type set has no length unit. Either a Length unit type RFC or an
  RFC-013 amendment blessing the string form is needed.
  **CLOSED by RFC-018** (2026-07-14): its `mm` literals extend RFC-001's
  closed set with an eleventh `Length` type, which also legalizes the
  RFC-013 example — `[tolerance: 0.15mm]` now lexes, type-checks (E110
  accepts Time or Length), and is the canonical unquoted spelling.
- RFC-013's E1005 ("net_class referenced before declaration") is
  unrepresentable in its own four-kind grammar — nothing references a class
  by name. Kept `[RESERVED]` in the registry until the vocabulary grows.
- RFC-013's normative placement example (`#[placement_hint] inst esp` as a
  *re*-annotation of an already-declared instance) is not valid grammar.
- RFC-011's accepted E9xx table (five codes) conflicts with the
  implementation's pre-existing E901–E908. Keeping E901–E908 is a
  **deviation from the Accepted RFC-011 text** — "the implementation got
  there first" is a reason to amend the note, not a claim of compliance.
  The registry documents the mapping; the accepted RFC/DR text still needs
  amending.
- The language spec's error-block table omits E10xx (its own layout section
  references it).
- RFC-002's "omitted obligation defaults to required" (implemented, and used
  by the spec's own examples) contradicts the RFC's explicit-obligation
  wording; RFC-001's in-`rule` comparison surface and RFC-007's literal
  impl-Trait desugaring remain out of scope pending `rule` syntax / a
  dedicated refactor, as previously documented above.

## External review round 2 (Codex, at 83567b9): dispositions

The second review verified the round-1 fixes held, corrected several
overstatements in the reply, and reproduced new residuals. Fixed in code:

- fmt: a comment trailing a ONE-line construct now rides the construct's last
  emitted line (closer), not the opening header — one-line trailing comments
  are held before interior emission and re-attached after it.
- fmt: comment-only trait/impl bodies keep their comments inside the braces
  instead of collapsing to `{}` with the comments exiled to EOF.
- fmt: instance attributes serialize in SOURCE order (never a fixed canonical
  order), so comments between/after attributes cannot migrate; an attribute
  sharing its line with the declaration leaves that line's comment to the
  declaration.
- Tolerance unit literals restricted to `Time` (RFC-013's
  `<Time-or-length-unit>`); `5V`/`100nF`/`1kohm` now E110 with a targeted
  help. Lengths remain the string escape hatch pending the note decision.
- Design-selection failures no longer discard collected source diagnostics:
  the pipeline carries `selection_error` alongside everything collected, and
  the CLI renders diagnostics to stderr before the exit-2 error (both modes);
  same for nothing-to-build.
- Command-specific flags validated: `--json` rejected on `fmt`, `--check`
  rejected on `check`/`build`.
- The registry-to-source completeness direction now requires a REAL
  `Diagnostic::error/warning` constructor call site (not any quoted literal);
  the lexer's shared error helper was restructured so E001/E107 are literal
  call sites.
- `10kΩ` is pinned end-to-end (tests/cli.rs): exactly one E101 with the
  rewrite help, no E103/E107 cascade, plus the documented E401 recovery
  follow-on — classified **partial** (lexer targeted; parser-recovery fallout
  is the pipeline's standing recovery policy, applied uniformly to all lex
  errors in value position).
- Gradeability: the JSON decoder test now asserts primary/secondary file and
  full secondary start/end positions on a fixture with guaranteed secondary
  labels; the intent non-leak assertion uses substrings that actually occur;
  the intent failing-fixture comparison covers codes, severities, messages,
  label texts, and help (everything but positions, which the attribute's
  physical presence necessarily shifts); layout zero-impact compares verdict,
  diagnostics, --json document, netlist, BOM, and designator lock.

Standing honest classifications (unchanged by code):

- F6/F11 remain **partial** against RFC-009's literal "never moved": interior
  comments of a construct that fmt reflows or collapses move to an adjacent
  line (never dropped — enforced by the EOF backstop). Full positional
  preservation needs comment-anchored trivia ownership or an RFC-009
  amendment.
- Blank-line handling: file-leading blanks and blanks inside a reflowed
  member list are dropped (canonical choice; deviation from "never removed").
- RFC-010/012/013 gradeability is materially stronger but still not the
  accepted "every fixture, every field" wording — tracked, not claimed.
- E9xx, E1005, tolerance-`mm`, the placement example, RFC-012's target list,
  and the zero-impact wording still require note-side amendments.

## RFC-014 (LSP) implementation notes (2026-07-14)

Implemented per DR-020: `cohdl lsp` (src/lsp.rs) — hand-rolled JSON-RPC/stdio
transport; `lsp-types` (pinned `=0.97.0`) + its serde requirements as the
project's single scoped dependency exception, confined to the LSP layer. All
four RFC capabilities ship with fixture tests (tests/lsp.rs), including the
diagnostics-equivalence suite against `cohdl check --json` (full four-field
projection over a multi-stage fixture corpus, both range endpoints, UTF-16
mapping with a non-ASCII fixture, secondary+help projection) and an
unsaved-buffer overlay test. Editor launch snippets: docs/lsp.md.

Scope honesty (review 3 corrections):

- **The real-client acceptance item is NOT satisfied.** RFC-014 explicitly
  requires at least one test against a real editor client, "not just
  unit-testing"; a subprocess protocol test is exactly what that sentence
  distinguishes from a live client. The item stays open until a real VS Code
  session is run and recorded here. (The earlier "satisfied at protocol
  level" wording contradicted itself and is withdrawn.)
- **DR-020's dependency rationale is narrower in practice than its text.**
  `lsp-types` supplies the typed response shapes (`Hover`, `Location`,
  `Range`, `Position`, `Uri`); initialize, dispatch, and publishDiagnostics
  payloads are raw `serde_json` values. This is an honest narrowing pending
  either consistent typed usage or a DR amendment.
- **POSIX only:** `file://` URIs with empty/`localhost` authority; Windows
  drive letters, backslashes, and UNC forms are not supported (documented in
  docs/lsp.md).
- Lifecycle (-32002 before initialize, single initialize, InvalidRequest
  after shutdown), save-included sync advertisement, case-insensitive
  headers, and `relatedInformation` capability negotiation are implemented
  and pinned in tests/lsp.rs. Transport recovery after a header block with
  no Content-Length is best-effort at blank-line boundaries only (pinned
  honestly in the malformed-frames test).

## Inherited tooling obligations from earlier RFCs (review 3, R10)

- **RFC-002 pin-reference hover — implemented (2026-07-14).** Hover on a pin
  USE SITE (`d.A` in net/nc/call statements, `target.A` on a trait-typed fn
  parameter) resolves to the pin's obligation/role, not only the
  declaration. Tested in tests/lsp.rs.
- **RFC-001 unit×prefix hover — implemented (2026-07-14).** Hover on a unit
  literal (device spec values, generic defaults/args, net voltage
  annotations) shows the literal's unit type and its allowed-prefix table
  row. Tested in tests/lsp.rs.
- **RFC-001 completion data — note-side conflict, needs a decision.**
  RFC-001 says the unit×prefix table must appear in LSP *completion*/hover
  data; RFC-014 scopes the server to exactly four capabilities and
  explicitly excludes completion. The hover half is now implemented; the
  completion half is a direct contradiction between two Accepted texts that
  only a note-side amendment can resolve. Not implemented.

## RFC-015 (IPC-2581) implementation notes (2026-07-14)

Implemented per DR-021: `cohdl build --emit ipc2581` writes
`out/<name>.xml`, an IPC-2581B1 document emitted natively from `DesignIr`
(`src/emit/ipc2581.rs`, hand-rolled XML — no new dependency). Both DR-021
gradeability gates are mechanical (tests/ipc2581.rs): schema validity runs
`xmllint --schema` against the IPC Consortium's published `IPC-2581B1.xsd`
(vendored at tests/schema/, fetched from webstds.ipc.org) over three
fixtures and both repo examples, and fidelity equivalence cross-checks
nets/components (vs. the `.net`), BOM items (vs. the CSV), constraints and
hints (generically vs. the parsed `layout.json`), and the full per-component
`COHDL_SPEC_*` map (vs. the IR's resolved specs) over that same corpus —
fixtures AND examples. The `logical-complete,physical-minimal` marker is
machine-readable (`COHDL_COMPLETENESS` NonstandardAttribute) and
human-visible (`FunctionMode/@comment`). Contract: docs/ipc2581.md.

Adversarial verification (same-day round, 4 attackers + skeptic refuters;
2 of 11 findings refuted as documented decisions): all 9 confirmed findings
fixed before landing — XML-1.0-illegal control characters made the document
non-well-formed (now U+FFFD-replaced, disclosed); literal tabs normalized
to spaces by conforming parsers, silently diverging from layout.json and
colliding XSD enterprise keys (now `&#9;`/`&#13;` character references);
hostile `designator_prefix` strings reached `RefDes/@name` raw and could
collide componentKeys (all refdes spellings now route through one
collision-free table); distinct MPNs could collapse to one `AvlMpn/@name`
aliasing a competitor's part (@name now the group key, raw MPN in @other);
spec-content fidelity was untested (deleting the spec emission passed the
suite — now pinned per component against the IR); the fidelity corpus
claim overstated example coverage (examples now genuinely fidelity-checked);
and `--emit` value errors on non-build commands outranked the command error
(command compatibility now checked first). Each has a named regression in
tests/ipc2581.rs.

Honest boundaries and decisions taken under the RFC's own latitude:

- **Opt-in flag, not always-on** — the RFC left "`--emit ipc2581` or an
  always-on artifact" to implementation review; the flag was chosen because
  the spec text requires the `--json` `"ipc2581"` key to be present "only
  when the artifact is emitted". A build *without* the flag removes a stale
  `<name>.xml` (same partner-safety rule as `layout.json`, review F5).
- **Artifact name `<name>.xml`** per the RFC's Design section
  (`<package-name>.xml`); the spec heading's "(ipc2581.xml)" is read as a
  format label, not a filename.
- **Physical placeholders**: the XSD requires `Datum`, `Component/Location`,
  and `Package/Outline` — emitted as zero-size/origin placeholders, which is
  the "minimal-valid-physical-section idiom" the RFC's Design section
  anticipated; the completeness marker governs.
- **Schema-required sections CoHDL has no concept for** (LogisticHeader
  roles/enterprises/persons, HistoryRecord): fixed deterministic content;
  every `xsd:dateTime` is the epoch instant, because byte-stable output is
  a Constitution hard constraint and the wall clock may not enter an
  artifact.
- **Constraint mapping — vendor extension, NOT native IPC semantics
  (review F2, corrected 2026-07-14)**: IPC-2581B1 has no user-named
  net-class constraint element — `LogicalNet/@netClass` is a closed enum
  (GROUND/POWER/SIGNAL, mapped from `[gnd]`/voltage annotations), and there
  is no standard `SpecRef` from a `LogicalNet` to a named constraint. So
  RFC-013 constraints ride `CadHeader/Spec` + `General/Property` entries
  under `cohdl:`-prefixed names, and placement hints ride a
  `COHDL_PLACEMENT_HINT` nonstandard attribute. These are **CoHDL vendor
  extensions recoverable only by a CoHDL-aware decoder** — they are NOT the
  self-describing native constraint semantics a generic IPC consumer (or
  Quilter's differential-pair flow) will honor. The earlier ledger wording
  ("this is the native-constraint-elements reading") **overstated it**: a
  compliance report cannot narrow RFC-015's Accepted text, which promises a
  direct mapping into native constraint elements
  (`rfc-015:42`, `10-language-specification.md:471`). This is recorded as a
  **deviation pending a note-side amendment**: either RFC-015/spec/DR-021
  are amended to state these are proprietary vendor extensions requiring a
  CoHDL adapter, or a partner-recognized mapping is obtained and tested in
  that consumer. Listed in the note-side amendment section below.
- **xmllint gate**: authoritative in CI (libxml2-utils installed in
  ci.yml); locally the validity test skips with a loud warning when xmllint
  is absent rather than failing unrelated work.
- **Not yet a complete Quilter handoff — real-partner gate OPEN (review
  F1)**: RFC-018 gave the document real footprint geometry (per-pad
  `Package/Pin`, courtyard `Outline`), closing RFC-015's named
  footprint-geometry future-work item. But a Quilter starter board also
  requires a valid board **outline** and **placed** footprints, and the
  document supplies neither: there is no `Profile`, and every
  `Component/Location` is `(0,0)` (layout has not been performed). So the
  artifact is a **schema-valid logical-interchange document with real
  footprint geometry**, not yet "a board a router can place and route"
  (`rfc-015:11`). The completeness marker states this honestly and must
  stay; the RFC's own instruction to validate against a real target before
  declaring done (`rfc-015:98`) remains **open** — no import/job-creation
  pass against a real IPC consumer has been recorded. This is the RFC-015
  acceptance blocker, alongside the RFC-014 real-VS-Code pass.

## RFC-016 (module system) implementation notes (2026-07-14)

Implemented per DR-022: file-tree-mirrors-module-tree (rooted at
`[package] name`), `use path::Name;` one-name imports, qualified
`pkg::mod::Name` references, per-module-path collision scoping, and `pub`
enforced across package boundaries. Architecture: `resolve.rs` REWRITES
every trait/device/part/fn reference identifier in place to its resolved
fully-qualified path (union symbol table + per-file imports + own-package
index + std prelude); the `World` maps are fq-keyed and every downstream
stage keeps exact-key lookups. New diagnostics: E207 (ambiguous unqualified
name, names every candidate), E208 (`use` collision, names both paths),
E209 (visibility violation, names the item and its package); unresolved
`use`/qualified paths reuse E202 with a closest-match suggestion.
Conformance: tests/modules.rs (including the RFC's mandatory two-package
colliding-name regression), plus fmt/LSP additions.

Decisions taken under the accepted text's latitude (each a candidate for a
note-side blessing, none contradicting it):

- **The std prelude.** The RFC is silent on std, but its load-bearing
  compatibility property ("a project that never imports anything doesn't
  feel modules' weight") requires std names to stay visible unqualified —
  so `std` is a real package whose `pub` items are implicitly in scope
  (Rust's prelude precedent, "Rust-inspired, not Rust-copied"). Qualified
  `std::Name` also works. Every std item was already `pub`.
- **Resolution precedence**: explicit `use` imports, then the own package's
  modules, then the std prelude. A project declaring `MLCC` therefore
  SHADOWS std's for bare references (previously a global duplicate error —
  the one observable behavior change for existing single-package projects,
  and a strictly-more-permissive one).
- **Files directly under `src/` live at the package root** (their stem is
  not a module segment) — the spec's own tree example says so
  (`prelude.cohdl → sparkfun`); nested files contribute directories + stem.
- **Package names sanitize to path-root segments** (`rpi-pico2` →
  `rpi_pico2`): `::`-path segments must lex as identifiers.
- **Intra-package cross-module name collisions are legal declarations**
  (per-module scoping) **and ambiguous at a bare reference** — E207 at the
  reference site naming every candidate, with deterministic first-candidate
  recovery. The RFC defines collision handling for same-module and
  use-site cases; this completes the matrix in its spirit.
- **Designs are excluded** (the RFC scopes itself to trait/device/part/fn
  paths): design names stay bare and project-global.
- **`pub use` is rejected** with a targeted parse error (Non-goals), and
  `use` requires the spec's canonical trailing `;`. An identical duplicate
  import is not an E208 collision (the RFC's wording: "from different
  paths").
- **fmt**: contiguous `use` runs sort by path (Tooling & operations);
  a full-line comment inside a run pins the author's order instead —
  comment preservation outranks sorting, consistent with every previous
  fmt decision (tests pin both behaviors).
- **Display vs identity**: diagnostics and emitters show the short (last-
  segment) spelling where they showed bare names before — netlist/BOM
  bytes for existing projects are byte-identical; identities in the IR
  (`IrInstance.device`, `impl_traits`) are fully-qualified. D002's
  `Polarized` anchor matches the trait's short name (and checks EVERY
  short-named candidate, so a same-named project trait cannot shade the
  std check out).
- **Byte-stability tie-breaks compare SHORT names.** Wherever "the
  lexicographically-smallest name wins" feeds output bytes — the RFC-005
  designator-prefix rule and provisional §2's ambiguous part binding —
  the comparison is on the short name (the pre-module flat order), with
  the fq path as a deterministic tiebreaker. Moving a declaration between
  modules/packages therefore never changes designators or the BOM.
- **`std` is a reserved package name** — a project claiming it would merge
  into the standard library's namespace (with cascading diagnostics inside
  std/ files the user cannot edit); rejected at project load.
- **A design sharing a bare name with a same-package declaration is E201**
  (the flat model's protection, kept; only cross-package/std shadowing is
  the disclosed new permissiveness).

Not implemented (RFC Non-goals): glob imports, `pub use` re-exports,
aliasing. Multi-package loading beyond project+std is NOT yet loadable
from disk — the resolver handles arbitrary package sets (exercised
directly in tests/modules.rs and tests/library.rs), but the project
loader still assembles exactly project+std; on-disk dependency loading
needs the distribution mechanics RFC-017 itself defers (a future RFC).
(Corrected 2026-07-14: this line originally promised the loading half
"arrives with RFC-017" — that overstated RFC-017's scope.)

Adversarial verification (two same-day rounds, 4 attackers + skeptic
refuters each; refuted findings were documented decisions): all confirmed
findings fixed pre-landing, each with a named regression.

Round 1 (contract/semantics): the designator-prefix order flip under fq
keys (HIGH: violated the RFC's own "behavior unchanged" compatibility
promise on fresh builds), the part-binding order flip + self-contradictory
build note, the `std` package-name merge, missing closest-match suggestions
at reference sites (World::suggest was dead code — now wired into every
unknown-name diagnostic), the undisclosed loss of design-vs-declaration
collision detection, the D002 shading case, and the false "nothing is
declared there" message for design imports (tests/modules.rs).

Round 2 (fmt/LSP/grammar, re-run after round 1's attackers were lost to
API errors — refuters confirmed every round-1 fix held): fmt idempotence
broke when a full-line comment sat INSIDE a multi-line `use` path (HIGH:
the pin/sort decision now depends only on between-import comments, and an
import's interior comments ride just above its canonicalized line —
tests/fmt.rs); trailing comments on non-final lines of a multi-line `use`
were exiled to EOF; the LSP's nested unsaved buffers landed at the package
root (module inference diverged from the CLI — every in-project overlay
now joins with its project-relative display; loose files stay separate
units per review R7 — tests/lsp.rs); the mandatory two-package regression
was strengthened to reference level (both qualified paths + a bare-name
probe); and four parser-recovery/span defects (`#[intent]`-on-use and
`pub use` anchoring, lone-segment span, broken-`use` resynchronization, a
stray `;` body swallowing following declarations, `use` inside a body
misparsing as a call) — tests/modules.rs `use_grammar_errors_anchor_and_recover`.

## RFC-017 (library registry) implementation notes (2026-07-14)

Implemented per DR-023 (as revised): `#[doc("relative/path")]` reference
documents (one or MORE per declaration, zero compilation impact — the
compiler never opens the files; paths recorded in `World::docs` and
surfaced by LSP part hover), and `footprint` as a fifth top-level
declaration kind resolved through RFC-016's machinery unchanged (module
paths, `use`, `pub`/E209, E207 ambiguity, closest-match suggestions).
`part`'s `footprint:` field is a SYMBOL reference — the string form is a
targeted parse error with migration help. Conformance: tests/library.rs +
LSP additions.

Decisions and honest boundaries:

- **The body is enforced EMPTY.** RFC-017 leaves footprint content
  unspecified; rather than silently accepting arbitrary tokens, a
  non-empty body is a targeted error naming RFC-018. "Symbol-resolution-
  complete, format-empty", made mechanical.
- **The netlist's footprint field now carries the resolved symbol's
  fully-qualified path** (`std::FP_C_0402_1005Metric`), not a KiCad
  library id. This is the disclosed breaking change's observable half:
  pcbnew can no longer resolve footprints from the emitted `.net` until
  RFC-018 gives `footprint` real geometry to project into `.kicad_mod`.
  The historical KiCad checkpoint (docs/demo) predates this and stands as
  executed; BOTH committed golden netlists (sensor-node and rpi-pico2)
  were regenerated (footprint fields only — designators, nets, BOM, and
  lock bytes are unchanged). The IPC-2581 document's package names become fq symbol
  paths (identifier-safe by construction; the hostile-footprint-string
  sanitization case is now unrepresentable and its regression was
  retired).
- **Stage-one migration executed** per the RFC's own two-stage plan:
  placeholder `pub footprint FP_<name> {}` declarations in
  `std/footprints.cohdl` and `examples/rpi-pico2/src/footprints.cohdl`,
  each carrying a `// was: "<KiCad id>"` comment preserving the original
  mapping for RFC-018's real-content stage. Placeholder names carry an
  `FP_` prefix (one KiCad name, `ESP32-S3-WROOM-1`, collided with the std
  DEVICE of the same sanitized name — the shared per-module namespace
  makes unprefixed placeholders unsafe).
- **`#[doc]` targets**: top-level declarations (rejected on `use`, which
  is an import, not a declaration). Doc paths are not existence-checked —
  the RFC names that as a real gap for a future lint, not silently
  assumed.
- LSP: goto-definition works on footprint references; hover on a part
  name shows MPN/MFR, the resolved footprint symbol, and `#[doc]` paths.
- The spec snapshot's "Footprints and pads (copad/cofp)" section was
  STALE against RFC-018's same-day naming correction (plain
  `pad`/`footprint`, "no rename" per its Alternatives/Decision); the
  implementation followed the RFCs' accepted text. **RESOLVED note-side
  2026-07-14**: the live spec now reads pad/footprint (with an explicit
  correction note) — snapshot refreshed, and the corrected section's own
  examples compile and project end-to-end against the implementation.
- **On-disk dependency loading is NOT part of this landing.** The
  resolver handles arbitrary package sets (tests drive it directly), but
  `load_project` still assembles exactly project+std; placing a second
  library on disk and compiling against it needs the distribution
  mechanics RFC-017 itself defers. The RFC-016 section's original
  forward promise was corrected accordingly.

Adversarial verification (same-day round, 3 attackers + skeptic refuters;
17 of 18 findings confirmed, 1 refuted as the documented import-precedence
contract): all fixed pre-landing, each with a named regression in
tests/library.rs — panic-mode recovery didn't know the `footprint`
contextual keyword (a preceding parse error silently swallowed a following
footprint declaration → phantom E202s); a misplaced footprint in a body
misparsed as a fn call and destroyed the rest of the body; invalid inst
attributes inside never-expanded fns were silently accepted (attr
validation moved to parse); footprint-body recovery cascaded past the
closing brace and unclosed bodies anchored their error on the NEXT
declaration (matched-brace skip + opener anchoring); `footprint {}`
missing its name got the generic message; `#[doc]` on an impl was
silently dropped (now a targeted error — impls are unnamed); the E802
help and reject-attrs messages still taught pre-RFC-017 forms;
provisional-syntax §2 still specified the string form; the committed
rpi-pico2 golden netlist had not been regenerated (now it is); the LSP
part-hover doc claim was untested (now asserted); and the RFC-016
section's multi-package-loading promise was corrected (above).

## RFC-018 (pad/footprint format) implementation notes (2026-07-14)

Implemented per DR-024 and RFC-018's same-day naming correction (plain
`pad`/`footprint`, no `copad`/`cofp` rename — the spec's section was
corrected note-side the same day; see the RFC-017 notes). `pad` is a sixth top-level
declaration kind (closed vocabulary: shape rect/circle/oval, layer
top_copper/bottom_copper/through_all, plating smd/plated_through_hole,
`drill` ⇔ plated_through_hole); `footprint` bodies gain `pad N: Symbol at
(x, y)` placements plus optional `courtyard`/`silkscreen_ref`. Both
resolve through RFC-016's machinery unchanged. Conformance:
tests/footprint.rs + LSP additions in tests/lsp.rs.

Decisions under latitude, and honest boundaries:

- **`Length` is an eleventh RFC-001 unit type** (symbol `mm`, no SI
  prefixes, signed — footprint offsets need negatives, same latitude
  Temperature already uses). RFC-018 writes `mm` literals throughout
  without amending RFC-001 explicitly; treating that as an implied
  amendment is the only reading under which the RFC's own examples lex.
  Side effect, deliberately embraced: RFC-013's `[tolerance: 0.15mm]`
  example is finally representable — E110 now accepts Time or Length,
  `fmt` unquotes `"0.15mm"` tolerances, and the note-side amendment item
  above is closed. A note-side RFC-001 amendment formalizing `Length`
  remains desirable and is requested here.
- **Pad-consistency (E807) runs at `cohdl build` only**, per the RFC:
  `check` does not bind parts, and the footprint-vs-device comparison is
  meaningless without a binding. The check compares the footprint's pad
  NUMBER SET against the bound device's physical pin numbers for the
  instance's variant, naming each missing/extra number, one report per
  (part, footprint) pair — over EVERY AVL entry's footprint (primary and
  alts; adversarial round — an alt-sourced part must fit the same land
  pattern, or the lie stays latent until a fab swaps sources).
- **Placeholder footprints are exempt from E807** — a placeholder is a
  footprint with a fully EMPTY body (no pads, no courtyard, no
  silkscreen_ref): RFC-017's stage-one shape, kept legal by the RFC's own
  Migration path while stage-two authoring proceeds. A placeholder emits
  no `.kicad_mod` and keeps the IPC-2581 zero-size-outline idiom. A
  courtyard-only footprint is NOT a placeholder (adversarial round): its
  empty pad set is checked against the device (and fails), and its
  authored geometry projects — authored content never silently vanishes.
- **Error codes**: E805 (pad declaration inconsistency), E806 (footprint
  body inconsistency), E807 (pad/device mismatch at build), all in the
  E8xx block the RFC reserves. Unresolved pad references reuse RFC-016's
  E202/E205, as the RFC itself prescribes.
- **Geometry projection**: `cohdl build` writes
  `out/footprints/<pkg>-<Name>.kicad_mod` (fq path with `::` → `-`, which
  is INJECTIVE because `-` cannot appear in identifiers — the first-cut
  `__` separator collided on names containing `__` and silently clobbered
  one artifact, an adversarial HIGH; the directory is removed and
  rewritten each build) for every content-bearing footprint referenced by
  a bound part's primary or alt entries; `build --json` lists them under
  `"kicad_mod"` (present only when non-empty — the same only-when-emitted
  pattern as `layout`/`ipc2581`). The IPC-2581 `Package` gains a real
  courtyard `Outline` and one logical `Pin` element per distinct electrical
  pad number (repeated physical placements are retained in its layer features;
  the schema's `Outline` requires a Polygon, so a CIRCLE courtyard projects
  there as its bounding square — disclosed; `.kicad_mod` keeps the true circle).
  The base shape vocabulary remains rect/circle/oval; bounded fabrication
  controls add an exact four-corner radius and one-corner chamfer. Authored
  controls project as KiCad roundrect/custom polygons and IPC `RectRound`/
  `Contour`; an external roundrect whose radius was never authored still
  projects as `rect`. Both emitters share `emit::geom`: all arithmetic on
  the lexer's exact femto-mm integers (corner halving at 10^-16 mm), no
  floats, no re-parsing of source text, canonical minimal rendering — the
  emitters cannot disagree, `1.0mm`/`1mm` project identically, and
  7..15-decimal literals survive exactly (the first cut truncated at six
  decimals and used f64 in the IPC-2581 corners).
- **Sizes are extents**: pad `size`, `drill`, and courtyard `size` must
  be > 0mm (E805/E806) — `Length` is signed for placement offsets only;
  a negative or zero extent produced schema-invalid IPC-2581 (the XSD's
  nonNegativeDoubleType) and inverted KiCad rects (adversarial round).
- **Stage-two migration is STARTED, not finished** — exactly as the RFC's
  completion bar allows ("real, non-mechanical authoring work"). A starter
  pad library (`std/pads.cohdl`) plus real KiCad-derived geometry for the
  two 0402 chip footprints landed; both example boards now project them,
  with the `.kicad_mod` outputs committed as goldens (netlist/BOM/lock
  bytes unchanged). The remaining six std footprints stay as exempt
  placeholders pending datasheet-derived authoring.
- LSP (RFC-018 Tooling): hover on a pad placement shows the resolved
  pad's shape/size/layer/plating (+drill); goto-definition works on pad
  symbols in placements.
- `fmt` canonical form: pad fields one per line in source order;
  footprint body members (placements, courtyard, silkscreen_ref) one per
  line in source order; `courtyard`/`silkscreen_ref` inline their fields
  like same-line attribute blocks.

Adversarial verification (same-day round, 5 attackers + 3-skeptic
majority refutation per finding, 74 agents): 15 findings confirmed (3
high), 8 refuted. All confirmed findings fixed pre-landing, each with a
named regression in tests/footprint.rs: E807/projection skipped alt-entry
footprints; the `::`→`__` artifact naming was not injective (two distinct
footprints silently collapsed into one file, reported twice in `--json`,
verdict still "pass" — now `::`→`-`); nameless `pad {`/`footprint {`
recovery double-counted the brace and swallowed every following
declaration plus a phantom unclosed-body error; an unclosed `courtyard {`
stole the footprint's closer and produced a phantom E202 for a declared
symbol (body loops now stop, without consuming, at top-level declaration
keywords and report the unclosed opener); one field typo cascaded into
5-8 phantom errors (sync is now paren-aware, field loops recover per
field instead of bailing, `length_pair`/`length_tuple` report a missing
`(` once); negative/zero extents passed and emitted schema-invalid XML;
the two emitters computed the same geometry differently (f64 vs 6-decimal
truncation) contradicting this ledger's own exactness claim; a circle
courtyard projected in `.kicad_mod` but was silently dropped from
IPC-2581; a courtyard-only footprint was misclassified as a placeholder;
`footprint:` naming a pad fell through to E202 instead of the E205
kind error; and E102/E105/top-level-list messages still described the
pre-Length world. Refuted (attacker taste, not normative): tabular
column alignment in fmt and negative length_match tolerance (magnitude
semantics not normative). (The earlier "duplicate-field last-wins"
disposition here was doubly wrong — the behavior was first-wins, and
review 5/R5-7 correctly made duplicate singleton AVL fields an error, so
neither wins.)

## External review round 3 (Codex, at f901cdf): dispositions

Fixed in code (regression tests named in parentheses):

- R1 — `fmt` unquotes a `[tolerance: …]` string only when it lexes as a
  `Time` literal; `"5V"`, `"100nF"`, `"abc"` stay quoted and idempotent
  (tests/fmt.rs `tolerance_quoted_non_time_stays_quoted`).
- R2 — same-line instance attributes serialize by source span, not
  category; empty trait/impl bodies with an opener-line trailing comment
  keep the braces open with the comment attached (tests/fmt.rs).
- R3 — `Args::validate()` enforces the full per-command flag matrix
  (including `lsp` rejecting all flags/positionals and `--std`/`--no-std`
  mutual exclusion), and every post-check exit-2 path (lock parse, out-dir
  creation, artifact writes, stale-layout removal) renders collected
  diagnostics before the error (tests/cli.rs).
- R4 — the intent failing-fixture comparison strips only positions from the
  parsed JSON model (file names, label messages, and help values all
  compared) on a fixture with guaranteed secondary+help; a new
  own-line-attribute mutation test requires the ENTIRE `--json` document
  byte-identical; the layout mutation test now compares the `--json`
  document; the registry scanner strips Rust comments (string/char-literal
  aware) before matching (tests/intent.rs, tests/layout.rs,
  tests/error_registry.rs).
- R6 — `analyze` distinguishes a genuinely nonexistent unsaved loose file
  (phantom fallback) from real project/std/pipeline load failures, which
  surface as `window/showMessage` and never publish a false-clean empty
  list; closing a phantom buffer publishes an explicit empty list
  (tests/lsp.rs `project_load_failure_surfaces_not_false_clean`,
  `broken_std_surfaces_as_message`, phantom-close case).
- R7 — diagnostic ownership is keyed per analysis unit (project root, or
  the file itself for loose files); re-checking one unit clears only its
  own stale URIs (tests/lsp.rs two-loose-files and two-files-one-project
  regressions).
- R8 — lifecycle state machine, `textDocumentSync` object with `save`,
  case-insensitive headers, `relatedInformation` gating, `-32602` for
  malformed positional params, `file://localhost` authority support, and
  client-URI-spelling preservation on publishes (tests/lsp.rs).
- R9 (gradeability half) — corpus equivalence harness with full-field
  comparison incl. `range.end.character`, exact hover text/ranges, exact
  definition end span, exact reference URIs+ranges, non-ASCII fixture.
- R10 (hover half) — pin-reference and unit-literal hover, above.

Documented/classified (not code):

- R5 — this report's older sections rewritten to remove the contradictory
  "retired"/"shipped-first"/"Fixed" framings (D003, E9xx, F10/F12); the
  README status wording made exact; src/ast.rs and src/lsp.rs header claims
  narrowed to what is true.
- R9 (client half) — the real-VS-Code acceptance item is open, above.
- R10 (completion half) — note-side conflict, above.

Note-side contradictions surfaced by review 3 (the `docs/design/` snapshot
is extraction-only — these need conol.ai edits, listed here so they are not
lost):

- `03-capability-architecture.md` describes a `tower-lsp` server with
  completion, a real VS Code extension, "no real formatter", and no JSON
  API — all stale against the actual `cohdl lsp`/`fmt`/`--json`.
- `00-root.md` still counts thirteen RFCs; newer pages say fourteen.
- `rfc-011-error-registry.md` records itself as DR-020 while note 7 says
  DR-009, and DR-020 is now RFC-014's dependency exception — two live
  DR-020s collide.

## External review round 4 (Codex, at b89b1b9): dispositions

Review 4 audited the RFC-015 IPC-2581 emitter (and reply3 residuals)
against commit b89b1b9. This repo is now at HEAD (717ccbc) with RFC-016,
RFC-017, and RFC-018 landed since — several findings were re-verified
against HEAD, where later work changed the picture. All twelve findings
were reproduced or contract-checked directly against HEAD; disposition:

Fixed in code (each with a named regression):

- **F3** — the XML escaper now applies the full XML 1.0 `Char` predicate:
  U+FFFE/U+FFFF (and any other forbidden scalar, not just C0) project to
  U+FFFD, and manufacturer `Enterprise/@id`s are built through a
  collision-free table over the post-projection value, so two vendors
  differing only in distinct control characters no longer alias one
  `enterpriseKey` (`src/emit/ipc2581.rs`; tests/ipc2581.rs
  `forbidden_scalars_projected_and_enterprise_ids_unique`).
- **F4** — the manifest package name is validated as an identifier
  (`valid_package_name`, `src/project.rs`) and a single-file basename is
  re-checked before any write, closing the `name = "../escaped"`
  write/delete traversal; a build without `--emit` deletes the stale
  `<name>.xml` ONLY when it carries the completeness marker (ownership
  established), and the `out/footprints/` cleanup removes only
  `.kicad_mod` files it owns (`src/main.rs`; tests/cli.rs
  `build_leaves_a_foreign_xml_untouched`, `traversal_package_name_is_rejected`).
  Transactional staging of the artifact set (atomic temp-file renames) is
  NOT implemented — a mid-write failure can still leave a mixed set; the
  completeness marker distinguishes ours, and this is recorded as a known
  limitation rather than claimed fixed.
- **F5** — the BOM/AVL identity is now `(MPN, manufacturer)` in BOTH the
  CSV and IPC emitters, so two parts sharing an MPN under different
  manufacturers keep their distinct rows/vendors/values (was: MPN-only,
  second vendor silently erased). MPN stays the primary sort key so the
  committed goldens are byte-unchanged. Empty required `mpn`/`mfr` values
  are rejected (E802) (`src/emit/bom.rs`, `src/emit/ipc2581.rs`,
  `src/check/generics.rs`; tests/ipc2581.rs
  `duplicate_mpn_across_manufacturers_keeps_both`).
- **F6** — `sanitize` now uses the ACTUAL vendored-XSD character classes
  (`<`/`>` are legal in both `qualifiedNameType` and `shortName` and are
  no longer stripped); the intersection is used for the BOM key, which is
  emitted in both a shortName and a qualifiedNameType slot. NOTE: the
  review's specific reproduction (a designator `R<1` diverging to `R_1`)
  is UNREACHABLE at HEAD — E804 rejects any designator that is not
  `[A-Z]+[0-9]+` at the source, so `<`/`>` never enters a designator to
  diverge. The test pins that source-side guarantee (tests/ipc2581.rs
  `designator_special_chars_are_rejected_at_source`).
- **F7** — the fidelity battery now checks `Component/@part` (non-empty),
  `@packageRef` resolution, `COHDL_DEVICE` presence, and — crucially —
  that every `PinRef/@componentRef` resolves to a real `Component/@refDes`.
  The false emitter/schema comments (the vendored XSD's `componentKeyRef`
  binds `LogicalNetPin`/`RefDes`, NOT `PinRef`, so PinRef agreement is the
  emitter's to guarantee, not the schema's) are corrected in place
  (`src/emit/ipc2581.rs`; tests/ipc2581.rs
  `component_attributes_and_pinrefs_are_semantically_faithful`).
- **F8** — `load_std_files` now returns `None` (a real error, surfaced as
  `window/showMessage`) for an existing-but-empty std directory, matching
  `load_project`; the LSP phantom-buffer fallback no longer publishes a
  false-clean `[]` (`src/project.rs`; tests/lsp.rs
  `empty_std_with_phantom_buffer_shows_message`).
- **F9** — pin use-site hover resolves `(device, selected variant)` and
  calls `pins_for` instead of scanning the first pin block, so a part
  bound to one variant shows that variant's physical pad (`src/lsp.rs`;
  tests/lsp.rs `pin_hover_respects_selected_variant`).
- **F10** — `initialize`/`shutdown` bind the request id BEFORE mutating
  lifecycle state (a notification can no longer consume init or shut the
  server down); the transport validates the JSON-RPC envelope
  (`jsonrpc == "2.0"` + string `method`, InvalidRequest otherwise);
  positions use checked `u32::try_from` (out-of-range → InvalidParams, no
  wrap); `localhost` matching is case-insensitive (`src/lsp.rs`;
  tests/lsp.rs `initialize_notification_does_not_consume_lifecycle`,
  `bad_jsonrpc_envelope_is_invalid_request`,
  `out_of_range_position_is_invalid_params`).
- **F11** — `unit_literal_hover` now scans `FnDef.generics` defaults, so
  hover on `fn f<V: Voltage = 3.3V>` works (`src/lsp.rs`; tests/lsp.rs
  `hover_on_fn_generic_default_literal`). The `Stmt::Layout` tolerance
  hover is NOT added — tolerance is stored as opaque source text (RFC-013),
  not a `UnitValue`, so there is no literal node to attach a hover to;
  noted rather than silently skipped.
- **F12.1** — the error-registry comment stripper recognizes Rust raw
  strings (`r#"…"#`) so a `Diagnostic::error("E999")` inside one is not a
  phantom call site (tests/error_registry.rs `raw_strings_are_not_call_sites`).
- **F12.3** — README updated: eighteen RFC areas (RFC-001…018), and the
  build-artifact list now names `out/footprints/*.kicad_mod` and the
  `--emit ipc2581` document.
- **F12.5** — a duplicate `--emit` flag is rejected rather than silently
  last-one-wins (`src/main.rs`; tests/cli.rs `duplicate_emit_flag_is_rejected`).

Documented / classified (not code):

- **F1 — the document is not yet a usable Quilter handoff (real-partner
  gate OPEN)**: recorded above in the RFC-015 notes and in docs/ipc2581.md.
  RFC-018 added real footprint geometry, but there is still no board
  outline/`Profile` and every component is at `(0,0)`; no real import pass
  has been recorded. This is an RFC-015 acceptance blocker, not claimed
  closed.
- **F2 — constraint mapping is a vendor extension, not native IPC
  semantics**: the earlier ledger wording was corrected in place (a
  compliance report cannot narrow Accepted text). Listed as a note-side
  amendment item below.
- **F4 (transactional staging)** — not implemented; known limitation, above.
- **F6 (designator divergence)** — unreachable at HEAD (E804); the sanitize
  charset was still corrected to the true XSD set as defense-in-depth.
- **F11 (layout-tolerance hover)** — not applicable (opaque text storage).
- **F12.2** — `docs/compliance-audit.json` now carries a leading
  `_historical_snapshot` marker: it is the frozen 2026-07-13 audit; the
  report is authoritative, and its D003 entries predate the 2026-07-14
  role-aware change.
- **F12.4** — the spec's copad/cofp inconsistency was already resolved
  note-side (snapshot refreshed at 717ccbc); confirmed pad/footprint.
- **F12.6** — the false PinRef-keyref schema comment is corrected (F7).

Note-side amendment item added by review 4 (needs a conol.ai edit):

- **RFC-015 constraint mapping (F2)**: RFC-015 (`rfc-015:42`) and the
  language spec (`10-language-specification.md:471`) promise RFC-013
  constraints mapped into IPC-2581's *native* constraint/net-class
  elements. IPC-2581B1 has no such user-named element, so the
  implementation emits `cohdl:`-prefixed `Spec`/`Property` vendor
  extensions + a `COHDL_PLACEMENT_HINT` attribute. Either RFC-015/spec/
  DR-021 should be amended to state these are proprietary vendor extensions
  requiring a CoHDL adapter, or a partner-recognized native mapping must be
  found and tested in that consumer.

## External review round 5 (Codex, at 041b2fa): dispositions

Review 5 audited the combined RFC-001–018 implementation (and reply4)
against `041b2fa`. Thirteen findings (R5-1…R5-13). Nine fixed in code with
named regressions; the rest are honest deferrals/documented deviations
recorded here. State after fixes: 283 tests passing.

Fixed in code (regressions named):

- **R5-2 (high) — false-clean unresolved names.** An unresolved qualified
  path in an UNCALLED function passed with verdict `pass`, because the
  unknown-name diagnostic came only from expansion, which never runs on a
  dead fn. The rewrite pass now diagnoses every instance-type/call-target
  that resolves to nothing — over EVERY body, called or not (E202/E504 with
  suggestions). Design-body cases reported by both the rewrite pass and
  expansion are collapsed by new exact-duplicate diagnostic dedup in
  `Diagnostics::sort`. (tests/modules.rs
  `unresolved_reference_in_uncalled_fn_is_caught`,
  `unresolved_reference_is_reported_once`.)
- **R5-3 (high) — unspellable/non-injective module identities.** A
  subdirectory or nested-file segment that is a reserved keyword
  (`src/device/`) or not an identifier (`src/power-supply/`) indexed
  declarations at an identity no qualified path can spell. Such segments are
  now rejected (E210) at project load. (tests/modules.rs
  `keyword_directory_is_unspellable_e210`,
  `hyphenated_directory_is_unspellable_e210`.) The cross-PACKAGE injectivity
  half (`acme-parts` vs `acme_parts` merging) is unreachable in the current
  single-project loader — it becomes reachable only with R5-1 (on-disk
  dependency loading), and is deferred with it; package NAMES keep the
  lenient rule (the examples' `rpi-pico2`/`sensor-node` names appear only in
  the artifact basename and internal keys, never spelled, since intra-package
  references are unqualified).
- **R5-4 (high) — pad/device check bypassable.** The check iterated only
  instantiated IR nodes, so an unused part with a mismatched non-empty
  footprint passed. It now walks `world.parts` (declaration-complete), so an
  unused part is caught. **PHASE CORRECTED by R6-5**: it runs at BUILD (not
  the check phase this entry originally moved it to) — RFC-018 pins the
  pad/device comparison to `cohdl build`, and moving it to `check` created a
  contract contradiction against the RFC and the error registry. The
  empty-placeholder exemption is RETAINED as a documented deviation from
  RFC-018's exact-match rule (RFC-017 stage-one migration); ending it is a
  migration-completion decision noted below.
- **R5-5 (high) — geometry overflow panic.** A parser-accepted huge `Length`
  reached `emit::geom` corner arithmetic (`femto * 10`) and panicked on
  `i128` overflow, after ordinary artifacts had been written. `Length` values
  are now bounded (`MAX_GEOM_FEMTO`, `length_in_geom_range`) and rejected at
  pad/footprint validation BEFORE any write (E805/E806); the corner helpers
  also use saturating arithmetic as defense-in-depth. (tests/footprint.rs
  `oversized_length_is_diagnosed_not_panicked`.)
- **R5-6 (high) — destructive foreign-file cleanup.** The build removed
  every `.kicad_mod` under `out/footprints/` before writing — an extension
  is not proof of ownership. It now removes only files carrying the
  `(generator "cohdl")` marker, preserving a foreign `.kicad_mod` exactly as
  a foreign `.xml` (tests/cli.rs `build_preserves_foreign_kicad_mod`).
- **R5-7 (high) — supply-chain grouping loses data.** (a) A duplicate
  singleton AVL field (`mpn`/`mfr`) silently kept the first via
  `AvlEntry::field`; it is now E802 with both spans. (b) Two parts sharing
  `(manufacturer, MPN)` but describing different components (device/binding/
  footprint) are rejected (E802) at declaration — one part number names one
  component, and the lossy `(MPN, mfr)` grouping would hide the disagreement.
  (tests/library.rs `duplicate_avl_field_is_rejected`,
  `inconsistent_parts_sharing_mfr_mpn_are_rejected`,
  `consistent_parts_sharing_mfr_mpn_are_allowed`.) The compliance-report
  "last-wins" note was wrong (it was first-wins); moot now — duplicates are
  an error.
- **R5-9 (medium) — `#[doc]` path invariant unenforced.** A doc path is
  package-relative (RFC-017); absolute, `..`-escape, empty, and URL forms
  are now rejected lexically at parse (existence stays a deferred lint).
  (tests/library.rs `doc_paths_must_be_package_relative`.)
- **R5-11 (medium) — JSON-RPC id type.** An object/array/bool id was treated
  as a request. The transport now validates the id is string/number/null,
  returns InvalidRequest (with a null response id) otherwise, and does not
  mutate lifecycle. (tests/lsp.rs `object_id_is_invalid_request`.)
- **R5-12 (medium-low) — `br#"…"#` byte strings.** The registry scanner
  recognized `r#"…"#` but not raw BYTE strings; extended to the `br` prefix.
  (tests/error_registry.rs `raw_strings_are_not_call_sites`.)

Documented / deferred (not code, or scope decisions):

- **R5-1 (high) — on-disk dependency loading is NOT implemented; RFC-016/017
  scope amendment requested.** The resolver handles arbitrary package sets
  (the tests drive it directly), but `load_project` still assembles exactly
  project + std — a `[dependencies]` section is ignored, so cross-library
  use through the real CLI/LSP is unreachable. Implementing a minimal
  local/path-dependency loader (manifest shape, recursive load, cycle/dup/
  missing detection, shared CLI+LSP package set) is the correct fix and is
  tracked as the next major RFC-016/017 work item. Until it lands, the
  library registry must NOT be described as usable end-to-end: this is a
  formal scope deferral, requested as a note-side amendment to RFC-016/017
  and the living spec, not a claim of completeness. (Publishing/hosting/
  version selection remain separately deferred by the RFCs themselves.)
- **R5-8 (medium-high) — IPC footprint geometry omits authored fields.** The
  IPC `Pin` subset carries shape/size/location/plating-type but NOT the
  pad's copper `layer`, its `drill`, or the footprint's `silkscreen_ref`
  (all present in the `.kicad_mod`). docs/ipc2581.md and the emitter header
  now state exactly what is omitted, and the marker governs; "real footprint
  geometry" is narrowed to "partial." Projecting the omitted fields needs
  the schema's `LandPattern`/`PadStackDef`/hole structures — future work,
  now disclosed rather than silently dropped.
- **R5-10 (medium) — module diagnostic/navigation polish.** Import goto-def
  (definition on a `use` path), FQ display when short names are non-unique
  (`f → f → f`, `(P, P)`), and kind/visibility-aware suggestions remain
  tooling gaps. Recorded as open RFC-016 human-reviewability items; not
  soundness. (Internal identity is correct; only the human projection loses
  information.)
- **R5-13 (medium) — Length/spec/formatter contract drift.** `Length` as
  RFC-001's eleventh unit type is an implied amendment (docs/compliance-
  report RFC-018 notes); the living spec and RFC-001 in `docs/design/` still
  declare ten (those are extraction-only snapshots — the amendment is a
  note-side conol.ai edit, requested). docs/lsp.md's capability table was
  updated to include the RFC-017/018 part-doc, footprint, and pad-placement
  hover/navigation. The formatter "aligned columns" sentence for footprint
  pad placements is not implemented (one-per-line, single-space); recorded
  as a note-side amendment item (implement alignment or amend the RFC) —
  a ledger cannot downgrade Accepted text, so this is filed, not dismissed.

Note-side amendment items added by review 5:

- Local on-disk dependency loading (R5-1): implement, or formally scope
  RFC-016/017 + spec to defer it (the library registry is not usable
  end-to-end until then).
- `Length` unit (R5-13): formal RFC-001/DR amendment + atomic update of
  every ten-type reference in the living spec/DRs/README.
- Formatter pad-placement column alignment (R5-13): implement or amend RFC-018.
- Empty-footprint placeholder exemption (R5-4): decide how the RFC-017
  stage-one migration ends — a build consuming an empty footprint currently
  passes silently rather than signalling incompleteness.

## External review round 6 (Codex, at 3f3ee20): dispositions

Review 6 found that several review-5 fixes were too close to the original
reproductions — the same defect classes survived in deeper forms. All ten
findings verified; nine fixed in code with adversarial regressions, one
(R6-10) is this consistency pass over the docs. State: 296 tests passing.

Fixed in code (regressions named):

- **R6-1 (high) — foreign-file overwrite + symlink escape.** Review-5's
  cleanup preserved a non-colliding foreign `.kicad_mod`, but the WRITE loop
  still overwrote a foreign file at the exact generated name and followed a
  symlink there to mutate a file outside `out/`. A single ownership-and-
  symlink-aware writer (`write_artifact`, src/main.rs) now backs every
  artifact write: it refuses a symlink destination (containment, via
  `symlink_metadata` + `create_new`/O_EXCL), and refuses to overwrite an
  existing regular file lacking the CoHDL ownership marker. (tests/cli.rs
  `build_refuses_exact_name_foreign_and_symlink`; the review-4 XML test was
  updated — `--emit` over a foreign file is now refused, not clobbered.)
- **R6-2 (high) — non-injective supply-chain identity.** The same-MPN guard
  compared `short()` names and raw generic text, so two parts with distinct
  fq devices sharing a leaf name (and different values) collapsed and the
  BOM/IPC kept the first value. It now compares FULLY-QUALIFIED device and
  footprint identities and NORMALIZED unit values (femto + unit, so `1kohm`
  ≡ `1000ohm`), over EVERY AVL entry (primary and alts). Because an
  inconsistency is now a check error and `build_artifacts` bails on check
  errors, the lossy emitter grouping never sees an inconsistent group.
  (tests/library.rs `same_mpn_distinct_fq_devices_are_rejected`,
  `equivalent_unit_spellings_are_the_same_component`,
  `alt_entry_mpn_conflict_is_checked`.)
- **R6-3 (high) — uncalled fn bodies unchecked.** Review-5 checked only that
  instance/call targets EXIST. A new pass (`check::bodies::check_fn_bodies`)
  semantically validates every fn body regardless of reachability: wrong-kind
  instance targets (trait/fn/pad → E205), wrong-kind call targets
  (device/part → E205), unresolved named generic arguments (E202), and
  net/nc references to unknown locals (E202). Messages mirror expansion's, so
  a called fn reported by both dedups. Full generic BOUND-checking stays a
  call-time concern (abstract fn generics have no concrete value at
  declaration). (tests/modules.rs `uncalled_fn_body_is_semantically_checked`,
  `valid_uncalled_fn_body_passes`.)
- **R6-4 (high) — package-root spellability.** E210 validated only `src/…`
  module segments. It now also validates the projected PACKAGE ROOT (a
  keyword manifest name like `device` is unspellable in a single-package
  project today) and `std/…` segments of a supplied std tree. (tests/modules.rs
  `keyword_package_root_is_e210`, `keyword_std_segment_is_e210`.)
- **R6-5 (medium) — E807 phase/cascade/API.** (a) The pad/device check moved
  BACK to the BUILD phase, restoring RFC-018's Accepted build-only contract
  (the check-phase move contradicted the RFC and registry); it keeps the
  declaration-complete `world.parts` walk. (b) It now SKIPS a part whose
  variant selection is structurally invalid, so a bad `[VARIANT]` selector no
  longer fabricates a spurious "extra pad" E807 on top of the real E903/E904.
  (c) Both declaration APIs (`check_declarations`/`check_declarations_in`)
  route through one `run_declaration_checks`. (tests/footprint.rs
  `invalid_variant_selection_does_not_cascade_to_e807`; the R5-4 test builds.)
- **R6-6 (medium) — doc-path grammar.** The lexical check accepted drive
  roots (`C:/…`), single-slash and non-`://` URI schemes (`file:/…`,
  `mailto:`, `data:`), and backslashes. It now rejects a backslash, any
  scheme/drive marker (a `:` in the first path segment), leading separators,
  `..` components, and empty paths. (tests/library.rs
  `doc_paths_reject_drive_roots_and_uri_schemes`.)
- **R6-7 (medium-low) — null id vs absent.** A malformed envelope carrying an
  explicit `"id": null` was dropped as a notification. The transport now
  responds to any frame with an `id` FIELD present (a request, even with a
  null id) — InvalidRequest with a null response id — while a true
  notification (no id) with a bad envelope is still dropped. (tests/lsp.rs
  `malformed_request_with_null_id_gets_response`, `malformed_notification_is_dropped`.)
- **R6-8 (medium-low) — raw C strings.** The registry stripper recognized
  `r#`/`br#` but not `cr#` (raw C string). Extended to the `cr` prefix.
  (tests/error_registry.rs `raw_strings_are_not_call_sites`.)
- **R6-9 (low/medium) — silkscreen range check.** `silkscreen_ref`
  coordinates skipped the `length_in_geom_range` bound every other geometry
  Length gets. Now routed through it. (tests/footprint.rs
  `oversized_silkscreen_ref_is_rejected`.) A full accepted-boundary matrix
  across every corner path remains a nice-to-have, noted.

Documentation consistency (R6-10):

- README no longer says RFC-008–018 are flatly "all implemented" — it names
  the open gaps (RFC-016/017 dependency loading, RFC-014/015 acceptance
  passes) inline; "ten unit-type literal forms" → eleven.
- docs/error-codes.md E102 now names both `Temperature` AND `Length` as
  signed.
- docs/lsp.md narrows `definition` to REFERENCE use sites and records that
  goto-def on a `use`-import path is an open R5-10 item.
- The R5-4 ledger entry's "CHECK phase" wording is corrected to build-only
  (R6-5), removing the both-contracts-asserted contradiction.

Still open (unchanged from review 5, restated honestly):

- R5-1 on-disk dependency loading (the compiler half of RFC-016/017 usability).
- R5-8 IPC layer/drill/silkscreen projection (documented omission).
- R5-10 import goto-def, collision-aware FQ display, kind/visibility-aware
  suggestions; the RFC-017/018 LSP doc-exposure gaps (device-declaration doc
  hover, footprint declaration location, footprint-reference hover, FQ-keyed
  nested-design docs).
- R5-13 note-side amendments: `Length` in RFC-001/spec, formatter pad
  alignment, empty-placeholder migration end-state.
- Transactional artifact staging; the real-VS-Code and real-IPC-consumer
  acceptance passes.

## External review round 7 (Codex, at 4122a33): dispositions

Review 7 found the review-6 fixes still split across shallow mechanisms —
final-filename vs directory traversal, the fn-body pass vs expansion, written
generic syntax vs resolved substitution, malformed-envelope vs method
dispatch, one registry scan direction vs the other. Nine findings; all fixed
or extended in code with regressions, plus honest scoping on the two that are
larger architectural items. State: 308 tests passing.

- **R7-1 (high) — containment + ownership.** The writer checked only the
  final path component and opted net/BOM/lock/layout out of ownership. Now:
  (a) `ensure_contained` refuses to build into an `out/` (or `out/footprints/`)
  reachable through a symlinked ancestor — a planted `out -> ../victim` no
  longer lets a build escape the project; (b) a per-build GENERATED-FILE
  MANIFEST (`out/.cohdl-manifest`, project-relative, sorted) is the single
  ownership primitive for EVERY artifact — `write_artifact` refuses to
  overwrite a file not in the prior manifest, and the stale sweep removes only
  manifest files, safely (`remove_owned` never follows a symlink to its
  target). `design.lock` is exempt (format-validated as `prior_lock` at build
  start, and committed for designator stability). The example goldens carry a
  committed manifest; a pre-manifest project's first build refuses its old
  outputs (delete `out/` once) — the SAFE migration. (tests/cli.rs
  `build_refuses_symlinked_out_dir`, `build_refuses_foreign_net_and_bom`,
  `build_manifest_enables_reownership`, plus the R6-1 exact-name/symlink
  tests.) Transactional staging across a mid-build refusal remains open.
- **R7-2 (high) — fn-body semantics.** The pass was extended well beyond
  existence: concrete-device pin existence (E203/E602), a unit-typed generic
  used as an instance type (E205), call arity (E502) and argument bases,
  device generic arity (E401) + concrete unit-literal type mismatch (E112),
  and missing structural-variant selectors (E904) — all now caught in an
  UNCALLED fn. (tests/modules.rs `uncalled_fn_body_deeper_semantics`.) Stated
  honestly (and the module doc no longer overclaims): this is NOT yet the
  single unified semantic checker shared with expansion — bound satisfaction
  over abstract fn generics, layout-constraint arity in fn bodies,
  duplicate-local and call-graph-cycle detection remain expansion's at call
  time. That unification is the deferred architectural item.
- **R7-3 (medium) — resolved identity.** The same-MPN signature now resolves
  generic DEFAULTS (so `R` and `R<1kohm>` with `V = 1kohm` are one component)
  and uses the EFFECTIVE footprint (an `alt` that omits `footprint:` inherits
  the primary's, not compared as empty). (tests/library.rs
  `default_equivalent_generics_are_the_same_component`,
  `omitted_alt_footprint_inherits_primary`.) A structured typed
  `ComponentIdentity` (vs a formatted string) and a defensive emitter-boundary
  assertion remain nice-to-haves.
- **R7-4 (medium) — E210 anchoring.** The package-root error is anchored to
  the first PROJECT source, not a compiler-owned `std/` file that loads first.
  (tests/modules.rs `keyword_package_root_anchors_to_project_file`.) The LSP
  false-clean-publish concern is noted; a dedicated manifest-error surface
  (`window/showMessage`) is a follow-up.
- **R7-5 (medium-low) — doc-path grammar.** A component-based grammar now
  rejects `./`, `..`, empty components (`docs//x`, trailing `docs/`), and a
  scheme/drive after `./` normalization — not just direct forms.
  (tests/library.rs `doc_paths_reject_dot_slash_and_empty_components`.)
- **R7-6 (medium-low) — method-shape id.** The transport classifies each
  method as request or notification and validates id presence: a
  notification presented with an id (including `exit`) gets InvalidRequest and
  does NOT perform the action; a request without an id is dropped.
  (tests/lsp.rs `notification_with_id_is_invalid_request`,
  `exit_with_id_does_not_terminate`.)
- **R7-7 (medium-low) — registry scan.** Both directions now share ONE
  literal-aware view (`strip_comments`, raw-string aware including `cr#`), so a
  code inside inert raw prose creates no obligation in EITHER direction.
  (tests/error_registry.rs `codes_in_text_ignores_raw_literals`.)
- **R7-8 (low) — variant guard.** The guard now skips the pad comparison for
  ANY ill-formed variant selection (including a selector on a non-variant
  device → E905), and a regression exercises the checker DIRECTLY (the branch
  the normal pipeline never reaches because E90x already blocks the build).
  (tests/footprint.rs `selector_on_non_variant_device_does_not_cascade_e807`.)
- **R7-9 (low/medium) — comment/name sweep.** Live source drift updated:
  E105/E210 registry rows, the footprints module comment (build phase),
  lex.rs and units.rs signed-unit comments, and the unit test names
  (`parses_all_unit_types`, `rejects_negative_except_signed_types`). The
  normative `docs/design` `Length` amendment stays separately open note-side.

Still open (unchanged): the fn-body/expansion unification (R7-2), transactional
staging (R7-1), R5-1 dependency loading, R5-8 IPC geometry fidelity, R5-10 LSP
polish + the LSP manifest-error surface (R7-4), R5-13 note-side amendments, and
the real editor/consumer acceptance passes.

## RFC-019 (VS Code extension) implementation notes (2026-07-15)

Implemented per DR-025: a real, buildable, installable VS Code extension at
`editors/vscode/`, packaging the already-Accepted `cohdl lsp` (RFC-014).
Closes RFC-014's two explicitly-deferred items — the marketplace-extension
packaging/grammar scope, and (partly) the real-client acceptance gate.

What shipped:

- **TextMate grammar** (`syntaxes/cohdl.tmLanguage.json`) registering `.cohdl`
  for syntax highlighting — a static capability the LSP protocol has no verb
  for, hence a genuinely separate artifact. Scope coverage is hand-authored
  from the Accepted grammar (RFC-001…018): keywords, pin roles, unit type
  names, unit literals (one regex class), attributes, strings, comments.
- **Client wiring** (`src/extension.ts`) — a thin `vscode-languageclient`
  spawn of `cohdl lsp`, identical in shape to the `docs/lsp.md` snippet, with
  the one new `cohdl.path` setting (default `"cohdl"`, PATH-resolved) and a
  VISIBLE activation-failure notification (RFC-019 Failure modes: a missing
  binary must never be a silent blank Problems panel). ZERO new diagnostic
  logic — the output is exactly `cohdl lsp`'s.
- **Grammar-coverage regression test** (`test/grammar.test.mjs`) — tokenizes a
  representative fixture against the committed grammar via the real
  `vscode-textmate`/`vscode-oniguruma` engine and asserts every keyword /
  literal-class token gets a scope (36 token classes; catches a keyword
  falling through to plain text). Tooling-repo CI only, NOT inside
  `cohdl check`/`cohdl build`.
- **CI job** (`vscode-extension` in `.github/workflows/ci.yml`) — npm install,
  `tsc` compile, grammar test, and `vsce package` the `.vsix`, on the same
  workflow as the Rust gate.

Honest boundaries / deviations:

- **The dependency-free constitution is a COMPILER constraint, not an editor
  one.** `editors/vscode/` necessarily uses `vscode-languageclient` (client)
  and, for its own test, `vscode-textmate`/`vscode-oniguruma`. These are
  confined to the extension package; the Rust crate (pipeline + emitters) and
  the DR-020 LSP-layer scope are untouched. This is consistent with RFC-019's
  "pure Layer-4 tooling" classification — but recorded explicitly so the
  zero-deps claim's scope stays precise.
- **The live-VS-Code acceptance pass is still a HUMAN checkpoint** (like the
  KiCad checkpoint, docs/demo). The extension builds, packages to a `.vsix`,
  and its grammar coverage is machine-tested; but a person actually installing
  it and exercising diagnostics/hover/goto in a live editor is not something
  this environment can run headlessly. RFC-014's real-client acceptance item
  is therefore MOVED from "no path to try it" to "packaged and buildable,
  awaiting a recorded human session" — a genuine narrowing, not a closure.
  Tracked here and in docs/lsp.md.
- **Grammar drift** is the disclosed maintenance risk (RFC-019 Failure modes):
  the TextMate grammar is hand-authored, so a future keyword-adding/renaming
  RFC must update `cohdl.tmLanguage.json` in the same change. The coverage
  test catches a dropped keyword (plain-text fallthrough) but not a
  mis-classified one; grammar review stays a human step. No compiler-enforced
  guarantee is possible for an external editor's grammar file.
- **No package-lock.json committed** (gitignored) — the CI job runs
  `npm install` against the pinned `^`-ranges. A committed lockfile for fully
  reproducible extension builds is a reasonable future tightening.

## rpi-pico2 Quilter-deliverability: real footprints + board outline (2026-07-15)

Goal: make `examples/rpi-pico2`'s `build --emit ipc2581` a document a layout
partner (Quilter) can actually consume. Quilter needs three things — a single
closed board outline, pre-loaded component footprints (it does not manage
footprints), and a valid netlist. The netlist was already faithful (RFC-015);
this change adds the other two.

What landed:

- **Real footprint geometry for every rpi-pico2 footprint.** All ~14 board
  footprints (`FP_QFN_60…`, `FP_Pico_Castellated_40`/`_3`, `FP_WQFN_10…`,
  `FP_USON_8…`, `FP_USB_Micro_B_Wuerth…`, `FP_Crystal_SMD_3225…`, `FP_SOT_523`,
  `FP_D_SOD_123`, `FP_SW_SPST_TL3342`, `FP_L_0806…`, `FP_R_0201…`, `FP_C_0201…`,
  `FP_C_0805…`) plus the board-used std `FP_LED_0603…` moved from RFC-017
  stage-one placeholders (empty body) to RFC-018 stage-two authored pad
  geometry. The std library also gained real bodies for `FP_C_0603…` and
  `FP_SOT_23_5`. Pad symbols live in an expanded `std/pads.cohdl` (19 pads).
  The IPC document now carries 160 `Pin` elements across 17 `Package`s — one
  per device pin, every footprint E807-consistent with its bound device — up
  from 4 pins on 2 footprints.
- **Board outline** — a new `board_outline { at: (cx, cy), size: (w, h) }`
  statement inside the design's `layout {}` block, projecting to the IPC-2581
  `Step/Profile` (a single closed rectangular polygon, ordered Datum < Profile
  < Package per the XSD sequence) and to `layout.json`. rpi-pico2 declares the
  51×21 mm Pico-2 perimeter.
- **R5-8 narrowed** — pad plating now rides `Pin/@mountType`
  (`SURFACE_MOUNT_PAD`/`THROUGH_HOLE_HOLE`). Copper layer and exact drill
  diameter still have no home on the B1 `PinType` (they need a Step-level
  `PadStackDef`); moot for this all-SMD, single-layer board.

Checks/tests: `E1006` (board-outline geometry — non-Length, non-positive
extent, out-of-range, duplicate, or inside a called `fn`) registered in
docs/error-codes.md; `tests/layout.rs` gains 8 board-outline cases
(layout.json projection, `null` absence, zero-impact on netlist/BOM/verdict/
lock, the four E1006 failure modes, fmt round-trip); `tests/ipc2581.rs` gains
Profile-emission, no-Profile-when-absent, and mountType cases and continues to
schema-validate the real rpi-pico2 output against `IPC-2581B1.xsd`.

Honest boundaries / deviations:

- **`board_outline` is a pragmatic extension, NOT an Accepted-RFC construct.**
  RFC-013's `layout {}` vocabulary is net-level (net_class/diff_pair/
  length_match); RFC-015 and RFC-018 both explicitly deferred board outline /
  stackup as named future work. This admits a board-level rectangular outline
  into the same block ahead of a governing RFC — the same "the door was opened
  ahead of its gate" posture RFC-013 itself was implemented under, per Tony's
  directive. An RFC on conol.ai supersedes this shape (e.g. polygon outlines,
  a dedicated board-level construct). Recorded here, not assumed permanent.
- **Footprint geometry is nominal, not fab-golden.** Where KiCad's official
  library ships the exact footprint named in a `was:` comment (the chip
  passives, SOT-523, SOD-123, SOT-23-5, Crystal 3225, QFN-60 lead grid), pad
  positions/sizes derive from it. Where it does not (the Wuerth micro-USB
  hand-solder land, USON-8 1.0×1.5, WQFN-10 2×2, the 0806 inductor, the Pico
  castellated headers), geometry is IPC-7351-nominal / datasheet-derived and
  authored here. Pad COUNT and NUMBERING are exact (E807 enforces match to the
  device); pad DIMENSIONS are a reasonable land pattern, not a
  manufacturer-verified one. RFC-018 already names dimensional accuracy as an
  unenforceable property; a real fab still reviews the library.
- **Outline shape is rectangle-only.** A closed rectangle is a valid Quilter
  seed; the true Pico-2 outline has rounded corners and a USB cutout. A
  polygon/arc outline is future work.
- **The partner handoff is still a HUMAN checkpoint.** "Deliverable to Quilter"
  means the document now carries outline + land patterns + netlist and is
  schema-valid — not that a Quilter round-trip has been run. Component
  placement remains Quilter's to perform; the `logical-complete,
  physical-minimal` marker still governs.

### Follow-up: component staging outside the outline (2026-07-15)

Loading the emitted document into Quilter surfaced a placement-model mismatch:
Quilter treats every component INSIDE the board outline as pre-placed/locked
and only places/routes the components left OUTSIDE it (docs.quilter.ai —
"prepare your input board file", "pre-placed components"). The
`physical-minimal` all-at-`(0,0)` placeholder therefore read as 49 components
locked at the board centre ("Components to Place: 0"), stacked in a blob.

Fix: `emit_ipc2581` now stages every component in a deterministic,
non-overlapping shelf-packed grid immediately to the RIGHT of the outline
(each component's full footprint bbox — pad extents ∪ courtyard — lies past
`outline_right + 5mm`), so Quilter treats all of them as placeable. Geometry
is exact over the femto integers (`emit::geom::mm_femto`), byte-stable, and
verified: for rpi-pico2 all 49 components sit fully outside the 51×21mm
outline with zero overlapping pairs. Designs with no `board_outline` keep the
`(0,0)` placeholder (nothing to stage against). This is still NOT a real
placement — it is the "unplaced, please place me" input a placement engine
consumes — so the `logical-complete,physical-minimal` marker is unchanged.
Tests: `tests/ipc2581.rs` gains staged-outside / origin-without-outline cases.

### Follow-up 2: locked placements + board-edge sizing (2026-07-15)

Second round of Quilter feedback (after component staging landed): the file
parsed, but two errors — "total component area exceeds the board" and "J1
dimensions exceed the board's dimensions". Root cause: `FP_Pico_Castellated_40`
(the 40-pin castellated header, J1) was (a) built rotated 90° so its ~50mm pad
span sat across the board's 21mm axis, and (b) board-sized — as a *movable*
component it alone ≈ the whole board area, so no placement fits.

Two fixes:

1. `FP_Pico_Castellated_40` re-authored — pads run along the two long (51mm)
   edges (pins 1-20 bottom, 21-40 top, 17.78mm row spacing), so J1 is
   ~50×20mm and fits inside the 51×21mm outline; its board-filling courtyard
   was dropped so the interior stays free for placement (the header is the
   board edge, not a solid keep-out).
2. New `place <inst> at (x, y)` layout statement (E1007) — a locked component
   placement. A placement tool treats a component positioned inside the
   outline as pre-placed/fixed, so `place` emits that component's
   `Component/Location` at the given point and excludes it from staging. The
   example locks the castellated header centred (`place hdr at (0mm, 0mm)`);
   the rest stage outside. Same pragmatic-extension status as `board_outline`
   (E10xx family, design-top-level only, projected into `layout.json` too).

Verified for rpi-pico2: J1 fits inside the outline and is locked; the 48
placeable components total 232mm² vs the 1071mm² board, all outside the
outline, none exceeding board dimensions, zero overlaps, schema-valid. This is
still not a real layout — it is a correctly-shaped placement-engine input.

## RFC-020 (board outline via DXF + oriented placement) implementation notes (2026-07-16)

RFC-020 retroactively formalizes — and corrects — the `board_outline` and
`place` constructs that were implemented ahead of any RFC (commits 86165d9,
1a0ce5f). The two corrections it mandates are now implemented:

1. **`board_outline` is DXF extraction, not a CoHDL-authored rectangle.** The
   surface syntax is `board_outline: "path.dxf"` (a project-relative path
   string, replacing the `{ at, size }` rectangle). At `cohdl build`, a narrow
   hand-rolled DXF parser (`src/dxf.rs` — pure, testable, zero-dependency,
   the same narrow-contract discipline RFC-018 set for pad geometry) extracts
   EXACTLY one closed `LWPOLYLINE`/`POLYLINE` on the `Edge.Cuts` layer;
   everything else in the file is ignored. Straight segments and arc bulges are
   both supported and translate directly to IPC-2581 `PolyStepSegment` /
   `PolyStepCurve` and to `layout.json`. New E1006 sub-cases name each failure
   (unreadable file, invalid DXF, no closed entity on the layer, not closed,
   too few vertices). The `pipeline::resolve_board_outline` step reads the file
   via an injected loader (the CLI reads the FS relative to the project root;
   tests pass a literal), keeping the pipeline FS-free.

2. **`place` gains an optional `rotate` (closed set {0, 90, 180, 270}).**
   Additive — every existing `place <inst> at (x, y)` is unchanged. The
   rotation rides IPC-2581 `Component/Xform/@rotation` and `layout.json`; an
   invalid value is a new E1007 sub-case. CoHDL performs no rotation math.

Migration (RFC-020's own required work): `examples/rpi-pico2` now references a
real DXF outline (`mechanical/pico2-outline.dxf`, a 51×21 mm rounded rectangle
— straight edges + four 90° corner arcs, exercising the arc path end to end)
and pre-positions its interface ports — the 40-pin castellated header centered,
the micro-USB and SWD headers at the short edges with `rotate 90`. The document
stays schema-valid against IPC-2581B1.xsd with the arc `Profile`.

Honest boundaries / deviations:

- **The DXF outline-layer convention is `Edge.Cuts`** (documented in
  docs/ipc2581.md). RFC-020 leaves the exact layer name to emitter docs; this
  matches KiCad's board-edge layer so a KiCad DXF export drops in.
- **Arc centers use `f64`.** The exact femto-rational center overflows i128, so
  the center a bulge implies is computed in `f64` and rounded to femto ONCE at
  extraction, then stored — both emitters read the same integer, so output
  stays byte-stable and the two emitters cannot disagree. Vertex coordinates
  themselves are parsed to exact femto (no float). Disclosed as the one float
  path, confined to arc-center derivation.
- **`place` reaches only top-level design instances** — unchanged from the
  original construct, and RFC-020 EXPLICITLY defers fn-nested placement (per
  Tony's direct decision; a real, named gap, not silently worked around). A
  board-edge component that needs a locked position must be instantiated at the
  design top level, not inside a reusable `fn`.
- **Not a general DXF parser** — only a single closed outline entity on one
  layer is ever read; self-intersecting-but-closed shapes are not validated
  (the mechanical engineer's / CAD tool's responsibility, RFC-020 Non-goals).
- **Rotation carrier — `Xform`, not `Location`.** RFC-020 / the spec say the
  rotation rides "IPC-2581's `Component/Location` rotation attribute", but the
  vendored B1 schema's `Location` (`PointType`) has NO rotation attribute — the
  schema's actual placement-transform carrier is `Component/Xform/@rotation`
  (`XformType`). The emitter uses `Xform` (schema-valid, `xmllint`-gated); the
  RFC's "Location rotation attribute" phrasing is read as "the component's
  rotation", faithfully. A note-side wording correction is the clean fix.
- **LSP hover for `board_outline` deferred.** RFC-020 Tooling says `cohdl lsp`
  hover on a `board_outline` statement *should* show the extracted bounding-box
  dimensions + resolved path. The LSP layer does no FS/DXF resolution today
  (the hover would have to read the file), so this is deferred and disclosed;
  the path is still shown by the generic string-literal hover. Tracked.
- **std needs no change** — `board_outline`/`place` are design-level layout
  constructs; the std library declares neither.

### Footprint mounting correction: through-hole USB + castellations (2026-07-16)

Tony flagged that some rpi-pico2 footprints were wrongly all-SMD when the real
parts are through-hole. A 17-footprint adversarial audit (one classifier + one
refuter per footprint, cross-checked against the KiCad official library) HIGH-
confidence-confirmed that 14 are genuinely SMD (QFN-60, WQFN-10, USON-8, SMD
crystal 3225, SOT-523, SOD-123, 0806 inductor, the 0201/0402/0603/0805 chips,
0603 LED, the SMD tactile TL3342 — leadless/chip packages with no drilled
holes; a QFN/DFN exposed thermal pad is still SMD). The three flagged as
through-hole:

- **Micro-USB (Würth 614105150721)** — the real receptacle is through-hole:
  KiCad's `..._Vertical_CircularHoles` land pattern is fully `thru_hole` (5
  signal pins drill 0.44mm + shield/mounting posts drill 1.35mm). Now 5 signal
  PTH + 4 shield/mount PTH pads. (The audit's automated verifier initially read
  the already-authored SMD footprint and called it "SMD, but a design-review
  risk — the real receptacle likely wants THT shell posts"; that circular
  self-check was overridden by the direct KiCad-library evidence.)
- **Pico castellations (40-pin header + 3-pin SWD)** — a castellation is a
  plated HALF-through-hole (drilled, plated, cut on depanel). Modeled as a
  plated through-hole (drill 1.0mm), matching KiCad's `RaspberryPi_Pico_Common_
  THT`. (KiCad also ships a fully-SMD `RaspberryPi_Pico_SMD` for the carrier-
  board view; the through-hole model is the faithful one for the board's OWN
  edge castellations.)

RFC-018 pads: `plating: plated_through_hole` + `drill:` + `layer: through_all`.
The `.kicad_mod` projection now carries `thru_hole … (drill …)`; the IPC-2581
logical `Pin` carries `type="THRU"` + `mountType="THROUGH_HOLE_PIN"`, while
the exact drill lives in `PadstackHoleDef` and a located drill-layer `Hole`.
E807 still holds (pad numbers unchanged), and the document stays schema-valid.

## RFC-021 (IPC-7351 canonical footprint naming) implementation notes (2026-07-16, twice-revised)

RFC-021 was revised twice the same day (DR-027). Final design: a `footprint`'s
OWN identifier — the same name RFC-016's module system resolves — IS its
IPC-7351B land-pattern designator, when the package prefix is in the closed
six-family set. There is NO separate `ipc_name` field (first revision removed
it), and NO third-party-CAD-tool footprint-name mapping construct of any kind
(second revision: CoHDL does not reference, import, or track KiCad/LCEDA/Allegro
footprint names — every footprint is CoHDL's own native RFC-018 geometry; the
name is that declaration's own identifier and nothing else).

- **Grammar** (`src/check/ipc7351.rs`, pure + unit-tested): a closed six-family
  parser — QFP, QFN (incl. SON/VQFN), SOIC/SOP, SOT, BGA, CHIP/MELF — density
  suffix a closed set `{N, L, M}`, dimensions hundredths-of-mm integers
  (IPC-7351B's own convention). The one substitution: IPC-7351's `-` → `_`
  (CoHDL identifiers disallow `-`); the closed-family names use no other
  punctuation, so no other mapping is needed. The field ORDER genuinely differs
  between families (QFP puts pitch first, QFN puts pins+density first, etc.);
  each template is parsed literally as RFC-021's Design table specifies.
- **Two declaration-time checks** (E808/E809, in `resolve::validate_footprint_name`,
  operating on the footprint's own identifier, never DRC): E808 = a name in a
  closed family that is malformed (names the specific parse failure); E809 =
  name-vs-geometry mismatch — pin count (distinct pad-number count minus the
  `_1EP` exposed pad) and pitch (the closest spacing between distinct pad
  numbers, exact over the femto integers — the
  nearest pair in a regular perimeter is axis-aligned, so its squared distance
  is exactly pitch²). CHIP/MELF check the 2-pad shape only. A name whose prefix
  is OUTSIDE the closed set parses to `UnknownFamily` and is left as an ordinary
  RFC-016 identifier, unchecked (free-form).
- **fmt**: the identifier is rendered verbatim (it is the name) — there is no
  longer any `ipc_name` field to reorder; formatting is idempotent.

Applied to the covered footprints — their identifiers ARE their IPC-7351 names,
each consistent with its own pad geometry: the three QFN-family parts
(`QFN60N40P700X700_1EP350X350` for the RP2350, `QFN10N40P200X200_1EP90X150`,
`QFN8N40P100X150_1EP40X120`), the two SOT parts (`SOT3P100X160X80N`,
`SOT5P95X290X160N`), and the CHIP passives (`CHIP_0201`/`CHIP_0402`/`CHIP_0603`/
`CHIP_0805`). Because CoHDL's IPC-7351 name captures only the nominal EIA land,
the former distinct `R_0201`/`C_0201` and `R_0402`/`C_0402` footprints (which
differ only in trivial per-part land tweaks) collapse into a single `CHIP_0201`
and `CHIP_0402` each — one name, one land.

Honest boundaries / deviations:

- **Uncovered families keep free-form `FP_*` names** (allowed — outside the
  closed six-family set the ordinary RFC-016 identifier grammar governs): the
  SMD crystal (3225), the SOD-123 diode, the 0806 inductor (0806 is not a
  standard EIA code), the micro-USB, the Pico castellated headers, the tactile
  switch, and the LED (RFC-021 scopes CHIP to resistors/caps; an 0603 LED is a
  chip but outside that stated scope). Extending the closed family set is the
  RFC's own named scoped follow-up.
- **No third-party-footprint mapping construct** — the second revision
  explicitly removed the `footprint_alias`-style backend-name idea. CoHDL has
  no `footprint_alias` / `kicad:` / `lceda:` / `default:` construct and this RFC
  introduces none; there is nothing to reconcile against an external library.
  The `// was: "…"` provenance comments in the footprint sources record only the
  package/dimensions the CoHDL-native geometry was derived from as reference
  data (permitted by the RFC) — they are comments, not a tracked mapping.
- **Irregular-layout downgrade-to-note not implemented** — RFC-021 says a
  footprint with a genuinely irregular pad layout should get grammar-checking
  only, with the geometry check emitted as a note rather than an error.
  Detecting "irregular" reliably is unimplemented; the geometry check runs
  strictly (a QFN/SOT/QFP name whose pins/pitch disagree with the pads is E809,
  not a note). Stricter, not looser; disclosed. Moot for the covered footprints
  (all regular).
- **LSP hover deferred** — RFC-021 Tooling says hover should surface the parsed
  family/pitch/span/density of a footprint name; the LSP layer doesn't yet parse
  the identifier for hover. Deferred + disclosed.

## IPC-2581 physical model — padstacks + placed copper (2026-07-16)

Finding `.co/invalid-ipc2581.xml` (confirmed, high): the emitted IPC-2581 was
XSD-valid and semantically faithful (components, nets, placements, outline all
agreed with the KiCad board), but carried only ABSTRACT `Package/Pin` land
patterns — no `PadStackDef`, no `LayerFeature/Pad`, no real layers/holes. So
Quilter parsed it but showed only dark package courtyards: no copper pads,
drills, references, or ratsnest. It also correctly flagged that the R5-8
"drills are moot (all-SMD board)" note was stale (the board now has 52
through-hole pads).

Fixed by emitting the full physical model in `src/emit/ipc2581.rs` (validated
against `tests/schema/IPC-2581B1.xsd`, and structurally against KiCad's own
`kicad-cli pcb export ipc2581 --version B` output as the reference):

- **`DictionaryStandard`** (Content) — one exact primitive per unique geometry:
  `RectCenter`/`Circle`/`Oval`, `RectRound` for a four-corner radius, or
  `Contour` for a one-corner chamfer. Copper, expanded mask, and reduced paste
  may therefore reference different primitives.
- **Real layers + stackup** (CadData) — the full nine-row top-to-bottom
  silk/paste/mask/copper/dielectric/copper/mask/paste/silk sequence, plus
  `Edge.Cuts` and the spanning drill layer, replacing the synthetic `TOP`.
- **`PadStackDef`** (Step) per unique copper/mask/paste geometry, plating,
  drill, and physical side. SMD stacks use only their resolved front or back
  copper/mask and enabled paste; THT stacks use both copper/mask faces plus a
  plated `PadstackHoleDef` carrying the real drill diameter.
- **`LayerFeature`** (Step) is side-aware and retains every physical placement,
  including repeated copper/paste/via/back-land placements for one electrical
  number. Each is positioned at its absolute board location, references its
  padstack, and is tied to the shared logical `PinRef` and net. Bottom-side
  placement mirrors local x and asymmetric pad geometry, and reverses the
  pad-local rotation before composing it with the component rotation.
- **Logical pins and accurate mount types** — `Package` has one `Pin` per
  distinct electrical pad number. A component is THMT only if at least one
  such terminal is implemented exclusively by PTH placements; thermal vias
  repeated under an SMD exposed-pad number do not turn the component THMT.
  `Component/@layerRef` independently selects F.Cu/B.Cu, and
  `Pin/@mountType` distinguishes `SURFACE_MOUNT_PAD`/`THROUGH_HOLE_PIN`.
- **Non-degenerate package outline** — a footprint that omits its courtyard
  (e.g. the castellated header, so its interior stays free) now gets an
  `Outline` from its pad extents instead of the degenerate `(0,0)-(0,0)`
  polygon that hid J1.

For rpi-pico2 the output now matches the KiCad reference element-for-element:
19 `PadStackDef`, 60 `PadstackPadDef`, 3 `PadstackHoleDef`, 46 SMT + 3 THMT
components, placed copper on F.Cu/B.Cu.

Marker updated: `logical-complete,physical-minimal` →
`logical-complete,placement-staged,unrouted` — the document now DOES carry
physical land patterns, so "physical-minimal" was itself an overclaim in the
wrong direction (understating). The honest remaining gaps are final placement
(unlocked components are staged, not placed) and routing (no traces).

Honest boundaries:

- **Not a routed/placed board.** Component placement is still staged (or
  `place`-locked); no copper traces. The marker says so.
- **No real Quilter import gate in CI.** XSD validity + structural regression
  tests (padstacks/holes/placed-copper/mount-types present) are the automated
  gate; an actual Quilter round-trip remains a human checkpoint (like KiCad).

## IPC-2581 pads render in Quilter — mask apertures, full stackup, colon-free refs (2026-07-16)

Follow-up to the physical-model work: even with placed copper, Quilter rendered
only dark component courtyards — **no pads**. Diagnosed by structurally diffing
the emitted document against KiCad 10's own `kicad-cli pcb export ipc2581
--version B` output (which Quilter renders) across every pad-bearing dimension.
The document was XSD-valid and carried 276 placed copper `Pad`s throughout; the
gaps were in what a real consumer composites and resolves, not in validity:

- **Solder-mask apertures were missing.** A consumer renders a visible pad as
  *copper revealed through a mask opening*; with only copper `LayerFeature`s
  (F.Cu/B.Cu) and no F.Mask/B.Mask/F.Paste features, there is no aperture to
  reveal the copper through → nothing drawn. Now every pad is instanced on its
  physical side's mask layer and every non-suppressed SMD aperture on that
  side's paste layer too, matching the reference exporter.
- **The stackup listed only copper.** The `StackupGroup` was F.Cu/dielectric/
  B.Cu (3 layers); a consumer that builds its renderable-layer model from the
  `StackupLayer` sequence never saw the mask layers. Now the full top→bottom
  fabrication stack (silkscreen/paste/mask/Cu/dielectric/Cu/mask/paste/
  silkscreen, 9 layers) is emitted, matching KiCad exactly, with a `FAB_LAYERS`
  single-source table feeding the `Layer` defs, `Content/LayerRef`s, and the
  `StackupLayer` sequence.
- **Pin land-pattern geometry was inline, not dictionary-referenced.** `<Pin>`
  carried an inline `<RectCenter>`/`<Circle>` (schema-valid but only the
  `StandardPrimitiveRef` form is honored by real importers) and lacked
  `electricalType`. Now every pin references a `PRIM_n` dictionary entry
  (closed over all package pads up front) and is `electricalType="ELECTRICAL"`.
- **Package names/refs carried the CoHDL `::` separator.** `:` is the XML
  QName/namespace delimiter; a consumer whose pin/pad resolution treats a
  package name as an NCName can fail to bind pins to their land pattern. The
  `<Package name>`/`packageRef` now collapse `::` → a single `-`
  (`rpi_pico2-CHIP_0805`, KiCad's own convention), staying matched to each
  other, with the original fq symbol preserved in the Package `comment`.

The rpi-pico2 document now matches the KiCad reference element-for-element on
the pad-render path: 724 copper `Pad`s, 36 `PadStackDef`s, 114 `PadstackPadDef`s,
F.Mask/B.Mask/F.Paste feature layers, a 9-row stackup, 156 `electricalType`
pins, zero colons, and every `packageRef` resolving to a `Package`. Determinism
(byte-identical across runs) and XSD validity are preserved.

Honest boundaries (unchanged):

- **Not a routed/placed board.** Component placement is still staged (or
  `place`-locked); no copper traces.
- **No real Quilter import gate in CI.** Structural regression tests + XSD
  validity are the automated gate; an actual Quilter render remains a human
  checkpoint (like the KiCad pcbnew checkpoint). Convergence to KiCad's
  known-renderable structure is the strongest signal available headless.
- **Stackup is a nominal 2-layer fab stack** (0.035/1.51/0.035mm), not a real
  fabricator's stackup — the mask/paste/silk rows are present with nominal
  (mostly zero) thicknesses so the layer model is complete.

## IPC-2581 coordinate frame — +y-up projection matching KiCad's export (2026-07-16)

Follow-up to the pad-render work: the document rendered upside-down relative to
KiCad (the crystal cluster appeared above the MCU in Quilter, below it in
pcbnew) because IPC-2581 is a +y-up frame while CoHDL authors +y-down (its
`place`/DXF coordinates read like KiCad's internal board frame). The emitter now
NEGATES every emitted y — component `Location`s, placed-copper/hole `Location`s,
`Package` `Pin`/courtyard `Outline` corners, and the board `Profile` (arc winding
flipped too) — while KEEPING rotation values. The pad transform negates the local
offset BEFORE rotation and the component position (not the final absolute; a
naive reflection mis-places rotated, y-offset pads).

Verified against KiCad's own `kicad-cli pcb export ipc2581 --version B` of the
same board (the objective ground truth, since KiCad's export renders correctly
in Quilter): all 49 component `(x, y, rotation)` and all 224 F.Cu copper-pad
positions match element-for-element (0 mismatches). The centralized helpers
(`geom::mm_y`/`mm_femto_y`/`corner_lo_y`/`corner_hi_y`) keep the projection
byte-stable and confined to the IPC emitter — the `.kicad_mod` files stay in
KiCad's native +y-down frame (their y is the exact negation of the IPC land
pattern's). The `tools/kicad_board.py` helper negates y on import to keep the
generated `.kicad_pcb` byte-identical to the pre-fix (correct) board.

## rpi-pico2 footprint audit — datasheet-verified packages (2026-07-16)

Every rpi-pico2 component footprint was checked against its manufacturer
datasheet (a fan-out of one verification agent per part). 11 of 17 were correct;
6 were wrong at the PACKAGE level (not minor land-pattern tweaks) and were
corrected to the datasheet package:

- **U1 W25Q32RVXHJQ (flash)** — was USON 1.0×1.5mm/0.40mm; real package is
  **8-XSON 2.0×3.0mm, 0.50mm pitch** (Winbond code "XH"). Now
  `QFN8N50P200X300_1EP61X220` (KiCad DFN-8 2×3 P0.5 land).
- **U3 RT6150B-33GQW (buck-boost)** — was QFN-10 2×2mm/0.40mm; real package is
  **WDFN-10 2.5×2.5mm, 0.50mm pitch, dual-row** (5/side), EP ~1.2×2.0mm (Richtek
  DS6150AB). Now `QFN10N50P250X250_1EP120X200`.
- **J3 USB** — was a through-hole micro-USB (modeled on an unrelated Würth part);
  the real Pico 2 uses a **micro-USB SMD** receptacle. Now `FP_USB_Micro_B_SMD`
  (5 signal SMD pads on 0.65mm pitch + 4 shield tabs; KiCad Amphenol
  10103594-0001LF land), and the MPN was corrected to a micro-USB SMD part.
- **D1 PMEG6010ELR (Schottky)** — was plain SOD-123 (gull-wing); real package is
  **SOD-123W/F** (Nexperia CFP3, flat-lead). Now `FP_D_SOD_123W`.
- **C1/C6 47µF (GRM188R60J476ME15)** — was CHIP_0805; Murata "188" size code is
  **0603**. Now `CHIP_0603`.
- **SW1 TP-1221U (BOOTSEL)** — was the larger TL3342 land; real body is
  ~4.2×3.2mm SMD tactile. Now `FP_SW_SPST_TP1221U` (Panasonic EVQPU-class land;
  demo-grade — flagged to confirm against the XKB drawing).

Correct (unchanged): RP2350A (QFN-60 7×7 — EP could optionally trim 3.5→3.4mm),
ABM8 crystal, DMG1012T (SOT-523), DFE201612E inductor, Würth 0603 LED, GRM155
(0402), GRM0335 (0201), Samsung 100nF (0402), Yageo RC0201/RC0402, Pico
castellations. The nine pads orphaned by the old (wrong) packages were removed
from `std/pads.cohdl`. Overlap re-scan of all 49 placed footprints: 0
overlapping pad-pairs. Footprints without an exact datasheet drawing (the switch
land, the flash EP, the WDFN EP orientation) are noted demo-grade for a human to
confirm — same status as the KiCad pcbnew checkpoint.

## RFC-022 (mechanical locating holes — `mount_hole`) implementation notes (2026-07-17)

RFC-022 (DR-028) adds a third footprint-body construct alongside `pad`,
`courtyard`, and `silkscreen_ref`: `mount_hole N: PLATING at (x, y) diameter D`,
for a mechanical locating hole (定位孔) with no electrical function, no net, and
no device pin to bind to. Grounded in KiCad's own `np_thru_hole` precedent.

Implementation (fully conformant, no deviations):
- **AST/parse** — `MountHole` + `MountHolePlating {non_plated, plated}`;
  `FootprintDef.mount_holes`. Parsed in the footprint body next to `pad`.
- **Disjoint numbering** — `mount_hole` numbers live in their own namespace
  (`validate_footprints` gives them a separate duplicate-check map) and are
  NEVER compared with `pad` numbers or the bound device's pins (E807 walks only
  `fp.pads`). A footprint may have `pad 1..2` and `mount_hole 1..2` with no
  collision — covered by `mount_hole_parses_disjoint_from_pads`.
- **Checks (E810)** — duplicate `mount_hole` number, non-`Length`/out-of-range
  offset or diameter, non-positive diameter, and (at parse) a `PLATING` outside
  `{non_plated, plated}`. All structural, local to one footprint, never DRC.
- **KiCad `.kicad_mod`** — `non_plated` → `np_thru_hole`, `plated` → an ordinary
  `thru_hole`, both with an empty pad number (no net). `is_placeholder` now also
  checks `mount_holes`, so a hole-bearing footprint is never treated as an empty
  RFC-017 placeholder.
- **IPC-2581** — each mount_hole projects into the physical model as a placed
  through-hole with no `PinRef` and no net (`PadStack.plated` / `PlacedPad.hole`
  added); the drill `Hole` and `PadstackHoleDef` carry `platingStatus`
  `PLATED`/`NONPLATED` (the schema's spelling — no underscore). A non_plated
  hole has no copper on any layer. mount_holes do not change a component's
  through-hole-mount classification.
- **fmt** — the RFC-009 formatter emits `mount_hole` lines in source order and
  round-trips idempotently (a real bug caught in review: the formatter first
  silently dropped mount_holes on reformat; now fixed + regression-tested).

Out of scope, per RFC-022's own decision: board-level mounting holes (a
design/board-level concept nearer `board_outline`, RFC-020) and non-circular
locating features (slots, keyed holes) — both disclosed, deferred gaps.

First consumer: `examples/openmicro`'s Kailh Choc V2 keyswitch footprint
(`FP_SW_Choc_V2`) now models its central Ø5.05mm box-stem hole and Ø1.6mm MX
mounting peg as two `non_plated` mount_holes, replacing an earlier plated-pad +
mechanical-device-pin workaround. Re-scan: 0 per-key LEDs cover a switch hole,
0 pad shorts, 0 out-of-bounds; IPC-2581 schema-valid.

## RFC-023 (non-circular locating holes) implementation notes (2026-07-19)

RFC-023 (DR-029) closes the gap RFC-022 disclosed and deferred: `mount_hole`
gains an optional `shape:` and a shape-dependent geometry field.

- **Grammar** — `mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D |
  size: (w, h)]`. `shape:` reuses RFC-018's existing `PadShape` closed set
  (`rect`/`circle`/`oval`) verbatim — no new enum was introduced.
- **Default** — absence of `shape:` means `circle`, so every `mount_hole`
  written before RFC-023 keeps its exact meaning. `fmt` never spells out the
  default, so pre-RFC-023 sources are byte-identical after reformatting
  (regression-tested).
- **Shape/geometry agreement** — `circle` takes `diameter D`; `rect`/`oval`
  take `size: (w, h)`. A mismatch is E810 naming expected vs. actual; when the
  shape was DEFAULTED rather than written, the diagnostic says so explicitly,
  so the error is not a mystery. `size:` arity is checked to be exactly `(w, h)`.
- **KiCad** — a rect/oval hole reuses the existing `np_thru_hole`/`thru_hole`
  path with its declared pad shape. KiCad's DRILL vocabulary is round-or-oval
  only, so a non-circular hole emits `(drill oval w h)` — the manufacturable
  slot that actually seats a rectangular leg. Circular holes emit byte-for-byte
  what RFC-022 already emitted.
- **IPC-2581** — the hole primitive carries the declared shape and `(w, h)`.
  IPC's `<Hole>` element carries a single scalar diameter, so a non-circular
  hole reports its MINOR axis (the slot width, the conventional drill for a
  slotted hole); the full extent is carried by the padstack primitive.
  Schema-valid (`xmllint` gate).

DEVIATION (disclosed): the accepted text is internally inconsistent about field
order. Its grammar line reads `[shape:] at (x, y) [geometry]` — which is also
RFC-022's existing order — while its own worked `KailhChocV2` example writes
`[shape:] [geometry] at (x, y)`. Implementing either one alone would make the
other spelling in the RFC fail to parse. Both are therefore ACCEPTED (each
component is introduced by a distinct keyword, so this remains a single-token
decision — no lookahead, Constitution-safe), and `fmt` normalizes to the
grammar line's order. Worth correcting in the RFC text.

First consumer: `examples/openmicro`'s `FP_SW_Choc_V2` — RFC-023's own
motivating part. Its locating peg was previously a Ø1.6mm circle, which the
Kailh PG1353 datasheet ("Recommended PCB Layout", pattern side) shows is really
a 2.00 x 1.50 rectangular leg; the round hole was 0.4mm short on one axis, so
the peg could not seat. Now modelled exactly. Note the axes swap under the
datasheet→footprint mapping `T(x,y) = (y,-x)` (verified rotation-invariantly, so
the footprint is not mirrored): the datasheet's 2.00 along ITS x is 2.00 along
our y, giving `size: (1.5mm, 2mm)`. The central pole was corrected 5.05 → 5.00mm
in the same pass.

## RFC-024 (array-typed instances and indexed references) implementation notes (2026-07-19)

**This RFC was REDESIGNED the same day, superseding its own first draft.** The
first implementation (commit e5c5628) built the withdrawn draft: `inst
sw[1..=13]: SW_KEY` as pure name-expansion sugar producing `sw1`…`sw13`, with
indexing usable ONLY inside a net's member list. That design was withdrawn per
Tony's direct correction — it could not address the real motivating need
(OpenMicro's WS2812 daisy-chain and per-element `place`), because those need one
specific indexed element, not a fan-out inside a single net. The notes below
describe the accepted redesign, which replaces it entirely.

- **Declaration** — `inst NAME: [Device; N]` in TYPE position (not name
  position). N is a positive integer literal; `N < 1` is E211.
- **One array, N real elements** — expansion runs in the expander's existing
  pass-1 instance loop, creating elements internally named `NAME_0`…
  `NAME_{N-1}`, exactly the names the RFC says the author would have hand-
  written. Each goes through the SAME `handle_inst` an ordinary `inst` does, so
  designator allocation (RFC-005), pin obligations (RFC-002) and trait
  satisfaction (RFC-003) apply completely unchanged.
- **`NAME[i]` is 0-based and valid EVERYWHERE** an ordinary instance reference
  is — net members, `place NAME[i] at (…)`, and `fn`-call arguments. This is the
  redesign's central correction and is enforced structurally: `resolve_pin_ref`
  de-indexes a `Single` rather than rejecting it, and `handle_placement` grew
  the same resolution.
- **A bare unindexed `NAME` is never a valid reference** for an array-typed
  instance — E211 suggesting `NAME[0]`. Enforced by construction: an array's
  NAME is never inserted into `local_insts`, only its elements are.
- **Bounds** — `0 <= i < N`, checked at every use site (net member, `place`,
  fan-out endpoints), reported as E202 naming the valid `0..=N-1` range and the
  length.
- **Range/list fan-out** — `NAME[a..=b].PIN`, `NAME[a..=b step k].PIN`, and
  `NAME[i, j, k].PIN` survive as SUGAR over `NAME[i]`, expanded in `handle_net`
  into individual `Single` references. Still net-member-only: `place`/`nc`/
  `fn`-args each take one element (E211), since "a range at once" has no single
  meaning there. The strided form is retained per the RFC's "remain valid,
  exactly as in the first draft's design", though the redesign's prose
  enumerates only the range and list forms.
- **fmt** — `[Device; N]` in type position, `NAME[i]`/`NAME[a..=b]`/
  `NAME[i, j]` in reference position, and `place NAME[i]`. An implicit stride
  of 1 is never spelled out, so unstrided ranges round-trip byte-identically.

DEVIATION (disclosed): `docs/design/10-language-specification.md` still carries
the WITHDRAWN first draft's "Instance arrays and range references" section. The
accepted RFC-024 document states that the specification "replaces the withdrawn
first draft's section with this corrected design", but that edit had not
propagated to the shared design repo at implementation time. The implementation
follows the RFC document (explicitly Accepted, dated 2026-07-19, marked as
superseding), not the stale specification section. Worth reconciling upstream.

Also fixed here: `tools/extract_design_repo.py` carried a HARDCODED page list
that never included the standalone RFC-022/023/024 documents, so a redesigned
RFC was invisible to the usual re-extract-and-diff check — the diff came back
clean while the RFC had in fact changed. The three note IDs are now in the list
(discovered by scanning the share root for note links).

First consumer: `examples/openmicro/src/main.cohdl` — five array-typed families
(`sw: [SW_KEY; 13]`, `d: [D_1N4148W; 13]`, `key_leds: [RGB_SK6812; 13]`,
`ambient_leds: [RGB_SK6812; 16]`, `mh: [MH_M2; 4]`), with its ROW/COL matrix and
power fan-outs using range/stride sugar, and its WS2812 daisy-chain nets,
`decouple` calls and per-element `place` statements all now written as
`key_leds[i]` / `ambient_leds[i]` — positions the withdrawn draft could not
express at all. Note the old flat `led[1..=29]` became two separate arrays,
exactly the "two independent chains of the same part" case the RFC names.

VERIFIED NOT netlist-byte-identical, and that is correct: renaming every element
(`led1` → `key_leds_0`) changes instance PATHS, and RFC-005 allocates
designators by path, so `design.lock` was regenerated and designators
reassigned. Structural equivalence was confirmed instead — 98 components → 98,
70 nets → 70, identical net names, identical part-value multiset, and **0 nets
whose (part-value, pin) membership changed**. Designators are contiguous per
prefix (C1-27, D1-13, LED1-29, SW1-15, …).

## Choc V2 footprint orientation correction (2026-07-19)

`FP_SW_Choc_V2` was rotated 270° relative to the Kailh PG1353 datasheet's
"Recommended PCB Layout (Pattern Side)". Every feature was off by the SAME
uniform rotation — contacts north instead of west, the mounting-leg slot
lower-left instead of lower-right.

**Why the earlier audit missed it.** When this footprint was first checked
against the datasheet, the verification compared *relative* angles between
features (to rule out mirroring, the classic bottom-view mistake). Relative
angles are rotation-INVARIANT, so a globally-rotated footprint passes that test
cleanly. The check proved "not mirrored" and was wrongly read as proving
"correctly oriented". Absolute per-feature orientation must be compared against
the datasheet frame, not just relative geometry.

Corrected by rebuilding the footprint directly in the datasheet's own frame —
the only transform now applied is the pattern-side (bottom) → top mirror plus
KiCad's Y-down convention, `(x, y)_drawing → (-x, -y)`:

| feature | datasheet (as drawn) | footprint |
|---|---|---|
| contact (Ø1.20) | (5.90, 0) | pad 1 @ (-5.9, 0) |
| contact (Ø1.20) | (3.80, 5.00) | pad 2 @ (-3.8, -5) |
| central pole | Ø5.00 @ (0,0) | mount_hole 1 |
| mounting leg | 2.00 x 1.50 @ (-5.15, -5.00) | mount_hole 2, `size: (2mm, 1.5mm)` @ (5.15, 5) |
| in-switch LED pad | (-4.93, 0) | deliberately not modelled (discrete SK6812MINI-E used) |

Note the leg is now `size: (2mm, 1.5mm)` — the datasheet's literal "2*1.5" with
NO axis swap, where the previous rotated frame required `(1.5mm, 2mm)`. That the
swap disappeared is itself corroboration that the frame is now right.

A footprint must reproduce the manufacturer's land pattern in the PART's own
frame; baking a rotation in means every reuse silently inherits it. The board
keeps its physical orientation by rotating the INSTANCES instead — all 13 key
placements gained `rotate 270`.

VERIFIED physically neutral: every switch pad and hole (52 rows across 13
switches) lands at the identical absolute board position with identical
rotation-aware extents before and after. Confirmed by comparing pad bounding
boxes, not local drill dimensions — under a 270° rotation `GetDrillSizeX/Y`
reports the LOCAL 2.00x1.50, which reads as a change while the absolute slot is
unchanged at 1.50 wide x 2.00 tall.

Operational note found while verifying: `out/*.xml` (the `--emit ipc2581`
document) is an on-demand artifact and can go stale relative to `layout.json`.
A stale `.xml` made `tools/kicad_board.py` place the switches unrotated, which
looked exactly like an emitter bug dropping `rotate 270`. Delete and re-emit
before trusting IPC-derived geometry.

## Choc V2 footprint: canonical frame restored + LED window centring (2026-07-19)

CORRECTION OF THE PREVIOUS ENTRY. The "orientation correction" ledgered above
fixed a NON-ERROR: the original footprint frame (contacts N/NE, leg SW) was the
part's true in-keyboard orientation all along. The Kailh PG1353 datasheet's
"Recommended PCB Layout (Pattern Side)" is simply DRAWN 90° turned on the page.
Three independent witnesses agree on the physical part, all matched
quantitatively (relative feature angles 52.8°/224.2° from contact 1):

1. the kiswitch community footprint `SW_Kailh_Choc_V2` (pads (0,-5.9)/(5,-3.8),
   leg (-5,5.15); thousands of shipped keyboards);
2. a bottom-view product photo of the real switch (pin south-centre, pin
   south-east, locating leg north-west — the leg being exactly the feature the
   listing circles);
3. the datasheet's own dimensions under the page→canonical map (x,y)→(y,-x).

The previous entry's mistake: after (correctly) discovering that a rotation-
invariant check cannot detect global rotation, the footprint was compared
against the datasheet PAGE frame as if that were the part's frame. It is not.
"Absolute orientation" must be judged against the part's canonical in-use
orientation; the page told the truth about dimensions, not about which way is
up. The 90°-rotated footprint + 13 compensating `rotate 270` placements kept
the BOARD correct throughout (verified identical a third time), but left the
footprint's own frame wrong for any reuse. Both are now removed: the footprint
is back in the canonical frame, instances unrotated.

Kept from the intervening work (real fixes, all verified):
- the RFC-023 slot for the leg — now `shape: oval size: (1.5mm, 2mm)` at
  (-5, 5.15): the datasheet's 2.00×1.50 obround (rounded ends as drawn), long
  axis N-S in the canonical frame. Notably the community footprint drills a
  Ø1.6 CIRCLE here, which a 2.0mm-long leg cannot enter — the datasheet slot is
  a genuine improvement over the reference, enabled by RFC-023.
- the Ø5.00 centre pole (datasheet recommended; community uses 5.05).

REAL deviation found and fixed: the datasheet places the switch's light window
at (0, +4.93) — on the centreline, due south. Our per-key SK6812MINI-E LEDs sat
at (+1.5, +4.7), a 1.5mm-east workaround from the era of the wrong hole
geometry, misaligning every LED with its switch window. All 13 per-key LEDs are
now centred on (0, +4.93) switch-relative. Verified: 13/13 centred, zero
LED-pad-vs-switch-hole clashes (the offset's original reason is gone), zero
different-net pad overlaps, 385 tests, IPC-2581 schema-valid.

## RFC-025 (rotated pad placements) implementation notes (2026-07-20)

`pad N: Sym at (x, y) [rotate ANGLE]` — RFC-020's closed {0, 90, 180, 270} set
reused verbatim on `PadPlace`. Checked at declaration as E811 (new row), the
same shape as `place`'s E1007. `fmt` renders the clause trailing and never
spells out the default 0, so pre-RFC-025 footprints stay byte-identical.
KiCad emitter: the accepted text's lossless option — KiCad's own 3-argument
`(at x y angle)` with `size` UNCHANGED, never a silent w/h swap. IPC-2581: on
the top side the pad's own rotation composes as `(component + pad) % 360`; a
bottom-side mirror reverses the local angle, so it composes as
`(component - pad) % 360`. Position is unaffected (rotation is about the pad's
own centre). `rotate 180` on a rect is accepted as the documented no-op.
Circle pads accept any value (no-op), per the RFC.

## RFC-026 (back-side placement) implementation notes (2026-07-20)

`place <inst> at (x, y) [rotate ANGLE] [side SIDE]` — closed {top, bottom},
default top; E1008 (new row). `rotate` and `side` parse in either order;
`fmt` canonicalizes to rotate-then-side and never writes `side top`.

- **layout.json**: `"side": "bottom"` emitted ONLY for bottom placements — a
  top-side placement's JSON object is byte-identical to its pre-RFC-026 form.
- **IPC-2581**: bottom components ride `layerRef="B.Cu"` + `mirror="true"` on
  the Component `Xform` (both existing schema machinery; xmllint-gated).
  Physical model: pad local x mirrors BEFORE rotation — the same
  flip-then-orient order pcbnew's own native convention applies — verified by
  an exact-Location test on an asymmetric footprint. The reflection also swaps
  left/right chamfer corners and reverses pad-local rotation. An SMD pad's
  footprint-local layer is then combined with the component flip to choose
  F.Cu/F.Mask/F.Paste or B.Cu/B.Mask/B.Paste; padstack definitions split by
  that resolved physical side. Through-hole pads span both faces regardless of
  component side; mount_hole positions mirror with the body.
- **tools/kicad_board.py**: reads Component `layerRef`; a B.Cu component is
  `Flip(anchor, True)`-ped then rotated — found and fixed a headless-pcbnew
  segfault: `FOOTPRINT::Flip` consults the owning board's layer table, so it
  must be called only AFTER `board.Add(fp)`.

First consumer: `examples/openmicro` — all 13 matrix diodes are now pre-placed
`side bottom`, vertical, one south of each key aligned under the switch's
B-contact column, and the USBLC6 ESD array is pre-placed at (7, -38.5) tight
to the USB-C receptacle on the USB2 path. First attempt put the diode column
NORTH of each key at x+5, which landed D1's pad exactly on J3's shield-hole
column (caught by the layer-aware overlap scan) — moved south. Verified:
13/13 diodes on B.Cu, ESD front-side by the port, 0 different-net same-layer
pad overlaps, 393 tests, fmt canonical, IPC-2581 schema-valid.

## RFC-027 (Quilter physics-constraint hints and CSV export) implementation notes (2026-07-20)

Seven structured attributes riding the existing `#[name(...)]` bracket — zero
new keywords, per the redesigned accepted text — plus the `diff_pair` physics
bracket, and the eight-file CSV export.

- **Parsing** — the seven names are recognized inside the existing attribute
  bracket and parsed with their own closed argument grammars (unit literals,
  `INST.PIN` references, named optional arguments, bare flags) — never as the
  opaque-string `Attr` shape. Net-only vs inst-only targeting, at-most-one of
  each kind per declaration, unknown/duplicate/missing arguments: all E1009 at
  parse. Reference EXISTENCE resolves at expansion (a referenced instance may
  be declared later in the body), in a pass that runs after every instance in
  the body exists.
- **Checks** — one `#[ground(primary)]` per design; per-merged-net duplicate
  kinds; pin references resolve against the target's device (its selected
  variant); a crystal signal pin must map to exactly one pad; array-typed
  instances cannot carry physics attributes (disclosed scope cut). All E1009.
- **DEVIATION (ledgered)**: unit-type mismatches on numeric arguments are
  E110/E1xx, NOT the E10xx sub-case RFC-027's Tooling section literally
  reserves. RFC-011's organizing principle ("unit-mismatch is unit-mismatch
  regardless of call site" — the same rule that relocated E402/E404 into E1xx)
  is the older, structural precedent; the E110 message names expected vs
  actual as required. Worth reconciling in the RFC text.
- **CSV export** — the eight files' headers/column order match the supplied
  template files byte-for-byte; scales are the templates' own (max_current mA,
  capacitance nF, impedances ohm, frequency GHz), booleans lowercase,
  components as final designators, pins as PAD numbers with a multi-pad pin
  flattening to one row per pad (the supplied `bypass_capacitors.csv`'s
  double-row shape). Emitted as a SET (header-only files included) whenever
  the design carries any physics fact; a design with none emits NO files, so
  pre-RFC-027 builds stay artifact-identical. `build --json` gains a `quilter`
  key, present only when emitted.
- **fmt** — attributes render as single-line prefixes in source order
  (byte-offset interleaved with `#[intent]`/`#[designator]`/
  `#[placement_hint]` on insts); the `diff_pair` bracket renders in fixed
  field order and an unannotated pair renders exactly as before.

First consumers: all six examples. `examples/openmicro` reproduces the
supplied templates nearly row-for-row (`Y1,U3,5,6`; `GND,true,false`; the two
USB diff pairs at 100/50/1) — the templates describe that exact board.
ccg6df-dualport: `#[bga_fanout]` + per-port `#[high_current]` + USB2 pair
values (90/45/0.48). imvp7-vcore and tida-00021: `#[switching_converter]` on
the TPS59650 with their real L/C networks. sensor-node/rpi-pico2:
`#[ground(primary)]` (+ 500mA VBUS on the pico2).

Verified: 400 tests (7 new: template-exact headers, scale/flatten rows,
no-facts-no-files byte-compat, E1009/E110 diagnostics, fmt round-trip), all
six examples check+build, fmt canonical.

## RFC-028 (physics-constraint attributes on fn Pin parameters) implementation notes (2026-07-20)

`#[bypass]`'s target now accepts a bare `Pin`-typed fn parameter (pin part of
the PinRef omitted — the same bare form net member lists always allowed), and
the instance arguments of `#[crystal_oscillator]`/`#[switching_converter]`
accept `Instance`-typed fn parameters — both resolved through the existing
`Binding::Pin`/`Binding::Instance` call-site machinery (RFC-006), exactly as
the accepted text specifies. No new grammar token, no new resolver, no new
error code: an unresolvable bare target is the same E1009 class, its message
naming both forms ("neither an `INST.PIN` reference nor a `Pin`-typed fn
parameter in scope").

Each call site of an attribute-bearing fn produces its own independently
resolved CSV row(s). First consumer: `examples/openmicro`'s `decouple` fn —
ONE `#[bypass(vdd, 100nF)]` line annotates all 25 call sites, each resolving
to its real target: the MCU's multi-pad VDD flattens to pads 24 and 48, the
USB-C receptacle's VBUS to A4/A9/B4/B9, and each per-key/underglow LED's VDD
individually — rows the supplied template could only approximate under the
old designator run.

Verified: 404 tests (4 new: per-call-site rows with multi-pad flattening,
Instance-parameter converter resolution, unresolvable-bare-target diagnostic,
fmt round-trip of the bare form), all six examples check+build, fmt canonical.

## Example set reduced to two boards; rpi-pico2 moves to USB-C (2026-07-20)

Per direct review: the example set is now `openmicro` and `rpi-pico2` only —
`sensor-node`, `tida-00021`, `imvp7-vcore`, and `ccg6df-dualport` removed.
`sensor-node` was the original MVP demo board anchoring the exit-criteria
golden tests; those tests now anchor to `rpi-pico2` (project loaded through
the package-aware `check_files_in(&proj.name, …)` — the compat `check_files`
builds under package `main`, which sensor-node's all-std part set never
exposed but rpi-pico2's project-local footprints do).

rpi-pico2's USB front-end is now the openmicro board's: the std HRO
TYPE-C-31-M-12 receptacle (mouth off the left board edge, land's PCB-edge
line 4.5mm from origin → placed at (-21, 0) rotate 270), USBLC6-2SC6 ESD
array pre-placed beside it at (-13, 0) on the USB2 path, 5.1k CC pulldowns
(std RES_5K1_0402) advertising a UFP sink, SBU1/SBU2 nc. Data path:
receptacle → ESD → the Pico's own 27R series terminations → RP2350. Both USB
segments carry annotated diff_pairs (100/50/1GHz, the openmicro values); the
HighSpeed net class covers all six USB nets. The std `decoupling_100n` fn
gained RFC-028's `#[bypass(vdd, 100nF)]`, so every pico2 decoupling call site
now emits its real bypass row (RP2350 IOVDD flattening across pads
1/11/20/30/38…).

Verified: 404 tests (exit-criteria goldens re-anchored + regenerated), both
examples check+build, board regenerated — J3/U4 placed as stated, 0
different-net same-layer pad overlaps, IPC-2581 schema-valid.

## USB-C receptacle edge position corrected on both boards (2026-07-20)

Per direct review: the HRO TYPE-C-31-M-12 sat 2.64mm too far inboard on BOTH
boards, its mouth recessed 0.85mm behind the board edge — a plug's shell would
jam on the board corner. Root cause: the offset was inferred from openmicro's
own earlier placement instead of the manufacturer's layout. The KiCad official
footprint carries NO "PCB Edge" reference line for this part, so the HRO
datasheet's "RECOMMEND P.C.B LAYOUT" was fetched and read: the PCB EDGE line
is 5.79mm below the pad-row reference — footprint-frame y = +1.86, i.e.
positioning holes 4.46mm from the edge, shell front overhanging ~1.8mm so the
plug seats fully. (Chain self-consistency check: ref→lower-slot-top 4.18 +
slot height 1.6 = 5.78 ≈ the 5.79 edge line.)

- openmicro: `place usbc at (0, -45.64) rotate 180` (was -43)
- rpi-pico2: `place usb at (-23.64, 0) rotate 270` (was -21)

Verified on both regenerated boards: outermost copper (shield slots) 0.81mm
inside the edge, zero different-net same-layer pad overlaps, IPC-2581
schema-valid, 404 tests with regenerated goldens.

## openmicro MCU on back + position-aware GPIO re-map (2026-07-20)

Per direct review (repeated routing failures traced to feature pin selection):

1. **MCU to the back**, `place mcu at (40.5, 0) rotate 90 side bottom` (RFC-026).
   The board interior is walled off by the 13 switch through-holes and the
   literal top-right by the 18x22mm joystick module, so the clear spot nearest
   the matrix is the right edge — computed by a free-space scan, verified 0
   different-net same-layer pad overlaps against the diodes it shares the back
   with.

2. **Matrix regrouped by PHYSICAL row/column.** The old grouping was index-order
   (ROW0 = sw0..3, which spans two physical rows), forcing long diagonal ROW/COL
   runs. New: r0={sw0,1}, r1={sw2..5}, r2={sw6..9}, r3={sw10..12};
   c0={sw2,6}, c1={sw0,3,7,10}, c2={sw1,4,8,11}, c3={sw5,9,12}. Every key keeps a
   unique (row, col) scan position (13 distinct pairs). ROW nets are now short
   horizontal runs, COL nets short vertical runs.

3. **Position-aware GPIO pad re-map.** Each flexible STM32F072 signal's pad was
   reassigned (greedy nearest-pad + 2-opt) so it exits the LQFP toward its
   feature, subject to the fixed peripheral pins (USB PA11/12, SWD PA13/14,
   crystal PF0/PF1) and the ADC constraint on JOY_X/JOY_Y (kept on PB1/PB0 =
   ADC_IN9/8). TOUCH stays TSC-capable (PA2 = G1_IO3). The 48-pad device↔pad
   bijection is preserved; 14 unused GPIOs remain optional/NC.

Measured effect (MST wire-demand per net, the routing lower bound): signal-net
demand (excl GND/V3V3/VBUS pours) 4148 -> 2667 mm (-36%); matrix ROW/COL nets
fell from ~165-193mm to ~79-98mm. The remaining longest nets (CC1/CC2, HSE) are
staged pull-down/crystal passives Quilter pulls in at placement, not structural.

Verified: 404 tests, fmt canonical, IPC-2581 schema-valid, 0 pad overlaps.

## openmicro MCU relocated to USB side (2026-07-20, supersedes right-edge spot)

Per direct review — the right-edge placement (40.5, 0) left the USB pair (the
board's only high-speed signal) ~51mm long. Moved the MCU to the clear back
spot nearest the USB-C/ESD: `place mcu at (-18, -41) side bottom` (top-left).
USB FS pair now ~22mm with the fixed USB pins (PA11/12) facing the ESD — a
clean rightward run, no detour across the body.

Calibration note found doing this: kicad_board.py's `Flip(anchor, True)` then
`SetOrientationDegrees(R)` gives back-side pad delta = Rot(R-180)·(-lx, ly),
NOT the naive mirror-then-rotate. A first pass used the wrong model and put the
USB pins on the far side of the MCU (30mm, wrong way); recalibrating from the
built board's actual pad positions gave R=0 as the USB-facing rotation.

The position-aware GPIO map was re-optimized for the new spot (greedy + 2-opt,
same fixed-pin/ADC constraints). Effect vs the original arbitrary layout:
signal-net wire demand 4148 -> 2916 mm (-30%); the right-edge spot was lower
total (2667) but its 51mm USB pair was the wrong trade for the one net where
length matters. 0 different-net same-layer pad overlaps; bijection preserved.

## RFC-029 implementation judgments (2026-07-24)

RFC-029 (package dependency versioning) implemented as accepted: exact-only
`[dependencies]`, `cohdl.lock` with sha256 content hashes verified on every
run, std restructured into an ordinary versioned package. Layout on disk:
`std/<X.Y.Z>/` where each version dir is a full package — `cohdl.toml`
(`[package] name`/`version`, verified against the resolution, E1106) plus
`src/*.cohdl` — exactly the shape every other package and project uses (per
Tony's direction: std is not special). Scoped judgments, each within the
RFC's stated scope or a disclosed deviation:

1. **Registry roots** (RFC assumes "packages available on disk", hosting out
   of scope): resolution searches `<project>/deps/<name>/` first, then
   `<lib_root>/<name>/`, then the RFC-030 cache. (Superseded in detail by
   the 2026-07-25 library-root entry below: the local root is the
   discovered `lib/`, and std is one family dir under it — originally it
   was the *parent* of a discovered `std/`, which made std's own location
   the definition of the registry root.)
2. **`cohdl update [PATH] [--dep NAME]`** vs the RFC's `cohdl update <name>`
   surface: the positional slot keeps the CLI-wide PATH convention (a bare
   name would be ambiguous against a path); `--dep` carries the package name.
3. **Migration** is the `update` command, not an interactive build-time
   offer (the CLI is non-interactive by design): `build`/`check` on a
   pre-RFC-029 manifest is a hard E1104 whose help names `cohdl update`,
   and `update` performs the append + first lock write automatically.
4. **First-resolution lock rows are written by `check` as well as `build`**
   — the RFC's gradeability section puts the mechanism at project load,
   before any parsing, which both commands share; a check that verified
   nothing on a fresh checkout would be a weaker ladder rung.
5. **E1105 is stderr prose in every mode**, never inside a `--json`
   diagnostics array (deviation from the Tooling section's blanket "codes
   ride the JsonDiag shape"; mirrors E000's documented classification —
   suppressing it from machine output would invite treating an overridden
   build as reproducible). The E11xx *errors* do ride the array.
6. **Unpinned targets** (single `.cohdl` files, the LSP's overlay analysis)
   resolve the newest version under the std root — they have no manifest to
   pin with and are outside the RFC's package scope; documented, not locked.
7. **Hash recipe**: sha256 over every regular file in the package dir
   (dotfiles excluded), sorted by `/`-normalized relative path, each file
   contributing `path NUL len NUL bytes` — a superset of the RFC's ".cohdl
   files, doc documents, footprint symbols" enumeration (the whole package
   is the identity, manifest included).

**Amendment (same day, per direct review)**: the first cut encoded the
version in the registry path (`std/<X.Y.Z>/`) — redundant with the
package's own `[package] version`, i.e. two sources of truth. Corrected:
the manifest is the SOLE version authority; a family dir offers versions
by reading its packages' manifests (the dir itself if it carries a
cohdl.toml, else each subdirectory that does), and directory names are
convention only — `deps/mypkg/current/` declaring 1.0.0 resolves a
`mypkg = "1.0.0"` pin. Duplicate declared identity across two dirs is a
hard E1106. The repo's std accordingly flattened to `std/` being the
package itself (`std/cohdl.toml` + `std/src/`); the content hash was
unaffected (package-relative paths unchanged), so committed locks stayed
valid across the restructure.


## RFC-030 implementation judgments (2026-07-27)

RFC-030 (registry.cohdl.org) implemented: the CLI surface + a full server
implementation under `registry/` (the RFC scopes the external contract
only; the server stack was directed separately — Cloudflare Workers/D1/
R2/KV/Assets with a Vite+TanStack+React UI). Scoped judgments:

1. **Transport = the system `curl`** — the constitution grants no HTTP
   dependency and RFC-030 names none; shelling out to the platform's own
   client keeps the crate at zero crate-dependencies. `COHDL_REGISTRY`
   overrides the default host (tests run a loopback mock).
2. **Archive = deterministic uncompressed POSIX tar** — the RFC says
   ".tar.gz (or equivalent)"; DEFLATE is not worth hand-rolling for
   kilobytes of source. Epoch mtimes + sorted entries: packing is
   byte-reproducible; transport-level compression is curl/HTTP's job.
3. **Login = the cargo shape** — "browser-based auth flow" implemented as
   open-the-account-page + paste the token (verified via POST /login and
   stored with the account's publish grants in ~/.cohdl/credentials.toml,
   gitignored by location).
4. **Cache = `~/.cohdl/registry` (`COHDL_HOME` overridable)** as a third
   RFC-029 registry family — a fresh `cohdl install` populates it and
   ordinary offline `check`/`build` then resolve from it; `build` itself
   never fetches (deterministic, network-free builds; E1102's help points
   at `cohdl install`).
5. **`update` = RFC-030 semantics** — re-resolve to the latest published
   exact version (registry first, local families as fallback, so std and
   vendored packages keep working); the manifest is rewritten only on a
   real bump; the RFC-029 lock rewrite is unchanged underneath. The NAME
   positional works when unambiguous; `--dep` disambiguates a name that
   collides with a directory.
6. **Scoped names in the compiler**: `[dependencies]` uses quoted TOML
   keys (`"@sparkfun/power" = "1.0.0"`); the module root sanitizes to
   `sparkfun_power` (the `@` carries no identifier value); cache/deps
   family dirs nest naturally (`@sparkfun/power/1.0.0`).
7. **Download integrity**: the unpacked cache content is re-hashed and
   must equal the server's declared hash before anything is recorded —
   a mismatch (E1206 hard form) deletes the cache entry rather than
   poisoning later E1103 checks.

## Package metadata + document rendering in the registry (2026-07-25)

RFC-030's Tooling section names "README rendering from a package's own
`#[doc(...)]`-referenced content" as web-UI parity scope, and a registry
that shows only names, hashes, and sizes cannot answer "what is this
package?". Two additive judgments, neither touching a verdict, a
designator, or an emitted byte:

1. **Three display-only `[package]` keys** — `description`, `license`,
   `repository`. RFC-029 specifies `name` and `version` as the manifest's
   identity fields; these three are display metadata, so they are parsed
   into `project::Manifest`, echoed by `cohdl publish` (including
   "— (no `[package] <key>` in the manifest)" for an absent one, so a
   publish never silently ships blank metadata), and stored **per version**
   in D1. Per-version because a manifest is a per-version fact: a published
   version is one immutable identity, so its metadata is too. Anything
   "package-level" — the description a search hit shows, the license the
   package page shows — derives from the newest version by subquery, so
   `versions` stays the single source of truth. `fmt` is unaffected by
   construction: it canonicalizes `[dependencies]` only and passes every
   other manifest section through byte-for-byte (asserted by
   `fmt_leaves_package_metadata_untouched`, since a silently-dropped
   construct is fmt's classic failure mode).

2. **Documents = the `#[doc]` set already in the archive** — no new
   language surface. At publish the server scans the tar's `.cohdl` files
   for `#[doc("path")]` references (same lexical package-relative grammar
   `parse.rs` enforces, `//` comments stripped), keeps the ones the archive
   actually contains, and stores that sorted list as the version's document
   index. `GET /api/doc?pkg&version&path` serves one file out of the
   immutable tar in R2. The endpoint serves **any** file the archive
   contains, not only declared documents: a rendered README's own figures
   are relative paths that were never declared themselves, and the whole
   tar is already public at `/packages/{name}/{ver}.tar`, so restricting it
   would break figures while protecting nothing. The `docs` list decides
   what the UI *presents*; the endpoint just serves bytes — with
   `Content-Security-Policy: sandbox`, `X-Content-Type-Options: nosniff`,
   a content type from a closed extension map that has no `text/html`
   entry, and immutable caching.

Two further consequences worth recording:

3. **The manifest inside the archive is now verified at publish.** The
   server re-reads `cohdl.toml` from the tar and refuses (400) a publish
   whose declared name or version disagrees with the URL — the same rule
   the compiler already lives by (the manifest is the sole identity
   authority), now enforced at the boundary where a client could otherwise
   assert anything. `publish` surfaces the server's message under E1202
   instead of a bare `HTTP 400`; no new error code (E1202 already covers
   "the server refused a publish").

4. **Markdown is rendered to React elements, never HTML.** Published
   documents are untrusted publisher content: the renderer
   (`registry/src/ui/markdown.tsx`) is a deliberately incomplete
   Markdown subset with no raw-HTML passthrough and no
   `dangerouslySetInnerHTML`, so an unsupported construct — including an
   inline `<script>` — degrades to literal text. Link and image URLs are
   restricted to http/https/mailto plus same-version relative paths, so no
   `javascript:` URL can reach an anchor.

Also fixed here: the https-only redirect shipped in 3cec262 applied to
loopback hosts too, which made `npm run dev` unusable (every request
301'd to https) and would have sent an HSTS header for `localhost` —
pinning every local project's port to https in the developer's browser.
Both are now skipped for loopback hostnames only; deployed traffic always
carries the zone hostname, so production enforcement is unchanged.


## The library root: `lib/` (2026-07-25)

std moved from `std/` to `lib/std/`, ahead of publishing the official
packages as many small libraries rather than one growing std. No RFC text
changes: RFC-029/030 never specify a repository layout — they specify that
a package's identity comes from its manifest and that dependencies resolve
through family dirs. This entry records the mechanism change that came
with the move.

1. **The registry root is now a thing in its own right.** RFC-029's first
   implementation defined the local root as *the parent of the discovered
   `std/`*, and gave std a branch of its own in `Registry::families`
   (`if name == "std" { the std root itself } else { its sibling }`). That
   made std's location the definition of everywhere else's, and made the
   repository root double as a package namespace — any top-level directory
   was a candidate family dir. `families()` is now uniform for every name,
   std included: `<project>/deps/<name>`, `<lib_root>/<name>`,
   `<cache>/<name>`.

2. **Root discovery asks about packages, not about std.**
   `deps::is_library_root` accepts a directory if at least one immediate
   subdirectory is a readable package family;
   `project::find_lib_root` walks the executable's ancestors for such a
   `lib/`, then tries the current directory. Discovery must be content-
   based rather than name-based because an installed binary's ancestors
   include `/usr/lib`. std is then resolved like any other library
   (`project::newest_std` = newest under `<lib_root>/std`), so
   `find_std_root` is gone; nothing in the resolver mentions std by name
   any more except the implicit-dependency rule, which is a language rule,
   not a path rule.

3. **Adding a library is a filesystem act, not a code change.** Verified
   end to end: a second package dropped beside std (`lib/passives/`,
   declaring `0.2.0`) resolves, locks with its own content hash, and emits
   from a pinned `[dependencies]` entry with no compiler change.

4. **Content hashes were unaffected** (the recipe is over package-relative
   paths, and `lib/std/` is the same package it was as `std/`), so
   committed `cohdl.lock` rows stayed valid across the move — the same
   property the RFC-029 flattening amendment relied on.

### Registry policy: no publish without a license (2026-07-25)

Operator decision, layered onto the metadata work above: **every published
version must declare `[package] license`.** A package a design can pin is a
package whose terms the design's owner must be able to read, so an
undeclared license is a refusal rather than a blank field on the page.

- **Enforced in both places, server authoritative.** The worker refuses the
  publish (400, surfaced by the CLI under E1202) after reading the manifest
  out of the archive; `cohdl publish` refuses earlier still — before packing
  or contacting the registry — so a license-less package never leaves the
  machine. `publish_without_a_license_never_reaches_the_network` proves the
  pre-flight ordering by pointing the CLI at a dead port: a license-less
  publish fails with the license message, and only a *licensed* one gets far
  enough to fail with E1204.
- **The value is not validated against a license list.** Proprietary and
  custom terms are legitimate (`LicenseRef-…`, "see LICENSE.txt"); what the
  registry refuses is silence, so `license = ""` and whitespace are refused
  exactly like an absent key. An SPDX allowlist would reject valid terms and
  is not the registry's judgment to make.
- **`description` and `repository` stay optional** — they are display sugar;
  a license is a condition of distribution.

Settled the same day: the repository is **MIT** throughout (root `LICENSE`,
`Copyright (c) 2026 Conol AI`), and every manifest in it declares that —
`lib/std/cohdl.toml`, both example designs, the three Cargo manifests
(narrowed from `MIT OR Apache-2.0`), `registry/package.json`, and
`editors/vscode/package.json` (which had pointed at a root `LICENSE` that
did not yet exist). `cohdl publish lib/std` therefore clears the gate.

Also corrected while testing the gate: `publish` and `login` reported an
unreachable registry as `publish failed: HTTP 0` / "the registry rejected
the token (HTTP 0)" — curl reports status 0 when the exchange never
completed. Both now render E1204, the code that exists for exactly this and
that RFC-030 requires be distinguishable from E1201 (rejected token), E1202
(rejected publish), and E1103 (hash mismatch).


## The `passive` library, and std's scope (2026-07-25)

Chip resistors and MLCCs left std for their own package. std keeps what
every library needs (the traits that form the prelude) and what the demo
boards need; `passive` carries the parts. Consequences worth recording:

1. **std 0.1.0 -> 0.2.0.** Removing public declarations is a breaking
   change, so the version moved rather than the content changing under a
   pin — the discipline RFC-029 exists to enforce. Both examples repin
   `std = "0.2.0"` and add `passive = "0.1.0"`; `std::status_led` went
   with the passives (it instantiated a 1k resistor, and a sub-circuit
   that needs a part from another library cannot live in std). The chip
   lands (`CHIP_0402`/`CHIP_0603` and their pads) moved too: a land now
   exists once, in the package whose parts use it.

2. **The catalog is generated, and the generator is the source of truth**
   (`tools/gen_passive.py`, ~9.7k parts). Two rules that shape it:

   - A part number is emitted only from a scheme read out of the
     manufacturer's own datasheet. Yageo RC_L resistors and Yageo CC
     capacitors both qualify. Samsung CL and Murata GRM do not — their
     numbers carry a thickness/electrode field that (size, dielectric,
     voltage, capacitance) does not determine — so Samsung alternates come
     from a table dumped from Samsung's product database
     (`tools/passive_data/samsung_cl.json`, 1636 parts) and Murata's
     appear only where individually verified. CoHDL requires a real `mpn`
     on every `alt` (E802), which is the right constraint: there is no way
     to name a second source without naming the part.
   - Which parts exist is data, never inference. Capacitor availability
     comes from the datasheets' own capacitance/voltage tables
     (`yageo_mlcc.json`), and every emitted Yageo part number is checked
     against Yageo's specsheet endpoint (`tools/verify_passive_mpns.py`).
     A number that does not resolve is dropped, so the catalog errs by
     omission and never by assertion.

3. **Two errors this caught, both real.** The packaging letter in a Yageo
   part number is coupled to the case size — `RC2512FR-...` does not
   exist, `RC2512FK-...` does — which the first generated cut got wrong
   for every 2010 and 2512 part. And parsing the C0G capacitance tables
   over-reached into voltage columns the CC series does not offer, which
   the endpoint check caught. Neither would have been visible from the
   compiler's side: both produce perfectly well-typed designs with
   unorderable BOMs.

4. **Scale is not a problem for the pipeline.** 9.7k parts add ~0.1 s to a
   debug `check` and ~0.02 s to a release one, linear in part count. The
   only scaling defect found was a parser hang (see below), not a cost.

## Parser: recovery that could not advance (2026-07-25)

`sync_in_block` stops *at* the `,`/`}` it finds so the caller can see the
delimiter. Three loops re-entered recovery on that same token and never
terminated — a hang, not a slow parse, appending a diagnostic per pass
until memory ran out. Reachable from ordinary malformed input: a bad
generic argument in a `part` type (`D<1e+06ohm, 1%>`), a stray comma in a
part body, a non-string AVL value (`mfr: 7`). Fixed by
`sync_in_block_advancing`, which consumes the token recovery stalled on
when the cursor did not move; the three sites that `continue` now use it.
Regression test asserts a bounded diagnostic count per shape — unbounded
IS the bug. This one matters beyond tidiness: the harness feeds
model-generated source into `check` in a loop, and that loop had a
reachable non-termination.

## Board-outline DXF made importable by real CAD (2026-07-26)

`examples/openmicro/mechanical/openmicro-outline.dxf` would not import into
Autodesk Fusion. The cause was not the geometry — it was that the file had
been hand-written to satisfy CoHDL's reader and nothing else.

RFC-020's reader (`src/dxf.rs`) is deliberately narrow: it scans for one
closed `LWPOLYLINE`/`POLYLINE` on `Edge.Cuts` and ignores everything else in
the file. A 56-line file containing only that entity therefore parsed
perfectly here while being invalid DXF: `ezdxf.readfile` rejects it, and so
does `ezdxf.recover` (the maximally permissive path), with
`DXFStructureError: missing 'AcDbPolyline' subclass in LWPOLYLINE`. What was
missing, beyond that subclass marker: the `HEADER` section (so no `$ACADVER`
— an importer must then assume a version predating LWPOLYLINE — and no
`$INSUNITS`, so millimetres were a guess), the `TABLES` section (the entity
claimed a layer no `LAYER` table defined), `BLOCKS`/`OBJECTS`, and entity
handles with their owner pointers.

The file is now a conforming AC1015 (R2000) document — same eight vertices,
same `tan(22.5°)` corner bulges, verified by a strict `ezdxf.readfile` plus
`Auditor` run with zero errors — and CoHDL's extraction is byte-identical
(`openmicro-layout.json` and the IPC-2581 `Profile` checksums are unchanged),
which is the property that made the swap safe to make.

Generated by `tools/make_board_outline_dxf.py` (a dev utility needing `ezdxf`,
in the same category as `tools/kicad_board.py` needing KiCad's own Python)
rather than hand-written, because hand-writing is what produced the invalid
file. Its output is byte-reproducible: the writer's `$TDCREATE`/`$TDUPDATE`
stamps, its `$FINGERPRINTGUID`/`$VERSIONGUID`, and the `<version> @ <ISO
timestamp>` string it records in the object dictionary are all pinned
afterwards by editing those records' values — never by deleting records,
which would orphan the handles pointing at them.

The general lesson, worth remembering the next time a narrow reader tempts a
stub input: **a file CoHDL accepts is not evidence the file is valid.** The
reader's narrowness is a deliberate contract, not a validation service.

## Pad slot drills — `drill: (w, l)` (2026-07-26)

A USB Type-C receptacle's shield legs seat in plated **slots**. RFC-018 gave
`pad` a scalar `drill:`, so the openmicro footprint had been carrying the
compromise in a comment: "the real part is a 0.6x1.7mm plated slot; CoHDL
drills are round, so approximated as a 0.6mm plated hole". A round 0.6mm hole
does not accept a flat stamped leg, so the board's own footprint was
describing a part that could not be assembled — the kind of defect a footprint
exists to prevent.

No Accepted RFC covers this (the share root was re-derived first: 41 notes,
all already extracted, RFC-030 still the highest), so it lands in
`docs/provisional-syntax.md` §9 pending ratification. Judgments:

1. **Reuse RFC-023's convention rather than invent one.** `drill: D` stays a
   round hole; `drill: (w, l)` is a slot — the same scalar-or-tuple split
   `mount_hole` already uses for `diameter D` vs `size: (w, h)`. One language
   convention for "this hole is elongated", not two.
2. **No new error code.** A malformed slot is an invalid pad declaration,
   which is what E805 already means. New codes are for new *kinds* of
   mistake, and this is not one.
3. **Two structural rules beyond per-value validation**, both of which
   describe holes that cannot be manufactured: a `circle` pad may not carry a
   slot (its hole would break out of the annular ring on the long axis — the
   help names `shape: oval`), and a slot may not exceed the pad's own size on
   either axis, with the diagnostic naming the offending axis rather than
   reporting a bare "too large".
4. **Projection reuses the paths oval mount holes already established.**
   KiCad gets `(drill oval w l)`; IPC-2581's single-scalar `<Hole>` gets the
   slot's **minor axis** — the width it is actually routed with — because the
   full extent is already carried by the padstack primitive. A test asserts
   the slot's LENGTH never appears as a hole diameter, since that is the
   silent failure mode.

`fmt` renders the tuple form and round-trips it (asserted, because a new
grammar form silently vanishing through `fmt` is this project's recurring
bug), and the LSP hover shares `fmt`'s renderer so the two cannot drift.
`lib/std/src/pads.cohdl` now declares the real slots — 0.6 x 1.7mm upper,
0.6 x 1.2mm lower — and `openmicro`'s four shield pins project as
`(drill oval …)` accordingly.

## openmicro footprint audit — USB-C shield, joystick, debug connector (2026-07-26)

Three footprints re-checked against manufacturer documents downloaded into
`examples/openmicro/docs/` (indexed in that directory's README).

**1. USB Type-C shield legs are slots, not round holes.** Confirmed and fixed —
see the pad-slot-drill entry above for the language work this required. Reading
the HRO TYPE-C-31-M-12 drawing's "RECOMMEND P.C.B LAYOUT" panel also corrected
two things the previous footprint had wrong:

- the shield lands were 1.0 x 2.1 / 1.0 x 1.6 with a 0.6 round hole; the drawing
  specifies 0.90 x 2.00 copper over a 0.60 x 1.70 slot (upper pair) and
  0.90 x 1.70 over 0.60 x 1.40 (lower). The 0.60/0.90 pair dimensioned at the
  panel's lower left is hole width vs copper width — a 0.15mm ring all round.
- the four merged corner pads sat at ±2.45 / ±3.25; the drawing's 4.80 and 6.40
  put them at ±2.40 / ±3.20. The eight narrow pads already matched its
  0.50/1.50/2.50/3.50 exactly, which is what made the outer pair's 0.05 error
  visible. Layout tolerance is ±0.05, so this was at the limit.

Leg positions (±4.32, and 4.18 between the rows) were already right.

**2. The joystick footprint was not a land pattern at all.** `FP_Joystick_RKJXV`
had eight generic 2.54mm header pads at invented positions — the real
RKJXV122400R has **ten** terminals in three groups plus ten mechanical holes.
Rebuilt from Alps' own geometry:

- VR1 three ø1 terminals on a column 8.73 right of the lever axis, 2.5 pitch;
  VR2 three ø1 terminals on a row 8.73 above it; switch four ø1.2 terminals at
  x = ±3.25 with rows 4.5 apart.
- mechanical: four ø1.5 frame-leg holes at (±6, ±6), two ø1.6 ±0.05 locating
  holes at (±4.3, 0), and ø2.6 clearance for the four 0.75mm bosses at
  (±3.5, ±3.5) — all RFC-022 `mount_hole`s, carrying no net.
- the `ThumbPointer` device gained the switch's real terminal count: four pads,
  paired by side (a+b / c+d) the way the board's other 4-terminal switch is
  modelled, because the terminals leave the switch body on its two sides.

  > **SUPERSEDED (2026-08-06):** by-side pairing was the wrong verdict — the
  > poles pair BY ROW (a+c / b+d), and by-side pairing bridged both poles onto
  > each net, holding JOY_SW at GND through the switch body. See "openmicro
  > joystick: push-switch pole pairing + 1mm east shift" below.

Source discipline worth recording: Alps publishes the mounting-hole drawing only
as a 427×446 bitmap, and its stacked dimensions are easy to misattribute — an
early reading of it put the frame holes at ±6.325/±5.0, which the STEP model
disproved (±6.0/±6.0). The committed pattern is from the **STEP model's**
cylinder centres, cross-checked against the drawing's labelled dimensions, which
agree everywhere they are unambiguous. Guessing the two coordinates the bitmap
could not resolve would have reproduced exactly the defect being fixed.

> **SUPERSEDED (2026-07-29):** the paragraph above records the wrong verdict.
> The drawing's ±6.325/±5.0 was correct all along and the STEP transcription
> was the error — see "Joystick footprint corrections" below.

Also: the footprint's origin is now the **lever axis** (the drawing's ø4
keep-out centre), not an arbitrary pad corner, so `place joy` no longer needs
the courtyard-centre back-offset the placeholder required — the placement moved
from (27.325, -28.575) to the corner cell centre (28.575, -28.575).

**3. The 4-pin 2.54mm SWD header became a 10-pin SMT debug connector.**
Amphenol ICC Minitek127 `20021121-00010T4LF` — 2x5, 1.27mm, vertical SMT, the
standard ARM Cortex-M debug form factor. Land pattern from drawing 20021121
sheet 2: pads 0.76 x 1.95, rows 4.35 apart centre-to-centre, derived from the
drawing's own 6.30 overall extent and 2.40 inter-row gap. Judgments:

- **Signals keep their standard Cortex-M positions** (1 VTref, 2 SWDIO, 3 GND,
  4 SWCLK, 5 GND, 9 GNDDetect, 10 nRESET) so an off-the-shelf SWD cable works.
  The UART takes pin 6 (SWO) and pin 8 (TDI) — a Cortex-M0 implements neither,
  so nothing is sacrificed. Pin 7 is the connector's KEY position and is the
  device's only `optional` pin.
- **USART2 on PA2/PA3**, confirmed against the ST datasheet's alternate-function
  table (AF1). USART1 was unavailable: all four of its candidate pins
  (PA9/PA10, PB6/PB7) are consumed by the key matrix. PA2/PA3 were previously
  in the design's `nc:` list, so no pin was taken from another function.
- the MPN's post-option digit is `0` ("WITHOUT POST"), so the drawing's two
  ø1.00 NPTH holes — explicitly "FOR THE PRODUCT WITH POST ONLY" — are
  correctly absent.

Consequences for the board: `J2` changes footprint AND net count (NRST, UART_TX
and UART_RX now reach it), and the joystick's land pattern and origin both move,
so the hand-routed `openmicro.kicad_pcb` needs both parts re-placed and
re-routed. That file was not touched. One benign note: the real joystick body is
larger than the placeholder, so its pad-inclusive courtyard now overlaps
`sw[5]`'s by 0.4mm — the part BODIES still clear each other by 0.45mm.

### openmicro side assignment: USB-C and per-key LEDs move to the back (2026-07-26)

Direct instruction, layered on the footprint audit above. `place … side bottom`
(RFC-026) now covers three groups, and the reasoning differs per group:

- **`usbc`** — the bottom is this board's component side (the MCU is already
  there), so the receptacle's SMD contacts land on `B.Cu` beside the MCU they
  serve. Its four shield legs are through-holes and span the board either way;
  verified on the regenerated board: contacts on `B.Cu`, legs on `F.Cu+B.Cu`
  with the 0.6x1.7 / 0.6x1.4 slots intact, and pad x mirrored (A6/A7 swap sides).

  **The `rotate 180` had to be REMOVED when the part moved to the back**, and
  this is the trap worth remembering: the footprint is drawn mouth-toward +y
  (the datasheet's PCB EDGE line sits 1.86mm on the +y side of the origin), so a
  FRONT-side placement needs `rotate 180` to aim the mouth at the board's -y
  edge. A flip already inverts y, so keeping the rotation as well cancelled it —
  the connector faced inboard and all 16 signal pads plus two shield legs landed
  OUTSIDE the y = -47.55 board edge. `check`/`build` cannot catch this: every
  pad, net and courtyard is still perfectly valid, the part is merely pointing
  the wrong way. It took reading pad y-coordinates off the generated board
  against the outline. With the flip alone, copper sits at y = -41.595 (signals),
  -42.510 and -46.690 (legs) — identical to the original front-side geometry,
  0.86mm clear of the edge, mouth overhanging as intended.
- **the 13 per-key LEDs** — a reverse-mount emitter fires through the board, so
  lighting a top-side switch means sitting under it.
- **the 16 perimeter LEDs stay on the front**, where their light escapes around
  the board edge rather than through it.

Side effect worth noting: with the per-key LEDs on the back, the 13 deliberate
LED-inside-switch courtyard overlaps disappear (the parts are no longer on the
same side), and flipping the receptacle cleared its overlap with the ESD array.
A layer-aware scan of the regenerated board — pads that overlap AND share a
copper layer — reports **0 shorts** across 65 placed parts, with exactly one
courtyard overlap left: the joystick against `SW11` by 0.49mm, the consequence
of the real RKJXV body being larger than the placeholder it replaced.

`J2` (the new debug connector) has no `place` statement, so it stages outside
the outline exactly as the 4-pin header it replaced did — it still needs a
position chosen.

## Footprint board cutouts — `window { … }` (2026-07-26)

The openmicro LED footprint was incomplete in a way that no amount of pad
checking would catch: the SK6812MINI-E is **reverse-mount**, so its die faces
the board and its light leaves through an aperture in the PCB — and CoHDL had no
way to say "route a hole here". RFC-018 gives footprints pads, a courtyard and a
silkscreen anchor; RFC-022/023 give them drilled holes. A cutout is neither.

Added `window { shape, at, size }` (provisional — docs/provisional-syntax.md
§10; the share root was re-derived when the pad-slot work landed and RFC-030 is
still the highest). Judgments:

1. **Same block shape as `courtyard`**, down to the closed shape vocabulary and
   the at-most-one rule, so the language has one spelling for "a shape on a
   layer". Validation is literally the same code path, keyword-parameterised, so
   the two cannot drift; all E806.
2. **Not `mount_hole`.** That construct means a DRILLED hole and would have
   emitted a 3.4mm "drill" where a fabricator needs a milled aperture — and
   folding cutouts into it would have redefined the Choc switch's oblong leg
   hole, which genuinely is a drilled slot. Two manufacturing operations, two
   constructs.
3. **Projected onto Edge.Cuts**, matching KiCad's own reverse-mount LED
   footprints (whose aperture geometry supplied the 3.4 x 3.0mm size — the
   SK6812MINI-E datasheet Rev. 02 states no cutout dimension anywhere, so this
   is convention and is labelled as such in the footprint). Corners stay square;
   the router radius is the fab's choice. A test asserts a window adds no drill.
4. **The field is boxed** (`Option<Box<Courtyard>>`): a window is rare, and
   inlining a second shape record made `ItemKind::Footprint` the outsized enum
   variant for every footprint without one (clippy caught it).

Verified end to end: the emitted `.kicad_mod` carries
`(fp_rect … (layer "Edge.Cuts"))`, KiCad's own parser reads it back as board
edge, and the regenerated board has 29 light windows — 13 with the per-key LEDs
on the back, 16 with the perimeter ring on the front, each oriented with its
placement.

**Known gap, deliberately left:** the window is not in the IPC-2581 document. A
per-component cutout must be transformed to board coordinates and subtracted
from the `Profile`; that is a bigger change than the KiCad projection and should
follow the RFC rather than precede it.

### ESD array follows the USB pair to the back (2026-07-26)

`esd` moved from (7, -38.5) on the front to (-7.8, -41) `rotate 180 side bottom`
— into the gap between the MCU's right courtyard edge (x = -13.26) and the
receptacle's left (x = -5.37), biased to the receptacle side so the UNPROTECTED
`USBC_*` stretch is the short one (6.4mm from connector pad to clamp): copper
before the clamp is copper the outside world can reach.

`rotate 180` is load-bearing. The USBLC6 passes each data line THROUGH the
package (I/O1 on pads 1 and 6, I/O2 on 3 and 4), so the part has an input side
and an output side. Unrotated on the back, pads 1/3 face -x (the MCU) and 6/4
face +x (the connector) — backwards, forcing the pair to cross itself twice.
With the rotation the path marches monotonically leftward, verified on the
generated board: connector A6 at x = -0.25, ESD in (pad 1) at -6.66, ESD out
(pad 6) at -8.94, MCU PA12 at -13.84. Still 0 shorts; clearances 0.63mm to the
receptacle courtyard and 3.66mm to the MCU's.

### Choc V2 fixing leg becomes a soldered pad (2026-07-26)

Direct instruction: the switch's third leg was an RFC-022 `mount_hole` (a bare
cleared hole); it is now a plated through-hole pad so it can be soldered and
take keypress force. The leg genuinely is metal — the CPG1353 parts list has it
as phosphor bronze C5191, qty 1 — so soldering it is sound, and leaving two
0.5mm contacts to carry the force was the weaker choice.

- **It had to become a real device pin.** A footprint's pad set must match its
  bound device's pins (E807), so `KeySwitch` gained a third pin. It is
  `optional`: the leg carries no signal, and `required` would oblige all 13
  switches to wire a pin with nothing to connect to. The netlist confirms the
  anchor is electrically dead — key-switch pins 1 and 2 appear in nets 13 times
  each, pin 3 zero times.
- **The hole is a SLOT** (2.0 x 1.5mm per the datasheet), so this is the first
  in-tree use of the `drill: (w, l)` form outside the USB-C shield legs.
- **Annular ring is 0.3mm, not the 0.5mm the contacts use.** The reverse-mount
  LED's land sits 0.85mm from the leg hole; a 0.5mm ring would have closed that
  to 0.35mm. Measured on the generated board, the tightest leg-to-foreign-copper
  gap is 0.545mm — and it is to the LED lands on the OPPOSITE side, which counts
  because a through-hole exists on both layers.
- The central ø5 boss hole stays a `mount_hole`: a plastic boss passes through
  it, so there is nothing to solder.

Still 0 pad overlaps across the board.

### LED apertures vs back-side copper (2026-07-26)

Adding the light windows created a class of conflict that could not exist
before: an aperture is a hole through the WHOLE board, so a front-side LED can
destroy a back-side pad. Reported by direct inspection, then found
systematically by scanning all 29 apertures against all 300 pads.

**Resolved by moving the back side, not the ring.** Direct instruction, and the
right call: the underglow ring's even spacing is the product feature a user
actually sees, so it is fixed geometry and the back side gets arranged around
its apertures. `ambient_leds[0]`/`[1]` are back at -33/-11. `esd` stepped 3mm
inboard to y = -38 — clear of the aperture's y -43.5..-40.5 while keeping the x
position between MCU and receptacle — and `mcu` moved from x = -18 to -18.6,
because 0.4mm of board between a ROUTED aperture edge and a pad is thinner than
a router holds comfortably; it now has 1.0mm, at a cost of 0.6mm on a USB pair
with ~21mm of margin. The USB path stays monotonic (connector -0.25 -> ESD in
-6.66 -> ESD out -8.94 -> MCU -14.44).

The earlier ring-moving attempt is left recorded below because its lesson stands
— the usable span is bounded on both sides — but the ring is no longer the thing
that moves.

`LED2`'s aperture cut `U1.5` (VBUS) and `U1.6` (USB_DP) by 0.325mm — a
consequence of two earlier changes meeting: the ESD array moved to the back
between the MCU and the receptacle, and the LED gained an aperture. Along the
top edge the back side now owns x -22.90..-13.10 (U3's pads), -9.60..-6.00
(U1's) and -4.77..4.77 (J3's), leaving a 3.5mm gap for a 3.4mm aperture. The
`-11` slot is simply not available any more, so the two left-hand top-edge ring
LEDs moved to -35 and -27, in the free span between the M2 mounting pad and the
MCU. The ring is deliberately unevenly spaced there — even spacing would cut
copper.

An intermediate attempt at -38/-29 was wrong and is worth recording: -38 put
`LED1`'s land on top of `H1`, the plated M2 mounting hole at (-42.5, -42.5). A
mount pad is a through-hole, so it occupies every layer and a front-side land
cannot share its footprint. The usable span for a top-edge ring LED is therefore
bounded on BOTH sides — by H1's pad on the left and the MCU's pads on the right.

Board now scans clean: 0 apertures cutting a pad, 0 shorts, 0 pads outside the
outline; the only courtyard overlap remains J1/SW11 at 0.49mm.

Note the layer subtlety this exercised: a front-side LED land MAY sit over a
back-side pad (different copper layers, no short) — only the aperture must clear
everything. Conflating the two would have forced the ring off the edge entirely.

### Joystick vs the key below it — rotation + ring inset (2026-07-26)

The real RKJXV body is 22.70mm deep in a 19.05mm key cell, so it was always
going to overrun its neighbour once the placeholder footprint was replaced.
Resolved by the user's two suggestions, both of which measure well.

**A correction worth recording.** The first assessment of this clash used the
body's WIDEST cross-section, which is at the board surface, and concluded a Choc
keycap fouled by 0.64mm. That was wrong: a keycap sits 3.5-7mm above the board,
and the RKJXV tapers. Measured per z-band off the STEP model, the face toward
the key is at -17.67 in the 3.5-5.0mm band, not -17.14. Clearance questions about
a tapering part have to be asked at the height of the thing you are clearing.

**`rotate 180`.** The body is not symmetric about the lever axis — in the
keycap band it reaches 10.90mm on the centre-push side and 10.54mm on the
potentiometer side. Turning the narrower side toward the key is free (the
footprint origin IS the lever axis, so the stick does not move) and also swings
the VR1 terminal column inboard toward the MCU its wipers feed. On its own it
converted a 0.10mm graze against a 16.5mm cap into 0.27mm of clearance.

**Ring pushed 1.5mm outboard** (inset 5.5 -> 4.0mm, all 16 LEDs), which frees
depth for the joystick to shift 1mm toward the board edge. The ring's even
spacing is preserved — it is the product feature — and its apertures still sit
2.5mm clear of the board edge.

Final clearances from the joystick body to the key, by height:

| against | clearance |
|---|---|
| switch base 13.95mm | +2.54mm |
| switch top plate 15.00mm | +2.01mm |
| Choc 1U cap 16.5mm deep | +1.26mm |
| Choc 1U cap 17.5mm deep | +0.76mm |

with 1.09mm still spare between the joystick body and the ring aperture on the
other side. The board now scans completely clean: 0 apertures cutting a pad,
0 shorts, **0 courtyard overlaps** (the J1/SW11 0.49mm overlap is gone — the
rotation moved the pad-inclusive courtyard off it), 0 pads outside the outline.

Cost of the fix: the stick sits 1mm off its cell centre. Cheap for a part that
is 3.65mm too deep for the cell it lives in.

## RFC-031 (silkscreen graphics) implementation notes (2026-07-27)

Accepted the same day and implemented in full: `footprint` gains an optional
`silkscreen { … }` block with four closed primitives (`line`, `circle`, `arc`,
`polygon`) plus the two semantic markers. Judgments and one deliberate
deviation:

1. **Marker standoff is measured from the pad's EDGE, not its centre.** The RFC
   says markers expand "at a small, fixed standoff (0.3mm …) from pad `N`'s own
   declared position". Taken literally that places a 0.2mm dot 0.3mm from the
   pad CENTRE — i.e. on top of a pad that is typically 0.9-1.5mm across, so the
   mark would be printed on solderable copper and defeat its own purpose. The
   implementation stands the mark 0.3mm off the pad's edge along the outward
   axis. Verified on the real board: 0 silk-over-pad overlaps across 15 placed
   silkscreen shapes.
2. **Outward direction is the dominant axis away from the pad centroid.** The
   RFC says "on the side of the pad closest to the footprint's outline" without
   fixing a rule. Snapping to whichever of ±x/±y dominates keeps the mark square
   to the package the way a hand-drawn one is, and keeps the result exact — no
   floating-point angle anywhere in the geometry path.
3. **Sizes the RFC does not fix** are conventional and recorded here: triangle
   0.8mm equilateral, cathode band a 0.3mm stroke spanning the pad's own
   cross-extent, arrow 0.9mm long, dot stroke 0.1mm. The RFC's own numbers (dot
   radius 0.2mm, standoff 0.3mm) are used as given.
4. **One expansion, two emitters.** `emit::silk::graphics` expands markers once;
   both the KiCad and IPC-2581 emitters consume it, so they cannot disagree
   about where a mark is — the same single-source discipline the slot-drill work
   used for hole geometry.
5. **IPC-2581's first silkscreen output** reduces every primitive to a filled
   `Contour` polygon rather than mapping shape-for-shape. This is forced by the
   schema, not laziness: `Features` accepts a `StandardShape`, and IPC's `Line`
   and `Arc` belong to the `Simple` substitution group, which cannot appear
   there — `Contour`'s `Polygon` can carry any of them. Reducing a stroke to its
   own outline is faithful (a plotted stroke IS a rectangle of the stroke width);
   only the round caps are dropped, and circles keep the native `Circle`
   element. The document still validates against IPC-2581B1.xsd with the new
   output present.
6. **E812** is a new sub-case in RFC-018's existing E8xx block, per the RFC's
   "reserves new E8xx sub-cases … no new block needed". A polarity marker
   requires at least two distinct electrical pad numbers; repeated physical
   placements of one number do not invent a second terminal to orient toward.

Applied where the user asked — every IC and every diode on openmicro: a pin-1
dot on the STM32 LQFP-48, the USBLC6 SOT-23-6, and std's SOT-23-5 LDO; a cathode
band on the SOD-123 matrix diodes (13 instances). Two bugs worth recording from
the build-out: the femto-mm constants were initially written 1000x too small
(1mm is 10^15 femto, not 10^12), which emitted a 0.0003mm-wide cathode band —
caught by reading the emitted geometry rather than trusting the source; and the
first test asserted a dot centre where the standoff point is, which the emitted
file corrected.

### MCU relocated beneath the cap-touch pad (2026-07-27)

Direct instruction: `mcu` moved from (-18.6, -41) — the top-left spot chosen to
sit near the USB endpoint — to (-28.575, 28.575) `side bottom`, directly behind
the 9mm capacitive sense pad in the bottom-left cell. Geometrically clean: 0
shorts, 0 apertures cutting a pad, 0 same-side courtyard overlaps (the sense pad
is on the front, the MCU on the back, so they share no layer).

Two consequences are recorded because neither is visible in a verdict:

1. **Capacitive sensing is degraded.** All 48 MCU lands now fall inside the
   sense electrode's own outline on the opposite face. A touch electrode works
   by measuring small capacitance changes, so copper and active silicon behind
   it both add fixed parasitic capacitance (lowering the signal-to-baseline
   ratio) and couple switching noise into the measurement. Conventional practice
   is a clear zone behind the electrode, or at most a hatched ground shield —
   not an IC. If the touch input proves insensitive or noisy, the options are:
   move the MCU off the pad, shrink the pad, add a hatched ground shield between
   them, or drive a guard ring. No compiler check covers this: CoHDL models
   connectivity and geometry, not field coupling.
2. **The USB FS pair grew from ~14mm to ~75mm.** Still well within spec for
   12Mbps signalling, but the placement no longer minimises the board's only
   high-speed pair, and the position-aware GPIO map in `openmicro_parts.cohdl`
   was assigned for the old location. The ESD array deliberately did NOT follow
   the MCU: a TVS clamp belongs at the connector, so what matters is the
   unprotected `USBC_*` stretch staying short (6.4mm), which it does.

## Debug port: 2×3 2.54mm SMD socket, bottom edge / bottom layer (2026-07-27)

The 10-pin 1.27mm Minitek127 debug connector is replaced with a generic 2×3
2.54mm SMD **female** socket. Ordinary example maintenance, no compiler change,
but three things are worth recording.

**The pinout shed two signals.** The requested table

    | GND | SWCLK | SWDIO |
    | GND | TXD   | RXD   |

has no 3V3 and no RESET, so `Debug_2x3` declares five required pin roles across
six pads — `GND: 1, 2` (both rows of column 1), `SWCLK: 3`, `TXD: 4`,
`SWDIO: 5`, `RXD: 6` — and `swd.V3V3` / `swd.NRST` were removed from the `V3V3`
and `NRST` nets. A debug probe that expects to sense target voltage or drive
reset can no longer do so through this connector; SWD alone still works, and
`NRST` remains reachable at the reset switch. The pad numbering follows the
socket's own odd/even convention (odd row at y = -2.52mm, even at +2.52mm), so
the user's table read column-by-column maps to pads 1/3/5 then 2/4/6.

**The land pattern is generic, not from a drawing.** `FP_Socket_2x3_254_SMD`
uses the standard 2.54mm vertical SMD pin-socket geometry — 2.54mm column
pitch, rows splayed outward to ±2.52mm, 1×3mm lands — rather than one vendor's
customer drawing. `examples/openmicro/docs/README.md` marks the Amphenol sheet
superseded rather than deleting it. `P_Debug127_Pad` and `FP_Debug_2x5_127_SMD`
are gone; nothing references them.

**One pre-existing placement had to yield.** The socket's courtyard is 8.3mm
tall, and at the bottom edge (`place swd at (0mm, 43mm) side bottom`) it clears
the board outline by 0.35mm — there is no room to move it inward. The bottom
matrix-diode row therefore moved 1.5mm north, from the 8.75mm-south-of-key
pitch to 7.25mm, keeping the row straight rather than kinking `d[10]` alone out
of alignment. The diodes are bottom-side parts tucked under top-side key
switches, so the shift costs nothing. Gap after the move: 0.83mm.

Scans after the change: 66 footprints, 0 shorts, 0 apertures cutting a pad, 0
silkscreen over a pad, 0 same-side courtyard overlaps, 0 pads outside the
outline; `openmicro.xml` validates against IPC-2581B1.

## ESD array to the centreline, quarter-turned, DP/DM channels swapped (2026-07-27)

`U1` (USBLC6-2SC6) moves from `(-7.8mm, -38mm) rotate 180` to
`(0mm, -37.2mm) rotate 270`, still `side bottom`, and the two protected lines
trade ESD channels. Example maintenance again — no compiler change — but the
rotation choice needs recording because it is not the arithmetically obvious
one, and the net edit looks like a mistake if read without the geometry.

**Why 270 and not 90.** The request was "90 degrees counterclockwise". Both
quarter turns put the part's two lead rows across the board instead of along
it, but only one is usable. At `rotate 90` the `*_A` terminals (pads 1/3 — the
connector side of each protected line) land on the row FARTHER from the
receptacle, so the unprotected `USBC_*` copper would have to run past the
protected `USB_*` copper to reach them. `rotate 270` puts pads 1/2/3 on the
row facing the receptacle (y = -38.34) and pads 4/5/6 on the row facing the
rest of the board (y = -36.06), which is the only arrangement where the clamp
sits in series rather than beside the line. The disagreement is a frame
question, not a spec question: CoHDL applies the RFC-025 angle in the board's
own frame after RFC-026's bottom-side mirror, so a back-side part's quarter
turns read reversed from the front. The geometry, not the label, decided it.

**Why the channels are swapped.** The USB-C receptacle's `DP` lands sit at
x = -0.25 and +0.75 (centroid +x) and its `DM` lands at +0.25 and -0.75
(centroid -x). After the turn, pad 1 is at x = -0.95 and pad 3 at +0.95. The
straightforward mapping (`USBC_DP` -> `IO1_A` = pad 1) would therefore put DP
on the DM side and force the pair to cross itself. Routing `USBC_DP` through
the `IO2` channel and `USBC_DM` through `IO1` keeps each line on its own side.
The USBLC6's two channels are identical rail-to-rail diode pairs off the same
VBUS/GND clamp, so nothing electrical changes — only the pad each line lands
on. The `USB_*` side follows the same swap so each line still enters and
leaves through one channel.

**What it bought.** The unprotected connector-to-clamp stretch drops from
6.4mm to 3.26mm per line, symmetric between DP and DM, straight down the
centreline. Board after the change: 66 footprints, 0 shorts, 0 apertures
cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard overlaps, 0 pads
outside the outline; `openmicro.xml` validates against IPC-2581B1. U1's
courtyard clears the receptacle's by 0.29mm and sits in the 3.46mm gap between
SW3 and SW4 (front-side parts, so no shared layer regardless).

**A regeneration note, not a compliance one.** `tools/kicad_board.py` takes
placements from the emitted IPC-2581 document, not from `layout.json`. A
`build` without `--emit ipc2581` leaves that document stale and the script
silently falls back to its staging grid — which is how the first attempt at
this change produced a board with U1 unrotated at the grid origin. Always
regenerate with `--emit ipc2581` before running the board script.

## USB-C locating holes, and the version cascade they forced (2026-07-27)

`FP_USB_C_Receptacle_HRO_TYPE_C_31_M_12` gains the two non-plated locating
holes it was always missing:

    mount_hole 1: non_plated at (-2.89mm, -2.6mm) diameter 0.6mm
    mount_hole 2: non_plated at (2.89mm, -2.6mm) diameter 0.6mm

The footprint's own comment had said these were "omitted (RFC-018 has no
non-plated plating)" — true when it was written, stale since RFC-022 added
`mount_hole` with exactly that vocabulary. The comment is corrected rather than
left as a standing excuse.

**Source.** HRO's customer drawing (`examples/openmicro/docs/type-c-31-m-12.pdf`,
rev 2020-12-08) shows two Ø0.60 circles in RECOMMEND P.C.B LAYOUT, dimensioned
5.78 apart, on the same y as the `4.18`/`5.79` chain that also fixes the SH
tabs. KiCad's `USB_C_Receptacle_HRO_TYPE-C-31-M-12` puts them at (±2.89, -2.6)
and agrees with our footprint on every other coordinate, which is the
cross-check that the datum is shared. Diameter follows the drawing (0.60) not
KiCad (0.65): the bosses are Ø0.50 and 0.60 is the vendor's stated fit, with
the trade-off noted in the footprint comment for fabs that cannot hold ±0.05.

**The cascade is the part worth recording.** `lib/std` is an ordinary RFC-029
package, so changing one footprint changed std's content hash and both examples
refused to build with E1103 — correct behaviour, since a locked version's bytes
are immutable. That forced a real chain, all of it through `cohdl update`, never
by hand-editing a lock:

1. `std` 0.2.0 -> **0.2.1** (footprint content changed; API unchanged, so patch).
2. Both examples repinned to std 0.2.1.
3. **`lib/passive` broke, and no example's `check` caught it.** passive pins
   `std = "0.2.0"`, which no longer existed on disk; `cohdl check lib/passive`
   reported E1102 while `cohdl check examples/openmicro` reported no errors. A
   project resolves each of its dependencies against the library root directly,
   so a dependency's own manifest is not re-resolved as part of the dependent's
   verdict. Worth knowing: green examples do not prove the libraries they
   depend on still resolve standalone.
4. passive repinned to std 0.2.1, which changed passive's hash, so passive
   0.1.0 -> **0.1.1**, and both examples repinned again. `cohdl update` also
   wrote passive its first `cohdl.lock`.

**Pre-existing, untouched.** `cohdl check lib/std` fails E1104 ("declares no
`[dependencies]` — RFC-029 requires an exact std pin") because std cannot pin
itself. This reproduces on the committed tree and is not a consequence of the
bump; it is left alone as out of scope here.

Both boards regenerated and clean: openmicro 66 footprints, rpi-pico2 8, each
with 0 shorts, 0 apertures cutting a pad, 0 silkscreen over a pad, 0 same-side
courtyard overlaps, 0 pads outside the outline. The new NPTH clear all copper
on both. Both IPC-2581 documents validate; 21 test binaries pass.

## Reset switch removed; HSE tank placed on the back (2026-07-28)

Two independent example edits, no compiler change.

**Reset switch gone.** `inst rst: SW_RESET` and its two connections (`rst.A` on
NRST, `rst.B` on GND) are removed, and with them the now-dead `ResetSwitch`
device, the `SW_RESET` part, `FP_SW_Reset`, and the `P_ResetSw` pad — nothing
else referenced any of them. `mcu.NRST` moves to the `nc:` list rather than
being silently dropped: RFC-002 exhaustiveness would have caught a dropped
required pin, but `nc:` is also the honest statement, since the pin is now
deliberately unconnected rather than merely unrouted.

**Electrical consequence, recorded because it is not a verdict.** The board now
has NO reset path at all. The 2×3 debug socket carries no RESET line (that was
the requested pinout), so NRST sits on the STM32F072's internal pull-up alone
and the only way to reset the part is a power cycle. ST additionally recommends
a 100nF from NRST to ground for noise immunity, which this board no longer has.
If in-system reset or that filter is wanted back, the cheapest restoration is a
100nF to GND on NRST plus either a button or a RESET pin on the debug socket;
no CoHDL check covers either, because both are conventions rather than
connectivity errors.

**HSE tank to the back.** `xtal`, `c_x1`, and `c_x2` were unplaced (staged
off-board by the board script). They now sit on the bottom side west of the
MCU:

    place xtal at (-36.5mm, 29.07mm) rotate 180 side bottom
    place c_x1 at (-36.5mm, 32mm) rotate 180 side bottom
    place c_x2 at (-36.5mm, 26.2mm) side bottom

West is the only free direction that is neither under the capacitive touch
electrode (which stops at x -33.32) nor under an LED light window; the crystal
sits 1.28mm off the package edge, centred on the y midpoint of PF0/PF1.

`rotate 180` is a real choice, not a default. The 3225 land puts XIN (pad 1)
and XOUT (pad 3) on OPPOSITE corners, so no quarter turn faces both terminals
at the MCU — the only question is which diagonal points east. At 180 each
terminal ends up 0.55mm in y from the pad it feeds (XIN at -35.40, 29.87 vs
PF0 at -32.74, 29.32); unrotated, both are 1.05mm off. The load caps flank the
crystal north and south, each beside the terminal it loads, ground ends turned
outward (C28's return leaves west, C29's east) so the two returns do not meet
under the tank.

Board after both changes: 69 footprints — up 3, since the reset switch had
never been placed and was only ever staged off-board — with 0 shorts, 0
apertures cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard
overlaps, 0 pads outside the outline. `openmicro.xml` validates against
IPC-2581B1; 21 test binaries, `cohdl fmt --check`, and `cargo fmt --check` all
clean. Still staged off-board and untouched here: 27 decoupling/bulk caps, 3
resistors, and the LDO.

## Bypass capacitors placed, and the RFC-024 gap that surfaced (2026-07-28)

26 of the 27 bypass capacitors now sit on the same face as the part each one
bypasses: 18 on the back (MCU ×3 100nF, the VDDA 1µF, USB-C, 13 key LEDs) and
8 on the front with the underglow ring. Each is 1.5–3.5mm from its own pin.
The 27th is `c_bulk`, held back for the reason given below. This
is the electrically meaningful arrangement — a bypass cap on the far face
reaches its pin through two vias, and that inductance is most of what the cap
was there to avoid.

**Why the example lost `decouple()`.** The caps were created inside
`fn decouple(vdd: Pin, gnd: Pin)`, and a fn-created instance has no
user-visible name: its path is `OpenMicro::__fn0_decouple::c`, a compiler
internal whose numbering depends on expansion order. `place` takes an instance
name. So placing them required hoisting all 27 into named design-level
instances, and with no call sites left `fn decouple` became dead and was
removed. The cost is real and worth naming: this example no longer demonstrates
RFC-006 fn expansion or RFC-028's "one attribute annotates every call site".
Both remain covered by `tests/quilter.rs`. The wiring is unchanged — the
`net _: vdd, c.A` pairs always merged into the same VBUS/V3V3/GND rails, so
the hoisted caps simply join those nets directly, and the emitted netlist has
the same membership it had before.

**A real RFC-024 gap, closed.** `#[bypass(key_leds[0].VDD, 100nF)]` did not
parse: the attribute's argument grammar stopped at a bare identifier, so the
`[` was a syntax error. RFC-024's accepted text is explicit that an array
element is a valid instance reference "EVERYWHERE an ordinary instance
reference is valid", and `#[bypass]`'s first argument is one — so this was a
deviation, not a design limit. Without it the choice was to place the LED caps
or declare them, not both, and 21 Quilter `bypass_capacitors.csv` rows would
have been lost.

The fix deliberately does NOT add a second index resolver. `handle_placement`
already contained the RFC-024 logic inline; it is now extracted as
`Expander::indexed_local` and both `place` and `#[bypass]` call it, so the two
cannot disagree about which element an index names. Only the "you named a bare
array" help text differs per caller, since the advice reads differently in a
`place` than in an attribute. E211 covers a range/index-list target (one
capacitor cannot bypass three pins) and E202 an out-of-bounds index — the same
codes, from the same code, as everywhere else. Four tests in
`tests/quilter.rs`; the error-code registry's E211 row now names `#[bypass]`.

Verification that the hoist changed no facts: the emitted
`bypass_capacitors.csv` target set is byte-identical to the committed one —
same 32 (component, pin, capacitance) rows, only the capacitor designators
renumbered.

**One capacitor is deliberately still staged.** `c_bulk`
(`#[bypass(ldo.VIN, 4.7uF)]`) has no placement, because the LDO it belongs to
has none either. An input bulk capacitor positioned away from its regulator is
not a bypass capacitor in any useful sense, so there is no correct answer here
until the LDO is placed; guessing a spot would have looked complete while being
wrong. Also still staged, and out of scope here: the LDO, and R1–R3 (the CC
pulldowns and the BOOT0 resistor).

**Crystal nudged 0.7mm.** `xtal`/`c_x1`/`c_x2` moved from x -36.5 to -37.2.
VDDA and VSSA leave the MCU's west face and their two bypass caps have to stand
in that gap; at -36.5 it was 1.23mm and a 0402 does not fit, which pushed the
VDDA decoupling 4.5mm out. The oscillator pays 2.66mm -> 3.36mm on its XIN run,
which is the cheaper of the two.

Board: 95 footprints placed (up from 69), 5 staged. 0 shorts, 0 apertures
cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard overlaps, 0 pads
outside the outline. IPC-2581 validates; 21 test binaries, `cargo fmt`,
clippy, and `cohdl fmt --check` across all four packages clean.

## Key LEDs turned 180° for the data chain (2026-07-28)

Every `key_leds[i]` placement gains `rotate 180`. The SK6812MINI-E land puts
DOUT (pad 2) and DIN (pad 4) on opposite columns — DOUT west, DIN east as
drawn — and the per-key chain runs left-to-right along each row. Unrotated,
every in-row hop left the west side of one LED and had to reach the east side
of the next, doubling back past both bodies: 24.5mm of net to cross a 19.05mm
pitch. Turned around, DOUT faces the next LED's DIN directly and the hop is
13.7mm of straight run.

The trade is real and worth stating rather than only quoting the total. Nine
in-row hops each save 10.9mm; the three end-of-row wraps each cost 11.2mm
more, because the chain now finishes a row at the far side and still has to
restart at the near side of the next. Net 349.6mm -> 285.3mm, and — the part
that actually matters for routing — nine traces stop crossing the footprints
they came from.

Those three wraps (48.2, 65.9, 48.2mm) are inherent to a chain that always
restarts at the left. Wiring the chain as a serpentine — row 0 left-to-right,
row 1 right-to-left, and so on — would cut each to roughly one pitch, but it
renumbers which physical LED is Nth in the data stream, which is a firmware
mapping change rather than a layout one. Not done here; it is a net-list edit
(`LED_D*`), not a placement edit, if it is ever wanted.

Nothing else moved: the light window and courtyard are both centred and
symmetric, so a half turn leaves the apertures and the bypass caps beside them
exactly where they were. Board still 95 footprints placed, 0 shorts, 0
apertures cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard
overlaps, 0 pads outside the outline; IPC-2581 validates; 21 test binaries pass.

## MCU moved out from under the touch electrode (2026-07-28)

`place mcu` goes from `(-28.575mm, 28.575mm)` to `(-22mm, 38.5mm)`, still
`side bottom` — right 6.6mm and down 9.9mm.

**Why the old position was wrong, in a way no check catches.** The 2026-07-27
entry recorded that putting the MCU under the 9mm capacitive electrode cost
touch sensitivity, and listed the escape routes. The decisive problem turned
out to be a different one: a QFP-48 on 0.5mm pitch is routed almost entirely
through vias inside its own outline, and the package (9.49mm) is very slightly
larger than the electrode (9mm). Every fanout via would have had to punch
through the electrode. That is not a sensitivity trade-off, it is a layout
impossibility — and CoHDL models connectivity and geometry, not "this copper
is a sensor", so nothing in the pipeline could have said so.

**Why right AND down.** Clearing the electrode in x alone would need the
package east of x -23.83, which drives it into `sw[10]`'s plated leg at
x -15.57..-13.47 — the bottom-left key's through-hole leg blocks both faces.
Clearing in y works: the corridor between the bottom-edge underglow LEDs at
x -33 and x -11 is free from y 33.32 down to the board edge. The package now
sits at x -26.75..-17.25, y 33.76..43.24, clear of the electrode by 0.44mm,
with its whole footprint free for fanout.

**Seven parts followed it.** The crystal, its two load caps, and the four MCU
bypass caps are all anchored to specific MCU pads, so they were re-solved
against the moved package rather than left behind. The crystal again stops
short of the package (4.4mm out, not hard against it) to keep a 0402-wide
column open for the VDDA/VSSA pair, which leaves the same west face two pins
along. That reservation is free: XIN still lands 3.32mm from PF0 and XOUT
5.50mm from PF1 — within 0.05mm of the pre-move distances.

**Two lengths grew, both accepted.** The TOUCH sense line is now 14.9mm of
routed length, where it was sub-millimetre with the part sitting on the
electrode; 15mm is ordinary for a sense trace, and it is a straight run down
the same corner. The USB FS pair grows to ~78mm, still well inside spec for
12Mbps signalling.

Board: 95 placed, 5 staged (`C26`, `R1`–`R3`, `U2`). 0 shorts, 0 apertures
cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard overlaps, 0 pads
outside the outline. IPC-2581 validates; 21 test binaries pass.

## Bypass capacitors handed to the auto-placer (2026-07-28)

All 26 `place` statements for the bypass capacitors are removed; the caps now
carry no coordinates and the board script stages all 27 outside the outline
(fully outside — none straddles the edge). This reverses the placement work
recorded earlier the same day, deliberately and at the board author's
direction, and the reasoning is worth keeping because it is the point of
RFC-027 rather than a change of mind about geometry.

Every one of these caps carries `#[bypass(target, value)]`, and that attribute
is precisely what an auto-placer consumes: `bypass_capacitors.csv` tells it
which pin each capacitor serves, so it can place each one against the fanout
that actually exists after routing. A hand-locked `place` is a guess made
before any routing exists, and it overrides the tool that has better
information. Removing the coordinates is not losing work — the constraint is
the work, and the constraint is unchanged: the emitted
`bypass_capacitors.csv` is byte-identical in its (component, pin, capacitance)
set to the committed baseline, 32 rows over 27 capacitors.

What deliberately stayed placed: `c_x1` and `c_x2`. They are the crystal's
LOAD capacitors, not bypass capacitors — they carry no `#[bypass]`, they are
part of the `#[crystal_oscillator]` group, and an oscillator tank is the one
thing on this board whose geometry should not be delegated.

The named-instance refactor from earlier today is kept rather than reverted to
`decouple()`. It is no longer strictly required — nothing needs a `place` name
right now — but naming is what makes a per-cap `place` possible at all, so it
stays for the day one capacitor has to be pinned by hand. The RFC-024
`#[bypass(NAME[i].PIN, …)]` fix stands on its own regardless: it closed a real
deviation from accepted text.

Board: 69 placed, 31 staged (27 bypass caps, `R1`–`R3`, `U2`). 0 shorts, 0
apertures cutting a pad, 0 same-side courtyard overlaps among the placed set.
`layout.json` carries 69 locked placements, and the only capacitors among them
are the two crystal load caps. IPC-2581 validates; 21 test binaries pass.

## 2026-07-28 — underglow ring halved to 8 LEDs, 2 per side at the edge thirds

`ambient_leds` goes from `[RGB_SK6812; 16]` to `[RGB_SK6812; 8]` at the board
author's direction: two per side instead of four. The positions are the THIRDS
of each 95mm edge, x = ±15.833 (`-47.5 + 95/3` and `-47.5 + 2*95/3`), also at
the author's direction, and they are the right choice for a reason worth
recording.

Halving the count is not the same as deleting two LEDs from each side, and the
ring had to be respaced rather than thinned. Keeping the old inner pair (±11)
would leave 36.5mm of dark at every corner; keeping the old outer pair (±33)
would leave a 66mm dark gap through the middle of every edge. Respacing was
therefore required, and among even spacings the thirds are what keep the ring
regular AROUND THE CORNERS as well as along an edge: 31.67mm between
neighbours on an edge and 39.13mm diagonally across each corner, a 1.236x
spread, verified against the generated board. Quarter-points (±23.75) would
have spaced an edge 47.5mm and a corner 27.9mm — a 1.70x spread, with the
corners reading as bright pinch points against dim edge centres.

This is a deliberate exception to the standing "do not move the ambient ring"
constraint, and does not weaken it. That constraint exists because the ring is
the product feature a user actually sees, so the back side gets arranged
around its apertures rather than the other way round. It forbids moving the
ring to make room for something else; it does not forbid the author changing
what the ring itself is. Nothing else on the board moved to accommodate this.

Everything downstream of the count followed:

- The chain shortened to `UG_D0`–`UG_D7`; `nc:` now floats
  `ambient_leds[7].DOUT`. Chain order and per-side rotations are unchanged —
  still clockwise from the top-left, still an extra 180 degrees on the top and
  left edges so DOUT faces the next LED's DIN.
- `VBUS`/`GND` fan-out narrowed to `ambient_leds[0..=7]`, and the rail comment
  from 29 RGB LEDs to 21.
- The ring's rail capacitors went from 8 to 4 (`c_uled8`/`10`/`12`/`14`
  dropped). The documented ratio — one 100nF per LED PAIR — is unchanged; at
  two LEDs per side a pair is now exactly one board edge, so this reads as one
  rail cap per side. `bypass_capacitors.csv` confirms it: `C43`/`C47`/`C48`/
  `C49` bypass `LED1`/`LED9`/`LED11`/`LED13`, the top/right/bottom/left pairs'
  leaders. 28 rows over 23 capacitors, down from 32 over 27.
- `#[high_current(500mA)]` on VBUS is unchanged and deliberately so: 500mA is
  the USB-C sink budget without PD negotiation — a supply-side limit, not a
  sum over the LEDs — so fewer LEDs does not change it.

The firmware is part of the same change, not a follow-up. `led_task` drove a
`[Grb; 16]` ring, and an 8-LED chain would have ignored the extra eight frames
silently while the hue rotation covered only half the wheel around the board.
The array is now `[Grb; 8]` with the hue step raised 16 -> 32: the product of
count and step is what has to stay at 256, not either number alone. Pin-map
and brightness-cap comments in `main.rs`, `ws2812.rs`, `fw/README.md` and the
companion app's blurb follow the 29 -> 21 LED total.

Designators: the eight removed LEDs and four removed capacitors went to
`design.lock`'s `[tombstones]`, so `LED3`–`LED8`, `LED15`, `LED16`, `C44`–
`C46` and `C50` are retired rather than reused. The gaps in the BOM are
RFC-005 stability working as specified, not an allocator fault.

Board: 61 placed, 27 staged (23 bypass caps, `R1`–`R3`, `U2`). 0 shorts, 0
apertures cutting a pad, 0 silkscreen over a pad, 0 same-side courtyard
overlaps, 0 pads outside the outline. Ring geometry read back off the
generated `.kicad_pcb` matches the source exactly, all eight on the front.
`cohdl check` clean, `fmt --check` canonical, IPC-2581 validates against
`IPC-2581B1.xsd`, output byte-stable across a repeat build, 21 test binaries
pass, `cargo fmt --check` clean on both the compiler and the firmware crates.

## 2026-07-28 — MCU + HSE tank 3mm west

At the board author's direction, `mcu` moves from x -22 to -25 and the whole
oscillator tank follows by the same 3mm (`xtal` -30.535 -> -33.535, `c_x1`
-29.435 -> -32.435, `c_x2` -31.635 -> -34.635). y is untouched on all four.
Moving the tank rigidly with the MCU is not optional — the load capacitors set
CL and the loop geometry is the point of the group — and the emitted board
confirms it held: PF0 -> XIN is still 3.318mm and each load cap still sits
1.679mm from the terminal it loads, unchanged to the micron.

Two things make west the available direction and a useful one.

Available: the 0.44mm that separates the MCU from the cap-touch electrode is a
separation in **y**, so no pure-x move can consume it. The measured figure
after the move is identical to before, 0.435mm. The earlier note that "moving
right-and-down is the only direction that clears it" was about escaping an
overlap, and remains true as history; it does not constrain travel along x
once the part is clear.

Useful: the package corner was overlapping `ambient_leds[5]`'s light window by
0.303mm. That window is an Edge.Cuts aperture — a real hole through the board,
not a keep-out — so although no PAD was cut (the aperture-vs-pad scan was
clean before the move and after it), the corner of a QFP-48 sat over open air,
and a QFP-48 on 0.5mm pitch is routed almost entirely by vias inside its own
outline. It now clears that window by 2.697mm. The next aperture east,
`key_leds[10]`'s window, goes from 6.0mm to 9.0mm of clearance on the face
most of the GPIO leaves by.

A first draft of the source comment claimed the east-face gain as "6.0mm to
9.0mm" against `key_leds[10]` alone, which read as if that were the nearest
obstacle east. It was not: `ambient_leds[5]`'s window is nearer, at 2.697mm,
and only failed to register as "east" before the move because the courtyard
was overlapping it. The comment in `main.cohdl` names both apertures and both
distances. Recorded because the corrected number is the one that bounds east
fanout.

Westward travel is capped at roughly 7.3mm by the crystal reaching mount hole
`H4` — a plated M2 barrel, so it obstructs the back side too despite the
footprint being front-side. The 3mm taken leaves 4.645mm of that headroom.

Board: 61 placed, 27 staged; 0 shorts, 0 apertures cutting a pad, 0 silkscreen
over a pad, 0 same-side courtyard overlaps, 0 pads outside the outline.
`cohdl check` clean, `fmt --check` canonical, IPC-2581 validates, output
byte-stable across a repeat build, 21 test binaries pass, `cargo fmt --check`
clean.

## 2026-07-28 — DEVIATION: `rotate` accepts any whole degree (RFC-020 superseded)

**This is a deliberate deviation from Accepted RFC text, directed by the board
author after the restriction was raised and explained.** RFC-020 §Design states
`ANGLE is one of {0, 90, 180, 270}`, "a closed four-value set — not an
open-ended angle type", and its Non-goals name both "arbitrary-angle rotation"
and "rotation math/collision reasoning". RFC-025 reuses that set for pad
placements. Both are now widened to any whole degree in `0..=359`, for `place`
(E1007) and `pad … rotate` (E811) alike. The RFC text is superseded pending an
RFC on conol.ai; nothing here should be read as the RFC having said this.

`360` and above stays an error rather than reducing mod 360. A full turn is `0`,
so nothing is lost, and `rotate 450` is far likelier a slip than a deliberate
90. Fractional degrees are still rejected — "any rotation degree" is read as any
whole degree, and a decimal angle would need a different literal type.

### Why this needed real work rather than deleting a `matches!`

Deleting the closed-set check is four lines. The reason RFC-020 closed the set
is the arithmetic behind it, and that is what had to be built.

**Determinism.** The Constitution's hard constraint is same source → same
netlist **bytes**. `f64::sin` resolves to the platform libm, whose final bit is
not guaranteed identical across macOS, glibc and musl, and a rotated pad
coordinate is emitted geometry. So `src/trig.rs` is a checked-in fixed-point
sine table (`tools/gen_trig.py`) and integer arithmetic over it — deterministic
by construction, not by luck. Scale is `2^64`, and only 0..=90 is tabulated with
the other quadrants derived by symmetry, which is what keeps `sin(90°)` exactly
`2^64` and `cos(90°)` exactly `0`.

Note `emit::silk` and `emit::kicad_mod` already use `f64` trig to tessellate
arcs and circles. That precedent was deliberately NOT followed: those call sites
round a radius-scaled result onto a femtometre grid orders coarser than the
error, whereas a rotation scales a coordinate already at full magnitude.

**Byte-compatibility, verified not assumed.** Because the table is exact at the
cardinals, every placement authored under the closed set emits identical bytes.
Regression-tested by rebuilding both examples: `rpi-pico2` produced a zero-byte
git diff across all artifacts, and `openmicro`'s four artifacts hashed
identically before and after the compiler change.

**Overflow.** `emit::geom::MAX_GEOM_FEMTO` is `10^30`; multiplied by a `2^64`
trig value the product needs 164 bits, so an `i128` multiply would wrap. Rather
than shrink the documented coordinate bound or the trig precision, the multiply
goes through a 128×128→256-bit intermediate and reduces by a shift (which is why
the scale is a power of two). Both the bound and the precision keep full
strength.

**Collision reasoning.** RFC-020's other non-goal is the load-bearing one. Every
placement check here — and every scan run against the generated board — compared
axis-aligned bounding boxes, which are exact at 90° multiples and wrong at 45°,
where a square's bbox is 41% oversized. `emit::silk`'s pad-extent calculation is
now `trig::bound_half_extents`, the bounding box of the turned pad: it reduces
to the old w/h swap at 90/270 and to the identity at 0/180, and is conservative
between, which is the only safe direction for a standoff. **The compiler still
does no placement collision checking**, exactly as before — that gap is
unchanged, but it is more consequential now, because a designer can author an
angle whose clearances no bbox can judge.

### The MCU at 45 degrees — what it actually cost

`place mcu … rotate 45 side bottom` is the first non-cardinal placement on
either board. Applying it exposed how much the closed set had been hiding.

At the MCU's existing `(-25, 38.5)` the turn was **not** legal, and the
bbox-based scan said it was. Exact polygon geometry found two real violations:
the diamond's south vertex reached 1.41mm INTO the cap-touch electrode — the
precise defect the earlier right-and-down move existed to remove — and its west
vertex touched the crystal's courtyard to 0.0003mm. The first scan missed the
second because it tested intersection AREA, and two courtyards touching exactly
have none; the scan now enforces a minimum clearance instead.

A 9.39mm courtyard turned 45° is a 13.28mm diamond, and the pocket between the
electrode (y 33.27) and the board edge (47.55) is 14.28mm — **1.00mm of slack in
total**. The only band that fits is cy 40.35..40.60; `40.5` is its middle,
giving 0.59mm to the electrode and 0.41mm to the board edge.

The tank followed. PF0/PF1 left a flat west face and now leave a diagonal one
pointing down-left, and the obvious move — tank on the pin normal — is
impossible: that normal runs at the board corner and would put the crystal at
y 47.9, off the edge. It stays west, beside the diamond. The oscillator pays
1.29mm: PF0→XIN + PF1→XOUT is 10.10mm where it was 8.82mm. Accepted for an 8MHz
HSE with 15pF loads, and recorded as the largest single cost of the rotation.

Other measured effects: TOUCH sense 14.9 → 16.9mm; USB FS pair ~78 → ~77.7mm
(unchanged in substance). Crystal-to-MCU 0.368mm is now the tightest same-side
courtyard pair on the board.

### Verification

Exact-geometry scan (true polygon distance, 0.15mm minimum clearance): 0
courtyard violations, 0 plan-view overlap with the touch electrode, 0 shorts, 0
apertures cutting a pad, 0 silkscreen over a pad, 0 pads outside the outline.
The scan's detector was itself positive-controlled by deliberately stacking
`c_x1` on the MCU, which it reported as a 1.7205mm² overlap.

`trig.rs` carries 9 unit tests (cardinal exactness, table-path/fast-path bit
equality, Pythagorean identity across all 360 degrees, inverse-rotation
cancellation, no overflow at `MAX_GEOM_FEMTO`). `tests/layout.rs` and
`tests/side_rotate.rs` gained arbitrary-angle acceptance, out-of-range
rejection, and an end-to-end IPC-2581 check that a 30° component places its pads
at `x="-3.133974596215561"` — the fixed-point table's own value, byte-identical
on repeat. 21 test binaries pass, `cargo fmt --check` and `clippy --all-targets`
clean, `cohdl check`/`fmt --check` clean on both examples, openmicro output
byte-stable across a repeat build, IPC-2581 validates against `IPC-2581B1.xsd`.

## 2026-07-28 — MCU placement re-derived after a Quilter routing failure

Quilter failed to route the board. This is the investigation and the fix.

### What the evidence said, including where it contradicted me

`out/` still holds the artifacts of a run that DID route (`openmicro.pre-quilter
.kicad_pcb` -> `openmicro.quilter-routed.kicad_pcb`, 1978 segments, 222 vias, 2
copper layers), so there was a working baseline to diff against rather than a
guess to make.

**First hypothesis, wrong:** that the 27 bypass capacitors staged outside the
outline could not be routed to. The baseline refutes it — its pre-quilter input
had **48** footprints staged off-board and Quilter moved every one onto the
board. Staging is the supported workflow and the earlier instruction to stage
the bypass caps was correct. Recorded because it was checked and disproved, not
assumed.

**Second finding, also a correction:** the baseline had the MCU itself staged,
and Quilter placed it at (-20.8, -41.5) rot 0 on the TOP side. That looks like
an argument for unlocking the MCU — but Quilter placed all 13 diodes on the top
side too, and it does not flip parts: they were top-side because they were
STAGED top-side. On a macropad the top is the keycap side. So wholesale
unlocking would move the MCU and the diodes to the wrong face, and "hand it all
back to the placer" is not available for this design. The fix had to be the
MCU's own placement.

The real regression is scope: the baseline locked 50 parts (mounting holes,
connectors, LEDs, switches, touch pad — user-facing only) and left 48 free. The
current board locks 61, having added the MCU, the ESD array, the crystal, both
load caps, all 13 matrix diodes and the debug socket. The diodes and the ESD
have defensible reasons; the MCU's hand-placement did not survive measurement.

### Measurement, and a metric that had to be thrown away

A first scoring pass hand-derived the mirror-then-rotate transform to predict pad
positions. It could not reproduce the loaded board to better than 7mm, so every
number it produced was discarded. The replacement mutates the real footprint
through pcbnew (`SetPosition`/`SetOrientationDegrees`/`Flip`) and reads the pad
centres back, with a self-test that asserts a 0.000000000mm round-trip before any
candidate is scored. The bug it caught was mine — `ORIG` stored a boolean where
the function expected `"B"`/`"F"`, so the self-test silently flipped the part.

Scoring is total net length plus, weighted higher, the fraction of the 48 pads
that can escape the package outline 2mm without hitting an obstacle. On a
**2-layer** board a 0.5mm-pitch QFP-48 is routed almost entirely by vias inside
its own outline, so the free collar is the routing resource, not a nicety.

| placement | net length | pads escaping | collar free | TOUCH | USB DP | edge |
|---|---|---|---|---|---|---|
| pocket, 45 deg (failing) | 1691.7mm | — | 56% | 16.9mm | 77.7mm | 0.36mm |
| pocket, square (earlier) | 1658.8mm | — | 84% | 14.2mm | 78.9mm | 4.30mm |
| **left margin, rot 90** | **1626.2mm** | **100%** | — | **7.8mm** | **68.6mm** | **2.81mm** |

The 45-degree turn was the worst option on every axis, and measurably so: a
9.39mm courtyard becomes a 13.28mm diamond, the pocket it sat in is 14.28mm, and
the diamond ate its own fanout collar (56% free against 84% for the same part
square). The diagonal escape was the point of the turn; on two layers the collar
it consumed was worth more.

### The fix

`place mcu at (-40mm, 24mm) rotate 90 side bottom` — the left margin at mid
height, the only candidate where all 48 pads escape. The tank follows onto the
MCU's +y face, where PF0/PF1 now emerge into open margin instead of pointing at
a board corner: `xtal (-39.5, 31.5) rotate 180`, caps at (-36.2, 33.8) and
(-42.8, 31.5).

Every net that matters got SHORTER, which is unusual for a placement move and is
why this is recorded as a fix rather than a trade: TOUCH sense 16.9 -> 7.8mm, USB
FS pair 77.7 -> 68.6mm, HSE loop 10.10 -> 6.90mm — shorter than the 8.82mm it was
before the 45-degree experiment — total net length 1691.7 -> 1626.2mm. The
via-through-electrode constraint is now satisfied by 1.94mm of X separation
rather than a 0.44mm Y margin, so it holds by construction.

### Two things this does NOT fix, both flagged rather than decided

1. **The board is 2 copper layers.** `tools/kicad_board.py` never calls
   `SetCopperLayerCount`, so every generated board is 2-layer by default. For a
   0.5mm-pitch QFP-48 plus 88 components, 66 nets, two WS2812 chains and parts
   on both faces, that is the most likely remaining cause of a routing failure,
   and no placement can compensate for it. A 4-layer variant already exists in
   `out/openmicro-manual-placement.kicad_pcb`. Raising the layer count is a
   product decision, not a placement one, so it is left to the board author.
2. **The GPIO map is stale.** `openmicro_parts.cohdl`'s position-aware ROW/COL/
   feature-pin assignment was derived for a long-superseded TOP-left MCU. It has
   now been wrong for three placements running, and re-deriving it is the obvious
   next lever.

### Verification

Exact-geometry scan (true polygon distance, 0.15mm minimum clearance): 0
courtyard violations, 0 plan-view overlap with the touch electrode, 0 shorts, 0
apertures cutting a pad, 0 silkscreen over a pad, 0 pads outside the outline.
61 placed / 27 staged. `cohdl check` and `fmt --check` clean, output byte-stable
across a repeat build, IPC-2581 validates against `IPC-2581B1.xsd`, 21 test
binaries pass, `cargo fmt --check` clean, `rpi-pico2` untouched.

## 2026-07-28 — perimeter underglow ring removed entirely

At the board author's direction, after Quilter still failed to route: the
underglow ring is deleted rather than reduced again. `ambient_leds` and its four
rail capacitors are gone, along with the `UG_D0`–`UG_D7` chain and every ring
`place`.

### Why this is a routing change and not just a feature cut

Each SK6812MINI-E is REVERSE-MOUNT: it fires through the board, so its footprint
carries a 3.4 x 3.0mm `window` — an Edge.Cuts aperture, a real hole. A hole
blocks BOTH copper layers. Eight of them sat around the perimeter at the edge
thirds, which is precisely the outer routing channel a 2-layer board needs for
the rail and matrix nets that have to get past the key field.

Measured with one script across both boards, so the two numbers are comparable:
the outer 8mm perimeter collar goes from **83.9% to 88.5% free**, and interior
Edge.Cuts items from **29 to 13**. Footprints 88 -> 76, nets 66 -> 58. No
rearrangement of the ring could have returned that channel — only removing it.

An earlier note in this file recorded that the ring's even spacing "IS the
feature" and that the back side gets arranged around its apertures rather than
the other way round. That reasoning stands as written and is now moot: the
feature was cut, so nothing is being arranged around it any more.

### Consequences carried through

- `mcu.LED_DATA_UG` (PA0) now drives nothing and moves to `nc:` — RFC-002
  requires every `required` pin be resolved, so this is not optional. PA0 is
  left free rather than repurposed, so restoring a ring is a net edit plus a
  `place` block, nothing structural.
- `bypass_capacitors.csv` drops the four ring rail caps: 25 rows, from 28.
- Designators: `LED1`, `LED2`, `LED9`–`LED14` and `C43`, `C47`–`C49` go to
  `design.lock`'s `[tombstones]`. The surviving 13 key LEDs keep `LED17`–`LED29`
  unchanged — RFC-005 stability, which is why the BOM reads from LED17.
- Firmware: `led_task` loses its second chain and its `led_ug` argument, the
  `[Grb; 8]` ring array and hue rotation are gone, and `ws2812.rs` is documented
  as a single-chain driver. Builds clean for `thumbv6m-none-eabi` with no
  warnings. LED totals 21 -> 13 across `fw/README.md` and the companion app.

### Still outstanding, unchanged by this

The board remains **2 copper layers** (`tools/kicad_board.py` never calls
`SetCopperLayerCount`), and `openmicro_parts.cohdl`'s position-aware GPIO map is
still derived for a long-superseded top-left MCU. Both were flagged in the
previous entry and neither is a placement question.

### Verification

Exact-geometry scan: 0 courtyard violations at 0.15mm minimum clearance, 0
plan-view overlap with the touch electrode, 0 shorts, 0 apertures cutting a pad,
0 silkscreen over a pad, 0 pads outside the outline. `cohdl check` and
`fmt --check` clean, output byte-stable across a repeat build, IPC-2581 validates
against `IPC-2581B1.xsd`, 21 test binaries pass, `cargo fmt --check` clean on the
compiler and the firmware, `rpi-pico2` untouched.

## 2026-07-28 — underglow ring restored; GPIO map re-derived for the current MCU

Two changes at the board author's direction: the 8-LED perimeter ring comes
back at the edge thirds, and the position-aware GPIO assignment — flagged stale
in the two previous entries — is finally re-derived.

### The ring costs what it cost before, and that is now recorded in the source

Restored exactly as specified: 2 per side at the thirds of each 95mm edge
(x = ±15.833), 4.0mm inset, top and left carrying the extra 180 degrees so each
DOUT faces the next DIN. `mcu.LED_DATA_UG` leaves `nc:` and drives the chain
again; the four per-side rail capacitors return (`bypass_capacitors.csv` back to
29 rows).

Its price is unchanged and is not the parts: each SK6812MINI-E is reverse-mount,
so it carries a 3.4 x 3.0mm `window`, and a window is an Edge.Cuts aperture that
blocks BOTH copper layers. Eight of them narrow the outer routing collar from
88.5% to 83.9% free. That measurement is now written into `main.cohdl` beside
the ring's own `place` block rather than living only in this ledger.

### GPIO: the honest result is that length was never the lever

The map was assigned position-aware for a TOP-left MCU and had survived three
placements without being revisited. Re-derived as a minimum-cost assignment of
functions onto free LQFP-48 pads.

Total pin-to-target length barely moves: **1061.3 -> 1016.8mm, 4.2%**. That is
the real finding, and it contradicts how the staleness was described in the two
previous entries — all 48 pads live inside a 9.4mm square while the targets are
spread over a 95mm board, so which pad a function gets cannot change the long
run. Re-deriving the map for LENGTH would have been close to pointless.

What it does change is escape DIRECTION. A pad whose bearing to its own target
exceeds 90 degrees has to wrap its trace back around the package, and on a
2-layer 0.5mm-pitch QFP-48 fanout that is what congests. Wrapping nets go from
**8 of 19 to 2 of 19** (mean mismatch 79.0 -> 42.7 degrees). The eight were
`ENC_A`, `ENC_B`, `ENC_SW`, `JOY_X`, `JOY_Y`, `UART_RX`, `UART_TX`, `UG_D0`.

The surviving two are `JOY_X`/`JOY_Y` and they are not fixable: ADC_IN8/IN9 are
bonded to PB0/PB1 (and the rest of ADC_IN0..9 to PA0..PA7) on this package, so
the analog pair cannot leave that face whatever the assignment.

Constraints the optimiser was given, none of them discovered by it:
  * PC13/PC14/PC15 sit behind the VBAT power switch — a few mA of drive, low
    speed cap — so they may only carry pins that are READ. They hold COL2/COL3
    (inputs with pull-down under COL2ROW); the encoder they used to hold moved
    to PB12/PB13/PB15. Without this the solver cheerfully put TOUCH, which has
    to drive a charge cycle, on PC13.
  * `JOY_X`/`JOY_Y` must be ADC inputs; `UART_TX`/`UART_RX` must be a real USART
    pair on this package.

USART1 over AF0 (PB6/PB7) is now reachable and was chosen. The old map had
PA9/PA10 *and* PB6/PB7 all consumed by the matrix, which is why the console was
pushed onto USART2; moving ROW1 to PA10 and COL1 to PB5 frees the AF0 pair, and
it lands 4.9mm and 7.0mm closer to the debug socket.

### A wrong answer that a second measurement caught

The first solve inferred each net's target from a designator: array element
`d[i]` was assumed to carry designator `D(i+1)`. RFC-005 tombstones make that
false — `COL0` actually lands on D6/D10, not D3/D7 — so the solver optimised
against partly-fictional geometry and produced a map that left `COL0` wrapping
at 106.8 degrees while reporting 2 wraps. It was caught by a separate checker
that reads U3's pad->net assignment straight out of the generated `.kicad_pcb`
and is told nothing about what the answer should be. The solver now derives
every target from the net itself; no designator is inferred anywhere.

A second mechanical check compares all 17 firmware pin bindings against the
`.cohdl` device declaration, since a silent divergence there is a functional bug
no board scan would find. It passes.

### Firmware

Follows the new map: rows PA9/PA10/PB3/PB8, columns PB4/PB5/PC14/PC13, encoder
PB12/PB13/PB15, joystick push PA15, `JOY_Y` on PA0/ADC_IN0, key chain PA8, ring
chain PB14. The ring renderer returns as `[Grb; 8]` with a 256/8 hue step. A
comment records that COL2/COL3 are on limited-drive pins and must never be
turned into outputs. Builds clean for `thumbv6m-none-eabi`, no warnings.

### Still outstanding

The board remains **2 copper layers** (`tools/kicad_board.py` never calls
`SetCopperLayerCount`). With the ring's eight through-board apertures back, this
is now the only large lever left untried, and it is a product decision.

### Verification

Exact-geometry scan: 0 courtyard violations at 0.15mm minimum clearance, 0
plan-view overlap with the touch electrode, 0 shorts, 0 apertures cutting a pad,
0 silkscreen over a pad, 0 pads outside the outline. GPIO result re-measured off
the built board, independent of the solver: 3 of 26 MCU signal pads face away,
and all three are unfixable (`JOY_X`, `JOY_Y`, and `SWDIO` on its fixed PA13).
`cohdl check` and `fmt --check` clean, output byte-stable across a repeat build,
IPC-2581 validates, 21 test binaries pass, `cargo fmt --check` clean on both the
compiler and the firmware, `rpi-pico2` untouched.

## Joystick footprint corrections — the drawing was right, the STEP read was not (2026-07-29)

A physical check found the joystick's ø1.5 frame-leg holes in the wrong place.
Re-derived `FP_Joystick_RKJXV` from the Alps mounting-hole drawing (Drawing
No.1, ±0.1), this time decoding the bitmap deliberately: the drawing was
upscaled, gridded, calibrated on the 2.5mm terminal pitch, and every dimension
chain attached to a measured feature — not guessed. Three defects, one
principle:

**1. Frame legs moved (±6.0, ±6.0) -> (±6.325, ±5.0).** The "12.65" chain
spans the two leg columns and "10" the two leg rows; halved, that is
±6.325/±5.0 exactly, and pixel measurement agrees (x = ±6.4 ± 0.1,
y = ±5.0 ± 0.15). This is the value the 2026-07-26 audit *rejected*: the STEP
transcription that overruled it was itself the misread. Everything sourced
from the STEP alone was therefore re-checked against the drawing.

**2. The four ø2.6 "holes" are not holes.** The drawing labels every drilled
feature "hole" with its own tolerance (6-ø1, 4-ø1.5, 2-ø1.6 ±0.05, 4-ø1.2);
the hatched ø4 / ø3.5 / 4-ø2.6 carry no such label and hatch = the legend's
"Prohibited wiring area". They are surface keep-outs — the four ø2.6 circles
are where the part's 0.75mm-tall bosses REST on the board — so drilling them
removed the very surface the part seats on. The four NPTH `mount_hole`s are
deleted; only the 2-ø1.6 ±0.05 locating holes remain drilled (a peg enters
each). CoHDL has no copper-keep-out construct, so the prohibited areas
survive only as a footprint comment for the router.

**3. Legs are soldered, and the switch group re-anchored.** The datasheet's
soldering caution — "Solder all metal inserted fixing including terminals &
metal lugs into a substrate" — names the frame lugs, so the legs became
plated 2.2mm pads (`P_Joy_Leg`, 0.35mm ring like the family's other pads) on
a new electrically dead `optional MNT: L1-L4` pin, tied to GND in the design
exactly like the EC11's mounting posts. And the switch rows moved
5.8/10.3 -> 5.75/10.25: the drawing anchors the dome's ø3.5 keep-out at
y = +8 (the left-side "8") with the "4.5" row span symmetric about it; the
0.05 offset was STEP-only data, within tolerance but not the drawing's word.

Consequences: `openmicro.net` changes (joy MNT joins GND), the emitted
footprint/IPC geometry changes, and the hand-layouted `openmicro.kicad_pcb`
(plus the gerbers and position files cut from it) carries the stale joystick
until J1 is re-imported and its region re-routed. BOM, designators and
`layout.json` are unchanged.

## openmicro joystick: push-switch pole pairing + 1mm east shift (2026-08-06)

Two fixes reported off the built board, both directed by the user.

**1. The centre-push never produced an event — the pole pairing was wrong.**
`ThumbPointer` grouped the switch's four terminals by SIDE (`SW_A: SA, SB` /
`SW_B: SC, SD`), the 2026-07-26 audit's "paired by side" verdict (now marked
superseded above). The land is the standard 6.5 × 4.5 tact-switch pattern, and
a tact switch's two stamped contact frames each exit as the two same-ROW legs
6.5mm apart: a+c are one pole, b+d the other, the 4.5mm row spacing separating
the poles. Grouping by side therefore put one leg of EACH pole on each net —
JOY_SW (PA15) and GND were joined through the switch's own contact frames, the
input read low permanently, and no press edge could ever fire. Neither the
site datasheet nor the series catalog states the internal connection (that
lives in the formal supply spec); the basis is the 6.5 × 4.5 geometry plus the
identical row-not-column correction the OpenMicro footprint audit made on the
since-removed EVQ-P7A reset switch. Now `SW_A: SA, SC` / `SW_B: SB, SD`.
Netlist: pads SB and SC swap nets (SA+SC on JOY_SW, SB+SD on GND); BOM,
designators and `layout.json` are unchanged by this half of the fix.

**2. `joy` moved 1mm east: (28.575, -29.575) → (29.575, -29.575).** The same
problem as the earlier 1mm south shift, in the other axis: the body's west
face sat 18.225mm from board centre at its widest, essentially touching
sw[1]'s 17.5mm keycap edge at 18.275. East holds nothing but board edge, so
the millimetre is free. Verified on a regenerated placement board: J1
body-to-body gap to the west key 0.900 → 1.900mm, 0 layer-aware pad shorts
across the 61 placed parts, every footprint + pad resolved.

Consequences: the routed `out/openmicro.kicad_pcb` is stale on both counts —
J1 moved and two of its pads swapped nets — so its joystick region needs
re-placing and re-routing (a Quilter re-run). The routed file was left
untouched; `.net`, `layout.json`, footprints and the IPC-2581 XML were
regenerated.

## Bounded annulus and segmented stencil geometry (2026-08-07)

`shape: annulus` now preserves an SMD ring as one electrical pad in the AST,
KiCad custom-pad output, and IPC-2581 (`Contour` plus circular `Cutout`).
`segmented_annulus(outer, inner, gap)` emits exactly four conservative paste
sector polygons without duplicating the logical pin. The resolver rejects
non-pad contexts, non-SMD/non-copper use, invalid or collapsing diameters,
paste outside copper, and geometry beyond the 100 mm / 512-segment /
520-vertex hard bounds. This closes the TDK/InvenSense T3902 manufacturer-land
gap without adding a general polygon API.

## openmicro-kbd → openmicro2: SF32 wireless redesign (2026-08-07)

Directed by the user: the `openmicro-kbd` example is renamed `openmicro2`
and rebuilt as an ORIGINAL design (no longer a Codex Micro clone) around the
SF32LB52EUB6 — BLE, the H0216F002AM 2.16" AMOLED touch module, a control row
of encoder / display / two encoders / joystick, a 3x5 key field replacing
the touch pad and first-row keys, and one microphone. 122 instances,
106 nets, 73 placements, 130 x 108mm outline; `cargo test` green including
the rewritten exit-criteria test.

**OK-F302-31115 identified; local part carries the display.** The
`lib/@contrib/display` audit (2026-08-04) left H0216F002AM logical-only
because the module spec names two connector codes without drawings. The
official OCN series drawing (retrieved 2026-08-07, hashed in
`examples/openmicro2/docs/README.md`) resolves `OK-F302-**115` as OCN's
0.3mm-pitch 1.0mm front-flip bottom-contact FPC family with an explicit
recommended pattern, and the n=31 table row matches the module's 31-finger
tail (the OK-14 series is 0.4mm board-to-board and cannot). The example
binds a LOCAL part `DISPLAY_FPC_OK_F302` on that drawing; the library part
stays blocked pending OCN confirmation of two residual interpretations,
both recorded in the footprint comment: which staggered row faces the FPC
opening, and pin 1's end (the 16-odd/15-even row split itself is forced by
arithmetic: span C=9.00 holds 16 pads, B=8.40 holds 15).

**Datasheet-driven electrical decisions.**
- Display VBAT (SIBO) is specified 3.7-4.5V with ABS MAX 4.6V (module
  p6/p7, tables that also contradict each other with a 4.5-6.5V "VDD" DC
  row — the interface table + AMR intersection is what's honored), so the
  display runs from the SGM41562B power-path SYS rail with a 1-cell Li-ion,
  never from 5V VBUS. This is what motivated battery power.
- The microphone is ANALOG (module connector into ADCP, supply from
  MIC_BIAS through the connector doc's own 100R + 100nF + 1uF-block
  wiring): the pinmux places PDM1 only on PA07/PA08 (display QSPI DIO2/3)
  or PA22/PA23 (the required 32.768kHz crystal), and I2S1 only across the
  same display group or the GPADC block — every digital-mic position is
  spoken for, the dedicated analog path is free.
- TP_VCC is the module's own touch-supply OUTPUT (p6) — left open; nothing
  on the board may drive it. MTP_PWR open per p6. VCI_EN joins VCI on 3V3
  (both carry the same "DDIC DCDC supply" description).
- Encoder/joystick pushes scan as matrix row 3 through their own 1N4148W
  diodes — the SF32's 45 GPIOs are exactly consumed; the freed pins fund
  the charger nINT line, the SGM2554 LED-rail gate, and the PA01
  calibration line on the debug socket (miniboard convention).

**Placement verification.** kicad_board.py rebuilt the board from the
IPC-2581 document (122/122 placed, every footprint + pad resolved); the
layer-aware pad-overlap scan (per the established method: copper-layer
intersection first, outline filter, no BOX2I.Common) found ONE real
collision — r_ant on the top-right M2 hole's 4mm GND annulus — fixed by
moving `mh[1]` to (48, -50), rescan clean: 0 overlaps across 73 placed
parts, 49 staged.

## Distribution tooling: release workflow, install.sh, self-update (2026-08-11)

No RFC governs binary distribution — this is repository tooling, not
language surface, added at the maintainer's direction (the same footing as
the arbitrary-rotation deviation). The constitution is honored: zero new
crate dependencies. `cohdl self-update` (src/selfupdate.rs) uses the
system `curl` per the RFC-030 precedent (a new `registry::http_get_follow`
adds redirect-following, which GitHub's CDN requires and plain `http_get`
deliberately lacks); the `.tar.gz` is unpacked by the system `tar` — an
extension of the system-tool route, not an RFC-030 precedent: RFC-030's
own archive is hand-rolled precisely because it is uncompressed, and
DEFLATE is not worth hand-rolling. Downloads verify against
`sha256sums.txt` with src/hash.rs's own SHA-256, and release versions are
RFC-029 exact triples — which structurally excludes the VS Code
extension's `vscode-v*` tags and any pre-release shape from selection (the
release workflow's gate also refuses to publish a non-triple tag, so the
contract cannot be split at the source).

Decisions of note:
- **One artifact contract, three consumers.** `cohdl-vX.Y.Z-<target>.tar.gz`
  (single binary inside) + `sha256sums.txt`, produced by
  `.github/workflows/release-cohdl.yml`, consumed identically by install.sh
  and self-update. Each file says so; renaming anything is a three-place
  change.
- **`[profile.dist]`** (stripped, no debug) exists because
  `[profile.release]` deliberately keeps `debug = true` for local
  profiling; shipping that would bloat every artifact several-fold.
- **Linux is musl-static only.** A self-update on a gnu-linked local build
  still installs the musl artifact — one binary per arch runs everywhere.
- **Self-replacement never touches /tmp.** The archive is staged and
  unpacked in a fresh `create_dir` (fail-if-exists) workdir inside the
  executable's own directory — which must be user-writable for the final
  rename anyway — closing the verify-then-install window a predictable
  path in a world-writable temp dir would open. `current_exe` is
  canonicalized first so the swap replaces the real binary, never a
  symlink node (macOS may report the invoked symlink). Stale workdirs from
  crashed runs are swept on the next update.
- **Release discovery paginates** (`per_page=100`, pages until empty, cap
  20) in both self-update and install.sh: a single-page read could be
  starved once >100 extension releases postdate the newest compiler tag.
- **`cohdl --version`** (new) prints `cohdl X.Y.Z (<release target>)`; the
  triple names which published artifact the binary corresponds to, and
  install.sh runs it to prove the installed binary works before reporting
  success.
- Tested end-to-end without network: tests/self_update.rs runs a copy of
  the real binary against a std-only local HTTP server (fetch across
  paginated release lists → verify → atomic self-replace with no workdir
  residue, plus --check, corrupted-download refusal, and up-to-date paths).

## Transitive dependency resolution (RFC-029 amendment, user-directed 2026-08-25)

RFC-029 as accepted resolved only the manifest's direct `[dependencies]`;
a package whose own manifest declared dependencies (RFC-030 made these
publishable facts) compiled fine as a project but failed as a dependency —
`cohdl add @espressif/esp32` produced a consumer that could not resolve
`qfn::…` footprints (E202) because the dependency's declared `qfn = "0.1.1"`
was never loaded. Seven published packages carry `qfn`/`soic` pins, so every
consumer of those hit this wall. User-directed amendment: resolution now
covers the transitive closure.

Semantics (all in `deps::resolve`, one walk shared by check/build, the LSP,
and the RFC-030 verbs):

1. **The closure walk.** Every resolved package's own `[dependencies]`
   joins the work set (BFS in declaration order — deterministic, so every
   diagnostic it can emit is too). The lock records the closure; rows no
   longer reachable are pruned. E1101/E1102/E1103/E1106 apply uniformly at
   every depth; a transitive E1102 names its requirer and anchors to the
   declaring manifest's own line.
2. **The project pin is the single authority.** A root `[dependencies]`
   pin wins silently over any dependency's pin for the same name. Two
   *dependencies* pinning different exact versions with no root pin is the
   new E1108 hard error — exact pins cannot be merged, and the help says to
   pin the name at the root to choose explicitly. No newest-wins, ever: a
   resolution policy choosing versions would put a verdict on the other
   side of a heuristic. (Root-wins over hard-error-always because exact
   pins would otherwise deadlock the ecosystem: a dep pinning `qfn 0.1.1`
   would conflict with every root that `cohdl update`d past it, and no
   root-side action could ever resolve it.)
3. **Offline stays offline.** check/build/LSP walk with `fetch: None` and
   keep the E1102 "run `cohdl install`" contract; `install`/`update` pass a
   fetch hook wired to the RFC-030 download, so missing closure members are
   fetched exactly where direct ones were. `add` fetches the added
   package's closure content into the cache (lock rows for the closure are
   first-resolution work for the next resolve, which sees the whole
   manifest — writing them from add's single-entry walk would prune every
   other row).
4. **std stays non-special.** A dependency's `std` pin is walked like any
   other name; the only exception is caller context, not the name: under
   `--no-std` or a `--std` override, std is already settled outside the
   registry, so the CLI passes it in `ResolveOpts::skip_transitive` and a
   dependency's std pin cannot re-introduce it.

Visibility is unchanged: every loaded package's `pub` symbols remain
referenceable by every other, so a root project can name a transitive
package it never declared. Flagged, not decided: requiring a package to
declare what it references is a separate strictness question for a future
RFC (it needs an answer for std, which no source file names explicitly).

Verification: seven fixture tests in tests/deps.rs (closure lock + byte
stability, requirer-named E1102, E1108 + root-pin resolution of it,
root-pin-wins selection, transitive E1103 tamper, prune-on-remove,
`--no-std` std skip) and two mock-registry tests in tests/registry.rs
(install fetches the closure; add caches the added package's closure). The
original blocker reproduces fixed live: a project depending on
`@espressif/esp32` alone now checks clean, `qfn 0.1.1` resolved and locked
transitively.

## Native .kicad_pcb emission (emitter tooling, user-directed 2026-08-25)

`cohdl build --emit kicad_pcb` writes a KiCad 10 board file directly —
`src/emit/kicad_pcb.rs`, zero dependencies, byte-stable — retiring the
pcbnew-scripted `tools/kicad_board.py` assembly (and with it the
kicad-board-needs-ipc-emit staleness trap: the emitter reads the same
checked IR as every other artifact, no sidecar files). Contract in
docs/kicad_pcb.md. Decisions of note:

- **Format target = what pcbnew 10 itself writes.** `(version 20260206)`,
  pad nets BY NAME with no board-level net table, TAB indentation, the
  stock 2-copper-layer `(layers)`/`(setup)` blocks verbatim (the 2-layer
  default is the pcbnew flow's status quo, consciously inherited). The
  grammar was pinned from three pcbnew-written boards, not from
  documentation.
- **The frame is the authoring frame.** CoHDL authors +y-down = KiCad's
  board frame; placements and footprint-local geometry pass through
  verbatim. The IPC emitter's y-negation is that document's +y-up
  requirement and must never leak here.
- **RFC-026 back side, empirically pinned.** KiCad stores the LEFT_RIGHT
  flip as y-mirror + 180° folded into the angle: fp angle = R+180 in
  (−180, 180], every local y negated, F.*→B.*, `(justify mirror)`,
  pad-local RFC-025 rotation REVERSED ((R−r) — reflection), asymmetric
  chamfer corners swapped VERTICALLY (the horizontal half of the flip is
  the folded 180). Derived from a pcbnew-written back-side footprint
  diffed against its source `.kicad_mod`, then verified semantically
  (footprint anchor/angle/side + absolute pad deltas + nets) against
  fresh pcbnew-assembled boards: both repo examples and both external
  OpenMicroKBD revisions (v1: 88 footprints / 33 bottom; v2: 140 / 53) —
  identical within pcbnew's float-nanometer reconstruction noise (the
  native coordinates are exact). All four native boards load in real
  pcbnew 10.0.4 with correct flip/net/outline inventories. The repeatable
  semantic oracle is `tools/validate_kicad_pcb.py`; it compares every
  footprint, pad copy/net, field, graphic, and Edge.Cuts item while ignoring
  random UUIDs, numeric net codes, and serialization order. A wrong-but-valid
  orientation remains invisible to check/build; opening the board is a human
  checkpoint, as ever.
- **One derivation, two dialects.** Pad plans and body graphics were
  extracted from the `.kicad_mod` emitter (`kicad_mod::pad_plans`,
  `body_graphics` — coordinates exact at 10^-16 mm so odd-femto corner
  halves survive) and both emitters render from them — the RFC-031
  anti-drift shape, now covering pads. The `.kicad_mod` output is
  byte-identical across the refactor (examples regression-verified).
- **Determinism.** uuids are sha256-derived from stable identity
  (RFC-4122-shaped, never random — the one part of pcbnew's output a
  reproducible emitter must not copy). Outline arc midpoints honor DXF
  winding (correct beyond 180°, unlike the retired script's bisector)
  and round ONCE to nanometers — KiCad's own resolution — keeping libm
  last-bit noise five orders below the rounding step; this is the silk
  fp_arc f64-tessellation precedent, not a placement-rotation exception.
- **Cross-validation correction (2026-08-27).** The first live four-board
  semantic diff caught the initial midpoint implementation interpreting the
  DXF winding flag in KiCad's displayed +y-down handedness. Numeric DXF
  coordinates pass through unchanged, so midpoint selection must stay in the
  DXF +y-up coordinate system and let KiCad flip only the visual handedness.
  Before correction, ordinary rounded corners loaded as 270-degree major
  arcs instead of 90-degree corners in all three outlined boards. Both winding
  directions and explicit major-arc selection now have direct unit tests.
- **Staging.** Unplaced instances stage on the same shelf the IPC
  document uses (one convention, two emitters; `staging_positions`
  shared); with no outline they take the retired script's 12 mm grid
  from (40, 40) — never stacked at (0, 0).
- **`--emit` is now repeatable with DISTINCT values** so the Quilter
  handoff and the board come from one build (previously writing one
  swept the other as stale). The same value twice keeps the F12.5
  duplicate error; command-compat-before-value ordering unchanged.
- **Routed-board protection is the ownership contract, not a special
  case.** A pcbnew-generated or hand-routed board at `out/<name>.kicad_pcb`
  is foreign to `.cohdl-manifest`, so the emitter refuses to overwrite
  it. Once CoHDL owns the path, later builds rewrite/sweep it — so route
  on a copy outside `out/` (the established `pcb/` convention),
  documented rather than heuristically guessed.
