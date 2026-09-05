# RFC-032: Virtual connectivity instances

## Problem

Large schematics need named, typed boundaries between logical regions such as
hierarchical pages. Those boundaries must participate in the same pin,
connectivity, and residual-DRC checks as every other instance, but they are not
physical components and must never appear in manufacturing output.

Before this RFC, CoHDL forced an inconsistent choice:

- model the boundary as an ordinary instance, then fail E801 because no honest
  manufactured `part` exists for it;
- bind a fake part, polluting designators, footprints, netlists, and the BOM;
- flatten or duplicate connectivity, losing the named typed boundary; or
- add a local exception to part binding or an emitter, weakening the guarantee
  that every manufactured instance is evidence-backed.

The real trigger is a multi-page board whose page-boundary devices carry
logical connectivity without representing fitted objects. The feature is for
AI authors and human reviewers who need the source structure to match the
schematic structure, and for tool builders who need one deterministic boundary
between checked connectivity and manufacturing IR.

## Goals

- Serve the Constitution's rank-1 correctness goal and rank-2 AI-verifiability
  goal by keeping logical connectivity checked while keeping manufacturing
  artifacts truthful.
- Preserve CoHDL's rank-1 correctness rule: every real manufactured instance
  still requires an evidence-backed part binding.
- Let logical page or hierarchy boundaries remain explicit, typed, and checked.
- Define one emitter-independent removal point so KiCad, IPC-2581, EasyEDA,
  BOM, footprint, placement, and future manufacturing backends cannot drift.
- Keep the feature local and gradeable: one zero-argument attribute on one
  instance, with closed rejection rules.
- Preserve deterministic designator allocation and byte-stable manufacturing
  output.

## Non-goals

- A general simulation-only object model.
- Do-not-populate (DNP/DNI) fitted components. Those remain real manufactured
  objects with real parts and assembly intent.
- Net aliases, global-label syntax, ports, buses, or a new hierarchy system.
- Allowing authors to hide a real component from the BOM or netlist.
- Allowing placement, designator, footprint, or manufacturing-physics metadata
  on a virtual instance.
- Inferring virtuality from the absence of a part binding.

## Design

`#[virtual]` is a zero-argument attribute on `inst`:

```cohdl
pub device PageBoundary {
    pins {
        VBAT: 1 [passive],
        GND: 2 [passive],
    }
}

design Board {
    #[virtual]
    inst power_page: PageBoundary

    inst charger: ChargerPart

    net VBAT: power_page.VBAT, charger.VBAT
    net GND: power_page.GND, charger.GND
}
```

A virtual instance participates normally in:

1. parsing and name resolution;
2. device/spec/variant checking;
3. pin-obligation checking and `nc` validation;
4. net formation and net merging;
5. residual DRC; and
6. read-only checked-design tooling such as the schematic explorer.

After those checks succeed, the build pipeline removes virtual instance paths
from the manufacturing IR before designator allocation and part binding. The
same removal also removes their direct net members, `nc` entries, placements,
and manufacturing-physics references; empty nets produced solely by removal
are dropped. Every manufacturing emitter consumes only this filtered IR.

The attribute is legal only when all of the following hold:

- it has no arguments;
- the instance is not bound to a `part`;
- it has no `#[designator(...)]`;
- it has no `#[placement_hint(...)]`; and
- it carries no manufacturing-physics attribute.

These are compile-time constraints, not emitter warnings. Virtuality is always
explicit; an ordinary part-less instance remains subject to E801.

## Type-system-first test

This RFC does not propose a `rule` or residual-DRC check. Virtuality changes the
classification of one instance at the checked-connectivity/manufacturing
boundary, so it is enforced structurally during instance expansion, before
artifact generation. Misuse is local to the annotated declaration and does not
require whole-graph numeric or emergent analysis.

Pin obligations, trait satisfaction, variants, and residual DRC continue to run
unchanged on the virtual instance. The feature therefore composes with the
existing type system instead of creating a way around it.

## Conceptual impact

This adds one state to the existing Instance concept: `virtual_only`. It does
not add a top-level declaration kind, a new connectivity primitive, or a new
emitter concept.

The canonical meaning of `virtual` is deliberately narrow: a typed logical
connectivity helper that is checked but not manufactured. It does not mean
"simulation model," "optional component," "DNP," or "unchecked." Keeping that
definition closed prevents overlap with real part instances, `#[intent]`, or
future assembly-population concepts.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | High | Med | High | Low | High |

- **Oracle — High:** the verdict must include all electrical checks before the
  instance is removed. The implementation has one ordered build boundary:
  check first, filter second, allocate/bind/emit last.
- **Netlist — High:** no manufacturing backend may emit a virtual object. The
  filter is centralized before all emitters rather than duplicated per backend,
  and tests assert absence from both BOM and netlist output.
- **Trust — High:** a real part cannot be annotated virtual, and virtual cannot
  carry designator, placement, or manufacturing-physics facts. These checks
  prevent the feature from becoming an escape hatch around E801.

## Gradeability

The compiler checks the feature at the earliest relevant stages:

- parser/attribute validation rejects unknown placement and arguments using the
  existing E010 attribute diagnostic family;
- instance expansion rejects part-bound virtual instances and incompatible
  designator/placement/physics attributes, also as E010 structural misuse;
- ordinary part-less instances remain rejected by E801;
- normal pin and net checks run before filtering, proving that `#[virtual]`
  cannot suppress connectivity errors; and
- build tests prove the helper merges connectivity while creating no BOM row,
  designator, netlist component, footprint, or manufacturing placement.

The conformance suite includes both a positive end-to-end artifact test and
adversarial negative cases for hiding a real part or explicit designator.

## AI-generatability

The syntax is local and self-describing: the model writes `#[virtual]`
immediately above the one logical-only instance. No external registry flag,
name convention, or emitter-specific configuration is required.

The closed rule is easy to teach: use it only for a non-fitted typed boundary;
never combine it with a part, designator, placement, or manufacturing physics.

## Alternatives

- **Use an ordinary part-less instance and special-case E801.** Rejected: the
  absence of a binding is often an authoring error. Inferring virtuality from
  missing evidence would weaken one of CoHDL's central guarantees.
- **Create a fake part/footprint.** Rejected: it deliberately makes the BOM and
  netlist untrue and consumes a designator for something that does not exist.
- **Use `#[intent("virtual")]`.** Rejected: RFC-012 defines `#[intent]` as
  opaque metadata with zero verdict and netlist effect; changing its meaning
  would violate that contract.
- **Add a new `virtual inst` keyword.** Rejected: attributes are the established
  syntax for declaration-local behavior (`#[designator]`,
  `#[placement_hint]`, physics attributes). A new keyword adds unnecessary
  lexer and grammar surface.
- **Filter separately in every emitter.** Rejected: backend-local filtering can
  drift and allows one future emitter to leak fake manufacturing objects. The
  manufacturing IR boundary must be single and shared.
- **Flatten the hierarchy or use repeated net names.** Rejected: this loses the
  explicit typed boundary and does not solve reusable structured connectivity.

## Compatibility

Purely additive. Existing source without `#[virtual]` is parsed, checked, and
emitted identically. Existing error codes and JSON document shapes do not
change. No deprecation cycle is required.

For new source using the feature, `check`-time tools may observe the virtual
instance because it is part of the checked connectivity graph. Manufacturing
artifacts intentionally do not. That distinction is the feature's contract,
not a compatibility discrepancy.

## Tooling & operations

- `cohdl fmt` preserves the attribute in canonical attribute order.
- `cohdl check` and LSP diagnostics include virtual instances in ordinary
  resolution, pin, and DRC checking.
- The schematic explorer may display virtual instances because it projects the
  checked design, making logical page boundaries visible to reviewers.
- `cohdl build` filters them before designator allocation, part binding, and all
  manufacturing emission.
- `check --json` and `build --json` gain no new top-level fields; misuse appears
  through the existing diagnostics array.
- Removal is deterministic and reversible from source: deleting the attribute
  restores ordinary-instance behavior and therefore the E801 requirement.

## Teaching cost

Low. Authors learn one distinction: real fitted objects are ordinary instances
with part evidence; logical-only typed boundaries are explicit `#[virtual]`
instances. The prohibition list is short and matches that distinction directly.

## Failure modes

- **A real component is marked virtual to hide it from manufacturing.** Rejected
  because a part-bound instance cannot be virtual.
- **A virtual helper is given a designator or placement.** Rejected because
  those facts assert a manufactured identity/location.
- **A virtual helper carries high-current, bypass, oscillator, converter, or
  fanout manufacturing physics.** Rejected; such facts must attach to real
  manufactured instances or nets as defined by their own RFCs.
- **An author assumes virtual means unchecked.** It does not: pin obligations,
  net resolution, and residual DRC run before removal.
- **A net contains only virtual pins.** It is checked as written, then omitted
  if it becomes empty at the manufacturing boundary.
- **A future backend reads pre-filter IR.** This would violate the shared build
  pipeline contract and must be caught by the zero-virtual-artifact regression
  tests before that backend ships.

## Migration path

No existing design must change. A design that currently models a logical page
boundary using a fake part may remove that fake part binding and add
`#[virtual]` to the boundary instance. The resulting manufacturing artifacts
must be reviewed once to confirm that only the fake component/designator/BOM
row disappeared and real connectivity remained unchanged.

## Decision

**Accepted — 2026-09-05.** CoHDL adds the zero-argument `#[virtual]` instance
attribute for non-fitted typed connectivity helpers. Virtual instances are fully
checked, then centrally removed before designator allocation, part binding, and
manufacturing emission. The compiler rejects any use that could hide a real
part or attach manufacturing identity/placement/physics to the helper. Recorded
as DR-038. The Language Specification is updated in the same change.
