# RFC-0XX (number to be allocated): Footprint frame-change declarations and upgrade notices

> **PROPOSAL — not accepted; no number allocated.** Draft, 2026-11-13, written in the direction the maintainer gave: the package declares the change with its release, and `cohdl update` lists the affected `place` statements. The RFC number, the decision-record number and every code (`E-TBD-*` below) are placeholders pending central allocation in the conol.ai design repository; this file is a working copy in the repository snapshot. Nothing here is implemented. Baseline: main `74403a6` (2026-11-13). Builds on RFC-018/025 (footprints, pad rotation), RFC-020/026 (`place`, back side), RFC-029/030 (versioned packages, `cohdl.lock`, the registry and its verbs), RFC-032 (subdesign layouts), RFC-033 (provisional; computed placements) and RFC-034 (supplier mappings, `compose_place`, the map's pad projection).

## Problem

A footprint's **frame** is its origin and its 0° axis: the coordinate system its pads are written in. A `place INST at (x, y) rotate θ` positions that frame on the board, so its meaning depends on the frame. When a package version moves a footprint's frame, every existing placement of that footprint still parses, checks and builds, and its copper moves.

- RFC-034's Migration path records the situation: "A design that places that footprint still passes `check` and `build`, but its placed copper turns with the land, and `cohdl update` reports only the version change." The library plan (W3) is aligning footprints to assemblers' frames under breaking versions, so this is now routine and not an accident.
- The only notice today is prose: each package's README lists its moved footprints as `(dx, dy, r)` triples, and `tools/realign_place.py` applies a hand-copied table of them.
- **Evidence (OpenWalkieTalkie Rev C, compiler 0.10.0).** `soic` 0.3.0 turned `SOIC8P127X790X216N` by `(0mm, 0mm, 270)`. Raising `soic` 0.2.0 → 0.3.0 while leaving U4's `place flash at (32.6mm, 14.2mm) rotate 180 side bottom` unchanged gave a clean `cohdl check` and `cohdl build`, with all eight pads a quarter turn away from the routed copper. `cohdl update` reported twelve version changes, `soic: 0.2.0 -> 0.3.0` one of them. Only the project's own board-frame pad comparison caught it. The correct line is `rotate 270 side bottom`.
- The footprint was reached through a part in another package (`flash::FLASH_W25Q128JVSIQ` names `soic::SOIC8P127X790X216N`), so the author never wrote the footprint's name and had no reason to read `soic`'s README.

Who this is for: board authors and AI agents raising a dependency; library authors, who need a way to state a frame change that tools can read and CI can verify; reviewers of a dependency bump.

## Goals

- A frame change is **declared** by the package version that makes it, as machine-readable data that is part of the package.
- Every command that changes a pin **reports** each affected statement in the project, with the exact rewritten statement where one can be computed.
- An un-migrated placement stays visible in `check` and `build` after the update, offline and deterministically.
- No verdict changes and no emitted byte changes, in keeping with the registry-metadata rule that such keys never "affect a verdict or an emitted byte".
- Library CI can verify a declaration against the geometry of two versions.
- Serves ladder ranks 1 (a board whose copper silently moved is a wrong output), 3 (reviewability of a bump) and 5 (faithful output).

## Non-goals

- **Rewriting source automatically.** The compiler reports and suggests; it never edits a `.cohdl` file. `tools/realign_place.py` remains the tool that applies a rewrite.
- Detecting geometry changes that are not frame changes (a land resized in place). See Alternatives, 2.
- Renamed, moved or removed footprints. Those already fail resolution.
- A part that re-points to a different footprint in a new version (Open questions, 6).
- Compensating a board routed outside CoHDL. The RFC-034 map's `frame` hash already makes `tools/smt_pos.py --map` fail closed on such a board.
- Ranges, compatibility layers or automatic selection of an older version.

## Design

### 1. The declaration: `frames.toml`

A package that has ever changed a footprint frame carries a file `frames.toml` at its root, beside `cohdl.toml`.

```toml
# frames.toml — footprint frame changes, oldest first. Append-only.

[[change]]
version = "0.3.0"
footprint = "SOIC8P127X790X216N"
kind = "transform"
at = ["0mm", "0mm"]
rotate = 270

[[change]]
version = "0.3.0"
footprint = "TSSOP24P65_TI_PW0024A"
kind = "transform"
at = ["0mm", "0mm"]
rotate = 270
```

- `version`: the version of this package in which the change took effect. Exact `X.Y.Z`.
- `footprint`: the footprint's path inside the package's namespace, as the fully-qualified name without the package prefix.
- `kind`, a closed set:
  - `transform`: the frame moved rigidly. `at = [dx, dy]` (canonical mm strings) and `rotate = r` (whole degrees, `0..=359`) are the RFC-034 §2 triple, old frame to new: the new frame's origin is at `(dx, dy)` and its 0° axis at `r`, both in the old frame. This is the triple package READMEs list today and the one `tools/realign_place.py` takes.
  - `pads`: pad numbers or the pad set changed, so no rigid motion relates old placements to new ones. It carries no `at`/`rotate` and an optional `note` string. It is reported and never rewritten.
- A record means: the footprint's frame in every version `≥ version` (up to the next record for the same footprint) relates to its frame in the versions before it by this change.
- **Cumulative and append-only.** The file in version `V` holds every record with `version ≤ V` in the package's history, sorted by (`version`, `footprint`). So the newer of any two versions alone describes everything between them, with no registry round trip per intermediate version.
- A package with no frame change has no file. An absent file means "no declared change".

**Where it lives, and why.**

- *In the package, not in the manifest.* The file is ordinary package content, so the RFC-029 content hash already covers it (`src/hash.rs` hashes every non-dot file) and the tar carries it. `[package] version` stays the only thing that defines identity; this file defines none. It is not a `cohdl.toml` key because an older compiler must be able to resolve the package unchanged, and to it the file is just hashed content.
- *In the package, not a registry sidecar.* A sidecar (the API-docs model) could be corrected after publishing, but it would be mutable, absent for path and vendored packages, and unavailable to offline `check`/`build` (§4). A notice that decides how a user's source is read must be pinned with the version.
- *Consequence:* a forgotten record cannot be added to a published version. The fix is a new version whose file carries the missing record under the version where the change really happened. Records about earlier versions are therefore legal; only removing or altering one is not.
- The registry exposes the records per version (package page and the version metadata response) so a bump can be reviewed before it is made. That is display; the hashed file is the authority.

**Validity** (checked wherever the package is loaded; **E-TBD-1**, error, in the package-resolution block): unparseable file; unknown key or `kind`; a `version` greater than the package's own; duplicate (`version`, `footprint`); `transform` without `at`/`rotate` or `pads` with them; records out of order. A record whose footprint no longer exists in this version is legal: the footprint may have been removed since.

### 2. Meaning of a `transform`, and composition

**Normative rule.** For a placement `P_old` written against the old frame, the placement that keeps the copper where it was is

```text
P_new = compose_place(P_old, PlaceData { at: (dx, dy), rotate: r, side: Top })
```

with RFC-034's shared `compose_place` (`src/placement.rs`), which already covers the bottom side (mirror the offset's x, then rotate; reverse the rotation). This is exactly what `tools/realign_place.py` computes today, on the same fixed-point table (`src/trig.rs`).

- **Several versions at once.** Moving a pin from `a` to `b` (`a < b`) crosses the records with `a < version ≤ b`. Per footprint they compose in ascending version order by the same function: `Δ = compose_place(Δ₁, Δ₂)`, i.e. `d = d₁ + Rot(r₁)·d₂`, `r = (r₁ + r₂) mod 360`. A composed change equal to `(0, 0, 0°)` is no change and is not reported.
- **Downgrade** (`b < a`): the same records, inverted and composed in descending order.
- If any crossed record for a footprint is `pads`, the composed result for that footprint is `pads`.
- The older version's file is not needed: the newer one is cumulative.

### 3. The upgrade notice

**When.** Whenever a command rewrites a `cohdl.lock` row to a different version: `cohdl update`, `cohdl add NAME@X.Y.Z` over an existing pin, `cohdl install` or a build that re-resolves after a hand-edited manifest. Direct and transitive rows alike; the trigger is the row, not the manifest line.

**What is affected.** After resolution the command runs the front end on the project (parse, resolve, expand; no emission) and computes, for each composed non-identity change `(package, footprint, Δ)`:

1. every expanded instance whose part's primary footprint is that footprint, whichever package the part comes from. The footprint is found through the part's `footprint:` reference, never by name matching;
2. for each such instance, its placement and the statement that produced it;
3. every supplier mapping (RFC-034) on an AVL entry whose effective footprint is that footprint.

**What is printed.** One block per changed footprint, statements in file and line order, on stderr like the existing `name: old -> new` lines; `--json` carries the same records under an additive `frame_changes` key.

```text
  soic: 0.2.0 -> 0.3.0
note[E-TBD-2]: soic 0.3.0 moved the frame of soic::SOIC8P127X790X216N by (0mm, 0mm) rotate 270
  src/main.cohdl:263  U4 (flash, part flash::FLASH_W25Q128JVSIQ)
      place flash at (32.600000mm, 14.200000mm) rotate 180 side bottom
    rewrite to keep the copper where it is:
      place flash at (32.600000mm, 14.200000mm) rotate 270 side bottom
  1 placement affected. Nothing was rewritten; `cohdl check` reports these until acknowledged (E-TBD-4).
```

How each kind of placement is reported:

| Placement | Report |
|---|---|
| Literal `place` in the project's design, including a dotted reach-in override (`place a.c_in …`) and an indexed array element (`place leds[3] …`) | The statement and its exact rewrite, formatted as `cohdl fmt` writes it. |
| Computed placement (RFC-033 expression, or a `place` inside a `for`) | The statement, each affected instance with its evaluated placement and the placement it would need. No rewrite: one source line stands for several placements. |
| Internal `layout {}` of a `subdesign` owned by the project | The internal `place` statement and its rewrite in the subdesign's local frame, once, however many uses there are. The use site is not rewritten: the subdesign's own frame did not move. |
| Internal layout of a subdesign owned by a **dependency** | Named with its owner package; no rewrite, since the source is not the project's. See §4 for when it is stale. |
| Instance with no placement | Counted ("3 unplaced instances use this footprint"); nothing to migrate. |
| `kind = "pads"` | Every instance on the footprint, with the record's `note`; never a rewrite. |
| Supplier mapping in the project's own parts | The clause and its re-expressed mapping (§6). |

The notice **never changes a verdict or an exit status**, and the lock and manifest are written exactly as they are today. If the project does not get through the front end under the new versions, the notice falls back to the declared changes alone (footprints and deltas) and says the statements could not be listed.

### 4. Staying visible: frame acknowledgement in `cohdl.lock`

A notice printed once reaches only whoever ran the command, and not a reviewer, a CI job or an agent that bumped versions in bulk. The evidence case was exactly that: the build stayed clean. So the state "this project's placements have not been confirmed against the new frames" is recorded where builds read it.

**Authoring versions.** Every frame-dependent statement — a `place`, or a supplier mapping — is written against the footprint frames of specific package versions:

- a statement in a **dependency** is written against the versions that dependency's own manifest pins, or, for a package it reaches only transitively, the version in the `cohdl.lock` it ships (both are in its hashed content). When neither names the footprint's package, nothing is reported for that statement;
- a statement in the **project** is written against the versions the project has acknowledged.

A lock row gains one optional key:

```toml
[[package]]
name = "soic"
version = "0.3.0"
hash = "sha256:…"
frames = "0.2.0"
```

- `frames` is the version whose frames the project's own statements are written against. **Absent means equal to `version`**, so every existing lock file is unchanged and nothing is pending.
- A pin-changing command sets `frames` to the previous acknowledged version when the composed change between it and the new version affects at least one statement owned by the project (the list of §3), or when that could not be determined. Otherwise it leaves the key absent.
- `cohdl update <dir> --ack-frames [NAME]` removes the key, for one package or all. It changes nothing else. It is the author's statement that the placements were migrated, made after applying the rewrites and rebuilding.

**The warning.** `check` and `build` report **E-TBD-4** (warning) at each stale statement, with the same text and rewrite as the notice:

- a project statement, when the composed change between the row's `frames` and its `version` affects it;
- a dependency-owned statement (a subdesign's internal `place`, a part's supplier mapping), when the composed change between its owner's authoring version and the resolved version affects it. This half needs no recorded state. The remedy is a version of that package authored against the new frame, or, for a placement, a reach-in override in the project.

It is deterministic and offline: its only inputs are the source, `cohdl.lock` and the pinned packages' content. It is a warning, so the verdict and every artifact are unchanged, like E814/E815. A supplier-mapping instance of it is reported only under `--supplier`, when the mapping is used.

**Cost and benefit.** The cost is one optional lock key, one flag and one warning; an author can acknowledge without migrating, and placements added between the update and the acknowledgement are flagged although they were written against the new frame. The benefit is that the failure in the evidence, a clean build with moved copper, becomes impossible to reach without an explicit step recorded in a reviewed file. **Recommended:** adopt §4 together with §3. If only one is accepted, §3 alone still improves on today, and §4 is the part that would have stopped the Rev C case for anyone who did not read the update output.

### 5. What a library author must do, and how CI verifies it

1. Move the footprint, and add its record to `frames.toml` under the new version.
2. Publish as a breaking version: a major version, or a minor version while `0.x`; never a patch (the library plan's rule). **E-TBD-3** (publish pre-flight, error) rejects a record whose `version` is a patch-level successor of the previous published version.
3. Re-express, in the same release train, every supplier mapping and subdesign layout in packages that use the footprint (§6), and raise their pin.

**Verification** compares the tree with the previous published version (the registry tar, verified against its hash). It is a tool run by library CI, and may later be offered as a publish pre-flight; `publish` itself stays compile-free.

- **History:** the tree's records with `version` less than the tree's own must equal the previous version's file, byte for byte after canonical ordering.
- **Per footprint present in both versions**, take the RFC-034 pad projection (`kicad_mod::pad_plans`: numbered pads, KiCad anchors) extended with each pad's symbol and rotation, plus the mount holes:
  - identical projections and no record: pass;
  - identical projections and a record: fail (a declared change that did not happen);
  - different projections and a `transform` record: apply it to the old projection and require a perfect match with the new one, as multisets per pad number within 0.001 mm, with RFC-034's matching rule; pad rotation must differ by `r`;
  - different projections and a `pads` record: pass;
  - different projections, no record: fail when a rigid motion explains the difference (an undeclared frame change), and otherwise report the change as geometry-only, which is outside this RFC.
- The check validates the declared triple; it never infers one. A two-pad land turned by 180° is still caught, because pad numbers swap positions.

### 6. Supplier bindings

An RFC-034 mapping `(d, r)` places the supplier's footprint in the *entry footprint's* frame. When that frame changes by `Δ = (d_Δ, r_Δ)`, the same physical relation is expressed in the new frame as

```text
d' = Rot(−r_Δ) · (d − d_Δ)        r' = (r − r_Δ) mod 360
```

(the inverse of `Δ` composed with the old mapping, again through `compose_place`). For a footprint moved onto the assembler's frame, `d'` and `r'` come out as `(0, 0, 0°)`, which is the point of the alignment.

- The clause lives in the part's package, which may not be the footprint's package. That package must re-express the mapping in the version that raises its footprint pin. A part package left on the old pin while the project resolves the new footprint version has a stale mapping; §4 warns for exactly this combination, where today the supplier CPL is silently wrong.
- The notice (§3) lists project-owned clauses with `d'`, `r'` written out.
- The library plan's frame evidence is keyed by (footprint, package version, supplier, part number) and is already invalidated by a moved footprint; nothing changes there.

### 7. Worked example: `soic` 0.2.0 → 0.3.0 and OpenWalkieTalkie U4

- `soic` 0.3.0 ships a `frames.toml` in the form of §1: five `transform` records, each `(0mm, 0mm) rotate 270`, replacing the README table as the authority (the README may still render it).
- Library CI compares 0.3.0 with the 0.2.0 tar: each of the five lands' pad projections matches after the declared quarter turn; no other footprint differs.
- `flash` 0.3.1 pins `soic = "0.3.0"` and its `supplier lcsc "C97521"` clause is the identity mapping, written against the new frame.
- In the project, `cohdl update` moves twelve rows. One crosses a record. The notice is the block shown in §3: one placement, `src/main.cohdl:263`, U4, with the rewrite `rotate 270 side bottom` (bottom side: `180 − 270 ≡ 270`). The lock row for `soic` gets `frames = "0.2.0"`.
- Had the author ignored it: every `cohdl check` and `cohdl build` reports E-TBD-4 at line 263 until the line is rewritten and `cohdl update --ack-frames soic` is run. Today the same state is a clean build.
- Had the project kept `flash` at a version pinning `soic = "0.2.0"`: E-TBD-4 on that part's supplier mapping under `--supplier lcsc`, naming `flash` as the package to raise.

## Type-system-first test

No `rule` is proposed. Frame agreement is not a property of connectivity or of types; it is a relation between a statement and the version of the geometry it was written against. The mechanism is package metadata plus a warning derived from pinned content.

## Conceptual impact

**Low–Medium.** No new language concept and no grammar. "Frame" is already RFC-034's word (the map's `frame` hash) and keeps that meaning. New: a package content file, one lock key, one CLI flag. Package identity (RFC-029) is untouched.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Med | Low | Low | High |

- **Diagnostics (Med):** one load error, one publish error, one note and one warning, all carrying a computed rewrite. The rewrite uses `compose_place` and `fmt`'s own printer, and a conformance test asserts that applying it reproduces the previous build's pad positions.
- **Trust (High):** a wrong or missing declaration misleads. Addressed by the CI verification (§5), by the declaration being hashed with the version, and by the registry showing it before the bump.

## Gradeability

- E-TBD-1 at package load, before any `.cohdl` parsing, like the other lock and manifest checks.
- E-TBD-4 after expansion, from pinned inputs only.
- Fixture: a two-version footprint package and a project placing it top and bottom, literal, in a `for`, inside a project subdesign and inside a dependency subdesign; expected notice, lock key, warning, and the pad positions after applying each suggested rewrite. A second fixture jumps two records at once and one downgrades.

## AI-generatability

The notice is written for an agent as much as a person: file, line, the statement as it is and the statement as it must be. An agent that raises a dependency can apply the rewrites and acknowledge without reading a README. No source construct is added to learn.

## Alternatives

1. **Major-version convention plus README lists** (today). Rejected as the whole answer: prose is not read by tools, the footprint is usually reached through a part in another package, and twelve version lines hide the one that matters.
2. **A geometry hash per footprint in `cohdl.lock`**, so any geometry change of a placed footprint is visible. It needs no declaration and also catches resized lands and undeclared moves. Not chosen as the mechanism: a hash says *that* something changed but not *how*, so no rewrite can be suggested; it fires on every courtyard or silkscreen tweak, which trains authors to acknowledge blindly; and it grows the lock by a row per footprint. It is complementary, and its detection half is what §5's CI check performs on the library side, where the author can act on it.
3. **A manifest key in `cohdl.toml`.** Rejected: an older compiler's manifest parser would have to accept it, and a list of records per footprint and version is data about the package's history, not part of its identity or its dependency set.
4. **A registry sidecar.** Rejected in §1: mutable, registry-only, unavailable offline.
5. **Make `check`/`build` fail** until acknowledged. Rejected: the direction given is a notice that changes no verdict, and an un-migrated design is a legal design whose author may be mid-migration.
6. **Rewrite the source during `cohdl update`.** Rejected (Non-goals): a pin change that edits design files is a different contract, and computed placements cannot be rewritten mechanically.
7. **Per-footprint acknowledgement** in place of per-package. More precise, larger lock, same workflow. Deferred until a package moves frames in two unrelated releases that one project straddles.

## Compatibility

Additive. Packages without `frames.toml`, and locks without `frames`, behave and hash exactly as today. No source, error code, designator or artifact byte changes. A new compiler reading an old lock sees nothing pending. An old compiler reading a lock with a `frames` key is the one break to settle: it must either be tolerated as an unknown key or the key must live elsewhere (Open questions, 3).

## Tooling & operations

- `cohdl update --json` gains `frame_changes`; `check --json` gains one warning code.
- `tools/realign_place.py` accepts `frames.toml` and a version pair in place of a hand-written change table; its refusal to realign twice stays.
- Registry: package page and version metadata show a version's records; publish runs the E-TBD-3 check server-side as well.
- LSP: E-TBD-4 as a diagnostic with the rewrite as a quick fix.
- Reversible: `--ack-frames` is undone by restoring the lock row; a downgrade produces the inverse notice.

## Teaching cost

For board authors: "when an update prints a frame notice, apply the rewrites, rebuild, then acknowledge." For library authors: one file, one record per moved footprint, verified by CI. One subsection each in the RFC-029 and RFC-034 entries of note 10.

## Failure modes

- **A library moves a frame and forgets the record.** CI (§5) fails for official packages. A community package without that CI can still publish it; this RFC does not make the registry verify geometry (Open questions, 5).
- **Acknowledging without migrating.** Possible by design; the acknowledgement is a line in a reviewed file.
- **Applying a rewrite twice.** The warning disappears after acknowledgement, and `realign_place.py` refuses a placement that no longer matches the build it was computed from.
- **A wrong triple that passes review.** The CI check compares it with the geometry; a triple that does not reproduce the new pads fails.
- **False warnings** for placements added after the update and before the acknowledgement (§4).
- **Notice lost in bulk output.** It repeats as E-TBD-4 on every build.

## Open questions and decisions requested

1. **Scope.** Accept the declaration and notice (§1–§3) and the lock acknowledgement with its warning (§4) together (recommended), or §1–§3 first?
2. **File and keys.** `frames.toml` at the package root with `[[change]]` records, cumulative and append-only. Accept the name and shape?
3. **Where the acknowledgement lives.** A `frames` key on the `cohdl.lock` row (recommended, if released compilers tolerate an unknown lock key or a lock-format bump is acceptable), a separate generated file, or a `cohdl.toml` section?
4. **Severity.** E-TBD-4 as a warning (recommended), or an error for the dependency-owned half, where the project cannot have migrated anything?
5. **Enforcement at publish.** Is geometry verification library CI only (recommended for now), or should the registry refuse a version whose pads moved without a record?
6. **Part re-pointing.** A part version that names a different footprint moves copper the same way and is declared by nobody. Extend this RFC with a record kind for it, or leave it to a follow-up?
7. **Breaking-version rule.** Enforce "never a patch" at publish (E-TBD-3), or keep it a library convention?
8. **Backfill.** Should moves already published under the library plan (`soic` 0.3.0 and any other aligned package) be recorded retroactively in each package's next version, so projects still on the older version get the notice when they cross?
9. **Acknowledgement verb.** `cohdl update --ack-frames [NAME]`, or a separate command?

## Migration path

None for designs. Library packages that already moved frames add `frames.toml` with the records their READMEs list, in their next version. Projects that already migrated by hand see nothing: their lock rows have no `frames` key, and no pin is crossing a record.

## Decision

**None — proposal.** Awaiting the maintainer's decisions on the nine points above, then central allocation of the RFC number, decision record and codes.
