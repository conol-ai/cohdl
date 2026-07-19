# CLAUDE.md

CoHDL v2 — a compiler for the AI-native PCB schematic language. Rust, single crate. The compiler pipeline and emitters have **zero external dependencies** (a deliberate choice: byte-stable, reproducible output is a hard constraint, so everything — TOML lock file, S-expression netlist, diagnostics JSON — is hand-rolled and deterministic). One scoped exception per RFC-014/DR-020: the LSP layer (`src/lsp.rs`) depends on pinned `lsp-types` (+ its `serde`/`serde_json` requirements) for the protocol's message shapes only — the JSON-RPC transport loop stays hand-rolled, and nothing outside the LSP layer may use these crates.

The previous implementation is on the `legacy` branch. It is reference material only — v2 is a ground-up rebuild; do not port legacy code.

## Source of truth

The language design lives in `docs/design/` (snapshot of the conol.ai design repository — see `docs/design/README.md`). Binding order:

1. `docs/design/10-language-specification.md` — what the language IS (Accepted RFCs only)
2. `docs/design/rfc-001…007-*.md` — the seven Accepted P0 RFCs (normative detail)
3. `docs/design/09-mvp-definition.md` — current scope line + exit criteria
4. `docs/provisional-syntax.md` — MVP-pragmatic choices for constructs with **no Accepted RFC yet** (part/MPN, net annotations, modules). These are provisional; an RFC on conol.ai supersedes them. (Pin roles and package variants graduated to RFC-008.)

Never implement beyond the MVP cut list in `09-mvp-definition.md` (no LCEDA, no incremental compilation). Originally-cut items that have since graduated via Accepted RFCs and ARE implemented: structural-variant pattern matching (RFC-008), `cohdl fmt` canonical form (RFC-009), `cohdl check/build --json` (RFC-010), the formal error-code registry (RFC-011), `#[intent("...")]` opaque-metadata annotations (RFC-012), layout constraints — `layout {}` + `#[placement_hint]` → `layout.json`, E10xx checks (RFC-013), the LSP server `cohdl lsp` (RFC-014), the IPC-2581 handoff document `build --emit ipc2581` (RFC-015; contract in `docs/ipc2581.md`, schema gate needs `xmllint`), the module system (RFC-016: file-tree module paths, `use` imports, cross-package `pub` enforcement, E207-E209; references are REWRITTEN to fully-qualified names in `resolve.rs` — downstream maps are fq-keyed, `resolve::short()` for display), the library registry (RFC-017: `#[doc]` reference documents + `footprint` as a resolvable declaration kind — `footprint:` in parts is a symbol reference, never a string), and the pad/footprint format (RFC-018: `pad` declarations with a closed shape/layer/plating vocabulary, `pad N: Sym at (x, y)` placements + courtyard/silkscreen_ref in footprint bodies, the `Length`/`mm` unit as RFC-001's eleventh type, E805-E807 — pad-vs-device consistency checked at BUILD, empty footprints exempt as stage-one placeholders — and geometry projection into `out/footprints/*.kicad_mod` + IPC-2581 `Pin` elements; starter pads in `std/pads.cohdl`, remaining placeholders in `std/footprints.cohdl`), mechanical locating holes (RFC-022: `mount_hole N: PLATING at (x, y) diameter D`, disjoint from pad numbering, E810; RFC-023 extends it with an optional `shape:` reusing RFC-018's `PadShape` set plus a shape-dependent geometry field — `diameter D` for a circle, `size: (w, h)` for rect/oval, absent `shape:` defaults to circle), and instance arrays + range references (RFC-024: `inst NAME[START..=END]: Device` and `NAME[S..=E].PIN` / `NAME[S..=E step N].PIN` / `NAME[i, j, k].PIN` inside net-member lists ONLY — pure expansion sugar, expanded in `check/expand.rs`, E202 for an out-of-range index and E211 for malformed/misplaced selectors; does NOT solve daisy-chain wiring or arithmetic-derived `place` data), and the VS Code extension (RFC-019/DR-025: a real installable extension at `editors/vscode/` — TextMate grammar + `vscode-languageclient` wiring over `cohdl lsp`, `cohdl.path` setting; TypeScript/Node package with its own CI job and grammar-coverage test — ZERO compiler changes; the dependency-free constitution scopes the Rust crate, not this editor package; live-VS-Code session is a human checkpoint like KiCad). RFC-001…024 are Accepted and implemented. Deviations from accepted text are ledgered in `docs/compliance-report.md`.

## Commands

- `cargo build` / `cargo test` — build and run all tests
- `cargo run -- check <file-or-dir> [--json]` — parse + resolve + type-check + residual DRC (RFC-010 `--json`)
- `cargo run -- build <file-or-dir> [--json] [--emit ipc2581]` — check + emit KiCad `.net` + BOM CSV (+ `design.lock`; `--emit ipc2581` adds the RFC-015 partner document)
- `cargo run -- fmt <file-or-dir> [--check]` — rewrite `.cohdl` into canonical form (RFC-009)
- `cargo run -- lsp` — the LSP server on stdio (RFC-014; editor setup in `docs/lsp.md`)
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
- `src/emit/` — KiCad `.net` (S-expr) + BOM CSV + `layout.json` (RFC-013) + IPC-2581 XML (RFC-015)
- `std/` — MVP std library (only what the demo board needs)
- `tests/` — fixture tests mapped 1:1 to the MVP exit criteria

## Non-negotiable invariants (from the Constitution / RFCs)

- Same source + same std → same verdict, same designators, same netlist **bytes**. No HashMap iteration order leaking into output — use BTreeMap or explicit sorts everywhere output-adjacent.
- Zero unit coercion; a bare number where a unit is expected is a compile error naming expected vs. actual.
- Every diagnostic names the exact construct (instance, pin, trait, unit) — never a bare "type mismatch".
- The designator allocator asserts injectivity as a postcondition on every run.
- Unicode `Ω` / `°C` must produce a targeted "use `ohm` / use `C`" diagnostic, not a generic lex error.
