# RFC-008: Exhaustive pattern-matching over structural variants

## Problem

Two real, already-encountered needs point at the same missing mechanism. First, the MVP implementation's `provisional-syntax.md` already introduced a **closed set consumed non-exhaustively**: pin roles (`input`, `output`, `bidirectional`, `passive`, `power_in`, `power_out`), where "unannotated" silently defaults to `passive` and the residual-DRC driver rules only ever inspect two of the six roles, with the other four handled implicitly by "isn't a driver role." Second, `provisional-syntax.md`'s cut list explicitly names **package/footprint variants** (`pins[VARIANT]`, `spec[VARIANT]`) as needed for any real device library but deliberately left unspecified — the same device (e.g. an MLCC) legitimately has different pin numbering or footprint depending on package (`C0402` vs `C0603` vs `C0805`), and nothing in the language today expresses "here are the finite shapes this device can take, and every one must be handled."

Both are the same underlying gap: CoHDL has closed, finite sets of structural variants (pin roles today; package variants tomorrow, and likely differential-pair roles, AVL alternates, or other closed enumerations later) with **no compiler-enforced exhaustiveness** — a case can be silently unhandled, exactly the "forgot to handle a case" smell note 2's Conceptual Model named as the motivating use case for this mechanism from the start.

Who this is for: **std-library device authors** (human or AI) who need to express "this device has N package variants, each with different pins/specs, and I must account for all of them" — and, more subtly, **the compiler itself**, which currently has an unexamined implicit default (pin role → `passive`) that this RFC should make an explicit, checked exhaustive match instead of a silent fallback.

## Goals

- Give devices a way to declare **package/footprint variants** — a finite, closed set of structural shapes a single device type can take, each with its own pin numbering and/or spec overrides.
- Make **selecting/handling a variant exhaustive**: wherever the compiler or a device definition needs to do something different per variant, an unhandled variant is a compile error, not a silent fallback to a default.
- Retrofit the existing pin-role mechanism to use the same exhaustiveness discipline — replacing the implicit "unannotated defaults to `passive`" convention with an explicit exhaustive treatment, closing the exact "silent default" gap the mechanism was introduced to avoid in the first place.

## Non-goals

- **Not a general pattern-matching expression language** (no `match` over arbitrary values, no destructuring beyond the closed variant sets this RFC defines). CoHDL is not gaining a general control-flow/expression system — this stays narrowly scoped to structural variants on devices and pins, consistent with the redesign's small, orthogonal-concept discipline.
- **Not solving **`module`**/**`use`** or general visibility.** Package variants are declared within the existing single-flat-scope model (`provisional-syntax.md` §1); this RFC does not depend on or block the eventual module RFC.
- **Not changing the four residual-DRC rules' logic** (RFC-004) — D003/D004 already consume pin roles; this RFC changes how roles are declared/defaulted, not what the DRC rules do with them.

## Design

### Package/footprint variants — a closed set declared on the device, matched exhaustively per use

```cohdl
pub device MLCC<C: Capacitance, V: Voltage = 10V, T: Tolerance = 10%> {
    variants { C0402, C0603, C0805 }

    pins[C0402] { A: 1, B: 2 }
    pins[C0603] { A: 1, B: 2 }
    pins[C0805] { A: 1, B: 2 }

    spec { capacitance: C, voltage_rating: V, tolerance: T }
    spec[C0402] { max_capacitance: 100nF }   // variant-specific spec override/addition
}
```

- `variants { ... }` declares the device's closed, finite set of structural variants — a plain identifier list, checked at parse time for duplicates.
- `pins[VARIANT] { ... }` declares that variant's pin layout. **Every variant listed in **`variants {}`** must have a **`pins[VARIANT]`** block — this is the exhaustiveness check**: a device with 3 declared variants but only 2 `pins[...]` blocks is a compile error naming the missing variant.
- `spec[VARIANT] { ... }` is optional per variant — a variant-specific addition or override to the device's base `spec {}`. Omitting it for a variant is fine (it just means that variant has no additional/overridden fields) — exhaustiveness applies to `pins[VARIANT]` (every variant must have a pin layout, since a device with no pins for some variant is meaningless) but not to `spec[VARIANT]` (a variant needing no spec override is a legitimate, common case).
- An instance selects its variant via a generic-style argument: `inst c1: MLCC<100nF, 16V, 10%>[C0603]` — the `[VARIANT]` suffix is required whenever a device declares `variants {}`; omitting it is a compile error (no implicit "pick the first/default variant" — consistent with "no magic defaults").

### Pin roles — retrofitted to the same exhaustiveness discipline

The existing provisional pin-role mechanism (`input`/`output`/`bidirectional`/`passive`/`power_in`/`power_out`) is unified with this RFC's variant-matching discipline, closing its one silent default:

```cohdl
pub device AP2112K_3V3 {
    pins {
        required VIN:  1 [power_in]
        required GND:  2 [power_in]
        required EN:   3 [input]
        optional NC:   4 [passive]     // now explicit, not an implicit default
        required VOUT: 5 [power_out]
    }
}
```

**Every pin must now carry an explicit role annotation.** The previous "unannotated defaults to `passive`" convention is retired — this was itself a small instance of the "silent default" smell this RFC's mechanism exists to eliminate. A pin with no role annotation is a compile error naming the pin and listing the six valid roles.

### Where residual DRC consumes roles — unchanged logic, now over a fully-explicit input

D003/D004 (single-driver, multi-driver) continue to classify `output`/`power_out` as driver roles and everything else as non-driver — this RFC does not change that classification logic, only guarantees every pin the DRC rules read has an explicit role rather than a silently-defaulted one.

## Type-system-first test

N/A — this RFC is a type-system mechanism (exhaustiveness checking on closed variant sets), not a `rule`/DRC proposal. It touches what residual DRC *reads* (pin roles) but adds no new DRC rule and changes no DRC logic.

## Conceptual impact

Extends **Device** and **Pin** (existing concepts) with a formalized closed-variant mechanism — no new core concept in the canonical vocabulary. `variants {}`/`pins[VARIANT]`/`spec[VARIANT]` is new grammar surface but reuses the existing `pins {}`/`spec {}` block shapes with a bracket-suffix qualifier, consistent with note 4's "prefer extending an existing concept over inventing a parallel mechanism." Retiring the pin-role default is a simplification (one fewer implicit rule to remember), not an addition.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Med | Med | Med | High (breaks all existing pin declarations + any device using implicit `passive`) | High |

**Compat (High):** this is the first RFC in the v2 backlog with a real compatibility cost — the MVP implementation already shipped with an implicit `passive` default; retiring it breaks every existing device declaration that relies on it. This must be called out honestly, not glossed over (see Migration path).
**Trust (High):** eliminating the one silent default the pin-role mechanism had directly strengthens the "no magic defaults" principle's actual enforcement, not just its stated intent.
**Oracle/Diagnostics/Netlist (Med):** package variants are a real new correctness surface (an instance's pin numbering now depends on its selected variant, which propagates to netlist output), but the underlying mechanism (structural matching, exhaustiveness) is well-understood and low-risk once specified.
**Grammar (Med):** `variants {}`/bracket-suffix blocks are new but small, regular grammar additions.

## Gradeability

Enforced at **type-check time**: the `variants {}` declaration and its required `pins[VARIANT]` blocks are checked for completeness the moment a device is declared (paralleling RFC-003's impl-time discipline) — a device missing a `pins[VARIANT]` block for a declared variant is a compile error at the device's own declaration, before any instance ever selects that variant. An instance's `[VARIANT]` selector is checked at the instantiation site (paralleling RFC-007's generic-instantiation checking) — selecting an undeclared variant name is a compile error naming the valid variant set. The retrofitted pin-role requirement is checked at the same point pin declarations are already checked (RFC-002's obligation-kind parsing) — no new pipeline stage, an extension of an existing one.

## AI-generatability

High for the new mechanism (a model already writing `pins {}`/`spec {}` blocks learns one new bracket-suffix convention, directly parallel to what it already knows), but the pin-role retrofit has a real one-time cost: existing generated `.cohdl` source relying on the implicit `passive` default will need every OK unannotated pin updated — a mechanical, compiler-driven fix (the diagnostic names the exact pin), not a conceptual one, but real generation-time cost for any AI regenerating pre-RFC-008 designs.

## Alternatives

- **Leave pin roles with their implicit default, add only package variants** — rejected: this would ship a new exhaustiveness mechanism while leaving an existing silent default in place, an inconsistent application of the RFC's own principle; if exhaustive matching is worth doing for variants, it's worth doing for the closed set that already existed.
- **A general **`match`**/pattern-matching expression language** (arbitrary destructuring, guards, etc.) — rejected per Non-goals: no concrete need beyond closed structural variants exists yet; a general expression language would be conceptual cost far beyond the job actually required, contradicting the redesign's narrow-generics precedent (RFC-007's explicit rejection of const-generics/richer bounds for the same reason).
- **Package variants as separate device types instead of one device with variants** (e.g. `MLCC_C0402`, `MLCC_C0603` as distinct devices) — considered, rejected: this is what a looser language would do, and it breaks the "one device, several shapes" modeling the Conceptual Model already anticipated; it would also duplicate every trait `impl` per variant-as-separate-device, a real composability regression.
- **A wildcard/default variant arm** (e.g. `pins[_]` as a catch-all) — rejected: this reintroduces exactly the "silent unhandled case" risk exhaustiveness exists to close; every variant must be named explicitly, no catch-all.

## Compatibility

**Breaks all existing pin declarations without an explicit role** — every device in the MVP-scope std library (and the demo board) must be updated to add explicit role annotations; this is a one-time, mechanical, compiler-flagged migration (see Migration path), not a silent behavior change. Package variants are additive (no existing device without `variants {}` is affected).

## Tooling & operations

- The "missing role annotation" diagnostic must list the six valid roles directly in the message, so an AI repair loop doesn't need to look them up elsewhere.
- The "missing `pins[VARIANT]` block" diagnostic must name the specific missing variant(s), not just "incomplete variant coverage."
- Reserve error-code sub-blocks for: missing pin-role annotation, undeclared variant selected at instantiation, missing `pins[VARIANT]` block for a declared variant — three distinct diagnostics, consistent with this backlog's established precision discipline.
- The informal error-code registry (`docs/error-codes.md`) must gain these entries in the same change, per the project's "ship with its spec update" discipline extended to the implementation's own registry.

## Teaching cost

Low. An author already writing `pins {}` learns one more required annotation per pin (the role) instead of an implicit convention to memorize — arguably lower teaching cost than the status quo, since "what's the default if I don't annotate" is no longer a question at all. Package variants are a natural extension of generics an author already understands from RFC-007.

## Failure modes

- **A model omits a pin-role annotation out of habit** (trained on the pre-RFC-008 convention) — must produce the specific, listed-roles diagnostic described in Tooling & operations, not a generic parse error, so the repair loop can fix it in one turn.
- **A device declares **`variants {}`** but a **`pins[VARIANT]`** block for one variant is copy-pasted incorrectly** (e.g. wrong pin count for that package) — this RFC's exhaustiveness check only confirms *presence* of a block per variant, not correctness of its content; a wrong pin layout for a real package is a data-accuracy problem for std-library review, not something this mechanism can catch (analogous to RFC-002's parallel caveat that `required`/`optional` miscategorization isn't compiler-catchable either).
- **An instance omits the **`[VARIANT]`** selector on a device that has variants** — must be a compile error naming the device's valid variant set, never a silent "pick the first one."

## Migration path

For the existing MVP-scope std library and demo board: every device's pin declarations need an explicit role annotation added (mechanical, one compiler-run to find every instance via the new diagnostic). No source semantically changes — every currently-unannotated pin becomes `[passive]` explicitly, preserving current DRC behavior exactly; this is a pure syntax-completeness fix, not a behavior change, despite being a breaking compatibility event for parsing.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-017 (see note 7). Language Specification (note 10) gains a "Structural variants" section covering both package variants and the retrofitted pin-role discipline. This RFC's migration (adding explicit pin roles everywhere) should be done in the same implementation pass as landing the RFC — per the project's "ship with its check" discipline, an Accepted RFC that changes required syntax ships together with a fixed std library, not a std library that's now retroactively non-compiling.
