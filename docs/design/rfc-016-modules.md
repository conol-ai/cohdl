# RFC-016: Module system (package::module::submodule::name)

## Problem

CoHDL has no namespace boundary today. Confirmed against real source: `project.rs` loads every `.cohdl` file (std and project source alike) into one flat file list; `resolve.rs`'s `World` resolves every device/trait/fn/part name against one global bucket, with no notion of "which file/package this name came from." `provisional-syntax.md §1` already documented this as provisional ("One flat global scope... There are no `module`/`use` declarations in the MVP... modules are 'Not yet specified' in note 10") and RFC-008 explicitly deferred designing it ("this RFC does not depend on or block the eventual module RFC").

This RFC is that eventual module RFC, triggered by a concrete, real need: Tony's proposed centralized library registry (RFC-017) requires a way to *name* a library, *import* specific items from it, and *resolve* a name like `sparkfun::power::buck::TPS62840` to an unambiguous declaration — none of which the current one-global-namespace design can express. Building the registry without first settling this would mean either inventing ad hoc, unreviewed namespacing rules inside the registry RFC, or perpetuating the flat-scope model at exactly the moment it stops being viable (multiple third-party libraries, each free to name a device `LDO` or `MLCC`, colliding in one global bucket).

Who this is for: **library authors** (need to name and organize what they publish), **library consumers** (need to import specific items without pulling in everything, and without collisions across libraries), and **AI authors** (need a regular, learnable way to reference a name that isn't ambiguous about which library/file it came from).

## Goals

- Give every top-level declaration (`trait`/`device`/`part`/`fn`) a **fully-qualified path** — `package::module::submodule::name` — derived from where it's declared, with no separate "namespace declaration" statement required (the path is structural, following the project's own directory/file layout, the same way Rust's module tree mirrors its file tree by default).
- Give an author a way to **import** specific paths into local scope (`use package::module::Name;`) so ordinary in-project code can keep referring to `Name` unqualified, exactly as it does today in the single-flat-scope model — this RFC must not make existing single-package projects (with no external library dependencies) any more verbose than they are now.
- Make **cross-library name resolution** and **in-project module organization** the same mechanism, not two — a project's own `src/` tree is just the local package's module tree; a dependency is another package's module tree, reached through its declared name.
- Preserve `pub`'s existing (currently-unenforced) visibility marker as the real enforcement point once modules exist — closing the exact gap `provisional-syntax.md` flagged ("`pub` is accepted and recorded but not enforced").

## Non-goals

- **Not designing the library registry's distribution/versioning/publishing mechanics** — that's RFC-017. This RFC only defines how a name resolves once a set of packages (however they got onto disk) is available to the compiler.
- **Not adding general re-export/aliasing sugar** (`pub use X as Y`, glob imports `use package::module::*`) in this first pass — a model/author writes explicit, one-name-per-`use` imports. Sugar can be a follow-up RFC once real usage shows the friction is real, not assumed.
- **Not changing name resolution for anything below the top-level-declaration granularity** — pin names, spec field names, generic parameter names stay exactly as scoped today (local to their enclosing declaration); this RFC is purely about top-level `trait`/`device`/`part`/`fn` paths.

## Design

### A package's module tree mirrors its file tree, by default

```javascript
my-library/
  cohdl.toml          # [package] name = "sparkfun"
  src/
    power/
      buck.cohdl       # module path: sparkfun::power::buck
      ldo.cohdl        # module path: sparkfun::power::ldo
    connectors/
      usb.cohdl        # module path: sparkfun::connectors::usb
    prelude.cohdl       # module path: sparkfun (files directly under src/ are the package root)
```

- `cohdl.toml`'s existing `[package] name = "..."` field (already real, unchanged) is the path's root segment.
- Each file's path under `src/` becomes its module segment, `/` → `::`, extension dropped — no separate `mod` keyword or module-declaration statement needed; the file tree *is* the module tree, exactly mirroring the project's own physical organization (a design already stated in note 2's "Locality of meaning" principle — this makes it structural, not just a review convention).
- A top-level declaration's fully-qualified path is `package::its-file's-module-path::Name`.

### Referencing a name: qualified path or `use`

```cohdl
// Fully qualified, always valid, no import needed:
inst ldo1: sparkfun::power::buck::TPS62840

// Or import once, use unqualified thereafter:
use sparkfun::power::buck::TPS62840;

inst ldo1: TPS62840
```

- `use path::Name;` — a new top-level statement, importing exactly one name into the current file's local scope. Importing the same local name twice (from different paths) is a compile error naming both source paths.
- Within a single package with no `use` statements and no cross-file name collisions, **behavior is unchanged from today** — every name in every one of the package's own files remains visible unqualified everywhere else in the same package, preserving the current single-flat-scope ergonomics for the common case (one project, no external dependencies). This is the load-bearing compatibility property: modules exist, but a project that never imports anything doesn't feel their weight.
- Cross-*package* names are never implicitly visible — a dependency's declarations must be reached via a qualified path or an explicit `use`, even if the depending project has zero files of its own with colliding names. This is deliberate: implicit cross-package visibility is exactly the "which library did this come from" ambiguity this RFC exists to remove.

### `pub` becomes real

- A declaration is visible **outside its own package** only if marked `pub` — the existing keyword, now enforced instead of merely recorded. Referencing a non-`pub` item from another package is a compile error naming the item and its actual visibility.
- Within a single package, `pub` has no effect (unchanged from today) — intra-package visibility stays fully open, consistent with the "one flat scope, no visibility boundary" reading `provisional-syntax.md` already documented for the single-package case.

### Name collision handling

- Two declarations in the same module-path with the same name is a compile error (unchanged principle, now scoped per-module-path instead of globally) — e.g. two `sparkfun::power::buck::TPS62840` declarations collide; a `sparkfun::power::buck::TPS62840` and an entirely separate `acme::power::TPS62840` do not (different packages, no ambiguity).
- An unqualified name that resolves to declarations in more than one *imported* path (two separate `use` statements importing two different `Foo`s under the same local name) is the "importing the same local name twice" error above — always caught at the `use` site, never deferred to the use site of the ambiguous name itself.

## Type-system-first test

N/A — this RFC is a name-resolution mechanism, not a `rule`/DRC proposal.

## Conceptual impact

**Medium-High — the second genuinely new core concept since the ground-up redesign began** (after RFC-013's Layout Constraint). Module/Package joins the canonical vocabulary. This is not, however, a parallel mechanism to anything existing — it's the formalization of a boundary note 2 and `provisional-syntax.md` both already anticipated ("modules are 'Not yet specified'") and RFC-008 explicitly deferred, so the conceptual cost was already priced in as inevitable, not a surprise addition.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Med | Med | Low | Med | Low | High | High |

**Compat (High):** the real, disclosed risk — `pub` becoming enforced is a genuine, if narrow, breaking change (see Compatibility). Multi-package projects (once RFC-017 exists) need every intended-external name marked `pub`, mechanically checkable but real migration work.
**Trust (High):** removing the "which library did this ambiguous name come from" risk directly serves the Constitution's "human reviewability & trust" ladder rank, especially once a real library ecosystem exists (RFC-017) and third-party name collisions become a real, not hypothetical, risk.
**Diagnostics (Med):** new diagnostics for unresolved qualified paths, `use` collisions, and `pub`-visibility violations — real new surface, reusing the existing E2xx (name resolution) block per RFC-011's organizing principle.
**Grammar (Med):** two new small, regular constructs (`use` statements, `::`-qualified paths) — no new expression forms, no ambiguity risk (a real grammar constraint per the Constitution's "deterministic grammar, no unbounded lookahead" hard requirement, checked against this shape: `::`-paths are LL(1)-parseable, consistent with existing turbofish `::<...>` precedent from RFC-007).
**Oracle/Netlist (Low):** no change to what's checked beyond name resolution itself, no netlist-format impact.

## Gradeability

Enforced entirely at the existing name-resolution stage (`resolve.rs`'s `World`), extended from a single flat bucket to a per-package, per-module-path bucket with explicit `use`-import edges — no new pipeline stage. Diagnostics: unresolved qualified path (names the path, suggests the closest match if one exists at a similar path — reusing the existing "suggest the exact fix" diagnostic-quality discipline), `use` collision (names both conflicting source paths), `pub`-visibility violation (names the item and where it's actually visible from). Mandatory regression: a fixture set with two packages sharing a colliding unqualified name, confirming both resolve correctly via their qualified paths and that no unqualified ambiguity leaks through.

## AI-generatability

High for the common case (single package, no imports) — literally unchanged from today, by design (see Design's compatibility property). Medium for the multi-package case: a model needs to learn `use path::Name;` and when a qualified path is required — but this is a small, regular addition directly analogous to Rust's own `use` statement, which is exactly the "Rust-inspired, not Rust-copied" pattern this whole redesign already uses (RFC-001's units, RFC-007's generics). No special-casing, no memorized exceptions.

## Alternatives

- **No module system at all — grow the library registry (RFC-017) on top of the existing flat namespace, disambiguating collisions ad hoc (e.g. auto-prefixing on conflict)** — rejected: this reproduces the redesign's own core complaint about v1's silent/implicit conventions (auto-prefixing on conflict is exactly a "correct by convention, not by the compiler" smell the Constitution forbids) — an author would have no way to *know* which `MLCC` they got without reading generated disambiguation output.
- **A flat, single-level namespace per package (no submodules, no file-tree mirroring)** — rejected: this doesn't scale to real libraries with many devices (a real vendor's registry entry could have hundreds of parts) and throws away the "locality of meaning" benefit file-tree mirroring gives for free — a reviewer already knows where to look for `power::buck::TPS62840` without reading a separate index.
- **Explicit **`mod`** declarations (Rust's actual mechanism) instead of implicit file-tree mirroring** — rejected: Rust's `mod foo;` exists partly to support non-file-tree-shaped module organization and partial/compile-time module gating, neither of which CoHDL has a use case for; implicit mirroring is simpler, has one fewer thing to keep in sync (the file's location is definitionally its module path, no possibility of drift), and note 2's "Locality of meaning" principle already wants file-and-module correspondence anyway.
- **Glob imports and re-export sugar in this same RFC** — rejected per Non-goals: no concrete friction demonstrated yet; adding convenience sugar before real multi-package usage exists risks guessing wrong about what's actually needed (the same "premature generality" reasoning RFC-013 used to reject a general layout-constraint plugin system).

## Compatibility

**Real, disclosed breaking change**: `pub` is enforced starting from this RFC. Every existing package (the demo boards, std library) that will ever be depended on by another package needs its intentionally-external items marked `pub` — but within a single, dependency-free package (the common MVP case today), **nothing changes**, since intra-package visibility stays fully open regardless of `pub` markings. This is a real migration cost only for the specific future case of multi-package usage, not a retroactive break of every existing `.cohdl` file today.

## Tooling & operations

- `cohdl check --json`/`cohdl lsp` (RFC-010/014) both need no schema change — new diagnostics reuse the existing `Diagnostic`/`JsonDiag` shape, just new `code` values in the existing E2xx block.
- `cohdl fmt` (RFC-009) gains formatting rules for `use` statements (one per line, sorted by path — following the same "one canonical way" discipline as every other construct) and `::`-qualified paths (no space around `::`, consistent with existing turbofish spacing).
- `cohdl lsp`'s goto-def (RFC-014) extends naturally: a qualified-path reference or an imported unqualified name both resolve to the same declaration span lookup already implemented — no new LSP capability needed, just a richer name-resolution input.

## Teaching cost

Low. This is deliberately the most "boring," most externally-precedented design in the backlog — file-tree-mirrors-module-tree plus explicit `use` is exactly Rust's own default module behavior (with Rust's `mod` keyword's flexibility cut, since CoHDL has no use case for it). An author who's seen any Rust code recognizes this immediately; an author who hasn't still only needs one new small rule.

## Failure modes

- **An author expects implicit cross-package visibility** (used to one flat global scope) and is confused why a dependency's device isn't visible without `use` — mitigated by the diagnostic explicitly naming the qualified path that *would* work, turning the failure into a one-line fix, the same "suggest the exact repair" discipline every diagnostic in this backlog already follows.
- **A library author forgets **`pub`** on something meant to be public** — caught immediately at any consuming package's first reference attempt, with a diagnostic naming exactly which item needs the marker — not a silent, hard-to-diagnose "why can't dependents see this" support question.
- **File-tree-mirrors-module-tree becomes awkward for a library wanting a module path that doesn't match a natural directory structure** — accepted as a real, narrow limitation for v1 (no `mod` escape hatch); if real library authoring proves this a genuine, frequent need, that's a scoped follow-up RFC per the Alternatives section's own reasoning, not evidence this RFC's default choice was wrong.

## Migration path

For any existing single-package project (all current examples/std library): **no migration required** — intra-package visibility is unaffected by `pub` enforcement. The moment such a package is depended on by another package (RFC-017), every item meant to be consumed externally needs `pub` — a one-time, compiler-flagged, mechanical addition (the exact same "one-time compatibility break, mechanically flagged" pattern RFC-008's pin-role retrofit already established as acceptable).

## Decision

**Accepted** — 2026-07-14. Recorded as DR-022 (see note 7). Language Specification (note 10) gains a "Modules and packages" section. This RFC is a direct prerequisite for RFC-017 (library registry) — RFC-017 must not be implemented before this RFC's `use`/qualified-path/`pub`-enforcement mechanism exists, per the project's "ship with its check" discipline applied to sequencing between RFCs.
