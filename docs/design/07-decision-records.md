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

# DR-010: #[intent(...)] annotations — a single opaque string, structurally enforced zero compiler impact

## Context

RFC-012 (note 6) designed the last backlog item before the gated RFC-013: a way to attach structured, human-readable design rationale to a declaration, distinct from an ordinary // comment, while guaranteeing it can never influence compilation. No real v1 implementation of this existed to ground against (checked origin/legacy — zero real intent code, only the license text's unrelated use of the word) — this was always a design-note-level proposal, never built, so this RFC designs it fresh against the same P2/gated-on-zero-netlist-impact framing v1's backlog stated.

## Options

1. Structured sub-fields (reason, ticket, author, etc.) inside the attribute.
2. A single opaque string argument, attachable to any top-level or body statement, with zero-impact enforced structurally (never threaded into any checking/emission function's input types) and tested via a mandatory non-impact regression (RFC-012's proposal).
3. A dedicated top-level intent {} block per design, decoupled from the specific declaration it explains.
4. Repurpose doc-comment syntax (/// ...) with implicit tooling conventions distinguishing it from prose.

## Decision

Option 2. #[intent("...")] — exactly one string literal, attachable to inst/net/nc/impl/device/trait/fn/part declarations, at most one per declaration. Parsed into the AST as an opaque, uninterpreted string field on its target node; the type checker, residual DRC, designator allocator, and netlist/BOM emitters never read this field — it is not a parameter any of those passes' functions accept, making the zero-impact guarantee true by construction, not convention.

## Rationale

Option 1 was rejected: structured sub-fields invite exactly the "why not make this field checkable too" slope this RFC exists to avoid — a single string keeps the temptation to encode real constraints as far from the mechanism as possible. Option 3 was rejected: decoupling rationale from its specific instance/net reproduces the "scattered prose, not locally attributable" problem // comments already have — attribution to one specific declaration is the entire point. Option 4 was rejected per the redesign's own established discipline (RFC-008 retired an implicit pin-role default for the same reason): making "is this comment structured metadata or prose" a matter of tooling convention rather than explicit syntax is exactly the ambiguity the project avoids everywhere else.

## Consequences

- No new core concept — same shape as the already-accepted #[designator("Xxx")] override attribute (RFC-005).
- Mandatory non-impact regression test: mutating any #[intent(...)] string (including strings that read like constraints) must never change a fixture's verdict, diagnostics, designator assignment, or emitted netlist/BOM bytes, byte-for-byte.
- cohdl fmt treats it as a single-line attribute preceding its target, reusing the existing attribute-placement convention — no new formatting category.
- cohdl check --json does not surface #[intent(...)] content — an explicit non-inclusion, not an oversight, until some future tooling RFC deliberately adds it.

## Revisit when

If real authoring reveals a genuine need for structured sub-fields (e.g. linking rationale to an external ticket system) — that would be its own RFC extending this mechanism, evaluated against the same "does this invite checkable-constraint creep" question this RFC asked and answered "no" to for the MVP-baseline single-string form.

# DR-011: Layout constraints — a closed, four-kind declarative vocabulary, zero schematic-correctness impact

## Context

RFC-013 (note 6) opened the layout door per GC-002's amendment (note 8) — Tony's explicit decision to admit layout constraints into the conceptual model now, ahead of GC-002's originally-stated "concrete partner requirement" trigger. Note 2's Conceptual Model had already pre-designed the seam for this moment: "a net_class/constraint decoration adjacent to Net/Rule, never a second connectivity mechanism, inspectable and gradeable like a rule, losslessly ignorable/passable through codegen." This RFC is the first exercise of that seam.

## Options

1. Fold layout constraints into rule (residual DRC), or wait for real in-language rule syntax before adding this at all.
2. A new layout { ... } top-level block with four closed constraint kinds (net_class, diff_pair, length_match, #[placement_hint(...)]), type-checked against their own closed vocabulary, threaded into a separate output artifact never read by the type checker/DRC/designator allocator (RFC-013's proposal).
3. Opaque-string-only constraints (extend #[intent(...)]'s pattern, no structured kinds).
4. A fully general, partner-schema-defined extensible constraint plugin system.

## Decision

Option 2. Layout Constraint becomes a new core concept, positioned adjacent to Net/Rule per the pre-designed seam — not a new connectivity mechanism, not a DRC rule. Four closed kinds cover the most common real PCB layout needs (impedance-class grouping, differential pairs, length matching, placement hints). Constraint data is emitted as a separate artifact (layout.json/netlist addendum), never merged into .net/BOM connectivity data, never read by any existing checking/emission pass — the same "not a parameter any pass accepts" structural guarantee RFC-012 established for #[intent(...)], now extended to a small, independently-type-checked vocabulary.

## Rationale

Option 1 was rejected: layout constraints are neither emergent-across-the-connectivity-graph (DR-006's DRC-classification criterion) nor structural-about-one-device (the type-system criterion) — they're declarative metadata about an orthogonal physical domain, and forcing them into rule's shape would misclassify them the same way RFC-011's E402/E404 misfiling misclassified unit-mismatch-at-generic-sites. Option 3 was rejected: unlike design rationale, these four kinds have real checkable structure (arity, net existence, declaration-before-use) worth type-checking — reducing them to prose would discard gradeability the redesign's whole thesis says is worth having wherever real structure exists. Option 4 was rejected as premature: with no concrete partner integration in hand (GC-002's honestly-disclosed gap), building general extensibility now is speculative generality with no real use case to validate it against.

## Consequences

- First genuinely new core concept added since the ground-up redesign began — the canonical vocabulary grows by one (Layout Constraint), a permanent addition whose cost is accepted per GC-002's explicit decision.
- Reserves a new error-code block, E10xx (five codes: E1001–E1005), following RFC-011's kind-of-mistake organizing principle.
- cohdl build gains a new, versioned output artifact alongside the existing .net/BOM.
- The constraint-kind list is explicitly provisional — per GC-002's disclosed design debt, expected to be revisited once a real partner layout-tool integration is scoped; a future reshaping of these four kinds is anticipated, not a stability violation.
- A mandatory zero-impact regression test (mirroring RFC-012's) must show that any layout {} content never changes the schematic-correctness verdict, any RFC-001–011 diagnostic, designator assignment, or .net/BOM bytes.

## Revisit when

The moment a real partner layout-tool integration is scoped — at that point, validate (and likely reshape) the four constraint kinds against the partner's actual schema needs, via a follow-up RFC extending this one, not a silent change to already-Accepted syntax.

(Numbering note: DR-011 is the next open decision-record number in the DR track; it is unrelated to RFC-011 "Error-code registry," which used its own number in the RFC track — the same non-collision pattern already noted for DR-013/RFC-013 earlier in this backlog.)

# DR-020: LSP support — a thin protocol server wrapping the existing pipeline, one scoped dependency exception

## Context

RFC-014 (note 6) closed a gap two of the project's own earlier Decision Records had already named as needed but never built: RFC-003's DR-013 explicitly flagged "LSP tooling must provide 'find all impls for this device' / 'find all impls of this trait' navigation, and must show the resolved by-name matches on hover for any empty impl block" — work deferred, not forgotten, at the time. With the MVP complete (RFC-001–013 implemented, 131 tests passing, real --json/fmt in place), this RFC picks that up as real Layer-4 tooling.

## Options

1. Hand-roll the entire LSP protocol (JSON-RPC transport and all LSP message data types) to preserve the project's zero-external-dependencies property exactly.
2. A thin JSON-RPC/stdio server (cohdl lsp) wrapping the existing pipeline::check()/json modules unchanged — diagnostics via publishDiagnostics (direct re-projection of the already-existing JsonDiag shape), hover for resolved impl mappings and pin obligations, goto-def, and "find all impls" references — depending on the lsp-types crate (pure data types, no I/O) for the protocol's message shapes while keeping the transport loop hand-rolled (RFC-014's proposal).
3. Ship diagnostics-only, defer hover/goto-def/references to a later RFC.
4. Build incremental compilation first, then the LSP, so re-checks are fast from day one.

## Decision

Option 2. cohdl lsp is a new subcommand, a thin server that calls the exact same pipeline::check() the CLI already uses — zero new diagnostic logic. Diagnostics, hover (on empty impl blocks and pins), goto-def, and trait/device reference-finding are all in scope, since all four were either RFC-010's existing output or explicitly pre-named as needed by DR-013. The lsp-types crate is accepted as a scoped, justified dependency exception — the LSP spec's own type surface is large and externally versioned, unlike CoHDL's own small, fixed output formats the project has good reason to hand-roll precisely.

## Rationale

Option 1 was rejected: hand-rolling an external, independently-evolving protocol's entire type surface has real ongoing maintenance cost with no corresponding coherence benefit — unlike the project's own JSON/netlist formats (small, fixed, worth controlling precisely), the LSP spec isn't CoHDL's to design. Option 3 was rejected: DR-013 already flagged hover/references as needed, not speculative — deferring them again would mean a second RFC still not closing a gap the project identified as real. Option 4 was rejected as unnecessary sequencing: the existing pipeline already runs the full 131-test suite in ~0.04s, fast enough at current project scale; incremental compilation remains real future work (tracked in note 3) but isn't a blocking prerequisite.

## Consequences

- First scoped dependency exception in the project's history — Cargo.toml gains lsp-types; the JSON-RPC transport loop itself stays hand-rolled, consistent with existing style.
- Mandatory equivalence test: the LSP's diagnostics must exactly match cohdl check --json's output for the same file, field-for-field — the same discipline RFC-010 established, extended to a second consumer of the same underlying Checked data.
- Note 3's capability map LSP row updates from ⛔ (cut) to 📐 (designed, implementation pending).
- A minimal editor-client launch snippet ships alongside the server as a usage example — not a full marketplace extension, which stays separate scope.

## Revisit when

If incremental compilation (already tracked separately) becomes necessary because real project sizes make full-recheck-per-edit noticeably slow — that's its own RFC, coordinated with this one's re-check trigger points, not a silent latency workaround bolted onto the LSP server itself.

# DR-021: IPC-2581 codegen backend — an honestly-partial handoff, phase one of the Quilter integration

## Context

RFC-015 (note 6) picked up RFC-013's own disclosed gap — a layout-constraint vocabulary built without a concrete partner requirement — using real prior research already in the workspace: the "Quilter as a CoHDL Backend Partner — Fit Analysis" note (Business workspace), which investigated Quilter's actual multi-vendor contract and concluded IPC-2581, not a bare KiCad .net, is the right long-term handoff format — but also flagged that CoHDL's part declarations reference footprints by name only (no geometry) and that CoHDL has no board-outline/stackup concept at all. Confirmed directly against real source (src/ast.rs, std/passives.cohdl): footprint fields are indeed name-only strings; grepping the whole repo for stackup/board-outline concepts returned nothing.

## Options

1. Ship only the existing KiCad .net handoff, treat it as sufficient, skip IPC-2581.
2. A new cohdl-codegen-style native Rust IPC-2581 emitter (src/emit/ipc2581.rs), built from the existing DesignIr, carrying netlist + components + specs + RFC-013's layout constraints inline, with an explicit "logical-complete, physical-minimal" honesty marker for the parts CoHDL genuinely cannot yet provide (footprint geometry, board outline/stackup) — scoped explicitly as phase one of a multi-phase integration (RFC-015's proposal).
3. Build full footprint-geometry resolution and board-outline support in the same RFC, so the document is physically complete.
4. Wrap KiCad's own C++ IPC-2581 I/O rather than emit natively.

## Decision

Option 2. cohdl build gains a new IPC-2581 emitter alongside (never replacing) the existing .net/BOM/layout.json outputs, hand-rolled in the project's existing style (no XML library dependency, the same discipline as the hand-rolled JSON emitters). The document is deliberately, visibly partial — a document-level marker states "logical-complete, physical-minimal" rather than letting the output silently overclaim completeness it doesn't have.

## Rationale

Option 1 was rejected: the fit-analysis note's own investigation superseded its original "just use .net" sketch once it examined Quilter's real multi-vendor contract (Allegro/Xpedition only reachable via IPC-2581). Option 3 was rejected as scope creep — the fit-analysis note itself named footprint sourcing as "the real gap, not the format," a separate, larger engineering problem that would make this RFC undeliverable if bundled in. Option 4 was rejected: CoHDL's data lives natively as a DesignIr, not a KiCad BOARD object; wrapping KiCad's I/O would require a lossy synthetic-board round-trip for no benefit over emitting directly from the IR the project already has, and breaks from the established native-Rust-emitter pattern (cohdl-codegen-kicad/-lceda).

## Consequences

- First new output format since the ground-up redesign began — real new netlist-dimension surface area (Coherence Matrix: Netlist High), even though it's a pure projection of already-validated data.
- The "logical-complete, physical-minimal" honesty marker is the single most load-bearing design decision here — it's what keeps this RFC consistent with the Constitution's "no silent gaps" hard constraint, applied to a new artifact rather than a new check.
- Mandatory schema-validity test (against the real IPC-2581B1.xsd) and fidelity-equivalence test (this emitter's content must agree with the existing KiCad/BOM/layout.json emitters for the same fixture) — the same "two consumers of the same data must agree" discipline RFC-010/RFC-014 already established.
- Explicitly named, tracked future work, not silently assumed solved: footprint-geometry resolution, board-outline/stackup as a real CoHDL concept, and actual end-to-end validation against Quilter once real API/import access exists.
- Does not solve the ECO/re-routing mismatch the fit-analysis note flagged (Quilter re-routes from scratch, no incremental update) — named as a workflow-level limitation, not an emitter-format problem.

## Revisit when

Once real access to Quilter's API or import flow exists — validate the emitted document against it directly, and use that real feedback (not assumption) to decide whether footprint-geometry resolution or board-outline support should be the next phase, and in what shape.

# DR-022: Module system — file-tree-mirrors-module-tree, explicit use, enforced pub

## Context

RFC-016 (note 6) is CoHDL's first real module/namespace system. Confirmed against real source (project.rs, resolve.rs): CoHDL has always had exactly one flat global name bucket, across every file including std — provisional-syntax.md already documented this as provisional, and RFC-008 explicitly deferred designing the real mechanism. This RFC was triggered by a concrete need: Tony's proposed centralized library registry (RFC-017) requires naming and resolving cross-package references, which a flat namespace cannot express safely.

## Options

1. No module system — grow the library registry on top of the flat namespace, disambiguating collisions ad hoc (e.g. auto-prefixing).
2. File-tree-mirrors-module-tree (no separate mod declaration), explicit one-name-per-use imports, pub enforced only across package boundaries (intra-package visibility stays fully open) — chosen per Tony's explicit decision to design the full general module system (project-internal + cross-library) in one RFC (RFC-016's proposal).
3. Explicit Rust-style mod foo; declarations instead of implicit file-tree mirroring.
4. Scope narrowly to only what the library registry needs (name → library + exported item), deferring general in-project module/use/visibility semantics.

## Decision

Option 2, per Tony's explicit choice to design the full general module system now rather than a registry-scoped narrow version. A package's module tree mirrors its file tree under src/, rooted at cohdl.toml's existing [package] name field. use path::Name; imports exactly one name; unqualified names stay fully visible within a single package with no imports (preserving today's ergonomics for the common single-package case). pub is enforced only across package boundaries — intra-package visibility is unaffected.

## Rationale

Option 1 was rejected: ad hoc disambiguation (e.g. auto-prefixing on conflict) is exactly the "correct by convention, not by the compiler" smell the Constitution forbids — an author couldn't know which MLCC they got without reading generated disambiguation output. Option 3 (explicit mod) was rejected: CoHDL has no use case for mod's actual justifications in Rust (non-file-tree-shaped organization, conditional compilation gating) — implicit mirroring is simpler and has one fewer thing that can drift from its own declared location. Option 4 (registry-scoped only) was explicitly not chosen — Tony's direct decision was to design the general system now, since project-internal and cross-library resolution are "deeply related anyway" and designing them separately risks solving the same problem twice, inconsistently.

## Consequences

- Second genuinely new core concept since the redesign began (after RFC-013's Layout Constraint) — Module/Package joins the canonical vocabulary, but formalizes a boundary note 2 and provisional-syntax.md already anticipated as inevitable, not a surprise addition.
- Real, disclosed breaking change: pub becomes enforced — but only across package boundaries; every current single-package project (std library, example boards) is unaffected until it's ever depended on by another package.
- Direct prerequisite for RFC-017 (library registry) — RFC-017 must not be implemented before this RFC's use/qualified-path/pub-enforcement mechanism exists.
- No glob imports or re-export sugar in this pass — explicit, one-name-per-use, deferred pending real usage friction.

## Revisit when

If real multi-package usage (once RFC-017 ships) reveals genuine friction from the lack of glob imports/re-export sugar, or from file-tree-mirroring being awkward for some library's natural organization — either is a scoped follow-up RFC, not evidence this RFC's defaults were wrong.

# DR-023: Library registry — source + documents + native footprints, skills explicitly deferred

## Context

RFC-017 (note 6) is the concrete registry Tony directed: a centralized place to publish/discover reusable .cohdl source, reference documents, manufacturer best-practice "skills," and footprints. Research confirmed no open, portable footprint file standard exists industry-wide — IPC-7351 is a naming/calculation methodology, not a file format; every CAD tool has its own proprietary/de-facto format (KiCad's .kicad_mod being the most "open" in spirit, but still KiCad's own). Given this real gap — and RFC-015's own named future work ("footprint-geometry resolution... named future work") — this RFC had to pick a format, not just a distribution mechanism.

## Options

1. Adopt KiCad's .kicad_mod as the library format.
2. Define a new, native, minimal CoHDL footprint format (.cfp — pads/shapes/layers only), authored directly by library maintainers, structurally checked against a device's declared pins at the point a part references it, projected by cohdl build into .kicad_mod/IPC-2581 geometry as needed — chosen per Tony's explicit decision favoring more control over an existing ecosystem (RFC-017's proposal).
3. Carry footprint geometry inline in .cohdl device/part declarations instead of a separate file.
4. Ship source + documents only for v1, defer footprints to a follow-up RFC.

## Decision

Option 2, per Tony's explicit choice. A Library is just a Package (RFC-016) with two new optional attachments: #[doc("path")] (one or more reference-document paths per declaration, same zero-impact discipline as #[intent(...)]/#[placement_hint(...)]) and a changed meaning for part's existing footprint: field — now a path to a .cfp file instead of a KiCad-library-reference string. .cfp pad numbers are checked against the bound device's declared pins at build time. Skills (manufacturer best-practice content) are explicitly deferred — this registry ships with exactly three content kinds (source, documents, footprints), not four, per direct decision.

## Rationale

Option 1 was the alternative directly considered and explicitly not chosen — Tony's stated reasoning: a native format gives CoHDL more control (structural validation at type-check time, projection into multiple downstream formats) at the cost of no existing ecosystem to leverage; recorded for completeness, not re-opened. Option 3 (inline geometry) was rejected: footprint authoring is a different kind of task (mechanical/geometric) typically done by a different contributor than the one writing electrical specs — keeping it a separate file keeps those authoring roles decoupled, the same reasoning that already keeps #[doc(...)] references separate from inline document text. Option 4 (defer footprints) was rejected: footprints were named by Tony as one of the four registry content kinds up front, and RFC-015 already flagged this gap once — deferring again would be a third RFC not closing it.

## Consequences

- Depends on RFC-016 (module system) landing first — this RFC's path-resolution for #[doc(...)]/footprints assumes RFC-016's package-relative resolution exists.
- Real, disclosed, non-mechanical breaking change: every existing part declaration's footprint: field changes meaning; the std library and example boards need real .cfp files hand-authored (genuine geometry work, not a mechanical retrofit like RFC-008's pin-role migration).
- New error codes in the existing E8xx block (designators & parts) for footprint/pin-count mismatches — the same "kind of mistake, not which pass" organizing principle RFC-011 established.
- Directly closes RFC-015's own named future-work item (footprint-geometry resolution) — cohdl build can now project real .cfp geometry into IPC-2581's physical section, though board-outline/stackup remains separately unaddressed.
- Skills get their own future RFC once the core registry's shape is proven in real use — not silently folded in, not silently dropped.

## Revisit when

Once skills' actual structure is scoped (free-form doc vs. structured/checkable data — an open question explicitly deferred, not answered by this RFC) — that's a dedicated future RFC. Also revisit if .cfp's deliberately minimal scope (no 3D models, no courtyard-beyond-basics) proves insufficient for real library-authoring needs.

# DR-023 amendment: footprint scope narrowed to symbol resolution, format deferred

## Context

Same day as DR-023's original acceptance, Tony directly corrected RFC-017's footprint design: (1) a footprint must be a named, resolvable symbol under RFC-016's module system — like every other cross-library reference — not a bare path string dangling off a part's footprint: field (which cannot be used, visibility-checked, or safely reused across libraries); (2) the footprint format itself (the .cfp grammar the original draft sketched inline) is out of scope for RFC-017 and belongs in a later, dedicated RFC.

## Options

1. Keep the original draft's design (bare path string + inline .cfp grammar) — rejected outright per Tony's direct correction, not seriously considered as a live option once raised.
2. Narrow RFC-017 to introduce footprint as a new top-level declaration kind resolved entirely through RFC-016's existing module-path/use/pub machinery — no new resolution mechanism, no format definition — leaving the declaration's internal content (and the pad/pin-count consistency check that depends on it) as an explicit, tracked gap for a future format RFC.
3. Fold the format question into this RFC anyway, just redesigned to route through symbol resolution (i.e. fix problem 1 but not problem 2).

## Decision

Option 2, per Tony's explicit two-part correction. footprint joins device/trait/fn/part as a fifth top-level declaration kind, resolved identically under RFC-016's rules. part's footprint: field now holds a symbol reference (resolved name), not a path string and not a format-string literal. The footprint { ... } declaration's body is left completely unspecified — RFC-017, as revised, is "symbol-resolution-complete, format-empty."

## Rationale

Option 1 was never a live option once Tony raised the correction — it was the exact defect being pointed out. Option 3 (fix resolution, keep format bundled) was rejected: it would have re-created the same problem DR-023's original text already flagged as a real risk elsewhere in the project (bundling two independently-decidable questions into one RFC) — resolution is answerable now by reusing RFC-016 wholesale; the format is a genuinely separate design problem (pad geometry, shape vocabulary, layer model) deserving its own focused RFC pass, not a rushed sub-section. Treating footprints via symbol resolution rather than a bare path also directly fixes the cross-library-reuse gap Tony named: two libraries can now use the same footprint declaration instead of one library only ever pointing at its own private file.

## Consequences

- RFC-017's Coherence Matrix row is revised downward on Compat/Trust/Grammar/Diagnostics/Netlist (all move from Med/High to Low/Med) — this RFC now delivers less than its original draft claimed, honestly disclosed as a narrower, still-real step rather than re-inflating the same row to look unchanged.
- The pad/pin-count consistency check between a footprint and its bound device — the original draft's central Trust argument — is no longer specified or enforced by RFC-017. It is explicitly named, tracked future work for the eventual format RFC, not silently assumed solved.
- Migration becomes two-stage: existing parts get their footprint: field converted to reference placeholder footprint symbols now (mechanical); giving those placeholders real content waits for the format RFC (not mechanical, not part of RFC-017's completion bar).
- #[doc(...)] (reference documents) is explicitly NOT converted to the same symbol-resolution treatment — a reference document is inert external content with nothing to gain from collision/visibility machinery; only footprints needed this fix, because only footprints needed cross-library reuse of a resolvable thing.
- A future, separately-numbered RFC now owns the footprint format question outright, inheriting RFC-017's Type-system-first classification (structural check, not DRC) as settled precedent.

## Revisit when

When the footprint format RFC is proposed — at that point, this amendment's "symbol-resolution-complete, format-empty" phasing is exactly what closes, the same way RFC-015's "logical-complete, physical-minimal" phasing is meant to close once footprint-geometry resolution and board-outline support land.

# Pending decision records (to be written as RFCs land)

(none — the backlog through RFC-017 (as amended above) is fully recorded above. The footprint format itself has no RFC number yet — it is tracked future work, not a pending decision record.)

# DR-024: Footprint format — copad/cofp, adopting Cadence's pad/footprint split

## Context

RFC-017 deliberately deferred the footprint format ("symbol-resolution-complete, format-empty"). Tony directed adopting Cadence Allegro's proven design: pads (padstacks) are defined once as standalone reusable objects; footprints reference pads by name, placing each at an offset, rather than inlining pad geometry per footprint. Tony specified the two new keywords directly: copad for pads, cofp for footprints.

## Options

1. A single declaration kind with pad geometry inlined per footprint (RFC-017's own original, withdrawn draft, before the symbol-resolution correction) — the alternative Cadence's design itself rejects, and the one this RFC exists to avoid re-adopting.
2. Two declaration kinds — copad (one reusable pad: shape, size, layer, plating) and cofp (a footprint: named pad references placed at offsets, plus courtyard/silkscreen-reference) — both resolved via RFC-016's existing module system, cofp retiring RFC-017's placeholder footprint keyword (RFC-018's proposal, per Tony's direct naming and design direction).
3. Merge pad and footprint into one keyword, distinguished by a structural-variant tag (RFC-008's pattern).
4. A richer, Cadence-parity padstack model (per-layer-independent geometry, vias, thermal reliefs).

## Decision

Option 2, per Tony's explicit design direction and naming. copad is a small, closed-vocabulary reusable pad primitive (shape ∈ {rect, circle, oval}, layer ∈ {top_copper, bottom_copper, through_all}, plating ∈ {smd, plated_through_hole}). cofp is a footprint composed of pad N: PadSymbol at (x, y) placements referencing copad symbols by name (resolved via RFC-016), plus courtyard/silkscreen_ref. cofp replaces RFC-017's placeholder footprint keyword outright — same role (what part.footprint: points to), real content for the first time.

## Rationale

Option 1 was rejected for exactly the reason Cadence's own design rejects it: no reuse across footprints, no single point of correction, real duplication risk as a footprint library grows. Option 3 (merge into one keyword with a variant tag) was rejected: pads and footprints have genuinely different reuse patterns and different consumers (many cofps reference one copad; nothing references a cofp) — this is a different relationship than RFC-008's variants (alternate shapes of the same device), so forcing them into that mechanism would misapply a pattern designed for a different problem. Option 4 (full padstack parity) was rejected as premature: CoHDL has no board-outline/stackup concept yet (RFC-015's still-open gap), so a padstack model needing multi-layer-independent geometry has no board context to place itself against — copad's single-layer-plus-through-all scope is the right-sized slice for now.

## Consequences

- One genuinely new core concept: Pad — a reusable geometric primitive referenced (never inlined) by footprints. This is the real conceptual move Cadence's design demonstrates and this RFC adopts, distinct from Footprint itself.
- Closes RFC-017's own deferred gap for real: the pad-count/numbering-vs-device consistency check RFC-017 could not specify (no real content existed) is now checkable, because cofp's pad list is real structured data.
- Directly closes RFC-015's named future-work item (footprint-geometry resolution) — cohdl build now has real geometry to project into .kicad_mod/IPC-2581, not an empty placeholder.
- Real but small, mechanical breaking change: footprint keyword retired in favor of cofp. Because RFC-017 shipped with no real footprint content anywhere (only empty placeholders, per its own two-stage migration), this rename has nothing real to migrate — a keyword swap, not a content rewrite.
- New failure mode disclosed honestly: because a copad may be referenced by many cofps, a wrong pad dimension is a single point of failure across every footprint referencing it — the flip side of the reuse benefit, not assumed away.
- No versioning/pinning mechanism for copad references — a cofp always resolves to whatever the referenced copad currently is, mirroring the same absence of version pinning at every other use-based resolution point in the language today.

## Revisit when

If a real need for per-layer-independent padstack geometry (vias, thermal reliefs) emerges — likely only once board-outline/stackup (RFC-015's gap) is itself addressed, giving such geometry a board context to mean something against. Also revisit if real library growth reveals copad's three-shape/two-plating vocabulary is too narrow — extend via a follow-up RFC, not a silent change to already-Accepted syntax.

# DR-024 correction (same day): keywords are pad/footprint, not copad/cofp

## Context

Same day as DR-024's original acceptance, Tony corrected the two invented keyword names: use plain pad and footprint instead of copad/cofp. Since RFC-017 had already claimed footprint as a top-level declaration kind (shipped with an unspecified body), this correction means footprint keeps its existing name and simply gains real, checkable content for the first time — no keyword rename anywhere, unlike the original draft's footprint → cofp swap. pad is newly reserved as the standalone reusable-pad declaration kind, replacing the invented copad name.

## Decision

RFC-018 is revised throughout: every copad reference becomes pad (the top-level reusable-pad declaration kind); every cofp reference becomes footprint (RFC-017's already-Accepted declaration kind, now with real content). The pad N: PadSymbol at (x, y) body-level placement statement inside a footprint declaration and the top-level pad { ... } declaration share the same keyword but occupy different grammatical positions — the same pattern already used elsewhere in the language (e.g. net/nc as body-level statements vs. other top-level forms), not a new ambiguity.

## Consequences

- RFC-018's Compat row moves from Med to Low — there is no keyword rename this time (footprint never changes name), only new content for an existing keyword plus one newly reserved keyword (pad). This is a smaller, more honest characterization than the original copad/cofp draft's.
- Teaching cost is marginally improved — plain English names need no explanation of an invented abbreviation's meaning, unlike copad/cofp.
- All downstream documentation (note 6, note 10, note 2) is updated in the same pass to use pad/footprint throughout, replacing copad/cofp.

## Revisit when

N/A — this is a same-day naming correction, not a design decision with its own future trigger. See DR-024's original "Revisit when" (padstack richness, vocabulary breadth) for the design's actual future triggers, unaffected by this naming correction.

# Pending decision records (to be written as RFCs land)

(none — the backlog through RFC-019 is fully recorded above.)

# DR-025: VS Code extension — a thin packaging + grammar layer over cohdl lsp

## Context

RFC-014 (LSP support) shipped a real, fully-wired cohdl lsp server (confirmed against real source: src/lsp.rs, tests/lsp.rs, all four capabilities implemented and equivalence-tested) but its own text explicitly deferred packaging: "a full marketplace extension (grammar, packaging) is separate scope per the RFC" — only a bare extension.js doc snippet in docs/lsp.md existed. docs/lsp.md itself flags a still-open acceptance item: "a pass in a live VS Code session has not yet been recorded." RFC-019 (note 6) closes both.

## Options

1. Leave the doc snippet as the only artifact indefinitely — status quo.
2. A real, buildable VS Code extension (editors/vscode/) — TextMate grammar for syntax highlighting (LSP has no highlighting verb, so this is a genuinely separate static artifact) + vscode-languageclient wiring identical to the existing doc snippet, packaged as an installable .vsix, with a cohdl.path setting replacing the snippet's hardcoded path (RFC-019's proposal).
3. A cross-editor Tree-sitter grammar instead of a VS-Code-specific TextMate grammar.
4. Bundle additional features (debugger, snippet library, format-on-save) into the same RFC.

## Decision

Option 2. Zero new diagnostic logic — the extension is a thin transport/packaging layer over the already-Accepted, already-tested cohdl lsp, the same "purely a new frontend" discipline RFC-014 established for the server itself relative to the compiler pipeline. New in this RFC: the TextMate grammar (hand-authored, derived from the real Accepted grammar across RFC-001–018), the .vsix packaging, and the cohdl.path settings key.

## Rationale

Option 1 was rejected: RFC-014's own text already named this exact gap as deferred, not solved — leaving it deferred indefinitely means the "ship with its check" / "ship with its spec update" discipline (note 6) never actually closes for the review-loop persona (a human in VS Code) this whole tooling layer exists to serve. Option 3 (Tree-sitter) was rejected as premature: no other editor's snippet currently asks for shared cross-editor highlighting, and TextMate is the minimum viable format for the one editor that needs packaged grammar today. Option 4 (bundle more features) was rejected as scope creep beyond the specific item RFC-014 deferred — debugger/snippets/format-on-save are separate, additive future RFCs once the base extension exists and is in real use.

## Consequences

- New in-repo directory editors/vscode/ (package.json, language-configuration.json, syntaxes/cohdl.tmLanguage.json, src/extension.ts, README.md) and a new CI job building/testing it.
- Closes RFC-014's still-open real-client acceptance item — this RFC's own verification step is running the extension against a real fixture in an actual VS Code session, not another round of server-only unit testing.
- New, disclosed, not-fully-solved risk: the TextMate grammar can drift from the real language grammar as future RFCs add/rename keywords (this project's own pad/footprint naming correction is a concrete recent example of the kind of change that would need a grammar update). A grammar coverage regression test catches some drift (unstyled fallthrough) but not mis-highlighting — mitigated by convention (future RFCs touching top-level keywords should update the grammar file, same discipline as note 10), not by a compiler-enforced guarantee, since none is possible for an external editor's grammar file.
- cohdl.path is the extension's only new settings surface in v1 — no version-compatibility handshake between the extension and the cohdl binary it spawns, a real named gap for a future RFC if it proves to matter.
- No change to any existing diagnostic, error code, designator, or netlist byte — purely additive, optional tooling.

## Revisit when

If real usage reveals grammar-drift is a recurring, painful problem — that's when auto-generating the grammar from the compiler's own lexer/parser definitions (rejected this pass as premature tooling investment) becomes worth building. Also revisit if a second editor develops a real, named need for shared highlighting — that's when a Tree-sitter grammar earns its cost.

# DR-026: Board outline (external file reference) + oriented placement — retroactive formalization + correction of an unauthorized implementation

## Context

board_outline { at: (cx, cy), size: (w, h) } and place at (x, y) were implemented directly on main (commits 86165d9, 1a0ce5f) to make examples/rpi-pico2's IPC-2581 output deliverable to Quilter — with no RFC, no decision record, and no note 6/note 10/note 2 update. The code's own comments acknowledged this ("pragmatic extension... pending an RFC"). Tony's direct review of the real implementation (reading src/ast.rs, src/ir.rs, src/check/expand.rs, docs/compliance-report.md) surfaced two real design defects beyond the process violation: (1) a board outline is a mechanical-engineering artifact (a DXF file from a mechanical engineer), not something CoHDL should author as a rectangle; (2) placement needs rotation, which the original implementation had no way to express at all — confirmed as the actual root cause of a real observed Quilter failure (a board-edge connector rotated 90° from its intended orientation).

## Options

1. Retroactively write an RFC that documents the existing { at, size } rectangle + coordinate-only place shape as-is, closing only the process gap.
2. Correct both defects while formalizing: board outline becomes a referenced external file path (never authored .cohdl geometry, mirroring RFC-017's #[doc(...)]/footprint-symbol precedent); place gains an optional rotate clause restricted to a closed four-value set (0/90/180/270), following the same closed-vocabulary discipline as RFC-001's units and RFC-008's pin roles (RFC-020's proposal, per Tony's direct correction).
3. Remove both constructs entirely, reverting to the pre-86165d9 state, and defer to a future RFC designed from scratch with no existing implementation to anchor against.
4. Add general 2D geometry/polygon-authoring syntax to .cohdl and/or an open-ended Angle unit type for arbitrary-degree rotation.

## Decision

Option 2, per Tony's explicit direction. board_outline: "path" replaces the rectangle-authoring block — a single string-literal path CoHDL never opens or validates. place at (x, y) [rotate ANGLE] gains an optional rotate clause; ANGLE is closed to {0, 90, 180, 270}. Both constructs stay non-DRC, checked structurally at declaration, zero schematic-correctness impact — RFC-013's own discipline, unchanged.

## Rationale

Option 1 was rejected outright: formalizing a shape already identified as wrong on the merits (a rectangle cannot represent a real board's mounting-hole cutouts, connector notches, or non-rectangular perimeter) would be process theater, not a real fix — the whole point of the RFC gate is to catch exactly this kind of defect before it's treated as settled. Option 3 (full revert) was rejected: the underlying capability (a board outline, a locked/oriented placement for edge-interface components) is a real, validated need — the rpi-pico2/Quilter failure was real, not hypothetical — reverting would lose real, working progress over a shape defect, when the shape can be corrected directly. Option 4 (general geometry authoring / open-angle type) was rejected per the project's own established narrow-first discipline (RFC-007 rejected const-generics, RFC-001 kept units closed until Length's concrete RFC-018 need justified extension) — no concrete need for anything beyond cardinal rotation has been shown, and CoHDL authoring real 2D CAD geometry directly contradicts the point of referencing an external mechanical file at all.

## Consequences

- board_outline's grammar surface shrinks (a path string, not an { at, size } geometry block) — a net reduction in what CoHDL's grammar owns, the opposite of conceptual-cost growth.
- Real, disclosed breaking change: rpi-pico2's existing rectangle-shaped board_outline must be replaced with a real DXF file reference — genuine non-mechanical migration work (an actual DXF must be sourced/authored), not a syntax-only fix, before this RFC is considered landed for that example.
- place's rotate clause is purely additive — every existing place at (x, y) statement (no rotate) is unchanged in meaning.
- The IPC-2581 emitter's board-outline responsibility changes shape: from synthesizing a rectangle Polygon to carrying a referenced file's outline geometry into Profile — real, scoped emitter implementation work, not optional.
- CoHDL still performs zero geometric validation of the referenced board-outline file's content (closed contour, self-intersection, etc.) and zero rotation math/collision reasoning — both disclosed, not silently assumed solved, consistent with DR-003's "layout stays a partner concern" line.
- Establishes a going-forward process point: any construct implemented directly on main without an RFC (as this one was) must be treated as provisional and non-final until a real RFC pass — including, where warranted, correcting the design rather than merely documenting it after the fact.

## Revisit when

If a genuine need for arbitrary-angle (non-cardinal) rotation emerges from real placement/fab-tooling usage — that's a scoped follow-up extending the closed rotation set, or introducing a real Angle unit type, not a reason to have opened it now. Also revisit if the board-outline file-reference mechanism proves insufficient for some board's real mechanical constraints not expressible as a single external file (unlikely, but not ruled out).

# DR-026 amendment (same day): scoped DXF geometry extraction required; fn-nested placement explicitly deferred

## Context

Same day as DR-026's original acceptance, Tony reviewed the revised design again and raised two further points: (1) a "reference the DXF, never parse it" board outline cannot actually produce IPC-2581's Profile element, which requires closed polygon/arc geometry embedded in the document itself — Quilter cannot import a document that merely points at an external file; (2) place has no way to reach an instance declared inside a called fn (confirmed against real source: src/check/expand.rs's handle_placement resolves only against a scope's local top-level instance names, and rejects place appearing inside a fn body outright). Tony's direction on the second point: defer it — support only top-level instances for now, rather than designing a path-qualification/disambiguation mechanism speculatively.

## Options

For the DXF question:

1. Keep "reference-only, never-parsed" — rejected outright, cannot meet the real requirement.
2. CoHDL extracts, narrowly, exactly one designated outline entity (a closed polyline, by convention on a fixed documented layer) from the referenced DXF, translates it into IPC-2581's native Profile/Polygon geometry, and ignores everything else in the file — the chosen option.
3. CoHDL becomes a general DXF/mechanical-CAD parser — rejected, unbounded scope.

For the fn-nested placement question:

1. Add a ::-separated path-qualification mechanism to place, reusing RFC-006's existing call-chain instance-naming scheme, with ambiguity-resolution rules for multiple calls to the same fn — designed in an intermediate draft of this RFC, then withdrawn.
2. Leave place scoped to top-level instances only, name the fn-nested case as a real, disclosed, deferred gap — chosen, per Tony's direct decision.
3. Add a new fn-level export/return mechanism so nested instances can be re-bound to a top-level name reachable by place.

## Decision

For board outline: Option 2 — scoped, single-entity DXF extraction, embedded as real IPC-2581 geometry. For placement: Option 2 — place continues to name only top-level design instances; reaching into a called fn is explicitly out of scope for this RFC, to be revisited only once a concrete design need justifies it.

## Rationale

The DXF question has no real alternative — "never parse" was not a real design option once IPC-2581's actual Profile requirement was traced through; this was a defect in the prior revision, not a legitimate trade-off. For the placement question, Option 1 (path-qualification) was withdrawn per Tony's direct call: the underlying gap is real, but no concrete design has yet needed it, and designing the right mechanism (path syntax, ambiguity rules) speculatively risks getting the shape wrong before real usage can inform it — the same reasoning the project has already applied to rejecting const-generics (RFC-007), deferring padstack richness (RFC-018), and keeping this same RFC's own rotation set closed rather than open-ended. Option 3 (a new fn export mechanism) was rejected as a strictly larger, unnecessary language feature for the same underlying reason.

## Consequences

- board_outline's build-time behavior is now real, scoped geometry-extraction work (not a no-op reference) — the IPC-2581 emitter must translate DXF polyline+bulge geometry into Profile/Polygon line+arc segments, a genuine, non-trivial but bounded implementation task.
- New E1006 sub-cases: missing/malformed/non-closed outline entity, unparseable DXF — real new diagnostic surface, disclosed rather than silently absorbed.
- place's scope is explicitly unchanged from before this whole RFC sequence began — top-level instances only. A component instantiated inside a called fn (e.g. a reusable connector sub-circuit) cannot be locked/oriented via place today. This is named, in both the RFC and note 10's "Not yet specified" list, as a real and disclosed limitation — not solved, not silently worked around by, e.g., discouraging fn-based connector sub-circuits without saying so.
- No dependency on RFC-006's call-chain path scheme in this revision (the withdrawn design would have depended on it; this revision does not).

## Revisit when

If a real design genuinely needs to place a component that only exists inside a called fn — at that point, design the path-qualification mechanism (or an export mechanism, or whatever the concrete need actually shows is right) against that real requirement, rather than the speculative shape considered and withdrawn here.

# DR-027: Footprint naming — adopt IPC-7351 (not JEDEC JESD30, not an invented scheme), no third-party footprint tracking

## Context

RFC-021 closed a gap left open since RFC-016/017/018: footprint declarations have a module-path identifier (for resolution) but no naming convention at all — any identifier that parses is equally legal, and nothing stops two libraries authoring "the same" real-world package from picking two unrelated arbitrary names. Footprint-naming research conducted alongside this RFC (worked examples: STM32F103C8T6's LQFP-48 7×7mm/0.5mm-pitch package, RP2350A's QFN-60 7×7mm/0.4mm-pitch exposed-pad package), cross-checked against real datasheets and JEDEC/manufacturer package data, confirmed two distinct real-world standards exist at two different layers: IPC-7351 (land-pattern/footprint naming + sizing methodology) and JEDEC JESD30 (package-body/outline designation). Tony's direct choice: adopt IPC-7351 specifically for footprint, corrected twice same day.

## Options

1. Leave footprint naming unconstrained — status quo, any identifier that resolves is accepted, no naming convention at all.
2. Adopt IPC-7351B's naming grammar as a new, optional, checked ipc_name field on footprint, alongside an otherwise-unconstrained free-form symbol name. This RFC's own first-drafted proposal.
3. Require the footprint declaration's own identifier — the same name RFC-016's module system resolves — to comply with IPC-7351B naming directly, with no separate field. This RFC's own first-corrected proposal, alongside a footprint_alias-style construct carrying third-party CAD tool (KiCad/LCEDA/Allegro) footprint names for cross-reference.
4. Same as Option 3, but with no third-party-footprint-tracking construct at all — CoHDL's footprint naming discipline applies solely to CoHDL's own native footprint/pad declarations (RFC-018); no backend-mapping, alias, or cross-reference of any kind exists. Chosen, per Tony's second, same-day correction.
5. Adopt JEDEC JESD30 instead — naming the package body/outline rather than the land pattern.

## Decision

Option 4, per Tony's two direct corrections of this RFC's own successive drafts. footprint (RFC-017/018's already-Accepted declaration kind) does not gain a new field — its own identifier is required to match one of a closed six-family-template set (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF) whenever the footprint's package family falls within that set. The identifier is grammar-checked against the matching template, and cross-checked against the footprint's own pad N: ... at (x, y) list for pin-count/pitch consistency when the layout is a regular rectangular-perimeter array. There is no footprint_alias, no backend-name table, and no reference of any kind to third-party CAD tool footprint libraries anywhere in this design — CoHDL's footprint/pad declarations (RFC-018) are its sole, native geometry model, and this RFC's naming discipline applies only to that model's own identifier.

## Rationale

Option 1 was rejected: leaving footprint naming permanently unconstrained means no shared convention is even possible by accident, and gives an AI-generating-a-footprint zero signal for a well-formed name beyond "resolves, doesn't collide" — the exact gap this RFC exists to close. Option 2 (a separate ipc_name field) was this RFC's own first draft, accepted and then corrected same day: Tony's direct objection was that two names for one thing is exactly the "two ways to identify the same thing" duplication the project rejects everywhere else. Option 3 (identifier-is-the-name, but with a third-party-backend-tracking construct alongside it) was this RFC's own first-corrected draft, itself further corrected same day: Tony's direct point was that CoHDL does not track or care about third-party footprint names at all — there is no mechanism to validate that a claimed KiCad/LCEDA/Allegro reference is real or current, and CoHDL's own footprint/pad declarations are a complete, self-contained geometry model with nothing to reconcile against an external library. Carrying such a reference would be tracking data with no real use and no correctness guarantee. Option 5 (JEDEC JESD30) was rejected for footprint specifically because JESD30 names the wrong layer — the package's mechanical body/outline, not the land pattern's copper geometry that footprint/pad (RFC-018) actually declare.

## Consequences

- footprint gains no new field and no new adjacent construct — this RFC constrains the identifier grammar of the declaration kind itself, for the closed six-family-template subset of IPC-7351B (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF) covering CoHDL's current real hardware.
- Two new checkable failure classes land in the existing E8xx block (designators & parts): malformed IPC-7351 footprint name (grammar), and name-vs-pad-geometry mismatch (pin count/pitch) for the geometry-regular families this RFC covers.
- A real, disclosed, ongoing consequence accepted deliberately: because IPC-7351 names are mechanically derived from geometry, a footprint's name changes whenever its geometry changes in a way that alters pin count/pitch/density (e.g. a density-level correction), and every use site referencing it must be updated. No second, stable-name layer was introduced to shield callers from this churn.
- No third-party-footprint-tracking mechanism exists in CoHDL, before or after this RFC — a real, deliberate scope exclusion, not an oversight. If a future integration genuinely needs to export to or reference an external CAD tool's footprint library, that is separate, not-yet-proposed scope.
- Geometrically irregular footprints (mixed pitch, non-perimeter layouts) get grammar-well-formedness checking only for a name that happens to match one of the closed templates — the geometry cross-check is honestly disclosed as not covering these, per RFC-021's Failure modes.
- Does not solve cross-library footprint deduplication (two libraries' footprints for the same real package still remain two distinct, non-deduplicated declarations at two different module paths) — RFC-017's already-disclosed non-goal, unchanged by this RFC.
- The two devices used to validate this RFC's derivation (STM32F103C8T6 → QFP50P900X900X160_48N; RP2350A → QFN60N40P700X700_1EP340X340) are named as the first real worked examples the std library should adopt once their pad content itself is authored.

## Revisit when

If real library authoring reveals the closed six-family-template set is too narrow (a real package family CoHDL needs doesn't fit any template) — extend the set via a scoped follow-up RFC, same discipline as RFC-001/RFC-018's own extension paths. Also revisit if the accepted rename-churn cost (Consequences above) proves too painful in real multi-library use. Also revisit if a genuine, concrete need emerges for CoHDL to export to or interoperate with a third-party CAD tool's footprint library — that would be a new, separately-scoped RFC weighing the real trade-offs of tracking external data, not a silent reintroduction of what was rejected here. Also revisit if a genuine need emerges for JESD30-style naming on package/variants (RFC-008) — a separate, not-yet-proposed RFC.

# DR-028: Mechanical locating holes in footprints — mount_hole, disjoint from pad numbering

## Context

Tony directed a new RFC to close a real gap in RFC-018's footprint model: some real footprints require a mechanical locating hole (定位孔) — e.g. a connector shell's alignment-pin holes — which has no electrical function, no net, and critically, no device pin number to bind to. RFC-018's own completeness guarantee requires every pad N: PadSymbol at (x, y) number to exactly match a bound device's declared pin numbers (RFC-002) — a locating hole cannot be expressed as a pad without breaking that guarantee or requiring a special-cased exception to it. Grounded against real, established practice: KiCad's own footprint format has a dedicated np_thru_hole (non-plated through-hole) pad type for exactly this distinction — a hole that exists in the footprint's manufactured geometry but carries no net, no pad number, no electrical role.

## Options

1. Model a locating hole as a pad with a new plating: mechanical value and no bound device pin — requires a special-cased exception to RFC-018's own pad-count/numbering completeness check, breaking its unconditional guarantee.
2. Fold locating holes into courtyard — conflates a keep-out boundary (no drill, soft placement convention) with a real drilled, manufactured feature — a category error.
3. A new footprint-body construct, mount_hole, numbered in its own namespace disjoint from pad numbers, never checked against the bound device's pins — position + diameter + a closed plated/non-plated flag (RFC-022's proposal, grounded in KiCad's np_thru_hole precedent).
4. A general mechanical-feature sub-language (slots, keyed holes, countersinks, arbitrary shapes) in the same RFC.
5. Include board-level mounting holes (a board's corner screw holes) in the same construct/RFC.

## Decision

Option 3. mount_hole N: PLATING at (x, y) diameter D is a new, optional footprint-body statement. N is a locating-hole-local counter, entirely disjoint from pad's pin-bound numbering — never checked against, or compared with, the bound device's declared pins. PLATING is a closed two-value set: non_plated (the common case) or plated (e.g. a chassis-ground stud, still with no net). diameter is a single Length-typed value, required regardless of plating. No layer: field — a mount_hole always spans through_all.

## Rationale

Option 1 was rejected: it would force a special case into RFC-018's own central, previously-unconditional pad-completeness guarantee — the same kind of structural inconsistency this project's discipline (DR-006, DR-017) has consistently rejected elsewhere in favor of a clean, disjoint mechanism instead. Option 2 was rejected: courtyard and a locating hole are different kinds of footprint content (soft keep-out convention vs. hard manufactured feature) — the same category-error reasoning RFC-018's own Alternatives already used to reject merging pad/footprint. Option 4 (general mechanical-feature language) was rejected as premature per this project's recurring narrow-scope-first discipline (RFC-007's rejected const-generics, RFC-018's own rect/circle/oval-only pad shapes) — no concrete need beyond circular locating holes has been shown. Option 5 (board-level holes) was rejected as a different owner entirely — a board's own mounting holes are a design/board-level fact analogous to board_outline (RFC-020), not a per-footprint one; bundling the two would conflate a footprint author's scope with a design's own board-level layout.

## Consequences

- footprint gains a third body-level construct (pad, courtyard, silkscreen_ref, now mount_hole) — no new top-level declaration kind, no new resolution mechanism.
- Real new emitter work: the KiCad .kicad_mod emitter projects non_plated as KiCad's own np_thru_hole pad type (the exact precedent this RFC is grounded in) and plated as an ordinary plated through-hole pad with no net; the IPC-2581 emitter projects both as hole/pin geometry with no net reference.
- New E8xx sub-cases (designators & parts, RFC-018's home for footprint-completeness checks): duplicate mount_hole number within one footprint, missing/malformed diameter, invalid PLATING value.
- Purely additive — every existing footprint with no mount_hole entries is completely unaffected, unchanged in emitted bytes.
- A real, disclosed non-goal: this RFC does not distinguish "this hole should have been board-level, not footprint-level" — that's a documentation/convention matter, not something the type system can catch, since both are geometrically identical (a hole at a position).

## Revisit when

If a real footprint needs a non-circular locating feature (a slot, a keyed/D-shaped hole) — extend mount_hole's shape vocabulary via a scoped follow-up RFC, the same extension discipline RFC-001/018 already established, not a silent grammar change. Also revisit if a genuine need for board-level mounting holes emerges — that is separate, not-yet-proposed scope, closer in spirit to board_outline (RFC-020) than to this RFC's footprint-local construct.

# DR-029: Non-circular locating holes — mount_hole gains shape/size, reusing pad's existing PadShape enum

## Context

RFC-022's own text explicitly named non-circular locating holes as deferred future work, to be triggered "if a real footprint needs" one. That real need materialized: the Kailh Choc V2 (Low Profile) switch's official datasheet ("Recommended PCB Mounting Pad Dimensions" page) was fetched and visually inspected, confirming a real, dimension-labeled 2.00mm × 1.50mm rectangular through-hole pad used as one of the switch's two mechanical mounting/support legs — distinct from its two circular electrical pin pads. Confirmed against real source (src/ast.rs): MountHole has exactly one geometry field (diameter: UnitValue), with no way to express a rectangular or oval shape.

## Options

1. Model the rectangular mounting leg as a pad with no bound device pin — rejected for the same reason RFC-022 itself rejected this for circular locating holes: it is non-electrical, has no pin to bind to, and would either be a category error or require a special-cased exception to RFC-018's unconditional pad-completeness guarantee.
2. Extend mount_hole with an optional shape: field (reusing RFC-018's existing PadShape enum: rect/circle/oval, unchanged, not redefined) and a shape-dependent geometry field — diameter for circle (the default, preserving every existing declaration's meaning), size: (w, h) for rect/oval, mirroring pad's own established shape-dependent-sizing convention exactly (RFC-023's proposal).
3. A separate construct (e.g. mount_slot) for non-circular locating holes.
4. Overload diameter to accept either a scalar or a (w, h) tuple, inferring shape implicitly from the field's arity.
5. A general 2D CAD/polygon authoring mechanism for arbitrary locating-feature shapes.

## Decision

Option 2. mount_hole gains an optional shape: field, one of {rect, circle, oval} (RFC-018's PadShape, reused verbatim). Absence of shape: defaults to circle, preserving every existing mount_hole declaration unchanged. The geometry field present must match the (explicit or defaulted) shape: diameter for circle, size: (w, h) for rect/oval — writing the wrong field for a given shape is a compile error, the same discipline pad already enforces for its own drill:/plating: pairing.

## Rationale

Option 1 was rejected for the identical reason RFC-022 rejected it for circular holes — no pin to bind to, and forcing it through pad would break RFC-018's unconditional completeness guarantee. Option 3 (a separate construct) was rejected: it would duplicate nearly everything mount_hole already does (disjoint numbering, plating, position, always-through_all) for what is really just a shape variation — the same "two constructs for one relationship" smell RFC-018's own Alternatives already rejected when considering separate pad/footprint constructs. Option 4 (implicit shape inference from field arity) was rejected: this is exactly the "correct by convention, not by the compiler" ambiguity the Constitution forbids elsewhere, and inconsistent with pad's own explicit shape: field precedent. Option 5 (general CAD authoring) was rejected as premature, unchanged from RFC-022's own reasoning — no concrete need beyond rect/circle/oval has been shown.

## Consequences

- mount_hole gains one new optional field and one new shape-dependent geometry rule — no new enum (PadShape already existed), no new top-level construct, no new resolution mechanism.
- Real new emitter work, but a direct reuse of existing code paths: the KiCad .kicad_mod emitter's np_thru_hole projection (already non-round-capable in KiCad itself) now also handles rect/oval mount_hole geometry; the IPC-2581 emitter projects the corresponding hole/pin geometry with no net reference, unchanged in spirit from RFC-022's circular case.
- New E8xx sub-cases (designators & parts, RFC-018/022's existing home for footprint-completeness checks): invalid mount_hole shape value, geometry-field/shape mismatch.
- Purely additive — every existing mount_hole declaration (all necessarily circular, since shape: didn't exist before) is unchanged in meaning; shape:'s default is circle.
- Real, non-mechanical follow-up work named but not required by this RFC's completion bar: the std library should gain an actual Kailh Choc V2 footprint using this new syntax — genuine content-authoring work, not part of what "Accepted" means here.

## Revisit when

If a real footprint needs a locating-hole shape beyond rect/circle/oval (a true slot with rounded ends, a keyed/D-shaped hole) — extend the shape vocabulary again via a scoped follow-up RFC, the same extension discipline this RFC itself just exercised, not a silent grammar change. Also revisit if real KiCad/IPC-2581 round-tripping reveals the rect/oval approximation is insufficient for some real hardware's actual rounded-rectangle geometry.
