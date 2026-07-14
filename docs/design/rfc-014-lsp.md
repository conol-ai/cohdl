# RFC-014: Language Server Protocol support

## Problem

With the MVP complete (RFC-001–013 all Accepted and implemented, 131 passing tests, `cohdl check`/`build --json` and `cohdl fmt` real) CoHDL has every piece an editor integration needs *except the editor integration itself*. Today, a human reviewer or an AI-assisted editing session gets feedback only by running `cohdl check` from a terminal and reading the output — there is no live, in-editor diagnostic squiggle, no hover-to-see-a-resolved-`impl`'s-name-matches (a capability RFC-003's own Decision Record flagged as needed: "LSP tooling must... show the resolved by-name matches on hover for any empty impl block, since that information exists only in the compiler's resolution once the body is empty"), and no "find all impls for this device" navigation (also named explicitly in RFC-003's DR-013 consequences, never built).

Who this is for: **human reviewers**, whose whole job per the Constitution's priority ladder (rank 3: "human reviewability & trust") is auditing generated `.cohdl` source — an LSP is how that happens in the tool they actually use (VS Code, any LSP-client editor), not by context-switching to a terminal. Secondarily, **AI coding agents operating inside an editor** (as opposed to a standalone repair-loop script) that want live diagnostics as they write, the same way a coding agent benefits from `rust-analyzer` today.

## Goals

- Implement `textDocument/publishDiagnostics`, re-using the exact `Checked`/`JsonDiag` pipeline output RFC-010 already produces — **zero new diagnostic logic**, purely a new transport/frontend for diagnostics that already exist.
- Implement `textDocument/hover` for the two cases RFC-003's own Decision Record already named as needed: showing a resolved by-name `impl` mapping (a device's fields matched against a trait's required roles) on hover over an empty `impl Trait for Device {}` block, and showing a pin's resolved obligation kind/role on hover.
- Implement `textDocument/definition` ("goto-def") for named references: a device/trait/fn name at a use site resolves to its declaration.
- Implement the "find all impls" navigation RFC-003's DR-013 flagged: `textDocument/references` on a trait name lists every `impl TraitName for ...` statement in scope; on a device name, lists every `impl ... for DeviceName` statement.

## Non-goals

- **Not full IDE features.** No code actions/quick-fixes, no rename-refactoring, no semantic-highlighting beyond what a client already gets from a TextMate/tree-sitter grammar, no workspace-wide symbol search. This RFC covers exactly the four capabilities named above — the ones already explicitly flagged as needed by an earlier Accepted RFC, not a speculative "what would be nice."
- **Not incremental compilation.** Every request re-runs the full `pipeline::check()` from scratch, the same as the CLI does today. Note 3's capability map already lists incremental compilation as its own, separately-gated future item — this RFC does not pull it in as a prerequisite. (See Failure modes for the latency consequence this accepts.)
- **Not a new diagnostic/type-checking mechanism.** This RFC is purely a protocol server wrapping the existing `pipeline`/`json` modules — if a diagnostic is wrong or missing, that's a bug in RFC-001–013's implementation, not something this RFC's own scope covers fixing.

## Design

### Architecture: a thin JSON-RPC/stdio server wrapping the existing pipeline

```javascript
editor (VS Code, etc.)
    │  JSON-RPC over stdio
    ▼
cohdl lsp   (new subcommand, new binary target `cohdl-lsp` or a mode of `cohdl`)
    │  in-process function calls — no subprocess spawning per request
    ▼
pipeline::check()  (existing, unchanged)
    │
    ▼
diag::Diagnostics, ast::* (existing, unchanged)
```

- New CLI surface: `cohdl lsp` — starts a JSON-RPC server on stdio, the standard LSP transport every editor client already expects.
- On `textDocument/didOpen`/`didChange`/`didSave`, the server re-parses and re-`pipeline::check()`s the affected file's containing project (same invocation `cohdl check` already does), then converts the resulting `Checked`'s diagnostics into LSP `Diagnostic` objects (`severity`, `range`, `message`, `code` — a direct field-by-field mapping from the existing `JsonDiag` shape RFC-010 already defined) and sends `textDocument/publishDiagnostics`.
- `textDocument/hover` on an `impl Trait for Device {}` span looks up the same by-name-matching resolution the type checker already computed during `pipeline::check()` (RFC-003's mechanism) and renders it as hover markdown — e.g. hovering `impl Capacitor for MLCC {}` shows `capacitance ← capacitance, voltage_rating ← voltage_rating, tolerance ← tolerance` (the resolved matches), even though the source body is empty. Hovering a pin shows its obligation kind (`required`/`optional`) and role (per RFC-008).
- `textDocument/definition` on a device/trait/fn/part name at any use site (an `inst` type, an `impl ... for X`, a generic bound, a call) resolves to the `Span` of that name's own declaration — already tracked by the existing `resolve.rs` pass, exposed here rather than computed anew.
- `textDocument/references` invoked on a trait or device name in an `impl` statement (the natural place to ask "what else implements/is implemented by this") returns every matching `impl` statement's span in the currently-open project — a linear scan over the already-parsed AST, no new index structure required at MVP-of-this-RFC scale.

### Dependency question: does this break the zero-external-dependencies constraint?

**Yes, and this RFC proposes accepting that, explicitly.** `Cargo.toml` today has empty `[dependencies]` (a real, deliberate project property per `src/emit/json.rs`'s own comment: "hand-rolled, zero external dependencies, per the project constitution"). A real LSP server needs, at minimum, JSON-RPC message framing over stdio (`Content-Length` header parsing) — implementable hand-rolled in well under 100 lines given the project already hand-rolls JSON serialization (`src/emit/json.rs`), but the *protocol's own message shapes* (`Diagnostic`, `Position`, `Range`, the full `initialize` handshake capabilities negotiation) are a large, standardized surface where hand-rolling has a real, ongoing maintenance cost with no corresponding coherence benefit — unlike JSON serialization (where the project's own output format is small, fixed, and worth controlling precisely), the LSP spec is externally versioned and evolves independently of CoHDL. This RFC recommends depending on the `lsp-types` crate (pure data-type definitions matching the LSP spec, no I/O, no async runtime) while continuing to hand-roll the JSON-RPC transport loop itself — a scoped, justified exception, not a wholesale dependency-policy reversal.

## Type-system-first test

N/A — this RFC adds no new checkable construct; it is a protocol-server frontend over the existing, already-Accepted diagnostic/resolution pipeline.

## Conceptual impact

None. No new language concept, no new syntax. This is Layer-4 tooling (per note 3's capability map) — the same category as `cohdl fmt` (RFC-009) and `cohdl check --json` (RFC-010), both already-Accepted precedents for "pure tooling wrapping the existing pipeline, zero conceptual cost."

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Med | Low | Low | High |

**Diagnostics (Med):** the LSP's diagnostics must be provably identical to what `cohdl check --json` already reports for the same file — this RFC inherits RFC-010's equivalence-testing discipline (see Gradeability) rather than risking a third, silently-diverging diagnostic surface.
**Trust (High):** an LSP is the single highest-leverage thing for the "human reviewability & trust" ladder rank (Constitution rank 3) — it's how a human reviewer actually audits generated source in practice, not a nice-to-have.
**Oracle/Grammar/Concepts/Netlist/Compat (Low):** zero impact — no new check, no new syntax, no netlist change, purely additive tooling.

## Gradeability

Enforced by a direct **equivalence test**, mirroring RFC-010's own: for every fixture, the LSP's `publishDiagnostics` payload for that file (severity, range, message, code) must exactly match what `cohdl check --json` reports for the same file, field-for-field. Hover/goto-def/references are tested by fixture-driven assertions: a fixture with a known empty `impl` block must hover to the exact expected resolved-mapping text; a fixture with a known cross-reference must goto-def to the exact expected span; a fixture with N known `impl`s of a trait must return exactly N reference locations.

## AI-generatability

N/A in the usual sense (this doesn't change what an AI author writes) — but genuinely relevant to an **AI coding agent operating inside an editor**: live diagnostics let such an agent see a mistake the moment it's typed, the same generate→check→repair loop the whole project exists to enable, just running interactively instead of as a batch script.

## Alternatives

- **Ship diagnostics-only (no hover/goto-def/references), defer the rest** — considered, rejected: hover-on-empty-`impl` and "find all impls" were both explicitly named as needed by RFC-003's own Decision Record when it was written — deferring them again would mean a second RFC still not closing a gap the project already identified as real, not speculative.
- **A full semantic-highlighting/tree-sitter grammar as part of this RFC** — rejected: editors already get adequate highlighting from a TextMate grammar (v1 had one; a v2 equivalent is separate, low-conceptual-cost work not gated on this RFC) — bundling it here would blur this RFC's scope beyond the four capabilities it's actually justified by.
- **Hand-roll the LSP protocol's data types too, avoiding any new dependency** — rejected: unlike the project's own output formats (JSON diagnostics, KiCad netlist — small, fixed, worth controlling precisely), the LSP spec's type surface is large and externally versioned; hand-rolling it duplicates a spec CoHDL doesn't own or benefit from re-deriving, with real ongoing maintenance cost as the LSP spec itself evolves. The transport loop (JSON-RPC framing) stays hand-rolled, consistent with the project's existing style; only the message *shapes* are borrowed.
- **Build incremental compilation first, so the LSP is fast, then build the LSP** — rejected as unnecessary sequencing: `pipeline::check()` on an MVP-scale demo board is fast enough (the whole existing test suite of 131 tests runs in ~0.04s) that a full re-check per keystroke-adjacent event is acceptable at current project scale; incremental compilation is real future work but not a blocking prerequisite (see Failure modes for the honest latency caveat at larger scale).

## Compatibility

Purely additive — a new subcommand (`cohdl lsp`), one new (scoped, justified) dependency (`lsp-types`), zero changes to any existing command, diagnostic, grammar, or netlist output.

## Tooling & operations

- `cohdl lsp` becomes the fourth CLI subcommand alongside `check`/`build`/`fmt` — `main.rs`'s `USAGE` string gains an entry.
- The VS Code extension (or any LSP-client editor config) needs a client-side launch config pointing at `cohdl lsp` — this RFC's scope includes the server; a minimal client launch snippet (not a full extension marketplace listing) should ship alongside it as a usage example.
- Reserves no new error codes — the LSP surfaces existing diagnostics; it does not introduce new ones.

## Teaching cost

Low for authors (nothing changes about writing `.cohdl` source). Low for editor setup (standard LSP client configuration, the same pattern every LSP-backed language extension already uses).

## Failure modes

- **Full re-check per edit event becomes too slow on a large project** — acceptable at current MVP/demo scale (confirmed: the whole test suite runs in ~0.04s); this is a real, named future limitation, not silently glossed over — the fix is incremental compilation, already tracked separately in note 3's capability map, not a hidden requirement of this RFC.
- `lsp-types`**' LSP-spec version drifts from what a client expects** — mitigated by pinning a specific `lsp-types` version and testing against at least one real client (VS Code) as part of this RFC's acceptance, not just unit-testing the server in isolation.
- **Hover/goto-def/references diverge from what **`cohdl check`** would report** (e.g. a bug where the LSP's in-process pipeline call takes a different code path than the CLI's) — caught by the mandatory equivalence test in Gradeability; both must call the exact same `pipeline::check()` function, not parallel reimplementations.

## Migration path

N/A — purely additive, no existing source or tooling behavior changes.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-020 (see note 7). Language Specification (note 10) gains a short "Language Server (`cohdl lsp`)" section (tooling reference, not a language-construct section, consistent with how RFC-009/010 are documented there). Note 3's capability map row for LSP updates from ⛔ to 📐 (design complete, implementation pending) — the same designed-vs-implemented distinction the map now tracks for every other RFC.
