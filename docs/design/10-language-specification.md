# 10. Language Specification

# Status

**Living document — reflects only what has been formally Accepted via RFC (note 6).** This is not a design-rationale document (that's note 2, Conceptual Model, and the RFCs/decision records in notes 6–7) — it is the compiled, current statement of *what the language actually is*, organized the way a reference manual is organized: by construct, so an author (human or AI) or a new RFC proposer can look up "how does X work today" without reading RFC history.

**Update discipline (see note 6, RFC lifecycle step 6):** every Accepted RFC must update this note in the same change that accepts it. If a construct is Accepted but not reflected here, treat that as an open defect against the RFC, not a documentation backlog item — this mirrors the exact discipline that fixed v1's "dormant DRC rule" problem, applied to documentation instead of compiler checks.

**How to read this note:** each section is a construct. Each entry states current, accepted behavior only — no proposals, no "planned," no v1 carryover. A construct not yet covered by an Accepted RFC simply doesn't appear here yet (see "Not yet specified" at the bottom for an honest list of what's still open).

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

Every pin declared on a `device` or `trait` carries an explicit **connection-obligation kind**, fixed at the definition and never overridable per-instance:

- `required` — must be resolved (see below) before any design containing an instance of this device can type-check.
- `optional` — may be left unmentioned entirely; needs no `net` and no `nc` declaration.

**Resolving a **`required`** pin** — exactly one of two states, checked exhaustively:

1. **Connected** — the pin reference appears in some `net` declaration (unchanged mechanism).
2. **Explicitly not-connected** — the pin reference appears in an `nc` declaration, a top-level design-body construct syntactically parallel to `net` (a flat pin-reference list), but semantically its opposite:

**Rules:**

- A `required` pin appearing in **neither** `net` nor `nc` is a compile error (unresolved).
- A `required` pin appearing in **both** `net` and `nc` is a compile error (contradictory).
- The exhaustiveness check runs once, at final `design` assembly, after all `fn` inlining/monomorphization — it does not fire inside an unassembled `fn` body, so a sub-circuit may intentionally leave pins for its caller to resolve.
- `nc` is never a second connectivity mechanism — a pin listed under `nc` joins no net.

*Note: this obligation-aware exhaustiveness check is the sole replacement for v1's blanket "any unconnected pin" warning (see Residual DRC below — that v1 rule does not carry forward as DRC; it's retired, superseded by this stricter, more correct mechanism).*

# Traits

*Accepted via RFC-003, see RFC-003: Trait-satisfaction-at-impl-time checking + DR-013.*

**Traits and devices are declared completely independently of each other.** A `device` never has a trait clause — it is only pins + specs, self-contained:

- `pins { ... }` on a trait declares **abstract pin roles** (not physical pin numbers) with an obligation kind (`required`/`optional`, per RFC-002).
- `spec { ... }` on a trait declares required spec fields, each with a unit type (per RFC-001's closed unit-type set).
- `Trait: OtherTrait` is a **sub-trait bound** — implementing the child trait requires satisfying the parent trait's requirements too, transitively.

`impl Trait for Device`** is its own free-standing top-level statement** — never embedded in either the trait's or the device's own declaration. It can appear anywhere in scope: the same module as the device, a different module, or a module dedicated to grouping implementations by trait.

**Satisfaction is checked by matching the trait's required names against the device's own already-declared pin/spec names, by name:**

When a device's own field names don't match the trait's required roles, the `impl` body contains an explicit mapping (the only thing an `impl` body ever contains):

**Rules:**

- Each `impl Trait for Device` is checked exhaustively the moment it is written: every pin role and spec field the trait (and its sub-trait bounds, transitively) requires must resolve — by name-match against the device's own pins/specs, or by explicit mapping if names differ — to a compatible device pin (matching obligation kind) or spec field (matching unit type, or matching generic parameter bound for generic devices).
- A device may have any number of separate `impl` blocks, one per trait, added at any time, anywhere in scope.
- A sub-trait bound requires a **separate satisfying **`impl`** for that sub-trait to exist somewhere in scope** — e.g. `impl Capacitor for MLCC` requires an `impl TwoTerminal for MLCC` to exist (it can be a different statement, anywhere).
- There is no partial `impl` — satisfaction is exhaustive, the same discipline RFC-002 established for pin connectivity, applied here to trait contracts.
- Explicit `impl` is always required — CoHDL does not use structural typing; a device is never considered to satisfy a trait just because its shape happens to match, without an explicit `impl` statement asserting it.
- CoHDL traits declare data-shape requirements only (pins, specs) — they do not have methods/behavior in the Rust sense. `rule` blocks remain the separate, existing mechanism for behavior/assertions.

*Note: the generic-parameter trait-bound case (e.g. *`fn foo<D: Capacitor>(...)`* — does the concrete type argument for *`D`* satisfy *`Capacitor`*?) is distinct from device-level *`impl`* checking above, and is not yet fully specified — see RFC-004's classification (Residual DRC below) and note 6's backlog: RFC-007 (generics-over-specs) must close this gap explicitly.*

# Designators

Accepted via RFC-005, see RFC-005: Collision-free designator allocation + DR-008.

This section documents guarantees, not new syntax — designator assignment is automatic and has no .cohdl source-level construct of its own (beyond the existing #[designator("Xxx")] override attribute and design.lock, both unchanged in shape from v1). What v2 changes is the allocation algorithm's correctness guarantee, worth documenting because authors and reviewers rely on it:

- No two live instances in a design ever receive the same designator. This is enforced as an explicit, checked postcondition on every compilation — not merely assumed from the algorithm's design.
- Designators are stable across rebuilds. A hierarchical path that already has an assignment in design.lock keeps it, regardless of source-order changes elsewhere in the design.
- Removed instances are tombstoned (moved to design.lock's [tombstones] table) — their designator is never reused by a future fresh assignment.
- Explicit #[designator("Xxx")] overrides are resolved before any fresh (automatic) designator is assigned — an override can never be silently clobbered by, or silently clobber, an automatic assignment.
- Two new instances needing the same prefix (e.g. both defaulting to U) are guaranteed different numbers, deterministically, regardless of the order they were declared or collected in — this closes the specific collision class v1 exhibited (two devices both assigned U3).

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

- v1's spec-not-satisfied / trait-not-impl / missing-spec-field checks (v1: E003–E005) — now `impl`-time trait satisfaction (see Traits above). E004's generic-parameter-bound half is not yet fully closed — pending RFC-007.
- v1's blanket unconnected-pin warning (v1: W001) — retired. Superseded by the pin connection-obligation exhaustiveness check (see Pins above), which is strictly more correct (it doesn't flag intentionally-optional pins the way v1's blanket rule did).
- v1's floating-net check (v1: W002) — reclassified to the type system: a `net` declaration naming zero instance pins is checkable from that declaration alone, so it becomes a compile-time structural check rather than a DRC rule. (Concrete syntax/mechanism for this is pending whatever RFC formalizes `net`/`design` type-checking rules in full.)

# Sub-circuit fns

Accepted via RFC-006, see RFC-006: Nested fn call semantics + DR-015.

A fn is a reusable circuit fragment — parameterized (including generically), instantiated and wired by calling it. fn calls may nest to arbitrary depth: a fn's own body may call other fns, exactly as if those nested calls had been written at the top level of the calling design.

Rules:

- A nested call expands with the exact same instantiate-and-wire semantics as a top-level call — every inst and net in the nested fn's body is produced, exactly as if it had been called directly.
- Generic substitutions thread outward-in: a nested call's generic arguments may reference the calling fn's own generic parameters (as in power_rail's V flowing into decoupling_cap:: above), and are fully resolved to concrete types before the innermost fn is expanded.
- Every produced instance/net is named from its full call-chain path, guaranteeing no collision between different call sites — including two separate calls to the same fn, or the same fn reached via different nesting paths.
- Cyclic call chains are a compile error. If expanding a call would re-enter a fn already active earlier in the same chain, this is rejected with a diagnostic naming the full cycle — never a silent infinite/truncated expansion.
- There is no depth limit on (acyclic) nesting.

# Generics-over-specs (partially specified)

Not yet fully accepted — see note 6's backlog, RFC-007. Generic parameters on device/fn declarations (e.g. MLCC<C: Capacitance, V: Voltage = 10V>, fn decoupling_cap<V: Voltage>(...)) are used throughout this note's examples and are load-bearing for RFC-001/002/003/006 as already accepted — but the full generic system (including how a generic parameter's trait bound is checked against a concrete type argument at a call/instantiation site — the "E004" gap flagged in Residual DRC above) has not yet been formally specified by its own RFC. Treat the generic syntax shown elsewhere in this note as provisionally stable (it's already load-bearing for four Accepted RFCs) but not yet a fully closed, standalone specification.

# Not yet specified

The following constructs are referenced conversationally (in the Conceptual Model, note 2, or in v1-legacy context) but have **no Accepted RFC yet**, and therefore no entry above. Do not assume any specific syntax for these until an RFC lands:

- Generics-over-specs full specification, including closing the E004 generic-bound trait-check gap flagged in Residual DRC above (RFC-007)
- Exhaustive pattern-matching over structural variants (RFC-008)
- `cohdl fmt` canonical form (RFC-009)
- `cohdl check --json` schema (RFC-010)
- Error-code registry (RFC-011)
- `#[intent(...)]` annotations (RFC-012)
- Everything else in the Conceptual Model (Part, Instance, Net, Module, Design) that hasn't yet had its concrete syntax/semantics pinned down by an Accepted RFC — note 2 describes their intended shape and philosophy; this note will gain a section for each only once an RFC formally accepts its concrete syntax.

This section should shrink over time as RFCs land — treat its length as an honest progress indicator for the redesign, not a to-do list to rush.
