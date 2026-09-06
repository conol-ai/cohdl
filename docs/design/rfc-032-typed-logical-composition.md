# RFC-032: Typed logical composition without manufacturing identity

**Status: Draft — design review revised 2026-09-06.**

## Problem

CoHDL needs a way to compose a design from named, typed logical regions that
can connect through explicit interfaces without becoming fabricated parts.
The requirement is structural composition, not reproduction of a paper or CAD
schematic's page layout.

An initial implementation introduced `#[virtual]` on an `inst`. It proved a
useful implementation fact: an object can participate in resolution, pin
checking, connectivity, and residual DRC, then be absent from manufacturing
artifacts. However, using a device-shaped instance as a page boundary risks
adding a special kind of fake component to solve what may actually be a
hierarchy and composition problem.

The design question is therefore broader and more fundamental. `module` is
already CoHDL's source-organization and namespace mechanism; it can organize
the declarations involved, but it does not itself create circuit structure or
connect pins. The remaining semantic choice is:

> Should reusable logical composition use the existing expansion-oriented
> `fn` concept, or does CoHDL need a first-class typed `subdesign` concept that
> preserves hierarchy without carrying manufacturing identity?

## Goals

- Serve the Constitution's rank-1 correctness and rank-2 AI-verifiability
  goals: logical composition must remain fully compiler-checked.
- Model circuit structure rather than editor pages or drawing sheets.
- Keep every fabricated component subject to normal part binding, designator,
  footprint, placement, and BOM rules.
- Give authors an explicit typed interface between reusable logical regions.
- Decide whether retained hierarchy is required, rather than assuming it from
  the motivating schematic presentation.
- Preserve deterministic, truthful manufacturing output.

## Non-goals

- Introducing `page`, `sheet`, or coordinates for schematic presentation into
  the core language.
- A general simulation-only object model.
- DNP/DNI assembly-population states.
- Allowing real parts to disappear from manufacturing artifacts.
- Choosing syntax before the semantic distinction between `fn` and
  `subdesign` is settled.
- Making the current `#[virtual]` prototype normative merely because it exists.
- Whole-subdesign placement, rotation, mirroring, local board outlines, or
  independent fabrication outputs in the first version.
- Arbitrary references into another subdesign's private internal instances.
- Automatic wiring or a second connectivity mechanism beside `net`.

## Design

This Draft uses a layered model and evaluates three composition designs. No
syntax in this section is Accepted.

### Layer 0: `module` organizes source, not circuit hierarchy

```cohdl
// power.cohdl
pub fn power_section(vbat: Pin, gnd: Pin) {
    // reusable circuit fragment
}
```

```cohdl
use power::power_section
```

RFC-016 already gives `module` one precise responsibility: file-tree
namespaces, imports, qualification, and visibility. A module may contain a
`fn` or a future `subdesign`, but importing a module must not instantiate
components, create nets, or affect manufacturing output. Reusing `module` as
the electrical composition boundary would conflate where code is declared
with what circuit structure exists in a design.

### Option A: existing `fn` composition

```cohdl
fn power_section(vbat: Pin, gnd: Pin) {
    inst charger: Charger
    net _: vbat, charger.VBAT
    net _: gnd, charger.GND
}

design Board {
    power_section(input.VBAT, input.GND)
}
```

RFC-006 already defines `fn` as an expansion mechanism for reusable circuit
fragments. This is the smallest conceptual solution when callers only need the
expanded instances and nets. It adds no concept or grammar.

Its limitation is intentional: expansion produces hierarchical internal paths
for hygiene and designators, but the call is not itself a retained typed design
object. If Explorer, diagnostics, independent checking, or later references
must address the composed region as a stable unit, `fn` may be insufficient.

Current `fn` also has no return value: its parameters may bind existing pins or
instances, but a caller cannot bind a new source-level name to an internal pin
created by a completed sibling call. Therefore this flat shape is not generally
available today:

```cohdl
design Board {
    power()       // internally creates regulator.OUT
    controller()  // cannot name power's internal regulator.OUT here
}
```

That does not yet prove a new concept is needed. RFC-006 permits nested calls,
and existing fn parameters can carry `Pin` references and trait-bound instance
references. A parent fn can own the shared component and pass its pins into
smaller child fragments:

```cohdl
fn controller(vdd: Pin, gnd: Pin) {
    inst mcu: McuPart
    net _: vdd, mcu.VDD
    net _: gnd, mcu.GND
}

fn system_power(vbat: Pin, gnd: Pin) {
    inst regulator: RegulatorPart
    net _: vbat, regulator.IN
    net _: gnd, regulator.GND
    controller(regulator.OUT, gnd)
}

design Board {
    inst battery: BatteryPart
    system_power(battery.VBAT, battery.GND)
}
```

This is composition by explicit dataflow rather than by drawing-sheet
boundaries. Different source modules may define the child fns; `use` imports
them, and one design or orchestration fn composes them. The hierarchy is still
deterministic in compiler-generated paths, but it is not a retained public
interface object.

### Option B: first-class `subdesign` composition

Illustrative syntax only:

```cohdl
subdesign PowerSection {
    ports {
        VBAT: Pin,
        GND: Pin,
    }

    inst charger: Charger
    net VBAT: self.VBAT, charger.VBAT
    net GND: self.GND, charger.GND
}

design Board {
    subdesign power: PowerSection {
        VBAT: input.VBAT,
        GND: input.GND,
    }
}
```

A `subdesign` would be a typed composition boundary with explicit ports and a
retained hierarchical identity. It would contain real component instances but
would not itself be a part, receive a designator, or appear in a BOM. CAD tools
could choose to render a subdesign on one page, several pages, or inline; that
presentation decision would remain outside the language.

The use site deliberately says `subdesign power`, not `inst power`. `inst`
currently denotes a concrete device occurrence whose manufacturing identity is
settled through part binding and designators. Reusing it would preserve the
same category ambiguity that makes `#[virtual] inst` questionable.

This option is conceptually cleaner if retained hierarchy is a real
requirement, but it has permanent cost: a new declaration and composition
statement, port rules, name-resolution rules, diagnostics, Explorer shape, and
interactions with `fn`, generics, modules, placement, and designators.

#### Minimum viable semantic contract

If `subdesign` is selected, its first version should be deliberately narrow:

1. **Declaration and use are distinct from physical instances.** A top-level
   `subdesign Name { ... }` declares a reusable composition. A body-level
   `subdesign local: Name { ... }` creates a logical hierarchy node. Neither is
   a `Device`, `Part`, or physical `Instance`.
2. **Ports reuse existing pin semantics.** A port is a typed connection point,
   with explicit connection obligations checked like ordinary pins. Every
   required port must be connected at the use site or explicitly handled by
   whatever accepted not-connected form applies to ports.
3. **`net` remains the only connectivity mechanism.** A port connects an
   internal net to an external net; the checker merges them into one electrical
   equivalence class. `subdesign` must not introduce implicit wiring.
4. **Internal objects remain ordinary.** Every real internal `inst` still
   requires part evidence, receives a stable designator, participates in DRC,
   and appears normally in manufacturing output.
5. **The container is logical only.** The subdesign node itself has no part,
   designator, footprint, placement, BOM row, or emitted component record.
6. **Hierarchy is retained in checked IR and flattened only for manufacturing.**
   Explorer, LSP, and diagnostics can address `Board::power::charger`; emitters
   receive the contained real components and nets, not a fake `power`
   component.
7. **Paths are deterministic.** Two uses of the same subdesign produce
   distinct, stable hierarchical paths that feed RFC-005's existing
   designator allocator without changing its collision guarantees.
8. **Nesting is composable and acyclic.** A subdesign may contain another
   subdesign, but direct or indirect recursive containment is a structural
   compile error naming the full cycle.
9. **Visibility stays RFC-016's job.** A `pub subdesign` may be referenced from
   another package; non-`pub` visibility, `use`, and fully-qualified paths obey
   the existing module rules unchanged.
10. **Generic syntax is reused, not reinvented.** If parameterized subdesigns
    are admitted, they use RFC-007's existing generic/spec-bound machinery and
    substitution rules; RFC-032 adds no second generic system.
11. **Internals are encapsulated by default.** External source connects only
    to declared ports. Reaching into `power::charger.VBAT` is not permitted in
    the first version; otherwise the port boundary would not be a real
    contract.

Whole-subdesign placement and physical group transforms remain out of scope.
Authors place the contained real instances individually through the existing
layout mechanism. This keeps the first version a schematic-composition feature
rather than a new physical-layout hierarchy.

### Option C: `#[virtual] inst` prototype

```cohdl
#[virtual]
inst boundary: BoundaryDevice
```

The prototype fully checks the instance and centrally removes it before
manufacturing. It is local and inexpensive to implement, but it represents a
logical composition boundary as a special device instance. That creates a
semantic exception precisely where `subdesign` could express the distinction
directly, and its page-boundary motivation can leak presentation concepts into
the language model.

The prototype remains evidence for implementation feasibility, not an
Accepted design.

### Recommended decision rule

Use `module + fn` unless a concrete workflow requires the composition boundary
to survive expansion as a named, typed, independently addressable object. Only
that retained-hierarchy requirement justifies `subdesign`.

Before selecting `subdesign`, the motivating design must be rewritten once
using existing module-scoped fns, Pin/instance parameters, and nested
orchestration. The rewrite must record exactly which requirement, if any,
cannot be expressed. A preference for peer-shaped blocks or pages is not such a
requirement; an inability to expose a stable typed interface after reasonable
fn decomposition may be.

Presentation grouping remains tooling metadata. Explorer may map one
`subdesign` to one page, several pages, or no dedicated page; it may also group
ordinary `fn` expansions visually. None of those view choices change language
semantics.

## Type-system-first test

This is not a residual-DRC proposal. Any accepted design must be structural and
checked before artifact generation.

- `fn` uses its existing parameter typing and expansion checks.
- `subdesign` would require statically typed ports, complete internal checking,
  and ordinary checks for every contained real instance.
- `#[virtual]` uses attribute validation and instance classification, but its
  conceptual fit remains under review.

No option may weaken E801 for ordinary real instances.

## Conceptual impact

| Option | Permanent conceptual impact |
|---|---|
| `module` alone | None, but insufficient: it organizes declarations and does not compose a circuit. |
| `fn` | None; reuse the existing circuit-fragment expansion concept. |
| `subdesign` | High; add a retained hierarchical composition concept and explicit port boundary. |
| `#[virtual] inst` | Medium; add a non-manufacturing state to `Instance`. |

The canonical vocabulary must remain clear:

- `Device` describes an electrical interface.
- `Part` supplies manufacturable evidence for a device.
- `Instance` is a concrete occurrence in a design.
- `module` organizes declarations, imports, and visibility.
- `fn` expands a reusable circuit fragment.
- A possible `subdesign` would group checked circuit structure behind typed
  ports; it would not mean schematic page.

## Coherence matrix row

| Candidate | Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|---|
| `fn` | Low | Low | Med | Low | Med | Low | High |
| `subdesign` | High | High | High | High | High | Low | High |
| `#[virtual]` | Med | Med | High | Med | High | Low | High |

- **`subdesign` Concepts/Grammar — High:** acceptance requires a complete
  definition of ports, containment, references, generics, and interaction with
  existing `fn` and `design` concepts.
- **Oracle — High:** every contained or virtualized structure must be checked
  before any manufacturing projection; no option may turn errors into omitted
  output.
- **Netlist — High:** logical containers must never become fake components,
  while their contained real components and connectivity must remain exact.
- **Trust — High:** no mechanism may hide a part-bound instance or bypass E801.

## Gradeability

The decision needs conformance tests that answer the semantic question, not
only parser tests:

- the motivating composition can or cannot be expressed with existing nested
  fns, with the specific unexpressible operation recorded rather than assumed;

- typed boundary inputs and outputs resolve correctly;
- missing or wrongly typed connections fail at the boundary;
- external references to undeclared/private internals are rejected;
- internal real components still require honest part bindings;
- composition preserves connectivity and residual DRC behavior;
- no logical container becomes a designator, footprint, component, or BOM row;
- two uses of the same composition remain hygienic and deterministic;
- nested composition works and recursive containment reports the full cycle;
- `pub`, `use`, and qualified paths behave identically to other RFC-016
  declaration kinds;
- Explorer and diagnostics either retain or deliberately erase the boundary,
  matching the chosen semantics.

The current `#[virtual]` tests cover only the prototype's manufacturing-safety
properties; they do not decide whether the abstraction belongs in the language.

## AI-generatability

`fn` is already teachable and local, but does not advertise a retained module
boundary. `subdesign` would be explicit and structurally legible to an AI, at
the cost of another declaration concept. `#[virtual]` is syntactically small
but requires learning why an apparent instance is not manufactured.

The preferred design should minimize exceptions: an AI should select an
abstraction based on whether it needs reusable expansion or retained typed
hierarchy, never based on how a schematic happens to be split into pages.

## Alternatives

- Global labels or net aliases: rejected as the main solution because they do
  not create a typed composition boundary.
- Reuse `module` as the electrical hierarchy: rejected because RFC-016 defines
  modules as source namespaces. Importing code must not instantiate a circuit,
  and one module may legitimately contain several reusable functions,
  subdesigns, devices, parts, or footprints.
- A `page` declaration: rejected because it encodes presentation rather than
  circuit semantics.
- Fake parts or footprints: rejected because they make manufacturing output
  untruthful.
- Inferring non-manufacturing behavior from a missing part: rejected because
  absence of evidence is normally an E801 authoring error.
- Keeping `#[virtual]` solely because implementation exists: rejected; an RFC
  evaluates language coherence, not sunk implementation cost.
- Add general return values and `let` bindings to `fn`: deferred unless the
  existing-fn rewrite proves that exposing an internal pin is the only missing
  operation. CoHDL is currently statement-oriented; a general expression and
  return model would be a separate permanent language expansion, not a trivial
  substitute for `subdesign`.
- Add a narrow `fn` output-port mechanism: potentially smaller than
  `subdesign`, but only justified by the same rewrite evidence. It must not add
  implicit connectivity or create a second kind of net alias.

## Compatibility

No accepted language change exists yet. The implementation in PR #33 is an
experimental prototype and must not be treated as stable syntax.

Choosing `fn` may require no language compatibility story. Choosing
`subdesign` would be additive but would need a migration path from any
prototype `#[virtual]` source. Accepting `#[virtual]` would also be additive,
but only after resolving the conceptual objection recorded here.

## Tooling & operations

- `fmt`, parser recovery, `check --json`, LSP, and Explorer must understand the
  accepted abstraction consistently.
- The checked-design representation must make hierarchy retention or erasure
  explicit; tools must not guess from names or editor pages.
- Manufacturing emitters must share one IR boundary and cannot independently
  decide whether a logical container is physical.
- `design.lock` continues to store only physical child-instance designators,
  keyed by their stable subdesign-qualified paths; no lock row is created for
  the logical container.
- Source changes must remain diffable and reversible.

Implementation complexity is **medium-high** even with the narrow contract.
It touches parser/AST, declaration-kind resolution, expansion and cycle
detection, checked IR hierarchy, diagnostics, formatter, LSP, Explorer,
designator-path regressions, and conformance tests. Manufacturing emitters can
remain comparatively simple if they continue consuming one centrally flattened
manufacturing IR. Adding group placement, public internal references, or page
semantics in the same change would make the scope high-risk and is excluded.

## Teaching cost

`module + fn` has the lowest new teaching cost and preserves both concepts'
existing meanings. `subdesign` has higher initial cost but may produce a
cleaner long-term distinction between reusable expansion and retained
hierarchy. `#[virtual]` is small in syntax but introduces a special instance
category and a prohibition list.

The RFC cannot compare teaching cost honestly until the retained-hierarchy
requirement is confirmed with concrete workflows beyond page presentation.

## Failure modes

- A CAD page concept becomes a permanent language concept.
- A source module import accidentally gains circuit-instantiation semantics.
- A logical container is mistaken for a fitted component.
- A real component is hidden from the BOM or netlist.
- `fn` is selected despite tools needing a retained addressable boundary.
- `subdesign` duplicates `fn` without a crisp semantic distinction.
- Two composition mechanisms produce different checking or naming rules.
- Explorer preserves hierarchy while compiler diagnostics or emitters flatten
  it inconsistently.

## Migration path

None while Draft. If `fn` is sufficient, prototype `#[virtual]` usages should
be rewritten as ordinary typed function composition. If `subdesign` is
accepted, prototype usages should migrate to explicit subdesign declarations
and port connections. No provisional syntax should be promised stable.

## Decision

**Draft — revised 2026-09-06.** No option is Accepted and no decision record is
assigned. Review must first determine whether retained typed hierarchy is a
real semantic requirement. If it is not, use `module` for source organization
and existing `fn` for circuit composition. If it is, specify `subdesign`
completely before implementation, while keeping `module` as namespace only.
The next design gate is a concrete rewrite using nested fns; only a precisely
recorded remaining gap may justify either a narrow fn-output extension or
`subdesign`. The `#[virtual]` implementation in PR #33 remains a prototype and
does not update the Language Specification.
