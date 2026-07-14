# 3. Capability Architecture

# Status

v2 — updated 2026-07-13 to reflect the complete backlog (RFC-001 through RFC-014, all Accepted) and the real, independently-verified implementation on conol-ai/cohdl's main branch (131 passing tests, confirmed by direct inspection, not memory). This replaces the prior version of this note, which described the discarded v1 implementation's status symbols against v2-named capabilities — an artifact of writing this map before any RFC had landed.

Status legend:

- ✅ designed + implemented — Accepted RFC exists AND confirmed present/working in the real main branch.
- 📐 designed, implementation pending — Accepted RFC exists (note 6/7/10 all in sync) but the real repo does not yet implement it.
- 🚪 reserved door, exercised — a seam note 2 pre-designed has now been formally opened via an Accepted RFC (layout constraints).
- ⛔ deliberately cut — see Constitution non-goals or note 9's MVP scope line.

# Layer 1 — Core concepts (the language engine)

| Capability | Status | Notes |
|---|---|---|
| Typed AST with spans on every node | ✅ | `cohdl-syntax`, ~965 lines |
| Deterministic PEG grammar + CST→AST lowering | ✅ | `grammar.pest` 252 lines; hard-constraint per Constitution |
| Name resolution (modules, `use`, visibility, 3-pass) | ✅ | `cohdl-sema` |
| Type checking + monomorphization + fn inlining → TypedDesign IR | ✅ | the "resolves + type-checks" verdict rung |
| Connectivity (union-find net merging) → ConnectivityIR | ✅ | the "connects" verdict rung |
| Nested `fn` calls (fn inside fn) | ◐ | **silently skipped** (`typeck.rs:1371`) — breaks composability principle |
| MPN propagation to instances | ◐ | field exists, type checker never populates it → BOM `<UNSPECIFIED>` |
| Intent annotations (`#[intent(...)]`) | 🚪 | reserved seam; metadata, never affects netlist |

# Layer 2 — Correctness & verification (the oracle)

| Capability | Status | Notes |
|---|---|---|
| DRC engine: expression evaluator + `rule` blocks | ✅ | `cohdl-drc` |
| Built-in rules E001, E002, W001, W002 | ✅ | fire on `drc_violations` fixture |
| Built-in rules E003 (SpecNotSatisfied), E004 (TraitNotImpl), E005 (MissingSpecField), W003 (SingleDriver), W004 (MultiDriver) | ◐ | **structurally present, not wired** — depend on `generic_substitutions` meta-keys the type checker never populates |
| Designator assignment via `design.lock` + tombstones | ✅ | persistent-identity artifact |
| Designator allocator correctness | ◐ | **collision bug**: `esd` and `ldo33` both got `U3` in `conol-pin` |
| Stable, documented error-code registry | ◐ | codes exist; formalizing them as a stable public contract is an AI-native requirement (RFC-003) |
| Machine-actionable repair suggestions in diagnostics | ○ | the seed of auto-repair; not built |
| Design Verdict as an explicit, queryable ladder | ○ | pipeline runs the stages but doesn't expose the monotonic reward ladder as an artifact |

# Layer 3 — Ecosystem output (the bridge to the physical world)

| Capability | Status | Notes |
|---|---|---|
| KiCad legacy `.net` (S-expr) emitter | ✅ | `cohdl-codegen-kicad`; 477-line real output on `conol-pin` |
| LCEDA Pro `.enet` JSON (v2.0.0) emitter | ✅ | `cohdl-codegen-lceda`; footprints resolved |
| BOM CSV — simple + AVL | ✅ | but emits `<UNSPECIFIED>` until MPN propagation fixed (Layer 1) |
| Format versioning stamped in output | ◐ | `.enet` carries v2.0.0; formalize as compat promise |
| Layout-constraint pass-through to backend | 🚪 | reserved door — net classes / placement hints faithfully carried to Quilter, ignored losslessly by targets that don't consume them |
| Additional targets (Altium, gEDA, SPICE deck) | ○ | evaluate via RFC; not by "competitor has it" |

# Layer 4 — Authoring & tooling (the AI-native interface)

| Capability | Status | Notes |
|---|---|---|
| LSP: diagnostics, hover, goto-def, completion | ✅ | `cohdl-lsp`, tower-lsp |
| CLI: `build` / `check` / `init` | ✅ | rustc-style colored diagnostics |
| `cohdl fmt` | ◐ | **placeholder** — "not yet implemented." Formatting matters for AI: a canonical form makes diffs & generation stable |
| Incremental compilation | ○ | LSP re-runs full pipeline every keystroke; fine for MVP, matters at scale |
| CLI ↔ fixture module-wrapping consistency | ◐ | `resolve_modules()` wraps non-main files; some fixtures only build under e2e flat-concat |
| VS Code extension | ✅ | TextMate grammar + `extension.ts` |
| **Agent-facing verify API** (stable JSON: verdict + diagnostics) | ○ | the single most important AI-native tooling gap: a stable, structured `cohdl check --json` the repair loop keys off |
| **Batch / server mode** for high-throughput generation grading | ○ | needed for RL / dataset generation at scale |

# Layer 5 — AI generation loop (the reason the language exists)

| Capability | Status | Notes |
|---|---|---|
| Compiler-as-oracle in a generate→check→repair loop | ◐ | conceptually the plan; needs the stable JSON verify API (Layer 4) to be real |
| Prompt scaffolding: language spec → context for LLM | ◐ | 16-page mdBook reference exists and is good raw material |
| Repair loop: feed diagnostics back to model, regenerate | ○ | depends on stable codes + actionable suggestions (Layer 2) |
| Corpus / dataset of (intent → valid `.cohdl` → verdict) | ○ | the training substrate; fixtures are a seed |
| RL environment (reward = Design Verdict ladder) | ○ | requires batch mode + explicit verdict artifact |
| Fine-tuned / specialized generation model | ○ | downstream of corpus + environment |

# Documentation & reference (cross-cutting)

| Capability | Status | Notes |
|---|---|---|
| mdBook language reference (16 pages) | ✅ | `docs/src/`; doubles as LLM grounding context |
| This design repository | ◐ | being built now — the governance memory |

# How to read this map when proposing a feature

Unchanged mechanism: a feature proposal states which layer(s) it touches and which verdict rung it strengthens. The map distinguishes designed (RFC Accepted) from implemented (verified in real source) — a real, separate implementation-focused agent is currently working through the 📐 items to bring the actual conol-ai/cohdl repository up to the full RFC-001–014 design. This design repository (notes 1–10) is fully specified; the remaining work is implementation catch-up, not further design — with one exception: RFC-013's layout-constraint vocabulary is explicitly flagged provisional and may need a follow-up RFC once a real partner integration is scoped (see note 8, GC-002's amendment).
