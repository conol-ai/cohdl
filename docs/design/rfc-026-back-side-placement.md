# RFC-026: Component placement on the board's back side

## Problem

Grounded directly in a real, universal PCB fact: dual-sided boards routinely place components on both the top and bottom of the board — e.g. bulk decoupling capacitors, secondary connectors, or backside RF shielding tabs placed on the bottom to save board area. Confirmed against real source (`src/ast.rs`): `Placement` — the `place <inst> at (x, y) [rotate ANGLE]` statement (RFC-020) — has exactly three fields: `inst`, `at`, `rotate`. There is no side/layer field anywhere. Every component `place`d today is implicitly, unconditionally top-side — there is no way to express "this instance sits on the back of the board" at all.

Confirmed against the real KiCad `.kicad_pcb` format (KiCad's own dev-docs file-format reference): a placed footprint's board side is a real, distinct property of the placement itself — `(layer F.Cu)` for front (top) or `(layer B.Cu)` for back (bottom) — carried on the footprint instance, separate from its `(at x y angle)` position/rotation. Placing a footprint on the back side is standard PCB CAD practice and, critically, requires the footprint's pad geometry to be mirrored (left-right flipped) relative to its top-side orientation — a real, well-established convention (viewing a bottom-side component "through" the board from the top), not merely a cosmetic relabeling.

## Goals

- Let an author state that a specific instance is placed on the board's bottom side, not the top — a real, common, and currently entirely unexpressible fact.
- Compose cleanly with RFC-020's existing `rotate` clause: a bottom-side component can still be rotated to any of the same closed `{0, 90, 180, 270}` values, measured in the mirrored (bottom-side) frame — the standard PCB CAD convention.
- Keep this a placement-time fact layered on top of an unchanged, already-declared `footprint` (RFC-018) — a footprint's own pad geometry is authored once, for its natural (top-side) orientation, and never needs a second, separately-authored "bottom-side" version.

## Non-goals

- **Not a per-pad layer override.** RFC-018's `pad.layer` (`top_copper` / `bottom_copper` / `through_all`) already exists and means something different and narrower: which single copper layer *one pad within an otherwise-fixed footprint* sits on (e.g. a castellated edge pad, or a bottom-side-only test point pad within a footprint that itself is still placed top-side). This RFC's `side` is a whole-component fact — "this entire instance, footprint and all, is flipped to the back" — and must not be confused with, or implemented by reusing, `pad.layer`. The two remain independent, unrelated mechanisms serving genuinely different questions.
- **Not board-level layer stackup** (how many copper layers a board has, their order) — that remains named future work per RFC-015's own disclosed gap, unaddressed here. This RFC only concerns which of the two *outer* sides (top/bottom) a component sits on, not inner-layer stackup.
- **Not automatic mirroring/reflection math authored by CoHDL.** CoHDL states the fact (`side: bottom`); actually mirroring the footprint's pad coordinates is downstream emitter work (see Tooling & operations) using each output format's own native mirroring convention — CoHDL itself performs no geometric reflection computation of its own, the same "declared fact, not computed" discipline `rotate` (RFC-020) already established.
- **Not per-pin/per-net "this side" declarations** — `side` applies to a whole placed instance, never to an individual pin or a subset of a footprint's pads. A footprint is placed on one side or the other, in its entirety; there is no concrete need shown for anything finer-grained.

## Design

```cohdl
design DualSidedBoard {
    inst mcu: MCU_ESP32S3
    inst bulk_cap: MLCC<10uF, 16V>
    inst rf_shield_tab: RF_Shield_Contact

    layout {
        place mcu at (0mm, 0mm)
        place bulk_cap at (5mm, 5mm) side bottom
        place rf_shield_tab at (12mm, -3mm) side bottom rotate 180
    }
}
```

- `place <inst> at (x, y) [rotate ANGLE] [side SIDE]` — `side` is a new, optional clause on the existing `Placement` statement (RFC-020). `SIDE` is closed to `{top, bottom}`. Omitted `side` defaults to `top` — every existing `place ... at (x, y) [rotate ANGLE]` statement, written before this RFC, is unchanged in meaning.
- `side` and `rotate` are fully independent, composable clauses — either, both, or neither may appear, in either order at the parse level (canonical order fixed by `cohdl fmt`, see Tooling & operations). `rotate`'s closed `{0, 90, 180, 270}` set is unchanged and applies identically regardless of `side` — a `side bottom rotate 90` component is rotated 90° within its own (mirrored) bottom-side frame, the same convention every mainstream PCB tool already uses.
- The referenced instance's `footprint` (RFC-018) is authored exactly once, for its natural orientation — `side bottom` never requires, or permits, a second, separately-authored mirrored footprint declaration. Mirroring is a placement-time, emitter-level transform applied to the one real footprint declaration, never a second copy an author maintains by hand.

## Type-system-first test

Not a `rule`/DRC proposal. `side`'s closed-set membership is a small, structural, local check at the point a `place` statement is declared:

1. **Closed-set membership** — `SIDE` must be one of `{top, bottom}`; any other value is a compile error listing the valid set, identical in shape to `rotate`'s existing closed-set check.
2. **No new emergent/cross-cutting check** — `side`'s value is entirely local to its own `Placement` statement; nothing about it depends on any other instance, net, or the whole connectivity graph. Never a residual-DRC candidate.

## Conceptual impact

Low. No new core concept, no new top-level declaration kind — `side` is one new optional field on an existing statement (`Placement`, RFC-020), directly parallel in shape and discipline to `rotate` (same RFC, same statement, same closed-set-with-default pattern). An author who already understands `rotate` needs almost no new mental model — just one more independent, orthogonal placement fact.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Med | Low | High |

**Trust (High):** `side`'s default (`top`) preserves every existing `place` statement's meaning exactly — no silent reinterpretation of any already-written design.
**Netlist (Med):** real, new emitter work is required (KiCad `.kicad_pcb`'s `(layer F.Cu | B.Cu)` on the placed footprint plus real pad-geometry mirroring; IPC-2581's equivalent side/mirror representation) — genuine new surface, not a free re-projection, so rated above the other Low cells honestly.
**Grammar (Low):** one new optional clause, closed-set-checked, on one existing statement — no new statement kind, no interaction with any other construct's grammar.
**Compat (Low):** purely additive; every pre-existing `place` statement is unaffected.

## Gradeability

- `SIDE`'s closed-set membership is checked at the point `place ... side SIDE` is declared — the same discipline, same error shape, as `rotate`'s existing check.
- No other new semantic check is introduced. `side`'s presence/value has no bearing on any existing check (pin obligations, trait satisfaction, designator allocation, footprint pad-count consistency) — all unaffected.
- Not a residual-DRC candidate — see Type-system-first test above.

## AI-generatability

High. A model that has already learned `rotate ANGLE` (RFC-020) needs only to recognize one additional, independently-composable, closed-set clause on the identical statement — no new vocabulary shape, no new closed-set-checking pattern to learn. "This part is on the bottom of the board" is an extremely common, well-understood real-world fact any model is very likely to already associate correctly with dual-sided board designs.

## Alternatives

- **Model board side as a property of the** `footprint` **declaration itself** (e.g. a `footprint`-level `default_side: bottom`) — rejected: a footprint's own declaration describes reusable geometry, independent of any specific board's layout; the same footprint (e.g. a generic 0603 MLCC) is placed on the top of some boards and the bottom of others, so "which side" cannot correctly live on the footprint — it is a fact about one placement, at one design, not about the reusable pad-geometry symbol itself.
- **Model board side as a property of the** `inst` **declaration** rather than `place` — rejected for the same reason RFC-020 itself already keeps `rotate` on `place` rather than `inst`: placement facts (position, rotation, and now side) are logically grouped together at the point a design lays itself out, in the same `layout { ... }` block, not scattered across the design body wherever an instance happens to be declared.
- **A general mirror/flip transform (arbitrary reflection axis)** — rejected per this project's recurring narrow-scope-first discipline (RFC-001's closed units, RFC-020's own closed rotation set, RFC-025's closed rotation set for pads): a PCB has exactly two sides; there is no concrete need for, or physical meaning to, an arbitrary reflection axis. A closed two-value `{top, bottom}` set is the entire real problem space.
- **Reuse RFC-018's **`pad.layer`** (**`top_copper`**/**`bottom_copper`**) at the placement level instead of a new** `side` **clause** — rejected: `pad.layer` answers a narrower, different question (which single copper layer one pad occupies within an otherwise-fixed footprint), not "which side of the board is this whole component on." Reusing the same vocabulary for two different questions would be exactly the kind of conceptual conflation this project's discipline (e.g. RFC-022's rejection of merging `mount_hole` into `pad`) consistently avoids.

## Compatibility

Purely additive. `side` is a new optional clause on `Placement`; every existing `place <inst> at (x, y) [rotate ANGLE]` statement (no `side`) is completely unaffected, unchanged in meaning and in every emitted byte.

**Depends on**: RFC-020 (board outline + oriented placement, already Accepted) — `side` extends `Placement`, RFC-020's own construct, in the identical way RFC-025 extended `PadPlace`. No dependency on RFC-018's `pad.layer` — the two remain independent mechanisms, per Non-goals/Alternatives above.

## Tooling & operations

- **KiCad **`.kicad_pcb`** emitter**: a `side: bottom` instance is emitted with `(layer B.Cu)` (instead of the default `F.Cu`) on the placed footprint, and every one of the footprint's own pad coordinates is mirrored (X-axis reflection, the real KiCad-native convention for a back-side footprint) before emission — genuine new, but bounded, emitter geometry work.
- **IPC-2581 emitter**: side is carried via IPC-2581's own existing per-component side/layer attribute (analogous to how RFC-020's rotation is already carried via IPC-2581's `Xform`-style mechanism) — no new IPC-2581 concept, direct reuse of existing machinery for a structurally similar "which physical layer is this on" fact.
- `cohdl fmt`: canonical clause order on a `place` statement becomes `place <inst> at (x, y) [rotate ANGLE] [side SIDE]` — `rotate` before `side` — a single fixed ordering, consistent with `cohdl fmt`'s "one canonical way" principle (RFC-009); an author writing `side` before `rotate` is reformatted, not rejected.
- Reserves a new E10xx sub-case (layout constraints, RFC-013/020's home for placement-related diagnostics): invalid `side` value on a placement.

## Teaching cost

Very low. An author who already knows `place ... rotate ANGLE` (RFC-020) already knows this entire mechanism's shape — one more independent, optional, closed-set, default-preserving clause on the same familiar statement.

## Failure modes

- **An author writes an invalid side value** (e.g. `side left`) — caught immediately at declaration, naming the valid `{top, bottom}` set.
- **An author expects **`side bottom`** to also change the referenced device's pin obligations, trait satisfaction, or designator** — it does not; `side` is purely a physical-placement fact, exactly as `rotate` already is, with zero bearing on any electrical/logical check.
- **An author expects CoHDL to catch a component placed on the wrong side for a real mechanical reason** (e.g. a connector that must face outward through an enclosure cutout) — CoHDL performs no such reasoning; `side`, like `rotate`, is a declared fact for a partner tool to act on, not a checked constraint, consistent with DR-003's "layout stays a partner concern" boundary.

## Migration path

No existing design requires migration — `side`'s default (`top`) is exactly today's only behavior, for every `place` statement written before this RFC. A real, optional, non-mechanical follow-up: any real dual-sided board design (once one exists in the repo) can adopt `side bottom` for its actual back-side components — genuine authoring work, not required by this RFC's completion bar.

## Decision

**Accepted — 2026-07-19.** `place <inst> at (x, y) [rotate ANGLE] [side SIDE]` — `side` is a new optional clause on RFC-020's existing placement statement, closed to `{top, bottom}`, defaulting to `top`. Fully independent of and composable with the existing `rotate` clause. Kept deliberately distinct from RFC-018's unrelated `pad.layer` concept (a narrower, per-pad fact, not a whole-component one). Purely additive; every existing `place` statement is unchanged. Recorded as DR-032 (see note 7). Language Specification (note 10) gains a "Component placement on the board's back side" subsection under Board outline and oriented placement.
