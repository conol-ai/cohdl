# RFC-025: Rotated pad placements in footprints

## Problem

Grounded directly in a real, common footprint shape: QFN and LQFP packages place the same rectangular (or oval) pad shape on all four sides of the package, but pads on the top/bottom edges are rotated 90° relative to pads on the left/right edges — e.g. a real KiCad `QFN-20-1EP_4x4mm_P0.5mm_EP2.5x2.5mm` footprint places pads 1/3/5 (left side) as `0.825mm × 0.25mm`, and pads 6/8/10 (top side) as the same physical pad shape rotated 90°, i.e. `0.25mm × 0.825mm` — the width and height swapped.

Confirmed against real source (`src/ast.rs`): `PadPlace` — the `pad N: PadSymbol at (x, y)` body-level statement inside a `footprint` declaration (RFC-018) — has exactly four fields: `number`, `pad`, `x`, `y`. There is no rotation field anywhere in a pad placement. `place` (RFC-020, board-level component placement) does have a `rotate` field, closed to `{0, 90, 180, 270}` — but that construct rotates a whole already-placed component on the board; it has no bearing on a pad's own orientation *inside* its footprint's local coordinate system, which is what a QFN/LQFP footprint author actually needs.

Confirmed against the real KiCad `.kicad_mod` emitter (`src/emit/kicad_mod.rs`): today it always emits a 2-argument `(at x y)` clause for every pad — never a 3-argument `(at x y angle)`, even though KiCad's own pad format supports a rotation angle in that third position (confirmed via KiCad's real `np_thru_hole`/pad model and forum documentation of the pad-properties angle field).

## Goals

- Let a footprint author express "this pad, at this position, is the same pad shape as its neighbors but rotated" — without needing to define a second, differently-sized `pad` symbol just to get the rotated appearance.
- Reuse RFC-018's existing pad-reuse discipline: one `pad` definition (e.g. `Rect_0_825x0_25mm`), placed multiple times, at multiple positions, in multiple orientations — never forcing a duplicate pad definition (`Rect_0_25x0_825mm`) whose only reason to exist is "the same physical pad, rotated."
- Serve both real cases correctly: `rect`/`oval` pads (where 90°/180°/270° rotation visibly changes the pad's footprint — width and height swap for 90°/270°) and `circle` pads (where rotation is a structural no-op, since a circle has no orientation).

## Non-goals

- **Not arbitrary-angle rotation.** Real KiCad footprints do support a genuinely free angle (e.g. a 22°-angled pad on some connector footprints) — but no concrete CoHDL need for anything beyond cardinal rotation has been shown. This RFC reuses RFC-020's exact closed `{0, 90, 180, 270}` set, the same narrow-first discipline this project has applied everywhere else (RFC-001's closed unit set, RFC-018's rect/circle/oval-only shapes, RFC-020's own closed rotation set for component placement).
- **Not board-level component rotation** — that is already `place ... rotate ANGLE` (RFC-020), unchanged, a separate mechanism at a separate level (board coordinates, not footprint-local coordinates). This RFC does not touch `place` at all.
- **Not automatic derivation of rotation from pad position** (e.g. "infer rotation because this pad is on the top edge"). CoHDL has no concept of "which edge of the footprint a pad is on" and inventing one to auto-derive rotation would be real, unjustified new machinery for a fact an author can simply state directly, the same "explicit over inferred" discipline RFC-018 already applies to `shape:`/RFC-023 applies to `mount_hole`'s `shape:`.
- **Not rotating **`courtyard`**/**`silkscreen_ref` — those are footprint-level, not per-pad, and no concrete need has been shown for rotating them independently of the whole footprint (which isn't itself rotatable — a footprint's own drawn orientation is fixed; only whole placed *components* rotate, via `place`).

## Design

```cohdl
pub pad Rect_0_825x0_25mm {
    shape: rect
    size: (0.825mm, 0.25mm)
    layer: top_copper
    plating: smd
}

pub footprint QFN20_4x4 {
    // Left side — pad's natural (unrotated) orientation
    pad 1: Rect_0_825x0_25mm at (-1.9375mm, -1.0mm)
    pad 3: Rect_0_825x0_25mm at (-1.9375mm, 0mm)
    pad 5: Rect_0_825x0_25mm at (-1.9375mm, 1.0mm)

    // Top side — same pad symbol, rotated 90°
    pad 6:  Rect_0_825x0_25mm at (-1.0mm, 1.9375mm) rotate 90
    pad 8:  Rect_0_825x0_25mm at (0mm, 1.9375mm) rotate 90
    pad 10: Rect_0_825x0_25mm at (1.0mm, 1.9375mm) rotate 90

    // Right side — rotated 180° (mirrors the left side's orientation)
    pad 11: Rect_0_825x0_25mm at (1.9375mm, 1.0mm) rotate 180
    pad 13: Rect_0_825x0_25mm at (1.9375mm, 0mm) rotate 180

    // Bottom side — rotated 270°
    pad 16: Rect_0_825x0_25mm at (1.0mm, -1.9375mm) rotate 270
    pad 18: Rect_0_825x0_25mm at (0mm, -1.9375mm) rotate 270

    courtyard { shape: rect, at: (0mm, 0mm), size: (4.5mm, 4.5mm) }
    silkscreen_ref { at: (0mm, -2.5mm) }
}
```

- `pad N: PadSymbol at (x, y) [rotate ANGLE]` — `rotate` is a new, optional clause on the existing `PadPlace` body-level statement. `ANGLE` is one of the closed set `{0, 90, 180, 270}` — reusing RFC-020's exact set and its own diagnostic (no free-form angle). Omitted `rotate` defaults to `0` (unrotated) — every existing `pad N: ... at (x, y)` statement, written before this RFC, is unchanged in meaning.
- The referenced `pad` symbol's own `shape`/`size` are never duplicated or restated — `rotate` is purely a placement-time fact, layered on top of the pad symbol's own already-declared geometry, the same relationship RFC-020's `place ... rotate` has to a component's already-declared footprint.
- For `rect`/`oval` pads, a 90°/270° rotation is defined as swapping the pad's own declared `(w, h)` for rendering purposes only — the underlying `pad` symbol's `size:` field is never mutated or reinterpreted; this is purely how the rotation is realized downstream (see Tooling & operations). A 180° rotation on a `rect`/`oval` pad has no visible geometric effect (a rectangle rotated 180° occupies the same footprint) but is still valid — some authors may prefer stating the true physical orientation explicitly for documentation/consistency, and CoHDL does not second-guess that.
- For a `circle` pad, any `rotate` value is accepted and is a structural no-op — a circle has no orientation, so `rotate 90` on a circular pad changes nothing about its rendered geometry. This is intentional, not an inconsistency: forbidding rotation on circular pads would be a special case with no benefit (an author copy-pasting a rotated pad-placement pattern across mixed pad shapes shouldn't need to special-case circles).

## Type-system-first test

This is not a `rule`/DRC proposal — it is a small, closed grammar extension checked structurally, at the point a `pad N: ... rotate ANGLE` statement is declared:

1. **Closed-set membership** — `ANGLE` must be one of `{0, 90, 180, 270}`; any other integer literal is a compile error listing the valid set, identical in shape to RFC-020's own `place ... rotate` check.
2. **No new emergent/cross-cutting check** — a pad's rotation is a fact entirely local to its own `PadPlace` statement; nothing about it depends on any other pad, footprint, or the whole connectivity graph. Never a residual-DRC candidate.

## Conceptual impact

Low. No new core concept, no new top-level declaration kind — `rotate` is one new optional field on an existing body-level statement (`PadPlace`, RFC-018), directly mirroring a mechanism the language already has in a structurally identical position (`place ... rotate`, RFC-020). An author who already understands component-level rotation needs zero new mental model to understand pad-level rotation — same closed set, same "declared fact, not computed," same optionality/default.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Med | Low | High |

**Trust (High):** `rotate`'s default (`0`, unrotated) preserves every existing `pad N: ... at (x, y)` statement's meaning exactly — no silent reinterpretation of any already-written footprint.
**Netlist (Med):** real, new emitter work is required (KiCad `.kicad_mod`'s `(at x y angle)` third argument; IPC-2581's per-pad rotation/transform representation) — this is genuine new surface, not a free re-projection of existing data, so it's rated above the other Low cells rather than under-claimed.
**Grammar (Low):** one new optional clause on one existing statement, closed-set-checked — no new statement kind, no new keyword beyond the already-reserved `rotate` (RFC-020 already reserved it at the `place`-statement level; this reuses the identical keyword in a new but structurally analogous position).
**Compat (Low):** purely additive; every pre-existing footprint declaration is unaffected.

## Gradeability

- `ANGLE`'s closed-set membership is checked at the point `pad N: ... rotate ANGLE` is declared — the same discipline, same error shape, as RFC-020's `place ... rotate ANGLE` check.
- No other new check is introduced. `rotate`'s presence/value has no bearing on RFC-018's existing pad-count/pin-number-matches-device-pins check, which is unaffected.
- Not a residual-DRC candidate — see Type-system-first test above.

## AI-generatability

High. A model that has already learned `place ... rotate ANGLE` (RFC-020) needs only to recognize the identical `rotate ANGLE` clause reused at a different (but structurally parallel) statement — no new vocabulary, no new closed set to memorize, and the QFN/LQFP per-side rotation pattern (0°/90°/180°/270° cycling around four sides) is a common, well-known real-world convention a model is very likely to already associate with this exact package family.

## Alternatives

- **Swap **`size: (w, h)`** to **`(h, w)`** per rotated pad instead of an explicit **`rotate`** field** — this is what real KiCad library footprints actually do today (confirmed: the real `QFN-20-1EP_4x4mm...kicad_mod` file swaps pad width/height per side, never emitting an explicit angle). Rejected for CoHDL specifically: it would force an author to define two separate `pad` symbols (`Rect_0_825x0_25mm` and `Rect_0_25x0_825mm`) for what is conceptually one physical pad shape used at two different orientations — directly undermining RFC-018's own "define once, place by reference many times" reuse discipline, and silently discarding the authorial fact "these are the same pad, just rotated" that a future consumer (a 3D-model viewer, a library-consistency linter) might reasonably want to know. An explicit `rotate` field keeps one `pad` definition and states the real fact directly.
- **A general 2D transform (arbitrary angle + mirroring) on pad placement** — rejected per this project's own recurring narrow-scope-first discipline (RFC-007's rejected const-generics, RFC-018's rect/circle/oval-only shapes, RFC-020's own closed rotation set): no concrete CoHDL need for non-cardinal angles or pad mirroring has been shown; QFN/LQFP's real, common pattern is fully served by the closed four-value set.
- **Infer rotation automatically from which edge of the footprint a pad sits on** — rejected: CoHDL has no "footprint edge" concept, and inventing one purely to avoid one explicit field per pad is real new machinery for no corresponding benefit over an author simply stating the rotation directly (the same "explicit over inferred" call already made for `shape:` in RFC-018/023).
- **Rotate the referenced **`pad`** symbol's own declared **`size:`** in place, per placement** — rejected: this would mean the same `pad` symbol's meaning changes depending on which `PadPlace` references it, breaking the "a `pad` symbol has one fixed geometry, referenced by many placements" invariant RFC-018 established. Keeping `rotate` a placement-local fact (not mutating the referenced symbol) preserves that invariant exactly.

## Compatibility

Purely additive. `rotate` is a new optional clause on `PadPlace`; every existing `pad N: PadSymbol at (x, y)` statement (no `rotate`) is completely unaffected, unchanged in meaning and in every emitted byte.

**Depends on**: RFC-018 (pad/footprint, already Accepted) — `rotate` extends `PadPlace`, RFC-018's own construct. Reuses RFC-020's closed rotation-angle set and keyword by direct precedent, not by a new dependency — RFC-020 is unmodified by this RFC.

## Tooling & operations

- **KiCad **`.kicad_mod`** emitter**: when `rotate` is non-zero, emit KiCad's own 3-argument `(at x y angle)` pad clause (confirmed real KiCad syntax) instead of today's always-2-argument form. For `rect`/`oval` pads specifically, the emitter has a choice of two KiCad-native representations that render identically — emit the pad's `(size w h)` unchanged and add the real `angle` argument (matching what an author declared), rather than silently swapping `w`/`h` and omitting the angle (which is what real hand-authored KiCad libraries do, per this RFC's own Alternatives research, but which would discard the explicit rotation fact CoHDL's grammar just captured). This preserves the author's stated intent losslessly in the emitted file, at the (accepted) cost of diverging cosmetically from typical hand-authored KiCad library conventions.
- **IPC-2581 emitter**: per-pad rotation is carried via IPC-2581's own existing pad/pin transform mechanism (the same `Xform`-style mechanism already used for RFC-020's board-outline/placement geometry) — no new IPC-2581 concept, a direct reuse of machinery this codebase already has for a structurally identical "rotate this geometry" fact.
- `cohdl fmt`: `rotate ANGLE` renders as a trailing clause on the `pad N: ... at (x, y)` line, directly mirroring `place ... rotate ANGLE`'s existing canonical form (RFC-009/RFC-020 precedent) — no new formatting rule category.
- Reserves a new E8xx sub-case (designators & parts, RFC-018's home for footprint-completeness checks): invalid `rotate` value on a pad placement.

## Teaching cost

Very low. An author who already knows `place ... rotate ANGLE` (RFC-020) already knows this entire mechanism — same keyword, same closed set, same "optional, defaults to unrotated" behavior, just usable at a second, structurally analogous placement site.

## Failure modes

- **An author writes a non-cardinal angle** (e.g. `rotate 45`) — caught immediately at declaration, naming the valid `{0, 90, 180, 270}` set.
- **An author rotates a **`circle`** pad expecting a visual change** — no error, but also no effect; this is documented explicitly (see Design) as an intentional no-op, not a silently-ignored mistake, since forbidding it would be an unhelpful special case.
- **An author expects **`rotate`** to also rotate the footprint's **`courtyard`**/**`silkscreen_ref` — it does not; those remain unaffected, since they are footprint-level, not per-pad (see Non-goals).

## Migration path

No existing footprint requires migration — `rotate`'s default (`0`) is exactly today's only behavior. A real, optional, non-mechanical follow-up: the std library's QFN/LQFP footprints (once authored, per RFC-018/021's own still-open real-content-authoring follow-up work) should use `rotate` for their top/bottom-side pads rather than defining duplicate rotated pad symbols — genuine authoring work, not required by this RFC's completion bar.

## Decision

**Accepted — 2026-07-19.** `pad N: PadSymbol at (x, y) [rotate ANGLE]` — `rotate` is a new optional clause on RFC-018's existing pad-placement statement, closed to `{0, 90, 180, 270}`, reusing RFC-020's exact rotation-angle set and keyword by direct precedent. Purely additive; every existing pad placement is unchanged. Serves the real, concrete QFN/LQFP per-side-rotated-pad pattern without forcing duplicate pad symbol definitions for the same physical pad shape at a different orientation. Recorded as DR-031 (see note 7). Language Specification (note 10) gains a "Rotated pad placements" subsection under Footprints and pads.
