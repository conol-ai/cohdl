# RFC-033: Parameterized circuit construction and bounded compile-time evaluation

> Number provisional pending central registry allocation; supersedes the M2 working draft. Baseline: main `0e3d7705ab393a5c64c20836540ad1bc2e49898e` (2026-09-10), with RFC-032 Accepted and included in note 10.

## Problem

CoHDL describes devices, physical instances, connections, reusable `fn` fragments and Accepted `subdesign` composition. RFC-024 gives `[Device; N]` with literal counts and indexes, but leaves count-dependent wiring and placement manual. A board family needs to change a channel count, wire N repeated channels, place them on a computed grid, and reuse the parameterized result inside another definition — without losing unit safety, pin obligations, stable physical identity, or the compiler's ability to explain a failure.

An RFC that only adds `for` leaves the central questions unanswered: what may a program compute, what may it construct, how do values differ from physical objects, and which checks survive generation? This RFC answers all four with one bounded compile-time evaluation model.

Who this is for: AI generating/repairing `.cohdl`, humans reviewing the resulting design, and library authors composing parameterized circuits. The three acceptance scenarios below are language-contract tests, not claims that an LED chain or RC network is a validated hardware product.

## Goals

* Define CoHDL programmability with explicit inputs, value domains, construction effects and checking stages — one semantics consumed by check/build, fmt, LSP, docs and Explorer; cohdl.ai consumes it rather than reimplementing it.

* Express a daisy chain, a repeated multi-component channel, and nested parameterized fn/subdesign composition through shared mechanisms (no separate "array arithmetic" or "layout arithmetic" engines).

* Preserve unit/trait/parameter-kind checking before construction and every real instance's connection obligations after assembly.

* Reject concrete value errors and excessive expansion deterministically, retaining the source expression and distinguishing call/iteration context.

* Preserve existing array identity and legacy outputs; specify the edit-stability limits of newly generated objects.

* Serve Constitution ranks 1–5: correctness/gradeability, AI-generatability, human reviewability, composability, faithful output. Reduced typing is a benefit, not proof of correctness.

## Non-goals

* Runtime execution/firmware, `while`, recursion, mutation, `break`/`continue`, arbitrary iterators, I/O or environment-dependent evaluation.

* General Bool/`if`, branch-selection semantics, value-returning const fn, general dimensional algebra, automatic parts selection. Count-controlled topology is provided through zero-or-more helper invocations.

* **Direct&#x20;**`inst`**/**`subdesign`**&#x20;declarations inside loop bodies (Candidate B) — deferred**, see Alternatives.

* New electrical assertions, inferred pin voltages/currents, a general constraint solver, simulation, or new residual DRC rules (the existing four remain exactly four).

* Multidimensional arrays, array-valued fn parameters, escaping iteration-local references, returned circuit collections, string interpolation or generated source identifiers.

* Integer generics on `device` declarations (E406), new pin-interface generation, global/exported `const`, general property introspection.

* A layout/router implementation, new coordinate frames, or placement reach-in to fn-created instances. RFC-032 group transforms and internal overrides remain available unchanged.

* Domain recipes (deriving component values from voltage/current requirements) remain M3; additional electrical contracts remain M4.

## Design

### 1. Meaning: finite construction, then the existing checks

This RFC adds bounded, typed compile-time computation plus operation loops over named physical/subdesign objects, retaining ordinary effectful `fn` calls. It is design-time computation; no loop or variable becomes firmware on the manufactured board.

Three categories stay distinct:

* **Values:** Int counts/indexes and physical quantities. Evaluation produces a value; it performs no circuit mutation, I/O, registry query or component allocation.

* **References:** physical Instance/Pin bindings, legal logical-port references and indexed array elements. A subdesign node is not an Instance; neither arrays nor nodes impersonate an Instance or Pin. References cannot be added, serialized into identifiers, or inspected for undeclared electrical facts.

* **Construction effects:** `inst` contributes a physical instance; `net` connectivity; `nc` an explicit non-connection; `fn` its expanded fragment; a `subdesign` use its logical node and real contents; `layout` placement facts. `for` repeats admitted construction effects. A `const` creates none.

Real components are declared in ordinary circuit bodies/arrays or created by called fn fragments; a `for` repeats `net`/`nc`/helper-call/placement operations over them. No loop instruction or fake container component reaches manufacturing IR.

### 2. One expression model, two domains

The evaluator handles **Int** (signed exact 64-bit structural integer — not a twelfth electrical unit) and the existing **Length** type. Domain −2^63…2^63−1; every literal and intermediate is checked.

| Operation                                                                   | Result and rule                                                                            |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Int `+ - *` Int; unary `+ -` Int                                            | Int, checked overflow; no wrapping/saturation                                              |
| Int `/` Int                                                                 | Int, truncated toward zero; zero divisor and MIN/−1 are errors                             |
| Int `%` Int                                                                 | Int, a = (a / b) * b + r; zero divisor and MIN/−1 are errors                               |
| Length `+ -` Length; unary `+ -` Length                                     | Length, exact existing fixed-point representation                                          |
| Int `*` Length or Length `*` Int                                            | Length, checked exact scaling; Int never becomes Length                                    |
| Length `/` Int                                                              | Length only if exactly representable; zero divisor, overflow or lost precision is an error |
| Int `+` Length, Length `/` Length, Length `*` Length, other unit arithmetic | Rejected                                                                                   |

Thus `10mm + n * 4mm` is valid; `10mm + n`, `3.3V + 1V` and `1mm / 0` are not. `1mm / 3` is rejected if not exactly representable — never rounded. Existing physical-coordinate range checks still apply after evaluation; no platform-libm dependency.

Length uses the existing signed i128 count of 10^-15 mm (`UnitValue.femto`); intermediates are checked in that representation. Arithmetic-derived Length values get canonical text from `geom::mm_femto` (minimal decimal + `mm`, no trailing zeros/negative zero): `1.00mm + 0mm` → `1mm`. A literal, parentheses around it, or pure parameter/constant forwarding preserves original text; arithmetic (including unary `-PITCH`) produces canonical text. No emitter invents its own spelling or puts an unevaluated expression into `UnitValue.text`.

Integer literals are decimal. A prefix sign directly on a decimal literal combines with its magnitude before the range check, so `-9223372036854775808` is valid and `9223372036854775808` is not. `-MIN` as an evaluated operation fails on overflow. Precedence, highest first: primary/parentheses → unary → `* / %` → `+ -` → range delimiter; binary arithmetic is left-associative; evaluation in source-tree order — reassociation must not hide overflow. Comparisons, booleans, casts, exponentiation are outside the grammar.

### 3. Local constants and shared integer generics

`const NAME: Int|Length = EXPR` is admitted in design, fn, subdesign, circuit-loop and layout bodies. A const is immutable, creates no physical object, is not exported. It may use visible constants, Length parameters, const Int parameters, enclosing binders and visible array `.len`; it cannot inspect pin voltages, instance specs, registry state, files or generated placements.

`const N: Int` extends the RFC-007 generic list for **both fn and subdesign** through the same binding/substitution mechanism (`helper::<N>(...)`, `subdesign bank: Bank<N>`, `[Bank<N>; M]`). Int defaults remain explicit integer literals; Int and Length arguments admit the stated expressions; other physical units keep literal/parameter-only rules. The implementation adds a distinct Int case beside `GenericBound::Unit`/`Traits` and `GenericValue::Unit`/`Device` — never a fake UnitValue, string-kind inference, or a second generic resolver. The contextual `const` marker distinguishes the structural value from an existing trait named `Int`; `N: Int` retains trait-bound meaning where such a trait exists.

Constants and array lengths within a lexical body form an acyclic dependency graph (forward references in the same body allowed; complete cycles reported, E1407). Child constants never escape to parent/sibling scopes; a layout constant is visible only in that layout and descendants. New constants, loop labels and binders cannot collide with or shadow visible instances, subdesign use names, arrays, nets, ports, fn parameters, generic parameters, constants or enclosing labels/binders — rejected at definition validation (E201). Disjoint sibling loops may both bind `i`.

### 4. One array evaluator, existing reference kinds

Physical and subdesign array lengths accept Int expressions and remain strictly positive (zero/negative remain E211). `.len` returns the declared length of a visible physical or subdesign array without constructing elements; scalars have no `.len`; no `bank.channels.len` across the port boundary (inside bank's own definition its arrays are queryable). This is one query, not general introspection.

Every already-legal single-index position uses the same evaluator: physical pins, subdesign ports, legal fn arguments, arrayed subdesign use sites, and **each indexed segment of an existing placement path** (`place banks[b].channels[i + 1].c at (...)`), with each segment's bounds checked against that node's own array. This extends selectors, not access rights: no electrical reach-in, no subdesign-as-Instance, no `nc` on a port (E1306 unchanged).

Range/list fan-out remains net-member-only. **Decision (R1):** inclusive `..=` fan-out and its optional step form are retained alongside the half-open `for`; a half-open selector is a possible future additive RFC-024 change, not included here. Bounds/entries/step admit Int expressions; positive step, start ≤ end, all selected elements in bounds; never clip. With length 3, `arr[0..=3 step 2].PIN` selects 0 and 2 and is valid; `arr[0..=3].PIN` selects 3 and fails E202. Equal inclusive endpoints select one element; a one-member net remains legal.

### 5. Finite `for` with mandatory labels

**Decision (L1):** every loop carries an authored label.

```text
for_stmt       := 'for' LABEL ':' IDENT 'in' expr '..' expr '{' loop_stmt* '}'
const_stmt     := 'const' IDENT ':' ('Int' | 'Length') '=' expr
int_parameter  := 'const' IDENT ':' 'Int' ('=' integer_literal)?
primary        := literal | IDENT | IDENT '.' 'len' | '(' expr ')'
```

The half-open range `a..b` enters a through b−1 in order; equal endpoints are empty; reversed endpoints fail E1404. Bounds must be concrete Int before entry. Signed bounds and nested loops depending on an enclosing binder are allowed; a negative array index still fails. No inclusive `for`, step, mutation, `break`/`continue` or arbitrary iterators.

| Context                             | Admitted operations                                                                                        |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Circuit loop in design/subdesign    | const, anonymous `net _`, nc, fn calls, nested for, placement-subset layout                                 |
| Circuit loop in fn                  | Same, subject to inherited fn placement restrictions                                                       |
| Loop inside layout                  | const, place, nested for; no inst, subdesign, fn call, net or nc                                           |
| Ordinary circuit body outside loops | Existing inst/subdesign/fn operations plus const/for/count expressions; existing owner restrictions remain |

Direct scalar/array `inst` and `subdesign` declarations in a loop body are rejected — including in empty loops — with E1406 where no more specific owner error applies; `subdesign` use in a fn remains E1307. A directly authored named net (`net NAME: …`) inside a loop body is likewise rejected with a single E1406 at the net name — including in empty or nested loops and in uncalled fn/subdesign definitions (PR-43 review revision, still Proposed; not yet Accepted): a per-iteration named rail is ambiguous about whether the frames join, and implicit frame-private named nets are unsafe to leave as defaults for this syntax. Declare the named net outside the loop and join it inside via an anonymous `net _` through the shared pins/ports; a named net local to a helper fn called from a loop keeps its ordinary per-frame privacy (fn isolation already governs it). Calling a fn from a loop may create that fn's ordinary local components (fn stays effectful; all effects are metered). Local constants are validated before operations, but no hypothetical instances are allocated to validate a skipped loop. A loop declares neither an electrical interface nor a coordinate frame; no outside-loop `label[i].r` interface exists.

### 6. Placement preserves ownership and override semantics

```cohdl
layout {
    for grid: n in 0..leds.len {
        place leds[n] at (10mm + (n % 5) * 4mm,
                         10mm + (n / 5) * 4mm)
    }
}
```

`at` accepts Length expressions; `rotate` accepts Int checked against the existing 0…359 rule; `side` remains literal. Placement context is inherited from the design/subdesign/fn owner, never recovered from a generated path string: board frame in a design, local-frame defaults in a subdesign composed through ordinary anchors/rotations/sides. A loop creates no additional transform or override priority. Fn→for→layout→place remains E1007; a board outline in fn remains E1006.

A loop may place a declared subdesign array or override an internal instance through RFC-032's dotted path; an outer override beats the inherited default for that instantiation only. Two same-precedence authored placements resolving to one target remain duplicates (E1007) even when one comes from a loop — no last-write-wins. Fn-local components remain unplaceable; the loop label is not a public group address.

### 7. Identity: hygienic labelled iteration frames

Current arrays derive element names via `element_name(base, i)`; subdesign/fn expansion extends `Scope.path` into the shared designator allocator. This RFC extends that one mechanism:

| Source of identity                    | Path                                                                      | Addressability                                           |
| ------------------------------------- | ------------------------------------------------------------------------- | -------------------------------------------------------- |
| Physical array element                | `Board::leds_3`                                                           | Existing physical instance/pin contexts                  |
| Subdesign array child                 | `Board::channels_3::c`                                                    | Ports for electricity; dotted paths for placement        |
| Loop-generated operation/helper local | `Board::__for_wiring_3::__net0`, `Board::__for_decouple_3::__fn0_helper::LINK` | Operation provenance; fn locals gain no placement access |

`__for_LABEL_VALUE` is an internal lexical frame keyed by **label + actual integer value** (decimal; negative as `neg` + magnitude), never by source ordinal or line number — not a subdesign node; no ports, part, designator, BOM row, default layout or override priority. The `_*for*` name family is reserved via the existing reserved-name rule; injectivity is tested against nested array/subdesign/fn paths.

Consequences (the reason labels are mandatory):

1. **Diagnostics name the failing iteration** — `E202 … Board::__for_links_9, N = 10, n = 9` identifies which repetition of which loop failed at a source span nine earlier iterations passed.

2. **Generated identity survives edits** — inserting a loop above another, reordering differently-labelled loops, or growing a range preserves every surviving label/value path, designator and lock row; only new iterations add rows. Renaming a label is the one edit that intentionally changes generated paths. An unlabeled ordinal scheme would shift every generated identity when a loop is inserted above.

3. **Sibling loops stay distinct** — both may bind `n`; same-spelled anonymous nets or helper locals in different frames never merge. Each iteration owns its fn-call and anonymous-net counters; explicitly shared pins/ports merge through existing rules; no private net escapes by printed name. A named net authored directly in the loop body is not one of these cases — it is rejected (E1406, see §5) rather than silently merged or privatized; helper-fn private named nets remain the named-per-iteration path.

A loop that only wires or places a predeclared object **never reparents it**: extending the RC array preserves `channels_0`…`channels_9` paths. The existing ordinal identity limit for fn calls *within* one frame remains disclosed: reordering those calls can reassign purposes without a helpful lock diff — compare connectivity/parts/layout too. Duplicate labels fail E201.

### 8. Static checks, concrete binding, final assembly

Statically decidable declaration errors are rejected uniformly across all supplied fn/subdesign definitions and loop scopes, regardless of calls, M2 syntax or reachability (a shared declaration validator over `src/check/bodies.rs` and the shared generic environment; see Compatibility for its one disclosed cost). One typed evaluator runs in three environments:

1. **Definition validation:** types known, values possibly unknown. `inst r: SeriesR<C>` for `C: Capacitance` fails uncalled — SeriesR expects Resistance; no value of C is needed. Empty loops cannot hide known declaration/signature/expression errors; forbidden direct declarations are rejected by context.

2. **Actual fn/subdesign activation:** bind arguments/defaults, validate the full body under that substitution before materializing children. Values depending on the arguments become concrete, including inside empty nested loops.

3. **Actual iteration:** bind the integer, validate newly concrete expressions, contribute admitted operations. Type-only validation allocates no hypothetical objects or dynamic frames.

A known zero divisor fails even with an unknown numerator (`i / 0` inside an empty loop fails); `1 / i` waits for an actual `i`; no algebraic simplification invents values. Constants evaluate in deterministic dependency order, memoized per typed/bound environment. `f::<0>()` where f contains `for empty: i in 0..0 { const BAD: Int = 1 / N }` fails under the binding; `for none: i in 0..0 { f::<0>(); }` checks f's signature and argument expressions but performs no callee value specialization; `f::<1 / 0>()` there fails at its own argument.

After expansion: existing connection merging, required-port validation, physical-pin obligations, residual DRC; build retains part/footprint/designator obligations. A helper may leave caller-owned pins to its caller; temporary port members survive until required-port checking and merging finish. Loop expansion cannot erase a required port or treat it as a physical nc. Errors carry original source, actual bindings and fn/subdesign/iteration provenance downstream.

### 9. Deterministic work limits across the full expansion graph

Metering activation: walk the complete syntactic expansion-reference graph rooted at the selected design (every fn call and legal subdesign use-site type, nested arrays, helpers in subdesigns, calls lexically inside loops; visited set; no instantiation; device/part references contribute no bodies). Activate metering if the graph contains an admitted const, const Int parameter, loop, `.len` or arithmetic expression. An Int default in a reached subdesign activates it; a reached-but-skipped M2 helper activates it; an unrelated uncalled M2 definition does not. A purely legacy selected graph keeps old resource behavior. Once active, metering covers the whole actual expansion including legacy declarations; source reorder, dead-constant elimination and optimizer caching cannot change activation or counts.

Limits (proposed language constants, not host heuristics or timeouts): **100,000** cumulative entered loop iterations, **1,000,000** generated work items, **64** active loop frames (across fn/subdesign activation; a logical node is not a frame). Exceeding a limit is a deterministic hard error (E1405) before materializing the excess item.

| Actual semantic event                 | Work items charged before materialization                                        |
| ------------------------------------- | -------------------------------------------------------------------------------- |
| Physical scalar/array element         | 1 per real instance                                                              |
| Fn invocation                         | 1 per entered call + its contributed items                                       |
| Subdesign scalar/array element        | 1 per logical node + declared ports + contributed items                          |
| Instantiated subdesign port           | 1 per port, including optional/disconnected                                      |
| Authored port-connection-block entry  | 1 per entry + 1 per scalar target reference; synthesized join not double-charged |
| Authored net/nc/place statement       | 1 per expanded statement; net/nc + 1 per expanded member, before dedup           |
| Layout constraint                     | 1 per expanded declaration + 1 per authored net reference                        |
| Physics attribute / diff-pair bracket | 1 per resolved record + 1 per authored pin/instance target                       |

Array bases, const declarations, layout blocks and loop headers charge nothing. Empty logical nodes and ports **do** count (no unmetered amplification through empty subdesigns). Charge in declaration-source and entered-integer order; compute large counts with checked arithmetic and reject over-budget requests before bulk allocation, preserving the same first failing source site as incremental charging. Failed expansion emits no successful or truncated build.

### 10. Lexing and diagnostics

**Minus has one lexical spelling.** Every `-` outside strings/comments is a standalone token; the numeric token carries no sign. The parser assigns roles: prefix in operand position, binary after a completed left operand (`n-1` ≡ `n - 1`; `n--1` subtracts a negative literal). Where the grammar expects a literal/prefix operand, a minus immediately adjacent to a numeric unit spelling assembles the existing signed literal with its original text and sign-inclusive span; the same literal validation applies to all consumers (`-1.00mm`, `-40C` retained; `-5V` remains E105). A space/comment/parenthesis prevents assembly: `- 1.00mm` and `-(1.00mm)` are unary arithmetic → canonical `-1mm`; `- 40C` remains invalid (no Temperature arithmetic). `1mm-1mm` subtracts two positive literals. Fmt preserves the literal-versus-unary AST distinction across reparsing.

`for` is already a token (`impl Trait for Device`); `const` and `in` are contextual, not globally reserved. Maximal unit-suffix rule kept: `10%` is a Tolerance literal, `10 % 3` is remainder, `10%3` is rejected (never reinterpreted by expected type); fmt spaces binary operators. Keep E102 at legacy negative-bare-number consumers; admitted Int expressions instead receive expression/bounds diagnostics. No symbol-table-dependent tokenization or unbounded lookahead.

Reuse existing errors where meanings apply: E201 (name collision — registry wording widened from "duplicate top-level declaration" to the lexical-scope collisions `expand.rs` already emits), E202, E211, E401, E403, E110/E111, E112/E113, E406 (unsupported const-generic owner), existing placement/DRC errors, E1306/E1307. New block (central allocation at acceptance):

| Code  | Failure kind                                                                                        |
| ----- | --------------------------------------------------------------------------------------------------- |
| E1401 | Expected compile-time Int/Length or a supported operand pairing; wrong kind or unsupported property |
| E1402 | Int/Length overflow or non-exact Length division                                                    |
| E1403 | Division/remainder by zero                                                                          |
| E1404 | Reversed for range                                                                                  |
| E1405 | Deterministic elaboration budget/depth exceeded                                                     |
| E1406 | Declaration/operation not admitted in this loop or expression context                               |
| E1407 | Cyclic constant/array-length dependency, naming the complete cycle                                  |

Primary span: the smallest failing original construct. For expansion-dependent errors the **main message** carries the violated constraint, concrete failing value/valid bound, and a compact activation path with bindings (`Board::bank::__for_links_9, N = 10, n = 9`) — essential context survives consumers showing only code/message/primary span. Secondary labels add enclosing call/use and loop-header spans. RFC-010 schema_version 1 retained. Deduplication keys include the expansion frame so distinct failing iterations are never conflated. No hidden truncation.

### Scenarios (normative acceptance shapes)

**LED chain** — computed connections over a physical array:

```cohdl
const N: Int = 10
inst leds: [AddressableLED; N]
net VCC [5V]: host.V5, leds[0..=(leds.len - 1)].VDD
net GND [gnd]: host.GND, leds[0..=(leds.len - 1)].GND
net DATA: host.DATA, leds[0].DIN
for links: n in 0..(leds.len - 1) {
    net _: leds[n].DOUT, leds[n + 1].DIN
}
nc: leds[leds.len - 1].DOUT
```

N = 1/2/10 contributes 0/1/9 neighbor nets. The label `links` names the operation site, not the LED array, a placement group or a subdesign; `place links` and `links[3]` are invalid. An out-of-range `n + 1` fails despite being well-typed Int.

**RC channels** — the full board-authoring workflow (define → instantiate ten → wire ports → place separately → adjust capacitor seven → grow to twelve):

```cohdl
subdesign RcChannel<R: Resistance, C: Capacitance> {
    ports {
        required IN: Pin
        required OUT: Pin
        required GND: Pin
    }
    inst r: SeriesR<R>
    inst c: ShuntC<C>
    net _: IN, r.A
    net _: OUT, r.B, c.A
    net _: GND, c.B
    layout {
        place r at (0mm, 0mm)
        place c at (3mm, 0mm)
    }
}

design FilterBoard {
    const N: Int = 10
    inst inputs: [SignalSource; N]
    inst outputs: [SignalSink; N]
    inst ground: Ground
    subdesign channels: [RcChannel<1kohm, 100nF>; N]
    net GND [gnd]: ground.GND

    for wiring: i in 0..channels.len {
        net _: inputs[i].OUT, channels[i].IN
        net _: channels[i].OUT, outputs[i].IN
        net _: ground.GND, channels[i].GND
    }

    layout {
        for placement: i in 0..channels.len {
            place channels[i] at (10mm + i * 8mm, 15mm)
            place inputs[i] at (10mm + i * 8mm, 5mm)
            place outputs[i] at (10mm + i * 8mm, 25mm)
        }
        place ground at (0mm, 15mm)
        place channels[6].c at (62mm, 17mm)
    }
}
```

4N+1 physical instances, 2N+1 net classes. At N = 10, channel 6's default capacitor origin is (61mm, 15mm); its override is (62mm, 17mm). At N = 12 the override and all earlier origins remain; surviving `channels_0::r`…`channels_9::c` keep paths, designators and terminal relationships; the shared ground class legitimately gains endpoints (the named `net GND` is declared outside the loop and joined per channel via anonymous `net _`, per §5).

**Nested reuse** — shared substitution through logical boundaries and helpers:

```cohdl
fn join(src: Pin, dst: Pin) {
    net _: src, dst
}

subdesign FilterBank<const N: Int, R: Resistance, C: Capacitance> {
    ports {
        required IN: Pin
        required GND: Pin
    }
    inst receivers: [SignalSink; N]
    subdesign channels: [RcChannel<R, C>; N]
    for wiring: i in 0..N {
        join(IN, channels[i].IN)
        join(channels[i].OUT, receivers[i].IN)
        net _: GND, channels[i].GND
    }
    layout {
        for placement: i in 0..N {
            place channels[i] at (i * 8mm, 0mm)
            place receivers[i] at (i * 8mm, 10mm)
        }
    }
}

subdesign bank: FilterBank<3, 1kohm, 100nF> {
    IN: source.OUT, GND: ground.GND,
}
layout {
    place bank at (10mm, 15mm)
    place bank.channels[1].c at (22mm, 17mm)
}
```

N/R/C pass through shared generic substitution; nested relative layout and board-level override paths are preserved. A separate regression repeats an effectful decoupling helper and proves one real capacitor per invocation.

## Rules

1. **Values, references and construction effects are distinct kinds.** Evaluation never constructs; references never compute; a `const` never creates a circuit object.

2. **Int is structural, not electrical.** It never binds a unit-typed parameter, never coerces to Length, and never weakens rejection of bare numbers in unit positions.

3. **Length arithmetic is exact.** No rounding, no reassociation that hides overflow, canonical result text per §2; literal spellings are preserved.

4. `const N: Int`**&#x20;extends RFC-007's one generic mechanism** for fn and subdesign — one binding/substitution/resolution/visibility discipline, no second resolver.

5. **Both array kinds share one count/index evaluator** while keeping physical-versus-logical element kinds; lengths remain strictly positive; `.len` is a length query, not introspection.

6. **Every loop is labelled; ranges are half-open; bodies contain only admitted operations.** Direct `inst`/`subdesign` or a directly authored named net in a loop body is E1406, including empty loops; fn calls retain ordinary local declarations.

7. **Loops introduce a lexical iteration scope, never an interface, coordinate frame, or reparenting.** Frames are keyed by label + value; wiring or placing a predeclared object never changes its path.

8. **Ports remain the only electrical boundary; placement remains the sole reach-in** (RFC-032 unchanged). Fn locals stay unplaceable; `nc` on a port stays E1306; subdesign-in-fn stays E1307.

9. **Static declaration errors are rejected uniformly** across all supplied definitions, called or not; unknown concrete values defer to actual binding; empty loops hide nothing decidable.

10. **Final assembly checks are preserved:** connection merging, required ports, physical-pin obligations, exactly four residual DRC rules, and full part/footprint/designator/BOM obligations for every real instance. Containers never gain manufacturing identity.

11. **Expansion is deterministically bounded:** 100,000 iterations / 1,000,000 work items / 64 frames, charged per §9 before materialization; no partial success.

12. **Diagnostics carry activation provenance in the main message** and deduplicate per expansion frame; original spans survive check/build JSON, LSP and Explorer.

13. **One lexical minus; contextual&#x20;**`const`**/**`in`**;&#x20;**`10%`**&#x20;stays Tolerance;&#x20;**`10%3`**&#x20;is rejected.** Signed-unit literal validation is unchanged in effect.

14. **cohdl.dev owns this semantics.** cohdl.ai generates source and consumes verdicts; it supplies no second preprocessor, private loop interpretation, or different arithmetic/limits.

## Type-system-first test

Not a residual-DRC proposal, and type-system-first, not type-system-only. Unit substitution, trait satisfaction and Pin-vs-Instance kinds remain type-level obligations checked at definition validation — `SeriesR<C>` for a Capacitance C fails uncalled, with no concrete value. Int range/index/overflow and finite generation are compile-time elaboration obligations checked at each actual binding/iteration. Required physical pins and required subdesign ports are structural obligations at final assembly, because only then is caller connectivity known. A known type is not a known value: `N + 1mm` is rejected at type validation; whether `[D; N]` is positive waits for concrete N; `leds[n + 1]` can be well-typed and still fail its last-iteration bound.

Preservation criterion: every generated instance carries the same resolved device, specs, pin roles/obligations, trait bindings and part requirements an equivalent hand-written instance would carry; failed checks are never replaced by unchecked placeholders. Verified with paired positive/negative fixtures, not claimed as a formal soundness proof. Four mandatory negative probes: wrong unit through nested forwarding; out-of-range well-typed index; missing required connection on a generated channel; deliberately cross-connected still-well-typed channels caught by the independent topology oracle.

## Conceptual impact

High. The permanent new capability is **typed parameterized circuit construction through bounded compile-time evaluation**; the permanent cost is the value/effect distinction plus hygienic labelled iteration frames. Canonical vocabulary additions:

* A **value** (Int/Length) is computed at design time and constructs nothing.

* A **const** names a value inside one lexical body.

* A **const Int generic** is a structural count parameter carried by RFC-007's one mechanism.

* A **for loop** repeats admitted operations over a finite integer range inside a labelled hygienic frame — never a circuit object, interface or coordinate frame.

`module`, `subdesign`, `fn`, `Instance`, `Net` keep their meanings. One evaluator serves counts, selectors and coordinates — no separate engines.

Explicit amendments at acceptance: RFC-001 (Int outside the unit set; exact Length ops; E102 ceases at newly admitted integer contexts), RFC-007 (non-unit const generics admitted; Length argument expressions), RFC-024 (checked Int expressions replace literal-only counts/indexes), RFC-032 (count/index computation, local const/for, shared declaration validation, expansion metering extended into it), RFC-006 (shared parameter binding/provenance; strengthened declaration validation), RFC-005 (loop-generated paths specified), RFC-009/010/011/014 (fmt/spans/errors/LSP ship together), `docs/apidocs.md` (schema v2, see Tooling), RFC-013/020/025/026 (expression-valued positions/rotations under existing geometry), RFC-029/030 (offline evaluation; no registry/network queries).

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
| -------- | ------- | ------ | ----------- | ------- | ------ | ----- |
| High     | High    | High   | High        | High    | High   | High  |

* **Concepts:** Int/const justified by count-dependent real arrays and finite iteration; runtime values and spec arithmetic excluded; B's declaration scope deferred.

* **Grammar:** one precedence table and expression AST; const generics extend the existing list; mandatory labels; contextual keywords steal no declaration names.

* **Oracle:** static body validation plus concrete per-use elaboration, then the existing pipeline; no ignored statements, hidden unknowns or skipped real parts.

* **Diagnostics:** original spans plus expansion frames through existing check/build JSON and LSP; negative/empty/overflow tests required.

* **Netlist:** loop-local names cannot short sibling iterations; no fake loop components; graph equivalence against hand-expanded oracles including pins, NC and placements.

* **Compat:** High impact, deliberately: byte-stable legacy outputs except the enumerated uniform-validation correction; signed/unit lexing and legacy fn counters need targeted regression (see Compatibility).

* **Trust:** deterministic failure on bounds/resource violations; no time-dependent evaluation or partial success.

Not a Low-impact parser convenience: acceptance updates the affected normative sections and decision record in one design change; implementation cannot rely on superseded RFC-007/024 restrictions while claiming them unchanged.

## Gradeability

The full acceptance matrix (retained from the working draft, unchanged in substance):

| Case                                                                              | Required result                                                                                               |
| --------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| LED N = 1/2/10                                                                    | 0/1/9 neighbor nets, shared power nets, explicit input/final nc, ordinary pin checks                          |
| Array count zero/negative/non-Int/overflow                                        | Error before element/node materialization; no zero-length shortcut                                            |
| Wrong R/C through nested subdesign/fn; unsatisfied trait; Instance vs Pin vs node | Existing typed distinctions, diagnostics with activation provenance                                           |
| Missing required port / internal required pin / nc on port                        | Existing distinct structural errors; containers conceal nothing                                               |
| Independent RC vs shared-input nested bank                                        | Each matches its own oracle; no false interface equivalence                                                   |
| Ten channels → capacitor-seven override → twelve                                  | Surviving paths/designators/positions and restricted endpoint partitions preserved; new parts fully accounted |
| Computed selectors in `banks[b].channels[i].c` placement                          | Each segment checked against its node's array; electrical reach-in rejected                                   |
| Subdesign relative layout, nesting, rotation/bottom, individual override          | Same transforms/precedence as explicit syntax; no loop coordinate frame                                       |
| Duplicate placement from loop + explicit statement                                | E1007; no last-write-wins; inherited-default override valid                                                   |
| Direct inst/subdesign/named net in loop, incl. empty                              | E1406; E1307 takes precedence for subdesign-in-fn                                                             |
| Fn→loop→subdesign or place; board outline in fn                                   | E1307/E1007/E1006; loops cannot change the fn owner                                                           |
| Uncalled duplicate-local or known generic mismatch                                | Same declaration error with/without M2 in caller; documented correction                                       |
| Empty body with unknown symbol or invariant `i / 0`; skipped `1 / i`              | Static failure where decidable; no invented iteration values                                                  |
| Skipped `f::<0>()` vs `f::<1 / 0>()`                                              | No callee specialization vs immediate invalid argument                                                        |
| Active fn cycle; recursive subdesign containment                                  | Existing stages, full cycle diagnostics; no recursion via budget discovery                                    |
| Int/Length mismatch, extrema, intermediate overflow, inexact division             | Exact-domain failures at original spans; no coercion/reassociation/rounding escape                            |
| Constant/length cycles, forward refs, nested shadowing                            | Complete cycle/name errors; deterministic dependency evaluation                                               |
| `.len` on both array kinds; scalar or private `bank.channels.len`                 | Shared valid query or targeted rejection; no new reach-in                                                     |
| Reversed for, inclusive fan-out bounds, zero step                                 | Declared endpoint/step errors; inclusive vs exclusive tested explicitly                                       |
| Loops reordered/extended; label renamed                                           | Surviving frame identity retained; rename changes paths explicitly; no reparenting                            |
| Repeated helper locals; predeclared subdesign children                            | Injective shared path construction; every real part retained; containers have no BOM rows                     |
| Reordered fn calls within a frame; repeated designator override                   | Disclosed ordinal limitation / ordinary collision rejection                                                   |
| Design→subdesign→M2 helper; M2 only in deepest body                               | Metering active before any materialization                                                                    |
| Pure legacy graph / unrelated uncalled M2 definition                              | No new meter; uniform declaration corrections apply independently                                             |
| Reached-but-skipped M2 helper, unused const, Int default                          | Deterministic whole-design metering without discovery instantiation                                           |
| Empty subdesign arrays, port-only nodes, connection entries                       | Counted without physical parts; no double-charged joins                                                       |
| Exactly at / one over each limit                                                  | Deterministic boundary tests with source/count/limit; no partial output                                       |
| All work-event kinds, duplicates, bulk counts                                     | Charged before materialization in specified order; optimization cannot change costs                           |
| Const Int/Length defaults and forwarding across packages/fn/subdesign             | One binding mechanism; exact dependency hashing retained                                                      |
| Existing trait `Int`, identifiers `const`/`in`, old signed/unit literals          | Existing meanings preserved; unsupported owners rejected (E406)                                               |
| `10%`, `10 % 3`, `10%3`; fmt fixed point                                          | Tolerance literal kept; spaced remainder parsed; ambiguous form rejected; fmt preserves AST/meaning           |
| Diagnostics with related information hidden                                       | Main message still distinguishes activations at one span                                                      |
| Docs v2 symbolic bodies → registry → viewer                                       | Accurate signatures/ports/canonical source; no invented counts or silent downgrade                            |
| Legacy golden artifacts + stricter declaration cases                              | Bytes/locks/verdicts stable except enumerated corrections                                                     |

Additional executed baseline: `docs/proposals/fixtures/m2-programmability/rc-workflow/` (main `0e3d770`, real locked parts) establishes the pre-M2 subdesign editing baseline — ten/twelve channels build reproducibly with surviving designators and stable placements. It does not test the evaluator, loops, validator or meter; those fixtures are new work under this RFC. Compare generated designs against explicit oracles with identical device/spec/part choices, endpoint partitions, nc decisions and effective placements; anonymous-net names are not topology.

## AI-generatability

High. Teach the existing object model first: physical arrays name components, subdesign arrays name reusable ported regions, fn expands a typed effectful fragment, loops repeat admitted operations, and one evaluator computes counts, selectors and coordinates. Diagnostics report actual binding/index/context at the original span, so a repair agent can act locally — the canonical repair task is shortening `0..leds.len` to `0..(leds.len - 1)` after the last-iteration index error names `__for_links_9, n = 9`. Private per-iteration construction requires extracting a helper or subdesign; that cost is disclosed, not hidden behind a direct-declaration shortcut. An in-bounds connection to the wrong channel can remain type-correct, so independent topology checks stay necessary.

## Alternatives

* **Candidate B — direct&#x20;**`inst`**/**`subdesign`**&#x20;declarations in loop bodies: deferred, not an optional switch.** Its private locals would need declaration scope, identity and placement/access contracts, and its direct-local RC form does not satisfy the separate-layout workflow (no external override interface). Reconsider only with a concrete workload demonstrating material extraction cost under A, compared under identical interfaces and checks. Deferral does not mean local construction is unsound.

* **Optional loop labels (L2) or layout-only omission (L3): rejected for this RFC.** Automatic lexical ordinals shift generated identity when unnamed sites are inserted or reordered, and adding a label later changes paths; L3 additionally makes header legality context-dependent. Mandatory labels (L1) buy stable diagnostics and edit-stable identity at the cost of naming placement-only loops.

* **Half-open fan-out selectors (R2): not included.** Retaining inclusive `..=` fan-out minimizes RFC-024 change; adding `arr[0..n].PIN` later is an additive change needing its own empty-selection/step/diagnostic/fmt rules.

* **Pure fn plus subdesign-only construction:** a separate language-role and migration proposal; fn remains effectful here.

* **Host-language interpretation / general-purpose computation now (tscircuit-style):** broader control flow, but requires effect/termination/packaging/diagnostic contracts outside this slice. tscircuit's map-based construction (core `c298605b…`) is authoring-ergonomics evidence, not proof of boundedness, unit rules or identity guarantees; its string-named repetition depends on identifier generation this RFC excludes.

* **Generated source identifiers / string interpolation: rejected** — they would bypass hygienic frames and the reserved-name discipline.

* **A loop-specific string-substitution pass or editor-side expansion: rejected** — tooling must share the compiler's distinction between symbolic type, bound value and assembled circuit; there is no second elaborator.

## Compatibility

Explicitly amends accepted exclusions rather than pretending prior text remains normative. No existing array spelling is replaced. Existing valid source using none of the new constructs preserves verdict, `design.lock` and every emitted byte, with **one enumerated correction**: statically invalid, previously unchecked *uncalled* definitions (duplicate locals, known generic mismatches) now fail uniformly under §8. Before acceptance, run the existing checker and a validator prototype over every package in `lib/` (including unused definitions) with a fixed source revision and exact dependency set; report newly rejected definitions by category — including a measured zero — with locations and migrations, and update the implementation ledger. The current declaration inventory (60 packages, 162 files, two nongeneric fns, no subdesigns) is a coverage gap, so a focused generic-fn/subdesign corpus is additionally required.

**Whitespace-sensitive&#x20;**`%`**&#x20;is an explicit lexical tradeoff:** `10%` stays Tolerance, `10 % 3` is remainder, `10%3` is rejected. Acceptance acknowledges the tradeoff. Regression coverage beyond happy-path loops: pin numbers, signed physical literals, trait parameters named `Int`, negative-number errors at old physical consumers, tolerance literals, old generic defaults/arguments, specs, coordinates.

Adopting loops in existing source is a design edit: it may rename anonymous/local nets or fn-local paths. Preserve topology and array identities, inspect the lock diff and reconcile layout mappings; do not auto-migrate routed boards or mass-rewrite source with fmt. Pure-constant arithmetic introduces no nondeterministic inputs. Inclusive fan-out and half-open for coexist by decision R1; `docs/error-codes.md` reflects E201's widened wording and preserves old E130x distinctions.

Depends on: RFC-001 (units; exact Length representation reused), RFC-002 (pin obligations at final assembly), RFC-004 (residual DRC unchanged), RFC-005 (designator allocator; loop paths feed it unchanged), RFC-006 (fn; effectful expansion and active-call cycle rules), RFC-007 (generics; extended with const Int through the same mechanism), RFC-009/010/011/014 (fmt/diagnostics/error registry/LSP), RFC-013/020/025/026 (layout; expression-valued positions), RFC-016 (modules/visibility), RFC-024 (arrays; literal-only counts replaced by checked expressions), RFC-029/030 (packages; offline evaluation), RFC-032 (subdesign; ports/hierarchy/placement contracts preserved and extended).

## Tooling & operations

Implementation seams (suggested, not an authorization to bypass acceptance): one expression AST/evaluator; the shared generic substitution environment gains Int; selector ASTs retain expressions until concrete resolution; body/layout walkers retain loop and const nodes (no text substitution); the Design/Sub/Fn placement owner is carried through lexical iteration frames; original spans and expansion provenance live in the shared checked data.

* `cohdl fmt` formats, never unrolls or constant-folds; verify semantic preservation as well as idempotence (dropping parentheses from `(n + 1) * 2` changes the circuit); binary operators are spaced; literal-versus-unary Length text and sign-inclusive spans round-trip.

* LSP identifies constants, loop binders and array declarations; shows concrete lengths/values only where a single instantiation determines them; message-only clients still receive §10's essential expansion context.

* Explorer consumes the same expanded instances and source mapping — no second JS elaborator.

* **Package API docs move to&#x20;**`schema_version: 2` for documents whose items need M2 representation (diagnostics JSON stays at version 1). Int generics use `bound: {"const": "Int"}`; items whose signature or body uses M2 carry a canonical `body_source` string (complete braced body, same formatter) and omit legacy insts/calls/nets summaries rather than presenting partial counts; generic-argument expressions use canonical authored text, derived spec values use §2's value-text rule. Registry validation/storage and the viewer support both versions before v2 uploads are claimed (update the v1-only envelope validator); unsupported versions produce an explicit unsupported-format result, never a silent empty-summary downgrade. Best-effort docs upload never fails a successful package publication.

* Implementation order follows the type boundary: typed expressions and shared substitution → declaration validation for new frames → concrete evaluation/expansion → final-graph/provenance integration → budgets → fmt/LSP/docs/Explorer, shipped together.

* No new CLI command, plugin, network call, JSON diagnostics schema or manufacturing dialect; no new compiler dependencies; no unordered-map iteration leaks; existing subdesign paths and port metadata are never discarded. A richer exported hierarchy remains RFC-032's own implementation obligation, neither claimed complete nor waived here.

## Teaching cost

Medium-high. Start with one and two LEDs, then the RC subdesign with named array elements and an independent layout. Teach: Int versus units; one-net fan-out versus repeated per-iteration nets; inclusive versus exclusive endpoints; type information versus concrete values; the loop label as an operation-site name (diagnostics + stable identity), never a circuit object. Teach helper/subdesign extraction as the construction path; mention B only as deferred design history. Source brevity and permanent conceptual cost are separate review dimensions.

## Failure modes

* Off-by-one neighbor loops — caught at the concrete failing iteration with its binding in the main message.

* Silent shorting across iterations — impossible by frame scoping; explicit sharing merges through existing rules.

* Vanished real components or fake loop BOM rows — impossible by construction; containers/frames carry no manufacturing identity, every real inst is emitted.

* Incorrectly scoped defaults or shadowing — rejected at definition validation.

* Hidden empty-body errors — decidable errors fail statically; skipped activations are never value-specialized.

* Resource exhaustion — deterministic E1405 at the declared limits, before materialization, never partial success.

* False claims guarded: successful generation is not electrical simulation; computable pitch does not prove clearance; a numeric N does not select a converter; the E140x block replaces no part/pin/DRC check. An editor may retain a visibly failed partial model, but a build that dropped a required component to recover is never reported successful.

## Migration path

Purely additive syntax; no existing `.cohdl` source requires change. Acceptance in one design change: record the scope and syntax decisions (A; L1 + R1), allocate the RFC number and E140x block centrally, amend RFC-001/005/006/007/024/032 and note 10 together, land the declaration-validator compatibility audit with its enumerated corrections, and update `docs/error-codes.md` and `docs/apidocs.md`. Implementation follows acceptance in the order given under Tooling; only implemented fixtures become runnable teaching source. General electrical calculation/selection remains M3; new electrical contracts remain M4; no pure-fn migration is implied.

## Decision

**Proposed — ready for acceptance review.** Scope is Candidate A (selected 2026-09-12): typed Int/Length compile-time computation, local constants, `const N: Int` generics shared by fn and subdesign, checked expression counts/indexes/selectors for both array kinds, mandatory-labelled half-open `for` loops over net/nc/helper-call/placement operations, expression-valued placement, hygienic label+value iteration frames feeding the existing designator allocator, uniform static declaration validation, full expansion-graph metering (100,000 iterations / 1,000,000 work items / 64 frames), the E1401–E1407 diagnostic block, one-token minus lexing, and package API docs schema v2. Direct loop-body `inst`/`subdesign` declarations (Candidate B) remain deferred. A PR-43 review revision (still Proposed, not Accepted) rejects directly authored named nets inside loop bodies with E1406 — the named net is declared outside the loop and joined inside via anonymous `net _` connections. Formal acceptance still requires the decision record, central RFC/error-code allocation, the pre-acceptance compatibility audit in §Compatibility, and synchronized amendment of note 10 and affected RFCs. No M2 behavior is claimed implemented or executed.
