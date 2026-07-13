# RFC-006: Nested fn call semantics

## Problem

v1's real source (`crates/cohdl-sema/src/typeck.rs:1370-1372`, confirmed by reading it directly) shows the exact bug: the design-body expansion loop handles a top-level `Call` statement by expanding the called `fn`'s body — instantiating its `inst` statements, wiring its `net` statements — but when that `fn`'s own body contains **another** `Call` statement (calling a second `fn` from within the first), the match arm is a literal no-op: `FnBodyStmtKind::Call(_) => { /* Nested function calls not yet supported. */ }`. Nothing is instantiated, nothing is wired, and — critically — **no error is raised**. A design that calls `fn_a()`, which internally calls `fn_b()` to instantiate a decoupling capacitor or a sub-regulator, silently ends up missing every instance and net `fn_b()` was supposed to contribute, with a clean compile and no diagnostic. This is a textbook instance of the redesign's core complaint: not a check that's dormant, but a code path that's simply absent, disguised as success.

Who this is for: any author (human or AI) composing designs out of reusable sub-circuits — which is the entire point of `fn` per the Conceptual Model ("a `fn` is a generic, reusable circuit fragment"; nesting them is "how real designs get composed out of small, trusted pieces instead of one flat instance list"). A sub-circuit library that can't call other sub-circuit library functions fails at the one job composability exists to do.

## Goals

- Nested `fn` calls (a `fn`'s body calling another `fn`) work correctly, to arbitrary depth, with the same instantiate-and-wire semantics as a top-level call.
- Generic parameter substitution threads correctly through nesting — a nested call's generic arguments may themselves reference the *outer* call's generic parameters, and must resolve to concrete types by the time the innermost `fn` is expanded.
- Naming/hygiene: every instance and net produced by expansion, at any nesting depth, gets a unique, deterministic, hierarchical name — no collisions between two different call sites of the same nested `fn`, and no collisions with RFC-005's designator allocator input.
- If nesting cannot be resolved for some structural reason (e.g. unbounded/infinite recursion), fail loudly with a precise diagnostic — never silently skip, which is the exact defect this RFC exists to close.

## Non-goals

- **Not adding new **`fn`** syntax.** `fn` declaration and call syntax are unchanged; this RFC is about the *expansion algorithm* correctly recursing, not a new language feature.
- **Not solving mutual/cyclic recursion as a supported feature.** `fn_a` calling `fn_b` calling `fn_a` is a structural error (see Design) — this RFC makes sure it's *caught*, not that it's somehow made to work; sub-circuits are a compile-time inlining mechanism, not a Turing-complete computation model, and cyclic composition has no sensible expansion (it wouldn't terminate).
- **Not covering the full interaction with RFC-007's generics-over-specs design** (which hasn't landed yet) beyond what's needed for correct generic-substitution threading through nesting — RFC-007 may refine the generic-parameter mechanism further; this RFC only needs nested calls to correctly thread whatever substitution mechanism already exists.

## Design

### Nested calls expand via the same recursive procedure as top-level calls

v1's actual bug is a **missing recursive case**, not a wrong architecture — the design-body-level expansion procedure (find the `fn` definition, build generic substitutions, instantiate each `inst` statement, wire each `net` statement) is correct; it simply needs to invoke *itself* when it encounters a `Call` statement inside a `fn` body, instead of doing nothing:

```text
fn expand_call(call, substitution_context, naming_context, depth):
    fndef = resolve(call.path)
    if fndef is None:
        return  // name resolution already reported an error elsewhere

    if naming_context.contains_active_call_to(fndef):
        error("recursive fn call: `{fndef}` is already being expanded in this call chain")
        return

    new_substitution = build_generic_substitution(call, fndef, substitution_context)
    new_naming = naming_context.push(fndef, call.site)

    for stmt in fndef.body:
        match stmt:
            Inst(inst) => instantiate(inst, new_substitution, new_naming)
            Net(net)   => wire(net, new_naming)
            Call(nested_call) =>
                expand_call(nested_call, new_substitution, new_naming, depth + 1)
                // <-- the actual fix: recurse, instead of doing nothing
```

- `substitution_context`** threads outward-in.** A nested call's generic arguments may reference the *outer* call's own generic parameters (e.g. `fn_a<C: Capacitance>() { fn_b::<C>() }` — `fn_b`'s argument `C` refers to `fn_a`'s own generic parameter). The substitution map passed into the recursive `expand_call` must already have the outer call's own substitutions resolved, so a nested call's generic arguments are resolved against concrete types by the time the innermost `fn` needs them — never left as an unresolved symbolic reference.
- `naming_context`** threads a call-chain path, not just a counter.** Every instance/net name is derived from the *full call chain* that produced it (e.g. `__fn0_fn_a::__fn1_fn_b::c1`), not a single flat `call_counter` — this guarantees two different call sites (even of the same `fn`, even at the same nesting depth) never collide, because the chain itself is part of the name, and naturally extends v1's existing `__fn{N}_{name}_{inst}` convention one level per nesting depth instead of leaving it flat.
- **Cyclic recursion is detected, not silently looped forever or silently skipped.** `naming_context.contains_active_call_to(fndef)` checks whether the *current* call chain already contains an active (not-yet-returned) call to the same `fn` definition — if so, this is a compile error naming the full cycle, not an infinite expansion and not a second silent no-op.

### Example: what this fixes

```cohdl
fn decoupling_cap<V: Voltage>(pin: Pin) {
    inst c: MLCC<100nF, V>
    net _: pin, c.A
    // (GND side wired by caller convention, illustrative)
}

fn power_rail<V: Voltage>(vdd_pin: Pin) {
    inst ferrite: Ferrite_Bead
    net _: vdd_pin, ferrite.IN
    decoupling_cap::<V>(ferrite.OUT)   // <-- nested call
}

design Board {
    inst mcu: MCU_ESP32S3
    power_rail::<3.3V>(mcu.VDD)
}
```

In v1, `power_rail`'s own `inst ferrite` and its `net` would be created correctly (it's a top-level call), but the nested `decoupling_cap::<V>(...)` call inside `power_rail`'s body would silently produce nothing — no `c` instance, no net wiring it in — with zero diagnostic, leaving `Board` looking complete while missing a decoupling capacitor entirely. In v2, `decoupling_cap` expands exactly as if it had been called directly from `Board`, with `V` correctly resolved to `3.3V` (threaded through `power_rail`'s own substitution), and its instance named uniquely along the full call-chain path.

## Type-system-first test

N/A — this is a compiler-internals expansion-algorithm fix (a missing recursive case), not a `rule`/DRC proposal.

## Conceptual impact

None. **Fn** (existing concept, per note 2) already stated nested calls are "a first-class, required capability from day one" — this RFC delivers exactly that stated intent, adding no new concept, syntax, or grammar. It closes a gap between the Conceptual Model's stated design and what the (discarded) v1 implementation actually did.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | High | High | High | N/A (pre-launch) | High |

**Oracle (High):** this closes an actual silent-wrong-output bug, not a cosmetic gap — a design that "compiled clean" while missing real instances/nets is exactly the kind of lying oracle DR-004/DR-006 exist to eliminate.
**Netlist (High):** directly affects whether nested-call-produced instances/nets appear in emitted netlists at all — this is as netlist-fidelity-relevant as a bug gets.
**Diagnostics (High):** the new cyclic-recursion detection is a new, must-be-precise diagnostic family (naming the full cycle, not just "recursion detected").
**Grammar/Concepts (Low):** no syntax change, no new concept — purely an expansion-algorithm fix plus a new error case.

## Gradeability

The recursive expansion procedure runs at compile time as part of the same design-body-expansion pass that already exists (unchanged pipeline stage) — no new pipeline stage, no DRC involvement. The cyclic-recursion check is itself a compile-time structural check (is the current call chain about to re-enter a `fn` it hasn't returned from yet?), decidable purely from the call graph being built during expansion — no need for a design/instance/connectivity-level check.

## AI-generatability

High, and a genuine improvement over v1: a model composing sub-circuits out of smaller `fn`s (the normal, encouraged composability pattern) no longer needs to know that nesting silently drops content — it can call one `fn` from inside another exactly as naturally as calling one at the top level, with the same semantics, the same generic-substitution behavior, and (if something goes wrong) the same class of precise diagnostic. Before this RFC, an AI author had no way to discover the nested-call gap except by manually inspecting the emitted netlist for missing instances — a much more expensive repair-loop cycle than a compile-time error would be.

## Alternatives

- **Leave nested calls unsupported, but make the compiler reject them with an error** (rather than fixing them) — considered as a fallback if full recursive expansion proved infeasible; rejected because full support is not actually hard (the existing top-level expansion procedure already does everything needed — it's a straightforward recursive generalization, not a new mechanism), and rejecting a core composability pattern would contradict note 2's explicit statement that nested calls are "first-class, required... not an edge case to patch later."
- **Flatten all nesting at parse/lowering time** (pre-expand nested calls into equivalent top-level calls before type-checking) — rejected: this would require the same substitution-threading and naming-uniqueness logic anyway, just relocated to an earlier, less-integrated pass; doing the recursion directly in the existing expansion procedure is simpler and keeps one pipeline stage responsible for all call expansion, not two.
- **Silently allow cyclic recursion with a depth cap** (expand up to N levels, then stop) — rejected: this is exactly the "compiles but silently wrong" failure mode this RFC exists to eliminate, just with a cap instead of zero levels; genuine cycles must be a compile error, not a silently-truncated expansion.

## Compatibility

N/A — pre-launch, no existing `.cohdl` source or v1 fixtures carrying forward.

## Tooling & operations

- The cyclic-recursion diagnostic must show the **full call chain** (e.g. "`fn_a` → `fn_b` → `fn_a`: recursive fn call detected"), not just "recursion detected at `fn_a`" — this is the same precision discipline RFC-003's sub-trait-chain diagnostics established.
- Fixture/golden-file tests must directly exercise: (a) a 2-level nested call, (b) a 3+-level nested call, (c) generic substitution threading through 2+ levels, (d) the cyclic-recursion detection case, and (e) two separate call sites of the same nested `fn` at the same depth, confirming their generated instance names never collide (this is the RFC-005-adjacent guarantee — the naming scheme here must feed a designator allocator input that's still guaranteed collision-free).
- `cohdl check --json`/LSP diagnostics for a cyclic-recursion error should carry the full chain as structured data (not just embedded in the message string), so tooling can render it as a clickable path, not just prose.

## Teaching cost

Low. An author already understands `fn` calls from the top level; this RFC makes nested calls behave identically, removing a special case rather than adding one — net teaching cost is arguably negative (one fewer "gotcha" to document: "nested calls don't work" is no longer a caveat the reference needs to carry).

## Failure modes

- **A model writes a deeply-nested call chain assuming unlimited depth is fine** — genuinely fine, as long as it's acyclic; no artificial depth cap is imposed by this RFC (unlike the rejected "depth cap" alternative) — only cycles are rejected.
- **Two separate call sites of the same nested **`fn`** produce colliding instance names** — must be prevented by construction via the call-chain-path naming scheme (see Design); a fixture test explicitly covers this case per Tooling & operations.
- **A cyclic call is only detected after significant expansion work has already happened**, producing a confusing partial result before the error — the check (`contains_active_call_to`) must run at the moment a call is *about to* be expanded, before any of its instances/nets are created, so a cyclic error never leaves partial, half-expanded state behind.
- **Generic substitution silently uses the wrong scope's value** (e.g. an inner call accidentally resolves against a stale outer substitution from a sibling call, not its actual caller) — must be prevented by threading substitution context strictly through the actual call tree being expanded (a stack-like structure naturally scoped to the current chain), never a flat/global substitution map shared across unrelated call sites.

## Migration path

N/A — pre-launch.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-015 (see note 7). Language Specification (note 10) gains a "Sub-circuit `fn`s" section reflecting this. RFC-007 (generics-over-specs) should treat this RFC's substitution-threading mechanism as the base its own generic-parameter design builds on for calls, nested or not.
