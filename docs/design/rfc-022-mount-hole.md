# RFC-022: Mechanical locating holes in footprints (mount_hole)

## Problem

RFC-018 gave `footprint` real content: numbered `pad` placements (each bound to a device pin, RFC-002), plus `courtyard` (a keep-out boundary) and `silkscreen_ref` (a reference point) — neither of which is drilled/manufactured geometry. But some real footprints require a **mechanical locating hole** (定位孔) — a hole with no electrical function at all, used to physically register the component during placement/assembly (e.g. a connector shell's alignment pin holes, a shielding can's mounting-tab holes). This is real, common footprint content RFC-018 has no way to express today: every `pad N: PadSymbol at (x, y)` line is checked against the bound device's declared pin numbers (RFC-018's own central completeness guarantee) — there is no pin number for a locating hole to bind to, because it isn't a pin at all.

Grounded against real, established practice: KiCad's own footprint format has a dedicated `np_thru_hole` (non-plated through-hole) pad type for exactly this — a hole that exists in the footprint's physical geometry but carries no net, no pad number, no electrical role. This is not a novel concept; it's adopting an established, real distinction CoHDL's footprint model has so far omitted.

Who this is for: **library authors** completing real footprints (RFC-018's own audience) who hit this gap the moment a real connector/shield/mounting-hardware footprint needs one; and the KiCad/IPC-2581 emitters (RFC-015/018), which need real drill/hole geometry to project instead of silently omitting it or forcing an author to misuse `pad` for something that isn't electrical.

## Goals

- A new footprint-body construct, `mount_hole`, for a single mechanical locating hole: position + diameter, with an explicit plated/non-plated flag (real footprints do occasionally need a plated locating hole — e.g. one doubling as a chassis-ground stud — even though the common case is non-plated).
- Explicitly **not numbered against any device pin** — this is the core structural distinction from `pad`. `mount_hole` entries have their own numbered namespace (no duplicate hole number), never checked against RFC-002's pin list or the footprint's electrical pad numbers.
- Close the gap honestly: today, a library author with a real locating-hole footprint has no correct way to express it in CoHDL at all (misusing `pad` would incorrectly imply an electrical pin binding that doesn't exist). This RFC exists so that case has a correct, checkable answer.

## Non-goals

- **Not a general mechanical-CAD authoring system.** `mount_hole` is exactly what its name says: a circular hole, at a position, with a diameter and a plated/non-plated flag. No non-circular locating features (slots, keyed/D-shaped holes), no press-fit/tolerance modeling, no countersink/counterbore geometry.
- **Not board-level mounting holes** (the four corner screw holes on a board, chassis mounting points). Those are a board-level concept, not a per-footprint one, and are out of scope here — if/when board-level mounting holes need a real construct, that's separate future work, analogous to how `board_outline` (RFC-020) is a design-level concept distinct from any one footprint. This RFC covers only holes that belong to one component's own footprint (e.g. a connector's integral alignment-pin holes).
- **Not 3D models or assembly-drawing detail** — unchanged non-goal from RFC-018.
- **Not a second padstack system.** A plated `mount_hole` is deliberately minimal (diameter + drill, no independent per-layer geometry) — the same single-layer-plus-through-all scope discipline RFC-018 already applied to `pad`.

## Design

```cohdl
// sparkfun/src/footprints/usb_c_shielded.cohdl

pub footprint USBC_16P_Shielded {
    pad 1: Rect_0_25x0_6mm at (-3.5mm, 2.6mm)
    // ... pads 2-16 ...

    mount_hole 1: non_plated at (-4.32mm, 0mm) diameter 1.2mm
    mount_hole 2: non_plated at (4.32mm, 0mm) diameter 1.2mm

    courtyard { shape: rect, at: (0mm, 0mm), size: (9.5mm, 8.0mm) }
    silkscreen_ref { at: (0mm, -4.5mm) }
}
```

- `mount_hole N: PLATING at (x, y) diameter D` — `N` is a **locating-hole-local counter, disjoint from the pad-number namespace**, starting at 1. It is never checked against the bound device's pins (the defining difference from `pad N: ...`) — a footprint may have `pad 1..16` and `mount_hole 1..2` in the same declaration with no collision, because they are different, independently-numbered sequences.
- `PLATING` is one of `non_plated` (the common case — a bare mechanical hole, no copper) or `plated` (a plated through-hole with no net — e.g. a chassis-ground stud). This mirrors `pad`'s existing `plating: smd | plated_through_hole` vocabulary but is spelled as its own closed two-value set here, since a `mount_hole` is never `smd` (it is, definitionally, a hole, not a surface pad).
- `diameter D` — a single `Length`-typed value (RFC-018's own precedent: `pad`'s `drill:` field). Required for every `mount_hole`, regardless of plating.
- No `layer:` field — a `mount_hole` always spans `through_all`, the same as any drilled hole; there is no meaningful single-side locating hole.

## Type-system-first test

Both checks this RFC introduces are structural and local to one footprint declaration, never DRC candidates:

1. **No duplicate `mount_hole` numbers within one footprint** — checked the moment the footprint is parsed. This uniqueness belongs only to the mechanical-hole namespace; electrical `pad` numbers may repeat when one terminal has multiple physical placements.
2. **`mount_hole` numbers never collide with, or get checked against, `pad` numbers or the bound device's pin list** — this is a non-check as much as a check: the RFC's core structural guarantee is that these are disjoint namespaces, verified by construction (the grammar and resolver simply never compare the two), not by an emergent cross-graph rule.

Neither is DRC: both are checkable from the one footprint declaration in isolation, the same shape RFC-018 already established for pad-count/numbering consistency.

## Conceptual impact

Low. No new top-level declaration kind, no new resolution mechanism — `mount_hole` is a third footprint-body construct alongside the existing `pad`/`courtyard`/`silkscreen_ref` trio, reusing the same `Length` unit type (RFC-001/018) and the same "closed vocabulary field block" shape every other CoHDL construct already uses. It does introduce one new idea worth naming honestly: a footprint may now contain geometry that is real, drilled, and manufactured, but carries no electrical meaning at all — a genuine but small addition to what "footprint content" means, not a new category of top-level thing.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Med | Low | Med |

**Netlist (Med):** the KiCad/IPC-2581 emitters (RFC-015/018) gain a real new geometry kind to project — a non-plated or plated hole with no net, distinct from every pad they already emit. This is genuine, if small, new emitter work.
**Trust (Med, not High):** this closes a real gap (a library author previously had no correct way to express this), but it does not itself add a new completeness *guarantee* the way RFC-018's pad-count check did — there is no bound "thing" a `mount_hole` must match (unlike a pad, which must match the device's pin list), so there is less for the compiler to hold an author accountable to here. Honest, not inflated.
**Concepts/Grammar/Oracle/Diagnostics/Compat (Low):** small, closed, additive vocabulary; reuses every existing mechanism (Length unit, footprint-body statement shape, RFC-016 resolution untouched since nothing new is resolved by name).

## Gradeability

- Duplicate `mount_hole` number within one footprint — checked at footprint declaration/parse time, naming the duplicated number.
- `diameter` missing or non-`Length`-typed — checked at declaration time, the same unit-type enforcement RFC-001 already guarantees everywhere.
- `PLATING` outside the closed `{non_plated, plated}` set — checked at declaration time, listing the two valid values.
- None of this runs in residual DRC.

## AI-generatability

High. `mount_hole`'s shape is small, closed, and directly parallels `pad`'s already-familiar shape (`N: KIND at (x, y) FIELD`) — an author or model that already knows how to write a `pad` line has effectively already learned `mount_hole`'s shape. The one real authoring cost, same as `pad`, is real datasheet-derived dimensions (hole diameter, position) — this RFC doesn't make that free, same honest limitation RFC-018 already stated for pad/footprint authoring generally.

## Alternatives

- **Model a locating hole as a **`pad`** with a new **`plating: mechanical`** value and no bound pin** — rejected: this would break RFC-018's own central guarantee (every `pad N: ...` number must exactly match a bound device pin, checked at `cohdl build`) by requiring a special-cased exception to that check for exactly this one plating value — a structural inconsistency, not a clean extension. Keeping `mount_hole` as its own construct with its own, disjoint numbering namespace is what lets RFC-018's pad-completeness guarantee stay unconditionally true, with zero special cases.
- **Fold locating holes into **`courtyard` — rejected: `courtyard` is a keep-out boundary (no drill, no manufactured geometry) — conflating it with a real drilled hole would be modeling two different kinds of footprint content (a soft placement-clearance convention vs. a hard manufactured feature) as one construct, the same kind of category error RFC-018's own Alternatives section already rejected for merging `pad`/`footprint`.
- **A general mechanical-feature sub-language** (slots, keyed holes, countersinks, arbitrary shapes) — considered, rejected as premature: no concrete need beyond circular locating holes has been shown yet; the same narrow-scope-first discipline RFC-018 applied to `pad`'s shape vocabulary (`rect`/`circle`/`oval`, not general polygons) applies here. A richer shape vocabulary is a scoped future RFC if/when a real footprint needs a non-circular locating feature.
- **Board-level mounting holes in the same RFC** — considered, rejected as out of scope: a board's four corner screw holes are a design/board-level fact (closer in spirit to `board_outline`, RFC-020), not a per-footprint one; bundling both would conflate two different owners (a footprint's own author vs. a design's own board-level layout) into one construct.

## Compatibility

Purely additive. `mount_hole` is a new, optional footprint-body statement — every existing footprint declaration with no `mount_hole` entries is completely unaffected, unchanged in meaning, unchanged in emitted bytes. No renumbering, no keyword changes to `pad`/`footprint`/`courtyard`/`silkscreen_ref`.

**Depends on**: RFC-018 (pad/footprint) — already Accepted. No dependency on RFC-016's resolution machinery beyond what RFC-018 already established, since `mount_hole` introduces no new named, resolvable symbol.

## Tooling & operations

- `cohdl lsp` hover on a `mount_hole N: ...` line should show its resolved diameter/plating, the same "resolve and show" precedent already established for `pad` lines.
- `cohdl fmt` needs one new formatting rule: `mount_hole N: PLATING at (x, y) diameter D` lines, aligned the same way `pad N: ...` lines already are.
- Reserves new error-code sub-cases in the existing E8xx block (designators & parts — RFC-018's own home for footprint-completeness checks): duplicate `mount_hole` number, missing/malformed `diameter`, invalid `PLATING` value. No new block.
- `cohdl build`'s KiCad `.kicad_mod` emitter projects `non_plated` as KiCad's own `np_thru_hole` pad type (the exact real-world precedent this RFC is grounded in) and `plated` as an ordinary plated through-hole pad with no net assigned. The IPC-2581 emitter projects both as `Hole`/`Pin` geometry with no net reference, consistent with how IPC-2581 already represents non-electrical drilled features.

## Teaching cost

Low. One new, small, closed-vocabulary footprint-body statement, directly parallel in shape to `pad` (which every footprint author already knows). The one real concept to learn — that `mount_hole` numbers are a separate namespace from pad numbers and are never checked against the device's pins — is a single, one-time fact, not a per-footprint cost.

## Failure modes

- **A library author uses **`pad`** instead of **`mount_hole`** for a locating hole**, to avoid learning the new construct — this would fail today at `cohdl build`'s existing pad-count/numbering check (RFC-018) the moment the pad number doesn't correspond to a real device pin, so this misuse is already caught, not silently accepted. `mount_hole`'s existence gives the correct alternative rather than leaving an author to work around the gap incorrectly.
- **A **`mount_hole`**'s diameter is dimensionally wrong but structurally valid** (a typo'd value) — this RFC's checks cannot catch this, the same disclosed, unavoidable limitation RFC-018 already named for pad dimensions generally.
- **A locating hole that should be board-level (not footprint-level) is modeled as a **`mount_hole`** on some arbitrary component** — this RFC has no check to prevent that conceptual misuse; it's a documentation/convention matter (see Non-goals), not something the type system can distinguish, since both are geometrically "a hole at a position."

## Migration path

No existing footprint declarations use `mount_hole` (it doesn't exist before this RFC), so there is nothing to migrate. Real, non-mechanical authoring work remains for library authors: any existing footprint that *should* have had a locating hole (e.g. a connector footprint currently missing its alignment-pin holes) can now be completed correctly — this is new content to author, not a retrofit forced by this RFC.

## Decision

**Accepted — 2026-07-17.** Recorded as DR-028 (see note 7). Adds `mount_hole` as a new, optional, footprint-body construct for non-electrical mechanical locating holes — disjoint in numbering from `pad`, never checked against a bound device's pins, closing a real gap RFC-018 left open (library authors previously had no correct way to express footprint-integral locating holes such as connector alignment pins). Grounded in KiCad's own established `np_thru_hole` precedent. Language Specification (note 10) gains a new subsection under "Footprints and pads."
