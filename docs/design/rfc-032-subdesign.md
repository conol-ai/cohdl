# RFC-032: Typed logical composition (subdesign)

## Problem

CoHDL needs a way to compose a design from named, typed logical regions that connect through explicit interfaces without becoming fabricated parts. The requirement is real structural composition — reusable, parameterized, hierarchical, package-distributable circuit building blocks — not a reproduction of a paper or CAD schematic's page layout (e.g. KiCad's hierarchical-sheet-symbol pattern, which moves a drawing convention onto a computer rather than modeling circuit structure).

An initial prototype (PR #33) introduced `#[virtual]` on an `inst`: an instance fully checked (pins, nets, DRC) but stripped before designator allocation, part binding, and manufacturing emission. It proved a useful implementation fact — checked connectivity can be separated from manufacturing output — but represents a hierarchy boundary as a fake device-shaped instance, which is exactly the "paper page moved onto a computer" smell, one level indirect. It was withdrawn.

The real gap, confirmed against a concrete workload (a 3-phase BLDC controller board), is two distinct needs neither `module` (RFC-016, source namespace only) nor `fn` (RFC-006, same-package inline expansion, no retained hierarchy path, no package citizenship) closes:

1. **Retained, addressable hierarchy with physical-placement reach-in.** A board section (e.g. a phase-driver sub-circuit) needs to stay addressable as a stable path, and the outer board needs to place its real internal components — a reusable sub-circuit's own definition cannot know its board-specific coordinates in advance.
2. **Cross-board, versioned reuse.** The same sub-circuit needs to be shippable and versioned like a registry package, reused across multiple boards — `fn` has no package/registry citizenship of its own.

Who this is for: **library/board authors** composing a design from reusable, hierarchical, physically-placeable sub-circuits instead of hand-copying repeated instance/net blocks; **library publishers** distributing a sub-circuit (e.g. a DC-DC converter, a phase driver) as a versioned package; **AI authors** needing a small, name-legible rule for when to reach for `fn` versus a retained, ported, placeable, package-citizen composition unit.

## Goals

- Serve the Constitution's rank-1 correctness and rank-2 AI-verifiability goals: logical composition must remain fully compiler-checked.
- Model circuit structure, not editor pages or drawing sheets.
- Keep every fabricated component subject to normal part binding, designator, footprint, placement, and BOM rules — the container itself must never gain manufacturing identity.
- Give authors an explicit typed interface (ports) between reusable logical regions, reusing RFC-002's existing pin-obligation semantics.
- Let a reusable sub-circuit take unit-typed/trait-bound parameters (e.g. input/output voltage, output current), reusing RFC-007's existing generics — not a second parameter mechanism.
- Let a sub-circuit author its own default internal layout, and let the outer design place it as one unit or override one specific internal instance's placement — a reusable circuit that can genuinely be "dropped onto a board," not a bag of parts requiring hand placement every time it's instantiated.
- Make the composition unit a first-class citizen of RFC-016's module system and RFC-029/030's package/registry/versioning machinery, exactly like `device`/`trait`/`fn`/`part`/`footprint`.
- Make a use-site instantiation behave like `inst` — nameable, referenceable everywhere an instance reference is valid, array-typeable per RFC-024.
- Support nesting from the first version.
- Preserve deterministic, truthful manufacturing output.

## Non-goals

- Introducing `page`, `sheet`, or schematic-presentation coordinates into the core language.
- A general simulation-only object model, DNP/DNI assembly-population states.
- Allowing real parts to disappear from manufacturing artifacts.
- General/unrestricted reach-in into a subdesign's internals: `net`, `spec`, and any other electrical/data access remain strictly behind ports. **Placement is the sole admitted exception** — a categorically different, physical-layout fact, not an electrical one.
- `#[virtual] inst` in any form — rejected outright; removed, not merged.
- A second generic/parameter system — `subdesign` reuses RFC-007 verbatim.
- A second array/indexing mechanism — `subdesign` reuses RFC-024 verbatim.

## Design

### `subdesign` — a fifth top-level declaration kind

`subdesign` is a new declaration kind, a peer of `device`/`trait`/`fn`/`part`/`footprint`, resolved through RFC-016's existing module-path/`use`/`pub` machinery unchanged — no new resolution mechanism. It declares a retained, typed, hierarchical composition boundary: explicit ports, real internal instances, its own default internal layout, and (optionally) generic parameters.

```cohdl
pub subdesign DcDcConverter<Vin: Voltage, Vout: Voltage, Iout: Current> {
    ports {
        required VIN: Pin,
        required GND: Pin,
        required VOUT: Pin,
    }

    inst reg: BuckRegulator<Vin, Vout, Iout>
    inst c_in: MLCC<10uF, Vin>
    inst c_out: MLCC<22uF, Vout>

    net _: VIN, reg.VIN, c_in.A
    net _: GND, reg.GND, c_in.B, c_out.B
    net _: VOUT, reg.VOUT, c_out.A

    layout {
        place reg at (0mm, 0mm)
        place c_in at (-3mm, 2mm)
        place c_out at (3mm, 2mm)
    }
}

pub subdesign PhaseDriver {
    ports {
        required PWM_HI: Pin, required PWM_LO: Pin,
        required VBUS: Pin, required GND: Pin, required PHASE_OUT: Pin,
    }

    inst hs_fet: Mosfet_NChannel
    inst ls_fet: Mosfet_NChannel
    inst gate_driver: HalfBridgeGateDriver

    net _: PWM_HI, gate_driver.HIN
    net _: PWM_LO, gate_driver.LIN
    net _: VBUS, hs_fet.DRAIN
    net PHASE: hs_fet.SOURCE, ls_fet.DRAIN, PHASE_OUT
    net _: GND, ls_fet.SOURCE

    layout {
        place hs_fet at (0mm, 0mm)
        place ls_fet at (0mm, 3mm)
        place gate_driver at (-4mm, 1.5mm)
    }
}
```

### Use sites behave like `inst`

```cohdl
design BldcController {
    subdesign vreg: DcDcConverter<24V, 5V, 2A> {
        VIN: input.VBAT, GND: gnd, VOUT: v5_rail,
    }

    subdesign phases: [PhaseDriver; 3]
    net _: mcu.PWM1H, phases[0].PWM_HI
    net _: mcu.PWM1L, phases[0].PWM_LO
    net _: mcu.PWM2H, phases[1].PWM_HI
    net _: mcu.PWM2L, phases[1].PWM_LO
    net _: mcu.PWM3H, phases[2].PWM_HI
    net _: mcu.PWM3L, phases[2].PWM_LO
    net _: vbus_rail, phases[0].VBUS, phases[1].VBUS, phases[2].VBUS
    net _: gnd, phases[0].GND, phases[1].GND, phases[2].GND
    net _: motor.PHASE_A, phases[0].PHASE_OUT
    net _: motor.PHASE_B, phases[1].PHASE_OUT
    net _: motor.PHASE_C, phases[2].PHASE_OUT

    layout {
        place vreg at (0mm, 0mm)
        place phases[0] at (10mm, 5mm)
        place phases[1] at (20mm, 5mm)
        place phases[2] at (30mm, 5mm)

        // Override: phase B's low-side FET needs extra thermal clearance on
        // this specific board — explicit beats the subdesign's own default.
        place phases[1].ls_fet at (22mm, 9mm)
    }
}
```

A `subdesign local: Name { PORT: value, ... }` (or `subdesign local: [Name; N]`, array-typed) use site creates one retained hierarchy node with a stable path (`BldcController::vreg`, `BldcController::phases_1`). Two uses of the same `subdesign` type are two distinct, independently-checked, independently-designatored nodes — the same guarantee RFC-006 already gives two separate `fn` calls.

### Cross-package citizenship

A package exporting `pub subdesign PhaseDriver` is depended upon, versioned, locked, and hash-verified exactly like any other RFC-029/030 dependency — no new distribution mechanism:

```toml
[dependencies]
"@acme/motor-drivers" = "2.1.0"
```

```cohdl
use motor_drivers::PhaseDriver;
```

## Rules

1. **Declaration/use are distinct from physical instances.** A `subdesign` is never a `Device`, `Part`, or physical `Instance`.
2. **Ports reuse RFC-002's existing pin-obligation semantics** (`required`/`optional`), checked exhaustively at the use site.
3. `net`** remains the only electrical connectivity mechanism.** A port merges an internal net with an external net into one equivalence class. `subdesign` introduces no implicit wiring.
4. **Internal objects remain ordinary.** Every real internal `inst` requires part evidence (E801 unweakened), receives a stable designator (RFC-005), participates in residual DRC, and appears normally in manufacturing output.
5. **The container is logical only.** A `subdesign` node has no part, designator, footprint, or BOM row. Manufacturing emitters receive the contained real components and nets, flattened, never a fake `subdesign` component. `design.lock` stores only physical child-instance designators, keyed by their subdesign-qualified paths; no lock row exists for the container.
6. **Hierarchy is retained in checked IR, flattened only for manufacturing.** Explorer, LSP, and diagnostics address `Board::power::charger` directly.
7. **Paths are deterministic.** Two uses of the same subdesign produce distinct, stable hierarchical paths that feed RFC-005's existing designator allocator without changing its collision guarantees.
8. **A **`subdesign`** declares its own internal **`layout { ... }`** block**, giving every internal instance a default relative position/rotation/side. **The outer design may place the whole subdesign instance as one unit** (`place phase_a at (x, y) [rotate ANGLE] [side SIDE]`) — this translates the subdesign's own internal relative layout onto the board at that anchor, rotating/mirroring every internal relative coordinate about the anchor exactly as RFC-025/026 already rotate a pad's geometry / mirror a back-side footprint's pads — the identical geometric operation, reused, not reinvented. **The outer design may also override one specific internal instance's placement** (`place phase_a.hs_fet at (...)`) — explicit beats the subdesign's own default, for that one instantiation only, never touching the subdesign's own definition or any other instantiation of it.
9. **Placement is the sole internals exception.** `place`/`rotate`/`side` may target a real internal instance through a dotted path walking into one or more nested `subdesign` instances. `net`, `spec`, and any other internal reference remain rejected — the port boundary is the only electrical/data contract. `fn`-internal instances remain unreachable by `place` (unchanged from RFC-020) — only `subdesign` retains a stable, checked path for `place` to resolve against.
10. **Nesting is composable and acyclic.** A `subdesign` may contain another `subdesign`, including within its own internal `layout {}` block (the same composability rule applies uniformly, with no special case for the top level). Direct or indirect recursive containment is a structural compile error naming the full cycle, mirroring RFC-006's cyclic-`fn`-call diagnostic discipline exactly.
11. **Visibility is RFC-016's job, unchanged.** A `pub subdesign` may be referenced from another package; non-`pub` visibility, `use`, and fully-qualified paths obey the existing module rules unchanged.
12. **Generic parameters reuse RFC-007's existing machinery verbatim** — unit-typed (`Vin: Voltage`) or trait-bound, with the same visible-default rules. RFC-032 adds no second generic/parameter system.
13. **Use-site instantiations behave like **`inst`: nameable, referenceable in every position an ordinary instance reference is valid (net members, `place`, fn-call arguments), and array-typeable (`subdesign phases: [PhaseDriver; 3]`, `phases[i]`) — reusing RFC-024's existing array/indexing mechanism verbatim. Confirmed against real source (`src/check/expand.rs`'s `Scope.arrays: BTreeMap<String, (i64, Span)>`): the array table is keyed by plain name → length, structurally independent of `Device`/`Part` typing, so a `subdesign` array is a clean reuse, not a new mechanism.

## Type-system-first test

Not a residual-DRC proposal. `subdesign` requires statically typed ports, complete internal checking, and ordinary checks for every contained real instance — all structural, checked at the earliest possible stage:

- Port type/obligation mismatches are checked at the use site, the same discipline RFC-002 already applies to device pins.
- Generic-parameter substitution is checked identically to RFC-007's existing device/fn generic substitution.
- Recursive containment is checked at declaration, the same discipline RFC-006 already applies to cyclic `fn` calls.
- A placement-reach-in path is resolved and checked the same way an ordinary top-level `place` target already is (RFC-020), just walking a longer, `subdesign`-qualified path.

No rule weakens E801 for ordinary real instances.

## Conceptual impact

**High, but justified by two independently-real, concretely-demonstrated needs** (retained placeable hierarchy; cross-package versioned reuse), not speculative generality. The canonical vocabulary:

- `Device` describes an electrical interface.
- `Part` supplies manufacturable evidence for a device.
- `Instance` is a concrete occurrence in a design.
- `module` organizes declarations, imports, and visibility — unchanged, never the composition boundary.
- `fn` expands a reusable circuit fragment inline, hygienically, with no retained path, no ports, no package citizenship, no placement reach-in — still the right tool for one-off, same-package repetition needing none of those.
- `subdesign` groups checked circuit structure behind typed ports, retains a stable hierarchical path, is a first-class package/registry citizen, carries its own default internal layout, and admits one narrow, principled placement reach-in. It never means schematic page.

An author (or AI) picks `fn` versus `subdesign` based on whether reuse crosses a package boundary, needs a retained placeable path, or needs its own shippable default layout — never based on how a schematic happens to be drawn.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| High | High | High | High | High | Low | High |

- **Concepts/Grammar (High):** a full new declaration kind — ports, containment, generic parameters, internal layout, the placement-reach-in path grammar, package resolution, and array-typed use sites all require complete specification, delivered above by direct reuse of RFC-002/006/007/016/020/024/025/026/029/030 rather than new mechanisms.
- **Oracle/Diagnostics (High):** every contained structure and every placement-reach-in path segment must be checked before any manufacturing projection; a `subdesign` never turns a checking gap into omitted output.
- **Netlist (High):** logical containers must never become fake components; contained real components/connectivity/placement must remain exact, including through the new placement-reach-in path and the whole-unit layout transform.
- **Compat (Low):** purely additive — no existing `.cohdl` source changes meaning; the withdrawn `#[virtual]` prototype had no Accepted status to migrate away from.
- **Trust (High):** no mechanism hides a part-bound instance or bypasses E801; placement reach-in is the sole, narrow, explicitly-scoped internals exception, never a general escape hatch.

## Gradeability

- Typed port inputs/outputs resolve correctly; missing or wrongly typed connections fail at the port boundary.
- External references to undeclared/private internals (net/spec) are rejected; only a `place` target path may reach a real internal instance.
- Internal real components still require honest part bindings (E801 unweakened).
- Composition preserves connectivity and residual DRC behavior.
- No logical container becomes a designator, footprint, component, or BOM row.
- Two uses of the same `subdesign` remain hygienic and deterministic (distinct stable paths), including array-typed uses.
- Nested composition works; recursive containment reports the full cycle.
- `pub`, `use`, qualified paths, and cross-package dependency resolution (RFC-029/030) behave identically to other RFC-016 declaration kinds.
- Generic-parameter substitution behaves identically to RFC-007's existing checks.
- A placement-reach-in path that fails to resolve at any segment names the exact failing segment.
- A whole-unit `place`/`rotate`/`side` correctly transforms every internal default position, and an explicit per-instance override correctly takes precedence over that default, for exactly that one instantiation.
- Explorer/LSP/diagnostics retain the hierarchy boundary; manufacturing emitters flatten it — both consistently, from one shared IR.

## AI-generatability

High. An author (human or AI) picks `fn` for same-package inline expansion needing no reach-in/package-citizenship/own-layout, and `subdesign` when reuse crosses a package boundary, needs a retained placeable path, or ships with its own default layout — three structurally distinct, name-legible triggers, never a judgment call about schematic page layout. Parameters, array-typed use sites, and placement all reuse mechanisms (RFC-007, RFC-024, RFC-020/025/026) an author already knows from `device`/`inst`, minimizing new vocabulary.

## Alternatives

- `#[virtual] inst`: rejected outright. It represents a logical composition boundary as a special device-shaped instance, creating a semantic exception precisely where `subdesign` expresses the distinction directly, and its page-boundary motivation risked leaking presentation concepts into the language model. The PR #33 prototype is removed, not merged.
- `module + fn`** sufficiency (no new concept)**: considered and initially recommended before a concrete workload was examined. Rejected once the BLDC example demonstrated two real, distinct needs (placement reach-in; cross-package versioned reuse) that `fn` genuinely cannot serve — `fn` has no retained path for `place` to resolve against and no package/registry citizenship of its own.
- **Full encapsulation, no reach-in of any kind** (an earlier draft of this RFC): rejected — it directly blocks the BLDC board's real placement requirement, which is a physical-layout concern categorically distinct from the electrical connectivity ports already correctly gate.
- **General/unrestricted reach-in** (arbitrary `net`/`spec`/attribute access): rejected — no concrete need beyond placement was shown; opening `net`/`spec` access would make the port boundary a non-contract, reopening exactly the risk encapsulation-by-default exists to prevent.
- **A new, second parameter mechanism for **`subdesign` distinct from RFC-007's generics: rejected — the values in question (voltage/current) are already exactly RFC-001 unit-typed values RFC-007's generics already carry; a second mechanism would repeat DR-016's already-corrected mistake (two independent trait-bound-checking paths needing later unification).
- **Deferring array-typed **`subdesign`** use sites**: considered, then rejected once confirmed against real source that RFC-024's array table (`Scope.arrays`) is already structurally independent of `Device`/`Part` typing — a clean, low-cost reuse, not a new mechanism, so deferral would be withholding a free capability without justification.
- **No whole-subdesign default layout** (place every internal instance individually, every instantiation): rejected — this would make a shipped/versioned sub-circuit package require every consumer to hand-place every internal component from scratch, undermining the cross-package-reuse goal this RFC exists to serve.

## Compatibility

Purely additive. No existing `.cohdl` source, error code, designator, or netlist byte changes meaning. The `#[virtual]` prototype from PR #33 has no Accepted status and is removed, not migrated — no real source ever depended on it as stable syntax.

**Depends on**: RFC-002 (pin obligations, reused for ports), RFC-005 (designator allocator, paths feed it unchanged), RFC-006 (`fn`, kept as the distinct same-package/no-reach-in/no-package-citizenship composition tool, and its cyclic-call diagnostic discipline reused for recursive-containment checking), RFC-007 (generics, reused verbatim for subdesign parameters), RFC-016 (module system, `subdesign` is a fifth resolvable declaration kind), RFC-020/025/026 (`place`/`rotate`/`side`, extended with the dotted `subdesign`-reaching path and whole-unit layout transform, reusing the identical rotation/mirroring math), RFC-024 (array-typed instances, reused verbatim for array-typed subdesign use sites), RFC-029/030 (package/registry, `subdesign` becomes a first-class dependency-resolvable citizen).

## Tooling & operations

- `fmt`, parser recovery, `check --json`, LSP, and Explorer understand `subdesign` consistently: hierarchy is retained end-to-end in the checked IR; only manufacturing emitters flatten it.
- `cohdl fmt`'s canonical form for `subdesign` follows the existing block-formatting convention (RFC-009): `ports { ... }` first, then generic parameter list (if any) per RFC-007's existing convention, then body statements in declaration order, then the internal `layout { ... }` block last.
- Reserves a new error-code block for `subdesign`-specific diagnostics: missing/extra port connection, port type mismatch, generic-parameter substitution failure (reusing RFC-007's existing diagnostic shape), recursive containment, placement-reach-in path segment not found, placement-reach-in path attempting to cross a net/spec boundary, array-typed use-site index errors (reusing RFC-024's existing diagnostic shape) — a new kind of mistake per RFC-011's organizing principle, distinct from ordinary `fn`/`place` diagnostics.
- `design.lock` gains no new top-level table; child-instance rows are keyed by their existing subdesign-qualified paths, unchanged in shape from ordinary `fn`-expansion-derived paths.

## Teaching cost

Medium. `subdesign` is a genuinely new declaration kind, but every one of its mechanisms (ports as pins, generics as RFC-007, array use sites as RFC-024, placement as RFC-020/025/026) is a direct, name-legible reuse of something an author already knows — the only new idea to teach is the placement-reach-in exception and the default-layout-with-override precedence rule, both concrete and small.

## Failure modes

- An author reaches for `subdesign` purely to make a design look tidier in an editor, with no real reuse-across-packages or placement-reach-in need — `fn` remains the right tool in that case; this RFC does not forbid `subdesign` for pure organization, but authors should prefer `fn` when no retained-path/reach-in/package-citizenship/own-layout benefit exists.
- An author attempts to reach into a `subdesign`'s internal `net`/`spec` — rejected, naming the port boundary as the correct contact surface.
- A placement-reach-in path names a `fn`-expanded instance instead of a `subdesign`-internal one — rejected, since `fn` retains no stable path; the diagnostic names this distinction directly.
- Recursive `subdesign` containment — rejected, naming the full cycle.
- An author expects a per-instance placement override to affect the subdesign's own definition or other instantiations — it does not; overrides are scoped to exactly one instantiation.
- A real component is hidden from the BOM or netlist — never possible by construction; the container itself carries no manufacturing identity, but every contained real `inst` is unconditionally emitted.

## Migration path

The `#[virtual]` prototype from PR #33 is removed, not migrated — it never reached Accepted status, so no real source depends on it as stable syntax. No existing `.cohdl` source requires any change; `subdesign` is new, optional, additive syntax.

## Decision

**Accepted — 2026-09-08.** `subdesign` is a new, first-class declaration kind: a retained, typed, hierarchical composition boundary with explicit ports, generic parameters (reusing RFC-007), its own default internal layout, and array-typeable `inst`-like use sites (reusing RFC-024) — resolved through RFC-016's existing module system and RFC-029/030's existing package/registry/versioning machinery exactly like `device`/`trait`/`fn`/`part`/`footprint`. It supports nesting from day one, and closes RFC-020/DR-026's long-deferred gap by admitting one narrow, principled exception: an outer design's `place` (with `rotate`/`side`) may target a real internal instance reached through a `subdesign` instance's stable path, either transforming the subdesign's own default layout as one unit or overriding one specific internal instance for one instantiation — never extended to `net`/`spec`/arbitrary internals. Every contained real instance remains fully part-bound, designatored, and DRC'd; the container itself carries no manufacturing identity of any kind. `#[virtual] inst` (PR #33's prototype) is rejected outright and removed. Recorded as DR-038 (see note 7). Language Specification (note 10) gains a "Typed logical composition (subdesign)" section.
