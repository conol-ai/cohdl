# RFC-001: Units-as-types

## Problem

Today (in the v1 implementation, now discarded) engineering values — capacitance, voltage, resistance — were bare numbers or loosely-typed spec fields. Nothing stopped a device definition or an instance from writing `capacitance: 100` where `100` could mean nF, µF, or an outright wrong unit, and nothing stopped a `voltage_rating` field from silently accepting a value meant for a different spec. This is a class of mistake with zero dedicated compiler coverage in v1 — not a dormant DRC rule, not even a planned one. It's exactly the kind of *structural* mistake the v2 redesign's central thesis says should be a compile-time type error, not a hoped-for review catch.

Who this is for: primarily the **AI author**, who benefits from an immediate, local, unambiguous error the moment it writes a wrong-shaped literal — instead of a wrong board silently compiling. Secondarily the **human reviewer**, who can trust that if a design type-checks, every engineering value is at least dimensionally sound, so review effort goes to values being *right*, not values being *nonsense*.

## Goals

- Serves the Constitution's rank-1 priority (correctness/gradeability) and the redesign's core thesis (strictness buys expressiveness) more directly than any other single mechanism — nearly every other P0 RFC (trait bounds on `Capacitor`, generics-over-specs, pin obligations expressed via spec-bearing traits) assumes unit-typed specs already exist.
- Make "wrong unit" and "bare number where a unit is required" compile errors, full stop.
- Keep the mechanism small and closed — a fixed enumerable set of unit types, not a general-purpose dimensional-analysis system.

## Non-goals

- **Not a dimensional-analysis / unit-algebra engine.** CoHDL does not compute `Voltage = Current × Resistance` or derive one unit from others. Units exist to *tag and compare* engineering values, not to run physics. If a future need for derived-unit computation appears, that is a new RFC — this one deliberately stays narrow (regularity over cleverness, ladder rank 6 <ways 2>).
- **Not a general user-extensible unit system in v1.** The unit type set is fixed and closed (see Design). User-defined units are out of scope; if std-library growth needs a new unit, that's an RFC to extend the closed set, not a language feature for arbitrary units.
- **Not solving numeric precision/rounding policy.** How literals are stored internally (fixed-point vs. float, precision loss on very large/small engineering-notation numbers) is an implementation detail for the type checker RFC/implementation phase, not scoped here.

## Design

### The closed set of unit types (v1 of the redesign, extended 2026-07-13)

A fixed, enumerable set of primitive unit types, each a distinct type in the type system:

| Unit type | Symbol | Example literal | Domain |
|---|---|---|---|
| `Voltage` | V | `3.3V`, `5V` | electrical potential |
| `Capacitance` | F | `100nF`, `10uF` | capacitance |
| `Resistance` | Ω (written `ohm` in ASCII source) | `10kohm`, `330ohm` | resistance |
| `Current` | A | `500mA`, `2A` | current |
| `Frequency` | Hz | `16MHz`, `32kHz` | frequency |
| `Time` | s | `10ms`, `1us` | time/duration |
| `Inductance` | H | `10uH`, `100nH` | inductance |
| `Power` | W | `250mW`, `1W` | power dissipation/rating |
| `Temperature` | C | `85C`, `-40C` | temperature range |
| `Tolerance` | % | `1%`, `0.5%` | tolerance/percentage |

This set is closed for v2 — extending it further beyond these ten types is a future RFC, evaluated the same way any new concept is (Coherence Matrix, conceptual cost).

**Amendment (2026-07-13):** the original closed set (six types: `Voltage`, `Capacitance`, `Resistance`, `Current`, `Frequency`, `Time`) has been extended to ten with `Inductance`, `Power`, `Temperature`, and `Tolerance`. `Inductance` and `Power` were explicitly named in the original RFC text as the two likely future candidates (inductors are as fundamental as capacitors/resistors; power dissipation/rating specs are ubiquitous). `Temperature` and `Tolerance` are added because operating-temperature ranges and component tolerance (±1%, ±5%, etc.) are just as common in real component specs as the original six. See Decision section for the amendment record.

Notes on the four new types:

- `Inductance` commonly uses `p/n/u/m` prefixes (most inductors in this domain's use cases sit in the pH–mH range).
- `Power` commonly uses `u/m/k` prefixes (µW for micro-power circuits up through kW).
- `Temperature` takes no SI prefix — it is not scaled that way. Canonical form is a signed integer/decimal directly followed by `C` (e.g. `-40C`, `85C`). This is the only unit type where negative literals are meaningful, and the grammar must allow a leading `-` for this type specifically.
- `Tolerance` takes no SI prefix and is dimensionless — canonical form is a decimal/int directly followed by `%` (e.g. `1%`, `0.5%`). It is included in the closed set (not left as a bare number) precisely because "tolerance is just a number" is exactly the kind of convention-not-type gap this RFC exists to close.

### Literal syntax

Every unit literal is a number immediately followed by an SI-prefixed unit symbol, no space, one canonical form per unit type:

```cohdl
spec {
    capacitance: 100nF
    voltage_rating: 10V
}
```

Standard SI prefixes are supported per unit type where physically meaningful (`p`, `n`, `u`, `m`, `k`, `M`, `G` — only the prefixes that make engineering sense for that unit; e.g. `Capacitance` commonly uses `p/n/u`, `Frequency` commonly uses `k/M/G`). `Temperature` and `Tolerance` take no prefix at all (see Design notes above). The **grammar defines a fixed table of (unit symbol × allowed prefixes)** — this keeps the lexer deterministic and avoids the model having to guess which prefixes are "valid enough" for a given unit (a direct application of the "deterministic grammar, no context-sensitive tricks" hard constraint).

**Resistance uses the ASCII symbol **`ohm` (not the Unicode `Ω` glyph) as the sole canonical form — this is a deliberate regularity choice: requiring a non-ASCII character in a token a model must reliably reproduce byte-for-byte is exactly the kind of "context-sensitive trap" the Constitution's grammar constraint warns about. One canonical way to write it; `Ω` is never accepted, not even as an alternate spelling (avoids the "two ways to express the same thing" model smell). The same ASCII-only principle now also governs `Temperature` (`C`, not `°C` or `°`).

### No implicit coercion — the actual strictness mechanism

- A literal's unit type is fixed at parse time from its suffix. `100nF` is a `Capacitance` value; it is never usable where a `Voltage` is expected.
- **A bare number is never valid** where a unit-typed spec is expected. `capacitance: 100` is a compile error (`E-UNIT-001`-style code, exact numbering deferred to RFC-011's registry pass) — not a warning, not a "we'll assume nF" default. This directly targets the "magic defaults" smell. This now also applies to tolerance and temperature fields, which previously might have been tempted to stay bare numbers by convention.
- No arithmetic or coercion between unit types. `10V + 5A` does not parse to anything meaningful and is rejected; this RFC does not define cross-unit operators at all (see Non-goals).
- Comparison operators (`<=`, `>=`, `==`, etc., used inside `rule` blocks for the narrowed residual-DRC checks like "net voltage ≤ rating") are defined **only between two values of the same unit type**. Comparing a `Voltage` to a `Current` is a compile error in the `rule` block itself, not a runtime DRC surprise. This extends naturally to `Power`, `Inductance`, `Temperature`, and `Tolerance` — e.g. comparing an operating-temperature spec to a tolerance percentage is a compile error, not a nonsensical DRC pass.

### Example: how this closes a real v1 gap

```cohdl
pub trait Capacitor: TwoTerminal {
    designator_prefix: "C"
    spec { capacitance: Capacitance, voltage_rating: Voltage, tolerance: Tolerance }
}

pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%>: impl Capacitor {
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V, tolerance: T }
}

inst c1: MLCC<100nF, 16V, 5%>   // fine — types match the generic bounds
inst c2: MLCC<16V, 100nF, 5>    // compile error: argument order/type mismatch AND
                                 // a bare number where Tolerance is required,
                                 // caught at monomorphization, not silently accepted
```

In v1, all three fields would have been bare-number-adjacent and any mixup was a human-review catch at best. In v2, swapping the arguments — or forgetting a unit suffix on tolerance — is a type error the moment the instance is declared.

## Type-system-first test

N/A — this RFC *is* a type-system mechanism proposal, not a `rule`/DRC proposal. (Included per the template's instruction to state this explicitly when the section doesn't apply.)

## Conceptual impact

Adds ten primitive types (`Voltage`, `Capacitance`, `Resistance`, `Current`, `Frequency`, `Time`, `Inductance`, `Power`, `Temperature`, `Tolerance`) to the type system — six at initial acceptance, four added in the 2026-07-13 amendment. This is a *foundational* conceptual addition, not a casual one — but it doesn't overlap any existing concept in the canonical vocabulary (Trait/Device/Part/Instance/Pin/Net/Spec/Rule/Module/Fn/Design/Designator); it gives **Spec** a real type system to be typed *with*, which Spec always conceptually needed. No renaming, no collision. The amendment doesn't change this analysis — it's the same mechanism applied to four more physically-real, ubiquitous-in-datasheets quantities, not a new kind of concept.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | High | Med | Low | N/A (pre-launch) | High |

Oracle (High): this is the mechanism that turns "wrong unit" from an uncovered mistake into a first-class type error — a real strengthening of what "correct" means, not a cosmetic addition.

Trust (High): a human reviewer can now trust that a type-checked design has no unit mixups anywhere, which meaningfully narrows what a review needs to focus on.

Grammar (Med): the SI-prefix table adds real grammar surface, but it's a small, fully-enumerable table, not open-ended lookahead — stays within the deterministic-PEG hard constraint. The amendment adds two prefix-less unit types (`Temperature`, `Tolerance`) and one signed-literal exception (`Temperature`), which are simple table entries, not new grammar mechanisms.

## Gradeability

Enforced entirely at **type-check time** (the earliest possible stage, per the redesign's tie-break rule): a literal's unit type is known at parse time; a mismatch against a declared spec type, a generic bound, or a comparison operator's operands is caught during type checking / monomorphization, before connectivity or DRC ever run. This is the single clearest example of "prefer the type system over DRC" in the entire backlog — there is no `rule` involved at all. The four amendment types are enforced identically.

## AI-generatability

High. The unit-literal grammar is a small, fixed, table-driven suffix system — a model doesn't need to memorize exceptions, just the (unit × prefix) table, which is documented once in the language reference. Because there's exactly one canonical spelling per unit (ASCII `ohm`, not `Ω`; ASCII `C`, not `°C`), the model never has to guess which of two spellings is "the right one" — removing an entire class of generation ambiguity. `Temperature`'s signed-literal exception and `Temperature`/`Tolerance`'s lack of prefixes are each a single documented rule, not a source of new ambiguity.

## Alternatives

- **Bare numbers + a naming convention** (e.g. `capacitance_nF: 100`) — rejected: pushes the unit into the field *name*, which is a convention, not a type; exactly the "correct-by-convention" smell the Constitution forbids.
- **A general dimensional-analysis system** (units compose via multiplication/division, e.g. `V = I * R`) — rejected for v1 of this RFC: high conceptual/implementation cost for a schematic-capture language that isn't a simulator (SPICE is an explicit non-goal); revisit only if a concrete need for derived-unit computation emerges.
- **User-extensible unit types via a **`unit`** declaration** — rejected for now: adds a new concept and grammar surface with no immediate justified use; the closed set (now ten types) covers the domain's actual current needs (passive components, timing/frequency, power, temperature, and tolerance specs). Can be proposed as its own RFC if the std library outgrows the fixed set.
- **Leaving **`Tolerance`** as a bare percentage number** (considered during the 2026-07-13 amendment) — rejected: this is exactly the "bare number where a unit is required" smell the RFC exists to eliminate; a percentage is dimensionless but still a distinct, typeable quantity that shouldn't silently coerce with, say, a raw count.

## Compatibility

N/A — pre-launch, no existing `.cohdl` source to break (per note 8's compatibility policy, nothing is a stable surface yet). The 2026-07-13 amendment is likewise compatibility-free for the same reason.

## Tooling & operations

- The (unit × allowed-prefix) table must be part of the published language reference and the LSP's completion/hover data — an AI author should be able to discover valid prefixes without trial and error. This table must be updated to include the four new types and their (no-)prefix rules.
- Unit-mismatch diagnostics must state both the expected and actual unit type by name (e.g. "expected `Voltage`, found `Capacitance`") — never a bare "type mismatch."
- This is the first entry in what will become the error-code registry (RFC-011) — reserve a code block (e.g. `E1xx`) for unit-system diagnostics specifically, so future additions to this RFC's family of checks don't collide with unrelated code ranges.

## Teaching cost

Low-to-medium. An AI-context author needs the fixed unit table (10 types, a handful of prefixes each, two of which take no prefix) — small enough to include in full in the prompt scaffold (Layer 5). A human reviewer needs to know units don't coerce — a single rule, easy to internalize, and one that immediately builds trust ("if it compiles, the units are right"). The amendment adds four more rows to memorize, not a new rule to learn.

## Failure modes

- **Model tries a plausible-but-wrong prefix** (e.g. `100pF` when `100nF` was intended) — this is a *value* mistake, not a *type* mistake, and this RFC does not catch it (units-as-types guarantees dimensional soundness, not numeric correctness — that's a different, likely un-catchable-by-compiler problem, appropriately out of scope).
- **Model reaches for the Unicode **`Ω` out of habit (trained on human-written schematics/docs that use it) — must produce a clear, specific diagnostic ("use `ohm`, not `Ω`") rather than a generic parse error, so the repair loop can fix it in one turn. The same applies to `°C`/`°` for `Temperature` — must diagnose "use `C`, not `°C`."
- **Someone tries to extend the unit set informally** (e.g. writing a spec field that's "obviously" a new unit by convention) — must fail to parse/type-check rather than silently accept an untyped bare number; if a genuine new unit is needed, that's RFC territory (see Alternatives).
- **Model forgets **`Temperature`** can be negative** and omits the `-` handling, or **tries to put an SI prefix on **`Temperature`**/**`Tolerance` (e.g. `1mC` meaning "1 milli-degree," which is nonsensical for this domain) — the grammar must reject any prefix on these two types outright, producing a clear diagnostic rather than silently parsing a bogus scaled value.

## Migration path

N/A — pre-launch.

## Decision

Accepted — 2026-07-13. Recorded as DR-007 (see note 7). This is the first Layer-1 P0 RFC to land; RFC-002 (pin obligations), RFC-003 (trait-at-impl checking), and RFC-007 (generics-over-specs) all assume unit-typed specs exist and should reference this RFC rather than re-deriving unit typing.

**Amendment — 2026-07-13 (same day, post-acceptance):** extended the closed unit set from six to ten types, adding `Inductance`, `Power`, `Temperature`, and `Tolerance`. No change to the core mechanism (no coercion, no arithmetic, type-check-time enforcement) — purely an extension of the enumerable set, consistent with the RFC's own stated amendment path ("extending it is a future RFC, evaluated the same way any new concept is"). Downstream RFCs that reference this RFC's unit system should treat the closed set as the ten-type version going forward.
