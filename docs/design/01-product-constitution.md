# 1. Product Constitution

# Status

v2 — ground-up language redesign, started 2026-07-13. The founding decisions below (mission, north star, domain boundary, interaction model) are carried forward unchanged from v1 — they are locked. Everything below the founding decisions — the conceptual model, the language's actual shape — is void and being redesigned from scratch. Do not assume any v1 syntax, concept, or mechanism survives unless it's re-derived here.

# Mission

CoHDL exists to make **schematic design AI-native**: a text programming language for PCBs in which an AI can express a complete, correct board from high-level intent, and a compiler can *prove* — mechanically, deterministically — whether that board makes electrical sense.

Software became AI-native because code is text and compilers/tests give machine-gradeable truth. Hardware schematic capture never crossed that line: it lives in GUI EDA tools whose "correctness" is locked inside proprietary UIs and human judgment. CoHDL's bet is that if a board is *text with a type system and a design-rule engine*, then the same generate → check → repair loop that made LLMs productive at software becomes possible for hardware.

> If CoHDL succeeds, "design me a board that does X" becomes a compilable, gradeable, repairable artifact — not a request for a human to open KiCad.
>

# North star (the tie-breaker) — unchanged from v1

**AI-native wins.** When two designs are both defensible, we pick the one that is easier for an AI to generate correctly and easier for the compiler to grade. Human ergonomics are a first-class concern — but they are the *second* priority, not the first.

Concretely, "AI-native" means three properties, in this priority order:

1. **Generatable** — the language surface is regular, local, and low-ambiguity, so a model can emit valid source without memorizing special cases.
2. **Gradeable** — every notion of "correct" is reduced to a deterministic compiler signal (parse / type / connectivity / DRC), so correctness is a machine reward, not a human opinion.
3. **Repairable** — when the design is wrong, the compiler's feedback is precise, localized, and actionable enough that a model can fix it in the next turn.

# The redesign's sharpened thesis: strictness is what buys expressiveness

This is the new idea this v2 pass adds on top of the unchanged mission above, and it is now the organizing design principle for the Conceptual Model (note 2):

> A language isn't strict instead of expressive, or expressive instead of strict — a strict compiler is what makes bold, generic, composable code safe to write in the first place. Rust's actual lesson isn't "add a borrow checker," it's that pushing correctness into the type system (making illegal states unrepresentable) is what lets you write fearless, highly composable, generic code on top of it — because the compiler, not convention, is watching your back.
>

Translated to hardware:

- Strict means: as many electrical/engineering mistakes as possible are caught as type errors at compile time, not as DRC violations discovered later, and definitely not silently accepted. A pin that's never connected, a spec that's missing, a trait requirement that isn't satisfied — these should be things the grammar and type system make impossible to express incorrectly, not things a separate rule engine has to notice after the fact.
- Expressive means: generics over specs, trait-bounded devices, composable sub-circuits, and pattern-matching over pin structures should let an author (human or AI) say a lot with orthogonal, reusable pieces — not verbose one-off declarations for every variant.

This reframes the v1 lesson (dormant DRC rules = missing reward signal) one level deeper: the real fix isn't just "wire the DRC rules," it's "ask why a DRC rule was needed for something the type system could have refused to parse/type-check in the first place." DRC is reserved for checks that are inherently cross-cutting or numeric/analog in nature (net voltage exceeds a rating, multiple drivers on a net) — not for structural mistakes a stricter grammar or type system could reject outright.

# Target users (in priority order) — unchanged from v1

1. **AI systems** (LLMs and RL agents) that author and repair `.cohdl` source — the primary "author."
2. **Hardware engineers** who read, review, correct, and trust the generated source — the primary "reviewer" and the ultimate accountable party.
3. **Tool builders** who consume the compiler's IR, diagnostics, and netlists (agents, CI, EDA integrations).

# Core jobs (what CoHDL is for) — unchanged from v1

- Turn a high-level hardware intent into a **complete, connected, spec'd** board description in text.
- Give a deterministic verdict on whether that description is electrically coherent — and push as much of that verdict as possible into compile-time type checking rather than post-hoc rule checks.
- Emit **industry-standard netlists + BOM** (KiCad `.net`, LCEDA `.enet`, BOM CSV) so the design flows into real layout and fabrication.
- Persist **stable identity** across revisions (reference designators) so an evolving design doesn't reshuffle a physical layout.

# Non-goals (what CoHDL deliberately does NOT do) — unchanged from v1

- Not an FPGA / RTL / logic-synthesis language. No HIR, no RTL lowering, no VHDL/Verilog.
- Not a layout / place-and-route engine. Physical placement and routing are a partner concern. CoHDL describes what connects to what and to what spec — not where copper goes.
- **Not a GUI schematic capture tool.** CoHDL is text-first; a canvas is at most a *view*, never the source of truth.
- Not a simulator (SPICE). We check structural, type-level, and rule correctness, not analog waveform behavior. (A door, not a promise — see Evolution Governance.)
- **Not a component database / distributor.** Part sourcing (MPN, AVL, stock) is *referenced* by the language, not *owned* by it.
- **No feature is added only because KiCad, Altium, or another product has it.**
- Not Rust, and not bound by Rust's specific mechanisms. Rust is a syntax and philosophy inspiration (strictness enabling expressiveness, trait-based composition, exhaustive matching), not a spec to port. We deliberately drop what hardware doesn't need — most notably full ownership/borrow-checking — because CoHDL's "memory" is a netlist graph, not a heap; the strictness we want is electrical/structural, not about aliasing and lifetimes.

# The layout door — unchanged from v1

Layout/routing stays out of scope today, but the conceptual model must leave a clean seam for layout constraints (placement hints, net classes, routing rules, differential pairs, length matching) to attach later — as declarative constraints on the netlist, consumed by a partner backend, never as an in-language router.

# Trade-off priority ladder (when values conflict, higher wins) — re-anchored for v2

1. Correctness / gradeability — and specifically: prefer catching a mistake as a compile-time type error over a DRC violation over a runtime/human-review catch, whenever the mistake is structural rather than genuinely cross-cutting or numeric.
2. **AI-generatability** — regularity and locality of the language surface.
3. **Human reviewability & trust** — a human must be able to predict and audit what the source means.
4. Composability — concepts combine cleanly; generics, traits, and sub-circuits reuse without special cases.
5. **Ecosystem fidelity** — faithful, lossless output to real EDA formats.
6. **Human authoring convenience** — nice-to-type syntax sugar.
7. **Feature breadth** — number of supported parts/rules/targets.

New tie-break added by this redesign: when a mistake could be caught either by the type system or by a DRC rule, prefer the type system — it ranks higher on gradeability (it's checked before the design is even fully built) and generatability (the model gets the error at the point of authoring the bad line, not after assembling a whole design). DRC survives for checks that are genuinely emergent from the whole graph (net-level voltage/current, multi-driver conflicts) — not for anything expressible as "this field/trait/pin was missing or wrong-typed."

# Design principles — re-anchored for v2

- **Text is the source of truth.** Everything — components, nets, specs, rules — is expressible and diffable in `.cohdl`. No hidden state in a binary or a GUI.
- The type system is the first oracle; DRC is the second. "Correct" is defined by a deterministic pipeline, but the pipeline should make illegal states unrepresentable wherever a type can do the job, and reserve rule blocks for what's inherently cross-cutting.
- Explicitness over hidden magic. A net is connected because the source says so. No auto-wiring the model can't see or explain. No pin is "probably fine to leave unconnected" — silence is not a valid electrical state; the language forces an explicit decision (connect it, or explicitly mark it not-connected).
- **Locality of meaning.** Reading one module should not require reading the whole design. Errors should point at the smallest responsible span.
- Strictness buys expressiveness. Because illegal states are unrepresentable, generics/traits/pattern-matching can be pushed further than a looser language would dare — the compiler, not convention, keeps composition safe.
- **Regularity over cleverness.** One consistent way to express a thing beats several clever ones.
- **Stable concepts over convenient features.** A new concept must be worth its permanent learning and generation cost.
- **Identity is persistent.** Reference designators and design identity survive edits.
- **Tooling and operations are part of the product** — diagnostics, LSP, formatter, and the feedback surface an AI repairs against are the AI-native interface, not afterthoughts.

# Hard constraints (must never be violated) — unchanged, with one addition

- The language must be **parseable by a deterministic grammar** with no unbounded lookahead or context-sensitive tricks that a model can't reliably reproduce.
- Every diagnostic must carry a **precise source span** and a stable, documented **error code**.
- Correctness must be **reproducible**: same source + same std version → same verdict, same designators, same netlist bytes.
- Generated netlists must be **faithful and lossless** with respect to the design's connectivity and specs — no silent drops.
- No language feature may exist that the type system or DRC cannot inspect (nothing "correct" purely by convention).
- New: no pin, spec, or trait requirement may be left in an ambiguous or implicit state that the type checker resolves by silent default or by skipping. If it can't be verified, it must fail to compile — not fail to fire a rule later. (This is the direct answer to v1's dormant-rule failure mode: push the check to compile time, and if it can't be pushed there, make its DRC-time absence a compile error, not a silent gap.)

# Compatibility promises — unchanged from v1

- Source compatibility across minor versions.
- Designator stability via design.lock.
- Output-format versioning for emitted netlists.
- Error-code stability — codes are deprecated, never silently repurposed.

# Long-term philosophy

CoHDL is a conceptual system with a memory, not a growing pile of parser rules. Its power comes from a small set of orthogonal concepts an AI can generate and a human can trust — governed so that as it absorbs more parts, more rules, and more targets, it stays explainable. This redesign's specific bet is that the way to keep that promise and grow expressive power is to keep pushing correctness earlier — into types, not into rules, and into rules, not into review.
