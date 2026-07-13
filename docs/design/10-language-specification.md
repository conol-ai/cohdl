# 10. Language Specification

# Status

**Living document — reflects only what has been formally Accepted via RFC (note 6).** This is not a design-rationale document (that's note 2, Conceptual Model, and the RFCs/decision records in notes 6–7) — it is the compiled, current statement of *what the language actually is*, organized the way a reference manual is organized: by construct, so an author (human or AI) or a new RFC proposer can look up "how does X work today" without reading RFC history.

**Update discipline (see note 6, RFC lifecycle step 6):** every Accepted RFC must update this note in the same change that accepts it. If a construct is Accepted but not reflected here, treat that as an open defect against the RFC, not a documentation backlog item — this mirrors the exact discipline that fixed v1's "dormant DRC rule" problem, applied to documentation instead of compiler checks.

**How to read this note:** each section is a construct. Each entry states current, accepted behavior only — no proposals, no "planned," no v1 carryover. A construct not yet covered by an Accepted RFC simply doesn't appear here yet (see "Not yet specified" at the bottom for an honest list of what's still open).

Milestone (2026-07-13): all seven P0 RFCs Accepted, MVP implemented and verified on the real conol-ai/cohdl main branch (65 tests, self-audited compliance report, KiCad-verified demo). RFC-008 (structural variants) is the first RFC drafted in response to a real implementation need, not just design foresight.

# Unit types

*Accepted via RFC-001, see RFC-001: Units-as-types + DR-007.*

CoHDL has a **closed set of ten primitive unit types**. Each is a distinct type; there is zero implicit coercion between unit types or from a bare number.

| Unit type | Canonical symbol | SI prefixes allowed | Signed? | Example literals |
|---|---|---|---|---|
| `Voltage` | `V` | standard (`p n u m k M G` as applicable) | no | `3.3V`, `5V` |
| `Capacitance` | `F` | `p n u` (typical range) | no | `100nF`, `10uF` |
| `Resistance` | `ohm` (ASCII only — never `Ω`) | standard | no | `10kohm`, `330ohm` |
| `Current` | `A` | standard | no | `500mA`, `2A` |
| `Frequency` | `Hz` | `k M G` (typical range) | no | `16MHz`, `32kHz` |
| `Time` | `s` | `p n u m` (typical range) | no | `10ms`, `1us` |
| `Inductance` | `H` | `p n u m` (typical range) | no | `10uH`, `100nH` |
| `Power` | `W` | `u m k` (typical range) | no | `250mW`, `1W` |
| `Temperature` | `C` (ASCII only — never `°C`) | **none** | **yes** — sole type allowing a leading `-` | `85C`, `-40C` |
| `Tolerance` | `%` | **none** | no | `1%`, `0.5%` |

**Literal syntax:** a number immediately followed by its unit symbol, no space (`100nF`, not `100 nF`). The grammar defines a fixed table of (unit × allowed prefix); a model or author should not assume a prefix is valid for a unit unless it appears in the table above.

**Rules:**

- A bare number where a unit-typed spec/field is expected is a compile error — never silently accepted, never defaulted.
- No arithmetic between unit types (`10V + 5A` does not parse).
- Comparison operators (`<=`, `>=`, `==`, …) inside `rule` blocks are valid only between two values of the *same* unit type.
- `Temperature` and `Tolerance` take no SI prefix at all — an SI-prefixed `Temperature`/`Tolerance` literal (e.g. `1mC`) is a grammar error, not a scaled value.
- `Temperature` is the only unit type whose literal may carry a leading `-`.

**Closed set:** extending beyond these ten types requires a new RFC (see note 6's backlog for the extension path); this is a deliberate scope boundary, not an oversight.

# Pins

*Accepted via RFC-002, see RFC-002: Pin connection-obligation typing + DR-012.*

Every pin declared on a device or trait carries an explicit connection-obligation kind, fixed at the definition and never overridable per-instance. Every pin (on a device) also carries an explicit role annotation (see Structural variants below — no unannotated pins, per RFC-008):

```cohdl
pub device MCU_ESP32S3 {
    pins {
        required VDD: 1 [power_in]
        required GND: 2, 3, 4 [power_in]        // pin bus — all required
        optional NC_1: 5 [passive]              // datasheet-defined no-connect
        optional NC_2: 6 [passive]
    }
}
```

- `required` — must be resolved (see below) before any design containing an instance of this device can type-check.
- `optional` — may be left unmentioned entirely; needs no `net` and no `nc` declaration.

**Resolving a **`required`** pin** — exactly one of two states, checked exhaustively:

1. **Connected** — the pin reference appears in some `net` declaration (unchanged mechanism).
2. Explicitly not-connected — the pin reference appears in an nc declaration, a top-level design-body (or fn-body, per implementation) construct syntactically parallel to net (a flat pin-reference list), but semantically its opposite:

```cohdl
net VDD: mcu.VDD, ...
net GND: mcu.GND, ...

nc: mcu.RTC_XTAL_IN, mcu.RTC_XTAL_OUT
```

**Rules:**

- A `required` pin appearing in **neither** `net` nor `nc` is a compile error (unresolved).
- A `required` pin appearing in **both** `net` and `nc` is a compile error (contradictory).
- The exhaustiveness check runs once, at final `design` assembly, after all `fn` inlining/monomorphization — it does not fire inside an unassembled `fn` body, so a sub-circuit may intentionally leave pins for its caller to resolve.
- `nc` is never a second connectivity mechanism — a pin listed under `nc` joins no net.

*Note: this obligation-aware exhaustiveness check is the sole replacement for v1's blanket "any unconnected pin" warning (see Residual DRC below — that v1 rule does not carry forward as DRC; it's retired, superseded by this stricter, more correct mechanism).*

# Traits

*Accepted via RFC-003, see RFC-003: Trait-satisfaction-at-impl-time checking + DR-013.*

**Traits and devices are declared completely independently of each other.** A `device` never has a trait clause — it is only pins + specs, self-contained:

```cohdl
pub trait TwoTerminal {
    pins {
        required A: pin
        required B: pin
    }
}

pub trait Capacitor: TwoTerminal {
    spec {
        capacitance: Capacitance
        voltage_rating: Voltage
        tolerance: Tolerance
    }
}

pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}
```

- pins { ... } on a trait declares abstract pin roles (not physical pin numbers) with an obligation kind (required/optional, per RFC-002) — trait-level pins do not carry the RFC-008 electrical role annotation (that's a device-pin concept, since abstract trait roles aren't physical pins yet).
- `spec { ... }` on a trait declares required spec fields, each with a unit type (per RFC-001's closed unit-type set).
- `Trait: OtherTrait` is a **sub-trait bound** — implementing the child trait requires satisfying the parent trait's requirements too, transitively.

`impl Trait for Device`** is its own free-standing top-level statement** — never embedded in either the trait's or the device's own declaration. It can appear anywhere in scope: the same module as the device, a different module, or a module dedicated to grouping implementations by trait.

**Satisfaction is checked by matching the trait's required names against the device's own already-declared pin/spec names, by name:**

```cohdl
impl TwoTerminal for MLCC {}   // MLCC already has pins A/B — names match, body empty
impl Capacitor for MLCC {}     // MLCC already has capacitance/voltage_rating/tolerance — empty
```

When a device's own field names don't match the trait's required roles, the `impl` body contains an explicit mapping (the only thing an `impl` body ever contains):

```cohdl
pub device TantalumCap<C: Capacitance, V: Voltage = 10V> {
    pins { Anode: 1 [passive], Cathode: 2 [passive] }
    spec { capacitance: C, voltage_rating: V }
}

impl TwoTerminal for TantalumCap {
    pins { A: Anode, B: Cathode }   // names differ — explicit mapping
}
```

**Rules:**

- Each `impl Trait for Device` is checked exhaustively the moment it is written: every pin role and spec field the trait (and its sub-trait bounds, transitively) requires must resolve — by name-match against the device's own pins/specs, or by explicit mapping if names differ — to a compatible device pin (matching obligation kind) or spec field (matching unit type, or matching generic parameter bound for generic devices).
- A device may have any number of separate `impl` blocks, one per trait, added at any time, anywhere in scope.
- A sub-trait bound requires a **separate satisfying **`impl`** for that sub-trait to exist somewhere in scope** — e.g. `impl Capacitor for MLCC` requires an `impl TwoTerminal for MLCC` to exist (it can be a different statement, anywhere).
- There is no partial `impl` — satisfaction is exhaustive, the same discipline RFC-002 established for pin connectivity, applied here to trait contracts.
- Explicit `impl` is always required — CoHDL does not use structural typing; a device is never considered to satisfy a trait just because its shape happens to match, without an explicit `impl` statement asserting it.
- CoHDL traits declare data-shape requirements only (pins, specs) — they do not have methods/behavior in the Rust sense. `rule` blocks remain the separate, existing mechanism for behavior/assertions.

# Designators

Accepted via RFC-005, see RFC-005: Collision-free designator allocation + DR-008.

This section documents guarantees, not new syntax — designator assignment is automatic and has no .cohdl source-level construct of its own (beyond the existing #[designator("Xxx")] override attribute and design.lock, both unchanged in shape from v1). What v2 changes is the allocation algorithm's correctness guarantee, worth documenting because authors and reviewers rely on it:

- No two live instances in a design ever receive the same designator. This is enforced as an explicit, checked postcondition on every compilation — not merely assumed from the algorithm's design.
- Designators are stable across rebuilds. A hierarchical path that already has an assignment in design.lock keeps it, regardless of source-order changes elsewhere in the design.
- Removed instances are tombstoned (moved to design.lock's [tombstones] table) — their designator is never reused by a future fresh assignment.
- Explicit #[designator("Xxx")] overrides are resolved before any fresh (automatic) designator is assigned — an override can never be silently clobbered by, or silently clobber, an automatic assignment.
- Two new instances needing the same prefix (e.g. both defaulting to U) are guaranteed different numbers, deterministically, regardless of the order they were declared or collected in — this closes the specific collision class v1 exhibited (two devices both assigned U3).
- A device's designator prefix comes from designator_prefix: "X" declared on a trait it implements (the lexicographically-smallest such trait name, if it implements several with a prefix); default "U" when none declares one.

# Residual DRC

*Accepted via RFC-004, see RFC-004: DRC/type-system reclassification pass + DR-014.*

`rule` (DRC) in v2 is deliberately narrow — reserved only for checks that are genuinely emergent from the whole connectivity graph, never for anything structural a device/trait/`impl` declaration could settle on its own (that belongs in the type system, per the mechanisms above). Auditing every check the discarded v1 implementation had, the following four (mapped from v1's rule names, not literal ports) are the complete v2 residual-DRC surface for now:

| Residual DRC concern | What it checks | Why it stays DRC |
|---|---|---|
| Voltage-exceed (v1: E001) | An instance's `voltage_rating` spec is less than the voltage observed on the net it's connected to. | Net voltage is emergent from every pin connected to that net — not knowable from one device's declaration alone. |
| Polarity-mismatch (v1: E002) | A device satisfying `Polarized` has its anode pin connected to a GND-annotated net. | "Is this net GND," and what else is connected to it, are net-graph-level facts. (Whether the device *is* `Polarized` is now a compile-time fact via `impl`, per Traits above — only the net-level part of this check stays DRC.) |
| Single-driver (v1: W003) | A net has exactly one output/driver-type pin connected — likely unfinished wiring. | Requires knowing every pin on the net and each one's driver role — whole-graph-emergent. |
| Multi-driver (v1: W004) | A net has more than one output/driver-type pin connected. | Same reasoning as single-driver. |

**What is explicitly NOT residual DRC in v2** (retired or moved to the type system — see Traits and Pins above for where each now lives):

- v1's spec-not-satisfied / trait-not-impl / missing-spec-field checks (v1: E003–E005) — now impl-time trait satisfaction (see Traits above), plus generic-instantiation trait-bound checking (see Generics below) for the parts of E004 that involve a generic parameter rather than a device declaration.
- v1's blanket unconnected-pin warning (v1: W001) — retired. Superseded by the pin connection-obligation exhaustiveness check (see Pins above), which is strictly more correct (it doesn't flag intentionally-optional pins the way v1's blanket rule did).
- v1's floating-net check (v1: W002) — reclassified to the type system: a net declaration naming zero instance pins (after expansion) is a compile-time structural error, not a DRC rule.

# Sub-circuit fns

Accepted via RFC-006, see RFC-006: Nested fn call semantics + DR-015.

A fn is a reusable circuit fragment — parameterized (including generically), instantiated and wired by calling it. fn calls may nest to arbitrary depth: a fn's own body may call other fns, exactly as if those nested calls had been written at the top level of the calling design.

```cohdl
fn decoupling_cap<V: Voltage>(pin: Pin) {
    inst c: MLCC<100nF, V>
    net _: pin, c.A
}

fn power_rail<V: Voltage>(vdd_pin: Pin) {
    inst ferrite: Ferrite_Bead
    net _: vdd_pin, ferrite.IN
    decoupling_cap::<V>(ferrite.OUT)   // nested call — fully supported
}

design Board {
    inst mcu: MCU_ESP32S3
    power_rail::<3.3V>(mcu.VDD)
}
```

Rules:

- A nested call expands with the exact same instantiate-and-wire semantics as a top-level call — every inst and net in the nested fn's body is produced, exactly as if it had been called directly.
- Generic substitutions thread outward-in: a nested call's generic arguments may reference the calling fn's own generic parameters, fully resolved to concrete types before the innermost fn is expanded.
- Every produced instance/net is named from its full call-chain path, guaranteeing no collision between different call sites — including two separate calls to the same fn, or the same fn reached via different nesting paths. The __-prefixed namespace this naming scheme uses is compiler-reserved — a user-declared instance/net name beginning with __ is a compile error, so the guarantee can't be forged.
- Cyclic call chains are a compile error. If expanding a call would re-enter a fn already active earlier in the same chain, this is rejected with a diagnostic naming the full cycle — never a silent infinite/truncated expansion.
- There is no depth limit on (acyclic) nesting.
- Calls use name::<generic-args>(args) (turbofish) when the fn has generic parameters, or name(args) otherwise. Generic arguments are positional.

# Generics-over-specs and generic trait bounds

Accepted via RFC-007, see RFC-007: Generics-over-specs and generic trait bounds + DR-016.

A generic parameter on a device or fn is one of exactly two kinds:

- Unit-type parameter — bound by one of RFC-001's ten unit types, with an optional visible default: C: Capacitance, V: Voltage = 10V.
- Trait-bound type parameter — bound by one or more traits, joined with +: D: Capacitor, D: Capacitor + Polarized.

```cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    pins { A: 1 [passive], B: 2 [passive] }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}
impl Capacitor for MLCC {}

fn add_decoupling<D: Capacitor>(target: D, pin: Pin) {
    net _: pin, target.A
}

fn add_protected_decoupling<D: Capacitor + Polarized>(target: D, pin: Pin) {
    // D must satisfy BOTH bounds
}
```

Trait-bound checking at instantiation — when a trait-bound generic parameter is substituted with a concrete type argument (a device<...> instantiation or a fn::<...>(...) call), the compiler checks, for each required trait, whether a satisfying free-standing impl RequiredTrait for ConcreteType statement (per Traits above) exists anywhere in the currently-compiled scope. If not, this is a compile error at the call/instantiation site naming the missing trait and the concrete type.

impl Trait-typed value parameters are sugar for an anonymous trait-bound generic type parameter — these two are equivalent, and the compiler treats them identically (one trait-bound-checking mechanism, not two).

Rules:

- A unit-type parameter may only be substituted with a literal or another unit-type parameter of the same unit type — RFC-001's zero-coercion rule applies identically to generic substitution, no exception.
- A trait-bound type parameter's concrete argument must have a satisfying impl for every listed trait bound, each independently checked.
- Defaults are only valid on unit-type parameters, and are always visible at the parameter declaration — never a hidden fallback introduced elsewhere.
- Trait-bound checking never looks up a device's own declared-traits list (no such list exists — see Traits above) — it always looks up free-standing impl statements in scope.

# Structural variants

Accepted via RFC-008, see RFC-008: Exhaustive pattern-matching over structural variants + DR-017.

Two closed, exhaustively-matched sets, replacing implicit defaults with explicit, compiler-checked coverage.

Pin roles — every device pin (not trait-abstract pins) carries an explicit role annotation from a closed six-value set:

```cohdl
pub device AP2112K_3V3 {
    pins {
        required VIN:  1 [power_in]
        required GND:  2 [power_in]
        required EN:   3 [input]
        optional NC:   4 [passive]     // explicit — no unannotated pins
        required VOUT: 5 [power_out]
    }
}
```

- Valid roles: input, output, bidirectional, passive, power_in, power_out.
- Every pin must have an explicit role — there is no default. An unannotated pin is a compile error listing the six valid roles.
- Driver-type roles (consumed by residual DRC's multi-driver rule): output, power_out.

Package/footprint variants — a device may declare a closed, finite set of structural shapes, each requiring its own pin layout:

```cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    variants { C0402, C0603, C0805 }

    pins[C0402] { A: 1 [passive], B: 2 [passive] }
    pins[C0603] { A: 1 [passive], B: 2 [passive] }
    pins[C0805] { A: 1 [passive], B: 2 [passive] }

    spec { capacitance: C, voltage_rating: V, tolerance: T }
    spec[C0402] { max_capacitance: 100nF }   // optional variant-specific addition/override
}

inst c1: MLCC<100nF, 16V, 10%>[C0603]   // [VARIANT] selector required at instantiation
```

Rules:

- variants { ... } declares the closed set; duplicates are a compile error.
- Every declared variant must have a pins[VARIANT] block — this is the exhaustiveness check, at the device's own declaration. A variant with no pin layout is a compile error naming the missing variant.
- spec[VARIANT] { ... } is optional per variant (unlike pins[VARIANT]) — a variant needing no spec override is legitimate.
- An instance of a device with declared variants must select one via a [VARIANT] suffix — there is no implicit default variant; omitting the selector is a compile error listing the valid variant set.
- A device with no variants {} block is unaffected — plain pins { ... } (no bracket) as shown throughout this note.

# Not yet specified

The following constructs are referenced conversationally (in the Conceptual Model, note 2, or in v1-legacy context) but have **no Accepted RFC yet**, and therefore no entry above. Do not assume any specific syntax for these until an RFC lands:

- `cohdl fmt` canonical form (RFC-009)
- `cohdl check --json` schema (RFC-010)
- Error-code registry (RFC-011)
- `#[intent(...)]` annotations (RFC-012)
- Everything else in the Conceptual Model (Part, Instance, Net, Module, Design) that hasn't yet had its concrete syntax/semantics pinned down by an Accepted RFC — note 2 describes their intended shape and philosophy; this note will gain a section for each only once an RFC formally accepts its concrete syntax.

This section should shrink over time as RFCs land — treat its length as an honest progress indicator for the redesign, not a to-do list to rush.
