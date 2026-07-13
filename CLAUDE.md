# CLAUDE.md

CoHDL v2 — a compiler for the AI-native PCB schematic language. Rust, single crate, **zero external dependencies** (a deliberate choice: byte-stable, reproducible output is a hard constraint, so everything — TOML lock file, S-expression netlist, diagnostics — is hand-rolled and deterministic).

The previous implementation is on the `legacy` branch. It is reference material only — v2 is a ground-up rebuild; do not port legacy code.

## Source of truth

The language design lives in `docs/design/` (snapshot of the conol.ai design repository — see `docs/design/README.md`). Binding order:

1. `docs/design/10-language-specification.md` — what the language IS (Accepted RFCs only)
2. `docs/design/rfc-001…007-*.md` — the seven Accepted P0 RFCs (normative detail)
3. `docs/design/09-mvp-definition.md` — current scope line + exit criteria
4. `docs/provisional-syntax.md` — MVP-pragmatic choices for constructs with **no Accepted RFC yet** (part/MPN, pin roles, net annotations, modules). These are provisional; an RFC on conol.ai supersedes them.

Never implement beyond the MVP cut list in `09-mvp-definition.md` (no fmt, no LSP, no --json API, no LCEDA, no pattern matching, no incremental compilation).

## Commands

- `cargo build` / `cargo test` — build and run all tests
- `cargo run -- check <file-or-dir>` — parse + resolve + type-check + residual DRC
- `cargo run -- build <file-or-dir>` — check + emit KiCad `.net` + BOM CSV (+ `design.lock`)
- `harness/` — the generate→check→repair demo harness (script, not product)

## Architecture (pipeline = the verdict ladder)

`parses ⊂ resolves ⊂ type-checks ⊂ connects ⊂ passes residual DRC ⊂ emits netlist`

- `src/span.rs`, `src/diag.rs` — SourceMap, spans, diagnostics (every diagnostic: stable code + precise span; codes registry in `docs/error-codes.md`)
- `src/lex.rs`, `src/ast.rs`, `src/parse.rs` — hand-written lexer + recursive-descent parser (deterministic, no unbounded lookahead — Constitution hard constraint)
- `src/resolve.rs` — cross-file symbol table (project files + `std/`)
- `src/check/` — the heart: units (RFC-001), impl satisfaction (RFC-003), generics (RFC-007), fn expansion with cycle detection (RFC-006), pin exhaustiveness at final assembly (RFC-002)
- `src/ir.rs` — flat post-expansion design IR (instances, nets, nc)
- `src/lock.rs` — design.lock + the pure-function designator allocator with checked injectivity postcondition (RFC-005)
- `src/drc.rs` — exactly four rules (RFC-004): voltage-exceed, polarity-mismatch, single-driver, multi-driver. **Never add a fifth structural rule** — that's a type-system job.
- `src/emit/` — KiCad `.net` (S-expr) + BOM CSV
- `std/` — MVP std library (only what the demo board needs)
- `tests/` — fixture tests mapped 1:1 to the MVP exit criteria

## Non-negotiable invariants (from the Constitution / RFCs)

- Same source + same std → same verdict, same designators, same netlist **bytes**. No HashMap iteration order leaking into output — use BTreeMap or explicit sorts everywhere output-adjacent.
- Zero unit coercion; a bare number where a unit is expected is a compile error naming expected vs. actual.
- Every diagnostic names the exact construct (instance, pin, trait, unit) — never a bare "type mismatch".
- The designator allocator asserts injectivity as a postcondition on every run.
- Unicode `Ω` / `°C` must produce a targeted "use `ohm` / use `C`" diagnostic, not a generic lex error.
