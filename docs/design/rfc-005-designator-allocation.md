# RFC-005: Collision-free designator allocation

## Problem

v1's real allocator (`crates/cohdl-sema/src/designator.rs`, confirmed by reading the actual source) works like this: keep a mutable `used: HashSet<String>` seeded from the lock file's existing designators + tombstones, then iterate instances in whatever order they were collected, inserting into `used` as it goes, calling `next_available(prefix, &used)` to find the lowest free number for each new instance. This is a **stateful, order-dependent, incrementally-bookkept algorithm** — its correctness depends on every insertion into `used` happening before the next lookup that could collide with it, on the lock file's `used` set being fully in sync with what's actually being computed, and on no code path skipping the bookkeeping step. The reported v1 bug — `esd` (`ESD_USB`) and `ldo33` (`AP2112K_3V3`) both landing on `U3` in the `conol-pin` fixture — is exactly the kind of bug this shape of algorithm produces: both devices fall back to the default `"U"` prefix (neither implements a trait with a mapped designator prefix), and somewhere in the stateful accumulation, the `used` bookkeeping did not correctly account for one of them before the other's lookup ran.

This is precisely the failure mode DR-005/DR-006 use as evidence for the whole redesign's thesis: not "add a test for this specific bug," but "an algorithm whose correctness depends on careful, order-sensitive mutation is the wrong *shape* for something this important" — the fix should make the bug class structurally impossible, not just patch the one instance of it that got noticed.

Who this is for: **every author whose design gets compiled** — designator collisions are physically dangerous (two components silently sharing a reference designator can make a BOM/netlist/assembly-drawing inconsistent in ways that are easy to miss and expensive to catch after fabrication). This is also foundational for **tooling** (design.lock is the persistent-identity artifact every diff/review/rebuild depends on).

## Goals

- Replace the incremental, order-dependent `used`-set-mutation algorithm with a **pure, total function** from (live instance paths + their prefixes + existing lock-file assignments + tombstones) → a designator assignment, computable in one pass with no iteration-order dependency.
- Make **injectivity** (no two distinct paths ever share a designator) a property provable directly from the function's construction, not something that happens to hold if every call site remembers to update shared mutable state correctly.
- Preserve every real v1 behavior worth keeping: designator stability across rebuilds, explicit `#[designator("Xxx")]` overrides, tombstoning on removal, never reusing a tombstoned designator.

## Non-goals

- **Not changing the designator format** (`PrefixNumber`, e.g. `C1`, `U3`) — that's a separate, orthogonal question (and not one raised as a problem). This RFC is about the *allocation algorithm's* correctness, not the designator string's shape.
- **Not changing **`design.lock`**'s file format** — TOML with `[designators]` / `[tombstones]` tables is fine; this RFC only changes the in-memory algorithm that populates them.
- **Not solving cross-run parallelism/concurrent-compilation safety.** A single compilation run is assumed to allocate designators sequentially in one process; distributed/concurrent build safety is out of scope unless it becomes a real requirement later.

## Design

### The allocator as a pure function, not a stateful loop

Model assignment as: given (a) the **prior lock-file state** (existing designator assignments + tombstones, both immutable inputs) and (b) the **live instance set** for this compilation (each with its hierarchical path, optional override, and prefix), produce a new assignment map via a single deterministic computation with an explicit, checkable invariant — rather than v1's mutate-`used`-as-you-go loop.

```text
$64
```

The critical difference from v1: **Step 3 computes the full reserved-number set for a prefix once, immutably, before assigning any fresh numbers**, and assigns fresh numbers as *positions in one sorted sequence* rather than repeated independent "scan up from 1 until free" searches against a set that's being mutated concurrently with the scanning. Two instances needing the same prefix can never race to claim the same number, because there is no shared mutable state being updated between one instance's assignment and the next — the entire reserved set and the entire list of "who needs a fresh number" are fixed before any assignment happens.

### Designator stability and tombstoning — unchanged behavior, now derived from immutable inputs

- A live path with a prior lock-file assignment keeps it — Step 1, unconditionally, regardless of source-order changes. (Unchanged from v1's intent; the difference is this is now a pure lookup against an immutable prior-state snapshot, not a lookup against a `self.designators` map that Step 3 will later mutate in place.)
- A path removed from the live set is tombstoned (moved to `[tombstones]`) in a separate, explicit step after assignment — its designator enters the permanently-reserved set for that prefix, so a future fresh assignment can never reuse it. This is unchanged from v1's intent but is now one of the immutable inputs Step 3's reserved-set computation reads, rather than a `HashSet` both loaded from and written back into during the same pass.
- Explicit `#[designator("Xxx")]` overrides are resolved in Step 2, before any fresh numbering happens in Step 3 — so a fresh-assignment position can never "step on" an override that hasn't been accounted for yet, because overrides are folded into the reserved set *before* Step 3 starts, not interleaved with it.

### Example: why this specifically closes the `esd`/`ldo33` bug class

In v1, if `esd` and `ldo33` were both new instances needing the default `U` prefix, and both were processed in the same Phase 3 loop, the bug is possible if anything about the loop's bookkeeping (the `used.insert(desig)` call, or the order instances were collected in) didn't atomically happen before the next lookup. In this RFC's design, both `esd` and `ldo33` are in the same "needs fresh `U` designator" group; the reserved-`U`-number set is computed once, immutably, before either is assigned; and their fresh numbers are the 1st and 2nd integers missing from that set, respectively, assigned by their position in one sorted list — there is no "lookup, then insert, then next lookup" sequence at all, so there's no window in which one assignment can fail to be visible to the other. Two instances **cannot** receive the same fresh number for the same prefix, because they're not each performing an independent search — they're reading off fixed positions in one pre-computed sequence.

## Type-system-first test

N/A — this is an algorithmic-correctness RFC (a compiler-internals design), not a language surface feature; there's no `rule`/DRC question to route.

## Conceptual impact

None. **Designator** (existing concept, per note 2) is unchanged in meaning — this RFC only changes the internal algorithm that computes designator assignments, not what a designator *is* or means to an author. No new syntax, no new concept.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | High | Med | High | N/A (pre-launch) | High |

**Oracle (High):** this directly fixes a class of bug that would make the compiler's own "correct" verdict a lie — two components sharing a designator is a real, physically dangerous inconsistency the v1 oracle didn't reliably prevent.
**Netlist (High):** designator collisions propagate directly into BOM/netlist output — a fix here is a direct netlist-fidelity improvement, not just an internals cleanup.
**Trust (High):** the explicit, checked injectivity postcondition (Step 4) means the guarantee isn't "we believe the algorithm is correct," it's "the compiler verifies its own allocator's output every single run" — a stronger, more auditable trust story than v1 had.
**Grammar/Concepts (Low):** no author-facing surface changes at all.

## Gradeability

The injectivity check (Step 4) is itself a compile-time assertion the compiler runs on its own output, every run — if the allocator ever produced a collision (which the construction should make impossible, but the check exists as a defense-in-depth backstop against an implementation bug in the allocator itself), that would be a compiler-internal error, not a silent bad netlist. This is the correctness guarantee's actual enforcement point — not a unit test that might not cover every input shape, but a runtime-checked postcondition on every real compilation.

## AI-generatability

Not directly applicable — this is a compiler-internals RFC; there's no `.cohdl` syntax for a model to generate differently. Indirectly: a model authoring designs can trust that `#[designator("Xxx")]` overrides behave predictably (conflict detection happens before fresh assignment, per Step 2/3 ordering), which is a small but real generatability improvement over an allocator whose override-conflict behavior depended on loop-iteration order.

## Alternatives

- **Patch v1's algorithm to fix the specific observed bug** (find and fix whatever exact ordering issue produced the `U3` collision) — rejected: this is patching a symptom of the wrong algorithm *shape*, not the shape itself; a future edge case in the same stateful-mutation pattern could reproduce an equivalent bug elsewhere, and there'd be no structural reason to trust it wouldn't.
- **Keep an incrementing global counter per prefix, reset per compilation** — rejected: still has the same class of correctness dependency on strict sequential processing order; doesn't buy anything over the reserved-set-then-positional-assignment approach, and is harder to reason about when tombstones/overrides interact with the counter.
- **A randomized/hash-based designator assignment** — rejected: breaks human reviewability (designators should read as a small, dense, meaningful sequence like `C1, C2, C3`, not arbitrary hash-derived numbers) and breaks the "same source → same output bytes" reproducibility hard constraint if the hash isn't perfectly stable across runs/versions.

## Compatibility

N/A — pre-launch, no existing `design.lock` files to migrate (the v1 fixtures are being discarded along with the rest of the v1 implementation, per DR-005).

## Tooling & operations

- The injectivity postcondition (Step 4) should itself be a stable, documented internal invariant — if it ever fails, that's a compiler bug report with a clear, specific signal ("allocator produced non-injective output"), not a confusing downstream BOM inconsistency discovered later.
- `cohdl build`/`check` output should be able to show, on request, the full reserved-number set computed for a given prefix — useful for debugging why a particular fresh instance received the designator it did, especially with tombstones/overrides in play.
- Golden-file/property-based tests should directly exercise the "two new instances, same prefix, in whatever order they're collected in" case (the exact `esd`/`ldo33` shape) and confirm the assignment is order-independent — i.e., re-running the same live set in a different collection order produces the identical assignment, not just a non-colliding one.

## Teaching cost

None for `.cohdl` authors — this is entirely a compiler-internals change. For future compiler implementers/maintainers, the teaching cost is actually *lower* than v1's algorithm: a pure function with an explicit postcondition is easier to reason about and test than a stateful loop whose correctness depends on call-order discipline.

## Failure modes

- **An implementer reverts to an incremental "keep a mutable used-set, insert as you go" shape for convenience**, believing it's equivalent — must be caught by the mandatory injectivity postcondition (Step 4) and by the order-independence test described in Tooling & operations; this RFC's design is specifically meant to make that regression class visible immediately, not months later on a real design.
- **Tombstones or overrides are folded into the reserved set inconsistently** (e.g. computed after fresh assignment has already started, reintroducing an ordering dependency) — must be prevented by construction: Steps 1–2 (existing + overrides) must fully complete, and their results fully fold into the reserved-number set, before Step 3 (fresh assignment) begins at all, for every prefix independently.
- **Two different prefixes are accidentally allowed to collide** (e.g. `C3` and `U3` are fine to coexist, but a bug conflates two different prefixes' reserved sets) — the reserved-number set must be computed **per-prefix**, never globally, since designators are only required to be unique as a whole string (prefix + number), and different prefixes naturally don't collide with each other.

## Migration path

N/A — pre-launch.

## Decision

**Accepted** — 2026-07-13. Recorded as DR-008 (see note 7 — this was the number originally reserved for this exact decision, so no renumbering needed). Language Specification (note 10) will note this in a new "Designators" section, since although this is compiler-internals, the *guarantees* (stability, no collision, tombstoning) are user-facing promises worth documenting even without new syntax. RFC-002/003's pin/trait mechanisms are unaffected — this RFC is fully orthogonal to them.
