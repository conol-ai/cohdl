# RFC-029: Package dependency versioning — std becomes a versioned registry library

## Problem

Confirmed against real source (`src/project.rs`, `parse_manifest`): a `cohdl.toml` manifest has exactly two real sections — `[package] name` and `[design] top` — plus a `version` key that is parsed and **immediately discarded** (`_ => {} // tolerated: version, future fields`). There is no `[dependencies]` section anywhere in the grammar, no dependency-resolution code path (`grep` for `dependenc`/`[deps]`/package-resolution in `src/project.rs`/`src/resolve.rs` returns nothing), and `std` itself is not a normal dependency a project names and versions — it is located by `find_std_dir`, a hardcoded search: `--std` flag → `COHDL_STD` env var → a path next to the compiler binary → cwd. **The same **`.cohdl`** source file can silently resolve against a different **`std`** on a different machine, or the same machine on a different day, with zero record of which one it actually built against.**

This is a real, long-disclosed gap, not new: RFC-016's Non-goals states *"Not designing the library registry's distribution/versioning/publishing mechanics — that's RFC-017."* RFC-017's Non-goals repeats it: *"Not the actual centralized hosting infrastructure... versioning/yanking policy... future work, likely their own RFC once real libraries exist to distribute."* DR-024's Consequences names it a third time: *"No versioning/pinning mechanism for copad [pad] references — a cofp [footprint] always resolves to whatever the referenced copad currently is, mirroring the same absence of version pinning at every other use-based resolution point in the language today."* This RFC is that deferred RFC, triggered by Tony's direct observation: hardware's essence demands strict version control, categorically more than software does — a `.cohdl` source file compiles to a physical fab order; once boards are ordered, the design is frozen in copper. Unlike a software dependency, there is no "hotfix the running system" recourse if a rebuilt design silently differs from what was actually fabricated.

Who this is for: **library consumers** (need a build today to produce the exact same netlist/footprint geometry as a build six months from now, even as `std` and other libraries keep evolving), **library authors** (need to publish a fix without silently breaking every existing consumer's next rebuild), and **AI authors** (need every `use` reference to resolve deterministically, with no environment-dependent "which version did I just get" ambiguity).

## Goals

- Give every package a real `[dependencies]` manifest section, naming each dependency and pinning it to an exact, unambiguous version — closing the "no dependency declaration mechanism exists at all" gap.
- Make dependency resolution **reproducible and durable**: a `cohdl.lock` file (mirroring RFC-005's `design.lock` precedent exactly) records the exact resolved version + content hash of every dependency the moment it's first resolved, and every subsequent build uses the locked values — never silently re-resolving to "whatever is current."
- Make `std` an ordinary (if always-implicitly-available) versioned package under this same mechanism — closing `find_std_dir`'s real, disclosed ambiguity as a direct consequence, not a separate fix.
- Reject any resolution mechanism that can silently produce different bytes for the same locked input — the single hard requirement Tony's directive establishes for this RFC.

## Non-goals

- **Not semantic-version ranges (**`^1.2`**, **`~1.2.3`**, **`>=1.0, <2.0`**).** See Design and Alternatives — range-based resolution is explicitly and permanently rejected for this language, not merely deferred, because it reintroduces exactly the silent-drift failure mode this RFC exists to close.
- **Not the actual hosting/publish infrastructure** (a package index server, a `cohdl publish` CLI, yanking policy, a web UI) — this RFC defines the manifest/lock/resolution mechanism assuming packages are available on disk (mirroring RFC-016/017's own scope boundary exactly). Real registry hosting is separate, not-yet-proposed future work.
- **Not a build cache or reproducible-build byte-identity guarantee beyond the compiler's own existing determinism** (RFC-005's designator allocator, `cohdl fmt`'s idempotence) — this RFC guarantees which library *content* a build resolves against, not that two different compiler versions produce identical output from that content.
- **Not retroactively re-litigating the footprint format or module system** — this RFC is additive to RFC-016 (module paths) and RFC-017/018 (library content kinds); it does not change how a name resolves *within* a locked dependency set, only how that dependency set itself is pinned.

## Design

### `[dependencies]`: a new manifest section, exact versions only

```toml
[package]
name = "sensor-node"
version = "0.3.0"

[dependencies]
std = "2.4.1"
sparkfun = "1.0.0"
adafruit-motors = "0.9.2"
```

- Every entry is `name = "X.Y.Z"` — a single, exact semver triple. **No range operators are valid grammar** (`^`, `~`, `>=`, `<`, `*`, `,` are all rejected at manifest-parse time with a diagnostic naming the offending entry and explaining that CoHDL requires exact versions).
- `[package] version` (already tolerated-but-ignored per real source) becomes real: every package, `std` included, declares its own version — this is what a dependent's `[dependencies]` entry names.

### `cohdl.lock`: the durable, checked resolution record

Mirrors RFC-005's `design.lock` precedent exactly (stable across rebuilds; explicit, checked postcondition rather than an assumption):

```toml
# cohdl.lock — generated. Do not hand-edit; run `cohdl update <name>` to change a pin.
[[package]]
name = "std"
version = "2.4.1"
hash = "sha256:1a2b3c4d..."

[[package]]
name = "sparkfun"
version = "1.0.0"
hash = "sha256:5e6f7a8b..."
```

- On first resolution (no `cohdl.lock` present, or a `[dependencies]` entry with no lock row), the compiler resolves the named exact version, records its version **and a content hash** (a hash over the dependency's full package content — every `.cohdl` file, `#[doc(...)]`-referenced document, and footprint symbol it carries), and writes `cohdl.lock`.
- On every subsequent build, the compiler **re-hashes the resolved package content and compares it against the locked hash** — a mismatch (the named version's content is not what it was when locked) is a compile error naming the package, both hashes, and refusing to build. This is the load-bearing guarantee: a version *number* alone is a human label a publisher could accidentally or maliciously reuse; the hash is what makes "version 2.4.1" actually mean one immutable, verified byte sequence.
- `cohdl.lock` is committed to the consuming project's own version control (the same convention `design.lock` already establishes) — it is itself the durable record of "what was this design actually built against," answerable months later without any external registry lookup.
- Updating a pin is a deliberate, explicit act: `cohdl update <name>` (or hand-editing `[dependencies]` to a new exact version and re-running `cohdl build`) re-resolves that one entry and rewrites its `cohdl.lock` row — never automatic, never triggered by an ordinary build.

### `std` becomes an ordinary versioned package

- `find_std_dir`'s hardcoded search (`--std` flag / `COHDL_STD` env / binary-adjacent path / cwd) is retired as the *resolution* mechanism. `std` is versioned and locked exactly like any other registry dependency — a project's `[dependencies]` names `std = "2.4.1"`, and `cohdl.lock` pins its exact content hash.
- `std` remains privileged in exactly one sense: every project has an implicit `std` dependency unless it explicitly opts out (mirroring Rust's own `std`/`no_std` precedent) — an author does not need to write `use std::...` imports any differently, and does not need to discover `std`'s existence; they only need to state which version they're pinned to, the same as any other dependency.
- `--std` / `COHDL_STD` remain available strictly as **local development overrides** (e.g. testing an unpublished `std` change against a real project) — when used, `cohdl build` emits a mandatory, unsuppressable warning that the build is not using the locked `std` and its result must not be treated as reproducible. This mirrors `#[intent(...)]`'s "decorative, not load-bearing" discipline in reverse: an explicit, visible escape hatch for real development need, never a silent substitute for the locked mechanism.

## Type-system-first test

N/A — this is a package-resolution/build mechanism, not a `rule`/DRC proposal. The one new checkable fact (locked hash matches resolved content) is structural and local to one dependency's own content, checked at the earliest possible point (before any `.cohdl` parsing begins) — consistent with this project's "prefer the earliest possible stage" gradeability discipline, just applied one layer below the type checker.

## Conceptual impact

**Low-Medium.** No new core language concept (Trait/Device/Part/Net/Design/Layout Constraint/Module are all unchanged) — this RFC extends **Package** (RFC-016's already-Accepted concept) with a version identity and a lock record, the same kind of extension RFC-002 made to Pin (adding an obligation kind) rather than inventing something orthogonal. `std` does not become a new concept either — it was always implicitly "the package everything depends on"; this RFC makes that implicit fact a real, checkable one.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Med | Low | Med | High |

**Trust (High):** this is the entire point of the RFC — a design's dependency set becomes durably, verifiably pinned, closing a real, three-times-disclosed gap (RFC-016/017 Non-goals, DR-024 Consequences).
**Compat (Med):** every existing project implicitly depended on whatever `std` `find_std_dir` happened to locate; adopting this RFC requires each project to add a real `[dependencies]` section and generate its first `cohdl.lock` — mechanical, compiler-flagged (see Migration path), not a silent behavior change.
**Diagnostics (Med):** new, real failure modes (hash mismatch, invalid version-range syntax, unresolvable named version) — reuses existing error-code organizing discipline (RFC-011), no new block needed beyond one reserved for package resolution.
**Grammar (Low):** one new manifest section (`[dependencies]`), one new file format (`cohdl.lock`, TOML like the manifest already is) — no `.cohdl` source-language grammar changes at all.
**Netlist/Oracle (Low):** zero impact on what's checked or emitted — this happens entirely before compilation begins.

## Gradeability

Enforced at project load, before any `.cohdl` parsing: (1) manifest parse rejects any non-exact-version `[dependencies]` entry, naming the offending syntax; (2) `cohdl.lock` presence/consistency is checked — a `[dependencies]` entry with no corresponding lock row triggers first-resolution (writes a new row); a lock row whose re-hashed content disagrees with its recorded hash is a hard compile error, never a warning, naming the package, both hashes, and refusing to proceed. This is a new pipeline stage, but it precedes and gates everything RFC-001–028 already check — a `.cohdl` file is never even opened against an unverified dependency set.

## AI-generatability

High. An AI author writes `[dependencies]` exactly the way it already writes `[package]`/`[design]` — `name = "X.Y.Z"`, no range syntax to reason about, no "which version constraint is safe" judgment call (the exact-version-only rule removes the entire class of semver-range reasoning a model would otherwise need to get right). `cohdl.lock` is machine-generated, never hand-authored — nothing new for a model to produce there at all.

## Alternatives

- **Semantic-version ranges (**`^1.2`**, **`~1.2.3`**), the mainstream software-registry default (npm/cargo)** — rejected outright, not merely deferred. Software's implicit assumption — "a minor/patch bump is safe to auto-adopt" — is actively false for hardware libraries: a "patch" correction to a footprint's pad geometry moves real copper; a "minor" addition to a device's spec can change a DRC verdict. Auto-resolving within a range is precisely the silent-drift mechanism Tony's directive identifies as categorically unacceptable for hardware, and it is the same failure class DR-024 already disclosed as a standing gap. Every version bump requires an explicit, visible author action (`cohdl update`) — never an implicit one.
- **Version number alone, no content hash** — rejected: a bare semver number is a human-chosen label, not a cryptographic guarantee. Nothing stops a publisher (accidentally, via a botched re-publish, or maliciously) from serving different content under an already-used version string. The hash is what makes "locked to 2.4.1" mean one immutable byte sequence, not "whatever answers to that name right now."
- **Leave **`std`** resolved by **`find_std_dir`**, version only third-party registry packages** — rejected: `std` is the single highest-consequence dependency in every project (every design touches it), and it is exactly the case Tony's directive names directly. Carving it out as a permanent exception would leave the worst instance of the problem unsolved while fixing only the easier, less-used case.
- **A build-time cache/vendoring mechanism (copy dependency source into the project) instead of a lockfile+hash** — rejected: vendoring duplicates content across every consuming project (real storage/consistency cost) and still needs a hash-equivalent integrity check to be trustworthy, so it doesn't actually remove the mechanism this RFC proposes — it just also adds a copy step. A lockfile referencing content by hash is the minimal mechanism that achieves the same guarantee.
- **Trust git commit SHAs as the version identity instead of semver + hash** — considered: a commit SHA is already content-addressed and immutable in the way this RFC wants. Rejected as the *sole* identity because it discards the human-readable "2.4.1, a minor bump" signal a semver number carries for a reviewer skimming a diff — this RFC uses both: semver for human legibility, hash for the actual cryptographic guarantee, the same "readable label + verified identity" pattern RFC-005's designators already use (a readable `C3` backed by a checked-unique allocation).

## Compatibility

**Real, disclosed breaking change, mechanical to fix.** Every existing project (the demo boards, and any project built against `std` today) has no `[dependencies]` section and no `cohdl.lock`. Adopting this RFC requires:

1. Adding `[dependencies]` naming the exact `std` version currently in use (and any other real dependencies).
2. Running `cohdl build` once to generate the initial `cohdl.lock` (first resolution, per Design).

No `.cohdl` source-language syntax changes — every existing `trait`/`device`/`part`/`fn`/`use` statement is completely unaffected. This mirrors RFC-008's own "one-time compatibility break, mechanically flagged" pattern (pin-role retrofit) exactly.

**Depends on**: RFC-016 (module system — package identity) and RFC-017 (library registry — the content-kind surface a package's hash covers). Does not depend on, and does not reopen, the still-deferred footprint format RFC.

## Tooling & operations

- `cohdl build`/`cohdl check --json` (RFC-010) both gain new diagnostic codes for manifest/lock errors — reuses the existing `Diagnostic`/`JsonDiag` shape, a new error-code block (package resolution — a new kind of mistake, per RFC-011's organizing principle, since none of the existing blocks own "which dependency version resolved").
- `cohdl.lock`'s format (TOML, one `[[package]]` table per dependency) is deliberately parallel to `design.lock`'s existing shape — an author or tool already familiar with one recognizes the other immediately.
- New CLI surface: `cohdl update [<name>]` — re-resolves one (or, with no argument, every) pinned dependency to its currently-named exact version and rewrites the corresponding `cohdl.lock` row(s). This is the **only** sanctioned way a locked hash changes; an ordinary `cohdl build` never mutates `cohdl.lock` except to add a first-resolution row for a brand-new `[dependencies]` entry.
- `cohdl fmt` gains a canonical form for `[dependencies]` (sorted by name, consistent with `use`-statement sorting already established by RFC-016's tooling section) — `cohdl.lock` itself is machine-generated and never passed through `fmt`.

## Teaching cost

Low. An author already writing `[package]`/`[design]` learns one more TOML section with one rule ("exact version only, no ranges") and one new generated file they commit but never hand-edit. The "why no ranges" rationale (hardware ≠ software patch-safety) is a one-sentence explanation, directly traceable to this project's own recurring "strictness buys expressiveness/trust" thesis (DR-005).

## Failure modes

- **An author writes a range (**`^1.2`**)** — caught immediately at manifest-parse time, before any resolution attempt, with a diagnostic explaining exact-version-only and suggesting the nearest valid exact version if one is discoverable.
- **A locked package's content has changed underneath its recorded hash** (a compromised or misconfigured registry mirror, a corrupted local cache) — caught at every build, hard error, never silently proceeding with mismatched content. This is the RFC's central guarantee under adversarial/accidental conditions, not just the cooperative case.
- **An author runs a local **`--std`** override and forgets** — mitigated by the mandatory, unsuppressable warning on every build using an override; CI pipelines should treat this warning as a build-blocking condition for anything intended to be reproducible (a documented convention, not a compiler-enforced one, the same honesty RFC-019's TextMate-drift risk already disclosed for an external, not-fully-compiler-enforceable concern).
- **Two projects pin different **`std`** versions and are later merged/compared** — expected and correct: they are genuinely different builds against genuinely different `std` content; this RFC makes that difference visible (`cohdl.lock` diffs) rather than hidden, which is the intended behavior, not a bug to smooth over.

## Migration path

For any existing single-package project with no real external dependency today (the common case): add `[dependencies]` naming the `std` version currently effectively in use (the version installed at the location `find_std_dir` currently resolves to), run `cohdl build` once to generate `cohdl.lock`, commit it. This is real but small, one-time, fully mechanical work — `cohdl build` should detect the pre-RFC-029 state (a manifest with no `[dependencies]` section) and offer to perform this migration automatically rather than merely erroring.

## Decision

**Accepted — 2026-07-24.** `[dependencies]` becomes a real manifest section, exact-semver-version-only (no ranges, ever — a permanent language rule, not an interim restriction). `cohdl.lock` records each resolved dependency's exact version and content hash, mirroring RFC-005's `design.lock` discipline; every build verifies the locked hash before compiling anything. `std` is retired as a hardcoded-path singleton and becomes an ordinary (implicitly-depended-upon-by-default) versioned registry package under this same mechanism, closing `find_std_dir`'s disclosed ambiguity and DR-024's disclosed "no versioning/pinning" gap directly. `--std`/`COHDL_STD` survive only as visibly-warned local development overrides. Recorded as DR-035 (see note 7). Language Specification (note 10) gains a "Package dependency versioning" section documenting `[dependencies]`/`cohdl.lock`/`std`'s new resolution path.
