# RFC-003: Trait-satisfaction-at-impl-time checking

## Revision note (2026-07-13, second revision)

Both prior drafts of this RFC were off-target. Draft 1 baked impl Trait directly into the device declaration (device MLCC<...>: impl Capacitor { ... }), which is structurally identical to C#'s class Foo : IBar — the type and its trait membership are declared as one inseparable unit. Draft 2 tried to fix "not flexible enough" by adding default methods, associated types, and blanket impls — all real Rust mechanisms, but none of them were the actual problem Tony pointed at. Tony's own example makes the real gap obvious:

```rust
pub trait TwoTerminal { /* ... */ }
pub trait Capacitor { /* ... */ }
pub trait Polarized { /* ... */ }

pub struct MLCC { /* ... */ }
impl TwoTerminal for MLCC {}
impl Capacitor for MLCC {}
```

The trait and the type are declared completely independently, and impl Trait for Type is its own free-standing declaration — not nested inside the type's own definition. A struct can gain new trait implementations later, in different modules, without ever touching its original definition, and the same struct can implement several unrelated traits via several separate impl blocks. This is the actual mechanism this RFC was missing. This third draft fixes exactly that, and nothing else — no default methods, no associated types, no blanket impls; those are real Rust features but they weren't what was asked for, and adding them now would be solving a problem Tony didn't raise while still not fixing the one he did.

## Problem

Both prior drafts modeled a device's trait membership as part of the device's own declaration syntax. That is the C#/Java-interface shape: a type's declaration is the only place it can ever claim to implement something. Real Rust traits decouple "what a type is" from "what a type implements" — you write the type once, and you write impl Trait for Type as its own separate statement, as many times as you want, wherever makes sense (often grouped by trait, not by type — e.g. all the TwoTerminal impls for many different devices sit together). This decoupling is what lets:

- A device be defined once and gain new trait implementations later, without editing the original device.
- Trait implementations for many different devices to be organized by trait (e.g. a file listing every impl TwoTerminal for X) instead of forcing every trait a device has to be declared at the device's own birth.
- A std-library author to implement a trait for a device defined elsewhere (in another module), the normal Rust pattern — not just self-declared traits at definition time.

Who this is for: std-library and device authors (human or AI), who should be able to write device MLCC { ... } once and then separately write impl Capacitor for MLCC { ... }, impl TwoTerminal for MLCC { ... }, etc. — matching how real trait-based composition actually reads and is organized in Rust, and avoiding forcing every trait onto one crowded device declaration.

## Goals

- Decouple impl Trait for Device from the device declaration — trait implementation is its own top-level statement, not a clause embedded in device { ... }.
- Keep the gradeability win from the very first draft: an impl block is checked for full satisfaction the moment it is declared — still the earliest possible point, still no instance or design required.
- Allow multiple, separate impl blocks for the same device, added anywhere in scope (including a different module from the device's own definition), each independently checked.
- Support sub-trait bounds (Capacitor: TwoTerminal) — a device must have a satisfying impl for every trait a claimed trait bounds on, but each can still be its own separate impl block.

## Non-goals

- Not adding default methods, associated types, or blanket/generic impls. These are real Rust mechanisms considered in the previous (rejected, unnecessary) revision, but they are a different question from the one this RFC is actually answering (decoupling impl from the type declaration). If any of them are wanted, they should be proposed as their own follow-up RFC, evaluated on their own merits — not bundled in here again.
- Not adding trait objects/dynamic dispatch. Unchanged reasoning from the prior draft: CoHDL designs are fully monomorphized at compile time; there is no runtime to dispatch at.
- Not defining orphan/coherence rules for conflicting impls of unrelated traits. A device may have as many independent impl Trait for Device blocks (for different traits) as needed; this RFC does not need a coherence rule because it does not introduce blanket/generic impls, which is the only place such conflicts could arise (see prior revision's now-withdrawn Non-goals for that discussion).

## Design

### Traits are declared independently of any type

Unchanged from earlier drafts in shape — a trait declares required pins (abstract roles, RFC-002 vocabulary) and required spec fields (unit-typed, RFC-001 vocabulary), plus optional sub-trait bounds:

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

pub trait Polarized {
    pins {
        required Anode: pin
        required Cathode: pin
    }
}
```

Traits never reference any specific device. This part was already correct in the first draft and is unchanged.

### Devices are declared independently of any trait

```cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    pins {
        A: 1
        B: 2
    }
    spec {
        capacitance: C
        voltage_rating: V
        tolerance: T
    }
}
```

This is the actual fix. device no longer has an : impl Trait clause at all. A device is just pins + specs — a plain, self-contained declaration, exactly like Tony's pub struct MLCC { /* ... */ }. Whether it satisfies any trait is not decided here, and doesn't need to be known here.

(Note: since pins {} on a bare device now needs no trait to reference, each pin's own required/optional obligation kind, per RFC-002, is declared directly on the device's own pins — unchanged mechanism, just no longer coupled to trait membership.)

### impl Trait for Device — a free-standing declaration, checked at the point it's written

```cohdl
impl TwoTerminal for MLCC {}
impl Capacitor for MLCC {}
```

- Each impl Trait for Device is its own top-level statement — not nested in, and not required to be co-located with, either the trait's or the device's own declaration. It can appear in the same module as the device, a different module, or a module dedicated to grouping implementations by trait (a common, idiomatic Rust organization this now makes possible in CoHDL too).
- A device may have any number of separate impl blocks, one per trait, added at any point (including after the device's original definition, in an entirely separate file) — this is the actual expressiveness gap the prior two drafts didn't close.
- Each impl Trait for Device block is still checked exhaustively, the moment it is written — the gradeability discipline from the first draft is fully preserved, just relocated to apply to the impl statement instead of a clause inside device:
- Sub-trait bounds still apply per-impl: writing impl Capacitor for MLCC requires the compiler to confirm TwoTerminal (which Capacitor bounds on) is also satisfied — either via an existing separate impl TwoTerminal for MLCC elsewhere in scope, or the compiler reports exactly that missing link (see Failure modes).

### Example: what this actually enables

```cohdl
// devices/passives.cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V> {
    pins { A: 1, B: 2 }
    spec { capacitance: C, voltage_rating: V }
}

pub device TantalumCap<C: Capacitance, V: Voltage = 10V> {
    pins { Anode: 1, Cathode: 2 }
    spec { capacitance: C, voltage_rating: V }
}

// traits/passive_traits.cohdl
pub trait TwoTerminal { pins { required A: pin, required B: pin } }
pub trait Polarized { pins { required Anode: pin, required Cathode: pin } }
pub trait Capacitor: TwoTerminal { spec { capacitance: Capacitance, voltage_rating: Voltage } }

// impls/capacitor_impls.cohdl — grouped by trait, the idiomatic Rust organization,
// now possible because impl is decoupled from both trait and device declarations
impl TwoTerminal for MLCC {}
impl Capacitor for MLCC {}

impl Polarized for TantalumCap {}
// note: TantalumCap does NOT implement Capacitor or TwoTerminal here —
// its pins are named Anode/Cathode, not A/B, so it would need its own
// impl TwoTerminal for TantalumCap if that trait were desired, or simply
// doesn't need to claim it — nothing forces every device into every trait
```

MLCC and TantalumCap are declared once, in devices/passives.cohdl, and never touched again. Their trait memberships live in impls/capacitor_impls.cohdl, organized by trait rather than scattered across every device's own declaration — exactly the organizational freedom the C#-like coupling in the first draft prevented.

## Type-system-first test

N/A — this RFC is a type-system mechanism, not a rule/DRC proposal.

## Conceptual impact

This changes where impl lives syntactically (a free-standing top-level statement instead of a clause on device), but the underlying concepts — Trait, Device — are unchanged in what they mean; only their relationship's syntax moves. This is a smaller conceptual change than the withdrawn second revision (no new mechanisms: no default methods, associated types, or blanket impls), while fixing the actual flexibility complaint. impl Trait for Device is a new top-level grammar form, but it introduces no new concept — it's the same trait-satisfaction relationship as before, just no longer forced into the device's declaration.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | High | High | Low | N/A (pre-launch) | High |

Concepts (Low, down from the withdrawn revision's High): no new concepts — decoupling impl's syntax from device's syntax doesn't touch the canonical vocabulary at all.
Grammar (Med): impl Trait for Device is new top-level grammar, but it's a single, simple, regular new statement shape — much smaller surface than the withdrawn revision's associated-type/blanket-impl grammar.
Oracle/Diagnostics (High): unchanged from the first draft's genuine strength — the check is still exhaustive and still runs at the earliest possible point (now: at the impl statement, rather than inside device).
Trust (High, restored from the withdrawn revision's Med): because there are no default methods or blanket impls, a device's full trait surface is still 100% determined by reading its explicit impl blocks — no hidden inheritance to trace. This is actually better for review organization than the first draft, since impls can be grouped in one place per trait, making an audit of "which devices implement Capacitor" a single-file read instead of a scan across every device declaration.

## Gradeability

Still enforced entirely at type-check time, at the earliest possible point — now specifically the point an impl Trait for Device statement is itself parsed and type-checked, wherever in the module tree it appears. No instance, no design, no rule/DRC involvement. Sub-trait bounds are still checked transitively: an impl Capacitor for MLCC requires the compiler to locate (anywhere in the currently-compiled scope) a satisfying impl TwoTerminal for MLCC, or report the gap precisely (see Failure modes) — this is the one place the check must look slightly beyond the single impl statement itself, to find sibling impl blocks for the same device, but this is still a compile-time, pre-instance, pre-design check.

## AI-generatability

High, and arguably improved over both prior drafts. A model can now generate a device once and, in a wholly separate step or file, generate impl Trait for Device statements — mirroring exactly how it would generate idiomatic Rust trait impls, which is presumably well-represented in any model's training data. The one added generation requirement: if a trait has sub-trait bounds, the model must ensure a satisfying impl exists somewhere in scope for each bound, not necessarily in the same statement — the diagnostic (see Tooling & operations) must make any gap immediately locatable so a repair loop doesn't need to search blindly.

## Alternatives

- Keep impl embedded in the device declaration (both prior drafts) — rejected: this is the exact C#/interface-shaped rigidity Tony flagged; it prevents adding a trait implementation after the fact or organizing implementations by trait rather than by device.
- Add default methods / associated types / blanket impls (the withdrawn second revision) — rejected as solving a different problem than the one raised; these remain available as independent future RFCs if genuinely wanted, but bundling them here was scope creep against the actual, narrower complaint.
- Require all impl blocks for a device to be co-located with the device's declaration (a softer coupling than embedding, but still same-file) — rejected: doesn't achieve the "group impls by trait" organizational freedom that's the actual point of decoupling; a real fix should allow impl blocks anywhere in scope, not just anywhere in the same file as the device.

## Compatibility

N/A — pre-launch. This third draft supersedes both the original fixed-shape-embedded-impl draft and the default-methods/associated-types/blanket-impls revision, both decided and reverted the same day; DR-013 (note 7) is being rewritten in place to reflect only this final design.

## Tooling & operations

- A missing sub-trait-bound implementation must produce a diagnostic naming which trait is missing an impl for which device, distinctly from a same-impl pin/spec mismatch (e.g. "impl Capacitor for MLCC requires impl TwoTerminal for MLCC, which was not found in scope" vs. "impl Capacitor for MLCC is missing required spec field voltage_rating").
- The LSP should offer "find all impls for this device" and "find all impls of this trait" navigation — since impls are no longer textually attached to either the device or the trait's own declaration, tooling must make both directions of lookup trivial, or the decoupling could otherwise make it harder (not easier) to answer "what does this device implement."
- Reserve an error-code sub-block for trait-satisfaction diagnostics (single impl mismatch, and separately, missing-sub-trait-bound-impl) — two distinct codes, not one, per the distinction above.

## Teaching cost

Low — this is a simplification relative to the withdrawn second revision, and arguably a wash relative to the very first draft (same total concepts: traits, devices, satisfaction-checking; just a different, more standard syntactic arrangement). Anyone who has read introductory Rust already has the right mental model.

## Failure modes

- A device has impl Capacitor for MLCC but no impl TwoTerminal for MLCC anywhere in scope — must be a compile error at the impl Capacitor statement itself (not deferred), naming the missing sibling impl explicitly (see Tooling & operations) so it's immediately actionable.
- Two conflicting impl Trait for Device statements for the exact same trait and device (e.g. accidentally duplicated) — must be a compile error at the second occurrence, naming the earlier one's location.
- A model assumes a device automatically implements a trait because its pins/specs happen to match, without writing the impl statement — must fail to compile; there is still no structural typing (unchanged principle from every prior draft) — an explicit impl is always required for a device to be considered a member of a trait, regardless of shape compatibility.

## Migration path

N/A — pre-launch.

## Decision

Accepted (third draft, final) — 2026-07-13. Supersedes both the embedded-impl-in-device draft and the default-methods/associated-types/blanket-impls revision from earlier the same day. DR-013 (note 7) rewritten in place to reflect only this design. Language Specification (note 10)'s Traits section will be rewritten to match. RFC-007 (generics-over-specs) should assume impl Trait for Device is a free-standing statement when it specifies how generic parameter bounds interact with trait satisfaction.
