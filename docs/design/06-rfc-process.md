# 6. Feature Proposal Process (RFC)

# Status

v2 — process carried forward, backlog reset, 2026-07-13. The RFC mechanism itself (why we have it, when it's required, the lifecycle) is exactly as effective as it was in v1 — this redesign didn't touch how we govern change, only what the change was. What's new: one mandatory template section, and a completely fresh backlog (the v1 RFC-001…010 backlog described bugs/features in an implementation that no longer exists).

# Purpose — unchanged

Major features go through a structured proposal, not a ticket. The one question every RFC must answer:

> Does this feature strengthen CoHDL's coherence — its ability to be reliably generated, gradeable, and trusted — or is it a local patch?
>

Small fixes don't need an RFC. An RFC is required when a change touches a core concept, the grammar, the oracle/verdict, the diagnostic/error-code contract, or scores High/Crit on any Coherence Matrix dimension.

# RFC template — one new mandatory section (marked NEW)

```text
# RFC-NNNN: [short title]

## Problem
What real problem does this solve? For whom (AI author / human reviewer / tool builder)?

## Goals
Which product goals + priority-ladder ranks does it serve?

## Non-goals
What does it deliberately NOT solve?

## Design
Proposed behavior, syntax, and/or IR change. Show example .cohdl source.

## Type-system-first test (NEW, mandatory if proposing a `rule`/DRC check)
Could this be expressed as a trait bound, a required spec, a pin obligation,
or another type-system mechanism instead of a `rule`? If not, why not —
specifically, what makes it emergent/cross-cutting/numeric rather than
structural? A `rule` proposal that skips this section is incomplete.

## Conceptual impact
Does it add/change a core concept? (If yes: justify the permanent cost.)
Guard the canonical vocabulary — does any name overlap an existing concept?

## Coherence matrix row
| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
Fill each cell Low/Med/High/Crit, then address every High/Crit.

## Gradeability
How does the compiler CHECK this? Which stage/rule/code enforces "correct"?
Prefer the earliest possible stage (type-check over DRC over review).

## AI-generatability
Can a model emit this without memorizing a special case? Is meaning local?

## Alternatives
What else was considered? Why not a `fn` / existing trait / existing rule?

## Compatibility
Breaks existing source, error codes, designators, or netlist bytes?
Deprecation cycle needed? (N/A until v2's first stable release ships.)

## Tooling & operations
What must be observable, formattable, diffable, reversible?
Does it change the `check --json` / error-code public surface?

## Teaching cost
What must an AI-context author + a human reviewer now learn?

## Failure modes
How can this be misused, misgenerated, or misunderstood?

## Migration path
How do existing designs move into the new model? (N/A pre-launch.)

## Decision
Accepted / Rejected / Delayed / Experimental — with date + link to decision record.
```

# The lifecycle of an RFC — unchanged, one step added

1. Draft — author fills the template. Incomplete gradeability, coherence, or (new) type-system-first sections = not ready.
2. Review against the ladder — reconcile with the Constitution's priority ladder and note 4's constraints. A rank-1/rank-2 violation is a blocker.
3. **Coherence check** — run the design regression checklist (note 8) and the matrix row (note 5).
4. **Decision** — Accepted / Rejected / Delayed / Experimental. Every accepted or notably-rejected RFC gets a decision record (note 7).
5. Ship with its check — an accepted correctness-affecting RFC ships together with the mechanism that enforces it (type checker rule, or narrowly, a DRC rule). No "we'll wire it later" — this is the single most important lesson carried forward from v1's postmortem.
6. (NEW) Ship with its spec update — an Accepted RFC is not done until note 10 (Language Specification) reflects it: new syntax gets a reference entry, changed semantics get the existing entry updated in place. The spec book is the single compiled statement of "what the language currently is" — RFCs and decision records are the history/rationale (the git log); the spec book is HEAD. An RFC whose Decision is "Accepted" but whose construct never appears in note 10 is exactly as incomplete as one that shipped without its compiler check — don't let the discipline that fixed v1's dormant-DRC problem quietly not apply to documentation.

# RFC backlog (v2 — reset from zero)

The v1 backlog (RFC-001…010) described defects and features in a discarded implementation; none of it carries over as-is. The new backlog is the actual redesign's open design questions — each of these needs a full RFC pass before implementation starts, because each is a Layer-1 decision that everything else depends on.

| RFC | Title | Priority | Why |
|---|---|---|---|
| RFC-001 | Units-as-types: the primitive unit type system (`Farads`, `Voltage`, `Ohms`, `Hertz`, …) and coercion rules — **Accepted 2026-07-13**, see child note RFC-001 + DR-007 | **P0 — done** | The foundational strictness mechanism; every spec, every device, every trait bound depends on this existing first |
| RFC-002 | Pin connection-obligation typing (`required` / `optional` / `nc`) and its exhaustiveness check — **Accepted 2026-07-13**, see child note RFC-002 + DR-012 | **P0 — done** | The direct type-level fix for v1's "silently unconnected pin" failure class |
| RFC-003 | Trait-satisfaction-at-`impl`-time checking rules — **Accepted 2026-07-13**, see child note RFC-003 + DR-013 | **P0 — done** | The direct type-level fix for v1's dormant E004/E005; must be designed before any std-library trait is written |
| RFC-004 | Classification pass: for each v1-era DRC concern (E001–E005, W001–W004 equivalents), decide "becomes a type mechanism" vs. "stays narrowed residual DRC" — **Accepted 2026-07-13**, see child note RFC-004 + DR-014 | **P0 — done** | Mandated by the Coherence Matrix's mitigation note — narrowing DRC without this is how a safety net silently disappears |
| RFC-005 | Designator allocator design: a provably collision-free, total/injective function over hierarchical path — **Accepted 2026-07-13**, see child note RFC-005 + DR-008 | **P0 — done** | Replaces v1's incrementing-counter allocator that collided; must be proven correct by construction, not tested after the fact |
| RFC-006 | Nested `fn` call semantics (monomorphization + inlining across call depth) — **Accepted 2026-07-13**, see child note RFC-006 + DR-015 | **P0 — done** | Must be first-class from v2 launch, not a P1 patch — composability depends on it |
| RFC-007 | Generics-over-specs syntax + visible-default rules — **Accepted 2026-07-13**, see child note RFC-007 + DR-016 | **P0 — done** | The expressiveness half of the core bet; must land alongside RFC-001–003, not after |
| RFC-008 | Grammar design for exhaustive pattern-matching over structural variants — **Accepted 2026-07-13**, see child note RFC-008 + DR-017 | **P1 — done** | Needed once devices/packages have enough variants to require it; can follow the P0 core |
| RFC-009 | `cohdl fmt` canonical form, co-designed with the grammar — **Accepted 2026-07-13**, see child note RFC-009 + DR-018 | **P1 — done** | Ship-from-launch requirement per the Constitution; design the form as the grammar is finalized, not after |
| RFC-010 | `cohdl check --json` schema, co-designed with the type checker's diagnostic shape — **Accepted 2026-07-13**, see child note RFC-010 + DR-019 | **P1 — done** | Same co-design discipline as RFC-009 |
| RFC-011 | Error-code registry (v2 baseline, informed by RFC-004's classification pass) — **Accepted 2026-07-13**, see child note RFC-011 + DR-009 | **P1 — done** | Depends on RFC-004 landing first — can't finalize codes until we know what moved from DRC to type-checking |
| RFC-012 | `#[intent(...)]` annotations (pure metadata) — **Accepted 2026-07-13**, see child note RFC-012 + DR-010 | **P2 — done** | Gated on zero-netlist-impact, same as v1; not urgent for the redesign's core bet |
| RFC-013 | Layout-constraint concept (the door) — **Accepted 2026-07-13**, see child note RFC-013 + DR-011. Gate lifted via GC-002's amendment (note 8), Tony's explicit decision to open the layout door ahead of its original "concrete partner requirement" trigger. | **Done** | Was gated pending a goal-change proposal — GC-002 amended and Accepted the same day; note 2's pre-designed seam ("net_class/constraint decoration adjacent to Net/Rule") is now exercised for the first time |

Milestone (2026-07-13, third time this day): the entire backlog — RFC-001 through RFC-013 — is now Accepted. RFC-013 is the first genuinely new core concept (Layout Constraint) added since the ground-up redesign began, and the first RFC whose enabling goal-change (GC-002) was decided same-day rather than inherited pre-accepted. Its constraint vocabulary is explicitly flagged provisional (note 8, GC-002 amendment) pending a real partner layout-tool integration — this is disclosed design debt, not a hidden gap.

Milestone (2026-07-13, fourth time this day): with the MVP implementation confirmed complete (131 passing tests, real `cohdl fmt`/`--json`/`layout.json` in the actual repository), **RFC-014 (LSP support) is Accepted**, closing a gap RFC-003's own DR-013 had explicitly flagged as needed back when the redesign began ("find all impls" navigation, hover-on-empty-`impl`). This is the first RFC to accept a scoped external dependency (`lsp-types`) as a justified exception to the project's hand-rolled-everything style — the transport loop stays hand-rolled; only the externally-versioned protocol's own message shapes are borrowed. Backlog is now RFC-001 through RFC-014, all Accepted.

Milestone (2026-07-14): with real implementation catch-up confirmed against the repo (146 tests, RFC-008/011/012/013 all now genuinely implemented, not just designed), **RFC-015 (IPC-2581 codegen backend)** is Accepted — the concrete next step in the layout-partner integration RFC-013 opened, grounded in the workspace's own pre-existing "Quilter as a CoHDL Backend Partner — Fit Analysis" research rather than fresh speculation. Explicitly scoped as an honestly-partial "logical-complete, physical-minimal" phase one — footprint-geometry resolution and board-outline/stackup support are named future work, not silently assumed solved. Backlog is now RFC-001 through RFC-015, all Accepted.

Milestone (2026-07-14, second time this day): Tony directed a centralized library registry to lower the cost of getting started — source, documents, skills, and footprints. Research surfaced a real fact worth recording: no open, portable footprint file standard exists industry-wide (IPC-7351 is a naming/calculation methodology, not a file format). This triggered two RFCs: RFC-016 (the module system CoHDL never had, now a real prerequisite) and RFC-017 (the registry itself, with a new native footprint format per explicit decision, skills deferred to its own future RFC). Backlog is now RFC-001 through RFC-017, all Accepted.

Milestone (2026-07-14, third time this day): Tony directly corrected RFC-017's footprint design, same day as its acceptance — a footprint must be a named, resolvable symbol under RFC-016's module system (so libraries can reuse each other's footprints by reference, not just by copying a file path), and the footprint format itself is out of scope for RFC-017, deferred to a future, separately-numbered RFC. RFC-017 is revised: footprint becomes a fifth top-level declaration kind (peer of device/trait/fn/part), resolved entirely through RFC-016's existing machinery, with its internal content left unspecified — "symbol-resolution-complete, format-empty," the same honest-phasing discipline RFC-015 established for IPC-2581. DR-023 gains a same-day amendment recording this correction rather than being silently rewritten.

Milestone (2026-07-14, fourth time this day): Tony directed adopting Cadence Allegro's proven pad/footprint split for the footprint format RFC-017 deferred. RFC-018 (Footprint format — pad/footprint) is Accepted, closing that gap: pad is a new, reusable pad primitive (shape/size/layer/plating); footprint — RFC-017's already-Accepted declaration kind — gains real, checkable content for the first time (composed of pad references placed at offsets, plus courtyard/silkscreen-reference geometry). Both resolve through RFC-016's existing module system — no new resolution mechanism, only new content. This directly closes RFC-017's own deferred pad-count/numbering consistency check and RFC-015's named future-work item (footprint-geometry resolution) — cohdl build finally has real geometry to project into .kicad_mod/IPC-2581. (Same-day correction: the initial draft used invented names copad/cofp; Tony corrected to plain pad/footprint — no keyword rename was actually needed, since footprint already existed from RFC-017.) Backlog is now RFC-001 through RFC-018, all Accepted.

Milestone (2026-07-15): with real implementation catch-up confirmed against the repo (RFC-016/017/018 all now genuinely implemented in source — src/lsp.rs, src/check/footprints.rs, src/emit/kicad_mod.rs, std/pads.cohdl, std/footprints.cohdl, real tests/modules.rs/tests/library.rs/tests/footprint.rs — not just designed), Tony directed the next RFC: a real VS Code extension. RFC-019 (VS Code extension for CoHDL) is Accepted, closing RFC-014's own explicitly-deferred packaging scope ("a full marketplace extension (grammar, packaging) is separate scope per the RFC") and its still-open real-client acceptance item (docs/lsp.md's own flag: "a pass in a live VS Code session has not yet been recorded"). Zero new diagnostic logic — a thin packaging + hand-authored TextMate grammar layer over the already-Accepted, already-tested cohdl lsp. New in-repo directory editors/vscode/. Backlog is now RFC-001 through RFC-019, all Accepted.

Milestone (2026-07-16): Tony reviewed the real implementation and found board_outline/place (a { at, size } rectangle board outline and place at (x, y) locked positioning) had been implemented directly on main (commits 86165d9, 1a0ce5f) with no RFC — the code's own comments admitted this ("pragmatic extension... pending an RFC"). Beyond the process violation, Tony identified two real design defects: a board outline is a mechanical-engineering artifact (a DXF file from a mechanical engineer, not a CoHDL-authored rectangle — confirmed as standard industry practice across Altium/Cadence/OrCAD/EasyEDA), and placement needs rotation (the actual root cause of a real observed Quilter failure — a board-edge connector rotated 90° from its intended orientation, which coordinate-only place had no way to express). RFC-020 (Board outline scoped DXF profile extraction + oriented placement) is Accepted, correcting both defects — revised twice further, same day, per Tony's continued direct review: first, "reference the DXF, never parse it" was corrected once Tony traced the real requirement — IPC-2581's Profile element needs closed polygon/arc geometry embedded in the document itself, so CoHDL must extract (narrowly — one designated outline entity, nothing else in the file) real geometry from the referenced DXF, not merely point at it. Second, Tony caught that place could not reach an instance declared inside a called fn at all (confirmed against real source); rather than design a path-qualification mechanism speculatively, Tony's direct call was to defer it — place supports only top-level instances for now, with the gap named honestly in the RFC and in note 10's "Not yet specified" list, not silently worked around. Real migration work required: rpi-pico2 needs an actual DXF board outline, correctly tagged on the documented convention layer, before this RFC is considered landed for that example. This is the project's first instance of correcting, not merely documenting, an unauthorized implementation — a process point recorded in DR-026 (plus a same-day amendment) for future reference. Backlog is now RFC-001 through RFC-020, all Accepted.

Milestone (2026-07-16, second time this day): following footprint-naming research (STM32F103C8T6/RP2350A worked examples, cross-checked against IPC-7351B, JEDEC JESD30, and real package datasheet dimensions), Tony directed adopting IPC-7351 — not JEDEC JESD30, not an invented scheme — as CoHDL's canonical footprint naming practice. RFC-021 (IPC-7351 as the canonical footprint naming practice) is Accepted, then revised twice same day per Tony's direct corrections: first, the initial draft added a separate, optional ipc_name field alongside an unconstrained footprint symbol name — Tony rejected the two-names-for-one-thing shape outright, requiring instead that the footprint declaration's own identifier (the same name RFC-016's module system resolves) comply with IPC-7351B naming directly, for a closed six-family-template set (QFP, QFN/SON, SOIC/SOP, SOT, BGA, CHIP/MELF). Second, Tony corrected a footprint_alias-style third-party-backend-name reference that had crept in — CoHDL does not track or care about third-party CAD tool (KiCad/LCEDA/Allegro) footprint names at all; every footprint is CoHDL's own native geometry (RFC-018), and this RFC's naming discipline applies solely to that declaration's own identifier, with no other construct introduced. Names are checked for grammar well-formedness always, and cross-checked against the footprint's own pad placements (pin count/pitch) where the layout is geometrically regular — closing the same class of "the name lies" gap RFC-018 closed for "the footprint lies." Real, disclosed trade-off accepted: because IPC-7351 names are geometry-derived, a footprint's name (and every use site referencing it) must change if its geometry changes — no stable-name layer was added to soften this. Backlog is now RFC-001 through RFC-021, all Accepted.

Note the shift in what's P0: in v1, P0 was "fix what's broken in a working compiler." In v2, P0 is "settle the type-system mechanisms that make the whole redesign's thesis true" — there's no partial credit for shipping Layer 3/4 work before these seven land, because nothing downstream can be honest until they do.

Milestone (2026-07-13): all seven P0 RFCs are Accepted. The Layer-1 type-system foundation — units-as-types, pin obligations, trait satisfaction, the DRC/type-system reclassification, a collision-free designator allocator, nested fn semantics, and generics-over-specs — is now formally specified end to end. RFC-004's flagged E004 dependency is closed by RFC-007. The redesign's central thesis (strictness buys expressiveness) has a complete Layer-1 specification to stand on. Remaining backlog (RFC-008–012) is P1/P2 tooling and polish; RFC-013 (layout) stays gated per its own governance. This is the natural point to revisit 9. MVP Definition, which was intentionally left void until this moment.

Milestone (2026-07-13, same day): the MVP is implemented and verified on the real conol-ai/cohdl main branch — 65 passing tests, a self-audited compliance report (docs/compliance-report.md, 8 independent audit agents + adversarial verification, 7 real deviations found and 6 fixed same-day), and an end-to-end demo with a real KiCad-imported netlist. Implementation surfaced two real, concrete needs (a closed-set pin-role default and unspecified package variants) that became RFC-008, drafted and Accepted the same day.

# The principle behind the process — unchanged

The RFC exists so that CoHDL's growth is explicit, traceable, and system-aware. A feature that can't fill the gradeability, coherence, and (now) type-system-first sections honestly is a feature CoHDL shouldn't ship.
