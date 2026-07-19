# cohdl.dev — Coherent Design Repository

# What this is

This is the living design repository for CoHDL, built by applying the *Coherent Complex Product Design* methodology to a single, sharpened goal:

> **CoHDL is the programming language that makes schematic (PCB) design AI-native** — the way software became AI-native. AI generates and repairs hardware; the compiler is the oracle that grades it.
>

This is not a feature backlog. It is the memory and governance system for an evolving conceptual system. Every feature must trace to a goal, every goal to a principle, every principle to a constraint, and every change must be checked against the whole.

# Status: v2 ground-up language redesign — full backlog Accepted, layout door opened (2026-07-13)

The founding decisions below are locked and unchanged. Notes 2 through 10 were reset and rewritten from scratch on 2026-07-13 per GC-003: the language's conceptual model was redesigned around a new central thesis, strictness buys expressiveness — push correctness into the type system wherever a mistake is structural (Rust-inspired, not Rust-copied), and reserve DRC only for genuinely emergent/numeric checks. All thirteen RFCs (RFC-001 through RFC-013) are now Accepted — units, pins, traits, DRC narrowing, designators, nested fns, generics (the seven P0 Layer-1 mechanisms), plus structural variants, cohdl fmt, cohdl check --json, the error-code registry, #[intent(...)] annotations, and — via GC-002's same-day amendment opening the previously-gated layout door — layout constraints. The MVP implementation was independently verified on the real conol-ai/cohdl repository (65 passing tests, self-audited compliance report, KiCad-verified demo) partway through this backlog and is now being extended to match the newer RFCs by a separate implementation-focused agent. See DR-005/DR-006 for the redesign's rationale, and note 8 for GC-002's amendment (the layout door, opened ahead of its original "concrete partner requirement" gate, per Tony's explicit decision). Note 9 (MVP Definition) has been reactivated — see its current content for the v0.1 scope line and demo scenario. Layout constraints' four-kind vocabulary (note 10) is explicitly flagged provisional pending a real partner integration.

# The founding decisions (locked with Tony, 2026-07-09)

These five choices anchor everything below. They are recorded as decision records in note 7 and must not be changed silently.

| # | Question | Locked answer |
|---|---|---|
| 1 | North star when values conflict | **AI-native.** The language exists so AI can reliably generate/verify hardware. Human ergonomics matter, but AI-generatability + machine-gradeable correctness win ties. |
| 2 | How AI interacts | **AI writes **`.cohdl`** source directly** (text-in / text-out). The compiler is the verifier/oracle in a generate → check → repair loop. |
| 3 | Scope of this design pass | Full 8-part living repository. |
| 4 | Domain boundary | **PCB / schematic now.** Layout & routing stay a partner concern (Quilter / DREAMPlace), but the conceptual model must leave a clean door for *layout constraints* later. |
| 5 | Where it lives | Co-located under the CoHDL product note (this tree). |

# The repository

Read these in order. Notes 1–8 are the eight-part living-system framework; note 9 packages that framework's critical path into a shippable v0.1 scope line.

1. **Product Constitution** — why CoHDL exists, who it's for, non-goals, the trade-off priority ladder, hard constraints, compatibility promises.
2. **Conceptual Model** — the stable concepts AI and humans reason with, reframed so the *model itself* is AI-native.
3. **Capability Architecture** — the five capability layers, so no feature is optimized locally.
4. **Principle → Constraint Mapping** — turning philosophy into enforceable design rules.
5. **Coherence Matrix** — how any new capability disturbs the rest of the system.
6. **Feature Proposal Process (RFC)** — the structured gate every major feature passes through.
7. **Decision Records** — the founding decisions plus the format for future ones.
8. **Evolution Governance & Design Regression** — how goals change without the product splitting its personality, plus the design-level regression checklist.
9. **MVP Definition** — reactivated: the hard scope line for v0.1, a demo scenario designed to prove the type-system thesis, and exit criteria — all against the completed Layer-1 spec. The execution checklist lives in Plan & ToDos.
10. **Language Specification** — the living reference book: what the language *currently is*, organized by construct, populated only from Accepted RFCs. RFCs and decision records are the rationale/history; this note is HEAD. Shipping an RFC now includes updating this note in the same change (note 6, lifecycle step 6).

# The operating loop

The methodology's continuous loop, instantiated for CoHDL:

1. Define goals → 2. Derive principles → 3. Convert to constraints → 4. Design the conceptual model → 5. Map capabilities → 6. Evaluate features via RFC → 7. Record decisions → 8. Check coherence → 9. Govern goal changes → 10. Run design regression tests → 11. Iterate.

# One-sentence summary

We are not stacking parser features onto a text HDL. We are building an evolving conceptual system in which every language, type-system, DRC, and tooling decision must explain how it makes hardware design **more reliably generatable and gradeable by AI** without losing the coherence that lets a human still trust the board.
