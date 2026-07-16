# RFC-021: IPC-7351 as the canonical footprint naming practice

# Problem

RFC-018 gave `footprint` real, checkable pad content, but left one thing completely unconstrained: **the name** a library author gives a `footprint` declaration. Today `QFN10_3x3`, `qfn_10x10_thing`, `MyFootprint_v2` are all equally legal — the only rule is RFC-016's ordinary identifier grammar. Two different libraries authoring "the same" real-world package (already an acknowledged, un-deduplicated gap per RFC-017's Failure modes) currently have no shared convention to converge on even by accident, and an AI generating a new `footprint` declaration has no signal for what a *good* name looks like beyond "resolves, doesn't collide."

Research conducted alongside this RFC confirms the real industry already solved this problem, twice, at different layers: IPC-7351 (IPC-7351B currently, 7351C pending) is the industry-standard naming and land-pattern-sizing methodology for SMD footprints — a structured, derivable string encoding package family, pitch, lead/ball span, height, pin count, and density (Least/Nominal/Most). JEDEC JESD30 separately names the package outline (the mechanical body — QFN, LQFP, BGA, etc.) that IPC-7351 then turns into a land-pattern name.

Tony's directive, refined twice same day: (1) footprint declarations must be named in IPC-7351 form directly — not carry a separate descriptive field alongside an unconstrained symbol name. There is exactly one name per footprint, and that name is the thing that resolves through RFC-016's module system and the thing a human/AI reads to understand the land pattern's geometry. (2) CoHDL does not reference or track third-party CAD tool footprint names at all — no KiCad/LCEDA/Allegro backend-mapping construct is in scope. CoHDL's footprint/pad declarations (RFC-018) are CoHDL's own native geometry, authored and owned by CoHDL library authors — IPC-7351 is adopted purely as CoHDL's own naming convention for CoHDL's own footprints, not as a bridge to any external tool's library. This RFC does not touch, reference, or extend any third-party-footprint-mapping mechanism, and none is introduced.

Who this is for: **library authors** (the RFC-017 audience), who now have a concrete, non-arbitrary naming discipline instead of "pick anything that resolves"; and **AI generating footprint declarations**, which can derive a correct name mechanically from a package's own known dimensions instead of inventing one.

# Goals

- Adopt IPC-7351B's naming grammar as the required naming format for every footprint declaration's own identifier — the same name that RFC-016's module-path resolution (use path::Name;) already uses. One name, one job, no parallel field.
- Check every footprint name in two stages: (1) grammar well-formedness against a closed set of IPC-7351B family templates, and (2) geometry cross-check, for package families with a fully regular pad layout (rectangular-perimeter SMD pin arrays — QFP/QFN/SOIC/SOP/SOT, and analogously BGA/CHIP/MELF), confirming the name's encoded pin count and pitch agree with the footprint's own pad placements.
- Establish IPC-7351's package-family vocabulary (QFP, BGA, SOIC, SOT, QFN/SON, CHIP, MELF, ...) and density-level suffix (`N`/`L`/`M`) as a closed, documented set CoHDL's tooling understands structurally, so a malformed or internally-inconsistent footprint name is a real compile-time diagnostic, not silently accepted.
- Accept the real consequence of this choice honestly: because IPC-7351 names are mechanically derived from geometry, a footprint's name changes if its geometry changes (e.g. density level N→L), and every use site referencing it must follow. This RFC does not soften that with a second stable-name layer — see Alternatives for why.

# Non-goals

- Not IPC-7351's land-pattern calculation formulas (the pad-size-from-lead-dimension math, tolerance/density-level geometry derivation). This RFC adopts IPC-7351 as a naming and consistency-check discipline layered on top of RFC-018's already-existing hand-authored pad/footprint geometry — it does not make CoHDL compute pad sizes from datasheet lead dimensions. Library authors still hand-author pad geometry exactly as RFC-018 established; this RFC only constrains what the result is named and checks the name agrees with what was hand-authored.
- **Not JEDEC JESD30** (package-outline designators). IPC-7351 names the *land pattern*, JESD30 names the *package body* — this RFC scopes only the land-pattern-naming layer CoHDL actually owns (`footprint`), not a parallel package-designator system. A future RFC could adopt JESD30-style naming for `package`/`variants` (RFC-008) if a real need emerges; not proposed here.
- Not a third-party-footprint tracking or mapping mechanism. CoHDL does not reference, import, or maintain any mapping to KiCad/LCEDA/Allegro/other CAD tool footprint libraries. Every footprint in CoHDL is CoHDL's own native declaration (RFC-018's pad/footprint design) — this RFC's naming discipline applies solely to that native declaration's own identifier. There is no per-backend name table, alias, or cross-reference of any kind, and none is introduced by this RFC.
- Not full IPC-7351B grammar coverage on day one. This RFC covers the package families needed for CoHDL's own real, in-flight hardware (QFP/LQFP/TQFP, QFN/SON, SOIC/SOP, SOT, BGA, and passive CHIP/MELF two-terminal parts) — the closed set below — not IPC-7351B's full catalog (which also covers connectors, relays, and other families CoHDL has no current examples for). A footprint for a package family outside this closed set is out of this RFC's checked scope (see Failure modes) — not blocked, but not name-validated either, until the set is extended by a follow-up RFC, same discipline as RFC-001's closed unit-type set.
- Not automatic name generation from pad geometry. A library author writes the footprint's IPC-7351 name by hand (deriving it from the datasheet, same authoring effort as writing the pad placements themselves); the compiler checks it, it does not invent it.
- Not a stable-symbol-name layer decoupled from geometry. This RFC deliberately does not introduce a second, human-chosen identifier to shield use sites from a footprint's geometry-derived name changing — see Alternatives for the trade-off this accepts.

# Design

## IPC-7351B naming grammar, adopted as the closed structural format for footprint names

IPC-7351B's naming convention is a family-specific template, but every family shares the same broad shape: package-family prefix + pitch + span/body dimensions + (pin count) + density suffix. CoHDL requires every footprint declaration's own name to match one of the following closed family templates (the ones covering CoHDL's real current hardware; see Non-goals for scope):

| Family prefix | Meaning | Template |
|---|---|---|
| `QFP` | Quad flat pack (incl. LQFP/TQFP, same land-pattern shape) | `QFP` + pitch(P) + leadspan_X + `X` + leadspan_Y + `X` + height + `-` + pins + density |
| `QFN` | Quad flat no-lead (incl. SON, VQFN) | `QFN` + pins + density + pitch(P) + body_X + `X` + body_Y + [`-1EP` + epad_X + `X` + epad_Y, if exposed pad] |
| `SOIC` / `SOP` | Small-outline IC | `SOIC` or `SOP` + pins + `P` + pitch + `X` + leadspan + `X` + height + density |
| `SOT` | Small-outline transistor | `SOT` + pins + `P` + pitch + `X` + body_X + `X` + body_Y + density |
| `BGA` | Ball grid array | `BGA` + pins + (`C` | `N`) + pitch(P) + cols + `X` + rows + `_` + body_X + `X` + body_Y + `X` + height + density |
| `CHIP`/`MELF` | Two-terminal passives (resistors/caps) | `CHIP`/`MELF` + `-` + EIA size code (e.g. `0402`, `0603`) — density suffix not applicable |

- Pitch, span, body, height, and exposed-pad dimensions are encoded in hundredths of a millimeter, no decimal point, no unit suffix — this is IPC-7351B's own convention (e.g. 50 = 0.50mm pitch, 900 = 9.00mm), adopted verbatim rather than reinvented, so a footprint name is directly comparable against the real industry's published naming convention rather than a CoHDL-specific dialect — a naming-convention benefit only; CoHDL still never references any actual third-party footprint library or file.
- Density suffix: N (Nominal — default, used unless a library author has a specific reason to deviate), L (Least — smallest pads, dense designs), M (Most — largest pads, rugged/hand-assembly designs). Closed three-value set, same discipline as RFC-001's unit-type table and RFC-008's pin-role set — a name with no density suffix, or a suffix outside {N, L, M}, is a compile error for any footprint claiming one of the closed families above.
- Because CoHDL identifiers cannot start with a digit and IPC-7351 names sometimes would (rare in the closed families above, but possible for some CHIP/MELF variants) — the family prefix is always the first token, so this does not arise for any template in this closed set; no escaping mechanism is introduced.

## footprint's own name is the IPC-7351 name — no separate field, no third-party mapping

```cohdl
// sparkfun/src/footprints/qfn.cohdl → module path sparkfun::footprints::qfn

use sparkfun::pads::smd::Rect_0_3x0_9mm;

pub footprint QFN10N40P300X300_1EP180X180 {
    pad 1: Rect_0_3x0_9mm at (-1.5mm, 1.0mm)
    pad 2: Rect_0_3x0_9mm at (-1.5mm, 0.5mm)
    pad 3: Rect_0_3x0_9mm at (-1.5mm, 0.0mm)
    // ... one entry per pad
    courtyard { shape: rect, at: (0mm, 0mm), size: (3.5mm, 3.5mm) }
    silkscreen_ref { at: (0mm, -2.2mm) }
}
```

- The identifier after pub footprint is the IPC-7351B designator, with - mapped to _ (CoHDL identifiers can't contain -) — a single, fixed, documented substitution, not a free-form escaping scheme.
- When present, the name is checked in two stages, against the declaration's own identifier:
- A footprint whose package family falls outside the closed six-template set (see Non-goals) is not required to follow this naming discipline — its name is checked only against RFC-016's ordinary identifier grammar, unchanged from today. This is a real, disclosed scope boundary, not silent leniency (see Failure modes).
- This is the entirety of CoHDL's footprint-naming surface. There is no separate backend-mapping construct, no per-CAD-tool name table, no third-party footprint reference of any kind — footprint/pad (RFC-018) is CoHDL's own, sole geometry model, and the name discussed in this RFC is that declaration's own identifier and nothing else.

## Example: the two devices used to motivate this RFC

```cohdl
// STM32F103C8T6 — LQFP-48, 7x7mm body, 0.5mm pitch, nominal density
pub footprint QFP50P900X900X160_48N {
    pad 1: Rect_0_3x1_5mm at (-4.5mm, 3.75mm)
    // ... 47 more pads
}

// RP2350A — QFN-60, 7x7mm body, 0.4mm pitch, exposed pad, nominal density
pub footprint QFN60N40P700X700_1EP340X340 {
    pad 1: Rect_0_2x0_75mm at (-3.35mm, 3.05mm)
    // ... 59 more pads
    // exposed thermal pad modeled as a pad entry on `through_all`/`smd` per RFC-018's existing vocabulary
}
```

These two are exactly the pair used to validate this RFC's naming derivation against real datasheets before drafting — the derivation used real package dimensions (pitch, body size, pin count, density) as reference data only; the resulting footprint declarations are CoHDL-native geometry, not references to or copies of any third-party CAD library's footprint files.

# Type-system-first test

Both checks this RFC introduces are structural and local to one `footprint` declaration's own already-written content — never DRC candidates:

1. Name grammar well-formedness — checkable the moment the declaration's identifier is parsed, against a fixed, closed grammar table (one of six family templates). Exactly the same shape as RFC-001's unit-literal grammar check.
2. Name-vs-pad-geometry consistency — checkable entirely from one footprint declaration's own pad N: ... at (x, y) list (pin count = number of pad entries; pitch = the regular spacing between them, when the layout is a uniform rectangular perimeter). No cross-declaration or cross-design lookup needed, so this is a compile-time (type-system) check, not DRC — consistent with RFC-018's own precedent for the pad-count-vs-device-pins check.

# Conceptual impact

Low. No new core concept, no new field — this RFC constrains the identifier grammar of an already-Accepted declaration kind (footprint, RFC-017/018), the same class of move RFC-008 made when it retired the implicit pin-role default in favor of a closed, checked vocabulary. The one real conceptual cost: footprint's name is no longer a free-form author choice — it is now derived, checkable data, coupling identifier stability to physical geometry stability for the closed families this RFC covers (see Alternatives for the trade-off accepted here).

# Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Med | Low | Med | Low | Low | Med |

Grammar (Med): footprint's identifier grammar is now constrained against a closed six-template family grammar (for the families this RFC covers) — a real, bounded grammar addition on top of RFC-016's existing identifier rules, not a rename or reshaping of anything else.

**Oracle (Low)**: no new DRC surface — both checks are type-system (see Type-system-first test above), resolved entirely from one declaration's own already-declared content.

Diagnostics (Med): two new, specific failure classes (malformed IPC-7351 name, name-vs-geometry mismatch), each naming the specific field that disagrees — new E8xx sub-cases (see Tooling & operations), not a new block.

Netlist (Low): the footprint's name is not itself emitted geometry; .kicad_mod/IPC-2581 pad-geometry projection (RFC-018) is unaffected by this RFC. A footprint's IPC-7351-derived name may incidentally read well as a component-attribute string in IPC-2581 output, but this RFC does not require any emitter change.

Compat (Med): this RFC constrains an existing declaration's own identifier — any future footprint written for one of the six closed families must be named per this discipline from the start (checked at declaration time), and a geometry change to an existing named footprint (e.g. changing density level) requires a name change and every use site to follow, honestly disclosed as real, ongoing churn risk (see Failure modes), not a one-time migration cost.

**Trust (Med)**: this is the mechanism that makes "the footprint's name lies about its own geometry" structurally impossible for the regular-layout families it covers — a real, new trust guarantee, though scoped to the geometry-regular families only (see Failure modes for what it does not cover).

# Gradeability

- Name grammar well-formedness is checked the moment the footprint declaration's identifier is parsed and type-checked — declaration time, the earliest possible stage, identical timing to RFC-018's pad internal-consistency check (drill:/plating: agreement).
- The geometry cross-check runs once the same footprint declaration's full pad list is visible — also at declaration time (not deferred to cohdl build, since it needs no external reference the way RFC-018's pad-count-vs-device check does — the device isn't involved here, only the footprint's own internal pad list vs. its own name). This is strictly earlier than RFC-018's device-cross-check, and thus strictly stronger gradeability for the part it covers.
- Diagnostics name the specific mismatched field (e.g. "footprint name QFN10N40P300X300 declares 10 pins, footprint declares 12 pad entries" or "footprint name declares 40 (0.40mm) pitch, footprint's pad spacing is 50 (0.50mm)"), following the same specific-mismatch-naming discipline RFC-018 established for pad-count/numbering diagnostics.
- A renamed footprint (e.g. density level corrected from N to L after re-deriving pad sizes) immediately breaks every stale use site referencing the old name at the earliest possible point (RFC-016's existing unresolved-name diagnostic) — the same "fail fast, fail specifically" property this RFC's other checks have, applied to the rename-churn trade-off this RFC accepts (see Alternatives).

# AI-generatability

High for the mechanical part: once a device's package family, pitch, span, height, pin count, and density level are known (all already needed to hand-author the pad placements themselves, per RFC-018's own honest Medium AI-generatability assessment), deriving the IPC-7351 name to use as the footprint declaration's own identifier is a pure, deterministic string-formatting function over data the model already has to gather — no new judgment call, no new datasheet lookup beyond what RFC-018 already required. This is a genuine generatability win over the status quo (an arbitrary, unconstrained footprint symbol name): a model no longer has to invent a name convention per-library, it has exactly one correct answer to compute, and that answer is now the only name it needs to track (no second field to keep synchronized, no third-party footprint library to look up). The geometry cross-check (Type-system-first test above) means a model that gets the derivation wrong is told immediately and specifically, rather than shipping a footprint whose name silently disagrees with its actual copper.

# Alternatives

- A separate, optional ipc_name field alongside an unconstrained free-form symbol name — this RFC's own first draft, explicitly reconsidered and rejected per Tony's direct correction: two names for one thing (a stable author-chosen symbol for resolution, a derived IPC-7351 string for description) is exactly the kind of "two ways to express/identify the same thing" duplication the project avoids everywhere else (cf. RFC-016's single-name-per-use rule, RFC-008's retirement of implicit pin-role defaults). A footprint has one identity; it should have one name.
- Maintain a footprint_alias-style backend-mapping construct carrying third-party CAD tool footprint names (KiCad/LCEDA/Allegro) alongside the IPC-7351 name — considered and rejected per Tony's direct correction: CoHDL does not track or care about third-party footprint names at all. CoHDL's footprint/pad declarations are its own native geometry model (RFC-018) — there is nothing for CoHDL to reconcile against an external tool's own library naming, and introducing such a mapping would be tracking data CoHDL has no use for and no way to keep correct (no mechanism validates that a claimed KiCad library reference actually matches anything real). If a future integration genuinely needs to export to or reference a third-party footprint library, that is separate, not-yet-proposed scope — this RFC does not anticipate or partially build toward it.
- **Adopt JEDEC JESD30 instead of, or alongside, IPC-7351** — considered, rejected for `footprint` specifically: JESD30 names the package *body* (the mechanical outline), not the *land pattern* (the copper geometry `footprint`/`pad` actually declare) — the wrong layer for what this RFC's construct owns. JESD30-style naming remains a legitimate candidate for a future RFC scoping `package`/`variants` (RFC-008) naming, not proposed here (see Non-goals).
- Invent a CoHDL-native naming scheme instead of adopting an existing standard — rejected per Tony's explicit direction and per the same reasoning RFC-018 already applied when it rejected copad/cofp in favor of plain English names: inventing new vocabulary when a well-understood, industry-proven convention already exists and does the job is unnecessary teaching cost with no compensating benefit.
- Mandate IPC-7351 naming for every footprint, regardless of package family — considered, rejected for this pass: CoHDL's non-goal scope (see Non-goals) deliberately excludes package families (connectors, relays, unusual mechanical parts) that don't fit the closed six-template set yet; mandating it universally today would force library authors to fabricate a nonsensical name for those cases. Scoping the requirement to the closed family set, with an honest disclosed gap for everything else, is the honest scope boundary (see Failure modes); revisit extending it once the family-template set is proven broad enough in real use (see the companion decision record's Revisit criteria).
- Full automatic name derivation from pad geometry (compiler computes the name, author never writes it) — rejected as this RFC's scope: IPC-7351B's own package-family classification (is this a QFP or a QFN? which one's the "body" vs. "lead span"?) is not fully recoverable from bare pad-position data alone without also knowing the physical package type, which CoHDL has no independent source for today (no package-body concept exists yet — see Non-goals' JESD30 discussion). The author states the family and dimensions once (by writing the footprint's name); the compiler's job is checking agreement, not inferring intent from geometry alone.

# Compatibility

Real, disclosed, ongoing constraint on footprint naming for the closed family set (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF): every new footprint declaration for one of these families must be named per the IPC-7351 discipline from the moment it's written (checked at declaration time). Existing footprint declarations for these families that don't yet follow this naming (all still placeholders per RFC-017/018's own disclosed migration state) need to be renamed to comply — a real, non-mechanical migration item (see Migration path), not silently grandfathered. footprint declarations for package families outside the closed set are unaffected — their names are checked only against RFC-016's ordinary identifier grammar, exactly as today.

No third-party-footprint-mapping construct exists in CoHDL before or after this RFC — there is nothing else to migrate or reconcile.

Depends on RFC-016 (module system — unchanged, this RFC adds no new resolution mechanism, only a stricter identifier-grammar constraint) and RFC-018 (pad/footprint — this RFC's geometry cross-check reads the exact pad N: ... at (x, y) structure RFC-018 already defined) landing first, both already Accepted.

# Tooling & operations

- cohdl lsp (RFC-014) hover on a footprint declaration whose name matches one of the closed family templates should surface the parsed family/pitch/span/density fields in human-readable form (e.g. "QFP, 0.5mm pitch, 9.00×9.00mm lead span, 1.60mm height, 48 pins, Nominal density") — the same "resolve and show more than what's literally written" precedent RFC-003's empty-impl-body hover and RFC-018's pad-symbol hover already established.
- cohdl lsp's rename-refactor support (if/when a future RFC adds one) should treat a footprint rename exactly like any other symbol rename — updating every use site — since this RFC's naming discipline makes renames (following a geometry change) a real, expected event rather than a rare one.
- New error codes in the existing E8xx block (designators & parts — the same block RFC-017/018's footprint/pad-count checks already live in, per RFC-011's "kind of mistake, not which pass" organizing principle): malformed IPC-7351 footprint name (unrecognized family prefix, missing density suffix, non-numeric dimension field — naming which), and name-vs-pad-geometry mismatch (pin count or pitch disagreement — naming the specific field and both values).
- cohdl fmt (RFC-009) is unaffected — no new field, no new formatting rule; footprint's existing declaration-header formatting is unchanged.
- cohdl build --json's existing artifact-path pattern is unchanged; no new artifact is introduced by this RFC.

# Teaching cost

Low-Medium. The IPC-7351B naming grammar is a real, external, already-documented industry standard, but the only thing an author needs to internalize is CoHDL's own closed six-family-template subset and the fact that a footprint's own name (not a side field, not a third-party mapping) must comply. Authors unfamiliar with IPC-7351 face a one-time cost learning the convention, but CoHDL's structural check (Type-system-first test above) actively teaches correct usage by pointing at specific disagreements rather than silently accepting a wrong name. The one added teaching point relative to an unconstrained-name design: authors must understand that renaming a footprint (after a geometry change) is expected, and that every referencing use site needs updating — a real, disclosed cost, not hidden.

# Failure modes

- A footprint's pad layout is genuinely irregular (mixed pitch, non-perimeter pin arrangement, asymmetric exposed pad) — this RFC's geometry cross-check is explicitly scoped to regular rectangular-perimeter layouts (see Design); an irregular footprint whose name matches one of the closed family templates still gets grammar-well-formedness checking, but geometry consistency is un-checked and disclosed as such (a compiler note, not silently assumed verified) — the same "this RFC's checks don't cover every case" honesty RFC-018 already modeled for pad-dimension accuracy.
- A footprint's geometry is edited (e.g. a pad's size/offset corrected) without renaming the footprint to match — for the regular-layout families this RFC covers, the geometry cross-check catches this at the next compile if the edit changes the derivable pin count/pitch (a real, structural guarantee); a change that doesn't affect pin count/pitch (e.g. a pad-size-only correction within the same density level) is not caught, since IPC-7351 names encode pitch/span/pins/density, not individual pad dimensions — the same acknowledged limitation RFC-018 already named for pad-dimension typos, now inherited by this RFC's naming check.
- Every use site referencing a footprint must be updated when its name changes (e.g. density level N→L after a re-derivation) — this is the direct, accepted cost of Alternatives' rejected "separate stable name" option. RFC-016's existing unresolved-name diagnostic catches every stale reference immediately and specifically, but this RFC does not reduce the number of call sites that need editing — a real, ongoing authoring cost for footprints whose geometry is still being tuned, disclosed here rather than hidden behind a stable-name layer.
- Two libraries' footprints for the same real-world package land on the same name by construction (a deliberate, positive side effect of a derivable standard) but still remain two distinct declarations at two different module paths — since CoHDL's module system scopes names per-package (RFC-016), this is not a collision (different fully-qualified paths), but it does not solve cross-library canonicalization either — this RFC only makes the fact that two footprints describe the same real thing visible (identical name, different paths), per RFC-017's own already-disclosed non-goal.
- A package family CoHDL's closed six-template set doesn't cover (e.g. connectors, relays, unusual mechanical parts) — such a footprint's name is checked only against RFC-016's ordinary identifier grammar; no IPC-7351 discipline is enforced or expected until the set is extended by a follow-up RFC. This is a real, disclosed scope boundary, not silent leniency.

# Migration path

Real, non-mechanical migration work: every existing footprint declaration (all still placeholders per RFC-017/018's own disclosed migration state) that belongs to one of the six closed families must be renamed to its correct IPC-7351 designator once its real pad content is authored — this is authoring work of the same class as RFC-018's own still-open pad-content migration, not a mechanical find-and-replace, since it requires knowing the part's actual package dimensions. Because no real footprint content exists anywhere yet (RFC-017/018's own disclosed state), there is no live use site actually depending on a non-compliant name today — this migration has zero real breaking blast radius yet, but is a required naming discipline for every footprint authored from this RFC forward. The two devices used to motivate this RFC (STM32F103C8T6 → QFP50P900X900X160_48N, RP2350A → QFN60N40P700X700_1EP340X340) are the first real worked examples and should be the first footprints in the std library to receive these names once their pad content itself is authored.

# Decision

Accepted (revised twice, same day) — 2026-07-16. Recorded as DR-027 (revised twice — see note 7). First revision: superseded this RFC's own same-day original draft, which added a separate, optional ipc_name field alongside an unconstrained footprint symbol name — per Tony's direct correction, the footprint declaration's own identifier (the same name RFC-016's module system resolves) must comply with IPC-7351B naming directly, no separate field. Second revision, same day: Tony directly corrected a footprint_alias-style third-party-backend-name reference that had been included — CoHDL does not track, map, or care about third-party CAD tool footprint names at all; every footprint is CoHDL's own native geometry (RFC-018), and this RFC's naming discipline applies solely to that declaration's own identifier, with no other construct introduced or touched. Final scope: for a closed six-family-template set (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF) covering CoHDL's current real hardware, footprint's own name is checked for grammar well-formedness always, and cross-checked against the footprint's own pad geometry (pin count/pitch) where the layout is regular. Language Specification (note 10) reflects this final design. Explicitly depends on RFC-016 and RFC-018 (both already Accepted) landing first — no new dependency introduced.
