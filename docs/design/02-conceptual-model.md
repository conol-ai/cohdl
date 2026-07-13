# 2. Conceptual Model

# Status

**v2 — redesigned from scratch, 2026-07-13.** This is the actual redesign: the previous concept table is void. What's preserved is the *test* the model must pass (below) and the electrical first-principles that don't change no matter the syntax (a board has components, connections, constraints, identity). Everything about *how* those are expressed is new.

# The test every concept must pass — unchanged in spirit, sharpened

> Can an AI reason about each concept in isolation, and can a human predict what it means without reading the whole design — **and is it structurally impossible to misuse in a way the compiler can't catch before DRC even runs?**
>

That third clause is new. In v1, "gradeable" meant "eventually checked, possibly by a DRC rule." In v2, gradeable defaults to "checked as a type error, as early and as locally as possible." A concept only gets to lean on DRC when the thing being checked is genuinely emergent from the whole connectivity graph — not when it's a property of one device/instance the type checker could have refused to accept.

# The core electrical primitives (first principles — domain truth, syntax-independent)

No matter what the language looks like, a schematic reduces to the same nouns. These aren't the redesign — they're the bedrock the redesign expresses more strictly:

- A **component type** with an electrical contract (what pins it has, what it needs to be valid).
- A **placed instance** of a component, with **stable physical identity**.
- **Connections** between instances' pins.
- **Typed engineering parameters** (capacitance, voltage, current — always with units, never bare numbers).
- **Constraints** — some local to one device, some emergent across the whole graph.
- A **hierarchy** for reuse and namespacing.

# The v2 core concepts

| Concept | What it is | What's new vs. v1 |
|---|---|---|
| **Trait** | An electrical contract: required pins, spec fields, sub-trait bounds a device must satisfy. | Trait satisfaction is checked **at **`impl`** time**, not deferred. A device claiming `impl Capacitor` without a `capacitance` spec is a compile error on the device definition itself — you cannot define an invalid `impl`. (v1: this was DRC rule E004/E005, checked per-instance, and was dormant.) |
| **Device** | A parametric component type: pins, package, specs, traits implemented. | Pins are typed with an explicit **connection-obligation** kind (`required`, `optional-explicit`, see Pin below). Generic specs (`Device<C: Farads>`) are unchanged in spirit — generics are how expressiveness survives strictness. |
| **Part** | A device bound to a real purchasable component: MPN, manufacturer, AVL alternates. | MPN binding is **type-checked as non-optional the moment a **`part`** is declared** — a `part` block that doesn't fully resolve its MPN is a compile error, not a silently-`None` field that leaks to `<UNSPECIFIED>` in the BOM later. This directly closes the v1 MPN-propagation bug at the type level instead of patching the codegen path. |
| **Instance** | A concrete placed component in a design, owning a reference designator. | Unchanged in shape. Still the identity-bearing concept. |
| **Pin** | A named connection point on an instance. | **New: every pin has a connection-obligation kind, checked exhaustively.** `required` pins must appear in some `net` or be explicitly marked `nc` (not-connected) — leaving a required pin absent from both is a compile error, not silence. This is the type-system answer to "a human forgot to wire something" — it becomes as unrepresentable as an unhandled match arm in Rust. |
| **Net** | A set of pins declared electrically connected. | Unchanged mechanism (still the only connectivity mechanism — no auto-wiring). New: net *participation* is exhaustively checked against every instance's required pins (see Pin) as part of type-checking, not as a late connectivity pass discovering orphans. |
| **Spec** | Typed engineering parameters on a device/instance. | Units are part of the type, not documentation — `Farads`, `Voltage`, `Ohms` are distinct types with no implicit numeric coercion between them or from a bare number. A bare `100` where a `Farads` is expected is a type error, full stop (closes off a whole class of "silently wrong units" mistakes that no v1 DRC rule specifically targeted). |
| **Rule** | A DRC assertion, now explicitly scoped to **cross-cutting / emergent / numeric** checks only. | **Narrowed on purpose.** If a check can be expressed as "does this one device/instance/trait satisfy a structural condition," it does NOT belong in `rule` — it belongs in the type system (trait bound, required spec, pin obligation). `rule` is reserved for things like "net voltage across these two pins ≤ rating" or "this net has more than one driver" — checks that only make sense once the whole graph exists. |
| **Module** | A namespace grouping devices/parts/fns; controls visibility (`pub`). | Unchanged. |
| **Fn (sub-circuit)** | A parameterized, reusable circuit fragment, monomorphized and inlined. | **Nested fn calls (fn calling fn) are a first-class, required capability from day one** — not an edge case to patch later. Composability is ladder rank 4; a sub-circuit language that can't nest sub-circuits fails its own test. |
| **Design** | The top-level board: instances + nets that compile and emit. | Unchanged. |
| **Designator** | The stable physical identity of an instance, persisted in `design.lock`. | Unchanged mechanism. New constraint carried from v1's own postmortem: the allocator must be proven collision-free by construction (a total function over hierarchical path, not an incrementing counter that can double-assign) — this is a design requirement on the *algorithm*, not just a bug to fix later. |

# The strictness mechanisms (the "hard to make mistakes" half)

These are the concrete, Rust-flavored (not Rust-copied) mechanisms that do the actual work of making illegal states unrepresentable:

- **Units as types, not comments.** `Farads`, `Voltage`, `Ohms`, `Hertz` are distinct types. No implicit coercion from a bare number or between unit types. Engineering-notation literals (`100nF`, `3.3V`) are how you write them; the type is what the compiler checks.
- **Exhaustive connection obligations on pins.** Every pin is `required` or `optional` by trait/device definition. A `required` pin must be resolved — connected via `net`, or explicitly `nc` — before a design type-checks. No implicit "unconnected pins are probably fine."
- **Trait satisfaction checked at **`impl`**, not at use.** You cannot define `device X: impl Capacitor` unless `X` truly has everything `Capacitor` requires, checked the moment the `impl` is written — invalid devices can't exist in the type system at all, so there's nothing for a later DRC rule to catch.
- **No optional/nullable specs by silent default.** A spec field is either required (compile error if missing) or has a default *visible in source* (never a hidden fallback the type checker fills in quietly). This directly targets the "magic defaults" smell v1's own principles warned about, and makes it a parse-level property instead of a code-review discipline.
- **Exhaustive pattern-matching over structural variants.** Package variants, differential-pair roles, AVL alternates — anywhere the v1 design had "a case you might forget to handle," v2 expresses it as a `match`-like construct the compiler can require to be exhaustive, the same way Rust makes forgetting an enum arm a compile error.
- **The designator allocator is a pure, collision-free function by construction** — not an incrementing-state algorithm that happens to usually work. Provable, not just tested.

# The expressiveness mechanisms (the half strictness pays for)

- **Generics over specs**, with visible defaults (`Device<C: Farads, V: Voltage = 10V>`) — say one device, get a family.
- **Trait composition** — devices implement multiple traits; traits can bound on sub-traits (`Capacitor: TwoTerminal`). This is exactly the Rust move: strict trait-checking is what makes composing many traits onto one device *safe* rather than a pile of ad hoc special cases.
- **Sub-circuit **`fn`**s that nest and monomorphize** — a `fn` is a generic, reusable circuit; nesting them (v1's broken case) is how real designs get composed out of small, trusted pieces instead of one flat instance list.
- **AVL / part alternates as a typed set**, not a loosely-optional field — expressive (many alternates, one canonical spec) without being unsound (every alternate must satisfy the same trait bounds as the primary).

# The compilation objects (derived, not authored) — reframed verdict ladder

The pipeline's output objects are conceptually similar to v1, but the *rungs* of the verdict ladder shift weight toward type-checking, reflecting where correctness now actually gets caught:

`parses ⊂ resolves ⊂ type-checks (traits satisfied, units sound, pin obligations resolved, specs complete) ⊂ connects ⊂ passes residual DRC (numeric/cross-cutting only) ⊂ emits netlist`

The **type-checks** rung is now doing most of what v1 spread across "type-checks" + a pile of DRC rules (E003–E005 in v1's numbering no longer need to exist as DRC — they become type errors). **Passes DRC** is now a much smaller, sharper rung: only checks that are inherently about the whole graph (net voltage across a set of pins, single/multi-driver conditions) remain there.

- **Diagnostic** stays the most important AI-facing object — `(code, severity, span, message)`, ideally with a machine-actionable suggestion. Unchanged in shape from v1; still elevated to first-class product surface.
- **design.lock** — hierarchical path → designator map with tombstones. Unchanged mechanism; stronger correctness requirement on the allocator (see above).
- **Netlist / BOM** — unchanged as the bridge to the physical world. New: because MPN and units are now compile-time-complete by construction, "the BOM lies" (v1's `<UNSPECIFIED>` bug) becomes structurally impossible rather than something to test for after the fact.

# Properties every concept must keep — unchanged from v1, still the test

Stable · Orthogonal · Composable · Learnable · Extensible · Toolable. Nothing here changed; what changed is *how much work the type system does to guarantee them* versus leaning on convention or a later rule pass.

# Model smells to reject (design regressions) — carried forward, one added

- **Two ways to connect.** Still rejected — connectivity is only ever `net`.
- **A concept that behaves differently by context.** Still rejected.
- **Correct-by-convention features.** Still rejected — and now interpreted more strictly: "the DRC can inspect it eventually" is no longer sufficient; if it's structural, it must be a type error, not a rule.
- **Overlapping names.** Still guarded — Trait / Device / Part / Instance / Net / Spec / Rule / Module / Fn / Design / Designator remains the canonical vocabulary (unchanged names; redesigned guarantees).
- **A feature that bypasses the core model.** Still rejected.
- **New smell: reaching for **`rule`** (DRC) when a type-system mechanism would do.** This is the v2-specific regression to watch for — the exact mistake that produced v1's five dormant rules. Every new `rule` proposal must justify why it *can't* be a trait bound, a required spec, or a pin obligation instead (see the RFC template, note 6).

# The seam for the layout door — unchanged from v1

When layout constraints arrive, they attach as a new concept adjacent to Net/Rule (a `net_class`/`constraint` decoration), never a second connectivity mechanism, inspectable and gradeable like a rule, losslessly ignorable/passable through codegen. Nothing about this redesign changes that seam — the strictness push applies to the schematic domain, not to pre-deciding how layout constraints will be checked before that door is even opened.

# What this redesign explicitly does NOT do

- It does not add ownership, borrowing, or lifetimes. Hardware's "resource" is a *pin claimed by exactly one net-membership per role*, which is already handled by the connectivity/net model — there is no aliasing problem to solve, so there is no borrow checker to build. Reaching for one would be conceptual cost with no corresponding job to do, which the Coherence Matrix (note 5) will reject on sight.
- It does not promise every DRC rule from v1 becomes a type error. Numeric/cross-cutting checks (net voltage vs. rating, multi-driver detection) stay in `rule` — the redesign narrows DRC's job, it doesn't eliminate it.
