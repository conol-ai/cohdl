# RFC-004: DRC/type-system reclassification pass

## Problem

DR-006 already decided the *principle* (structural checks move to the type system; genuinely emergent/numeric checks stay in DRC) and *pre-classified* two of v1's nine rules (E003–E005 → type system; W003–W004 → residual DRC) as illustrative examples. But DR-006 never actually audited all nine rules against real v1 source — it reasoned from memory/paraphrase. This RFC does the actual audit, against the real rule implementations in the (still-cloned, soon-to-be-discarded) v1 repo, and is the formal classification pass note 6 and the Coherence Matrix both require before residual DRC's scope can be considered settled. Skipping this — assuming the earlier illustrative examples were the complete picture — is exactly how a check could silently fail to migrate to either bucket and quietly disappear, the same failure shape DR-006 exists to prevent.

Who this is for: **compiler implementers** (whoever eventually builds the v2 type checker and residual DRC engine) — this RFC is their authoritative checklist for "does every v1 check have a home in v2." Secondarily, **RFC-011 (error-code registry)**, which cannot finalize codes until this classification is final.

## Goals

- Enumerate all nine v1 DRC rules (E001, E002, E003, E004, E005, W001, W002, W003, W004) from the actual v1 source, not from memory.
- Classify each as **"becomes a type-system mechanism"**, **"stays residual DRC"**, or **"reconciled/superseded by an already-accepted RFC's mechanism"** (a third bucket DR-006 didn't anticipate, needed because RFC-001–003 already redesigned some of this ground).
- Leave zero rules unclassified — this is the actual point of the pass.

## Non-goals

- **Not implementing anything.** This RFC is a classification/routing decision, not new mechanism design — RFC-001–003 already built the type-system mechanisms two of these checks route into; RFC-007 will need to extend one of them (trait bounds on generics) for E004.
- **Not preserving v1's rule IDs.** E00x/W00x numbering is v1-specific; v2's error-code registry (RFC-011) assigns its own codes once this classification is final.
- **Not re-opening RFC-001/002/003's already-decided designs.** Where a v1 rule is superseded by one of them, this RFC states the mapping — it doesn't modify those RFCs.

## Design — the actual classification, audited against real v1 source

| v1 rule | What it checked (from real v1 source, `crates/cohdl-drc/src/rules.rs`) | Classification | Where it lands in v2 |
|---|---|---|---|
| **E001** `voltage_exceed` | Instance's `voltage_rating` spec < the voltage annotation on the net it's connected to. | **Stays residual DRC.** | Net voltage is emergent from the whole net's connections — genuinely cross-cutting, exactly DR-006's "stays DRC" criterion. Still expressible as a `rule` block per the Conceptual Model's residual-DRC scope, now checked with RFC-001's unit-typed `Voltage` comparison (type-checked for unit-match inside the rule itself, per RFC-001's design). |
| **E002** `polarity_mismatch` | A device satisfying `Polarized` has its `A` (anode) pin connected to a GND-annotated net. | **Stays residual DRC**, with one input now type-checked. | "Is this net GND" and "is this pin connected to that net" are graph-emergent facts — stays DRC. But "does this device implement `Polarized`" is now answerable at compile time via RFC-003's `impl` mechanism (no longer inferred from a string-matched `impl_traits` substitution key, as v1 did) — the rule's *precondition* gets sounder, even though the rule itself stays DRC. |
| **E003** `spec_not_satisfied` | A device implementing a trait requiring spec fields is missing one, checked **per-instance** via `generic_substitutions`. | **Becomes a type-system mechanism.** | Fully superseded by RFC-003: satisfaction (including all required spec fields) is checked once, at the `impl Trait for Device` statement, using the device's own declared fields — not per-instance, and not dependent on a meta-key the type checker has to remember to populate (the exact v1 failure mode). E003 as a DRC rule is retired; its job is now done earlier and more reliably. |
| **E004** `trait_not_impl` | A generic argument doesn't implement a trait its parameter bound requires. | **Becomes a type-system mechanism — but not fully closed yet.** | This is a **generic-parameter trait-bound check** (e.g. `fn foo<D: Capacitor>(...)`, is the concrete `D` argument's device actually `impl Capacitor`?) — distinct from RFC-003's device-level `impl` checking, and squarely RFC-007 (generics-over-specs) territory. RFC-003 supplies the mechanism (`impl Trait for Device` lookups); RFC-007 must specify how a generic parameter's trait bound is checked against the concrete type argument at the call/instantiation site. **Flagging this explicitly as an open dependency RFC-007 must close** — not fully resolved by RFC-003 alone, unlike E003/E005. |
| **E005** `missing_spec_field` | A trait spec field not provided in the device instantiation — checked at **instantiation** time (later than E003's already-late per-instance check). | **Becomes a type-system mechanism.** | Fully superseded by RFC-003, same as E003 — in fact E003 and E005 in v1 were checking near-identical things at two different times (definition-adjacent vs. instantiation), both symptoms of "no single early checkpoint existed." RFC-003's `impl`-time check collapses both into one earlier, single check. |
| **W001** `unconnected_pin` | **Any** pin on an instance lacking a net connection — v1 did not distinguish required vs. optional pins (that distinction didn't exist in v1). | **Reconciled by RFC-002 — narrower in v2, and correctly so.** | RFC-002 already added an equivalent, stronger check: every `required` pin must resolve to `net` or explicit `nc`. But RFC-002 deliberately allows `optional` pins to be left unmentioned — which v1's W001 would have flagged as a warning. **This is not a gap**, it's the intended behavior: v1 had no obligation-kind concept, so it warned indiscriminately; v2 only enforces resolution where the device/trait author declared an actual obligation. W001 as a blanket "any unconnected pin" rule is retired; RFC-002's exhaustiveness check replaces it with a strictly more correct, obligation-aware version. |
| **W002** `floating_net` | A net declared with zero instance pins connected (only external references or none). | **Becomes a type-system mechanism.** | This is checkable from a single `net` declaration in isolation — does this specific `net` block name at least one instance pin? — without needing the rest of the connectivity graph. Per the type-system-first test, this is structural, not emergent, so it should be a compile-time check on the `net` declaration itself (a `net` with zero instance-pin members is a compile error, not a DRC warning) — narrowing residual DRC further than DR-006's original two-rule example anticipated. |
| **W003** `single_driver` | A net has exactly one output-type (driver) pin connected — likely unfinished wiring. | **Stays residual DRC**, per DR-006 (unchanged from the original classification). | Requires knowing every pin on the net and each one's driver-role — genuinely emergent from the connectivity graph, the clearest "stays DRC" case in the whole set. |
| **W004** `multi_driver` | A net has more than one output-type (driver) pin connected. | **Stays residual DRC**, per DR-006 (unchanged from the original classification). | Same reasoning as W003 — emergent, graph-wide, correctly DRC's job. |

### Summary of the classification

- **Type system (4):** E003, E004*, E005, W002 (*E004 needs RFC-007 to fully close the generic-bound-checking half; RFC-003 supplies the base mechanism).
- **Residual DRC (4):** E001, E002, W003, W004 — all genuinely emergent/numeric/cross-cutting, matching the Conceptual Model's narrowed definition of what `rule` is for.
- **Reconciled/retired, not carried forward (1):** W001 — superseded by RFC-002's obligation-aware exhaustiveness check, which is strictly more correct than blanket-flagging every unconnected pin.

This is a materially more complete result than DR-006's original two-example illustration — it resolves all nine rules, not two, and surfaces one genuine open dependency (E004 → RFC-007) that hadn't been explicitly flagged before.

## Type-system-first test

This entire RFC *is* the type-system-first test, applied exhaustively rather than illustratively. Each row's classification states explicitly why it landed where it did, per DR-006's criterion: local to one device/instance/declaration → type system; emergent from the whole connectivity graph → residual DRC.

## Conceptual impact

None beyond what RFC-001–003 already introduced — this RFC routes existing v1 concerns to already-accepted mechanisms (or confirms they stay DRC); it adds no new concept, syntax, or grammar of its own. The one net-new item is the **W002 → type-system** classification, which doesn't need a new concept either — it's a structural completeness check on the existing `net` declaration form (a `net` naming zero instance pins is already inspectable from that declaration alone).

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | High | Med | Low | N/A (pre-launch) | High |

**Oracle (High):** this is the pass that makes the Coherence Matrix's own mandatory mitigation real — "narrowing DRC without confirming the type-system replacement covers the same ground" was the exact risk flagged when DRC's scope was first narrowed; this RFC is that confirmation, done exhaustively.
**Trust (High):** a reviewer (or future contributor) can now see, rule by rule, that nothing from v1's real check surface silently vanished — each of the nine has an explicit, justified destination.
**Diagnostics (Med):** W002's move to type-system status means a "floating net" error now fires earlier (at the `net` declaration) rather than after full DRC runs — a message-timing change worth tracking when RFC-011 assigns codes.

## Gradeability

This RFC doesn't add a new gradeability mechanism — it confirms which rung of the verdict ladder each of the nine v1 concerns now lives on: `type-checks` for E003/E004(partial)/E005/W002, `passes residual DRC` for E001/E002/W003/W004, and "superseded, no longer a distinct check" for W001 (its job is done by the pin-obligation exhaustiveness check instead, which itself is part of `type-checks`).

## AI-generatability

Improves for the four checks now caught at type-check time (earlier feedback, smaller diff to fix) and stays the same for the four that remain DRC (no regression — they were already appropriately graph-level checks). The E004→RFC-007 open dependency is the one place a model could get inconsistent feedback until RFC-007 lands: a generic-bound trait mismatch might not yet be caught as early as this RFC ultimately intends. This is called out explicitly so it isn't mistaken for "already solved."

## Alternatives

- **Treat all nine as "stays DRC" and just wire them as v1 intended** — rejected: this is literally the option DR-006 already rejected; re-litigating it here would contradict a settled decision without new information.
- **Classify only by "is it currently implemented as DRC in v1," ignoring whether a type-system mechanism already exists for it** — rejected: RFC-001–003 already built mechanisms that make E003/E005 (and most of W001) redundant as DRC rules; refusing to route to them would mean maintaining two independent, potentially-inconsistent checks for the same electrical fact.
- **Defer classifying W002 until a "structural net-shape" RFC is written** — considered, rejected: W002's check (does this net declaration name at least one instance pin) is simple enough and clearly structural enough to classify now, without needing its own dedicated RFC; it will be implemented as part of whatever RFC formalizes `net`/`design` type-checking rules, without requiring new conceptual design here.

## Compatibility

N/A — pre-launch. No v1 rule IDs (E00x/W00x) carry forward as codes; RFC-011 assigns v2's own registry informed by this classification.

## Tooling & operations

- RFC-011 (error-code registry) must treat this table as its primary input for which checks need type-checker diagnostic codes vs. residual-DRC rule codes.
- The residual DRC engine's rule set for v2 is exactly {E001-equivalent, E002-equivalent, W003-equivalent, W004-equivalent} — four rules, not nine. Anyone implementing the v2 DRC engine should treat a fifth "structural" rule request as a signal to re-run the type-system-first test, not to just add it.
- E004's open half (generic-bound checking) must be tracked as a named follow-up inside RFC-007's own scope, not silently assumed solved by this RFC or by RFC-003.

## Teaching cost

None — this RFC doesn't add anything an author needs to learn; it's a routing decision for implementers and RFC-authors, invisible to `.cohdl` authors themselves.

## Failure modes

- **Assuming E004 is fully solved by RFC-003 alone** — explicitly flagged as false in this RFC; RFC-007 must close the generic-bound half or E004's real job stays unimplemented in v2 despite looking "classified."
- **Re-implementing W001 as a literal port of v1's blanket unconnected-pin check** — would directly contradict RFC-002's deliberate design (optional pins may be silently unmentioned); must not happen even though it would superficially look like "restoring a v1 check."
- **Treating this classification as final and immutable** — it's final for the nine v1 checks that exist today; if std-library growth surfaces a genuinely new structural or emergent check later, it goes through the same type-system-first test on its own merits, per note 6's RFC template, not by amending this table.

## Migration path

N/A — pre-launch.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-014 (see note 7). This unblocks RFC-011 (error-code registry), which can now assign codes against a settled classification. RFC-007 (generics-over-specs) must explicitly close the E004 generic-bound-checking gap flagged here as part of its own scope — referenced, not solved, by this RFC.
