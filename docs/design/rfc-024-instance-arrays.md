# RFC-024: Array-typed instances and indexed references

## Status

Redesigned same day, per Tony's direct correction. The first draft of this RFC proposed inst NAME[START..=END]: Device as pure name-expansion sugar — sw[1..=13]: SW_KEY meant nothing more than thirteen separately-named instances sw1..sw13, with the range usable only inside a net's member list (sw[1..=4].A). Tony's correction: this is not what was wanted. The real request is inst key_leds: [RGB_SK6812; 13] — one real, array-typed instance, indexed by a literal (key_leds[0]) anywhere an instance reference is valid today (net members, place, decouple, fn-call arguments) — not a naming trick scoped to one syntactic position. This revision replaces the first draft's design entirely.

## Problem

Grounded directly in examples/openmicro/src/main.cohdl: 42 near-identical inst lines (13 key switches, 13 matrix diodes, 29 addressable RGB LEDs across two separate WS2812 chains, 4 mounting holes). Confirmed against real source (src/ast.rs): InstStmt.name is a bare Ident, and PinRef.base — the thing referenced everywhere an instance is used (net members, and, per src/check/expand.rs, place's and decouple's instance arguments) — is likewise always a bare Ident. There is no way today to declare one name that denotes a family of same-typed instances and address individual members of that family wherever an instance reference is otherwise valid.

The real, motivating downstream need this RFC must serve (which the first draft's design could not, since it restricted range expressions to net-member lists only): OpenMicro's real WS2812 daisy-chain wiring (led1.DOUT → led2.DIN, led2.DOUT → led3.DIN, ...) and its real per-LED place/decouple statements both need to address one specific array element by index, not just fan out one pin name across a whole range inside a single net — a materially different access pattern than the first draft supported.

## Goals

- inst NAME: [Device; N] declares one real, array-typed instance — NAME itself is the array's name; NAME[i] (i a literal integer, 0 <= i < N) denotes one specific, individually-real instance element.
- NAME[i] is valid everywhere an ordinary instance reference is valid today — inside a net's member list (NAME[i].PIN), as place's target (place NAME[i] at (...)), as decouple's arguments (decouple(NAME[i].PIN, NAME[i].PIN2)), and as a fn-call argument — not restricted to one syntactic position, correcting the first draft's central defect.
- Each array element remains a fully real, individually-addressable instance after resolution — its own designator (RFC-005), its own pin-obligation tracking (RFC-002), its own trait satisfaction (RFC-003) — exactly as if an author had hand-written NAME_0: Device, NAME_1: Device, ..., NAME_{N-1}: Device (the internal designator/diagnostic identity is per-element; the source-facing name is NAME[i]).
- Keep the one genuinely good idea from the withdrawn first draft: a compact range/list form usable specifically inside a net's member list, for uniform fan-out across many elements sharing one pin name (e.g. key_leds[0..=3].VDD on a shared power rail) — now defined as sugar expanding to individual NAME[i].PIN references, consistent with the real array-typed foundation rather than being the only indexing mechanism that exists.

## Non-goals

- Not a general loop/iteration construct (for i in 0..13 { ... }). NAME[i]'s index must be a literal integer — there is no variable, no computed index, no iteration over an array's elements as a single statement. This means OpenMicro's daisy-chain (key_leds[i-1].DOUT, key_leds[i].DIN for each i) and its per-element place coordinates still require one net/place statement written per index — this RFC makes each such statement addressable and correct-by-construction (a real, checked reference to a real element), but does not generate the 12 (or 13, or 16) statements automatically. That is real, disclosed, deferred future work (see Alternatives) — not solved here, not silently implied solved by "arrays" being in this RFC's title.
- Not multi-dimensional arrays — [Device; N] is a single flat dimension; no [Device; N][M] or similar. No concrete need for this has been shown.
- Not array-of-arrays or arrays of non-device-typed things — an array's element type is always a single device type (matching one inst's existing type-position grammar), same restriction an ordinary inst already has.
- Not solving arithmetic-derived per-element data (grid-formula place coordinates) — an author still writes each element's literal coordinates by hand, one place NAME[i] at (...) statement per element; this RFC does not introduce coordinate expressions or formulas.

## Design

### Array-typed instance declaration

```cohdl
design OpenMicro {
    inst key_leds: [RGB_SK6812; 13]
    inst ambient_leds: [RGB_SK6812; 16]
    inst sw: [SW_KEY; 13]
    inst d: [D_1N4148W; 13]
    inst mh: [MH_M2; 4]
}
```

- inst NAME: [Device; N] — N is a positive integer literal, the array's fixed length. NAME is the array's own name; there is no bare NAME reference with no index (unlike a plain inst, NAME alone is never itself a valid instance reference — an author must always index it: NAME[i]).
- key_leds and ambient_leds are two separate arrays of the same device type — this is the ordinary, expected way to express "two independent chains of the same LED part," not a special case.
- Array declarations follow the same name-collision rule as ordinary inst — a second declaration (array or plain) reusing NAME is a compile error.

### Indexed references, valid everywhere an instance reference already is

```cohdl
design OpenMicro {
    // ... array declarations above ...

    net LED_D0: mcu.LED_DATA_KEY, key_leds[0].DIN
    net LED_D1: key_leds[0].DOUT, key_leds[1].DIN
    net LED_D2: key_leds[1].DOUT, key_leds[2].DIN
    // ... one net per chain link, same as hand-writing led1/led2/... today ...

    decouple(key_leds[0].VDD, key_leds[0].GND)
    decouple(key_leds[1].VDD, key_leds[1].GND)

    layout {
        place key_leds[0] at (-8.025mm, -23.875mm)
        place key_leds[1] at (11.025mm, -23.875mm)
    }
}
```

- NAME[i] (a PinRef's base, or place's/decouple's instance argument) resolves to the one real element at index i — checked against the array's declared length N (0 <= i < N); an out-of-bounds index is a compile error naming the valid range.
- This is the central correction from the first draft: NAME[i] is not a special net-member-list-only form — it is a real reference usable in every position an ordinary instance name already is.

### Range/list fan-out, inside a net's member list only

```cohdl
net VBUS [5V]: usbc.VBUS, key_leds[0..=12].VDD, ambient_leds[0..=15].VDD
```

- NAME[START..=END].PIN and NAME[i1, i2, i3, ...].PIN remain valid, exactly as in the first draft's design, but now defined as sugar over the real indexing mechanism above: key_leds[0..=12].VDD expands to key_leds[0].VDD, key_leds[1].VDD, ..., key_leds[12].VDD — a pure textual equivalence to writing each indexed reference out by hand.
- This fan-out sugar remains scoped to net-member lists only (matching a real, common pattern: many array elements sharing one pin on one rail) — place/decouple always take one single index, since placing or decoupling "a range at once" has no single sensible meaning (each element needs its own coordinates; decouple already takes two explicit pin arguments per call).

## Type-system-first test

Every check this RFC introduces is structural, local, and resolved at the point a reference is used — never a DRC candidate:

1. Array-length/index-bounds checking — NAME[i] (or a range/list's endpoints) must satisfy 0 <= i < N against the array's own declared length; checked wherever the reference appears, naming the valid range on failure.
2. After resolution, every existing per-instance check runs unchanged on the one real element NAME[i] denotes — pin obligations (RFC-002), trait satisfaction (RFC-003), designator allocation (RFC-005) — exactly as they would for a hand-declared instance; no new semantic checking beyond bounds-checking the index itself.

## Conceptual impact

Low-Medium. One genuinely new idea: an inst may now be array-typed ([Device; N]), and an instance reference may now be an indexed expression (NAME[i]) rather than always a bare name — a real addition to how "instance identity" is expressed, but still fully reducible to "N real instances, individually addressable," the same underlying concept inst always had. No new top-level declaration kind, no new resolution mechanism beyond bounds-checking an integer index.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Low | Med | Low | Low | High |

Trust (High): every array element remains a fully real, individually-checked instance after resolution — nothing about designator uniqueness, pin-obligation exhaustiveness, or trait satisfaction is weakened or deferred; indexing is purely a reference-time addressing mechanism.
Concepts (Med, up from the first draft's Low): genuinely new — an instance reference is no longer always a bare identifier; this is a real, if bounded, addition to the language's reference grammar, corrected from the first draft's (wrong) claim that this was pure sugar with no new concept.
Grammar (Med): [Device; N] in inst type position, and NAME[i] (or NAME[RANGE_OR_LIST] inside net-member lists) as a reference form, generalized across every position an instance reference already appears — a real, broader grammar surface than the withdrawn first draft's net-member-only scope.
Diagnostics (Med): out-of-bounds index (checked per-use-site, not just per-declaration) is genuine new diagnostic surface, present at every reference position now, not just one.
Netlist/Compat (Low): the resolved form (one real instance per index) is byte-identical in meaning to a hand-written instance — no new emitter logic; purely additive, no existing inst/net/place/decouple statement changes meaning.

## Gradeability

- Array declaration: N must be a positive integer literal; array-name collision (with another array or an ordinary inst) — checked at declaration.
- Every indexed reference (NAME[i] in a net member, place, decouple, or fn-call argument; or a range/list's endpoints inside a net member) — bounds-checked against the array's declared length at the point of use, naming the valid 0..N range on failure.
- After resolution to a real element, every existing check (RFC-002/003/005/008, etc.) runs completely unchanged — no new semantic checking beyond the bounds check above.
- None of this runs in residual DRC.

## AI-generatability

High for the core mechanism (inst NAME: [Device; N], NAME[i] used anywhere) — small, closed, directly motivated by real repeated-family designs, and generalizes cleanly to every reference position rather than requiring a model to remember "ranges work here but not there" (the first draft's real weakness). Still Medium overall for the whole repetition problem: because there is no loop construct (Non-goals), a model must still emit one net/place/decouple statement per array element by hand for chain-wiring and per-element placement — this RFC removes the "which flat name is element 7" bookkeeping burden but does not remove the need to write N statements for genuinely per-element data.

## Alternatives

- The first draft's design: pure name-expansion sugar, indexing scoped to net-member lists only — this RFC's own withdrawn predecessor. Rejected per Tony's direct correction: it could not address daisy-chain wiring or per-element place/decouple at all, since those need one specific indexed element, not a range-fan-out inside a single net's member list — the actual motivating need this RFC exists to serve.
- A general for/loop construct that also generates daisy-chain nets and per-element place data automatically — considered, still rejected as a separate, larger design question: closing that gap well requires solving computed/arithmetic per-iteration data (chain-neighbor references, grid-formula coordinates), which deserves its own focused RFC once array-typed instances (this RFC) exist as the thing such a construct would iterate over. Named explicitly as the natural next step, not bundled in here.
- Multi-dimensional arrays (e.g. a 2D grid of switches indexed sw[row][col]) — rejected as premature: no concrete need has been shown yet; OpenMicro's own matrix is expressed via ROW/COL nets, not a 2D instance grid. A scoped follow-up if a real need emerges.

## Compatibility

Purely additive. inst NAME: [Device; N] is new grammar; no existing plain inst NAME: Device statement changes meaning. NAME[i] as a reference form is new; every existing bare-name instance reference (net ..., sw1.A, ...) is completely unaffected, since sw1 (a plain, non-array instance) is still referenced exactly as before.

Depends on: RFC-002 (pin obligations), RFC-005 (designators), RFC-016 (name resolution) — all already Accepted; each applies unchanged to a resolved array element.

## Tooling & operations

- cohdl fmt needs a canonical form for [Device; N] in type position and NAME[i]/NAME[RANGE_OR_LIST] in reference position — small, closed additions parallel to existing bracketed-selector conventions (pins[VARIANT], RFC-008).
- cohdl lsp hover on NAME[i] should resolve to that specific element's real identity (designator, resolved specs) — the same "resolve and show" precedent already established for ordinary instances.
- Reserves new E2xx sub-cases (name resolution, RFC-016's home): out-of-bounds array index, array-name collision, malformed array-length literal — no new block.

## Teaching cost

Low-Medium. [Device; N] in declaration position and NAME[i] in reference position are both small, closed, and directly analogous to array indexing in any mainstream language — the one real new idea to teach is that NAME alone (unindexed) is never itself a valid reference for an array-typed instance, unlike a plain inst.

## Failure modes

- An author references an out-of-bounds index (key_leds[13] when the array has 13 elements, valid indices 0..=12) — caught immediately, naming the valid range; not a silent gap.
- An author expects a loop/range to auto-generate daisy-chain or per-element place data — this RFC does not do that (see Non-goals); the author must still write one statement per element, now correctly addressed via NAME[i] rather than needing 13 separately-remembered flat names.
- This RFC does not solve OpenMicro's daisy-chain or grid-place repetition automatically — real, disclosed remaining verbosity, explicitly named, not silently implied solved.

## Migration path

No existing design requires migration. OpenMicro's own main.cohdl could adopt inst key_leds: [RGB_SK6812; 13], inst ambient_leds: [RGB_SK6812; 16], inst sw: [SW_KEY; 13], inst d: [D_1N4148W; 13], inst mh: [MH_M2; 4] for its five repeated families, with its ROW/COL matrix nets and VBUS/GND fan-out nets using the range/list sugar, and its daisy-chain nets and per-LED place statements rewritten to use key_leds[i]/ambient_leds[i] indexing (still one statement per link/element, now indexed rather than flat-named) — real, optional authoring work, not required by this RFC's completion bar.

## Decision

Accepted (redesigned) — 2026-07-19. Supersedes this RFC's own same-day first draft (name-expansion sugar scoped to net-member lists only), per Tony's direct correction. inst NAME: [Device; N] declares a real array-typed instance; NAME[i] (a literal integer index) is a real reference form valid everywhere an ordinary instance reference already is — net members, place, decouple, fn-call arguments — resolving to one fully real, individually-checked instance element. A range/list fan-out sugar remains available inside net-member lists only, defined as an expansion over the real indexing mechanism. Explicitly does not introduce a loop/iteration construct or arithmetic-derived per-element data — both real, disclosed, deferred future work. Recorded as a DR-030 revision (see note 7). Language Specification (note 10) replaces the withdrawn first draft's "Instance arrays and range references" section with this corrected design.
