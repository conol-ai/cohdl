# RFC-023: Non-circular locating holes (mount_hole shape/size)

## Problem

RFC-022 introduced `mount_hole` for mechanical locating holes, deliberately scoped to circular holes only (`diameter D`), naming non-circular shapes as an explicit, deferred gap: "if a real footprint needs a non-circular locating feature (a slot, a keyed/D-shaped hole) — extend `mount_hole`'s shape vocabulary via a scoped follow-up RFC."

That real need now exists, grounded in a real datasheet: the Kailh Choc V2 (Low Profile) switch's official "Recommended PCB Mounting Pad Dimensions" drawing (visually inspected, dimension labels confirmed: **2.00mm × 1.50mm**) specifies two **rectangular through-hole pads** as the switch's two mechanical mounting/support legs — distinct from its two circular through-hole electrical pins. This is not a hypothetical case: it is the standard footprint for a real, widely-used mechanical keyboard switch family.

Confirmed against real source (`src/ast.rs`): `MountHole` today has exactly one geometry field, `diameter: UnitValue` — there is no way to express a rectangular (or oval) locating hole. Modeling the Choc V2's mounting legs as a `pad` is wrong for the same reason RFC-022 itself established: these legs are non-electrical (mechanical-only, no net) and have no device pin number to bind to, so forcing them through `pad`'s pin-bound numbering would either be a category error or require breaking RFC-018's unconditional pad-completeness guarantee (RFC-022's own central rationale, unchanged here).

Who this is for: library authors modeling real switch/connector/mounting-hardware footprints whose mechanical legs are non-circular — starting concretely with a std-library Kailh Choc V2 footprint.

## Goals

- Extend `mount_hole` to support the same closed shape vocabulary `pad` already has (`rect`, `circle`, `oval` — RFC-018's existing `PadShape` enum, reused verbatim, not reinvented) — closing exactly the gap RFC-022 named and no more.
- Keep `mount_hole`'s core guarantees from RFC-022 completely unchanged: numbering stays disjoint from `pad`'s pin-bound numbers, never checked against the bound device's declared pins; `plating` stays the same closed two-value set (`non_plated`/`plated`); always spans `through_all`.

## Non-goals

- **Not a general 2D shape/geometry language.** The same three shapes `pad` already supports, nothing more — no arbitrary polygons, no rounded-rectangle corner-radius control, no keyed/D-shaped/slotted profiles beyond what `rect`/`oval` already express reasonably well. If a future real footprint needs a shape beyond these three, that is separately scoped future work, not silently expanded here.
- **Not board-level mounting holes** — unchanged non-goal from RFC-022; still out of scope, still a design/board-level concept closer to `board_outline` (RFC-020).
- **Not changing **`pad`**'s own shape/size grammar** — this RFC only extends `mount_hole` to reuse the existing `PadShape` enum and `pad`'s existing shape-dependent-sizing convention; `pad` itself is untouched.

## Design

```cohdl
// std/switches/kailh_choc_v2.cohdl

pub footprint KailhChocV2 {
    pad 1: Round_1_0mm_THT at (-2.75mm, 3.0mm)   // electrical pin, top
    pad 2: Round_1_0mm_THT at (2.75mm, -3.0mm)   // electrical pin, bottom

    mount_hole 1: non_plated shape: rect size: (2.0mm, 1.5mm) at (-6.75mm, 0mm)
    mount_hole 2: non_plated shape: rect size: (2.0mm, 1.5mm) at (6.75mm, 0mm)

    courtyard { shape: rect, at: (0mm, 0mm), size: (15.5mm, 15.5mm) }
    silkscreen_ref { at: (0mm, -8.5mm) }
}
```

- `mount_hole N: PLATING [shape: SHAPE] at (x, y) [diameter D | size: (w, h)]` — a `shape:` field is now accepted, one of `rect`, `circle`, `oval` (RFC-018's existing `PadShape` closed set, reused, not redefined).
- `shape:`** is optional; its absence means **`circle` — this preserves every existing `mount_hole` declaration's meaning unchanged (RFC-022's own examples, and any real footprint authored since, all continue to parse and mean exactly what they meant before this RFC).
- **The geometry field is shape-dependent, mirroring **`pad`**'s own established convention exactly**: `circle` (explicit or defaulted) takes `diameter D` (a single `Length` value, unchanged from RFC-022); `rect`/`oval` take `size: (w, h)` (two `Length` values, the same tuple shape `pad`'s own `size:` field already uses for `rect`/`oval`).
- Writing `diameter` alongside `shape: rect`/`shape: oval`, or writing `size:` alongside `shape: circle` (explicit or defaulted), is a compile error naming the mismatch — the same shape-dependent-field-consistency discipline `pad` already enforces for its own `drill:`/`plating:` pairing (RFC-018).
- `PLATING`, `at (x, y)`, the disjoint-from-`pad` numbering namespace, and the always-`through_all` layer are all unchanged from RFC-022.

## Type-system-first test

Both checks this RFC adds are structural, local to one `mount_hole` declaration, and checkable the moment it is parsed — never DRC candidates:

1. `shape:`**, when present, must be one of the closed three-value set** — identical checking shape to `pad`'s existing `shape:` field (RFC-018), just reused on a second construct.
2. **The geometry field present must match what the (explicit or defaulted) shape requires** — `diameter` for `circle`, `size:` for `rect`/`oval`, never both, never neither. This mirrors `pad`'s existing `drill:`-required-iff-`plating: plated_through_hole` consistency check exactly, applied to a different field pairing.

## Conceptual impact

Low. No new core concept, no new top-level declaration kind, no new enum — `mount_hole` gains one optional field (`shape:`) whose values are drawn from an enum (`PadShape`) that already exists and is already fully specified (RFC-018). The shape-dependent-geometry-field convention this RFC applies to `mount_hole` is likewise not new — it is `pad`'s own established pattern, reused verbatim on a second construct rather than inventing a parallel one.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Low | Low | High |

**Trust (High):** this closes a real, concrete gap RFC-022 itself flagged and disclosed honestly rather than silently working around — a real datasheet (Kailh Choc V2) needed exactly this shape and had no correct way to express it before this RFC.
**Grammar/Diagnostics (Low):** one optional field, reusing an existing enum and an existing shape-dependent-sizing pattern — the smallest possible grammar surface for the job.
**Netlist (Low):** the KiCad `.kicad_mod`/IPC-2581 emitters already know how to project `rect`/`oval` geometry for `pad` (RFC-018) — projecting the same shapes for `mount_hole` (still net-less, still `np_thru_hole`/plated-with-no-net per RFC-022) is a direct reuse of existing emitter code paths, not new geometry-handling logic.
**Compat (Low):** every existing `mount_hole` declaration (circular, using `diameter`) is unchanged in meaning — `shape:`'s absence defaults to `circle`, so this is purely additive.

## Gradeability

- `shape:` value outside `{rect, circle, oval}` — checked at declaration/parse time, the same E8xx sub-case class RFC-022 already established for `mount_hole`.
- Geometry-field/shape mismatch (`diameter` with `rect`/`oval`, `size:` with `circle`) — checked at declaration time, naming the specific mismatch, mirroring `pad`'s existing `drill:`/`plating:` diagnostic precedent (RFC-018).
- Neither check runs in residual DRC.

## AI-generatability

High. An author or model that already knows `pad`'s `shape:`/`size:` convention (RFC-018) has already learned everything this RFC adds — the same enum, the same shape-dependent-sizing rule, applied to a second, already-familiar construct (`mount_hole`, RFC-022). No new concept to memorize.

## Alternatives

- **A separate construct for rectangular/oval locating holes** (e.g. `mount_slot`) — rejected: this would duplicate nearly all of `mount_hole`'s existing behavior (disjoint numbering, plating, position, `through_all` layer) for what is really just a shape variation — the same "two constructs for one underlying relationship" smell RFC-018's own Alternatives rejected when considering separate constructs for pads vs. footprints.
- **Overload **`diameter`** to accept either a scalar or a **`(w, h)`** tuple, inferring shape implicitly** — rejected: this would make a footprint's rendered shape depend on an author's field-shape choice rather than an explicit, readable `shape:` declaration — exactly the kind of "correct by convention, not by the compiler" ambiguity the Constitution forbids elsewhere, and inconsistent with `pad`'s own explicit `shape:` field precedent.
- **A general 2D CAD/polygon authoring mechanism for locating features** — rejected as premature, unchanged from RFC-022's own reasoning: no concrete need beyond `rect`/`circle`/`oval` has been shown; reusing `pad`'s existing closed set is the right-sized slice.

## Compatibility

Purely additive. Every existing `mount_hole N: PLATING at (x, y) diameter D` declaration (RFC-022's only prior shape) is unchanged in meaning — `shape:`'s default is `circle`, so no migration is required for any existing footprint.

**Depends on**: RFC-018 (for the reused `PadShape` enum and shape-dependent-sizing convention) and RFC-022 (`mount_hole` itself) — both already Accepted.

## Tooling & operations

- `cohdl lsp` hover on a `mount_hole` line should show its resolved shape and geometry (diameter or size), the same "resolve and show" precedent already established for `pad` (RFC-018) and `mount_hole` (RFC-022).
- `cohdl fmt` needs one small update: format `shape:`/`size:` on `mount_hole` lines using the exact same spacing/comma convention already used for `pad`'s own `shape:`/`size:` fields — no new formatting category, a direct extension of an existing rule.
- Reserves new E8xx sub-cases (designators & parts, RFC-018/022's home for footprint-completeness checks): invalid `mount_hole` shape value, geometry-field/shape mismatch.
- `cohdl build`'s KiCad `.kicad_mod` emitter projects a `rect`/`oval`-shaped `mount_hole` using the same non-round `np_thru_hole` pad geometry KiCad itself supports (KiCad's `np_thru_hole` pads are not restricted to round shapes — this was already true of the real target format RFC-022 grounded itself in); the IPC-2581 emitter projects the corresponding rect/oval hole/pin geometry with no net reference, unchanged in spirit from RFC-022's circular case.

## Teaching cost

Low. An author who already knows `pad`'s `shape:`/`size:` convention has zero new concept to learn — this RFC's entire teaching cost is "the same field, on a second construct." Authors who only ever need circular locating holes experience no change at all (the default is unchanged).

## Failure modes

- **A rectangular/oval **`mount_hole`**'s dimensions are wrong but structurally valid** (a typo'd size) — this RFC's checks cannot catch this, the same disclosed, unavoidable limitation RFC-018/022 already named for pad/hole dimensions generally.
- **An author picks **`shape: rect`** when the real hardware's hole is actually a rounded-rectangle or a true slot** (rounded corners, not a sharp rectangle) — `rect`'s geometry is an approximation for such cases, the same honest limitation `pad`'s own `rect` shape already carries; not solved by this RFC, consistent with the Non-goals above.

## Migration path

No existing `mount_hole` declaration requires migration — `shape:`'s default (`circle`) preserves every prior declaration's meaning exactly. Real, non-mechanical authoring work remains separately: the std library should gain a real Kailh Choc V2 footprint (using this RFC's new `shape: rect size: (2.0mm, 1.5mm)` `mount_hole` syntax for its two mounting legs) — genuine new content to author, not a retrofit this RFC's completion bar requires.

## Decision

**Accepted — 2026-07-19.** Recorded as DR-029 (see note 7). Extends `mount_hole` (RFC-022) with an optional `shape:` field (reusing RFC-018's existing `PadShape` closed set: `rect`, `circle`, `oval`) and a shape-dependent geometry field (`diameter` for `circle`, unchanged and still the default; `size: (w, h)` for `rect`/`oval`, new) — closing the exact non-circular-locating-hole gap RFC-022 itself named and deferred. Grounded in a real datasheet (Kailh Choc V2 switch, 2.0mm × 1.5mm rectangular through-hole mounting legs). Purely additive; no existing `mount_hole` declaration changes meaning. Language Specification (note 10) updates the "Mechanical locating holes (mount_hole)" section in place.
