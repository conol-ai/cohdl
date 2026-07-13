# 4. Principle → Constraint Mapping

# Status

v2 — reset for the ground-up redesign, 2026-07-13. Same purpose as v1 (turn principles into yes/no-checkable rules), rewritten against the v2 Constitution and Conceptual Model. The most important addition is a whole new principle — strictness buys expressiveness — that v1 didn't have a name for.

# Purpose — unchanged

Principles are inspiring but unenforceable until they become concrete constraints. A constraint is good when you can answer "did we violate it?" with yes/no.

# Principle: strictness buys expressiveness (new in v2)

Illegal states should be unrepresentable wherever possible; the resulting safety margin is what licenses generics, traits, and composition to go further than a looser language would dare.

Constraints:

- Any mistake that is structural (belongs to one device/instance/trait, not the whole graph) must be a compile-time type error, never a rule/DRC finding. Before adding a rule, the RFC must show why the check can't be a trait bound, required spec, or pin obligation instead.
- No implicit unit coercion, ever. Farads, Voltage, Ohms, etc. are distinct types; a bare number where a unit is expected is a type error.
- No pin may be silently unconnected. Every required pin must resolve to a net membership or an explicit nc before the design type-checks.
- Trait satisfaction is checked at the impl site, not deferred to instance-checking or DRC. An invalid impl must fail to compile, so no instance of it can ever exist to be checked later.
- A new expressiveness feature (generic, trait bound, pattern-match form) must not weaken any of the above. If a generic mechanism makes it possible to construct an instance that bypasses pin-obligation or trait-satisfaction checking, the mechanism is rejected regardless of how convenient it is.

# Principle: AI-generatability (regularity & locality) — carried forward

Constraints:

- The grammar must stay deterministic — no context-sensitive parsing, no unbounded lookahead.
- One canonical way to express each thing; a second syntax replaces the first, never coexists.
- **No feature whose correctness depends on state outside the current file's imports.**
- cohdl fmt must define a canonical form from the language's initial release — not a placeholder shipped "for now." This is now non-negotiable given v1's own experience with deferring it.

# Principle: gradeability (the compiler is the oracle) — sharpened

*Every notion of "correct" must reduce to a deterministic compiler signal — and the earliest possible signal wins.*

Constraints:

- No language feature may exist that neither the type system nor DRC can inspect.
- Every rule/type error must produce a **stable error code + precise span + message naming the violated constraint.**
- The pipeline must be reproducible: same source + std version → identical verdict, designators, netlist bytes.
- A check that could be caught at type-check time but is instead left to DRC is a design defect, not a valid choice, unless it's demonstrably cross-cutting/numeric (net voltage, multi-driver). This is the sharpened form of v1's "dormant rules are bugs" lesson: the fix isn't just "wire the DRC rule," it's "ask whether DRC was the right layer at all."
- New "correctness" claims ship with the check that enforces them, in the same change — unchanged from v1, still non-negotiable.

# Principle: human reviewability & trust — carried forward

Constraints:

- Any pin that joins a net must do so through a **visible **`net`** declaration**. No invisible/auto connectivity.
- Diagnostics must be actionable by a human reader, not just a machine.
- Intent should be attachable (#[intent(...)]) — reserved seam, never affects the netlist.
- A design must be reviewable in diff form — small logical change, small local source diff.
- No "magic defaults" — every spec default is either required or visibly defaulted in source, never silently filled by the type checker.

# Principle: explicitness over hidden magic — carried forward, extended

Constraints:

- Connectivity is never inferred — only from explicit net declarations.
- Specs and packages that affect output must be **written or explicitly defaulted with a visible default**.
- Codegen must not inject nets, components, or connections beyond ConnectivityIR.
- New: a pin's connection state is never ambiguous. "Not mentioned" is not a valid state — it must be net-connected or explicitly nc. This turns v1's implicit-silence problem into an explicitness violation with its own constraint, not just a hoped-for DRC catch.

# Principle: locality of meaning — carried forward

Constraints:

- Reading one module must not require reading the whole design to know what its symbols mean.
- Every diagnostic points at the **smallest responsible span**.
- A fn sub-circuit's behavior must be fully determined by its parameters + imports — no ambient design state. This constraint now explicitly includes nested fn calls: a nested call's behavior must be equally determined by its own parameters/imports, with no special-casing based on call depth (closing the exact gap that made v1's nested-fn support silently break).

# Principle: regularity over cleverness — carried forward

Constraints:

- Reject syntax sugar that saves keystrokes but adds a parsing special case.
- A new concept must not overlap an existing one (guard the canonical vocabulary).
- Prefer extending an existing concept over inventing a parallel mechanism.
- New: prefer extending the type system over adding a new DRC rule, given a genuine choice between the two — this is the same principle, sharpened for the redesign's central bet.

# Principle: stable concepts over convenient features — carried forward

Constraints:

- A new concept must justify its **permanent generation + learning + maintenance cost** in the RFC.
- Concepts do not change meaning across versions without a **deprecation cycle**.
- Error codes are stable contracts — and because v2's error-code registry will look substantially different from v1's (several DRC codes become type-checker diagnostics instead), the initial registry for v2 must be treated as the stable baseline going forward, not a work-in-progress to reshuffle casually after first publication.

# Principle: persistent identity — carried forward, strengthened

Constraints:

- Reference designators persist in `design.lock`, keyed by hierarchical path; removed instances are **tombstoned**, never reused.
- The designator allocator must be collision-free by construction, not merely collision-free in the fixtures tested so far. Prefer an algorithm whose totality/injectivity can be argued directly (e.g. a pure function of hierarchical path + prefix + an ordinal derived deterministically from declaration order) over an incrementing global counter — the latter is exactly the shape of algorithm that produced v1's U3 collision.
- A revision that doesn't change a component's identity **must not change its designator**.

# Principle: tooling & operations are part of the product — carried forward

Constraints:

- The compiler must expose a stable, structured verify output (cohdl check --json) — designed alongside the type checker this time, not retrofitted.
- Diagnostics, error codes, and formatter output are **versioned surfaces** with compatibility promises.
- Anything the AI repair loop depends on is public API, governed accordingly.

# Principle: faithful, lossless ecosystem output — carried forward

Constraints:

- Every connection and spec in ConnectivityIR must appear in the emitted netlist — **no silent drops**.
- Emitted formats carry their **target-format version**.
- Data a target can't represent must **fail loudly or be documented as dropped**.
- Because MPN and specs are now type-complete by construction, the BOM emitter must never need an <UNSPECIFIED> fallback path at all — if the type checker did its job, that code path is dead code; if it isn't dead code, that's a signal the type system has a gap, not a signal the BOM emitter needs a patch.

# The layout door — constraints on the seam, unchanged from v1

- Layout constraints attach as declarative decorations on Net/Instance, never a second connectivity/routing mechanism.
- Must be inspectable and gradeable.
- Must be losslessly ignorable/passable.
- Nothing about the door may weaken the "CoHDL is not a router" non-goal.

# Using this note — unchanged

In any RFC, quote the specific constraint a feature might strain and argue why it doesn't. If a proposal can't be reconciled with a rank-1 or rank-2 constraint, it's rejected regardless of rank-6 convenience. New for v2: if a proposal adds a rule, it must first show it fails the "could this be a type-system mechanism instead" test — that test is now a mandatory RFC section (see note 6).
