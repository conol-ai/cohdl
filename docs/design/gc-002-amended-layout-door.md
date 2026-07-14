# GC-002 (amended): Admit layout constraints into the conceptual model

## Original goal

CoHDL describes schematic connectivity and specs only. Layout/routing (placement, net classes, differential pairs, length matching, routing rules) is out of scope — a partner backend's job, referenced by the language only through the "seam" note 2 and the Constitution both already describe but never formalized into real syntax.

## New goal

Layout constraints become a real, formalizable concept in the v2 conceptual model — declarative, non-executing metadata attached to nets/instances (net classes, differential-pair pairing, length-matching groups, placement hints), consumed by a partner layout tool, never interpreted or acted on by CoHDL itself. CoHDL still does not place, route, or reason about physical geometry — it only lets an author *state* a layout-relevant constraint in source, so it survives the netlist handoff instead of living only in a human's head or an out-of-band spec sheet.

## Reason

Tony's explicit directive (2026-07-13): open this door now, superseding GC-002's prior "staged, requires a concrete partner requirement" gate.

**Honest flag, not silently waived**: GC-002's original acceptance criterion explicitly required *"a concrete partner requirement"* before opening this door — this amendment proceeds without one currently in hand. This is a real, acknowledged deviation from the gate's own stated bar, made because Tony chose to open the door now rather than wait for that trigger. Recording this plainly here (rather than pretending the criterion was met) is the same discipline this whole project has held all day: never assert a bar was cleared when it wasn't.

## Strategic context

The seam for this was **already deliberately designed into note 2 and the Constitution during the 2026-07-13 redesign** — "a `net_class`/`constraint` decoration adjacent to Net/Rule, never a second connectivity mechanism, inspectable and gradeable like a rule, losslessly ignorable/passable through codegen" (note 2, "The seam for the layout door"). This amendment is therefore not inventing new domain-boundary architecture; it's exercising a seam the redesign already anticipated and left ready, one layer earlier than originally planned (before a concrete partner requirement forced the question).

## Affected users

No change to priority order (AI author → human reviewer → tool builder). New capability for the **tool builder** tier specifically: a partner layout tool now has a real, structured surface (declarative constraints in the netlist/IR) to consume instead of needing side-channel communication with whoever authored the schematic.

## Affected principles

Does **not** touch the trade-off priority ladder or the "not a layout/place-and-route engine" non-goal — CoHDL still never places or routes anything; it only lets a constraint be *stated*. This is consistent with (not a revision of) the Constitution's Non-goals: "Physical placement and routing are a partner concern. CoHDL describes what connects to what and to what spec — not where copper goes." Layout *constraints* (the what-should-hold) are declarative data about the netlist; layout *execution* (the where-copper-goes) remains entirely out of scope, unchanged.

## Affected concepts

New concept: **Layout Constraint** — a declarative annotation, syntactically similar in shape to a `rule` (a first-class, inspectable statement) but semantically inert to CoHDL's own compilation (parses, type-checks against a closed set of constraint kinds, never affects verdict/netlist connectivity — the same "structurally enforced, not conventionally enforced" zero-impact discipline RFC-012's `#[intent(...)]` established for design rationale, applied here to layout instead). No existing concept (Trait/Device/Part/Instance/Pin/Net/Spec/Rule/Module/Fn/Design/Designator) is renamed or reshaped.

## Affected capabilities

Adds a new Layer (or a new Layer-2/3 capability, see note 3's capability map) for layout-constraint parsing/type-checking and constraint-data emission into the netlist/IR. Zero impact on any already-Accepted RFC (RFC-001–012) — a layout constraint is additive metadata riding alongside the existing netlist, not a change to how nets/pins/devices/traits/generics/variants already work.

## Compatibility risks

None for existing source — purely additive syntax. Real risk: without a concrete partner tool actually consuming this data yet, there's a genuine chance the constraint *kinds* chosen (net class, diff-pair, length-match, placement hint) don't match what a real partner integration eventually needs — this is the direct consequence of proceeding without the originally-required partner requirement, named honestly above, not glossed over.

## Required migrations

None — additive.

## Design debt created

**The constraint-kind list is provisional, not partner-validated.** Should be revisited the moment a real layout-tool integration is scoped, and explicitly flagged as such in RFC-013 itself (see its Compatibility/Failure modes sections) — this debt is the direct, named cost of opening the door before a concrete partner requirement existed.

## Operational risks

Scope creep risk: once layout constraints exist as a real concept, there's pressure to let CoHDL "just check" more layout-adjacent things itself (trace width vs. current, keep-out zones) — each such request must pass the same Coherence Matrix conceptual-cost scrutiny as any other, and must be independently justified, not waved through because "we already opened the layout door."

## Communication plan

This note (amending GC-002 in place, per Evolution Governance's own convention of tracking goal changes explicitly) plus RFC-013 (note 6) plus a new Decision Record (note 7) plus a Language Specification section (note 10) — the same four-artifact discipline every other Accepted RFC in this backlog has followed.

## Decision

**Accepted** — 2026-07-13, per Tony's explicit directive. Supersedes GC-002's prior "staged" status. The concrete-partner-requirement criterion is explicitly waived by this decision, not silently met — see Reason above. RFC-013 (note 6) is the resulting RFC, evaluated in full against the standard template despite this GC unlocking it early.
