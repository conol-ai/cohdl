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
  courtyard `Outline` and one `Pin` element per placement (the schema's
  `Outline` requires a Polygon, so a CIRCLE courtyard projects there as
  its bounding square — disclosed; `.kicad_mod` keeps the true circle).
  Roundrect is not in the closed shape vocabulary, so KiCad-derived
  roundrect pads project as `rect` — a disclosed approximation, not a
  claim of identity. Both emitters share `emit::geom`: all arithmetic on
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
`Pin` carries `type="THRU"` + `mountType="THROUGH_HOLE_HOLE"` (the drill still
has no IPC `Pin` home — review R5-8, unchanged). E807 still holds (pad numbers
unchanged), and the document stays schema-valid.

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
  name-vs-geometry mismatch — pin count (pad count minus the `_1EP` exposed pad)
  and pitch (the closest pad-center spacing, exact over the femto integers — the
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

- **`DictionaryStandard`** (Content) — one `EntryStandard` primitive
  (`RectCenter`/`Circle`/`Oval`) per unique pad shape.
- **Real layers + stackup** (CadData) — F.Cu/B.Cu/F.Mask/F.Paste/B.Mask +
  Edge.Cuts, and a 2-layer stackup, replacing the single synthetic `TOP` layer.
- **`PadStackDef`** (Step) per unique (shape, plating, drill): `PadstackPadDef`
  on F.Cu/F.Mask/F.Paste (+ B.Cu/B.Mask for through-hole) and a plated
  `PadstackHoleDef` carrying the real drill diameter.
- **`LayerFeature`** (Step) on F.Cu (all pads) + B.Cu (through-hole): each
  placed `Pad` at its absolute board position (component location + the pad
  offset rotated by the component's cardinal rotation — exact integer, no
  trig), referencing its padstack, and tied to its component pin (`PinRef`) and
  net (`Set/@net`).
- **Accurate mount types** — `Component/@mountType` SMT vs THMT (THMT if the
  component has any through-hole pad), on the physical F.Cu layer (was the
  synthetic `TOP`/`OTHER`); `Pin/@mountType` `THROUGH_HOLE_PIN` (an electrical
  pin, not a non-electrical `_HOLE`).
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
  mask (and SMD paste) layers too, matching the reference exporter.
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
