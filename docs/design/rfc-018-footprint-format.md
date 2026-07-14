# RFC-018: Footprint format — pad/footprint, Cadence-style pad/footprint split

## Problem

RFC-017 introduced `footprint` as a fifth top-level declaration kind, resolved through RFC-016's module system — but deliberately left its internal content completely unspecified ("symbol-resolution-complete, format-empty"), deferring the actual geometry format to a future RFC. This is that RFC.

Tony's directive: adopt Cadence Allegro's proven design — **pads and footprints are separate, independently-reusable objects**. In Cadence, a padstack (pad) is defined once (shape, size, layers, plating) and footprints reference padstacks by name, placing each at an offset, rather than inlining pad geometry inline per footprint. This is not incidental to Cadence's design — it's why large, real-world footprint libraries stay maintainable: a "0.3mm × 0.9mm rectangular SMD pad" is defined once and reused across every QFN/SOIC/0402-shaped footprint that needs it, and a plating/size fix to that one pad definition propagates everywhere it's referenced instead of requiring a find-and-fix across every footprint that happens to have copy-pasted the same numbers.

Same-day naming correction: the first draft of this RFC used invented names copad/cofp. Tony corrected this — the plain English keywords pad and footprint are used instead. Because RFC-017 already claimed footprint as a top-level declaration kind, this means footprint keeps its existing name and simply gains real, checkable content for the first time; only pad is a genuinely new keyword.

Who this is for: **library authors** (the same audience as RFC-017) who now have a concrete format to author real footprints in, and **the netlist/IPC-2581 emitters** (RFC-015) that have been carrying footprint as a bare name reference since the redesign began, waiting for exactly this RFC to give them real geometry to project.

## Goals

- Split footprint content into two independently-declared, independently-resolvable kinds: pad (a single reusable pad definition — shape, size, layer, plating) and footprint (a named collection of pad references, each placed at an offset, plus courtyard and silkscreen-reference geometry).
- Both pad and footprint are top-level declaration kinds resolved through RFC-016's existing module-path/use/pub machinery — exactly like device/trait/fn/part before them. A pad library and a footprint library can live in entirely separate packages, cross-referenced by name.
- Give RFC-017's placeholder footprint keyword real, checkable content for the first time — same declaration-kind role (what a part's footprint: field points to), same keyword name, now backed by real pad references instead of an unspecified body.
- Introduce pad as a new, standalone top-level declaration kind.
- Close RFC-017's own named gap: give `cohdl build`'s footprint-geometry projection (into `.kicad_mod`, or inline into RFC-015's IPC-2581 document) something real to project.

## Non-goals

- Not a general geometry/CAD language. pad/footprint cover exactly what's needed for netlist/BOM/IPC-2581 fidelity and basic placement — pad shape/size/layer/plating, pad placement offsets, courtyard, silkscreen reference point. No arcs, no arbitrary polygons beyond simple rect/circle/oval, no parametric expressions.
- **Not 3D models, assembly-drawing detail, or silkscreen art** — same scope boundary RFC-017's original (now-superseded) draft already drew; unchanged by this RFC.
- Not a padstack-per-layer-stack system (Cadence's padstacks can differ per board layer for vias, thermal reliefs, etc.) — pad in this RFC covers only single-layer SMD/through-hole pad shapes sufficient for the devices CoHDL currently models (no vias, no internal-layer thermal relief authoring). A richer padstack model is future work if a real need for internal-layer geometry emerges.
- **Not solving board outline / layer stackup** — still separately unaddressed, per RFC-015's own named future work; this RFC's footprint is a per-component footprint, not a board-level artifact.
- Not automatic pad/footprint generation from IPC-7351 formulas — library authors hand-author pad/footprint declarations directly, same non-goal RFC-017 already stated.

## Design

### `pad`: one reusable pad definition

```cohdl
// sparkfun/src/pads/smd.cohdl → module path sparkfun::pads::smd

pub pad Rect_0_3x0_9mm {
    shape: rect
    size: (0.3mm, 0.9mm)
    layer: top_copper
    plating: smd
}

pub pad Round_0_5mm_THT {
    shape: circle
    size: (0.5mm)
    layer: through_all
    plating: plated_through_hole
    drill: 0.3mm
}
```

- `shape`: one of `rect`, `circle`, `oval` (closed set — see Alternatives for why not a general polygon language).
- `size`: shape-dependent — `(w, h)` for `rect`/`oval`, `(d)` for `circle`.
- `layer`: one of `top_copper`, `bottom_copper`, `through_all` (closed set, extensible only via a future RFC if real multi-layer padstack needs emerge — see Non-goals).
- `plating`: `smd` (surface-mount, no drill) or `plated_through_hole` (requires `drill:`).
- `drill`: required when `plating: plated_through_hole`, omitted (compile error if present) when `plating: smd`.

### footprint: composed of pad references

```cohdl
// sparkfun/src/footprints/qfn.cohdl → module path sparkfun::footprints::qfn

use sparkfun::pads::smd::Rect_0_3x0_9mm;

pub footprint QFN10_3x3 {
    pad 1: Rect_0_3x0_9mm at (-1.5mm, 1.0mm)
    pad 2: Rect_0_3x0_9mm at (-1.5mm, 0.5mm)
    pad 3: Rect_0_3x0_9mm at (-1.5mm, 0.0mm)
    // ... one entry per pad, matching the bound device's pin count and numbering
    courtyard { shape: rect, at: (0mm, 0mm), size: (3.5mm, 3.5mm) }
    silkscreen_ref { at: (0mm, -2.2mm) }
}
```

- Each pad N: PadSymbol at (x, y) line places one instance of a pad symbol at an offset relative to the footprint's own origin. PadSymbol is resolved via RFC-016 exactly like any other cross-library reference (local name, or used, or fully qualified). This body-level pad N: ... placement statement and the top-level pad { ... } declaration share the same keyword but occupy different grammatical positions — the same pattern already used elsewhere in the language (e.g. net/nc as body-level statements distinct from other top-level declaration forms), not a new ambiguity.
- Pad numbers (`N` in `pad N: ...`) **must exactly match the bound device's declared pin numbers** (RFC-002) — this is the check RFC-017 deferred, now real and enforceable because footprint's pad list is real, structured data (not an unspecified body).
- `courtyard` and `silkscreen_ref` are unchanged in shape from RFC-017's original (superseded) illustrative sketch — they were never the contested part; only the pad-authoring mechanism needed the Cadence-style split.
- The same pad symbol can be referenced by any number of footprint declarations, in any package that can resolve it — this is the entire point of the split: a pad library maintainer fixes Rect_0_3x0_9mm's size once, and every footprint referencing it is correct without being touched.

### `part`'s `footprint:` field now points to a real footprint

```cohdl
use sparkfun::footprints::qfn::QFN10_3x3;

pub part TPS62840_QFN10: TPS62840<...> {
    primary { mfr: "Texas Instruments", mpn: "TPS62840DLCT", footprint: QFN10_3x3 }
}
```

- Unchanged in shape from RFC-017: footprint: holds a symbol reference, resolved via RFC-016. What changes is that the referenced footprint symbol now has real, checkable content, instead of an empty placeholder.

## Type-system-first test

The two checkable things this RFC introduces are both structural, local, and resolved at the point of declaration/reference — never DRC candidates:

1. pad internal consistency — drill: present iff plating: plated_through_hole — checked the moment a pad is declared, exactly the same "checkable from the declaration alone" shape RFC-001's unit-type rules already established.
2. footprint pad-count/numbering consistency against its bound device — checked at the point a part references a footprint (the exact mechanism RFC-017 deferred), against that one device's own already-declared pin list (RFC-002). Fully local to one part+device+footprint triple, never emergent-across-the-connectivity-graph.

## Conceptual impact

Medium. One new top-level declaration kind (pad), plus real content for the already-Accepted footprint declaration kind (RFC-017), both resolved via RFC-016's already-Accepted machinery — no new resolution mechanism, consistent with RFC-017's precedent. One genuinely new core concept: Pad, a reusable geometric primitive, referenced (never inlined) by footprints — this is the actual conceptual move Cadence's design demonstrates and this RFC adopts.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | Low | Med | Med | Low | High |

Trust (High): this is the mechanism that finally makes "the footprint lies" (a footprint's pad set silently not matching its bound device's pins) structurally impossible — closing the gap RFC-017 explicitly deferred, the same class of guarantee RFC-003 already established for MPN completeness.

Concepts (Med): one new core concept (Pad), but it's the smaller of two possible shapes this RFC could have taken (see Alternatives) — reusing RFC-016's resolution machinery keeps this from being two new concepts.

Grammar/Diagnostics/Netlist (Med): one new declaration-kind grammar to parse (pad, small closed vocabulary — see Design) plus real content for footprint's existing (previously empty) body, a new completeness check (pad-count/numbering match), and real geometry finally available to project into .kicad_mod/IPC-2581 (RFC-015).

Compat (Low): no keyword rename — footprint keeps its name from RFC-017, only its body gains real, checkable content. The only real compatibility surface is pad being newly reserved as a top-level declaration keyword.

Oracle (Low): no new DRC surface — both new checks are type-system, per Type-system-first test above.

## Gradeability

Both checks land at the earliest point the relevant declaration is fully visible:

- pad's drill:/plating: consistency — checked the moment the pad declaration is parsed and type-checked, at declaration time.
- footprint's pad-count/numbering-vs-device consistency — checked at the point a part declaration's footprint: field resolves to a footprint symbol (the same point MPN completeness is checked, RFC-003's precedent), at cohdl build (mirroring RFC-017's check-vs-build split exactly: cohdl check does not require footprint resolution; cohdl build does, since it needs real geometry to emit).

Diagnostics name the specific missing/extra/duplicate pad number, and for `pad`, the specific inconsistency (`drill: present with plating: smd` or `drill: missing with plating: plated_through_hole`).

## AI-generatability

Medium, same honest assessment RFC-017 gave for footprint authoring generally: writing a pad's geometry or a footprint's pad placement still requires real datasheet-derived dimensions — this RFC does not make that free. What it does improve over a hypothetical single-file-per-footprint format: once a reasonable pad library exists (a genuinely small, reusable set — most SMD passives and common IC pad shapes recur constantly across footprints), authoring a new footprint for a new device is mostly "place N existing pad symbols at N offsets," not "invent N pad shapes from scratch" — a real generatability win the split buys, consistent with why Cadence's design works well for large real-world libraries.

## Alternatives

- Keep RFC-017's original single-file inline-pad-geometry sketch (pads defined directly inside each footprint declaration, no separate pad) — this was RFC-017's own withdrawn draft, superseded before this RFC even started. Rejected again here for the same reason Cadence's own design rejects it: no reuse across footprints, no single point of correction when a pad's geometry needs fixing, real duplication risk as a library grows past a handful of footprints.
- A richer, per-layer-stack padstack model matching Cadence's full padstack feature set (vias, thermal reliefs, per-layer independent geometry) — considered, rejected as premature: CoHDL has no board-outline/stackup concept yet (RFC-015's named gap, still open), so a padstack model rich enough to need multi-layer-independent pad geometry has no board context to place itself against. pad's single-layer-plus-through-all scope is the right-sized slice until that context exists — extending later is a scoped follow-up, not a redesign.
- Merge pad and footprint into one keyword with a variants-like mechanism (RFC-008's structural-variant pattern) distinguishing "is this a pad or a footprint" — rejected: pads and footprints are genuinely different kinds of things with different reuse patterns and different consumers (a footprint references N pads; nothing references a footprint), unlike RFC-008's variants (which are alternate shapes of the same device). Two keywords is the honest reflection of two distinct concepts, not an artificial split.
- Invented, co-prefixed names (copad/cofp) — this RFC's own first draft. Tony corrected this same day: plain pad/footprint read better and need no new vocabulary — footprint in particular was already the right word (RFC-017 had already claimed it), so there was no reason to abandon it for a synonym. Superseded before this RFC's Decision below; recorded here for continuity, not re-opened.

## Compatibility

Small, disclosed change: footprint keeps its existing keyword (no rename) — RFC-017's already-Accepted declaration kind simply gains real, checkable body content for the first time. pad is a newly reserved top-level keyword. Because RFC-017 shipped with footprint's body deliberately unspecified (no real footprint content exists anywhere yet — the std library and example boards' parts still point at placeholder footprint symbols per RFC-017's own two-stage migration), there is no real content to migrate away from — every existing placeholder footprint declaration can now, for the first time, actually be filled in with real pad references.

**Depends on RFC-016 (module system) and RFC-017 (footprint-as-symbol) landing first** — this RFC is the format RFC RFC-017 explicitly deferred to, and inherits RFC-017's resolution mechanism and keyword unchanged.

## Tooling & operations

- cohdl lsp (RFC-014) hover on a footprint's pad N: PadSymbol line should show the resolved pad's shape/size/layer (the same "resolve and show, even though nothing is restated" precedent RFC-003's empty-impl-body hover already established).
- cohdl fmt (RFC-009) needs formatting rules for the new pad declaration kind's field block (parallel to spec {}'s comma-space convention) and for footprint's pad N: Symbol at (x, y) body lines (aligned columns, matching the existing pin-bus-wrapping precedent).
- Reserves new error codes in the existing E8xx block (designators & parts, RFC-005/011/017's home for part-completeness checks): pad drill/plating inconsistency, footprint pad-count/numbering mismatch against bound device, unresolved pad reference inside a footprint (though this last one is really just RFC-016's existing unresolved-name diagnostic, reused).
- `cohdl build --json`'s existing `"kicad_mod"`/`"ipc2581"` artifact-path pattern is unchanged; those emitters now have real pad/footprint geometry to project instead of nothing.

## Teaching cost

Low-Medium. The resolution half (module paths, use, pub) is zero new teaching cost — identical to every other declaration kind under RFC-016. The genuinely new surface is pad's small closed vocabulary (3 shapes, 2 platings, 3 layers) and footprint's pad-placement syntax — both deliberately small, enumerable, and shaped like existing CoHDL block syntax (spec {}, pins {}) rather than a new syntax family. Understanding why pads are separate (the Cadence-derived reuse argument) is a one-time conceptual cost for library authors, not a per-footprint cost. Reusing plain English names (pad/footprint) instead of invented abbreviations keeps this cost as low as it can be.

## Failure modes

- **A footprint's pad count/numbering doesn't match its bound device's pin count** — caught at `cohdl build`, naming the specific mismatch, exactly the failure mode RFC-017 named as its central deferred Trust argument, now actually closed.
- A pad's geometry is dimensionally wrong but structurally valid (right shape/plating, wrong size — a typo'd dimension) — this RFC's structural checks cannot catch this; explicitly the same acknowledged limitation RFC-008/017 already stated for real-world data accuracy. Because a pad may now be referenced by many footprints, a wrong pad dimension is a single point of failure across every footprint that references it — this is the flip side of the reuse benefit, named here honestly: reuse means one mistake propagates further, not that mistakes are less likely. Library review discipline matters more, not less, once pads are shared.
- A footprint references a pad that later changes shape/size (the pad library maintainer widens a pad for a new use case, unaware an existing footprint relies on the old size) — this RFC has no versioning or pinning mechanism; a footprint always resolves to whatever the referenced pad currently is. This mirrors how use-based dependencies work everywhere else in the language today (no version pinning exists yet at the module-resolution level either) — named here as a real, currently-unaddressed risk rather than assumed away by the reuse design.

## Migration path

Existing part declarations already point at RFC-017's placeholder footprint symbols (empty, unspecified content, per RFC-017's own two-stage migration). This RFC's migration is real, non-mechanical authoring work — not part of this RFC's "ship with its check" completion bar, same discipline RFC-017 already applied — build out a small starter pad library (common SMD pad shapes) and author real footprint pad placements for each existing part's already-existing placeholder footprint declaration, closing RFC-017's second migration stage for real this time. No keyword renaming is needed anywhere.

## Decision

Accepted — 2026-07-14. Recorded as DR-024 (see note 7). Introduces pad as a new, reusable, first-class top-level declaration kind; gives RFC-017's already-Accepted footprint keyword real, checkable content for the first time (no rename). Directly closes RFC-017's own named deferred gap (the footprint format) and RFC-015's own named future-work item (footprint-geometry resolution) — cohdl build can now project real geometry into .kicad_mod/IPC-2581. Language Specification (note 10) replaces RFC-017's "footprint format not yet specified" note with a full pad/footprint section. Explicitly depends on RFC-016 and RFC-017 landing first.
