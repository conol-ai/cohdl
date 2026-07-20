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
- Calls use name::(args) (turbofish) when the fn has generic parameters, or name(args) otherwise. Generic arguments are positional.

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

# Canonical form (cohdl fmt)

Accepted via RFC-009, see RFC-009: cohdl fmt canonical form + DR-018.

cohdl fmt defines exactly one canonical textual serialization for every construct above — no configuration, no style options. It is a pure function of the parsed AST (parse → re-serialize, never text-level munging), which makes it idempotent (fmt(fmt(x)) == fmt(x)) and semantically inert (never changes a design's parse tree, type-check verdict, or emitted netlist/BOM bytes) by construction, not just by testing.

Rules:

- 4-space indentation, no tabs. One statement per line inside any block (pins {}, spec {}, net, variants {}).
- Trailing line comments (// ...) are preserved verbatim, never reformatted or moved.
- At most one consecutive blank line; existing author-placed blank lines (e.g. grouping related declarations) are preserved, never inserted.
- No trailing whitespace; exactly one trailing newline per file.
- Pin declarations: required NAME: PINSPEC [role] — one space before [role], no space inside the brackets. Pin buses wrap with continuation lines aligned under the first pin number when they'd exceed the line-length target.
- spec {} / generic argument lists: comma-space-separated (MLCC<100nF, 16V, 10%>).
- net declarations: net NAME [annotation]: member, member, … — no space before the annotation bracket; members wrap aligned under the first member after the colon.
- impl Trait for Device {} stays on one line when the body is empty (the common case) — never split just because it's empty.
- variants {} / pins[VARIANT] / spec[VARIANT]: no space between the keyword and [.
- Turbofish (name::<Arg1, Arg2>(args)): no space around ::.
- Soft line-length target: 100 columns before wrapping is triggered.

fmt does not fix missing required syntax — a pin missing its [role] bracket is a parse error already (per RFC-008); cohdl fmt requires already-parsing source and is not a repair tool. cohdl fmt --check (non-mutating) is the CI/review-gate variant of the mutating cohdl fmt.

# Structured diagnostics (cohdl check --json)

Accepted via RFC-010, see RFC-010: cohdl check --json schema + DR-019.

cohdl check --json / cohdl build --json emit exactly one JSON document to stdout — a direct, versioned re-projection of the existing diagnostic pipeline's output, with zero invented content and zero information loss relative to the plain-text renderer.

```json
{
  "schema_version": 1,
  "verdict": "fail",
  "diagnostics": [
    {
      "code": "E110",
      "severity": "error",
      "message": "expected `Voltage`, found `Capacitance`",
      "primary": {
        "file": "src/main.cohdl",
        "start_line": 12, "start_col": 18,
        "end_line": 12, "end_col": 24,
        "message": "this net annotation must be Voltage-typed"
      },
      "secondary": [],
      "help": ["did you mean `net VBUS [5V]: ...`?"]
    }
  ]
}
```

Rules:

- schema_version — an integer, bumped only on a breaking change to this schema's own shape (new diagnostic codes/messages are ordinary content, not schema changes) — any consumer must check it before parsing further.
- verdict — "pass" or "fail", computed identically to the plain-text CLI's existing exit-code logic (any error-severity diagnostic present ⇒ "fail").
- diagnostics — a flat, ordered list (no per-pipeline-stage nesting — the pipeline doesn't tag diagnostics by stage, and --json never carries information plain-text output lacks). Each entry: code, severity ("error"/"warning"), message, primary (span resolved to 1-based file/start_line/start_col/end_line/end_col, plus its own message), secondary (zero or more, same shape), help (a string list, verbatim).
- cohdl build --json additionally includes a build object naming emitted artifact paths, present only when verdict is "pass".
- Equivalence guarantee: --json output and plain-text output always report the identical diagnostic set for the same input, field-for-field — this is a tested property (see RFC-010's Gradeability), not just a design intention.
- --json is not a repair tool and does not fix anything — it is the same read-only diagnostic pipeline, restructured.

# Error-code registry

Accepted via RFC-011, see RFC-011: Error-code registry (formal v2 baseline) + DR-009.

The full code-by-code listing lives in `docs/error-codes.md` (the single source of truth — not duplicated here). This section states the organizing principle and stability rule.

**Stability rule**: a code is issued once and never repurposed. If a check's behavior changes enough that its old meaning no longer applies, the code is retired (marked `[DEPRECATED]`, kept documented) and a new one is issued — never edited in place.

**Organizing principle**: a block is chosen by **kind of mistake**, not by which compiler pass happens to catch it.

| Block | Owner mechanism |
|---|---|
| E00x | CLI invocation (pre-pipeline, not a source diagnostic) |
| E0xx | Lexing & parsing |
| E1xx | Unit system (RFC-001) — all unit-mismatch diagnostics, regardless of call site |
| E2xx | Name resolution |
| E3xx | Trait satisfaction at impl (RFC-003) |
| E4xx | Generics (RFC-007), excluding unit-mismatch (that's E1xx) |
| E5xx | Sub-circuit fns (RFC-006) |
| E6xx | Design assembly & nets |
| E7xx | Pin connection obligations (RFC-002) |
| E8xx | Designators & parts (RFC-005) |
| E9xx | Structural variants (RFC-008) |
| D00x | Residual DRC (RFC-004) — exactly four, never more |

**Enforcement**: a mechanical, CI-run completeness test checks both directions — every diagnostic code literal in compiler source has a registry row, and every non-deprecated registry row has a real call site in source. This closes the same "structurally present but not actually enforced" gap class DR-006 named for DRC rules, applied here to the diagnostics registry.

# #[intent(...)] annotations

Accepted via RFC-012, see RFC-012: #[intent(...)] annotations (pure metadata) + DR-010.

#[intent("...")] attaches a single, opaque, human-readable rationale string to a declaration — structured attribution distinct from an ordinary // comment, with a guaranteed zero impact on compilation.

```cohdl
#[intent("100nF chosen per ESP32-S3 datasheet §3.4 decoupling recommendation, not a generic default")]
inst c_esp_decouple: MLCC_100nF_16V_0402
```

Rules:

- Exactly one string literal argument — no structured sub-fields.
- Attachable to: inst, net, nc, impl, device, trait, fn, part. At most one #[intent(...)] per declaration — a duplicate is a compile error.
- Zero compiler impact, by construction: the type checker, residual DRC, designator allocator, and netlist/BOM emitters never read this field — it is not a parameter any of those passes' functions accept. Mutating an #[intent(...)] string can never change a design's verdict, diagnostics, designator assignment, or emitted netlist/BOM bytes.
- cohdl fmt places it as a single-line attribute directly preceding its target, matching the existing #[designator("Xxx")] convention.
- cohdl check --json does not surface #[intent(...)] content in its diagnostics schema.
- If a stated "intent" reads like a checkable constraint, it is not enforced — it's decorative prose. Real constraints belong in the type system (RFC-001/002/003/007) or residual DRC (RFC-004), never in an #[intent(...)] string.

# Layout constraints (the door)

Accepted via RFC-013, see RFC-013: Layout-constraint concept (the door) + DR-011. Enabled by GC-002's amendment (note 8) — Tony's explicit decision to open the layout door ahead of its originally-stated "concrete partner requirement" trigger.

Layout Constraint is a new core concept, positioned adjacent to Net/Rule (per note 2's pre-designed seam) — never a second connectivity mechanism, never a DRC rule. It lets an author state a layout-relevant fact for a partner layout tool to consume; CoHDL itself never places, routes, or reasons about physical geometry.

```cohdl
design SensorNode {
    net USB_DP: usb.DP, esp.IO20
    net USB_DM: usb.DN, esp.IO19

    layout {
        net_class HighSpeed { USB_DP, USB_DM }
        diff_pair(USB_DP, USB_DM)
        length_match(USB_DP, USB_DM) [tolerance: 0.15mm]
    }

    #[placement_hint("near USB connector, short trace to ESP32-S3 USB pins")]
    inst esp
}
```

Four closed constraint kinds:

- net_class NAME { net, net, ... } — a named group of nets sharing a layout treatment. NAME must be declared before use.
- diff_pair(net_p, net_n) [differential_impedance: IMPEDANCE, single_ended_impedance: IMPEDANCE, frequency: FREQ] — exactly two existing nets, declared as a differential pair. The trailing bracket is optional (added by RFC-027, see Quilter physics-constraint hints below) — IMPEDANCE values are Resistance-typed (RFC-001), FREQ is Frequency-typed. Omitting the bracket preserves the original, unannotated form's meaning exactly; every diff_pair(...) statement written before RFC-027 is unchanged.
- length_match(net, net, ...) — two or more existing nets that must be length-matched, with an optional [tolerance: ...] bracket.
- #[placement_hint("...")] — a single opaque string on an inst, same shape and zero-impact discipline as #[intent(...)] (RFC-012).

Rules:

- layout { ... } is a new top-level block inside a design (or fn body). Referenced nets must already exist.
- The first three kinds are exhaustively type-checked against their own closed vocabulary (unknown net reference, duplicate net_class name, wrong diff_pair arity, length_match with fewer than two nets, net_class referenced before declaration are all compile errors — see error-code block E10xx).
- Zero schematic-correctness impact, by construction: layout-constraint data is emitted as a separate artifact (layout.json/netlist addendum) — the type checker, residual DRC, designator allocator, and .net/BOM emitters never read it. Mutating any layout {} content can never change a design's verdict, RFC-001–011 diagnostics, designator assignment, or .net/BOM bytes.
- CoHDL does not verify that a length_match tolerance is actually met, or that a diff_pair is physically routed as one — it has no geometry to check against. The data is purely passed through to whatever partner tool consumes it.
- The four constraint kinds are explicitly provisional — per GC-002's disclosed design debt, expected to be revisited once a real partner layout-tool integration is scoped. A future reshaping is anticipated, not a stability violation of this RFC.

# Quilter physics-constraint hints

Accepted (redesigned same day) via RFC-027, see RFC-027: Quilter physics-constraint hints and CSV export + DR-033 revision. Revision note: the original acceptance added seven new bare layout {} statement keywords. Tony corrected this same day ("use attributes style syntax... do not add so many keywords") — this section reflects the revised design: every fact attaches directly to the net/inst declaration it's actually about, as a structured #[name(...)] attribute reusing the existing attribute-bracket syntax (RFC-005's #[designator(...)], RFC-012's #[intent(...)], RFC-013's #[placement_hint(...)]) — zero new keywords added to the lexer. Grounded in eight real CSV files Tony supplied, matching Quilter's own documented "Physics Constraints" mechanism (docs.quilter.ai/physics-constraints/*).

```cohdl
design SensorNode {
    #[high_current(500mA)]
    net V3V3 [3.3V]: ldo.VOUT, mcu.VDD, u2.VIN

    #[ground(primary)]
    net GND: ldo.GND, mcu.GND, u2.GND

    #[impedance(50ohm, frequency: 1GHz)]
    net HDMI_CLK: mcu.HDMI_CLK, hdmi.CLK

    #[bypass(mcu.VDD, 100nF)]
    inst c1: MLCC<100nF, 16V>

    #[crystal_oscillator(mcu, XTAL_IN, XTAL_OUT)]
    inst y1: Crystal_8MHz

    #[switching_converter(inductor: l1, input_capacitor: c_in, output_capacitor: c_out)]
    inst u2: BuckConverter
    inst l1: Inductor_2_2uH
    inst c_in: MLCC<10uF, 16V>
    inst c_out: MLCC<22uF, 16V>

    #[bga_fanout]
    inst mcu: MCU_BGA_256
}
```

Seven structured attributes, each mapping 1:1 to a real, externally-documented Quilter constraint field schema:

- #[ground(PRIMARY [, region_pour])] on a net — PRIMARY closed to {primary, secondary}; at most one primary ground net per design (checked). region_pour a bare optional flag, defaults absent (⇒ false).
- #[high_current(CURRENT [, power_pour])] on a net — CURRENT a Current-typed value (RFC-001). power_pour a bare optional flag. Quilter's documented "Power Nets" constraint.
- #[impedance(IMPEDANCE, frequency: FREQ)] on a net — IMPEDANCE a Resistance-typed value, FREQ a Frequency-typed value (both RFC-001).
- #[bypass(INST.PIN, CAPACITANCE)] on the bypass capacitor's own inst declaration — INST.PIN an already-declared instance + pin (RFC-002); CAPACITANCE a Capacitance-typed value. The capacitor's own designator is read off the inst the attribute is attached to. Per RFC-028, INST.PIN's position also accepts a bare Pin-typed fn parameter name (e.g. #[bypass(vdd, 100nF)] inside a fn body) — see "fn Pin parameter targets" below.
- #[crystal_oscillator(PARENT_INST, PIN_1, PIN_2)] on the crystal's own inst declaration — PARENT_INST an already-declared instance; PIN_1/PIN_2 two of PARENT_INST's declared pins. Per RFC-028, any of these three arguments may instead be a bare Pin-typed fn parameter name — see "fn Pin parameter targets" below.
- #[switching_converter(inductor: INST [, input_capacitor: INST] [, output_capacitor: INST])] on the converter's own inst declaration — inductor required; the two capacitor arguments each optional (per Quilter's own docs), all already-declared instances. Per RFC-028, any instance argument here may instead be a bare Pin-typed fn parameter name — see "fn Pin parameter targets" below.
- #[bga_fanout] on a BGA's own inst declaration — a bare attribute, no arguments; presence ⇒ generate_fanout: true.

## fn Pin parameter targets

Accepted via RFC-028, see RFC-028: Physics-constraint attributes on fn Pin parameters + DR-034. Closes a real gap: a reusable decoupling fn (RFC-006's own idiomatic pattern) instantiates its own capacitor internally, wired to a Pin-typed parameter — #[bypass(...)]'s target could not name that parameter, since RFC-027's original checker only accepted a literal top-level INST.PIN.

```cohdl
pub fn decouple(vdd: Pin, gnd: Pin) {
    #[bypass(vdd, 100nF)]
    inst c: C_100n
    net _: vdd, c.A
    net _: gnd, c.B
}

design Board {
    inst mcu: MCU_ESP32S3
    decouple(mcu.VDD1, mcu.GND)
    decouple(mcu.VDD2, mcu.GND)
    // ... 23 more real call sites ...
}
```

- #[bypass(...)], #[crystal_oscillator(...)], and #[switching_converter(...)]'s target/instance arguments accept a bare Pin-typed fn parameter name (e.g. vdd above) in addition to a literal top-level INST.PIN/INST reference — the same bare PinRef grammar (identifier, optional .pin) already legal everywhere else in the language (e.g. inside a net member list: net _: vdd, c.A).
- Resolution reuses the existing resolve_pin_ref/Binding::Pin machinery (RFC-006) unchanged — no new grammar, no new binding concept. At expansion, each real call site of the fn produces its own real, independently-resolved attribute instance and CSV row — a fn called 25 times with an attribute-bearing inst inside it produces 25 real, independently-resolved facts, one per call site, never one shared fact for the whole fn definition.
- #[ground(...)], #[high_current(...)], and #[impedance(...)] are unaffected — they attach to a net, and a net's own name already resolves identically per call site whether declared inside a fn body or at the top level (unchanged since RFC-006/RFC-024); there is no analogous gap for these three.
- An identifier that resolves to neither a bound fn parameter nor a top-level instance in scope is a compile error naming what wasn't found — unchanged in kind from RFC-027's own diagnostic. Error codes stay in the existing E10xx family — no new block.

Rules:

- These seven attributes are structurally distinct from the existing generic, opaque-string Attr (#[intent(...)]/#[placement_hint(...)]/#[designator(...)], all exactly-one-string-literal) — each carries its own real, closed argument grammar (unit-typed literals, bare flags, pin/instance references, named optional arguments), parsed and structurally checked, not opaque prose. They share the surface #[name(...)] bracket syntax with the existing attributes (no new bracket/lexer token) but are recognized as their own closed set of attribute names, each with its own fixed argument shape.
- Every pin/instance reference argument must resolve to an already-declared instance/pin — unresolved is a compile error naming what wasn't found. At most one of each attribute kind per declaration (the same discipline #[intent(...)]/#[placement_hint(...)] already enforce).
- diff_pair's extension (see Layout constraints above) is the one exception — that fact stays a layout{} statement, since it is inherently about a pair of nets, not attachable to one single declaration.
- cohdl build emits eight new CSV artifacts (bga_components.csv, bypass_capacitors.csv, crystal_oscillators.csv, differential_pairs.csv, ground_nets.csv, high_current_nets.csv, single_ended_impedance_signals.csv, switching_converters.csv) — headers/column order matching Tony's real supplied files exactly, one row per net/inst carrying the corresponding attribute (an empty file with just the header row when a design declares none). cohdl build --json's build object gains one new key per CSV file (path), present only when emitted.
- Not automatic constraint inference — Quilter's own docs describe most of these as auto-detected from naming/topology; CoHDL does not replicate this. Every constraint here is an explicit author-written attribute. An author who wants Quilter's own auto-detection to run can simply omit the corresponding attribute — Quilter's detection still operates on the plain netlist CoHDL already emits.
- CoHDL performs no validation that a stated constraint (impedance, frequency, current) is physically achievable — that is Quilter's own downstream Physics Rule Check validation, consistent with every prior layout-adjacent RFC's discipline.
- Zero schematic-correctness impact, by construction — none of these attributes are read by the type checker, residual DRC, designator allocator, or .net/BOM emitters.
- Error codes stay in the existing E10xx family (layout constraints, RFC-013/020's home): unresolved pin/instance reference per attribute, invalid PRIMARY value, duplicate primary ground net, missing required argument (switching_converter's inductor), unit-type mismatch on any numeric argument, duplicate attribute of the same kind on one declaration — no new block.

# Language Server (cohdl lsp)

Accepted via RFC-014, see RFC-014: Language Server Protocol support + DR-020.

cohdl lsp starts a JSON-RPC/stdio server for editor integration — a thin frontend over the exact same pipeline::check() the CLI already uses. No new diagnostic logic, no new checks.

Capabilities:

- textDocument/publishDiagnostics — the same diagnostics cohdl check --json reports for a file, pushed live on open/change/save. Guaranteed field-for-field identical to the CLI's --json output for the same input (mandatory equivalence test).
- textDocument/hover — on an empty impl Trait for Device {} block, shows the resolved by-name matches (which device field satisfied which trait requirement) even though the source body is empty; on a pin, shows its resolved obligation kind and role.
- textDocument/definition — a device/trait/fn/part name at any use site resolves to its declaration's span.
- textDocument/references — invoked on a trait or device name in an impl statement, lists every matching impl in the currently-open project ("find all impls of this trait" / "find all impls for this device").

Rules:

- Every request re-runs the full pipeline from scratch — no incremental compilation (tracked separately as future work, not a blocking prerequisite).
- The server depends on the lsp-types crate for the LSP spec's own message shapes — the project's first scoped exception to its otherwise hand-rolled, zero-external-dependency style. The JSON-RPC transport loop itself stays hand-rolled.
- cohdl lsp introduces no new error codes, no new syntax, no new diagnostic content — it is purely a new transport/frontend for what RFC-001–013 already check.

# IPC-2581 output (ipc2581.xml)

Accepted via RFC-015, see RFC-015: IPC-2581 codegen backend (Quilter handoff) + DR-021.

cohdl build gains a new emitted artifact — a partially-specified IPC-2581 document (.xml, conforming to IPC-2581B1.xsd) — alongside the existing .net/BOM/layout.json outputs, never replacing them. Grounded in real prior research (the "Quilter as a CoHDL Backend Partner — Fit Analysis" note): IPC-2581 is the vendor-neutral contract that reaches Cadence Allegro and Siemens Xpedition (KiCad-only handoffs can't), and carries netlist + specs + layout constraints in one file.

What the document carries (all mapped from existing, already-validated CoHDL data — no new type-checking):

- The logical netlist — every net and its member pins, from the same connectivity data the KiCad emitter already uses.
- Every instance's resolved designator (RFC-005), bound part's MPN/manufacturer (RFC-003-guaranteed complete), and footprint name (referenced, not resolved to pad geometry).
- Resolved unit-typed spec values (RFC-001), carried via IPC-2581's own component-attribute mechanism.
- RFC-013's layout constraints (net_classes/diff_pairs/length_matches/placement_hints), mapped into IPC-2581's native constraint/net-class elements.

Rules:

- The document is deliberately, visibly partial — a document-level marker states "logical-complete, physical-minimal," since CoHDL has no footprint-geometry resolution or board-outline/stackup concept today. The document never silently claims completeness it doesn't have.
- CoHDL does not (and this RFC does not make it) own footprint pad geometry — a part's footprint field stays a name reference, same as it is for the KiCad emitter.
- cohdl build --json's build object gains an "ipc2581" key (path), present only when the artifact is emitted — same pattern as the existing "layout" key.
- This is explicitly phase one of a multi-phase partner integration. Footprint-geometry resolution, board-outline/stackup support, and real end-to-end validation against an actual layout partner are named, tracked future work — not silently assumed solved by this RFC.
- Does not solve the layout-partner ECO/re-routing mismatch some generative routers exhibit (full re-route on every netlist change, no incremental update) — a workflow-level limitation for whoever operates the loop, not something this emitter's format can fix.

# Modules and packages

Accepted via RFC-016, see RFC-016: Module system (package::module::submodule::name) + DR-022.

CoHDL's first real namespace mechanism. A package's module tree mirrors its file tree under src/, rooted at cohdl.toml's [package] name field — no separate mod declaration needed.

```cohdl
// sparkfun/src/power/buck.cohdl → module path sparkfun::power::buck

// Fully qualified, always valid:
inst ldo1: sparkfun::power::buck::TPS62840

// Or import once, use unqualified thereafter:
use sparkfun::power::buck::TPS62840;
inst ldo1: TPS62840
```

Rules:

- Each file's path under src/ becomes its module segment (/ → ::, extension dropped). A top-level declaration's fully-qualified path is package::its-file's-module-path::Name.
- use path::Name; imports exactly one name into local scope. Importing the same local name twice (from different paths) is a compile error naming both source paths.
- Within a single package with no use statements, every name in every one of the package's own files stays visible unqualified everywhere else in the package — unchanged from today's behavior for any project with no external dependencies.
- Cross-package names are never implicitly visible — reachable only via a qualified path or an explicit use.
- pub is now enforced, but only across package boundaries — referencing a non-pub item from another package is a compile error naming the item and its actual visibility. Intra-package visibility is unaffected by pub (unchanged from today).
- Two declarations at the same module path with the same name is a compile error, now scoped per-module-path instead of globally — a sparkfun::power::buck::TPS62840 and an unrelated acme::power::TPS62840 never collide.
- No glob imports (use path::*) or re-export sugar in this pass — explicit, one-name-per-use only. Deferred pending real usage friction, not because they're rejected in principle.

# Library registry: documents and footprints

Accepted (revised) via RFC-017, see RFC-017: Library registry (cohdl source + docs + footprint symbols) + DR-023 (+ same-day amendment). Revision note: the original acceptance defined a native .cfp footprint file format with a path-string reference from part. Same day, Tony corrected this: footprints must resolve as named symbols under the module system (for cross-library reuse), and the footprint format itself is deferred to a future, separately-numbered RFC. This section reflects the revised design.

A Library is just a Package (see Modules and packages above) with two new optional content kinds. Skills (manufacturer best-practice content) are explicitly deferred to a future RFC — this registry ships with exactly three content kinds: .cohdl source, reference documents, and footprint symbols (with the footprint symbol's internal content itself deferred — see below).

Reference documents — #[doc(...)]:

```cohdl
#[doc("datasheets/TPS62840.pdf")]
#[doc("app-notes/buck-converter-layout-guidelines.pdf")]
pub device TPS62840<...> { ... }
```

- One or more #[doc("relative/path")] attributes per declaration (unlike #[intent(...)]'s at-most-one rule).
- Paths are relative to the library's package root. The compiler never opens these files — same zero-compilation-impact discipline as #[intent(...)] (RFC-012) and #[placement_hint(...)] (RFC-013). cohdl lsp (RFC-014) may surface these paths on hover as a natural extension of its existing hover capability.

Footprints and pads — see "Footprints and pads (pad/footprint)" section below for the real, Accepted design (RFC-018, same day). RFC-017's original placeholder footprint keyword never shipped with real content; RFC-018 gives it real content for the first time (no rename).

# Footprints and pads (pad/footprint)

Accepted via RFC-018, see RFC-018: Footprint format — pad/footprint, Cadence-style pad/footprint split + DR-024 (+ same-day naming correction). Supersedes RFC-017's placeholder footprint keyword's empty body (which shipped deliberately empty, "symbol-resolution-complete, format-empty") — footprint (unchanged keyword, RFC-017's own declaration kind) now has real content for the first time, adopting Cadence Allegro's proven design: pads are defined once, standalone, and reused by reference across footprints, rather than inlined per footprint. (Note: the same-day first draft of RFC-018 used invented names copad/cofp; Tony corrected these to plain pad/footprint before acceptance — this section reflects the final, corrected names.)

pad — one reusable pad definition:

```cohdl
// sparkfun/src/pads/smd.cohdl → module path sparkfun::pads::smd

pub pad Rect_0_3x0_9mm {
    shape: rect
    size: (0.3mm, 0.9mm)
    layer: top_copper
    plating: smd
}

pub pad Round_0_5mm_THT {
    shape: circle
    size: (0.5mm)
    layer: through_all
    plating: plated_through_hole
    drill: 0.3mm
}
```

- shape: one of rect, circle, oval (closed set).
- size: shape-dependent — (w, h) for rect/oval, (d) for circle.
- layer: one of top_copper, bottom_copper, through_all (closed set).
- plating: smd or plated_through_hole.
- drill: required when plating: plated_through_hole; a compile error if present when plating: smd.

footprint — composed of pad references:

```cohdl
// sparkfun/src/footprints/qfn.cohdl → module path sparkfun::footprints::qfn

use sparkfun::pads::smd::Rect_0_3x0_9mm;

pub footprint QFN10_3x3 {
    pad 1: Rect_0_3x0_9mm at (-1.5mm, 1.0mm)
    pad 2: Rect_0_3x0_9mm at (-1.5mm, 0.5mm)
    pad 3: Rect_0_3x0_9mm at (-1.5mm, 0.0mm)
    // ... one entry per pad, matching the bound device's pin count and numbering
    courtyard { shape: rect, at: (0mm, 0mm), size: (3.5mm, 3.5mm) }
    silkscreen_ref { at: (0mm, -2.2mm) }
}
```

```cohdl
use sparkfun::footprints::qfn::QFN10_3x3;

pub part TPS62840_QFN10: TPS62840<...> {
    primary { mfr: "Texas Instruments", mpn: "TPS62840DLCT", footprint: QFN10_3x3 }
}
```

Rules:

- pad and footprint are both top-level declaration kinds, resolved through RFC-016's module-path/use/pub machinery exactly like device/trait/fn/part — no new resolution mechanism. footprint keeps the same keyword RFC-017 already introduced; pad is the one genuinely new top-level keyword.
- Each pad N: PadSymbol at (x, y) line in a footprint places one instance of a pad symbol at an offset relative to the footprint's own origin. PadSymbol resolves like any other cross-library reference. This body-level pad N: ... placement statement and the top-level pad { ... } declaration share the same keyword but occupy different grammatical positions (the same pattern already used for net/nc as body-level statements vs. other top-level forms).
- Pad numbers (N) must exactly match the bound device's declared pin numbers (RFC-002) — checked at the point a part's footprint: field resolves to a footprint symbol, at cohdl build (the same point MPN completeness is checked, RFC-003's precedent). This is the check RFC-017 deferred, now real because footprint's pad list is real structured data.
- The same pad symbol may be referenced by any number of footprint declarations, in any package that can resolve it — a single point of correction for a reused pad shape. The flip side, disclosed honestly: a wrong pad dimension is a single point of failure across every referencing footprint.
- No versioning/pinning for pad references — a footprint always resolves to whatever the referenced pad currently is, the same as every other use-based resolution in the language today.
- cohdl build projects a resolved footprint's pad geometry into whatever the active emitter needs: a .kicad_mod file for the KiCad .net output, or inline geometry for RFC-015's IPC-2581 document — directly closing RFC-015's own named future-work item (footprint-geometry resolution).
- pad/footprint's scope is deliberately minimal: no 3D models, no per-layer-independent padstacks (vias, thermal reliefs), no board outline/stackup (still separately unaddressed, per RFC-015).

## Mechanical locating holes (mount_hole)

Accepted via RFC-022, see RFC-022: Mechanical locating holes in footprints (mount_hole) + DR-028; extended by RFC-023 (Non-circular locating holes) + DR-029. Closes a real gap in the pad/footprint model above: some real footprints require a mechanical locating hole (定位孔) — e.g. a connector shell's alignment-pin holes, or (per RFC-023) a switch's rectangular mounting legs — which has no electrical function, no net, and no device pin number to bind to. Grounded in KiCad's own established np_thru_hole (non-plated through-hole) precedent for exactly this distinction.

```cohdl
pub footprint KailhChocV2 {
    pad 1: Round_1_0mm_THT at (-2.75mm, 3.0mm)
    pad 2: Round_1_0mm_THT at (2.75mm, -3.0mm)

    mount_hole 1: non_plated shape: rect size: (2.0mm, 1.5mm) at (-6.75mm, 0mm)
    mount_hole 2: non_plated shape: rect size: (2.0mm, 1.5mm) at (6.75mm, 0mm)

    courtyard { shape: rect, at: (0mm, 0mm), size: (15.5mm, 15.5mm) }
    silkscreen_ref { at: (0mm, -8.5mm) }
}
```

- mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D | size: (w, h)] — a third footprint-body construct alongside pad, courtyard, and silkscreen_ref.
- N is a locating-hole-local counter, entirely disjoint from pad's pin-bound numbering — a footprint may have pad 1..16 and mount_hole 1..2 in the same declaration with no collision, since they are independently-numbered sequences. mount_hole numbers are never checked against the bound device's declared pins (RFC-002) — this is the defining structural difference from pad, and it is what keeps RFC-018's pad-count/numbering completeness guarantee unconditional rather than requiring a special-cased exception.
- PLATING is a closed two-value set: non_plated (the common case — a bare mechanical hole) or plated (e.g. a chassis-ground stud, still carrying no net). There is no smd value — a mount_hole is definitionally a hole, never a surface pad.
- shape: is optional, one of rect, circle, oval — RFC-018's existing PadShape closed set, reused verbatim (no new enum). Absence of shape: defaults to circle, preserving every mount_hole declaration written before RFC-023's shape/size extension.
- The geometry field is shape-dependent, mirroring pad's own established convention exactly: circle (explicit or defaulted) takes diameter D (a single Length value); rect/oval take size: (w, h) (two Length values, the same tuple shape pad's own size: field already uses). Writing diameter alongside shape: rect/shape: oval, or size: alongside shape: circle, is a compile error naming the mismatch.
- No layer: field — a mount_hole always spans through_all.
- All checks this construct introduces are structural and local to one footprint declaration: no duplicate mount_hole numbers within one footprint; diameter/size: must be present and Length-typed and must match the (explicit or defaulted) shape; PLATING must be one of the two closed values; shape:, when present, must be one of the three closed values. None run in residual DRC.
- cohdl build projects non_plated as KiCad's own np_thru_hole pad type in the .kicad_mod emitter (KiCad's np_thru_hole pads are not restricted to round shapes, so rect/oval mount_hole geometry reuses the same emitter code path as circular ones) and plated as an ordinary plated through-hole pad with no net assigned; the IPC-2581 emitter projects both as hole/pin geometry with no net reference.
- Explicitly out of scope: board-level mounting holes (a board's own corner screw holes are a design/board-level concept, closer in spirit to board_outline, RFC-020, than to a per-footprint construct) and any locating-hole shape beyond rect/circle/oval (true slots with rounded ends, keyed/D-shaped holes) — both real, disclosed, deferred gaps, not silently solved.

# Array-typed instances and indexed references

Accepted (redesigned same day) via RFC-024, see RFC-024: Array-typed instances and indexed references + DR-030 revision. Revision note: the original acceptance made inst NAME[START..=END]: Device pure name-expansion sugar, with indexing usable only inside a net's member list. Tony corrected this same day: the real request is inst key_leds: [RGB_SK6812; 13] — one real, array-typed instance, indexed by a literal (key_leds[0]) anywhere an instance reference is valid — not a naming trick scoped to one syntactic position. This section reflects the revised design, grounded directly in examples/openmicro/src/main.cohdl's real repetition (42 near-identical inst lines) and its real WS2812 daisy-chain wiring and per-LED place statements, both of which need to address one specific array element by index.

Array-typed instance declaration:

```cohdl
design OpenMicro {
    inst key_leds: [RGB_SK6812; 13]
    inst ambient_leds: [RGB_SK6812; 16]
    inst sw: [SW_KEY; 13]
    inst d: [D_1N4148W; 13]
    inst mh: [MH_M2; 4]
}
```

- inst NAME: [Device; N] — N is a positive integer literal, the array's fixed length. NAME is the array's own name; unlike a plain inst, a bare NAME (unindexed) is never itself a valid instance reference — an author must always index it: NAME[i].
- key_leds and ambient_leds are two separate arrays of the same device type — the ordinary way to express two independent chains/families of the same part.
- A second declaration (array or plain inst) reusing NAME is a compile error, the same collision rule ordinary inst already has.

Indexed references, valid everywhere an ordinary instance reference already is:

```cohdl
design OpenMicro {
    net LED_D0: mcu.LED_DATA_KEY, key_leds[0].DIN
    net LED_D1: key_leds[0].DOUT, key_leds[1].DIN
    net LED_D2: key_leds[1].DOUT, key_leds[2].DIN
    // ... one net per chain link ...

    decouple(key_leds[0].VDD, key_leds[0].GND)
    decouple(key_leds[1].VDD, key_leds[1].GND)

    layout {
        place key_leds[0] at (-8.025mm, -23.875mm)
        place key_leds[1] at (11.025mm, -23.875mm)
    }
}
```

- NAME[i] (i a literal integer, 0 <= i < N) resolves to one real, individually-checked instance element — usable as a net-member base (NAME[i].PIN), as place's target (place NAME[i] at (...)), as decouple's arguments (decouple(NAME[i].PIN, ...)), and as a fn-call argument — every position an ordinary bare instance name is already valid, not restricted to one syntactic position.
- An out-of-bounds index (e.g. key_leds[13] when N is 13, valid indices 0..=12) is a compile error naming the valid range, checked wherever the reference appears.
- After resolution, every existing per-instance mechanism applies completely unchanged to NAME[i] — its own designator (RFC-005), its own pin-obligation tracking (RFC-002), its own trait satisfaction (RFC-003) — exactly as if it had been hand-declared with its own bare name.

Range/list fan-out, inside a net's member list only:

```cohdl
net VBUS [5V]: usbc.VBUS, key_leds[0..=12].VDD, ambient_leds[0..=15].VDD
net COL0: mcu.COL0, d[0, 4, 8, 12].Cathode
```

- NAME[START..=END].PIN and NAME[i1, i2, i3, ...].PIN remain valid inside a net's member list, now defined as sugar expanding to individual NAME[i].PIN references — key_leds[0..=12].VDD means exactly key_leds[0].VDD, key_leds[1].VDD, ..., key_leds[12].VDD, a pure textual equivalence.
- This fan-out sugar is scoped to net-member lists only — place/decouple always take one single index (each array element needs its own coordinates; decouple already takes two explicit pin arguments), so "place/decouple a whole range at once" has no single sensible meaning and is not supported.

Rules:

- NAME[i]'s index (and a range/list's endpoints) must be a literal integer, checked against the array's declared length N wherever the reference appears — never a variable or computed index; this RFC introduces no expression language.
- No general loop/iteration construct is introduced — fn (RFC-006) remains the sole mechanism for "repeat a parameterized sub-circuit with systematic wiring." Because there is no loop construct, daisy-chain wiring (e.g. a WS2812 chain's DOUT→DIN pattern) and arithmetic-derived per-element place/decouple data (e.g. grid-formula coordinates) are NOT auto-generated — an author still writes one net/place/decouple statement per array element by hand; this RFC makes each such statement correctly, individually addressable via NAME[i], but does not generate the statements. Both are real, explicitly disclosed, deferred future work.
- No multi-dimensional arrays (NAME[i][j]) and no array-of-non-device element types — an array's element type is always a single device type, the same restriction an ordinary inst already has.
- Error codes stay in the existing E2xx block (name resolution, RFC-016's home): out-of-bounds index, array-name collision, malformed array-length literal — no new block, per RFC-011's "kind of mistake" organizing principle.

### Rotated pad placements

Accepted via RFC-025, see RFC-025: Rotated pad placements in footprints + DR-031. Closes a real gap in the pad/footprint model above: QFN/LQFP footprints place the same pad shape on all four package sides, with top/bottom-side pads rotated 90° relative to left/right-side pads — a real, common pattern (confirmed against a real KiCad QFN-20-1EP_4x4mm footprint).

```cohdl
pub footprint QFN20_4x4 {
    // Left side — pad's natural (unrotated) orientation
    pad 1: Rect_0_825x0_25mm at (-1.9375mm, -1.0mm)
    pad 3: Rect_0_825x0_25mm at (-1.9375mm, 0mm)

    // Top side — same pad symbol, rotated 90°
    pad 6: Rect_0_825x0_25mm at (-1.0mm, 1.9375mm) rotate 90
    pad 8: Rect_0_825x0_25mm at (0mm, 1.9375mm) rotate 90

    // Right side — rotated 180°
    pad 11: Rect_0_825x0_25mm at (1.9375mm, 1.0mm) rotate 180

    // Bottom side — rotated 270°
    pad 16: Rect_0_825x0_25mm at (1.0mm, -1.9375mm) rotate 270

    courtyard { shape: rect, at: (0mm, 0mm), size: (4.5mm, 4.5mm) }
    silkscreen_ref { at: (0mm, -2.5mm) }
}
```

- pad N: PadSymbol at (x, y) [rotate ANGLE] — rotate is a new, optional clause on pad's existing placement statement. ANGLE is closed to {0, 90, 180, 270} — the exact same set and keyword place ... rotate (RFC-020) already uses, reused here by direct precedent rather than a new mechanism. Omitted rotate defaults to 0 (unrotated) — every existing pad N: ... at (x, y) statement is unchanged in meaning.
- rotate is purely a placement-time fact — the referenced pad symbol's own shape/size fields are never mutated or restated. One pad definition can be placed at many positions, at many different rotations, exactly mirroring how one place-able component's footprint stays fixed while its board-level orientation varies.
- For rect/oval pads, 90°/270° visibly swaps the pad's effective width/height; 180° has no visible geometric effect but is still valid (some authors state it for documentation/consistency). For a circle pad, rotate is accepted at any of the four values but is a structural no-op (a circle has no orientation) — intentional, not an inconsistency, so a rotation pattern can be copy-pasted across mixed pad shapes without branching on shape.
- Unlike real hand-authored KiCad libraries (which achieve the rotated appearance by silently swapping a pad's w/h and omitting any angle), CoHDL's .kicad_mod emitter emits the pad's declared size unchanged plus a real KiCad (at x y angle) rotation argument — this preserves the author's stated rotation fact losslessly, deliberately diverging from typical KiCad-library convention to avoid discarding that fact.
- rotate's closed-set membership is checked at declaration, identical in shape to place ... rotate's existing check. No other new semantic check is introduced; rotate's value has no bearing on RFC-018's existing pad-count/pin-number-matches-device-pins check.
- Error codes stay in the existing E8xx block (designators & parts, RFC-018's home for footprint-completeness checks): invalid rotate value on a pad placement — no new block.

## Footprint naming: names must comply with IPC-7351

Accepted (revised twice, same day) via RFC-021, see RFC-021: IPC-7351 as the canonical footprint naming practice + DR-027 (revised twice). Revision history: the original acceptance added a separate, optional ipc_name field alongside an unconstrained footprint symbol name. Same day, first correction: a footprint has one identity and should have one name — the footprint declaration's own identifier (the same name RFC-016's module system resolves) must itself comply with IPC-7351B naming, no new field. Second correction, same day: an intermediate revision had also carried a third-party-CAD-tool-name reference alongside the identifier — this was removed. CoHDL does not track, reference, or map to any third-party CAD tool's footprint library (KiCad, LCEDA, Allegro, or otherwise). Every footprint is CoHDL's own native geometry declaration (RFC-018); this section's naming discipline applies solely to that declaration's own identifier, and nothing else. This section reflects the final design.

For a closed six-family-template subset of IPC-7351B — QFP (incl. LQFP/TQFP), QFN (incl. SON/VQFN), SOIC/SOP, SOT, BGA, CHIP/MELF — CoHDL requires the footprint declaration's own name to match that family's IPC-7351B template. Chosen in preference to JEDEC JESD30 (which names the package body, not the land pattern) or an invented CoHDL-native scheme.

```cohdl
pub footprint QFN10N40P300X300_1EP180X180 {
    pad 1: Rect_0_3x0_9mm at (-1.5mm, 1.0mm)
    // ...
}
```

- The identifier after pub footprint is the IPC-7351B designator itself, with - mapped to _ (CoHDL identifiers can't contain -) — a single, fixed substitution, not a free-form escaping scheme. E.g. the IPC-7351B designator QFN10N40P300X300-1EP180X180 becomes the CoHDL identifier QFN10N40P300X300_1EP180X180.
- CoHDL's closed set of recognized IPC-7351B family templates (pitch/span/height/pin-count/density-suffix encoded per IPC-7351B's own convention — hundredths of a millimeter, no decimal point):Family prefixMeaning`QFP`Quad flat pack (incl. LQFP/TQFP)`QFN`Quad flat no-lead (incl. SON, VQFN)`SOIC` / `SOP`Small-outline IC`SOT`Small-outline transistor`BGA`Ball grid array`CHIP` / `MELF`Two-terminal passives (EIA size code, no density suffix)
- Density suffix is a closed three-value set: N (Nominal, default), L (Least), M (Most) — a missing or out-of-set suffix is a compile error for any name matching one of the closed families.
- A footprint's name is checked in two stages, whenever it matches one of the closed family prefixes: (1) grammar well-formedness against the family-template table above (declaration time); (2) geometry cross-check, for geometrically-regular families only (QFP, QFN, SOIC/SOP, SOT; BGA/CHIP/MELF analogously) — pin count and pitch derived from the footprint's own pad N: ... at (x, y) placements must agree with what the name encodes. A mismatch is a compile error naming the specific disagreement (e.g. declared vs. actual pin count or pitch). Irregular/mixed-pitch layouts get stage (1) only — geometry consistency is not checked for these, disclosed as a real scope boundary, not an oversight.
- A footprint whose package family falls outside the closed six-template set (e.g. connectors, relays) is unaffected — its name is checked only against RFC-016's ordinary identifier grammar, unchanged from before this RFC.
- No third-party-footprint-tracking construct exists in CoHDL. There is no per-CAD-tool name table, alias, or backend-mapping mechanism of any kind — CoHDL's footprint/pad declarations (RFC-018) are the sole, complete geometry model, and this naming discipline covers exactly and only that declaration's own identifier.
- Real, accepted trade-off: because IPC-7351 names are geometry-derived, a footprint's name changes if its geometry changes in a way that alters pin count/pitch/density (e.g. a density-level correction), and every use site referencing it must be updated to the new name — RFC-016's existing unresolved-name diagnostic catches every stale reference immediately, but the actual edit is real, ongoing authoring work. No second, stable-name layer was introduced to avoid this — see RFC-021's Alternatives for why.

# Editor support: VS Code extension

Accepted via RFC-019, see RFC-019: VS Code extension for CoHDL + DR-025.

A real, buildable, installable VS Code extension lives at editors/vscode/ — thin packaging over the already-Accepted cohdl lsp (RFC-014). Introduces no new language semantics, no new diagnostic, no new checkable construct.

What it adds:

- A hand-authored TextMate grammar (syntaxes/cohdl.tmLanguage.json) registering .cohdl for syntax highlighting — a static capability the LSP protocol itself has no verb for, so this is a genuinely separate artifact from the server.
- src/extension.ts wires vscode-languageclient to spawn cohdl lsp, turning on RFC-014's four capabilities (diagnostics, hover, goto-def, references) — identical spawn shape to the pre-existing doc snippet in docs/lsp.md, now packaged rather than copy-paste boilerplate.
- One new settings key, cohdl.path (default "cohdl", resolved via PATH), replacing the doc snippet's hardcoded binary path.

Rules:

- Zero new diagnostic logic — the extension's output is exactly cohdl lsp's output, unmodified; RFC-014's existing equivalence suite (tests/lsp.rs) continues to be the source of truth, not a new server-side test.
- A new grammar-coverage regression test (CI-only, not part of cohdl check/cohdl build) asserts every real keyword/literal-class token in a fixture corpus gets a TextMate scope, not plain-text fallthrough.
- The TextMate grammar can drift from the real language grammar as future RFCs add/rename keywords — this is a disclosed, not-fully-solved risk (no compiler-enforced guarantee is possible for an external editor's grammar file); the convention going forward is that any RFC introducing/renaming/removing a top-level keyword should update cohdl.tmLanguage.json in the same change it updates this note.
- Purely additive — no existing .cohdl source, diagnostic code, designator, or netlist byte is affected; a user who never installs the extension experiences zero change.
- Closes RFC-014's own explicitly-deferred packaging scope and its still-open real-client acceptance item (a live VS Code session actually exercising cohdl lsp, previously unverified per docs/compliance-report.md).

# Board outline and oriented placement

Accepted via RFC-020, see RFC-020: Board outline (scoped DXF profile extraction) + oriented placement + DR-026 (+ same-day amendment). Corrects an unauthorized implementation: board_outline/place were built directly on main (no RFC) with a rectangle-authoring shape and coordinate-only placement — Tony's direct review identified both as real design defects (a board outline is a mechanical-engineering DXF artifact, not a CoHDL-authored rectangle; placement needs rotation, the actual cause of a real Quilter failure). This section documents the corrected, Accepted design, revised twice further same day: board_outline requires CoHDL to actually extract the outline geometry from the referenced DXF (a reference-only design cannot produce IPC-2581's required inline Profile geometry); place is scoped to top-level instances only, with reaching into a called fn explicitly deferred (see Not yet specified).

Board outline — scoped extraction of one entity from a referenced DXF:

```cohdl
design Pico2 {
    layout {
        board_outline: "mechanical/pico2-outline.dxf"
    }
}
```

- board_outline: "path" — a single string-literal path, relative to the project root (same convention as #[doc(...)], RFC-017). At most one per design, design-top-level only.
- At cohdl build, CoHDL opens the referenced DXF and extracts exactly one designated outline entity — by convention, a closed LWPOLYLINE/POLYLINE on a fixed, documented layer name (the convention is emitter documentation, not fixed in the .cohdl grammar — see Tooling & operations below). Straight segments and arc bulges are both supported. Everything else in the DXF is never read — other layers, entities, text, dimensions are out of scope, the same narrow-contract discipline pad/footprint (RFC-018) established for pad geometry. CoHDL is not, and does not become, a general DXF/mechanical-CAD parser.
- A missing, malformed, non-closed, or unparseable outline entity is a compile error at cohdl build (an E1006 sub-case) naming the specific problem.
- The extracted geometry is embedded directly in IPC-2581's Profile/Polygon element and in layout.json — this is what makes the emitted document actually Quilter-importable. CoHDL still performs no validation of the outline's mechanical sensibility beyond confirming it's one closed loop — self-intersection, manufacturability, and real-world correctness remain the mechanical engineer's/CAD tool's responsibility.

Placement — coordinates + a closed-set rotation, top-level instances only:

```cohdl
layout {
    place hdr at (0mm, 0mm) rotate 90
}
```

- place at (x, y) [rotate ANGLE] — rotate is optional (default 0, unrotated); ANGLE is one of a closed set: 0, 90, 180, 270 — not an open-ended angle unit type.
- at's two Length-typed values, design-top-level-only restriction, and at-most-one-placement-per-instance are unchanged from the construct's original (now-corrected) shape. names a top-level instance of the design only — an instance created inside a called fn is not reachable by place; this is a real, disclosed, deferred gap (see Not yet specified), not silently solved.
- cohdl build passes the rotation value through unchanged into IPC-2581's Component/Location rotation attribute and layout.json — CoHDL performs no rotation math, no collision reasoning against the rotated footprint's actual extent. A declared fact for a partner tool to act on, identical in spirit to #[placement_hint(...)]'s existing discipline.

Rules (both constructs):

- Both stay non-DRC — structural, checked at declaration/build time (well-formed path string / at-most-one / outline-entity-exists-and-closes for board outline; closed-set membership + top-level-instance-lookup for placement) — never emergent-across-the-graph checks.
- Zero schematic-correctness impact, by construction — same guarantee RFC-013 established for every layout-adjacent construct; neither construct is read by the type checker, residual DRC, designator allocator, or .net/BOM emitters.
- Error codes stay in the existing E10xx family (E1006 gains real sub-cases: missing/malformed/non-closed outline entity, unparseable DXF; E1007 gains the rotation sub-case) — no new block, per RFC-011's "kind of mistake" organizing principle.
- No general 2D geometry/CAD authoring syntax exists in .cohdl — this is a scoped reference-and-extraction mechanism, not a geometry-authoring one. No arbitrary-angle rotation, rotation math, or collision/interference checking — all explicitly out of scope, consistent with DR-003's "layout/routing stays a partner concern" boundary.

## Component placement on the board's back side

Accepted via RFC-026, see RFC-026: Component placement on the board's back side + DR-032. Closes a real gap in the placement mechanism above: every place statement was implicitly, unconditionally top-side, with no way to express a real, universal PCB fact — dual-sided boards routinely place components (bulk decoupling caps, secondary connectors, backside shield tabs) on the bottom.

```cohdl
design DualSidedBoard {
    inst mcu: MCU_ESP32S3
    inst bulk_cap: MLCC<10uF, 16V>
    inst rf_shield_tab: RF_Shield_Contact

    layout {
        place mcu at (0mm, 0mm)
        place bulk_cap at (5mm, 5mm) side bottom
        place rf_shield_tab at (12mm, -3mm) side bottom rotate 180
    }
}
```

- place at (x, y) [rotate ANGLE] [side SIDE] — side is a new, optional clause on the existing place statement. SIDE is closed to {top, bottom}. Omitted side defaults to top — every existing place ... at (x, y) [rotate ANGLE] statement, written before this RFC, is unchanged in meaning.
- side and rotate are fully independent, composable clauses — either, both, or neither may appear. rotate's closed {0, 90, 180, 270} set is unchanged and applies identically regardless of side — a component rotated 90° on the bottom is rotated within its own (mirrored) bottom-side frame, the same convention every mainstream PCB tool already uses.
- The referenced instance's footprint (RFC-018) is authored exactly once, for its natural orientation — side bottom never requires, or permits, a second, separately-authored mirrored footprint declaration. Mirroring is a placement-time, emitter-level transform applied to the one real footprint declaration, never a second copy an author maintains by hand.
- Kept deliberately, fully independent from RFC-018's pad.layer (top_copper/bottom_copper/through_all) — that answers a narrower, different question (which single copper layer one pad occupies within an otherwise-fixed footprint), not "which side of the board is this whole component on." The two mechanisms are never merged or confused.
- cohdl build's KiCad .kicad_pcb emitter emits (layer B.Cu) (instead of the default F.Cu) on a side: bottom instance's placed footprint, and mirrors every one of the footprint's own pad coordinates (X-axis reflection, the real KiCad-native convention) before emission. The IPC-2581 emitter carries side via its own existing per-component side/layer attribute — no new IPC-2581 concept.
- side's closed-set membership is checked at declaration, identical in shape to rotate's existing check. No other new semantic check is introduced; side's value has no bearing on any existing check (pin obligations, trait satisfaction, designator allocation, footprint pad-count consistency).
- Not board-level layer stackup (how many copper layers a board has, their order) — that remains named future work per RFC-015's own disclosed gap. This RFC only concerns which of the two outer sides a component sits on.
- Error codes stay in the existing E10xx family (layout constraints, RFC-013/020's home for placement-related diagnostics): invalid side value on a placement — no new block.

# Not yet specified

The following constructs are referenced conversationally (in the Conceptual Model, note 2, or in v1-legacy context) but have no Accepted RFC yet, and therefore no entry above. Do not assume any specific syntax for these until an RFC lands:

- Skills (manufacturer best-practice guidance) — explicitly deferred per RFC-017's own direct decision; not yet even scoped (free-form doc vs. structured/checkable data is an open question).
- A richer padstack model (per-layer-independent geometry, vias, thermal reliefs) — explicitly out of scope per RFC-018, likely only meaningful once board outline/stackup (below) is addressed.
- General layer stackup as a real CoHDL concept — named future work per RFC-015, still not addressed by RFC-020 either (RFC-020 covers only the 2D board perimeter, not stackup).
- Arbitrary-angle (non-cardinal) rotation, or an open Angle unit type — explicitly deferred per RFC-020's own direct decision; a scoped future RFC if a real need emerges.
- place reaching an instance declared inside a called fn — explicitly deferred per Tony's direct decision (RFC-020/DR-026 amendment). place today resolves only against a design's own top-level instances; a component instantiated by a reusable sub-circuit fn (e.g. a connector helper) cannot currently be locked/oriented. A path-qualification mechanism was considered and withdrawn pending a real concrete need.
- Glob imports / re-export sugar for the module system — deferred per RFC-016, pending real usage friction.
- Board-level mounting holes, and any locating-hole shape beyond rect/circle/oval (true slots with rounded ends, keyed/D-shaped holes) — board-level holes explicitly deferred per RFC-022's own direct decision (closest existing analog is board_outline, RFC-020, but no construct exists yet); non-rect/circle/oval shapes explicitly deferred per RFC-023's own direct decision.
- A general loop/iteration construct (e.g. auto-generating daisy-chain net wiring between consecutive array elements, or arithmetic-derived per-instance place/decouple data such as grid-formula coordinates) — explicitly deferred per RFC-024's own direct decision; array-typed instances (NAME: [Device; N], NAME[i] indexing) are the foundation such a construct would iterate over, but every daisy-chain net and every per-element place/decouple statement stays hand-written, one at a time, today.
- Multi-dimensional array-typed instances (e.g. sw[row][col]) — explicitly deferred per RFC-024's own direct decision; no concrete need has been shown (OpenMicro's own keyboard matrix is expressed via ROW/COL nets, not a 2D instance grid).
- Everything else in the Conceptual Model (Part, Instance, Net, Design) whose concrete syntax/semantics hasn't been directly pinned down by an Accepted RFC beyond what's already threaded through the sections above — note 2 describes their intended shape and philosophy in full.

As of 2026-07-20, RFC-001 through RFC-028 are all Accepted (RFC-017 revised same day per Tony's footprint-scope correction; RFC-018 gives RFC-017's placeholder footprint keyword real pad/footprint content, corrected same day from invented names copad/cofp to plain pad/footprint; RFC-019 packages the already-Accepted cohdl lsp for real VS Code use; RFC-020 corrects an unauthorized board-outline/placement implementation per Tony's direct review, revised twice further same day to require real scoped DXF geometry extraction and to explicitly defer fn-nested placement rather than solve it speculatively; RFC-021 adopts IPC-7351 as CoHDL's canonical footprint naming practice, revised twice same day per Tony's direct corrections; RFC-022 adds mount_hole, a footprint-body construct for mechanical locating holes disjoint from pad's pin-bound numbering, grounded in KiCad's np_thru_hole precedent; RFC-023 extends mount_hole with an optional shape:/size: pair, reusing RFC-018's existing PadShape enum, grounded in a real datasheet — the Kailh Choc V2 switch's rectangular mounting legs; RFC-024 adds array-typed instances (inst NAME: [Device; N]) with real, indexed instance references (NAME[i]) valid everywhere an ordinary instance reference already is — net members, place, decouple, fn-call arguments — redesigned same day from an initial name-expansion-sugar draft per Tony's direct correction, grounded in the real OpenMicro macropad's 42-instance repetition and its real WS2812 daisy-chain wiring/per-LED placement needs, explicitly not introducing a loop construct or auto-generating daisy-chain/grid-place data; RFC-025 adds an optional rotate clause to pad placements inside footprint, reusing RFC-020's exact closed {0, 90, 180, 270} rotation set and keyword by direct precedent, grounded in a real KiCad QFN footprint's per-side-rotated-pad pattern, deliberately not adopting the KiCad-library convention of silently swapping pad width/height instead; RFC-026 adds an optional side clause to place, closed to {top, bottom}, defaulting to top, fully independent of and composable with rotate, grounded in the real KiCad .kicad_pcb per-component layer/mirroring mechanism, deliberately kept distinct from RFC-018's unrelated pad.layer concept; RFC-027 adds seven structured Quilter physics-constraint attributes (#[ground(...)], #[high_current(...)], #[impedance(...)], #[bypass(...)], #[crystal_oscillator(...)], #[switching_converter(...)], #[bga_fanout]) attached directly to the net/inst declaration each fact describes, reusing the existing #[name(...)] attribute-bracket syntax, redesigned same day from an initial seven-new-bare-keyword draft per Tony's direct correction, plus an additive optional bracket on diff_pair (RFC-013) for Quilter's three extra numeric fields; grounded in eight real CSV files Tony supplied matching Quilter's own documented Physics Constraints schema, explicitly not auto-inferring any constraint; RFC-028 extends #[bypass(...)], #[crystal_oscillator(...)], and #[switching_converter(...)]'s target/instance arguments to also accept a bare Pin-typed fn parameter, reusing the existing resolve_pin_ref/Binding::Pin machinery (RFC-006) confirmed real in src/check/expand.rs, closing a real gap where a reusable decoupling fn's own internal bypass capacitor could not carry #[bypass(...)] at all — zero new grammar, zero new binding concept, purely a checker correction with each real call site producing its own independently-resolved CSV row).
