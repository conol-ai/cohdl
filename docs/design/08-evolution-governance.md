# 8. Evolution Governance & Design Regression

# Purpose — unchanged

Goals can change — but not silently. This note holds the governance process, the goal-change proposals, and the design-regression checklist.

# Goal-change proposal format — unchanged

```text
# GC-NNNN: [title]
Original goal · New goal · Reason · Strategic context ·
Affected users · Affected principles · Affected concepts ·
Affected capabilities · Compatibility risks · Required migrations ·
Design debt created · Operational risks · Communication plan ·
Decision: accept / reject / stage / experiment
```

# GC-001: Reframe CoHDL from "text HDL that's AI-generatable" to "the AI-native schematic language" — unchanged, still accepted

Carried forward from v1 with no changes — this GC is about the mission, which the 2026-07-13 redesign explicitly kept locked. Full text preserved below for continuity.

- **Original goal** — A text-based PCB HDL; AI generation is a promising downstream hypothesis.
- New goal — CoHDL exists to make schematic design AI-native.
- Reason — Tony's directive (2026-07-09).
- Decision — Accepted (2026-07-09). Still in force; unaffected by the 2026-07-13 conceptual-model redesign.

# GC-002: Admit layout *constraints* into the conceptual model — AMENDED, now Accepted (2026-07-13)

Carried forward from v1 as "staged, requires a concrete partner requirement." Amended and Accepted 2026-07-13 per Tony's explicit directive to open the layout door now — see the full amended proposal as its own child note for the complete GC-NNNN format writeup (original goal, reason, strategic context, affected users/principles/concepts/capabilities, compatibility risks, design debt, decision). Key point carried into that note honestly: the original "concrete partner requirement" trigger was not met — this decision explicitly waives it rather than pretending it was satisfied.

- Decision — Accepted (2026-07-13), amended from "staged." See child note for full text. Resulting RFC: RFC-013 (note 6).

# GC-003: Ground-up redesign of the conceptual model around "strictness buys expressiveness" (NEW)

- Original goal — CoHDL's conceptual model (Trait/Device/Part/Instance/Pin/Net/Spec/Rule/Module/Fn/Design/Designator) as shaped by the v1 implementation, with a DRC layer carrying most of the correctness burden beyond parsing/type-checking.
- New goal — The same electrical primitives, re-expressed so that structural correctness (units, trait satisfaction, pin connection completeness, MPN completeness) is enforced by the type system at the earliest possible point, with DRC narrowed to genuinely emergent/numeric checks only. Explicitly Rust-inspired (not Rust-copied): strictness mechanisms exist specifically to license more ambitious generics/trait composition, not merely to add friction.
- Reason — Tony's directive (2026-07-13): redesign the language from the ground up to deliver two "tastes" — hard to make mistakes in (for both human and AI authors) and expressive. Forgetting v1's conceptual-model content while keeping the governance scaffolding (RFC template, Coherence Matrix, decision-record format).
- Strategic context — v1's own postmortem (DR-004, now superseded by DR-006) already identified that dormant DRC rules were a critical defect; this GC generalizes that lesson into a design principle instead of a bug-fix list, and applies it before any more implementation work happens, rather than patching an existing compiler.
- Affected users — No change to priority order (AI author, then human reviewer, then tool builder) — but the promise to each is sharper: an AI author gets earlier, more precise feedback (a type error instead of a late DRC finding); a human reviewer gets a design that's correct-by-construction in more places, so review effort concentrates on genuinely judgment-requiring aspects.
- Affected principles — Adds a new principle, "strictness buys expressiveness," at the same tier as gradeability/generatability (see Constitution + note 4). Sharpens "gradeability" to prefer the earliest possible check stage. Narrows the practical scope of "the compiler is the oracle" specifically for DRC, while strengthening it overall (more of the oracle's job moves earlier, not away).
- Affected concepts — Trait, Pin, Part, and Rule are redefined (see note 2); Device/Instance/Net/Spec/Module/Fn/Design/Designator keep their conceptual shape but gain stricter guarantees. No concept is renamed; the canonical vocabulary is preserved by design (guarding against the "overlapping names" smell during a redesign is exactly when it matters most).
- Affected capabilities — The v1 capability map (note 3) is entirely reset: Layer 1 absorbs work that used to live in Layer 2, and every capability's status resets to "not yet built" since the v1 implementation is being discarded. This is the most disruptive-looking change, but it's disruptive to the implementation timeline, not to the mission or user priority.
- Compatibility risks — None for existing users/customers (nothing has shipped externally as v1 "0.1.0" beyond internal fixtures per the v1 notes). The real risk is internal: any prompt scaffolding, fixtures, or partner conversations (e.g. Quilter fit-analysis) that assumed v1's syntax will need to be revisited once v2's grammar is settled.
- Required migrations — None for external source (there is none). The v1 Rust implementation and its fixtures are not migrated — they're discarded and rewritten against the new design, per Tony's explicit decision.
- Design debt created — A temporary gap where note 9 (MVP Definition) has no content, since scoping an MVP before the Layer-1 RFCs (note 6, RFC-001–007) land would repeat the mistake DR-005/006 are trying to fix (designing downstream before the foundation is settled). This debt is intentional and tracked, not accidental.
- Operational risks — The biggest risk is scope creep in the other direction: over-engineering the type system with mechanisms (e.g. anything ownership/borrow-checking-shaped) that have no corresponding hardware "resource contention" problem to solve. Mitigated by the Conceptual Model's explicit "what this redesign does NOT do" section and by the Coherence Matrix's conceptual-cost scrutiny on every new mechanism.
- Communication plan — This repository (notes 1–8, reset in place; note 9, voided) is the communication of the redesign — same pattern as GC-001's own note.
- Decision — Accepted (2026-07-13). Supersedes the v1 conceptual model in its entirety; recorded also as DR-005/DR-006.

# Compatibility, migration & deprecation policy — unchanged in principle, reset in practice

- Compatibility — Once v2's first stable release ships, .cohdl source is stable across minor versions; error-code meanings are stable; netlist formats are versioned. Until then, nothing is stable — this redesign period is explicitly pre-compatibility-promise, which is why note 9 stays void for now.
- Migration — Every breaking change post-launch ships a changelog entry and, where source is affected, an automated cohdl fmt-based migration or documented manual path.
- Deprecation — Deprecate, don't delete, once there's something to deprecate. Error codes are tombstoned like designators — never repurposed.

# Design regression checklist — carried forward, two items added

Run this for every significant change, now and especially during the redesign itself, since a redesign is exactly when it's easiest to accidentally regress the very things it's trying to fix:

- Can a *new* AI-context author still emit valid source knowing only the core concepts?
- Can an existing human reviewer *predict* what the new behavior means without reading the whole design?
- Does this make CoHDL **easier** to explain, or harder?
- Does it preserve the trade-off priority ladder (no rank-1/2 sacrifice for rank-6 gain)?
- Does it introduce a **special case** an author/AI must memorize?
- Does it create a **second way** to do something the model already does?
- Is every new notion of "correct" backed by a compiler check (gradeable)?
- Does every new/changed diagnostic have a stable code + precise span + actionable message?
- Does it keep the netlist a **faithful, lossless** projection of source?
- Does it preserve reproducibility?
- Does it preserve designator stability?
- Does it change a public surface? If yes → decision record + deprecation plan.
- Does it stay inside the **"not a router" non-goal**?
- Does it make future features easier or harder?
- If we maintain this for five years, is it still worth the conceptual cost?
- (New) If this narrows or removes a DRC/rule check, has a type-system replacement been designed and shown to cover the same ground? (Directly enforces DR-006 — the exact discipline v1 skipped.)
- (New) Does this strictness mechanism have a corresponding expressiveness mechanism in the same change (a generic, a trait-composition rule, a pattern-matching form), or does it only add friction without licensing more composable code on top of it? (Enforces the redesign's central thesis — strictness must pay for itself in expressiveness, not just exist for its own sake.)

# The one rule that holds it together — unchanged

> CoHDL can evolve without drifting **only if every change is explicit, traceable, and system-aware.** We are not stacking parser features. We are maintaining an evolving conceptual system where every change must explain which goal it serves, which principle it follows, which concept it touches, which capability it affects, how the compiler grades it, and how it avoids product drift.
>
