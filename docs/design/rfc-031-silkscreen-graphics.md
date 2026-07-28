# RFC-031: Silkscreen graphics for footprints (pin-1 markers, polarity/direction indicators)

## Problem

Grounded in a real, universal PCB manufacturing/assembly need: an assembler (human or pick-and-place operator) must be able to visually determine a component's correct orientation from the board's silkscreen alone — most commonly a **pin-1 marker** on an IC/connector (a dot, triangle, or notch near pin 1) and a **polarity/direction marker** on a two-terminal polarized part like a diode or tantalum capacitor (a cathode band, an arrow). Confirmed against real source (`src/ast.rs`): `FootprintDecl` (lines 212-226) has exactly `pads`, `mount_holes`, `courtyard: Option<Courtyard>`, `silkscreen_ref: Option<SilkscreenRef>` — no field, construct, or mechanism anywhere for drawing *any* silkscreen graphic. `SilkscreenRef` (lines 285-292) is a fixed-purpose reference-designator text placement (`x, y, size, visible`) — it positions KiCad's built-in `fp_text reference "REF**"` object, nothing else; it cannot draw a dot, triangle, line, or any other shape. `Courtyard` (lines 275-281) draws exactly one fixed shape (a margin-inflated rectangle outline on `F.CrtYd`) for exactly one purpose (placement keep-out), and is not read at all by the IPC-2581 emitter (confirmed: zero matches for `courtyard`/`silkscreen` in `src/emit/ipc2581.rs`). Grepping the whole codebase for `silk`, `graphic`, `polarity_mark`, `pin_1`/`pin1` returns zero matches beyond the two existing fixed-purpose fields above — there is no generic drawable-shape concept in the language at all today.

Today, an author (or an AI) wanting a pin-1 dot or a diode-direction arrow has no way to express it — the only escape hatch would be hand-editing the emitted `.kicad_mod` file after the fact, silently defeating CoHDL's own "the emitted artifact is a lossless projection of declared source" guarantee that every prior footprint RFC (018/022/023/025) has upheld.

Who this is for: **library authors** (need to draw a correct, reviewable pin-1/polarity mark once per footprint, the same real work they'd do today by hand in KiCad's footprint editor); **assemblers/pick-and-place operators** (the actual real-world consumer of this mark — it must render correctly in the emitted `.kicad_mod`/IPC-2581 output); **AI authors** (need a small, closed, named vocabulary for "mark pin 1" / "mark polarity" rather than having to compute raw triangle/dot coordinates by hand).

## Goals

- A way to draw real silkscreen graphics inside a `footprint` declaration — closed to a small set of well-understood drawing primitives (line, circle, arc, polygon), the same "closed set, not a general authoring language" discipline every unit/shape/rotation vocabulary in this project already follows (RFC-001's ten units, RFC-018's `PadShape`, RFC-020/025/026's closed rotation/side sets).
- A small, closed set of **semantic marker** shorthands — `pin_1_marker` and `polarity_marker` — that expand to real, checked primitive shapes, so the common cases (pin-1 dot/triangle, diode/cap polarity band or arrow) never require an author to hand-compute raw coordinates, and so the *intent* ("this marks pin 1") is legible in source, not just in the emitted geometry.
- Correct, lossless projection into both existing footprint-consuming emitters: the KiCad `.kicad_mod` silkscreen layer (`F.SilkS`/`B.SilkS`, using KiCad's own `fp_line`/`fp_circle`/`fp_arc`/`fp_poly` graphic-item forms) and IPC-2581 (which, per Non-goals below, currently emits nothing for any footprint graphics — this RFC is also the first RFC to give IPC-2581 real silkscreen output at all).

## Non-goals

- **Not a general 2D vector-graphics/CAD authoring language.** The primitive set is closed to four shapes (`line`, `circle`, `arc`, `polygon`), each with a small, fixed set of fields — the same scope discipline RFC-020's DR-026 already established when it explicitly rejected "general 2D geometry/polygon-authoring syntax" for board outlines in favor of a scoped DXF-reference mechanism. CoHDL is not becoming a drawing tool.
- **Not a third, independent shape enum.** This RFC reuses and extends RFC-018's existing `PadShape` (`Rect`, `Circle`, `Oval`, `RoundRect`) is *not* reused directly (its fields are pad-specific — width/height/roundrect_ratio tied to a `Pad`'s own geometry model) — instead this RFC introduces one new, silkscreen-specific `SilkShape` enum, but keeps it as small and closed as `PadShape`, and explicitly does not invent a third parallel shape concept beyond what real silkscreen art needs (line/circle/arc/polygon — no rect-with-fill, no text, no images).
- **Not silkscreen text beyond the existing **`silkscreen_ref`**.** Freeform silkscreen text labels (a component's value printed on silk, a library logo, etc.) are a real, plausible future need but are not addressed here — this RFC is scoped to *graphics* (marks/lines/shapes), leaving arbitrary text as tracked future work, the same "don't solve a need not yet shown" discipline this project applies recurrently (e.g. RFC-017 deferring skills, RFC-029 deferring registry hosting).
- **Not courtyard or silkscreen_ref changes.** Both existing constructs are untouched — `courtyard` still draws exactly its one fixed keep-out rectangle on `F.CrtYd`; `silkscreen_ref` still places exactly the reference-designator text object. This RFC adds a third, independent footprint-body construct alongside them, never merging or reshaping the first two (the same "keep categorically distinct things separate" discipline that kept `mount_hole` disjoint from `pad`, RFC-022).
- **Not DRC/schematic-correctness impact of any kind.** Silkscreen graphics are purely cosmetic/manufacturing-assembly aids — zero bearing on pin obligations, trait satisfaction, designator allocation, or netlist connectivity, the same zero-impact discipline every footprint-body construct (`pad`, `mount_hole`, `courtyard`, `silkscreen_ref`) already guarantees.
- **Not automatic inference of pin-1/polarity location from a device's pin roles.** An author must explicitly declare a `pin_1_marker`/`polarity_marker` (or a raw primitive) at an explicit coordinate — CoHDL does not infer "this is pin 1, therefore draw a dot here" from `Pad`/`Pin` data, the same "never silently infer a checkable/visual fact" discipline RFC-027 established for physics-constraint attributes (no auto-inference the way Quilter's own detection works).

## Design

### Silkscreen shape primitives

```cohdl
pub footprint SOT23_3 {
    pad 1: Rect_0_4x0_6mm at (-0.95mm, -1.05mm)
    pad 2: Rect_0_4x0_6mm at (0.95mm, -1.05mm)
    pad 3: Rect_0_4x0_6mm at (0mm, 1.05mm)

    silkscreen {
        line from (-1.4mm, -0.5mm) to (-1.4mm, 1.5mm) width 0.15mm
        line from (-1.4mm, 1.5mm) to (1.4mm, 1.5mm) width 0.15mm
        line from (1.4mm, 1.5mm) to (1.4mm, -0.5mm) width 0.15mm

        pin_1_marker near pad 1 shape dot
    }

    courtyard { shape: rect, at: (0mm, 0mm), size: (3.0mm, 3.4mm) }
    silkscreen_ref { at: (0mm, -2.0mm) }
}

pub footprint SOD123 {
    pad 1: Rect_0_9x1_2mm at (-1.65mm, 0mm)   // cathode
    pad 2: Rect_0_9x1_2mm at (1.65mm, 0mm)    // anode

    silkscreen {
        polarity_marker cathode_pin 1 shape band
    }

    courtyard { shape: rect, at: (0mm, 0mm), size: (4.2mm, 2.0mm) }
    silkscreen_ref { at: (0mm, -1.5mm) }
}
```

Four closed graphic primitive kinds, valid inside a new `silkscreen { ... }` footprint-body block (a sibling of `pad`, `mount_hole`, `courtyard`, `silkscreen_ref`):

- `line from (x1, y1) to (x2, y2) width W` — a straight silkscreen stroke.
- `circle at (x, y) radius R width W [fill FILL]` — `FILL` closed to `{none, solid}`, defaults `none`.
- `arc at (x, y) radius R start_angle A1 end_angle A2 width W` — angles closed to the same degree convention as `rotate` (0-360, any integer degree — not restricted to the cardinal `{0,90,180,270}` set used elsewhere, since arc endpoints are a genuinely continuous real-world need, unlike whole-component/pad orientation).
- `polygon [(x1, y1), (x2, y2), (x3, y3), ...] [fill FILL]` — three or more vertices, `FILL` closed to `{none, solid}`, defaults `solid` (the common case — a filled pin-1 triangle).

All four reuse `Length`-typed coordinates/dimensions (RFC-001's existing `Length` unit type, already used throughout `pad`/`mount_hole`/`courtyard`), a single new closed enum for fill, `SilkFill { None, Solid }`.

### Semantic marker sugar

Two closed marker shorthands, each expanding to real, checked primitive shapes at compile time — the same "sugar expands to a real, checked mechanism" precedent RFC-024 established for array range/list fan-out sugar:

- `pin_1_marker near pad N shape SHAPE` — `SHAPE` closed to `{dot, triangle}`. `N` must be an already-declared `pad` number on this same `footprint` (checked — see Gradeability). Expands to a `circle`/`polygon` primitive positioned at a small, fixed standoff (0.3mm, a conventional pin-1-marker clearance) from pad `N`'s own declared position, on the side of the pad closest to the footprint's outline. `dot` expands to a filled `circle` (radius 0.2mm); `triangle` expands to a filled `polygon` (a small equilateral triangle pointing at the pad).
- `polarity_marker cathode_pin N shape SHAPE` — `SHAPE` closed to `{band, arrow}`. `N` must be an already-declared `pad` number (the part's cathode/negative terminal). `band` expands to a `line` primitive (a short, wide stroke) drawn immediately adjacent to pad `N`, perpendicular to the line between the two terminal pads — the conventional diode cathode-band silkscreen mark. `arrow` expands to a `polygon` primitive (a small filled triangle) pointing away from pad `N` toward the anode — the conventional "current flow" diode arrow convention.

Both markers require `N` to reference a real `pad` already declared in the same `footprint` — this is what makes the marker *checked*, not merely decorative: a `pin_1_marker near pad 7` on a 3-pad footprint is a compile error naming the real, valid pad range, exactly the same discipline `mount_hole`/`pad` numbering-consistency checks already use.

## Type-system-first test

Not a `rule`/DRC proposal. The only real checks this RFC introduces are structural and local to one `footprint` declaration:

1. **Primitive grammar validity** — each of the four primitive kinds' own field shapes (line needs two points + width; circle needs center + radius + width; arc needs center + radius + two angles + width; polygon needs ≥3 vertices) — checked at parse/declaration time, the same closed-grammar discipline as `pad`'s own shape-dependent field checking (RFC-018/023).
2. **Marker-target existence** — `pin_1_marker`/`polarity_marker`'s referenced pad number `N` must exist on the same `footprint` — a local, single-declaration lookup (this `footprint`'s own `pads` list), not an emergent, whole-design, or whole-connectivity-graph fact. This is exactly the kind of check DR-006's classification logic assigns to the type system, not residual DRC — nothing here requires knowing anything beyond this one `footprint` declaration's own already-declared content.

Neither check is emergent/cross-cutting/numeric in the DRC sense — both are settled entirely from the `footprint` declaration's own text, so neither belongs in residual DRC.

## Conceptual impact

**Low-to-Med.** No new top-level declaration kind (this is a new body-level block inside the already-Accepted `footprint`, the same relationship `mount_hole`/`courtyard`/`silkscreen_ref` already have). One genuinely new small idea is added: a closed, four-kind drawable-primitive vocabulary (`SilkGraphic`), plus two marker shorthands that are pure sugar over it — comparable in size/shape to RFC-027's seven physics-constraint attributes, but scoped entirely within one existing declaration kind rather than spanning `net`/`inst` across a whole design.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | Low | Med | Med | Low | High |

**Concepts (Med):** one new closed primitive vocabulary (`SilkGraphic`, four kinds) plus two sugar markers — a real, disclosed addition, not free, but scoped tightly to one existing declaration kind and directly analogous in shape/size to precedents already Accepted (RFC-018's `PadShape`, RFC-027's seven attributes).

**Grammar (Med):** a new `silkscreen { ... }` body-level block with four primitive statement forms plus two marker forms — genuinely more surface than a single new clause (unlike RFC-025/026's one-clause additions), honestly reflecting that this is closer in scope to RFC-018's original pad/footprint split than to a small extension.

**Diagnostics (Med):** new, real checks (primitive field-shape validity per kind, marker-target pad existence) reserve new E8xx sub-cases (designators & parts, RFC-018's existing home for footprint-completeness checks) — reuses the existing block, no new block needed.

**Netlist (Med):** real, new emitter work in both `.kicad_mod` (new `fp_line`/`fp_circle`/`fp_arc`/`fp_poly` output on `F.SilkS`/`B.SilkS`) and IPC-2581 (the *first* real silkscreen output IPC-2581 has ever emitted — previously zero, per Problem) — genuinely new surface area, not a free re-projection.

**Trust (High):** the entire point — an assembler's pin-1/polarity mark is now a real, checked, reviewable fact traceable to a specific declared pad, rather than a hand-edited escape hatch outside CoHDL's own emitted-artifact guarantee.

**Compat (Low):** purely additive — `silkscreen { ... }` is a new optional block; every existing `footprint` declaration without one is completely unaffected, unchanged in every emitted byte.

## Gradeability

- Primitive field-shape validity (arity, required fields per kind) is checked at declaration, the moment the `silkscreen { ... }` block is parsed — the earliest possible stage, per this project's "prefer the earliest possible stage" discipline.
- Marker-target pad existence (`pin_1_marker near pad N` / `polarity_marker cathode_pin N`) is checked immediately against the same `footprint`'s own already-declared `pads` list — a single-declaration, non-emergent lookup, checked at the same point `pad`/`mount_hole`'s own numbering/uniqueness checks already run (`src/check/footprint.rs`).
- Not a residual-DRC candidate — see Type-system-first test above.

## AI-generatability

High for the marker sugar (the common case): "mark pin 1" / "mark the cathode" are extremely common, well-understood real-world asks a model already has strong priors for, and the marker syntax needs only a pad number and a closed shape choice — no coordinate math. Medium for raw primitives (decorative outline lines, custom silk art): a model must supply real coordinates, the same cost as `pad`'s own `at (x, y)` placements already carry — not a new kind of difficulty, just the ordinary cost of specifying geometry explicitly.

## Alternatives

- **Extend **`SilkscreenRef`** itself to carry arbitrary drawable content** — rejected: `SilkscreenRef` is a fixed-purpose reference-designator text placement (KiCad's own `fp_text reference` object) — conflating it with general graphics would be exactly the "two unrelated concepts merged because they share a layer" category error this project's discipline (RFC-022's rejection of merging `mount_hole` into `pad`) consistently avoids.
- **Reuse **`PadShape`** directly instead of a new **`SilkShape`** enum** — rejected: `PadShape`'s variants (`Rect`, `Circle`, `Oval`, `RoundRect`) and `Pad`'s own fields (`width`, `height`, `drill`, `layers`, `roundrect_ratio`) are tied to pad-specific concerns (copper/soldermask/drill geometry) that don't apply to a silkscreen stroke (which has a stroke width, not a filled area with a drill) — forcing silkscreen graphics through `Pad`'s shape model would either need a pile of pad-only fields silkscreen never uses, or a parallel meaning for the same field names. A small, new, purpose-built `SilkGraphic`/`SilkShape` pair, deliberately kept as narrow as `PadShape`, is more honest than overloading an unrelated existing type.
- **No semantic marker sugar at all — raw primitives only** — considered: would be a smaller, simpler RFC. Rejected because the actual stated need ("show pin 1 of mcu or direction of diodes") is exactly the case sugar meaningfully helps: without it, an author must hand-compute a dot/triangle/arrow's exact offset and orientation relative to a specific pad every time, a real, repetitive, error-prone task with an extremely common, well-known real-world answer shape — precisely the kind of case this project's own precedent (RFC-024's array/range sugar) says deserves sugar over a mechanism that already exists (here: the raw primitives).
- **Auto-infer pin-1/polarity marks from **`Pin`**/**`Pad`** role data (RFC-008's pin roles)** — rejected: RFC-008's pin roles (`input`/`output`/.../`power_in`/`power_out`) describe *electrical* function, not *physical position relative to a package outline* — there is no reliable, general mapping from "this pin is `power_in`" to "draw a dot here." Auto-inference would also violate this project's recurring "never silently infer a checkable/visual fact" discipline (RFC-027's explicit non-goal).
- **A single unified **`mark { ... }`** construct covering both pin-1 and polarity** — considered: pin-1 markers and polarity markers have different real shapes (`dot`/`triangle` vs. `band`/`arrow`) and different real target semantics (nearest a numbered pin vs. specifically the cathode of a two-terminal part) — keeping them as two distinctly-named marker forms is more legible and avoids a single construct silently accepting a shape value that doesn't make sense for its actual real-world convention (e.g. `mark kind: pin_1 shape: band` would be nonsensical).

## Compatibility

Purely additive. `silkscreen { ... }` is a new, optional footprint-body block; a `footprint` declaration without one is completely unaffected — unchanged in meaning and in every emitted byte. No existing `pad`, `mount_hole`, `courtyard`, or `silkscreen_ref` declaration changes meaning.

**Depends on**: RFC-018 (`pad`/`footprint`, already Accepted) — `silkscreen { ... }` is a new body-level construct inside `footprint`, the same relationship `mount_hole` (RFC-022) already has. Reuses RFC-001's `Length` unit type for all coordinates/dimensions. Directly closes a real, previously-unaddressed gap in IPC-2581's own emitter (RFC-015) — silkscreen output there is genuinely new, not a change to existing behavior.

## Tooling & operations

- **KiCad **`.kicad_mod`** emitter**: each `SilkGraphic` primitive projects directly onto KiCad's own native silkscreen graphic-item forms — `line` → `fp_line`, `circle` → `fp_circle`, `arc` → `fp_arc`, `polygon` → `fp_poly` — all on layer `F.SilkS` (or `B.SilkS`, carried per RFC-026's own `side` mechanism when the whole component is placed on the bottom — silkscreen mirrors along with everything else the same way pad geometry already does). This is a direct, lossless mapping — KiCad's own graphic-item vocabulary already matches this RFC's closed primitive set almost exactly, which is itself evidence the four-primitive scope is right-sized, not arbitrary.
- **IPC-2581 emitter**: gains its first-ever silkscreen output — each `SilkGraphic` primitive is projected into IPC-2581's own `Profile`/`Polygon`/line-segment constructs on its silkscreen layer entity, following the same "map to the format's own native equivalent, don't invent a CoHDL-specific extension" discipline every prior emitter RFC (015/018/020) already established.
- `cohdl fmt`: canonical form for `silkscreen { ... }` follows the existing block-formatting convention (RFC-009) — one statement per line, semantic markers (`pin_1_marker`/`polarity_marker`) always placed before raw primitives within the block, for consistent readability across footprints.
- `cohdl check --json`/`cohdl build --json` gain no new top-level fields — `silkscreen`'s diagnostics flow through the existing diagnostics array (RFC-010), same as every other footprint-body check.
- Reserves new E8xx sub-cases (designators & parts, RFC-018's existing home for footprint-completeness checks): malformed primitive (wrong field count/type for its kind), invalid closed-set value (`SilkFill`, marker `shape`), marker referencing a non-existent pad number.

## Teaching cost

Low for the marker sugar — "mark pin 1" and "mark the cathode" map directly onto real, everyday PCB-assembly vocabulary any author (human or AI) already understands by name. Low-to-medium for raw primitives — four small shape kinds, each with an obvious, minimal field set (start/end, center/radius, center/radius/angles, vertex list) mirroring what any PCB CAD tool's own "draw" toolbar already offers.

## Failure modes

- **An author references a pad number that doesn't exist on this footprint** (`pin_1_marker near pad 7` on a 3-pad footprint) — caught immediately at declaration, naming the footprint's actual valid pad range.
- **An author supplies a malformed primitive** (e.g. a `polygon` with fewer than three vertices, or a `circle` missing `radius`) — caught at declaration, naming the specific missing/invalid field for that primitive kind.
- **An author expects a marker's exact drawn shape/position to be independently verified against the real device's actual pin-1 location** (e.g. that `pin_1_marker near pad 1` really does correspond to the physical pin CoHDL's own `Pin`/`Pad` numbering calls pin 1) — this RFC checks only that pad `N` *exists*; it is the author's own responsibility (as with every footprint's own pad placement today) to ensure `N` is actually the electrically-correct pin. This mirrors the exact trust boundary RFC-018 already draws for pad-count/numbering consistency: CoHDL guarantees internal consistency (the referenced pad exists, matches a real device pin per RFC-002/018's existing checks) but not that the human-intended real-world meaning is correct.
- **An author expects CoHDL to infer marker placement automatically from pin roles** — it does not; every mark is an explicit, author-written statement, consistent with this RFC's own Non-goals.

## Migration path

No existing `footprint` declaration requires any change — `silkscreen { ... }` is a new, optional block; every footprint without one is unaffected. A real, optional, non-mechanical follow-up: any existing footprint in the std library or an example board can adopt a `pin_1_marker`/`polarity_marker`/raw primitive once an author chooses to add one — genuine authoring work, not required by this RFC's completion bar.

## Decision

**Accepted — 2026-07-27.** `footprint` gains a new, optional `silkscreen { ... }` body-level block (a sibling of `pad`/`mount_hole`/`courtyard`/`silkscreen_ref`), carrying a closed, four-kind drawable-primitive vocabulary (`line`, `circle`, `arc`, `polygon`, via a new `SilkGraphic`/`SilkShape`/`SilkFill` type family) plus two semantic marker shorthands (`pin_1_marker`, `polarity_marker`) that expand to real, checked primitives referencing an already-declared pad number. Zero schematic-correctness impact; purely additive; no existing footprint construct is changed. Recorded as DR-037 (see note 7). Language Specification (note 10) gains a "Silkscreen graphics for footprints" subsection under Footprints and pads.
