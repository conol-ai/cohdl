# RFC-020: Board outline (scoped DXF profile extraction) + oriented placement

## Status

Revised twice, same day, per Tony's direct corrections.

1. First correction: the original unauthorized implementation's rectangle-authoring board_outline and coordinate-only place were wrong on the merits (a board outline is a mechanical-engineering DXF artifact; placement needs rotation) — fixed in the first revision.
2. Second correction: that first revision's "reference the DXF, never parse it" stance for board_outline doesn't survive contact with the real requirement — IPC-2581's Profile element needs closed polygon/arc geometry embedded in the document, not a pointer to an external file, so CoHDL must extract (narrowly) the outline entity. This revision fixes that.
3. This same second review also surfaced that place cannot reach an instance declared inside a called fn (confirmed against real source: place resolves only against a scope's local top-level names). Tony's direct call: defer this — place supports top-level instances only, for now. Not solved by a new path-resolution mechanism; named as an explicit, disclosed gap instead, consistent with the project's own repeated practice of deferring a mechanism until a real concrete need proves it's worth the cost (RFC-007's const-generics rejection, RFC-018's padstack-richness deferral, this same RFC's own closed-rotation-set decision).

## Problem

board_outline { at: (cx, cy), size: (w, h) } and place <inst> at (x, y) were implemented directly on main (commits 86165d9, 1a0ce5f) with no RFC. Beyond the process violation, the shape itself had two real defects, both now corrected: a board outline is a mechanical-engineering artifact (a DXF file), not a CoHDL-authored rectangle; and placement needs rotation, not just coordinates — the actual cause of a real observed Quilter failure (a board-edge connector rotated 90° from its intended orientation).

## Goals

- Board outline: extract, narrowly, one designated outline entity from a referenced DXF file and embed its real geometry (a closed polygon, with straight segments and/or arcs) into IPC-2581's native Profile/Polygon element and into layout.json — this is what actually makes the emitted document Quilter-importable. Everything else in the DXF (other layers, entities, text, dimensions) is never read.
- place gains an optional rotate clause, restricted to a closed four-value set (0/90/180/270) — not an open-ended angle type — following the same closed-vocabulary discipline as RFC-001's units and RFC-008's pin roles.
- place continues to name only top-level design instances — unchanged from the original construct's own restriction. A component instantiated inside a called fn cannot currently be placed; this is a real, known gap, explicitly deferred (see Non-goals), not solved by this RFC.
- Retroactively formalize both constructs with the full RFC template.

## Non-goals

- Not a general DXF/mechanical-CAD parser. CoHDL parses exactly one thing out of a referenced file: a single closed outline entity, by convention on a fixed, documented layer name. It does not understand DXF's block references, dimensions, text, hatching, or any entity other than the outline — the same narrow-contract discipline pad/footprint (RFC-018) already established for pad geometry.
- Not validating the outline's mechanical correctness beyond confirming it forms one closed loop — self-intersection, manufacturability, and real-world sensibility remain the mechanical engineer's/CAD tool's responsibility, never CoHDL's.
- Not placing an instance declared inside a called fn. place only resolves against the design's own top-level instances, exactly as it did before this RFC. Reaching into a nested fn call (e.g. to place a connector instantiated by a reusable sub-circuit) is a real, named, deferred capability — explicitly not solved here, per Tony's direct decision to defer until a concrete need proves it's worth a real design pass (path-qualification, ambiguity resolution, etc. — considered in this RFC's earlier draft, withdrawn per Tony's direction; see Alternatives).
- Not general 2D geometry/CAD authoring in .cohdl source, not arbitrary-angle rotation, not rotation math/collision reasoning, not general layer stackup — all unchanged from the prior revision.

## Design

### Board outline: scoped extraction of one entity from a referenced DXF

```cohdl
design Pico2 {
    layout {
        board_outline: "mechanical/pico2-outline.dxf"
    }
}
```

Surface syntax unchanged: board_outline: "path", one per design, design-top-level only.

- At cohdl build, CoHDL opens the referenced DXF and looks for exactly one designated outline entity — by convention, a closed LWPOLYLINE/POLYLINE on a fixed, documented layer name (the exact convention is emitter-documentation, not fixed in the .cohdl grammar — see Tooling & operations). Straight segments and arc bulges are both supported (DXF's polyline bulge factor and IPC-2581's Polygon line/arc segments are a direct, lossless translation).
- Everything else in the file is ignored.
- A missing, malformed, non-closed, or unparseable outline entity is a compile error at cohdl build (a new E1006 sub-case) naming the specific problem — never a silent empty or wrong Profile.
- The extracted geometry is embedded directly in IPC-2581's Profile/Polygon and in layout.json.

### Placement: coordinates + a closed-set rotation, top-level instances only

```cohdl
layout {
    place hdr at (0mm, 0mm) rotate 90
}
```

- place <inst> at (x, y) [rotate ANGLE] — rotate is optional (default 0, unrotated); ANGLE is one of {0, 90, 180, 270}.
- <inst> names a top-level instance of the design only — unchanged resolution from the original construct. If <inst> was instantiated inside a called fn, it is not reachable and place reports the same "not an instance in this design" error it always has.
- at's two Length-typed values, design-top-level-only restriction, at-most-one-placement-per-instance are unchanged.
- cohdl build passes the rotation value through unchanged into IPC-2581's Component/Location rotation attribute and layout.json — CoHDL performs no rotation math, no collision reasoning.

## Type-system-first test

Both constructs remain non-DRC:

- Board outline's checks (well-formed path, at-most-one, design-top-level-only, and now: the outline entity exists/parses/closes) are all structural properties of one declaration and one referenced file, checked once — never emergent-across-the-connectivity-graph.
- Placement's rotation check (closed-set membership) and its instance-name lookup (against the design's own top-level names, unchanged mechanism) are both structural and local.

## Conceptual impact

Low-Medium. No new core concept. Real new work is scoped and narrow: a bounded DXF-entity parser (one designated layer/entity, nothing else) and one small grammar addition (rotate). place's own resolution scope is explicitly not expanded in this RFC — deferring that keeps this RFC's conceptual cost at exactly what the first revision already claimed, not the larger cost the withdrawn path-qualification design would have added.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Low | Med | Med | High |

Trust (High): this revision is what actually closes the real gap — a board outline that produces a genuinely Quilter-importable Profile. The "reference-only, never-parsed" stance would have shipped a construct that compiles cleanly but cannot do its one job.

Diagnostics (Med): real new failure modes to name precisely (missing/malformed/non-closed outline entity) — genuine new diagnostic surface.

Netlist (Med): the IPC-2581 emitter's board-outline responsibility is now real geometry-extraction-and-translation work, not a rectangle synthesis or a bare reference.

Concepts/Grammar/Oracle (Low), Compat (Med): unchanged from the prior revision — rotate is additive; board_outline's body syntax is unchanged from the prior revision (still a path string), only its build-time behavior gains real content.

## Gradeability

- Board outline: checked at cohdl build — presence of the designated outline entity, that it forms one closed loop, that the file parses as valid DXF at all. Each failure is a distinct, named E1006 sub-case.
- Placement: unchanged mechanism — the named instance must exist among the design's own top-level instances (existing check, unchanged scope); rotation is closed-set membership, checked at declaration (E1007 sub-case).
- Neither runs in residual DRC.

## AI-generatability

High for placement — a closed four-value rotation set is exactly as easy to generate correctly as any other closed vocabulary in the language (RFC-001's units, RFC-008's pin roles), and naming a top-level instance is unchanged from every other .cohdl construct that already does so. Lower, and honestly so, for the board outline's DXF content — never something an AI author writing .cohdl source needs to generate; only the reference (a path string) is authored in .cohdl.

## Alternatives

- Keep "reference-only, never-parsed" for the board outline — rejected: cannot actually produce a Quilter-importable document, defeating this RFC's whole purpose.
- CoHDL becomes a general DXF/mechanical-CAD parser — rejected: unbounded scope with no corresponding need; the narrow single-entity extraction is the right-sized middle ground.
- Extend place to resolve a ::-separated path into a called fn's instances — considered in an earlier draft of this RFC (reusing RFC-006's existing call-chain naming scheme), withdrawn per Tony's direct decision: the gap is real, but not yet proven necessary by any concrete design, and the mechanism (path syntax, disambiguation of multiple calls to the same fn) is real added complexity that should wait for an actual need to shape it correctly, rather than being designed speculatively now. Consistent with this project's own repeated practice (RFC-007's rejected const-generics, RFC-018's deferred padstack richness, this same RFC's own closed rotation set) — deferred, not solved, and named honestly as a real limitation rather than silently worked around.
- Open-ended Angle unit type instead of a closed four-value rotation set — unchanged rejection from the prior revision: no concrete need shown yet for non-cardinal rotation.

## Compatibility

Two real, disclosed breaking changes, unchanged in scope from the prior revision:

1. board_outline's body syntax is a path string (unchanged from the prior revision); its build-time behavior now does real geometry extraction. rpi-pico2 needs a real DXF with a correctly-tagged outline entity before this RFC is landed for that example.
2. place's grammar gains an optional rotate clause — purely additive; every existing place <inst> at (x, y) statement (no rotate) is unchanged in meaning and unchanged in what instances it can name (top-level only, as always).

Depends on: RFC-013, RFC-015 — already Accepted. (RFC-006's call-chain path scheme is explicitly not depended on in this revision — that reuse was part of the withdrawn fn-nested design.)

## Tooling & operations

- The DXF outline-layer convention (which layer/entity name designates the board outline) must be documented once, clearly, in the emitter's own docs — real, necessary documentation work.
- cohdl lsp hover on a board_outline statement should show the extracted outline's bounding-box dimensions alongside the resolved file path.
- Error-code registry: E1006 gains real new sub-cases (missing/malformed/non-closed outline entity, unparseable DXF); E1007 gains the rotation sub-case. Both stay in the existing E10xx family.

## Teaching cost

Low, unchanged from the prior revision. Board outline: referencing an external file is an established pattern; the DXF outline-layer convention is a one-time thing to learn per project, documented in emitter docs. Placement rotation: a fourth closed-set vocabulary in the language, a familiar pattern by now. place's top-level-only restriction is exactly what it always was — no new concept to learn, and the deferred fn-nested case means there's nothing new to explain about it either.

## Failure modes

- The referenced DXF has no entity on the designated outline layer, or the entity isn't closed — now a real, detectable, named compile error (previously undetectable under the withdrawn reference-only design).
- A DXF outline entity exists but describes a self-intersecting or otherwise nonsensical-but-closed shape — CoHDL's structural check cannot catch this; a real, disclosed remaining gap, the mechanical engineer's/CAD tool's responsibility.
- A component that needs a locked, oriented position is instantiated inside a called fn — place cannot reach it; this is a real, disclosed, deferred limitation, not silently worked around. The workaround today is to instantiate board-edge/mechanically-significant components directly at the design's top level rather than inside a reusable fn, when a locked placement is needed.

## Migration path

rpi-pico2 needs a real DXF file with its board outline tagged on the documented convention layer before this RFC is considered landed for that example — genuine, non-mechanical work. Its existing place hdr at (0mm, 0mm) statement is unaffected (assuming hdr is already a top-level instance, which the original failure this RFC fixes implies it is); adding rotate 90 (or the correct value) closes the specific Quilter failure.

## Decision

Accepted (revised) — 2026-07-16. This revision: (1) board_outline now requires CoHDL to extract exactly one designated outline entity from the referenced DXF and embed its real geometry in IPC-2581's Profile — reference-only was insufficient for the real Quilter-import requirement; (2) place's scope is explicitly not expanded to reach fn-nested instances in this pass — that gap is real and disclosed, deferred per Tony's direct decision until a concrete need justifies designing the right mechanism for it. Recorded as a DR-026 amendment (see note 7). Language Specification (note 10)'s "Board outline and oriented placement" section is updated to reflect this design; its "Not yet specified" list gains an entry for fn-nested placement.
