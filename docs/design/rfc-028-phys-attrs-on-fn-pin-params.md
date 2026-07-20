# RFC-028: Physics-constraint attributes on fn Pin parameters

## Problem

Grounded directly in a real gap surfaced applying RFC-027 to a real design: which of a board's `inst`-declared capacitors should carry `#[bypass(...)]`? The clean, semantically-correct answer for a decoupling capacitor instantiated via a reusable `fn` — the common, idiomatic pattern RFC-006 exists to support —

```cohdl
pub fn decouple(vdd: Pin, gnd: Pin) {
    inst c: C_100n
    net _: vdd, c.A
    net _: gnd, c.B
}
```

cannot be expressed today. RFC-027's `#[bypass(INST.PIN, CAPACITANCE)]` requires a literal `INST.PIN` reference — an already-declared top-level instance name plus one of its pins. Inside `decouple`'s body, the bypassed pin is the `Pin`-typed parameter `vdd`, not a named top-level instance's pin — there is no `INST.PIN` an author can write. The only way to carry the attribute today would be to inline all 25 real call sites into 25 hand-written capacitor instances, destroying exactly the abstraction RFC-006 exists to provide — a real, disclosed regression, not a hypothetical one.

Confirmed against real source (`src/check/expand.rs`): `resolve_pin_ref` — the existing, single mechanism that resolves any bare `PinRef` — already checks `scope.bindings` first and resolves a `fn` parameter of type `Pin` through `Binding::Pin((String, String))`, a real `(instance path, logical pin name)` pair populated per call site (`bindings.insert(param.name.name.clone(), Binding::Pin(target))`, confirmed at the call-expansion site). This is exactly the same resolution path RFC-027's own `#[bypass(...)]`, `#[crystal_oscillator(...)]`, and `#[switching_converter(...)]` target arguments already use for a top-level `INST.PIN`/`INST` reference — the gap is not that resolution is impossible, but that RFC-027's own design (written before this case was considered) implicitly assumed its target arguments would only ever appear at the design's top level, never inside a `fn` body.

## Goals

- Let `#[bypass(...)]`'s (and every other RFC-027 attribute's instance/pin-reference argument) target a `fn`'s own `Pin`-typed parameter, not only a top-level `INST.PIN`, so a decoupling `fn`'s own internal capacitor can carry the attribute.
- Each real call site of the `fn` produces its own real, individually-resolved CSV row — the bypassed pin resolves to whatever concrete `(instance, pin)` that specific call site actually binds, exactly as if the `fn`'s body had been hand-inlined at that call site (RFC-006's own expansion discipline, applied here to attribute-argument resolution rather than to `inst`/`net` statements).
- Extend all three RFC-027 attributes whose target argument is a pin/instance reference (`#[bypass]`, `#[crystal_oscillator]`, `#[switching_converter]`) uniformly, not `#[bypass]` alone — the same underlying gap (a `fn`-internal reference that RFC-027 didn't anticipate) applies identically to all three.

## Non-goals

- **Not a new resolution mechanism.** `resolve_pin_ref` and `Binding::Pin` already exist and already do exactly this job for every other bare `PinRef` in the language (net members, `place`/`decouple` arguments, per RFC-006's own nested-call design). This RFC does not add a new binding/resolution concept — it only extends which grammatical positions RFC-027's attribute-argument parser is willing to accept a bare (unqualified) `PinRef` in, and confirms that the checker calls the same existing `resolve_pin_ref` function for that position it already calls for every other pin reference.
- **Not new semantics for an attribute inside a **`fn`** body vs. at the top level.** `#[bypass(vdd, 100nF)]` inside `decouple`'s body means exactly what `#[bypass(mcu.VDD, 100nF)]` would mean if hand-written at a top-level `inst c: C_100n` line targeting the same pin — the attribute's meaning is unchanged; only the argument's resolution path differs (through a call-site binding instead of a literal top-level name), and that resolution happens once per real call site, at expansion.
- **Not extending **`#[ground(...)]`**, **`#[high_current(...)]`**, or **`#[impedance(...)]`**.** These three attach to a `net`, not an instance/pin reference — a `net`'s own name is already resolvable identically whether declared inside a `fn` body or at the top level (RFC-006's existing per-call-site net-naming scheme, unchanged since RFC-024's DR-030). There is no analogous gap for these three; this RFC does not touch them.
- **Not solving the general "attribute argument grammar" question for hypothetical future attributes.** This RFC extends exactly the three RFC-027 attributes whose argument shape is a bare pin/instance reference, closing a real, concrete gap RFC-027 itself left open — not a speculative general mechanism for arbitrary future attribute argument kinds.

## Design

```cohdl
pub fn decouple(vdd: Pin, gnd: Pin) {
    #[bypass(vdd, 100nF)]
    inst c: C_100n
    net _: vdd, c.A
    net _: gnd, c.B
}

design Board {
    inst mcu: MCU_ESP32S3
    // ... real call sites, one per bypassed pin ...
    decouple(mcu.VDD1, mcu.GND)
    decouple(mcu.VDD2, mcu.GND)
    // ... 23 more ...
}
```

- `#[bypass(TARGET, CAPACITANCE)]`'s `TARGET` argument (previously `INST.PIN` only) now also accepts a bare `Pin`-typed `fn` parameter name — the exact same `PinRef` grammar the language already has (an identifier, optionally followed by `.pin`), simply used with the `.pin` part omitted, which is already valid `PinRef` syntax everywhere else in the language (e.g. inside a `net` member list: `net _: vdd, c.A` — `vdd` there is already a bare, unqualified `PinRef`).
- The same extension applies identically to `#[crystal_oscillator(PARENT_INST, PIN_1, PIN_2)]`'s three arguments and to `#[switching_converter(inductor: INST, ...)]`'s instance arguments, wherever the referenced value is itself a `Pin`-typed `fn` parameter rather than a top-level instance name.
- Checking reuses `resolve_pin_ref` (or the existing instance-argument equivalent for `#[switching_converter]`'s whole-instance arguments) exactly as-is — no new function, no new binding table. The checker for these three attributes' target arguments is corrected to call the same resolver every other pin/instance reference in the language already calls, rather than assuming a top-level-only name.
- At expansion (RFC-006's existing per-call-site inlining pass), each real call to a `fn` containing an attribute-bearing `inst` produces its own real, fully-resolved attribute instance — the attribute's target resolves to that call site's actual bound `(instance, pin)`, exactly as RFC-006 already does for every `inst`/`net` statement inside a called `fn`'s body. A `fn` called 25 times with an attribute inside it produces 25 real, independently-resolved attribute facts, one per call site — never one shared/ambiguous fact for the whole `fn` definition.

## Type-system-first test

Not a `rule`/DRC proposal — the check is the exact same reference-resolution check RFC-027 already specified for these three attributes' target arguments, now simply applied at the correct, already-existing resolution function (`resolve_pin_ref`) rather than a narrower top-level-only lookup. No new check is introduced; an unresolvable reference (e.g. a plain identifier that is neither a top-level instance name nor a bound `fn` parameter in scope) is still a compile error naming what wasn't found — unchanged in kind from RFC-027's own diagnostic.

## Conceptual impact

Low. No new core concept, no new binding mechanism, no new grammar token. This RFC corrects a real, narrow oversight in RFC-027's own target-argument checker — extending which existing, already-resolvable reference forms (a bare `Pin`-typed `fn` parameter, which the language's `PinRef` grammar and `resolve_pin_ref` function already fully support everywhere else) three specific attributes are willing to accept.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Low | Low | High |

**Grammar (Low):** no new syntax — a bare `PinRef` (identifier, no `.pin`) is already valid grammar everywhere in the language; this RFC only widens which attribute-argument positions accept it.
**Netlist (Low):** the eight CSV files RFC-027 already specified are unaffected in shape — this RFC only affects how many rows are produced and what each row's target-reference field resolves to per real call site; no new file, no new column.
**Trust (High):** each attribute instance, after resolution, is exactly as real and individually-checked as a hand-written top-level one — RFC-006's own per-call-site expansion discipline is reused unchanged, not weakened.
**Compat (Low):** purely additive; every existing top-level `#[bypass(INST.PIN, ...)]`/`#[crystal_oscillator(...)]`/`#[switching_converter(...)]` attribute (written before this RFC) is unaffected, unchanged in meaning.

## Gradeability

- Reference resolution for these three attributes' target arguments now runs through `resolve_pin_ref`/the existing instance-resolution equivalent, at the same point (per real call site, during RFC-006's expansion pass) every other `fn`-body pin/instance reference is already resolved.
- An unresolvable reference (an identifier that is neither a bound `fn` parameter nor a top-level instance in scope) remains a compile error, unchanged in kind and code from RFC-027.
- No new residual-DRC surface — see Type-system-first test above.

## AI-generatability

High. A model that already understands `fn` parameter binding (RFC-006) and `#[bypass(...)]`'s shape (RFC-027) needs to learn exactly one fact: the same bare-identifier form already legal inside a `net` member list (`net _: vdd, c.A`) is equally legal as an RFC-027 attribute's target argument. No new syntax to memorize, no special case distinguishing "top-level bypass" from "fn-internal bypass" beyond which name happens to be in scope.

## Alternatives

- **Require flattening every **`fn`** call containing a bypass-worthy capacitor into a hand-written top-level **`inst` — this is the real, rejected status quo this RFC exists to fix: it destroys RFC-006's entire reason for existing (reusable, parameterized sub-circuits) for the sole purpose of attaching one attribute, a real and disproportionate cost.
- **Leave **`fn`**-internal bypass capacitors permanently un-annotatable, relying solely on Quilter's own auto-detection for these** — this was the interim, honestly-disclosed position before this RFC (RFC-027's own Non-goals already frame omission as a designed-for, not broken, state: an author who omits the attribute gets Quilter's real auto-detection, which runs on the plain netlist regardless). This RFC is not required for CoHDL to function — Quilter's detector already handles the unannotated case correctly, as the real supplied `bypass_capacitors.csv` (25 real `C→U2` rows, auto-detected) demonstrates. This RFC is accepted because it closes a real, avoidable gap between "what an author can state explicitly" and "what is semantically true," at essentially zero marginal mechanism cost (the resolver already exists) — not because the unannotated state was ever broken.
- **A new, separate **`fn`**-scoped attribute-declaration mechanism, distinct from RFC-027's existing attributes** — rejected: the existing attributes' grammar (a bare `PinRef`/instance-name argument) already covers this case once the checker calls the resolver that already exists; inventing a parallel mechanism would duplicate machinery for no benefit, the same "prefer extending an existing concept" discipline this project applies throughout (e.g. RFC-016's DR-022 rationale).
- **Extend all seven RFC-027 attributes uniformly, including the three net-attached ones** — considered and rejected: `#[ground]`/`#[high_current]`/`#[impedance]` attach to a `net`, and a `net`'s own name already resolves identically per call site whether declared inside a `fn` body or at the top level (unchanged since RFC-024/RFC-006) — there is no analogous unresolvable-reference gap for these three, so extending them would be addressing a problem that doesn't exist for that subset.

## Compatibility

Purely additive. Every existing `#[bypass(INST.PIN, ...)]`, `#[crystal_oscillator(INST, PIN_1, PIN_2)]`, and `#[switching_converter(inductor: INST, ...)]` attribute at the design top level is completely unaffected, unchanged in meaning and in every emitted CSV byte.

**Depends on**: RFC-027 (Quilter physics-constraint hints, already Accepted) — this RFC extends three of its seven attributes' argument-resolution rules. Reuses RFC-006's existing per-call-site expansion and `Binding`/`resolve_pin_ref` machinery unchanged — no new dependency.

## Tooling & operations

- `cohdl build`'s CSV emission for `bypass_capacitors.csv`/`crystal_oscillators.csv`/`switching_converters.csv` is unchanged in format — each real call site simply now may contribute a real row where before it silently could not carry the attribute at all.
- `cohdl fmt` requires no new rule — the attribute's canonical single-line form is unchanged; only the identifier appearing inside its argument list may now be a `fn` parameter name instead of a top-level instance name, which `fmt` already renders identically (both are just identifiers).
- No new error-code sub-case beyond what RFC-027 already reserved (unresolved pin/instance reference) — the same diagnostic now correctly fires (or doesn't) for a `fn`-parameter reference the same way it already does for a top-level one.

## Teaching cost

Very low. An author who already knows `fn` parameters can be referenced as bare pins inside a `net` member list (RFC-006's own established pattern) needs no new mental model — the exact same reference form is now also legal as one of these three attributes' target arguments.

## Failure modes

- **An author writes **`#[bypass(vdd, 100nF)]`** outside any **`fn`** body, where **`vdd`** isn't a bound parameter or a real top-level instance** — caught immediately as an unresolved reference, the same diagnostic RFC-027 already specifies.
- **An author expects one shared CSV row for a **`fn`** definition rather than one row per call site** — incorrect; every real call site that binds the attribute-bearing parameter produces its own independent row, exactly mirroring RFC-006's existing "expansion produces what hand-writing would have" discipline.
- **An author tries to extend this pattern to **`#[ground]`**/**`#[high_current]`**/**`#[impedance]` — these three are unaffected by this RFC (see Non-goals); a `net`'s name was never the gap.

## Migration path

No existing design requires migration — this is purely additive new-reference-form support. A real, optional, non-mechanical follow-up: any existing `decouple`-style `fn` whose internal capacitor is a genuine bypass cap can now add `#[bypass(vdd, 100nF)]` (or the equivalent for `#[crystal_oscillator]`/`#[switching_converter]`) to its own body — genuine, disclosed authoring work, not required by this RFC's completion bar.

## Decision

**Accepted — 2026-07-20.** `#[bypass(...)]`, `#[crystal_oscillator(...)]`, and `#[switching_converter(...)]` (all RFC-027) now accept a bare `Pin`-typed `fn` parameter as a target/instance argument, in addition to a literal top-level `INST.PIN`/`INST` reference — reusing the existing `resolve_pin_ref`/`Binding::Pin` resolution machinery (RFC-006) unchanged, with zero new grammar or resolution concept. Each real call site of an attribute-bearing `fn` produces its own independently-resolved CSV row, exactly mirroring RFC-006's existing per-call-site expansion discipline. `#[ground]`/`#[high_current]`/`#[impedance]` are unaffected — they attach to a `net`, which has no analogous gap. Recorded as DR-034 (see note 7). Language Specification (note 10) updates the "Quilter physics-constraint hints" section in place to document the extended target-argument grammar for these three attributes.
