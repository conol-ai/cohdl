# RFC-002: Pin connection-obligation typing

## Problem

In v1, nothing stopped a required pin from being left off every `net` in a design — the mistake was silent. The closest thing v1 had to a catch was W003/W004 (single/multi-driver DRC rules), which don't even cover this case; an unconnected-but-required pin (e.g. an MCU's `VDD` never wired to anything) wasn't reliably caught by anything. This is exactly the "a human forgot to wire something" failure class the v2 redesign's thesis targets: it's a structural property of one instance's pins against its device definition, not something that needs the whole connectivity graph to detect, so it belongs in the type system, not in DRC.

Who this is for: primarily the **AI author**, who gets an immediate, local error the moment a design is checked, naming exactly which pin on which instance is unresolved — rather than a floating pin that silently ships. Secondarily the **human reviewer**, who no longer needs to manually audit "did every required pin get used somewhere" — the compiler already guarantees it.

## Goals

- Make "forgot to connect a required pin" a compile-time type error, not silence.
- Make "intentionally left unconnected" an explicit, visible, distinct declaration — not the same as forgetting.
- Keep the mechanism exhaustive: every required pin on every instance in the final design must resolve to exactly one of two states, with no third silent option.

## Non-goals

- **Not a general nullable/optional-value system for the language.** This RFC only concerns pin resolution, not a broader `Option<T>`-style mechanism for other concepts. If a future need arises for optionality elsewhere, that's a separate RFC.
- **Not solving float/analog pin behavior** (e.g. modeling what "floating" actually does electrically). `nc` is a structural marker for the compiler, not a simulation of an open circuit — CoHDL is not a simulator (Constitution non-goal, unaffected by this RFC).
- **Not changing how **`net`** works.** `net` remains the only connectivity mechanism (Constitution/Conceptual Model hard constraint) — this RFC adds a second, distinct *non*-connectivity declaration (`nc`) that can never be confused with or substitute for `net` membership.

## Design

### Two connection-obligation kinds, declared on the device/trait, not the instance

Every pin in a `device` or `trait` pin definition carries an explicit obligation kind:

- `required` — this pin **must** be resolved (see below) for any instance of this device to type-check.
- `optional` — this pin may be left unmentioned entirely; it needs no `net` and no `nc` declaration. Reserved for pins the datasheet itself defines as safe to leave floating (test points, truly optional features) — not a way to opt out of the strictness this RFC exists to add.

The obligation kind is part of the **device/trait definition**, not chosen per-instance — this mirrors RFC-003's "trait satisfaction checked at definition time" philosophy: whether a pin is required is a fact about the component, not a choice an instance author makes per board. (An instance cannot downgrade a `required` pin to optional; that would let a device definition's contract be quietly weakened per use, which is exactly the kind of context-dependent meaning-shift note 2's "model smells" section rejects.)

### Resolving a required pin: `net` or explicit `nc`, nothing else

A `required` pin on an instance resolves in exactly one of two ways:

1. **Connected** — the instance's pin reference appears in some `net` declaration, same mechanism as v1/unchanged.
2. **Explicitly not-connected** — the instance's pin reference appears in an `nc` declaration:

`nc` is a **top-level design-body declaration**, syntactically parallel to `net` (a flat list of pin references), so it reads and generates exactly like a `net` block a model already knows how to emit — no new grammar shape, just a new keyword with a different semantic (a pin listed under `nc` joins no net; it is marked resolved-as-absent). This deliberately avoids inventing a structurally new declaration form (regularity over cleverness).

**A **`required`** pin that appears in neither a **`net`** nor an **`nc`** declaration is a compile error** at design type-check time — the exhaustiveness check (see Gradeability). There is no silent third option.

### Interaction with `fn` sub-circuits and pin pass-through

A `fn` sub-circuit often receives a device/instance as a parameter and only wires *some* of its pins internally, intentionally leaving others for the caller to resolve. This is not "forgetting" — it's normal composition, and the obligation check must not fire prematurely inside the `fn` body.

**Resolution: the exhaustiveness check runs once, at final **`design`** assembly, after all **`fn`** inlining/monomorphization — never inside an unassembled **`fn`** body.** A `fn`'s own required-pin obligations are inherited by whatever calls it; only when a pin's owning instance is finally placed inside a top-level `design` (directly or through nested `fn` inlining) must every one of its `required` pins be resolved. This is consistent with Conceptual Model's fn semantics (nested calls monomorphize and inline; a `fn`'s behavior is fully determined by its parameters — nothing here creates ambient state or a new resolution mechanism, just the obvious point at which the check must apply).

### Example: what this catches that v1 didn't

In v1 this design might have type-checked and even emitted a netlist with `VDD` simply absent from any connection — a real, physically broken board that the compiler called "correct." In v2 this is a compile error naming the exact pin.

## Type-system-first test

N/A — this RFC is itself the type-system mechanism that replaces what would otherwise have needed a DRC-style "unconnected pin" rule (the closest analog to v1's dormant W003/W004, which didn't actually cover this case anyway). The alternative of catching this via DRC was considered and rejected (see Alternatives) — the mistake is a property of one instance's pins against its own device definition, not an emergent property of the whole connectivity graph, so per DR-006's classification logic it belongs in the type system.

## Conceptual impact

Extends **Pin** (existing concept) with an obligation kind — no new concept invented, no overlap with the canonical vocabulary. Adds one new top-level syntax form, `nc`, which is conceptually a sibling of `net` (both are pin-reference lists at the design-body level) but semantically its opposite (marks absence of connection, not presence) — this is a deliberate, minimal addition, not a parallel connectivity mechanism (see Non-goals: `net` remains the only way to connect).

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Low | High | High | Med | N/A (pre-launch) | High |

Oracle (High): closes a previously entirely-uncovered mistake class (unconnected required pins) — a genuine strengthening of what "correct" means.



Diagnostics (High): the exhaustiveness check must name the exact unresolved pin and instance — this is a new, frequently-firing diagnostic family, so its message quality matters immediately (see Tooling & operations).



Netlist (Med): nc-marked pins must be represented in emitted netlists in a way that's faithful (KiCad/LCEDA both have "no connect" conventions) — an emitter-level requirement to track, not a design risk.



Grammar (Low): nc reuses the exact shape of an existing declaration (net's pin-list syntax) — no new parsing complexity.

## Gradeability

Enforced at **type-check time**, specifically as an exhaustiveness pass run once per fully-assembled `design` (after all `fn` inlining/monomorphization): for every instance in the design, every `required` pin must appear in exactly one of {some `net`, the `nc` list}. A pin appearing in *both* a `net` and the `nc` list is also a compile error (contradictory declaration — the same pin can't be both connected and explicitly not-connected). This is the earliest possible stage per the redesign's tie-break rule, and requires no `rule`/DRC involvement at all.

## AI-generatability

High. A model already must emit `net` declarations to describe a working board; `nc` is the same syntax shape with different meaning, so there's no new construct-family to learn, only a new keyword and when to reach for it (any datasheet-optional pin the design doesn't use). The exhaustiveness error, when it fires, names the exact instance and pin — giving the repair loop a one-line fix (`add mcu.VDD to a net or to nc`), which is about as cheap a repair as a diagnostic can offer.

## Alternatives

- **Leave all pins implicitly optional (v1's status quo)** — rejected: this is the exact failure mode the RFC exists to close.
- **A DRC rule that scans for unconnected pins after the whole design is built** — rejected per the type-system-first test: the mistake is local to one instance's pins vs. its own device definition, fully knowable without the whole connectivity graph, so catching it at type-check time is both earlier (better AI-generatability) and structurally guaranteed (no dormant-rule risk, unlike v1's W003/W004).
- **A single **`Option<Pin>`**-style wrapper type instead of a separate obligation-kind + **`nc`** declaration** — considered, rejected: would require threading optionality through every pin reference site, a much larger grammar/type-system surface for the same guarantee; the two-declaration-forms approach (`net` / `nc`) is simpler and reuses an existing declaration shape (regularity over cleverness).
- **Making **`nc`** a per-pin attribute instead of a top-level declaration** (e.g. `#[nc] mcu.RTC_XTAL_IN` inline) — rejected: breaks locality of a different kind (obligations become scattered per-instance rather than visible in one place alongside `net`s), and doesn't match the "one canonical way, mirrors `net`'s shape" goal.

## Compatibility

N/A — pre-launch, no existing `.cohdl` source to break.

## Tooling & operations

- The exhaustiveness diagnostic must state the instance path and pin name explicitly (e.g. `required pin 'mcu.VDD' is unresolved: add to a net or to nc`) — this will likely be one of the most frequently-fired diagnostics in early AI-generation attempts, so message quality directly affects repair-loop efficiency.
- A pin declared in both a `net` and `nc` must produce a distinct diagnostic from "unresolved" (contradictory, not missing) — these are different mistakes and an AI repair loop benefits from telling them apart.
- The LSP should offer hover info on any pin reference showing its obligation kind (`required`/`optional`) directly from the device/trait definition, so a human reviewer or the model (if given LSP-derived context) can check without re-reading the whole device source.
- Reserve an error-code sub-block adjacent to RFC-001's unit-diagnostics block for pin-obligation diagnostics (final numbering deferred to RFC-011).

## Teaching cost

Low. Two new keywords (`required`/`optional` on pin declarations, `nc` at the design-body level), each with one clear rule. The `fn`/nested-call deferral behavior (check only fires at final design assembly) is worth one sentence in the reference but doesn't add author-facing complexity — an author writing a `fn` doesn't need to think about it at all; it only matters to someone building the type checker.

## Failure modes

- **Model marks a pin **`nc`** just to silence the error, without checking if it should actually be connected** — a real risk, since `nc` becomes an "escape hatch." Mitigated partially by keeping `nc` visible and diffable (a reviewer scanning the design body sees every `nc` declaration in one place, same as `net`s) — this is a human-reviewability safeguard, not a compiler one; the compiler can't know intent, only structure.
- **A device definition marks a pin **`optional`** when it should be **`required` (under-specifying the contract) — this is a std-library-authoring mistake, not something this RFC's mechanism catches; device/trait authors bear responsibility for correctly transcribing the datasheet's actual pin requirements, same as any type declaration can be "technically valid but wrong."
- **A **`fn`** accepts an instance parameter and never exposes a way for the caller to resolve its remaining required pins** — an authoring/API-design mistake in the `fn`'s own signature, not a language-level gap; well-designed sub-circuit `fn`s should either fully resolve all pins internally or clearly pass through what's left (this is a std-library and generics-design concern that RFC-007 should account for when it lands).

## Migration path

N/A — pre-launch.

## Decision

Accepted — 2026-07-13. Recorded as DR-012 (see note 7 — DR-008/009/010 stay reserved for RFC-005 / RFC-004+011 / RFC-012 respectively, so this decision takes the next open number; deliberately not "DR-011" to avoid confusion with the unrelated RFC-011 error-code-registry item). RFC-003 (trait-satisfaction-at-impl-time) and RFC-006 (nested fn semantics) should reference this RFC for how pin obligations interact with trait definitions and fn inlining, rather than re-deriving pin resolution.
