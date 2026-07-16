# RFC-021: IPC-7351 as the canonical footprint naming practice

# Problem

RFC-018 gave `footprint` real, checkable pad content, but left one thing completely unconstrained: **the name** a library author gives a `footprint` declaration. Today `QFN10_3x3`, `qfn_10x10_thing`, `MyFootprint_v2` are all equally legal — the only rule is RFC-016's ordinary identifier grammar. Two different libraries authoring "the same" real-world package (already an acknowledged, un-deduplicated gap per RFC-017's Failure modes) currently have no shared convention to converge on even by accident, and an AI generating a new `footprint` declaration has no signal for what a *good* name looks like beyond "resolves, doesn't collide."

Research conducted alongside this RFC confirms the real industry already solved this problem, twice, at different layers: **IPC-7351** (IPC-7351B currently, 7351C pending) is the industry-standard **naming and land-pattern-sizing methodology** for SMD footprints — a structured, derivable string encoding package family, pitch, lead/ball span, height, pin count, and density (Least/Nominal/Most). **JEDEC JESD30** separately names the *package outline* (the mechanical body — QFN, LQFP, BGA, etc.) that IPC-7351 then turns into a *land-pattern* name. Every real CAD library (KiCad's `kicad-footprints`, SnapEDA, Ultra Librarian) either directly emits IPC-7351-derived names or a human-readable variant of the same fields (body size + pitch + pin count), specifically because it lets an engineer infer geometry from the name alone, and lets tooling validate that a name and its actual geometry agree.

Tony's directive: adopt IPC-7351 (not JEDEC JESD30, not an invented CoHDL-native scheme) as CoHDL's required footprint naming practice — every `footprint` declaration's name must be, or must carry, a valid IPC-7351-derived designator, and `cohdl build`/`cohdl check` should be able to verify a footprint's declared IPC-7351 name is actually consistent with its own pad geometry (closing the same class of "the name lies" gap RFC-018 already closed for "the footprint lies").

Who this is for: **library authors** (the RFC-017 audience), who now have a concrete, non-arbitrary naming discipline instead of "pick anything that resolves"; and **AI generating footprint declarations**, which can derive a correct name mechanically from a package's own known dimensions instead of inventing one.

# Goals

- Adopt IPC-7351B's naming grammar as CoHDL's **canonical footprint-name format**, applied as a structured, parseable string — not prose guidance library authors are merely encouraged to follow.
- Give `footprint` declarations a new, optional `ipc_name: "..."` field carrying the IPC-7351-derived designator, checked for internal well-formedness (matches the IPC-7351 grammar for its stated package family) and, where the package family's pad geometry is fully regular (rectangular-grid SMD pin arrays — the common QFP/SOIC/SON/QFN case), cross-checked against the footprint's own `pad` placements for consistency (pin count, and pitch when uniform).
- Keep the existing free-form `footprint` symbol name (RFC-016/017 module-path identifier) completely unchanged in role — `ipc_name` is additive metadata on top of it, not a replacement for module-path resolution. A footprint is still reached by `use path::Name;`, never by its `ipc_name`.
- Establish IPC-7351's package-family vocabulary (QFP, BGA, SOIC, SOT, QFN/SON, CHIP, MELF, ...) and density-level suffix (`N`/`L`/`M`) as a closed, documented set CoHDL's tooling understands structurally, so a malformed or internally-inconsistent `ipc_name` is a real compile-time diagnostic, not silently accepted prose.

# Non-goals

- **Not IPC-7351's land-pattern *****calculation***** formulas** (the pad-size-from-lead-dimension math, tolerance/density-level geometry derivation). This RFC adopts IPC-7351 as a **naming and consistency-check** discipline layered on top of RFC-018's already-existing hand-authored `pad`/`footprint` geometry — it does not make CoHDL compute pad sizes from datasheet lead dimensions. Library authors still hand-author `pad` geometry exactly as RFC-018 established; this RFC only names the result and optionally checks the name agrees with what was hand-authored.
- **Not JEDEC JESD30** (package-outline designators). IPC-7351 names the *land pattern*, JESD30 names the *package body* — this RFC scopes only the land-pattern-naming layer CoHDL actually owns (`footprint`), not a parallel package-designator system. A future RFC could adopt JESD30-style naming for `package`/`variants` (RFC-008) if a real need emerges; not proposed here.
- **Not retroactively renaming or migrating every existing footprint symbol name.** `ipc_name` is optional and additive — existing footprints (all still placeholders per RFC-017/018's own disclosed migration state) are unaffected until a library author chooses to add one.
- **Not full IPC-7351B grammar coverage on day one.** This RFC covers the package families needed for CoHDL's own real, in-flight hardware (QFP/LQFP/TQFP, QFN/SON, SOIC/SOP, SOT, BGA, and passive CHIP/MELF two-terminal parts) — the closed set below — not IPC-7351B's full catalog (which also covers connectors, relays, and other families CoHDL has no current examples for). Extending the set is a scoped follow-up, same discipline as RFC-001's closed unit-type set.
- **Not automatic **`ipc_name`** generation from pad geometry.** A library author writes `ipc_name` by hand (deriving it from the datasheet, same authoring effort as writing the `pad` placements themselves); the compiler checks it, it does not invent it.

# Design

## IPC-7351B naming grammar, adopted as a closed structural format

IPC-7351B's naming convention is a family-specific template, but every family shares the same broad shape: **package-family prefix + pitch + span/body dimensions + (pin count) + density suffix**. CoHDL adopts the following closed set of family templates (the ones covering CoHDL's real current hardware; see Non-goals for scope):

| Family prefix | Meaning | Template |
|---|---|---|
| `QFP` | Quad flat pack (incl. LQFP/TQFP, same land-pattern shape) | `QFP` + pitch(P) + leadspan_X + `X` + leadspan_Y + `X` + height + `-` + pins + density |
| `QFN` | Quad flat no-lead (incl. SON, VQFN) | `QFN` + pins + density + pitch(P) + body_X + `X` + body_Y + [`-1EP` + epad_X + `X` + epad_Y, if exposed pad] |
| `SOIC` / `SOP` | Small-outline IC | `SOIC` or `SOP` + pins + `P` + pitch + `X` + leadspan + `X` + height + density |
| `SOT` | Small-outline transistor | `SOT` + pins + `P` + pitch + `X` + body_X + `X` + body_Y + density |
| `BGA` | Ball grid array | `BGA` + pins + (`C` | `N`) + pitch(P) + cols + `X` + rows + `_` + body_X + `X` + body_Y + `X` + height + density |
| `CHIP`/`MELF` | Two-terminal passives (resistors/caps) | `CHIP`/`MELF` + `-` + EIA size code (e.g. `0402`, `0603`) — density suffix not applicable |

- **Pitch, span, body, height, and exposed-pad dimensions are encoded in hundredths of a millimeter, no decimal point, no unit suffix** — this is IPC-7351B's own convention (e.g. `50` = 0.50mm pitch, `900` = 9.00mm), adopted verbatim rather than reinvented, so an `ipc_name` string is directly comparable against the real industry's own footprint libraries (KiCad, SnapEDA) rather than a CoHDL-specific dialect.
- **Density suffix**: `N` (Nominal — default, used unless a library author has a specific reason to deviate), `L` (Least — smallest pads, dense designs), `M` (Most — largest pads, rugged/hand-assembly designs). Closed three-value set, same discipline as RFC-001's unit-type table and RFC-008's pin-role set — an `ipc_name` with no density suffix, or a suffix outside `{N, L, M}`, is a compile error.

## `footprint` gains an optional `ipc_name` field

```cohdl
// sparkfun/src/footprints/qfn.cohdl → module path sparkfun::footprints::qfn

use sparkfun::pads::smd::Rect_0_3x0_9mm;

pub footprint QFN10_3x3 {
    ipc_name: "QFN10N40P300X300-1EP180X180"   // IPC-7351B-derived designator

    pad 1: Rect_0_3x0_9mm at (-1.5mm, 1.0mm)
    pad 2: Rect_0_3x0_9mm at (-1.5mm, 0.5mm)
    pad 3: Rect_0_3x0_9mm at (-1.5mm, 0.0mm)
    // ... one entry per pad
    courtyard { shape: rect, at: (0mm, 0mm), size: (3.5mm, 3.5mm) }
    silkscreen_ref { at: (0mm, -2.2mm) }
}
```

- `ipc_name` is a **string literal**, not a symbol reference — unlike `footprint`'s own module-path identity, an IPC-7351 name is descriptive metadata about the land pattern's geometry, not a cross-library-resolvable thing anything else refers to by name. (See Alternatives for why this isn't itself a resolvable symbol.)
- `ipc_name` is **optional** on any `footprint` declaration — a footprint with no `ipc_name` compiles exactly as it does today under RFC-018, unaffected by this RFC.
- When present, `ipc_name` is checked in two stages:**Grammar well-formedness** — the string must parse against one of the closed family templates above (right prefix, right field order, valid density suffix). A malformed `ipc_name` (wrong family prefix, missing density suffix, non-numeric dimension field) is a compile error naming the specific parse failure and which template it was closest to.**Geometry cross-check** (where derivable) — for package families with a fully regular pad layout (`QFP`, `QFN`, `SOIC`/`SOP`, `SOT` — rectangular-perimeter pin arrays with uniform pitch), the compiler derives pin count and pitch directly from the footprint's own `pad N: ... at (x, y)` placements and confirms they match the `ipc_name`'s encoded pin count and pitch. A mismatch (e.g. `ipc_name` says 10 pins but the footprint places 12 `pad` entries, or claims 0.4mm pitch but the placements are spaced 0.5mm apart) is a compile error naming the specific disagreement. `BGA`'s column/row grid and passive `CHIP`/`MELF` sizes are checked the same way, adapted to their own template shape. This is deliberately **not** attempted for irregular/mixed-pitch layouts — see Non-goals and Failure modes.

## Example: the two devices used to motivate this RFC

```cohdl
// STM32F103C8T6 — LQFP-48, 7x7mm body, 0.5mm pitch, nominal density
pub footprint LQFP48_7x7_P0_5 {
    ipc_name: "QFP50P900X900X160-48N"
    pad 1: Rect_0_3x1_5mm at (-4.5mm, 3.75mm)
    // ... 47 more pads
}

// RP2350A — QFN-60, 7x7mm body, 0.4mm pitch, exposed pad, nominal density
pub footprint QFN60_7x7_P0_4_EP3_4 {
    ipc_name: "QFN60N40P700X700-1EP340X340"
    pad 1: Rect_0_2x0_75mm at (-3.35mm, 3.05mm)
    // ... 59 more pads
    // exposed thermal pad modeled as a pad entry on `through_all`/`smd` per RFC-018's existing vocabulary
}
```

These two are exactly the pair used to validate this RFC's naming derivation against real datasheets before drafting — cross-checked against JEDEC/manufacturer package data and KiCad's own real library names.

## `footprint_alias` (Footprint Binding design note) gains an `ipc` field slot

The pre-existing `footprint_alias` construct (Footprint Binding — Design) already reserves per-backend keys (`kicad:`, `lceda:`, `allegro:`, `default:`). This RFC adds `ipc:` as a recognized key, carrying the same `ipc_name` string, so a design that hasn't yet adopted a real `footprint`/`pad` declaration (still on the alias/string-map path) can still declare its intended IPC-7351 identity:

```cohdl
footprint_alias LQFP48_7x7 {
    kicad:   "Package_QFP:LQFP-48_7x7mm_P0.5mm"
    ipc:     "QFP50P900X900X160-48N"
    default: "LQFP-48"
}
```

This key is not type-checked against geometry (an alias has no `pad` placements to cross-check against) — it is accepted at the same fidelity as the other string-valued backend keys, useful only for authoring clarity and cross-referencing. Once a real `footprint`/`pad` declaration exists for the same part, its `ipc_name` field (checked, per Design above) is the source of truth.

# Type-system-first test

Both checks this RFC introduces are structural and local to one `footprint` declaration's own already-written content — never DRC candidates:

1. `ipc_name`** grammar well-formedness** — checkable the moment the string literal is parsed, against a fixed, closed grammar table (one of six family templates). Exactly the same shape as RFC-001's unit-literal grammar check.
2. `ipc_name`**-vs-pad-geometry consistency** — checkable entirely from one `footprint` declaration's own `pad N: ... at (x, y)` list (pin count = number of `pad` entries; pitch = the regular spacing between them, when the layout is a uniform rectangular perimeter). No cross-declaration or cross-design lookup needed, so this is a compile-time (type-system) check, not DRC — consistent with RFC-018's own precedent for the pad-count-vs-device-pins check.

# Conceptual impact

Low. No new core concept — `ipc_name` is a new optional field on the already-Accepted `footprint` declaration kind (RFC-017/018), the same shape of addition RFC-012's `#[intent(...)]` and RFC-013's `#[placement_hint(...)]` made to other declaration kinds, except this field is **not** zero-impact metadata (see Coherence matrix below) — it is checked, by design, because unlike `#[intent(...)]`'s deliberately-decorative prose, an IPC-7351 name has real, checkable structure worth not letting drift from the geometry it claims to describe.

# Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Low | Med | Low | Low | Med |

**Grammar (Med)**: one new optional field (`ipc_name: "..."`) on `footprint`, plus a closed six-template family grammar the compiler must parse and validate — a real, bounded grammar addition, not a rename or reshaping of anything existing.

**Oracle (Low)**: no new DRC surface — both checks are type-system (see Type-system-first test above), resolved entirely from one declaration's own already-declared content.

**Diagnostics (Med)**: two new, specific failure classes (malformed `ipc_name`, `ipc_name`-vs-geometry mismatch), each naming the specific field that disagrees — new E8xx sub-cases (see Tooling & operations), not a new block.

**Netlist (Low)**: `ipc_name` is descriptive metadata, not physically emitted geometry — it may optionally be carried into IPC-2581 output as a component-attribute string (mirroring how `#[intent(...)]` content could be surfaced by a future tooling RFC), but this RFC does not require any emitter change; the existing `.kicad_mod`/IPC-2581 pad-geometry projection (RFC-018) is unaffected.

**Compat (Low)**: purely additive — every existing `footprint` declaration (all still placeholders per RFC-017/018's disclosed migration state) compiles unchanged with no `ipc_name` present.

**Trust (Med)**: this is the mechanism that makes "the footprint's name lies about its own geometry" structurally impossible for the regular-layout families it covers — a real, new trust guarantee, though scoped to the geometry-regular families only (see Failure modes for what it does not cover).

# Gradeability

- `ipc_name` grammar well-formedness is checked the moment the `footprint` declaration is parsed and type-checked — declaration time, the earliest possible stage, identical timing to RFC-018's `pad` internal-consistency check (`drill:`/`plating:` agreement).
- The geometry cross-check runs once the same `footprint` declaration's full `pad` list is visible — also at declaration time (not deferred to `cohdl build`, since it needs no external reference the way RFC-018's pad-count-vs-device check does — the device isn't involved here, only the footprint's own internal pad list vs. its own `ipc_name` string). This is strictly earlier than RFC-018's device-cross-check, and thus strictly stronger gradeability for the part it covers.
- Diagnostics name the specific mismatched field (e.g. "ipc_name declares 10 pins, footprint declares 12 pad entries" or "ipc_name declares 40 (0.40mm) pitch, footprint's pad spacing is 50 (0.50mm)"), following the same specific-mismatch-naming discipline RFC-018 established for pad-count/numbering diagnostics.

# AI-generatability

High for the mechanical part: once a device's package family, pitch, span, height, pin count, and density level are known (all already needed to hand-author the `pad` placements themselves, per RFC-018's own honest Medium AI-generatability assessment), deriving the matching `ipc_name` string is a **pure, deterministic string-formatting function** over data the model already has to gather — no new judgment call, no new datasheet lookup beyond what RFC-018 already required. This is a genuine generatability win over the status quo (an arbitrary, unconstrained footprint symbol name): a model no longer has to invent a name convention per-library, it has exactly one correct answer to compute. The geometry cross-check (Type-system-first test above) means a model that gets the derivation wrong is told immediately and specifically, rather than shipping a footprint whose declared identity silently disagrees with its actual copper.

# Alternatives

- **Adopt JEDEC JESD30 instead of, or alongside, IPC-7351** — considered, rejected for `footprint` specifically: JESD30 names the package *body* (the mechanical outline), not the *land pattern* (the copper geometry `footprint`/`pad` actually declare) — the wrong layer for what this RFC's construct owns. JESD30-style naming remains a legitimate candidate for a future RFC scoping `package`/`variants` (RFC-008) naming, not proposed here (see Non-goals).
- **Invent a CoHDL-native naming scheme instead of adopting an existing standard** — rejected per Tony's explicit direction and per the same reasoning RFC-018 already applied when it rejected `copad`/`cofp` in favor of plain English names: inventing new vocabulary when a well-understood, industry-proven convention already exists and does the job is unnecessary teaching cost with no compensating benefit. IPC-7351 names are also directly comparable against the real ecosystem's own footprint libraries (KiCad, SnapEDA), which a CoHDL-native scheme would never be.
- **Make **`ipc_name`** the footprint's actual module-path/symbol name (require it, replace the free-form identifier)** — rejected: RFC-016's module-path identifier resolution (`use path::Name;`) needs a stable, author-chosen, human-memorable symbol name (`QFN10_3x3`), while IPC-7351 names are long, mechanically-derived, and change if the pitch/span/density changes — coupling them would mean a routine density-level change (`N`→`L`) forces every `use` site referencing that footprint to be rewritten. Keeping them as two separate fields (module-path name for resolution, `ipc_name` for descriptive identity) avoids this churn entirely, mirroring how `#[designator(...)]` (RFC-005) is a separate override field from an instance's own declared name.
- **Make **`ipc_name`** mandatory on every **`footprint`** declaration** — considered, rejected for this pass: CoHDL's non-goal scope (see Non-goals) deliberately excludes package families (connectors, relays, unusual mechanical parts) that don't fit the closed six-template set yet; making the field mandatory today would force library authors to fabricate a nonsensical `ipc_name` for those cases. Optional-with-checking-when-present is the honest scope boundary; revisit mandating it once the family-template set is proven broad enough in real use (see the companion decision record's Revisit criteria).
- **Full automatic **`ipc_name`** derivation from **`pad`** geometry (compiler computes the name, author never writes it)** — rejected as this RFC's scope: IPC-7351B's own package-family classification (is this a QFP or a QFN? which one's the "body" vs. "lead span"?) is not fully recoverable from bare pad-position data alone without also knowing the physical package type, which CoHDL has no independent source for today (no package-body concept exists yet — see Non-goals' JESD30 discussion). The author states the family and dimensions once (in the `ipc_name` string); the compiler's job is checking agreement, not inferring intent from geometry alone.

# Compatibility

Purely additive. `footprint`'s existing grammar (RFC-018) is unchanged in every way it was before this RFC; `ipc_name` is a new optional field. No existing `footprint` declaration (all placeholders, per RFC-017/018's own disclosed state) needs any change to keep compiling. `footprint_alias`'s existing `kicad:`/`lceda:`/`allegro:`/`default:` keys are unchanged; `ipc:` is a new, also-optional recognized key.

Depends on RFC-016 (module system — unchanged, this RFC adds no new resolution behavior) and RFC-018 (`pad`/`footprint` — this RFC's geometry cross-check reads the exact `pad N: ... at (x, y)` structure RFC-018 already defined) landing first, both already Accepted.

# Tooling & operations

- `cohdl lsp` (RFC-014) hover on a `footprint` declaration with an `ipc_name` present should surface the parsed family/pitch/span/density fields in human-readable form (e.g. "QFP, 0.5mm pitch, 9.00×9.00mm lead span, 1.60mm height, 48 pins, Nominal density") — the same "resolve and show more than what's literally written" precedent RFC-003's empty-`impl`-body hover and RFC-018's pad-symbol hover already established.
- `cohdl fmt` (RFC-009) needs one new formatting rule: `ipc_name: "..."` as a single-line field inside `footprint {}`, placed first (before `pad` entries), matching the existing convention of metadata-like fields (e.g. RFC-013's `#[placement_hint(...)]`) preceding structural content.
- New error codes in the existing **E8xx block** (designators & parts — the same block RFC-017/018's footprint/pad-count checks already live in, per RFC-011's "kind of mistake, not which pass" organizing principle): malformed `ipc_name` (unrecognized family prefix, missing density suffix, non-numeric dimension field — naming which), and `ipc_name`-vs-pad-geometry mismatch (pin count or pitch disagreement — naming the specific field and both values).
- `cohdl build --json`'s existing artifact-path pattern is unchanged; no new artifact is introduced by this RFC. A future tooling RFC could optionally add `ipc_name` as a carried component-attribute string in the IPC-2581 emitter (RFC-015) — not required by this RFC, named here only as a natural, non-blocking follow-up.

# Teaching cost

Low-Medium. The IPC-7351B naming grammar is a real, external, already-documented industry standard — an AI-context author or human reviewer already familiar with real PCB footprint libraries (KiCad, SnapEDA, Ultra Librarian all use IPC-7351-derived names) needs zero new vocabulary, only CoHDL's specific closed six-family-template subset and the `ipc_name:` field syntax itself (a single-line string field, same shape as every other string-valued field already in the language). Authors unfamiliar with IPC-7351 face a one-time cost learning the convention — but this cost exists in the real industry regardless of CoHDL, and CoHDL's structural check (Type-system-first test above) actively teaches correct usage by pointing at specific disagreements rather than silently accepting a wrong name.

# Failure modes

- **A **`footprint`**'s pad layout is genuinely irregular** (mixed pitch, non-perimeter pin arrangement, asymmetric exposed pad) — this RFC's geometry cross-check is explicitly scoped to regular rectangular-perimeter layouts (see Design); an irregular footprint with an `ipc_name` present gets grammar-well-formedness checking only, with geometry consistency un-checked and disclosed as such (a compiler note, not silently assumed verified) — the same "this RFC's checks don't cover every case" honesty RFC-018 already modeled for pad-dimension accuracy.
- `ipc_name`** correctly derived at authoring time, but the underlying **`pad`** geometry is later edited without updating **`ipc_name` — the geometry cross-check catches this at the next compile (a real, structural guarantee), but only for the regular-layout families this RFC covers; for irregular layouts (previous bullet), a stale `ipc_name` can silently drift from reality, same acknowledged risk RFC-018 already named for pad-dimension typos.
- **Two libraries' footprints for the same real-world package compute the same **`ipc_name`** by construction** (this is a deliberate, positive side effect of adopting a derivable standard rather than free-form names) **but still remain two distinct, non-deduplicated symbols** at two different module paths, per RFC-017's own already-disclosed non-goal — this RFC does not add deduplication; it only makes the fact that two footprints describe the same real thing *visible* (same `ipc_name` string, different symbols), which is strictly better than today's fully arbitrary names but does not solve cross-library canonicalization.
- **A package family CoHDL's closed six-template set doesn't cover** (e.g. connectors, relays, unusual mechanical parts) — `ipc_name` simply isn't usable for these yet; a library author either omits it (allowed, per Design) or the field is rejected with a diagnostic naming the unsupported family, never silently accepted as some other family's template.

# Migration path

N/A for existing footprints in the sense of a required change — `ipc_name` is optional and its absence changes nothing (see Compatibility). Real, non-mechanical adoption work for library authors who want the new discipline: for each existing (placeholder or real) `footprint` declaration, look up the part's actual package data and hand-derive its correct IPC-7351 name, same authoring effort class as RFC-018's own still-open pad-content migration (real geometry work, not part of this RFC's "ship with its check" completion bar). The two devices used to motivate this RFC (STM32F103C8T6's `LQFP48_7x7_P0_5`, RP2350A's `QFN60_7x7_P0_4_EP3_4`) are the first real worked examples and should be the first footprints in the std library to receive real `ipc_name` values once their `pad` content itself is authored.

# Decision

Accepted — 2026-07-16. Recorded as DR-027 (see note 7). Adds `ipc_name` as a new, optional, checked field on `footprint` (RFC-017/018's already-Accepted declaration kind) — no rename, no change to existing grammar. Adopts IPC-7351B's naming convention (not JEDEC JESD30, not an invented scheme) as CoHDL's canonical footprint-naming practice, scoped to a closed six-family-template set (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF) covering CoHDL's current real hardware. Also adds a recognized `ipc:` key to the pre-existing `footprint_alias` construct (Footprint Binding — Design note) for authors still on the alias/string-map path. Language Specification (note 10) gains a new subsection under "Footprints and pads," documenting `ipc_name`'s field syntax, the closed family-template grammar, and the two checks (well-formedness, geometry cross-check for regular layouts). Explicitly depends on RFC-016 and RFC-018 (both already Accepted) landing first — no new dependency introduced.
