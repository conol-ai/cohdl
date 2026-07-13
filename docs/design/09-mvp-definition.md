# 9. MVP Definition

# Status: reactivated 2026-07-13 — all seven P0 RFCs Accepted

This note was intentionally void from the start of the ground-up redesign (see the prior version's own text, preserved in note history) until the Layer-1 type-system foundation was actually, formally specified — not aspirational. That condition is now met: RFC-001 through RFC-007 are all Accepted (see 6. Feature Proposal Process (RFC)'s milestone entry), and 10. Language Specification is a real, implementable reference — units, pins, traits, the narrowed residual-DRC surface, designators, nested fns, and generics. This note now does what it was always meant to: draw the hard scope line for v0.1, define the demo that counts as proof, and state exit criteria — against the actual design, not a placeholder.

# The one job the MVP must do — sharpened for v2

> Take a plain-language hardware intent, have an AI emit .cohdl source, have the type checker — not a DRC afterthought — catch structural mistakes (wrong units, unresolved pins, unsatisfied trait bounds) as compile errors the moment they're written, have the AI repair them using those precise diagnostics, and land on a design that emits a real, importable KiCad netlist + BOM — with the narrowed residual-DRC engine catching only what's genuinely emergent (net voltage, driver conflicts).
>

This is a sharper bar than the original (v1-era) MVP Definition set. That version's proof was "the loop closes and the netlist is honest." This version's proof must additionally show the type system doing the catching that DRC used to (fail to) do — because that's the actual thing the redesign bought, and an MVP that doesn't demonstrate it hasn't proven the redesign's central bet, only that a compiler exists.

# Why this MVP is scoped narrower than "implement everything in the spec"

Inversion check: the way this MVP fails is by trying to build a production-grade compiler (full grammar, full LSP, full codegen parity with v1) before proving the core bet works at all. The spec (notes 1–10) defines the target language; it does not require building every corner of it before the first proof. This MVP is scoped to the smallest slice of the spec that can demonstrate the thesis end-to-end — not the whole spec.

# Scope line

## In scope for MVP (v0.1)

Grammar & parser — enough of the deterministic grammar to parse:

- trait declarations (pins {}, spec {}, sub-trait bounds)
- device declarations (pins with required/optional, spec {}, generic parameters — both unit-type and trait-bound)
- Free-standing impl Trait for Device statements (empty-body and mapped-body forms)
- fn declarations and calls, including nested calls
- design bodies: inst, net, nc
- The ten unit-type literal forms (RFC-001's table, including the Temperature/Tolerance exceptions)

Type checker — the actual heart of this MVP:

- Unit-type checking with zero coercion (RFC-001)
- Pin connection-obligation exhaustiveness (RFC-002)
- Trait satisfaction at impl statements, by-name matching + explicit mapping (RFC-003)
- Generic parameter resolution: unit-type substitution and trait-bound-at-instantiation checking (RFC-007)
- Nested fn call expansion with correct substitution threading and call-chain-path naming, plus cycle detection (RFC-006)

Residual DRC — the narrowed four-rule engine only (RFC-004): voltage-exceed, polarity-mismatch, single-driver, multi-driver. Nothing beyond this list.

Designators — the collision-free allocator (RFC-005), design.lock with tombstones, #[designator("Xxx")] overrides.

Codegen — KiCad .net emitter and BOM CSV, faithful to whatever the (much smaller, MVP-scope) std library and demo board actually need. LCEDA .enet is not required for MVP (see cut list) — one real target format is enough to prove netlist fidelity; a second is redundant proof for a v0.1.

Minimal std library — only the traits/devices the demo board needs (see Demo scenario below) — not a general-purpose parts library. Every device/trait added must be added because the demo needs it, not speculatively.

A generate → check → repair harness — a script (not a product UI) that: prompts an LLM with the language reference (note 10) for .cohdl source, runs the type checker + residual DRC, feeds diagnostics back verbatim on failure, and repeats until clean or an attempt cap is hit.

## Explicitly cut from MVP (do not build)

- cohdl fmt canonical formatter (RFC-009, not yet even drafted) — nice for stable diffs, not required to prove the core thesis. A v1-era formatter placeholder is acceptable for this MVP; ship a real one before any public release, not before this proof.
- cohdl check --json structured verify API (RFC-010, not yet drafted) — the repair harness can parse CLI diagnostic output directly for MVP purposes; a stable JSON schema is a real requirement before Layer 5 (RL environment, batch grading) but not before this first proof.
- Formal error-code registry (RFC-011) — codes can exist informally (stable strings, just not yet a published, versioned registry) for MVP; formalize before any external-facing release.
- Exhaustive pattern-matching over structural variants (RFC-008) — no demo-board device needs package/pin-role variants complex enough to require this; defer until a real std-library device does.
- #[intent(...)] annotations (RFC-012) — explicitly gated on zero-netlist-impact and not urgent; unrelated to proving the type-system thesis.
- The layout-constraint door / any layout work (RFC-013, gated behind its own goal-change proposal) — completely untouched, per the Constitution's non-goal and DR-003.
- LCEDA .enet emitter, additional codegen targets — one faithful target (KiCad) is sufficient proof; adding a second is feature-breadth (ladder rank 7), the lowest priority.
- RL environment, batch/server mode, fine-tuned generation model — Layer 5 work explicitly sequenced after this MVP proves the loop closes at all with an off-the-shelf model.
- Full LSP (hover, goto-def, completion, "find all impls" navigation) — nice-to-have tooling flagged throughout the RFCs' "Tooling & operations" sections, but not required to run the demo harness. A bare CLI (build/check) is sufficient for MVP.
- Incremental compilation — irrelevant at MVP scale (one small demo board).

## What makes this cut list different from the original (v1-era) MVP's cut list

The original MVP's Phase 1 ("Honest Oracle") was about fixing an existing implementation's defects. This MVP has no existing implementation to fix — it's building fresh, informed by the design. The equivalent discipline here is: build only the type-checker mechanisms the demo board actually exercises, in the order the P0 RFCs were decided (units → pins → traits → DRC narrowing → designators → nested fns → generics), since that's also a reasonable implementation dependency order (each RFC's mechanism is a prerequisite for the ones after it in the demo's own complexity).

# Demo scenario (the acceptance test)

A single concrete scenario, deliberately designed to exercise the type system's new catches, not just "does it eventually compile":

1. Input: a one-paragraph natural-language spec for a small board — reuse the same domain as the original MVP's scenario for continuity: "An ESP32-S3-based sensor node: USB-C power/data, one MEMS microphone, one status LED, a 3.3V LDO regulator, standard decoupling."
2. Generate (attempt 1): an LLM (no fine-tuning), prompted with note 10's language reference plus a handful of example .cohdl snippets (device/trait/impl/fn examples already in note 10), emits a first-draft .cohdl source file.
3. Check (attempt 1): run the type checker. This attempt is expected to fail — the demo is specifically designed to be run enough times that at least one early attempt hits at least one of: a unit-type mismatch, an unresolved required pin, or an unsatisfied trait bound (impl missing or mismatched). If attempt 1 happens to be clean, that's fine, but the transcript must show the type checker's diagnostics firing correctly on some attempt during the run — this is non-negotiable, because catching this class of mistake at compile time is the entire point of the MVP, not an optional nice-to-have if the model happens to get it right immediately.
4. Repair: diagnostics are fed back to the model verbatim; it regenerates; repeat up to N attempts (suggest N ≤ 5).
5. Land: the design reaches a clean verdict — parses, resolves, type-checks (units sound, pins resolved, traits satisfied, generics resolved) — passes the narrowed 4-rule residual DRC, and cohdl build emits a real .net + BOM CSV.
6. Human checkpoint: open the .net in real KiCad, visually confirm a coherent, connected schematic with real designators (no collisions) and a BOM with real MPNs (via part bindings) for every instance.

What's different from the original MVP's demo: step 3 is now a required part of the proof, not incidental. The demo record must show at least one genuine type-checker catch — a specific diagnostic, naming a specific line, that the model then correctly repaired. Without that, the demo proves "a compiler exists" but not "the compiler catches the things DRC used to miss," which is the actual thesis.

# Exit criteria (definition of done)

All of the following, true simultaneously:

- Grammar parses every construct listed in "In scope" above, on at least the demo board's actual source.
- Unit-type checking fires correctly: a fixture with a deliberately wrong-unit spec produces the correct diagnostic, naming the expected vs. actual unit type.
- Pin exhaustiveness fires correctly: a fixture with an unresolved required pin produces the correct diagnostic; a fixture with a pin in both net and nc produces the contradictory-declaration diagnostic.
- Trait satisfaction fires correctly: a fixture with a device missing a required pin/spec for its claimed impl produces the correct diagnostic naming the trait and the gap; a fixture with a missing sub-trait-bound impl produces the correct chain diagnostic.
- Generic trait-bound checking fires correctly: a fixture instantiating a generic with a type argument lacking the required impl produces the correct diagnostic.
- Nested fn calls expand correctly to at least 2 levels in a fixture, with correct substitution threading and no naming collisions; a cyclic-call fixture produces the cycle diagnostic.
- The designator allocator fixture test confirms no collisions across at least one fixture with multiple same-prefix instances (the esd/ldo33-style case).
- The residual DRC engine's 4 rules each fire on a dedicated fixture designed to trigger them.
- The demo scenario runs end-to-end at least once, produces a transcript showing at least one genuine type-checker catch + repair cycle, and the resulting .net/BOM pass the human KiCad checkpoint.
- Every P0 RFC's decision record (note 7) accurately reflects what was actually built (no RFC claims a mechanism that the implementation doesn't actually enforce — this is the exact "dormant rule" failure mode the whole redesign exists to prevent, now checked against the implementation, not just the design).

# What "MVP" explicitly does not mean here

- Not "the full language spec implemented." Notes 1–10 describe the target; this MVP builds the smallest slice that proves the thesis.
- Not "production-hardened, formatted, or fully tooled." cohdl fmt, the JSON API, and the error-code registry are real pre-release requirements, not MVP requirements.
- Not "the AI writes the board perfectly on the first try." The type checker catching a first-draft mistake and the model repairing it is the proof, not a failure.
- Not a UI/product surface. The demo harness is a script; product UX is separate scope (Design 🖼️ note tree).

# How this note relates to the rest of the repository

Scope decisions here must not contradict notes 1–8, or any Accepted RFC (notes under 6). Where this note cuts something (formatter, JSON API, error-code registry, LCEDA, layout), it's citing an existing RFC's own stated priority (P1/P2/Gated) or a Constitution non-goal — not inventing a new cut. The execution checklist — task-level breakdown, sequencing, implementation tracking — belongs in Plan & ToDos as its own note (the old one there is stale and marked as such; a fresh one should replace it, tracking this MVP's scope rather than v1's defect list).
