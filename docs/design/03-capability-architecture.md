# 3. Capability Architecture

# Status

v2 — reset for the ground-up redesign, 2026-07-13. The v1 map showed what a working 141-crate implementation already did. That implementation is being discarded/rewritten to match the new design (per Tony's decision), so this map can no longer honestly claim anything is "✅ working." It instead maps the same five capability layers as a build-order dependency graph for the new design — which layer's guarantees must exist before the next layer can be honest.

Status legend, redefined for a pre-implementation redesign: 🎯 required at v2 launch · 🚪 reserved door (not v2) · ⛔ deliberately cut (see Constitution non-goals).

# Layer 1 — Core concepts (the language engine)

This is where the redesign's central bet lives: push correctness into types, not into a later rule pass.

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

Coherence stakes: almost everything in this layer is now "the thing that used to be a dormant DRC rule." If this layer is done right, Layer 2 (below) shrinks dramatically — which is the whole point of the redesign.

# Layer 2 — Correctness & verification (the oracle)

Narrower than v1 by design. The oracle's job is now split: most of it lives in Layer 1's type checker; only genuinely emergent/numeric checks remain here.

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

Mostly unchanged in purpose from v1; what changes is that its inputs (MPN, specs) are now type-guaranteed complete, so the emitters get simpler, not more complex.

| Capability | Status | Notes |
|---|---|---|
| KiCad legacy `.net` (S-expr) emitter | ✅ | `cohdl-codegen-kicad`; 477-line real output on `conol-pin` |
| LCEDA Pro `.enet` JSON (v2.0.0) emitter | ✅ | `cohdl-codegen-lceda`; footprints resolved |
| BOM CSV — simple + AVL | ✅ | but emits `<UNSPECIFIED>` until MPN propagation fixed (Layer 1) |
| Format versioning stamped in output | ◐ | `.enet` carries v2.0.0; formalize as compat promise |
| Layout-constraint pass-through to backend | 🚪 | reserved door — net classes / placement hints faithfully carried to Quilter, ignored losslessly by targets that don't consume them |
| Additional targets (Altium, gEDA, SPICE deck) | ○ | evaluate via RFC; not by "competitor has it" |

# Layer 4 — Authoring & tooling (the AI-native interface)

Unchanged in importance; unchanged in requirement that tooling is the product, not polish.

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

Unchanged in importance; sequencing is, if anything, more strict this time, because the redesign's whole premise is "don't build the reward loop on a lying oracle" — and this time we're building the oracle right from the start instead of discovering it lied after the fact.

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

Unchanged mechanism from v1: a feature proposal states which layer(s) it touches and which verdict rung it strengthens. What changed is the map's shape — Layer 1 is now doing more of the correctness work, Layer 2 is deliberately smaller, and the bar for adding anything to Layer 2 is "prove it can't be a Layer-1 type-system mechanism first" (see Conceptual Model's new model smell).

The v2 critical path, in priority order:

1. Design and freeze the type system's strictness mechanisms (Layer 1: units-as-types, trait-at-impl-time checking, pin obligations, generics) — this is the actual redesign; everything else is downstream of it.
2. Design the narrowed residual-DRC + collision-free designator allocator (Layer 2) — prove by construction, not by patching after a bug report.
3. Co-design the error-code registry + cohdl check --json schema with the type checker itself (Layers 2/4) — don't defer this to "after the compiler works," since in v1 deferring it is exactly what let the schema lag behind the implementation.
4. Only then build the repair loop and RL environment (Layer 5) — same discipline as v1, reasserted with more force since there's no partial implementation to fall back on if this order is skipped.
