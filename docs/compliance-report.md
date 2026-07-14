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
- **Constraint mapping**: IPC-2581B1 has no user-named net-class element
  (`LogicalNet/@netClass` is a closed enum, mapped from `[gnd]`/voltage
  annotations), so RFC-013 constraints ride `CadHeader/Spec` +
  `General/Property` entries under `cohdl:`-prefixed names — the schema's
  own named-specification mechanism. This is the "native constraint
  elements" reading documented in docs/ipc2581.md; if a partner integration
  later needs a different projection, that's a docs-level contract change,
  not a language change.
- **xmllint gate**: authoritative in CI (libxml2-utils installed in
  ci.yml); locally the validity test skips with a loud warning when xmllint
  is absent rather than failing unrelated work.
- **Not claimed**: end-to-end validation against a real consuming tool
  (Quilter). DR-021's own "revisit when" gates that on real access; tracked
  future work alongside footprint geometry (RFC-018 now Accepted for that)
  and board outline/stackup.

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
- The spec snapshot's "Footprints and pads (copad/cofp)" section is STALE
  against RFC-018's same-day naming correction (plain `pad`/`footprint`,
  "no rename" per its Alternatives/Decision) — a note-side amendment
  item; the implementation follows the RFCs' accepted text.
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
`pad`/`footprint`, no `copad`/`cofp` rename — the spec snapshot's section
remains stale, ledgered under RFC-017). `pad` is a sixth top-level
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
column alignment in fmt, duplicate-field last-wins (spec'd elsewhere),
negative length_match tolerance (magnitude semantics not normative).

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
