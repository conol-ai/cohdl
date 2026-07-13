# RFC-007: Generics-over-specs and generic trait bounds

## Problem

Generic parameters (`Device<C: Capacitance, V: Voltage = 10V>`, `fn decoupling_cap<V: Voltage>(...)`) already appear throughout RFC-001, RFC-002, RFC-003, and RFC-006's examples, and are load-bearing for all four — but no RFC has formally specified what a generic parameter *is*, what it can be bound by, how defaults work, or — critically — **how a generic type parameter's trait bound is checked against a concrete type argument at a call/instantiation site.** RFC-004's classification pass explicitly flagged this as the still-open half of v1's E004 (`trait_not_impl`): the device-level half is closed by RFC-003, but the generic-parameter half needs its own design.

Reading v1's real source clarifies exactly what's needed: v1 actually had **two separate, overlapping mechanisms** for this — a generic *type* parameter bound checked via `type_implements_trait` (`crates/cohdl-sema/src/typeck.rs:1465`), and a *value* parameter typed `impl Trait` checked via a different path in `check_call` (`typeck.rs:1421-1461`). Both call the same underlying trait-satisfaction lookup, but critically, **that lookup reads **`dev.impl_traits` — a device's own embedded list of implemented traits. RFC-003 removed that field entirely (traits are now satisfied via free-standing `impl Trait for Device` statements) — so even the *mechanism* v1 used to check this is now stale, not just unspecified. This RFC must both formalize the generic-parameter system and rewire its trait-bound checking to look up RFC-003's free-standing `impl` statements instead of a device's own (now nonexistent) trait list.

Who this is for: **std-library and device/fn authors** (human or AI) writing anything generic — which per the examples already accepted, is nearly everything of substance. Secondarily, this closes the last named open dependency from RFC-004, letting RFC-011 (error-code registry) finalize.

## Goals

- Formally specify what a generic parameter is: a **unit-type parameter** (bound by one of RFC-001's ten unit types, e.g. `C: Capacitance`) or a **trait-bound type parameter** (bound by one or more traits, e.g. `D: Capacitor` or `D: Capacitor + Polarized`).
- Specify visible-default syntax for unit-type parameters (`V: Voltage = 10V`), consistent with the Constitution's "no magic defaults" principle.
- Specify exactly how a trait-bound type parameter is checked against a concrete type argument, at the point of a generic instantiation (`device<...>` or `fn::<...>(...)`) — rewired to check against **free-standing **`impl Trait for Device`** statements** (RFC-003), not a device's own embedded trait list (which no longer exists).
- Unify v1's two separate mechanisms (generic type parameter bounds, and `impl Trait`-typed value parameters) into one, since they are the same concept — a function parameter typed `impl Trait` is sugar for an anonymously-named generic type parameter bound by that trait, exactly as in Rust.

## Non-goals

- **Not adding associated types, default methods, or blanket impls to the generic system.** These were considered and explicitly withdrawn during RFC-003's design process as solving a different problem than the one raised; this RFC does not reopen that question.
- **Not adding const-generics or value-level generic parameters beyond unit-typed specs.** A generic parameter is either a unit type or a trait bound — not an arbitrary compile-time value/expression parameter.
- **Not solving generic-parameter variance, higher-kinded generics, or where-clauses beyond simple trait-bound lists.** CoHDL's generic system is deliberately narrow — proportional to a schematic-capture DSL's actual needs, not a general-purpose type system.

## Design

### Two kinds of generic parameter — unit-type and trait-bound

```cohdl
// Unit-type parameter: bound by one of RFC-001's ten unit types, optional visible default
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}

// Trait-bound type parameter: bound by one or more traits (space via `+`)
fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A   // `target`'s pins/specs are known through D's trait bound
}

fn add_protected_decoupling<D: Capacitor + Polarized>(target: D, pin: Pin) {
    // D must satisfy BOTH bounds — both must have a free-standing impl for the concrete type
}
```

- A **unit-type parameter** (`C: Capacitance`) may only be substituted with a literal or another unit-type parameter of the *same* unit type (RFC-001's zero-coercion rule applies identically to generic substitution — no exception for generics).
- A **trait-bound type parameter** (`D: Capacitor`) may only be substituted with a concrete device type that has a satisfying free-standing `impl Capacitor for ThatDevice` statement **somewhere in scope** at the point of instantiation.
- Multiple trait bounds (`D: Capacitor + Polarized`) require the concrete type argument to have a satisfying `impl` for **every** listed trait, each independently looked up.
- Defaults (`V: Voltage = 10V`) are only valid on unit-type parameters, always visible in source at the parameter declaration — never a hidden fallback introduced elsewhere.

### Trait-bound checking at instantiation — rewired to RFC-003's free-standing impls

The actual mechanism this RFC delivers: when a generic type parameter bound by trait(s) is instantiated with a concrete type argument, the compiler checks — for each required trait — whether a satisfying `impl RequiredTrait for ConcreteType` statement exists anywhere in the currently-compiled scope:

```cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V> {
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V }
}
impl TwoTerminal for MLCC {}
impl Capacitor for MLCC {}

fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A
}

design Board {
    inst c1: MLCC<100nF, 16V>
    add_decoupling::<MLCC>(c1, some_pin)   // OK: impl Capacitor for MLCC exists in scope
}
```

If no satisfying `impl` exists for the concrete type argument, this is a compile error at the call/instantiation site, naming the missing trait and the concrete type — the same diagnostic discipline RFC-003 established for device-level `impl` checking, just triggered from the generic-instantiation side instead of the device-declaration side.

### Unifying `impl Trait`-typed value parameters with generic type parameters

v1 had a second, separate mechanism: a function parameter written `param: impl Trait` (not a generic type parameter, but a *value* parameter whose declared type is expressed as a trait bound). This RFC treats this as **pure sugar for an anonymous generic type parameter**, exactly as Rust does:

```cohdl
// These two are equivalent — the compiler desugars the first into the second:
fn add_decoupling(target: impl Capacitor, pin: Pin) { ... }
fn add_decoupling<__Anon0: Capacitor>(target: __Anon0, pin: Pin) { ... }
```

This means there is exactly **one** trait-bound-checking mechanism in the compiler (the generic-parameter one, rewired above), not two independent code paths that could silently diverge or be maintained inconsistently — directly addressing the "two mechanisms doing overlapping jobs" smell v1's real source exhibited.

## Type-system-first test

N/A — this RFC is entirely a type-system mechanism (generic parameter resolution and trait-bound checking), not a `rule`/DRC proposal.

## Conceptual impact

Formalizes generic parameters as an explicit part of **Device**, **Trait**, and **Fn** (all existing concepts, per note 2) — no new core concept invented. The unification of value-parameter `impl Trait` syntax into generic type parameters is a simplification (one mechanism instead of two), not an addition. This RFC's main conceptual contribution is closing a gap: previously "generic parameter" was used informally across four RFCs without ever being pinned down as its own concept with its own rules.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | High | High | Low | N/A (pre-launch) | High |

**Oracle (High):** this is the mechanism that finally closes RFC-004's flagged E004 gap — a real strengthening of correctness, not cosmetic.
**Diagnostics (High):** trait-bound-at-instantiation failures will be common during early std-library authoring (any generic `fn`/device call with a mismatched type argument fires here), so message precision matters immediately, same as RFC-003's device-level equivalent.
**Grammar (Med):** trait-bound generic parameter syntax (`D: Capacitor + Polarized`) and the `impl Trait`-as-parameter-type sugar are both small, regular additions — no fundamentally new grammar shape, since bounded generics already appear informally in four accepted RFCs' examples.
**Concepts (Low):** no new core concept; this is formalization + unification of what already existed informally/redundantly.

## Gradeability

Enforced at **type-check time**, at the point a generic type parameter with a trait bound is substituted with a concrete type argument (a `device<...>` instantiation or a `fn::<...>(...)` call) — the earliest possible stage, consistent with every prior P0 RFC's discipline. This directly reuses RFC-003's `impl`-lookup mechanism rather than inventing a parallel one, and directly retires v1's redundant second mechanism (`check_call`'s separate `impl Trait`-parameter path), so there's exactly one code path to keep honest going forward.

## AI-generatability

High. A model writing a generic `fn` or device already needs to know a trait's requirements (from RFC-003); this RFC only adds the rule "the concrete type argument needs a satisfying `impl` in scope," which is the same mental model as device-level trait satisfaction, just applied at a call site instead of a declaration site — no new conceptual category to hold in mind. Unifying `impl Trait`-parameter sugar with generic type parameters also means a model only needs to learn one trait-bound-checking rule, not two subtly different ones (closing a real generatability risk v1's two-mechanism design created: a model could plausibly get the two forms' error behavior confused).

## Alternatives

- **Leave the generic-parameter/trait-bound system informally specified**, relying on the examples already used in RFC-001/002/003/006 — rejected: this is exactly the "unspecified but load-bearing" gap the note-10 "partially specified" flag was called out for; formalizing it removes ambiguity for both implementers and future RFC authors.
- **Keep v1's two separate mechanisms** (generic type parameter bounds and `impl Trait`-typed value parameters as genuinely distinct features) — rejected: they're the same concept, and maintaining two independent code paths for the same trait-bound-checking logic is exactly the kind of redundant-mechanism risk note 4's "prefer extending an existing concept over inventing a parallel mechanism" principle warns against.
- **Check trait bounds against a device's own declared traits list** (as v1 did via `dev.impl_traits`) — rejected outright: that field no longer exists in v2's device declaration (RFC-003 removed it in favor of free-standing `impl` statements); this RFC must check against `impl` statements in scope, not resurrect a removed field.
- **Add const-generics / value-parameter generics** — rejected for v1 of this RFC: no concrete motivating use case yet; CoHDL's generic system stays narrow (unit-type or trait-bound only) until a real need for more surfaces.

## Compatibility

N/A — pre-launch, no existing `.cohdl` source to break. Note: this RFC's design is consistent with (does not contradict) the generic syntax already used in RFC-001/002/003/006's examples — those examples remain valid under this RFC's formal rules.

## Tooling & operations

- A trait-bound-at-instantiation failure diagnostic must name the specific missing trait and the concrete type argument (e.g. "`add_decoupling::<Resistor>(...)`: `Resistor` does not implement `Capacitor`"), matching RFC-003's precision discipline.
- The LSP should offer hover on any generic parameter showing its full bound (unit type, or the full trait list for a trait-bound parameter), and on any `impl Trait`-typed value parameter, show its desugared generic-parameter form for clarity.
- Reserve an error-code sub-block for generic-instantiation trait-bound failures, distinct from RFC-003's device-declaration-level `impl` failures (they're checked at different sites and should be independently identifiable in diagnostics/tooling) — final numbering deferred to RFC-011, which this RFC unblocks.

## Teaching cost

Low-to-medium. An author already using generics informally (as every example in RFC-001/002/003/006 already does) needs one new formal rule: a trait-bound type parameter's concrete argument must have a satisfying `impl` in scope, checked at the call site — directly analogous to Rust generics, likely familiar to anyone with that background. The `impl Trait`-as-parameter-sugar unification is a simplification, not an addition to what must be learned.

## Failure modes

- **A model instantiates a generic with a concrete type that doesn't have any **`impl`** at all for the required trait** (not even wrong, just entirely un-implemented) — must produce the same precise diagnostic as a wrong-trait mismatch, not a different/confusing error class.
- **A model writes **`impl Trait`**-typed parameter syntax expecting different behavior from a named generic type parameter** — must not happen, since this RFC makes them strictly equivalent by desugaring; any implementation that lets them diverge is a bug against this RFC's own design.
- **Unit-type parameter substituted with a value of the wrong unit type** — already prevented by RFC-001's zero-coercion rule; this RFC doesn't weaken that in any way for the generic case.
- **A trait-bound check silently reads a stale/removed device-level trait list** (the literal v1 bug this RFC must not reintroduce) — must be explicitly guarded against in implementation: there is no `impl_traits` field on `Device` in v2 at all (removed by RFC-003), so this failure mode should be structurally impossible, not just avoided by discipline.

## Migration path

N/A — pre-launch.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-016 (see note 7). This closes RFC-004's flagged E004 dependency and unblocks RFC-011 (error-code registry) to proceed with a complete classification. This is the last P0 RFC in the redesign's critical path (note 6's backlog) — with RFC-001 through RFC-007 all Accepted, the Layer-1 type-system foundation the redesign's central thesis depends on is now formally specified. Language Specification (note 10) will replace its "Generics-over-specs (partially specified)" section with a fully-specified one reflecting this RFC.
