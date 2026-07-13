# 7. Decision Records

# Purpose — unchanged

Complex systems need memory. Every major decision gets a record: what we chose, the context, the alternatives, why, the trade-offs, and what would make us revisit.

Format for every record:

```text
# DR-NNNN: [title]
## Context — what situation led to this?
## Options — what alternatives were considered?
## Decision — what did we choose?
## Rationale — why?
## Consequences — benefits, costs, risks, constraints that follow.
## Revisit when — what future conditions would justify changing it?
```

# DR-001: CoHDL is AI-native, not human-first — unchanged, still in force

## Context

The product note framed CoHDL as "a text HDL that happens to be AI-generatable." Tony sharpened this on 2026-07-09: CoHDL exists *to make schematic design AI-native*.

## Options

1. Human-first HDL, AI-friendly as a bonus. 2. AI-native: AI-generatability + machine-gradeability win ties. 3. Verifier-first. 4. Co-equal, decide case by case.

## Decision

Option 2 — AI-native. Reaffirmed unchanged by the 2026-07-13 ground-up redesign — this decision is about why CoHDL exists, not how it expresses hardware, so the redesign doesn't touch it.

## Rationale — unchanged

Software became AI-native because code is text + compilers/tests give machine-gradeable truth. CoHDL's differentiator is bringing that loop to hardware.

## Consequences — unchanged

Correctness > generatability > human review > convenience on the priority ladder. Diagnostics/verdict are first-class.

## Revisit when — unchanged

Evidence that AI generation is not the dominant authoring path, or human review is failing because source is too AI-shaped.

# DR-002: AI writes `.cohdl` source directly (text-in / text-out) — unchanged, still in force

Unchanged from v1 in every section — text-in/text-out remains the interaction model; the 2026-07-13 redesign changes the shape of the text, not the fact that it's text.

# DR-003: Layout/routing stays a partner concern; the model reserves a door — unchanged, still in force

Unchanged from v1 in every section. The redesign doesn't touch the layout boundary; the seam described there (declarative constraints on Net/Instance) is exactly as valid against the v2 conceptual model as it was against v1's.

# DR-004 (superseded): Gradeability is rank-1; dormant rules are defects

## Context (historical)

v1 shipped five dormant DRC rules, a designator collision, and unpropagated MPNs. Under AI-native framing these were treated as P0 defects against the rank-1 gradeability principle, ahead of new features.

## What superseded it

DR-004 was correct as far as it went, but it diagnosed the symptom (dormant rules = missing reward signal) without diagnosing the deeper cause (the rule/DRC layer was being asked to catch things a stricter type system should have made unrepresentable in the first place). See DR-006 below, which replaces DR-004's fix-the-symptom framing with a redesign-the-layer framing. DR-004 is kept here for history; do not re-cite it as current guidance — cite DR-006 instead.

# DR-005: Ground-up language redesign — void the v1 conceptual model, keep the mission

## Context

2026-07-13: Tony directed a full redesign of the language "from the ground up," explicitly asking to forget the v1 repository's content while keeping the governance scaffolding (RFC template, Coherence Matrix format, decision-record format). The trigger: a wish for the language to deliver two "tastes" simultaneously — (1) strict, so both humans and AI find it hard to make mistakes, and (2) expressive.

## Options

1. Patch v1's implementation to close its known gaps (dormant rules, designator collision, MPN, nested fn) without touching the conceptual model.
2. Redesign the conceptual model from scratch, keeping the founding decisions (mission, north star, interaction model, domain boundary) locked.
3. Redesign everything, including the founding decisions.
4. Pause and re-scope only the syntax, keeping v1's concept table (Trait/Device/Part/…) as-is.

## Decision

Option 2. The founding decisions (DR-001/002/003) stay locked; the conceptual model (note 2) and everything derived from it (notes 3–9) are redesigned around a new central thesis: strictness buys expressiveness — push correctness into the type system wherever a mistake is structural, and reserve DRC only for genuinely emergent/numeric checks.

## Rationale

Tony's two "tastes" are not actually in tension — they're the same move software languages like Rust already made: a stricter type system is what licenses more ambitious composition (generics, trait bounds, pattern matching) because the compiler, not convention, guarantees safety. Applying this to hardware directly targets v1's own postmortem: every one of v1's headline defects (dormant DRC rules, unpropagated MPN, silently-unconnected pins implied by dormant W003/W004) was a structural mistake that a stricter type system could have made unrepresentable, rather than something that inherently needed a rule engine.

## Consequences

- Notes 1–8 (Constitution through Evolution Governance) are rewritten against the new conceptual model; note 9 (MVP Definition) is voided until the new design is stable enough to scope a v0.1 around.
- The existing Rust implementation (conol-ai/cohdl) is to be discarded/rewritten to match whatever the new design lands on — no code-level work starts until the Layer-1 RFCs (note 6, RFC-001–007) are decided.
- DRC's scope narrows substantially; every v1-era DRC concern must be explicitly reclassified (RFC-004) as "becomes a type mechanism" or "stays residual DRC" before implementation — this is a mandatory step, not an optimization.
- Risk: over-indexing on strictness could hurt authoring convenience or teaching cost. Mitigated by keeping "human authoring convenience" at ladder rank 6 (unchanged) and by requiring every new strictness mechanism to ship an equally-considered expressiveness counterpart (generics, trait composition) in the same RFC, not as an afterthought.

## Revisit when

If, after the redesign ships, the type system is demonstrably harder to generate correctly than v1's looser model was (i.e., strictness cost more generatability than it bought in gradeability) — that would mean the central bet failed and the trade-off ladder needs re-litigating, not just the implementation.

# DR-006: Narrow DRC's job; push structural checks into the type system

## Context

Directly derived from DR-005's redesign thesis. v1's five dormant rules (E003–E005 SpecNotSatisfied/TraitNotImpl/MissingSpecField, W003–W004 SingleDriver/MultiDriver) were structurally present but never wired — DR-004 treated this as "wire them." This record replaces that instruction.

## Options

1. Wire the v1 rules as originally designed, keep DRC's scope as broad as v1's.
2. Reclassify each dormant rule: if it's checking something local to one device/instance/trait (SpecNotSatisfied, TraitNotImpl, MissingSpecField), move it into the type system as a compile-time check; if it's genuinely emergent (SingleDriver/MultiDriver span the whole net graph), keep it as narrowed residual DRC.
3. Delete the checks entirely as out of scope.

## Decision

Option 2. SpecNotSatisfied/TraitNotImpl/MissingSpecField become type-checker diagnostics, checked at impl/declaration time (see note 2's Trait/Part concepts, note 6 RFC-003). SingleDriver/MultiDriver remain residual DRC, since "does this net have more than one driver" is inherently a property of the whole connectivity graph, not of any single device.

## Rationale

A rule engine that only runs after a whole design is assembled is the wrong place to catch a mistake that's fully determined by one device's own definition — it's both a worse AI-generatability experience (the model finds out its mistake much later, with a bigger diff to fix) and a worse gradeability guarantee (v1 proved a "wire it eventually" rule can just stay unwired). Moving these three checks earlier removes the entire class of "it's structurally present but not actually enforced" bug, because there's no second wiring step left to forget.

## Consequences

- RFC-004 (note 6) is the formal mechanism for this reclassification, and must complete before RFC-011 (error-code registry) can finalize its codes.
- The residual DRC engine in v2 is deliberately smaller in scope than v1's — this is a feature of the redesign, not a regression, and should not be treated as "less DRC coverage" without checking the type-system replacement first (Coherence Matrix's mandatory mitigation).
- Every future proposal to add a rule must pass the type-system-first test (RFC template, note 6) before being accepted.

## Revisit when

If a genuine class of structural check is found that truly cannot be expressed in the type system (e.g. requires unbounded lookahead across the whole file to resolve locally) — then, and only then, does it belong back in DRC, and that exception should itself get a decision record.

# DR-007: Units-as-types — a closed, non-coercing set of engineering unit types

## Context

RFC-001 (note 6) proposed the first Layer-1 P0 mechanism: a fixed set of primitive unit types with zero implicit coercion, replacing v1's bare-number/loosely-typed spec fields — a class of mistake v1 had no dedicated compiler coverage for at all. Initially accepted with six types (Voltage, Capacitance, Resistance, Current, Frequency, Time); amended same-day by Tony to ten types, adding Inductance, Power, Temperature, and Tolerance.

## Options

1. Bare numbers + naming-convention units (e.g. capacitance_nF: 100).
2. A closed set of unit primitive types, zero coercion, ASCII-only canonical literal spellings (RFC-001's proposal).
3. A general, user-extensible dimensional-analysis system with unit arithmetic (V = I × R).
4. Leave units untyped indefinitely and rely on human review + narrowed DRC.

## Decision

Option 2. Units are first-class, closed-set primitive types — ten types as of the 2026-07-13 amendment: Voltage, Capacitance, Resistance, Current, Frequency, Time, Inductance, Power, Temperature, Tolerance. No implicit coercion between unit types or from bare numbers. Canonical literal spellings are ASCII-only (ohm not Ω, C not °C) — one spelling per unit, no alternates. No cross-unit arithmetic. Two types (Temperature, Tolerance) take no SI prefix by design; Temperature is the sole type whose literal may carry a leading - sign.

## Rationale

This is the highest-leverage single mechanism in the redesign's P0 backlog: nearly every other P0 RFC (trait bounds carrying spec requirements, generics-over-specs, pin obligations expressed through spec-bearing traits) assumes unit-typed specs already exist. Option 1 was rejected as a "correct-by-convention" smell explicitly forbidden by the Constitution. Option 3 was rejected as conceptual/implementation cost with no corresponding job — CoHDL is a schematic-capture language, not a simulator (SPICE is an explicit non-goal), so unit algebra isn't needed, only unit tagging and comparison. Option 4 was rejected outright — it's the status quo the whole redesign exists to fix. The same-day amendment to ten types followed the RFC's own stated extension path because Inductance/Power were already flagged as likely next candidates in the original RFC text, and Temperature/Tolerance close the same "bare number where a unit is required" gap for two more ubiquitous datasheet quantities.

## Consequences

- Ten primitive types enter the type system (four added same-day post-acceptance) — the largest single conceptual addition of any P0 RFC so far, but justified because it gives Spec (an existing concept) the type system it always needed, rather than inventing something orthogonal.
- Comparison operators inside rule blocks are now type-checked for unit-match across all ten types, closing a residual-DRC failure mode before the narrowed DRC engine (RFC-004) is even designed.
- The unit set is closed at ten for this version — extending it further requires its own RFC, evaluated on conceptual cost like any other addition.
- The grammar must support two prefix-less unit types and one signed-literal exception (Temperature) — small, enumerable table additions, not new grammar mechanisms.
- Reserves an error-code block for unit-system diagnostics ahead of RFC-011's full registry pass.

## Revisit when

A concrete need for derived-unit computation emerges, or the closed set of ten proves too narrow for real std-library growth — either should be its own RFC, not a silent extension of this one.

# DR-012: Pin connection-obligation typing — exhaustive required/optional + explicit nc

## Context

RFC-002 (note 6) proposed the second Layer-1 P0 mechanism: every pin on a device/trait definition carries an explicit obligation kind (required/optional), and every required pin on every instance in a fully-assembled design must resolve to either a net membership or an explicit nc declaration. This directly targets a class of mistake v1 had no real coverage for — a required pin (e.g. an MCU's VDD) silently left off every net, with the closest existing checks (W003/W004 single/multi-driver) not even covering this case.

## Options

1. Leave all pins implicitly optional, as in v1 — no obligation typing at all.
2. Obligation kind declared on the device/trait (required/optional), resolved per-instance via net or a new explicit nc declaration, checked exhaustively at final design assembly (RFC-002's proposal).
3. Catch unconnected pins via a DRC rule that scans the whole connectivity graph after assembly.
4. A general Option-style wrapper type threaded through every pin reference site.

## Decision

Option 2. Pin obligation kind is fixed at the device/trait definition (not overridable per-instance). A required pin must appear in exactly one of {some net, the nc list} by the time a design is fully assembled (after all fn inlining) — appearing in neither, or in both, is a compile error. nc is a new top-level design-body declaration, syntactically parallel to net.

## Rationale

The mistake is structural — a property of one instance's pins against its own device definition — not emergent from the whole connectivity graph, so per DR-006's classification logic it belongs in the type system, not DRC, and not left uncovered (the v1 status quo this whole redesign exists to fix). A general Option-style wrapper type was rejected as unnecessarily heavyweight compared to a second declaration form (nc) mirroring net's existing shape.

## Consequences

- Extends Pin (existing concept) with an obligation kind — no new core concept, no vocabulary collision.
- Adds one new top-level syntax form, nc, deliberately shaped like net so there's no new construct-family to learn — but semantically net's opposite, and must never be confused with a second connectivity mechanism.
- The exhaustiveness check runs once, at final design assembly after all fn inlining/monomorphization — never inside an unassembled fn body.
- Netlist emitters (KiCad/LCEDA) must represent nc-marked pins faithfully using each format's own "no connect" convention.

## Revisit when

If real std-library/fn authoring reveals cases where the required/optional split needs finer granularity than a binary kind — that would be a new RFC extending this mechanism, not a silent change to it.

# DR-013 (final): impl Trait for Device is a free-standing statement, matched by name against the device's own already-declared fields

## Context

RFC-003 (note 6) went through three drafts the same day, each correcting the previous based on Tony's direct feedback:

- Draft 1 embedded impl Trait inside the device declaration itself (device MLCC<...>: impl Capacitor { ... }) — structurally identical to C#'s class Foo : IBar, where a type's trait membership can only ever be declared at the type's own definition. Tony rejected this as insufficiently Rust-like.
- Draft 2 responded by adding default methods, associated types, and blanket/generic impls — real Rust mechanisms, but not the one Tony was actually pointing at, as his own example showed: impl TwoTerminal for MLCC {} as a bare, separate, top-level statement, with trait and struct declared completely independently of each other and of any impl. This was withdrawn as solving a different problem than the one raised.
- Draft 3 (final) decouples impl Trait for Device into its own free-standing statement, and — per Tony's further clarification — makes satisfaction resolve by matching names directly against the device's own already-declared pins/specs, so an impl body is empty in the common case (nothing to restate), with an explicit in-body mapping only when a device's own field names genuinely differ from the trait's required role names.

## Options

1. Embed impl Trait in the device declaration (draft 1, rejected — too rigid, C#-shaped).
2. Keep embedded impl, but add default methods / associated types / blanket impls on top (draft 2, withdrawn — solves a different problem than the one raised).
3. Decouple impl Trait for Device into its own free-standing top-level statement, satisfied by matching trait-required pin/spec names against the device's own already-declared pins/specs by name, with an in-body mapping only when names differ; empty body is the common case (draft 3, final).
4. Structural typing — no explicit impl at all, satisfaction inferred purely from shape match, no impl statement needed.
5. Require every impl body to always restate the full pin/spec mapping verbatim, even when names already match (a literal reading of "Rust impl bodies always contain something").

## Decision

Option 3. device declarations no longer have a trait clause — a device is just pins + specs, self-contained. trait declarations are unchanged (required pins/specs, sub-trait bounds). impl Trait for Device is its own statement, written wherever convenient (commonly grouped by trait, in a dedicated module) — never required to be co-located with either declaration. A device may have any number of separate impl blocks, one per trait, added at any time. Satisfaction is checked by matching the trait's required names against the device's own already-declared pin/spec names — when everything matches by name, the impl body is empty (impl Capacitor for MLCC {}); when a device's own names differ from the trait's required roles, the impl body contains an explicit role: actual mapping, reusing the same shape used elsewhere for pin/spec declarations. This check still runs exhaustively at the moment each impl statement is written — the gradeability discipline is unchanged from every prior draft, just relocated and clarified.

## Rationale

The actual flaw in draft 1 wasn't "not enough mechanisms" (draft 2's answer) — it was that trait membership was permanently welded to the type's own declaration, exactly the C#/Java interface shape. Decoupling impl restores the two properties Tony's example demonstrates: a type can gain trait implementations after its own definition is finished, and implementations can be organized by trait rather than forced onto every device's declaration. Option 4 (pure structural typing, no impl statement at all) was rejected for the same reason it was rejected in every prior draft: an explicit, reviewable impl statement is worth keeping — a device should never be considered to satisfy a trait just because its shape happens to match, without someone (human or AI) explicitly asserting the relationship. Option 5 was rejected per Tony's specific follow-up: CoHDL traits have no methods to implement (unlike Rust), so forcing every impl body to restate information the compiler can already read directly off the device's own declaration is pure redundant busywork with no correctness benefit — the mirror image of the "magic defaults" smell the Constitution already forbids, just in the other direction ("magic restatement" instead of "magic omission").

## Consequences

- This is a smaller conceptual change than draft 2 (Concepts: Low, vs. draft 2's High) — no new mechanisms, no new core concept, just a relocated syntax form for an existing relationship (Trait ↔ Device satisfaction), now resolved by direct name-matching against already-declared fields.
- Trust is High (draft 2 had dropped it to Med due to hidden default-method/blanket-impl behavior) — with no default methods or blanket impls, and with by-name matching meaning an empty impl body still has a fully-resolved, inspectable meaning (nothing hidden — the compiler can always show exactly which device field satisfied which trait requirement), a device's full trait surface remains 100% auditable.
- A missing sub-trait-bound impl (e.g. impl Capacitor for MLCC without a sibling impl TwoTerminal for MLCC anywhere in scope) must be a compile error at the point impl Capacitor for MLCC is written, naming the specific missing sibling.
- When by-name matching fails and no mapping is given, the diagnostic must suggest the exact fix (add a role: actual mapping) rather than a generic "unsatisfied trait" error.
- LSP tooling must provide "find all impls for this device" / "find all impls of this trait" navigation, and must show the resolved by-name matches on hover for any empty impl block, since that information exists only in the compiler's resolution once the body is empty.
- RFC-007 (generics-over-specs) should assume impl Trait for Device is free-standing and name-matched when it specifies generic-bound interactions with trait satisfaction.

## Revisit when

If real std-library authoring reveals a genuine need for shared default behavior across many devices, abstracting a trait over "some unit type," or implementing a trait for a whole family of devices at once (draft 2's three withdrawn mechanisms) — these remain legitimate future RFCs on their own merits, just not bundled into this one. Also revisit if by-name matching proves too permissive in practice (coincidental name matches causing incorrect satisfaction) — see RFC-003's Failure modes for the specific risk and current mitigation (naming discipline + LSP hover visibility, not a compiler-enforced guarantee).

(Numbering note: DR-013 is the next open decision-record number after DR-012; it is unrelated to RFC-013 "Layout-constraint concept" in note 6's backlog — DR-NNNN and RFC-NNNN are separate tracks that happen to share a number here, same as the earlier DR-011/RFC-011 non-collision.)

# DR-014: DRC/type-system reclassification — all nine v1 rules audited against real source, not memory

## Context

DR-006 established the principle (structural checks move to the type system; emergent/numeric checks stay DRC) and pre-classified two illustrative examples (E003–E005 → type system; W003–W004 → residual DRC) from a paraphrase of v1's rules. RFC-004 did the actual audit against the real v1 source (crates/cohdl-drc/src/rules.rs in the still-cloned v1 repo), covering all nine rules (E001, E002, E003, E004, E005, W001, W002, W003, W004) rather than two.

## Options

1. Treat DR-006's two-rule illustration as sufficient and leave the remaining seven unclassified until they come up individually.
2. Audit all nine rules exhaustively against real v1 source now, classify each into "type system," "stays residual DRC," or "reconciled/retired by an already-accepted RFC," and surface any open dependencies explicitly (RFC-004's proposal).
3. Re-implement all nine rules as DRC verbatim, deferring the whole reclassification question.

## Decision

Option 2. Full classification: E003, E005 → type system, fully superseded by RFC-003's impl-time trait satisfaction (both were checking near-identical things at two different, too-late points in v1). E004 → type system, but only partially closed — it's a generic-parameter trait-bound check, which needs RFC-007 (generics-over-specs) to fully specify; flagged as an explicit open dependency, not silently assumed solved. W002 → type system — a net naming zero instance pins is checkable from that one declaration alone, narrower than DR-006's original two-rule illustration anticipated. W001 → reconciled/retired — RFC-002's obligation-aware exhaustiveness check is a strictly more correct replacement (v1's blanket "any unconnected pin" warning would have wrongly flagged intentionally-optional pins RFC-002 permits to stay unmentioned). E001, E002, W003, W004 → stay residual DRC, all genuinely emergent from the whole connectivity graph.

## Rationale

Options 1 and 3 both risk the exact failure DR-006 was written to prevent: a check silently not migrating to either bucket and quietly disappearing. Auditing against real source (rather than the illustrative paraphrase DR-006 used) surfaced two things DR-006's two-rule example didn't anticipate: W002 is structural enough to become a type check (not just the two DR-006 named), and E004 is a genuinely different mechanism (generic-bound checking) from E003/E005 (device-level impl checking) that RFC-003 alone doesn't fully close — an important distinction lost if the classification stayed at DR-006's illustrative level.

## Consequences

- v2's residual DRC engine has exactly four rules (E001/E002/W003/W004-equivalents), not the nine v1 had or the "unspecified remainder" DR-006 left open.
- RFC-011 (error-code registry) can now proceed using this classification as its primary input.
- RFC-007 (generics-over-specs) must explicitly close the E004 generic-bound-checking gap as part of its own scope — this is now a named, tracked dependency, not an assumption.
- Implementers must not re-add W001 as a literal port of v1's blanket check — doing so would directly contradict RFC-002's deliberate design (optional pins may be silently unmentioned).

## Revisit when

If std-library growth surfaces a genuinely new structural or emergent check not among these nine, it goes through its own type-system-first test per note 6's RFC template — this classification is closed for the nine v1-era checks, not a permanently frozen list for all future checks.

# DR-008: Collision-free designator allocation — a pure function, not a stateful loop

## Context

RFC-005 (note 6) replaced v1's designator allocator, confirmed by reading the actual v1 source (crates/cohdl-sema/src/designator.rs): a stateful loop that mutates a shared used: HashSet while iterating instances, calling a "scan up from 1 until free" search per instance. This shape of algorithm produced the observed esd/ldo33 both-get-U3 collision in the conol-pin fixture — both devices fall back to the default U prefix, and the incremental bookkeeping did not correctly separate their two independent searches.

## Options

1. Patch v1's algorithm to fix the specific observed collision, keep the stateful-loop shape.
2. Replace the incremental "mutate a shared used-set, scan per instance" loop with a pure function: compute the full reserved-number set per prefix once, immutably, from prior assignments + tombstones + overrides, then assign fresh numbers as positions in one sorted sequence — with an explicit, checked injectivity postcondition on every run (RFC-005's proposal).
3. Keep an incrementing global counter per prefix, reset per compilation.
4. Randomized/hash-based designator assignment.

## Decision

Option 2. The allocator becomes: partition live instances into (existing assignment / explicit override / needs-fresh-assignment) — a total, non-overlapping partition; resolve overrides and fold them into a per-prefix reserved-number set together with prior assignments and tombstones, all before any fresh assignment happens; assign fresh numbers to prefix-grouped instances (sorted by hierarchical path) as the sorted sequence of positive integers missing from that prefix's reserved set — the Nth instance needing prefix P gets the Nth missing integer, not an independent search. Assert the whole result is injective as an explicit runtime postcondition on every compilation, not just a unit test.

## Rationale

Option 1 (patch the specific bug) was rejected as fixing a symptom of the wrong algorithm shape rather than the shape itself — a future edge case in the same stateful-mutation pattern could reproduce an equivalent bug with no structural reason to trust otherwise. Option 3 (global counter) still depends on strict sequential processing order, buying nothing over the reserved-set-then-positional approach. Option 4 (randomized/hashed) was rejected: designators should read as a small, dense, meaningful sequence (C1, C2, C3) for human reviewability, and hash-based assignment risks the "same source → same output bytes" reproducibility hard constraint if the hash isn't perfectly version-stable. Option 2 makes injectivity a property of the construction (two instances needing the same prefix read off different fixed positions in one pre-computed sequence — there is no window where one's assignment isn't yet visible to the other) rather than something that happens to hold if the loop's bookkeeping never slips.

## Consequences

- No change to designator format, design.lock's file format, or any author-facing concept — this is purely a compiler-internals correctness fix.
- The injectivity postcondition is checked on every real compilation, not just in a test suite — a compiler-internal error if it ever fails, never a silently-shipped bad netlist.
- Tombstones and overrides must be fully folded into the reserved-number set before fresh assignment begins, for every prefix independently — implementers must not reintroduce an ordering dependency here.
- Different prefixes must have independently-computed reserved sets — designators are only unique as prefix+number, so C3 and U3 coexisting is correct, not a bug.
- Property/golden-file tests must directly exercise the "two new instances, same prefix, arbitrary collection order" case and confirm order-independence, not just absence-of-collision on one fixture.

## Revisit when

If cross-run parallel/concurrent compilation of the same design becomes a real requirement (explicitly out of scope for this RFC) — that would need its own design pass on top of this one, not a silent extension.

# DR-015: Nested fn calls — recurse into the existing expansion procedure, detect cycles explicitly

## Context

RFC-006 (note 6) fixed v1's confirmed bug (crates/cohdl-sema/src/typeck.rs:1370-1372): the design-body expansion loop correctly expands a top-level fn call (instantiating its insts, wiring its nets), but a Call statement found inside a called fn's own body — i.e. a nested call — matched a no-op arm: FnBodyStmtKind::Call(_) => { /* Nested function calls not yet supported. */ }. No instances, no nets, no diagnostic — a silently incomplete design that compiled clean.

## Options

1. Make the compiler reject nested calls outright with an error, rather than fixing them.
2. Recursively generalize the existing expansion procedure so a nested Call triggers the same instantiate-and-wire expansion as a top-level call, with generic-substitution context threaded outward-in and instance/net naming derived from the full call-chain path (not a flat counter); detect and reject cyclic call chains explicitly, at the point a cycle would be entered (RFC-006's proposal).
3. Pre-flatten nested calls into equivalent top-level calls at an earlier lowering pass.
4. Allow cyclic recursion up to a fixed expansion depth cap.

## Decision

Option 2. The expansion procedure becomes properly recursive: encountering a Call inside a fn body invokes the same procedure that already handles top-level calls, carrying forward (a) the accumulating generic-substitution map, resolved outward-in so a nested call's generic arguments are concrete by the time they're needed, and (b) a call-chain-path naming scheme so every produced instance/net has a name derived from its full nesting path, guaranteeing uniqueness across call sites without a flat, order-dependent counter. A call chain that would re-enter a fn definition already active on the same chain is a compile error naming the full cycle, checked at the moment the cycle would be entered — before any partial expansion happens.

## Rationale

Option 1 was rejected: note 2 already stated nested calls as "first-class, required... not an edge case to patch later," and the fix isn't actually hard — the existing top-level expansion procedure already does everything needed; it's a straightforward recursive generalization, not a new mechanism, so rejecting the feature outright would be giving up on an already-designed capability over a missing recursive case. Option 3 (pre-flatten) was rejected as needing the identical substitution/naming logic anyway, just relocated to a less-integrated earlier pass, for no benefit. Option 4 (depth-capped cyclic recursion) was rejected outright: it reproduces the "compiles but silently wrong" failure mode this RFC exists to eliminate, merely with a cap instead of zero levels — genuine cycles have no sensible expansion (they wouldn't terminate) and must be a compile error, not a silently-truncated one.

## Consequences

- No new concept, syntax, or grammar — this closes a gap between the Conceptual Model's already-stated intent and what the discarded v1 implementation actually did.
- Instance/net naming for fn-expansion output is now derived from the full call-chain path, extending v1's existing __fn{N}{name}{inst} convention one level per nesting depth — this must still feed correctly into RFC-005's designator allocator (collision-free by construction regardless of how many instances a deep call chain produces).
- Cyclic-recursion diagnostics must show the full call chain, not just the point of detection — the same precision discipline RFC-003 established for sub-trait-chain failures.
- Fixture/golden-file tests must directly exercise 2-level and 3+-level nesting, generic substitution threading through multiple levels, cyclic detection, and same-depth sibling call sites of the same fn (confirming no naming collision).

## Revisit when

If RFC-007 (generics-over-specs) reveals the substitution-threading mechanism here needs to change shape to support a richer generic system — that would be an extension coordinated with RFC-007, not a silent divergence.

# DR-016: Generics-over-specs — unify trait-bound checking into one mechanism, wired to free-standing impls

## Context

RFC-007 (note 6) formalized generic parameters (unit-type and trait-bound) and closed RFC-004's flagged E004 gap. Reading v1's real source revealed the gap was worse than "unspecified": v1 had two separate mechanisms for trait-bound checking — a generic type parameter path (type_implements_trait, typeck.rs:1465) and a distinct value-parameter impl Trait path (check_call, typeck.rs:1421-1461) — and both read dev.impl_traits, a device's own embedded trait list that RFC-003 already removed in favor of free-standing impl Trait for Device statements. So the mechanism itself was stale, not just undocumented.

## Options

1. Leave the generic-parameter/trait-bound system informally specified, relying on examples already used across RFC-001/002/003/006.
2. Formalize two generic parameter kinds (unit-type, trait-bound), unify v1's two separate trait-bound-checking mechanisms (generic type parameters and impl Trait-typed value parameters) into one by treating the latter as sugar for an anonymous generic type parameter, and rewire that one mechanism to check against RFC-003's free-standing impl statements instead of the removed dev.impl_traits field (RFC-007's proposal).
3. Keep v1's two mechanisms as genuinely distinct features.
4. Check trait bounds against a device's declared traits list, resurrecting impl_traits in some form.

## Decision

Option 2. A generic parameter is either unit-type-bound (one of RFC-001's ten types, optional visible default) or trait-bound (one or more traits via +). param: impl Trait value-parameter syntax desugars to an anonymous trait-bound generic type parameter — one mechanism, not two. Trait-bound checking at generic instantiation looks up free-standing impl RequiredTrait for ConcreteType statements in scope (RFC-003's mechanism), never a device-level trait list.

## Rationale

Option 1 was rejected: the gap was actively dangerous, not just undocumented — the underlying mechanism read a field RFC-003 had already removed, so any implementation following the "informal" examples literally would have silently failed or needed to resurrect dead code. Option 3 was rejected per note 4's "prefer extending an existing concept over inventing a parallel mechanism" — two independently-maintained trait-bound-checking code paths for the same underlying question is exactly the redundancy risk that principle exists to prevent, and it's also a real AI-generatability risk (a model could plausibly expect the two syntaxes to behave subtly differently). Option 4 was rejected outright: resurrecting impl_traits would directly contradict RFC-003's decision to decouple trait implementation from device declaration.

## Consequences

- Exactly one trait-bound-checking code path exists going forward, reused by both generic type parameters and impl Trait-typed value parameters (via desugaring).
- This is the mechanism that finally closes RFC-004's E004 gap — trait-bound-at-instantiation failures are now checked at compile time, at the call/instantiation site, with the same diagnostic precision discipline RFC-003 established for device-declaration-level checking.
- RFC-011 (error-code registry) is now unblocked — RFC-004's classification is fully closed, no remaining "partially specified" dependency.
- Note 10's "Generics-over-specs (partially specified)" section is replaced with a fully-specified one.

## Revisit when

If a genuine need for const-generics, value-parameter generics, or richer where-clause-style bounds emerges from real std-library authoring — these were explicitly scoped out of this RFC and would need their own proposal, not a silent extension.

# DR-017: Exhaustive pattern-matching over structural variants — package variants + retrofitted pin roles

## Context

RFC-008 (note 6) formalized two related gaps surfaced by the actual MVP implementation (verified on the real main branch, 65 passing tests, self-audited via docs/compliance-report.md): (1) provisional-syntax.md's pin-role annotation already existed as a closed six-value set, but unannotated pins silently defaulted to passive — an implicit default the redesign's own principles forbid; (2) package/footprint variants (pins[VARIANT], spec[VARIANT]) were explicitly named in the same document as needed but deliberately left unspecified, pending this RFC.

## Options

1. Add package variants as a new mechanism; leave the existing pin-role default as-is (inconsistent application of exhaustiveness).
2. Formalize package variants as a closed variants {} set requiring a pins[VARIANT] block per variant (exhaustive at the device declaration), and retrofit pin roles to require an explicit annotation on every pin, retiring the implicit passive default (RFC-008's proposal).
3. A general pattern-matching/expression language covering arbitrary destructuring.
4. Model package variants as entirely separate device types instead of one device with variants.

## Decision

Option 2. variants { ... } declares a device's closed, finite package/footprint set; every declared variant requires a pins[VARIANT] block (exhaustiveness checked at the device's own declaration); spec[VARIANT] is optional per variant. An instance of a device with declared variants must select one via [VARIANT] at the instantiation site — no implicit default variant. Separately, every pin declaration now requires an explicit role annotation from the closed six-value set (input/output/bidirectional/passive/power_in/power_out) — the previous "unannotated → passive" convention is retired.

## Rationale

Option 1 was rejected as an inconsistent half-measure — if exhaustiveness is worth adding for a new mechanism (package variants), it's worth applying to the closed set that already existed with a silent default (pin roles), especially since that default was itself flagged by this exact RFC's own motivating problem statement. Option 3 (general pattern matching) was rejected per the established narrow-generics precedent (RFC-007 rejected const-generics/richer bounds for the same reason: no concrete need beyond closed structural variants, and a general expression language is conceptual cost disproportionate to the job). Option 4 (separate device types per variant) was rejected: it duplicates every trait impl per variant, a real composability regression, and doesn't match the Conceptual Model's original "one device, several shapes" framing.

## Consequences

- Extends Device and Pin (existing concepts) — no new core concept.
- Real, one-time compatibility break: every existing device declaration in the MVP-scope std library and demo board needs an explicit pin-role annotation added — mechanical and compiler-flagged, not a silent behavior change (every currently-unannotated pin becomes explicitly [passive], preserving current DRC behavior exactly).
- Package variants are additive — no existing device without variants {} is affected.
- This RFC's migration (adding roles everywhere) ships in the same implementation pass as the RFC itself, per the project's "ship with its check" discipline — no interim state where the std library fails to compile against its own spec.
- Reserves three new error-code sub-blocks: missing pin-role annotation, undeclared variant selected at instantiation, missing pins[VARIANT] block for a declared variant.

## Revisit when

If a genuine third use case for closed-variant exhaustiveness emerges (beyond pin roles and package variants) — e.g. differential-pair roles or AVL alternates needing the same discipline — extend this same mechanism rather than inventing a parallel one.

# DR-018: cohdl fmt canonical form — a pure AST-to-text serializer, grounded in the real repository's existing style + real drift

## Context

RFC-009 (note 6) defined cohdl fmt's canonical form, informed by reading the actual std library and demo example on the real main branch rather than designing in the abstract. This surfaced two concrete, pre-existing gaps: std/passives.cohdl was never updated for RFC-008's mandatory pin-role annotations, and std/connectors.cohdl's SBU1/SBU2 pins are missing their role brackets entirely — both real RFC-008 compliance bugs, found while grounding this RFC, not invented by it.

## Options

1. A configurable formatter (indent width, line length, etc. as options).
2. A pure AST-to-text serializer with one canonical form, no configuration — parse the existing grammar, re-serialize deterministically; idempotent and semantically inert by construction, checked by dedicated regression tests (RFC-009's proposal).
3. A regex/text-munging pass over the existing source instead of an AST-based serializer.
4. Defer fmt entirely until a later maturity stage, since the MVP shipped without it.

## Decision

Option 2. One canonical textual form for every construct across RFC-001–008, codified from the styling conventions already universal in the real repository (4-space indent, one statement per block line, comma-space separation, trailing comments preserved verbatim). cohdl fmt parses to the existing AST and re-serializes — never text-level munging — which is what makes idempotence and semantic inertness properties of the construction, not just testing goals. fmt does not fix missing required syntax (e.g. a missing pin-role bracket stays a parse error); formatting and repair are different jobs.

## Rationale

Option 1 was rejected per the "one canonical way" principle applied to layout, the same as it's applied to syntax — configurability would just relocate the "two ways to express the same thing" smell from the grammar to the formatter. Option 3 (text-munging) was rejected because it can't cleanly guarantee idempotence/semantic-inertness the way an AST-based serializer can by construction — a regex pass risks subtly different behavior on inputs it wasn't tested against, while an AST re-serialization is total by definition (anything that parses has exactly one canonical output). Option 4 (defer) was rejected: the real repository already shows concrete drift (the passives/connectors gaps above) that a formatter would have caught immediately, and the Constitution already classifies a canonical form as a generatability constraint, not optional polish — deferring it further just lets more drift accumulate.

## Consequences

- No new concept, syntax, or grammar — pure tooling.
- Real, one-time diff when first run on the existing repository: std/passives.cohdl needs role brackets added; std/connectors.cohdl's SBU1/SBU2 gap must be fixed by hand first (fmt requires already-parsing source).
- cohdl fmt --check (non-mutating, CI-friendly) ships alongside the mutating cohdl fmt.
- The repair-loop harness should run generated source through fmt before diffing attempts, directly serving the AI-generatability goal this RFC targets.
- Idempotence and semantic-inertness are the two mechanically-checkable correctness properties for fmt itself, tested against every existing fixture plus the std library and demo example.

## Revisit when

If a genuine need for configurable style emerges from real multi-team/multi-organization usage (unlikely at this stage, and would need strong justification against the "one canonical way" principle) — that's a future RFC, not a silent addition of flags.

# DR-019: cohdl check --json schema — a direct, versioned re-projection of the existing Diagnostic struct

## Context

RFC-010 (note 6) defined --json's schema, grounded in the real Diagnostic/Span/SourceMap types (src/diag.rs, src/span.rs) already shipped — code, severity, message, a primary Label (span + message), secondary labels, and help lines. main.rs's own header comment confirms the MVP explicitly cut this ("no --json"), leaving the repair-loop harness dependent on scraping the same text a human reads.

## Options

1. A machine-parseable prefix line format (e.g. file:line:col: code: message) layered onto the existing text renderer, instead of full JSON.
2. A versioned JSON schema that directly re-projects the existing Diagnostic struct's fields with zero invented content, plus a top-level verdict computed identically to the CLI's existing exit-code logic (RFC-010's proposal).
3. A full LSP server.
4. Diagnostics nested per pipeline stage (parse/typecheck/DRC) rather than one flat list.

## Decision

Option 2. cohdl check --json / cohdl build --json emit exactly one JSON document to stdout: schema_version (int, bumped only on breaking schema shape changes), verdict ("pass"/"fail", matching diagnostics.has_errors()), and a flat diagnostics array whose entries map 1:1 onto the real Diagnostic struct's fields (code, severity, message, primary, secondary, help), with spans resolved to 1-based file/line/col via the existing SourceMap.

## Rationale

Option 1 was rejected: still text-format-coupled, and loses structure (secondary labels, multi-line help) JSON represents naturally. Option 3 (LSP) was rejected per note 9's explicit MVP cut ("full LSP") — a stateful protocol server is much more than the repair-loop harness needs to stop text-scraping. Option 4 (per-stage nesting) was rejected because the existing Diagnostics collector already flattens across pipeline stages — diagnostics don't self-report which stage produced them — so inventing that categorization for JSON alone would mean the JSON schema carries information the plain-text renderer doesn't have either, violating this RFC's own gradeability principle (JSON output must be provably equivalent to plain-text output, not a superset or divergent view).

## Consequences

- No new concept, no new diagnostic content — purely a structured re-projection of existing pipeline output.
- Mandatory equivalence test: for every fixture, --json output and plain-text output must report the identical set of diagnostics (code/severity/span/message/help), field-for-field — any divergence is a bug in one of the two renderers, not an accepted difference.
- schema_version must be checked by any consumer before parsing further — a versioning discipline that protects RFC-011 (error-code registry) from needing to touch this schema's shape when it formalizes code's guarantees.
- The repair-loop harness migrates off text-scraping onto --json in the same implementation pass, per the project's "ship with its consumer" precedent (RFC-008/009).

## Revisit when

If a genuine need for incremental/query-style diagnostic access (ask about one span, one file) emerges from real LSP work — that's the eventual LSP RFC's job, not a silent extension of this single request/response schema.

# DR-009: Error-code registry v2 baseline — formalize stability, close three real gaps found against real source

## Context

RFC-011 (note 6) formalized the informal registry (docs/error-codes.md), which had itself already flagged two "wrinkles" in its own text (unit-mismatch checks at generic sites reporting as E402/E404 instead of an E1xx code; a standalone Ω reporting under E001 instead of E101). Auditing the registry against the real compiler source and against RFC-008/RFC-010 (both landed after the informal registry was written) surfaced a third, more serious gap: RFC-008's three promised structural-variant diagnostics were never wired to any code at all — no call site exists in source for any of them.

## Options

1. Treat "formalize" as "freeze the informal registry as-is," documenting the two known wrinkles as accepted quirks rather than fixing them.
2. Formalize the registry's stability rule (issue once, never repurpose, only deprecate) as an enforced, mechanically-checked contract; fix the two flagged misfilings (E402→E112, E404→E113; standalone Ω gets its own E107); and close the RFC-008 gap with a real E9xx block (five codes, not the RFC's originally-estimated three, since deriving them properly split "missing selector" into two distinct cases) (RFC-011's proposal).
3. Leave RFC-008's codes unimplemented indefinitely, since MVP-scope demo fixtures don't currently exercise them.

## Decision

Option 2. The registry keeps its existing block-per-mechanism structure but the organizing principle is now explicit: a block is chosen by kind of mistake, not by which compiler pass happens to catch it — this is what E402/E404's misfiling violated. Two real renumberings (E402→E112, E404→E113) and one real split (Ω gets E107, distinct from E101). A new E9xx block (five codes: E901–E905) closes the RFC-008 gap with real call sites, not just registry rows. A new E00x entry (E000) documents the pre-pipeline CLI-invocation failure path RFC-010 left implicit. A mechanical, CI-enforced completeness test runs in both directions: every code in source has a registry row; every non-deprecated registry row has a real call site.

## Rationale

Option 1 was rejected: the informal registry's own text explicitly anticipated RFC-011 fixing these wrinkles ("clean up when RFC-011 formalizes codes") — freezing them as accepted quirks would mean this RFC skipped the one job it was flagged to do. Option 3 was rejected: RFC-008 is already Accepted and its own text promised these diagnostics; leaving them permanently unwired reproduces exactly the "structurally present but not actually enforced" failure class DR-006 named for DRC rules, now in the diagnostics-registry domain instead. The two-directional completeness test is what prevents this exact gap from recurring silently for any future RFC.

## Consequences

- Real, one-time breaking renumbering: E402→E112, E404→E113. Acceptable pre-first-release (no external consumer depends on these two codes yet); this is precisely the kind of change the stability rule exists to prevent from happening again post-launch.
- RFC-008's real diagnostic count is five (E901–E905), not the three originally estimated in RFC-008's own text — a correction surfaced by deriving the exhaustiveness rules properly rather than re-using an unverified estimate.
- The completeness test becomes a permanent CI gate, not a one-time audit — every future diagnostic-producing code change is checked against the registry automatically.
- Note 10 gains a summary "Error-code registry" section (block-ownership table + stability rule); the full code-by-code listing stays solely in docs/error-codes.md to avoid two sources of truth.

## Revisit when

If a future RFC's diagnostic needs don't fit any existing block's "kind of mistake" grouping — that RFC should propose its own new block (as RFC-008 did for E9xx here), not force-fit into an existing one for the sake of avoiding a new block number.

# Pending decision records (to be written as RFCs land)

- DR-010 — #[intent(...)] as pure non-netlist metadata, once RFC-012 is decided (carried forward from v1's pending DR-007, renumbered).
