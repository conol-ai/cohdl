# RFC-030: registry.cohdl.org — the package registry service

## Problem

RFC-029 (Package dependency versioning) gave every package a real, exact-version-pinned, content-hash-verified identity — but deliberately, explicitly did not touch where a package's bytes actually come from: its own Non-goals state "not registry hosting/publishing infrastructure (a package index, `cohdl publish`, yanking policy)... real hosting remains separate, not-yet-proposed future work." This mirrors the identical scope boundary RFC-016/RFC-017 already drew for the same reason. Today, `cohdl.lock`'s locked version+hash has nothing to resolve *against* except whatever happens to already be on disk — there is no real answer to "where does a fresh clone of this project get `sparkfun = "1.0.0"` from."

This RFC is that deferred piece: a real, hosted registry service (registry.cohdl.org), a real namespace/trust scheme distinguishing who is allowed to publish what, and the cohdl CLI surface (login, publish, add, remove, install, update) that talks to it — composing with, never reopening, RFC-029's already-Accepted version/lock/hash mechanism.

Who this is for: library consumers (need cohdl add/cohdl install to just work, the same one-command expectation npm install/cargo add already set); library authors (manufacturers publishing an official parts library, community members publishing a helper package) who need a real place to put content and a real identity attached to it; AI authors (need a small, closed, learnable namespace grammar to know whether a package name it's about to use is official, manufacturer-verified, or community-contributed, without a separate lookup).

## Goals

- A real, reachable registry service at `registry.cohdl.org`, modeled directly on npm/crates.io's proven shape (a package index + tarball/content storage + a web UI for browsing/search) — **the server-side technology stack is explicitly out of scope for this RFC** (see Non-goals); this RFC specifies the registry's external contract (namespace rules, API surface, auth model), not its implementation.
- A closed, three-tier namespace scheme, each tier legible in the name itself with no separate lookup needed to know its trust level:Bare name (`std`, `sensors`) — reserved exclusively for CoHDL's own official packages.`@brand/name` (`@sparkfun/power`) — reserved for a verified component-manufacturer's own official account.`@contrib/name` (`@contrib/awesome-connectors`) — open, any authenticated user, one shared namespace prefix rather than an unprefixed free-for-all.
- A cohdl CLI dependency-management surface that manages [dependencies] directly and resolves/publishes against the registry: cohdl add <package>, cohdl remove <package>, cohdl install, cohdl update, plus cohdl login/cohdl publish for authoring — composing directly with RFC-029's exact-version/cohdl.lock/hash-verification mechanism, adding zero new version-resolution semantics of its own.
- A read-only discovery surface for both packages and the public `part` declarations they contain: `cohdl search QUERY [--json]`, backed by a stable `GET /search` registry endpoint. Search requires neither a project nor a login and never changes `[dependencies]`, `cohdl.lock`, or the local package cache.

## Non-goals

- **Not the server-side technology stack.** This RFC does not specify a database, a language/framework, a storage backend, or a deployment topology for `registry.cohdl.org` — only the registry's *external contract*: what a client can ask it, what it returns, and the namespace/auth rules it enforces. This is a deliberate scope boundary matching RFC-016/017's own "define the resolution contract, not the hosting mechanics" precedent.
- **Not re-opening RFC-029's version/lock semantics.** Exact-version-only `[dependencies]`, `cohdl.lock`'s version+hash pinning, and the hard-error-on-hash-mismatch rule are all unchanged, unextended, and non-negotiable inputs to this RFC — the registry's job is only to be a trustworthy place those already-specified mechanisms resolve *against*.
- **Not yanking policy, package deletion, or vulnerability advisories.** Real, disclosed future work — npm/crates.io both have mature yank mechanisms this project should eventually study and adapt, but scoping that now (before real publishing volume exists to inform it) risks the same "speculative generality" mistake RFC-013 explicitly avoided for layout constraints.
- **Not organization/team-account management within a namespace** (e.g. multiple humans co-publishing under one `@brand`) — a real, likely need eventually, but this RFC scopes account identity to "one authenticated account owns a namespace," deferring multi-member ownership to a follow-up once real manufacturer accounts exist to need it.
- **Not a package search/discovery ranking algorithm** — package-and-part search is real and its request/response contract is stable, but *how* matching results are ranked (downloads, recency, relevance) remains implementation detail. The contract guarantees deterministic result ordering for one deployed registry revision, not one permanent ranking formula.
- **Not private/scoped registries or self-hosted mirrors** — `registry.cohdl.org` is the one official registry this RFC specifies; a private-registry story is real future work if a concrete enterprise need emerges, mirroring this project's recurring "don't build for a need not yet shown" discipline.

## Design

### Namespace scheme — three closed tiers

```toml
[dependencies]
std = "2.4.1"                              # tier 1 — CoHDL official
"@sparkfun/power" = "1.0.0"                 # tier 2 — manufacturer official
"@contrib/awesome-connectors" = "0.3.0"     # tier 3 — community
```

- **Tier 1 — bare name** (`^[a-z][a-z0-9_]*$`, matching RFC-016's existing package-name grammar unchanged). Reserved exclusively for packages published by the CoHDL project's own official account. A publish attempt to an unclaimed bare name from any other account is rejected outright — bare names are never first-come-first-served, unlike npm's own (much-regretted) original model. This is the direct fix for the trust question RFC-029 raised: an author typing `use std::...` must never need to check who published it.
- **Tier 2 — **`@brand/name`. `brand` must be a **verified manufacturer account** — verification is a real, human-gated process (the registry maintainers confirm the account represents the actual named company, analogous to npm's organization-verification and crates.io's namespace-reservation-for-known-projects precedents), not self-service. An unverified account cannot claim any `@brand/...` prefix at all; attempting to publish under an unclaimed or unverified brand prefix is rejected, naming the verification requirement.
- **Tier 3 — **`@contrib/name`. Any authenticated account may publish any not-yet-taken `@contrib/name` — first-come-first-served within this one shared prefix, exactly mirroring npm's ordinary unscoped-package model, but confined to this one namespace so it can never collide with or be confused for tier 1/2 trust.
- Package-name uniqueness is enforced globally within each tier's own rules: `@sparkfun/power` and `@contrib/power` are two entirely distinct, non-colliding names (different tiers, different owners) — this is intentional, the same "different module path, no collision" principle RFC-016 already established for cross-package name resolution, applied here one level up at the registry's own namespace.
- A package's tier is determined structurally, by its name's own shape — never by a separate metadata flag an author could misstate. `cohdl check`/`cohdl lsp` (and any future registry-aware tooling) can display a package's trust tier directly from its `[dependencies]` entry, with zero registry round-trip needed for the common case.

### Registry API surface (external contract only — implementation unspecified)

- `GET /packages/{name}/{version}` — resolve one exact version to its content hash + download URL. This is the single call cohdl install's (and RFC-029's) first-resolution path needs.
- GET /packages/{name} — list all published versions of a package (for cohdl add/cohdl update to discover the greatest semantic version, and for the web UI's package page).
- `GET /packages/{name}/{version}.tar.gz` (or equivalent) — the actual package content (source, `#[doc(...)]`-referenced documents, footprint symbols — the same content RFC-029's hash covers), fetched by exact version.
- `POST /packages/{name}/{version}` (authenticated) — publish. The registry independently re-computes the content hash server-side and returns it in the response — `cohdl publish` never trusts its own local hash computation as the published-to-registry truth; the registry's own computed hash is what `cohdl.lock` records for a fresh install.
- `POST /login` — authentication, returning a token the CLI stores locally (see Tooling & operations) for use on subsequent `publish` calls.
- `GET /search?q={QUERY}&kind={KIND}&limit={LIMIT}&offset={OFFSET}` — unauthenticated, read-only package-and-part discovery for the CLI. `q` is trimmed, required, free of Unicode control characters, at least three Unicode scalar values, and at most 128 UTF-8 bytes. `kind` is `all` (default), `package`, or `part`; `limit` is a canonical decimal from 1 through 50 (default 20); `offset` is a canonical decimal from 0 through 10000 (default 0), applied independently to both result families. Invalid input returns 400; non-GET methods return 405 with `Allow: GET`.
- The `/search` response has this exact fixed-order shape. Only
  `description`, `intent`, `manufacturer`, and `mpn` are nullable; they are
  JSON `null`, never absent, while `primary` is always a boolean:

```json
{
  "query": "TPS59650",
  "packages": {
    "results": [
      {
        "name": "@ti/dcdc",
        "tier": "brand",
        "latest": "0.1.1",
        "description": "Texas Instruments DC/DC controller devices, parts, and package land patterns.",
        "updated": "2026-08-11T03:05:57.424Z"
      }
    ],
    "has_more": false
  },
  "parts": {
    "results": [
      {
        "package": "@ti/dcdc",
        "tier": "brand",
        "version": "0.1.1",
        "fq": "ti_dcdc::controllers::multiphase::CTRL_TPS59650",
        "name": "CTRL_TPS59650",
        "device": "ti_dcdc::controllers::multiphase::TPS59650",
        "intent": null,
        "manufacturer": "Texas Instruments",
        "mpn": "TPS59650RSLR",
        "primary": true
      }
    ],
    "has_more": false
  }
}
```

  Each family fetches one row beyond `limit`, trims it, and sets its own `has_more`; a family excluded by `kind` is `{"results":[],"has_more":false}`. Package matching is an ASCII-case-insensitive substring over the complete name and the first 1024 Unicode scalar values of the latest description; non-ASCII description characters match literally, and that same bounded description prefix is returned. Results are ordered by exact name, name prefix, other matches, recency, then name. Part matching covers the owning package name, fully-qualified and short names, device, intent, arguments, structural variant, and primary/alternate AVL fields within the bounded projection described below; the displayed manufacturer/MPN pair is the first primary or alternate entry that matches either field, otherwise the primary entry, otherwise null/null with `primary: true`.
- Other search/browse endpoints backing the web UI remain implementation detail. The stable `/search` contract is deliberately separate from the existing `/api/search` browser endpoint, so extending the CLI does not change the web UI's package-only response shape.

### `cohdl` CLI surface

```bash
cohdl login                              # opens a browser-based auth flow, stores a token locally
cohdl publish                            # packages the current project per its cohdl.toml, publishes to the registry
cohdl search TPS59650                    # search packages and public parts; no project or login required
cohdl search TPS59650 --json             # the same bounded result set as one JSON document

cohdl add @sparkfun/power                 # add a dependency (resolves greatest semantic version, writes [dependencies] + cohdl.lock)
cohdl add @sparkfun/power@1.0.0           # add a package pinned to one exact version
cohdl remove @sparkfun/power              # remove a package from [dependencies] (and prunes its cohdl.lock row)
cohdl install                             # install all dependencies: resolve every [dependencies] entry against cohdl.lock
cohdl update                              # update dependencies: resolve each to its greatest semantic version
cohdl update @sparkfun/power              # update one named dependency only
```

- cohdl add <package> is the primary way a new dependency enters a project. It queries the registry for the package's published versions (GET /packages/{name}), resolves to the greatest semantic version (or the exact version given via @X.Y.Z), validates the name against the three-tier namespace grammar, writes the resulting exact-version entry into [dependencies], and performs RFC-029's first-resolution (writing the new cohdl.lock row with the registry's own server-computed hash) in the same step — an author never hand-edits [dependencies] for the common case.
- cohdl remove <package> deletes the named entry from [dependencies] and its corresponding row from cohdl.lock in one step — the direct, symmetric inverse of add.
- cohdl install is the ordinary, no-argument entry point for a fresh clone or a CI run — it performs exactly the resolution RFC-029 already specifies (check cohdl.lock, first-resolve any [dependencies] entry with no lock row yet, hard-error on any hash mismatch) against registry.cohdl.org as the content source. This RFC adds no new resolution rule here — install is simply "run RFC-029's existing resolution, now with a real place to fetch from."
- cohdl update (no argument) re-resolves every [dependencies] entry to the greatest published semantic version for that package, rewriting [dependencies] and cohdl.lock together — this is RFC-029's own already-specified "deliberate, visible act" pin-update path, given a real command surface. cohdl update <name> scopes the same re-resolution to one named package, leaving every other pinned dependency untouched. Neither form is ever triggered implicitly by cohdl build/cohdl install — an author always invokes update explicitly.
- `cohdl publish` requires a prior `cohdl login` (the CLI refuses with a clear diagnostic naming the missing auth step, never a bare network error). It validates the local `cohdl.toml`'s `[package] name`/`version` against the three-tier namespace rules *before* any network call — a bare-name or unverified-`@brand` publish attempt is rejected locally, with the same message the server would give, so an author gets the real reason immediately rather than after an upload round-trip.
- `cohdl search QUERY` calls the stable registry `/search` endpoint and prints package hits followed by public-part hits. It is usable from any directory, carries no project path, does not read credentials, and performs no manifest, lock-file, or cache mutation. On success, `--json` changes presentation only: it contains the identical bounded result set as the human rendering. Publisher-owned text is control-sanitized in human output. A valid query with no matches is a successful empty result; malformed queries are invocation failures, and registry/network/protocol failures reuse the E1204 registry-unreachable/protocol diagnostic (whose help says to check the network and `COHDL_REGISTRY`, then retry the search). In `--json` mode that failure emits the existing diagnostic JSON document rather than a partial discovery document.
- Part discovery is derived from the package API-documentation sidecar emitted by `cohdl docs`. Only package-local `items` whose kind is `part` and whose `pub` field is true are indexed — never private declarations and never the sidecar's `foreign` dependency items. An item's fully-qualified path must be rooted under the uploading package's server-derived module root and end in its declared short name, so a publisher cannot advertise another package's path as its own. Within fixed resource-safety projection budgets, the searchable record includes the owning package name, part symbol and fully-qualified path, device, intent, arguments, structural variant, and primary/alternate AVL field names and values; pathological excess projection data is deterministically omitted while the original sidecar remains stored. Only the package's most recently published version is searchable: publishing a new version clears the old search rows, uploading docs for that most-recently-published version atomically replaces its rows, and uploading docs for an older version never displaces the current index.
- Search reports the most recently published version of each package, matching the registry catalogue's existing meaning of "latest". This is intentionally a publication-order view, distinct from `cohdl add`/`cohdl update`'s greatest-exact-version resolution rule; every result names the exact version so the distinction is visible rather than implicit.

## Type-system-first test

N/A — this RFC specifies a hosted service's external contract and CLI commands, not a rule/DRC proposal. The one real new check (three-tier namespace validity) is structural, checked from the package name's own shape alone, both locally (pre-publish, pre-add) and server-side (authoritative) — consistent with this project's "prefer the earliest possible stage" discipline, just applied one layer outside the compiler itself.

## Conceptual impact

**Low.** No new `.cohdl` language concept at all — `Package` (RFC-016) and its version identity (RFC-029) are unchanged; this RFC adds an external service and CLI verbs, not new source-language grammar. The one new idea — a package's three-tier namespace/trust scheme — is a property of the *registry*, not the language; a `.cohdl` file's `use`/`[dependencies]` syntax is completely unaffected.

## Coherence matrix row

| Concepts | Grammar | Oracle | Diagnostics | Netlist | Compat | Trust |
|---|---|---|---|---|---|---|
| Low | Low | Low | Med | Low | Low | High |

Trust (High): this is the entire point — a real, verifiable, tiered publisher-identity scheme is what makes RFC-029's exact-version/hash pinning actually mean something in practice (pinning a hash is only as trustworthy as the place you first got it from).

Grammar (Low): [dependencies] keys may now contain @brand/name/@contrib/name shapes — a small, closed grammar extension to the existing package-name token, not a new syntax category.

Diagnostics (Med): new, real CLI-level diagnostics (missing login, unverified-brand publish attempt, bare-name-not-owned publish attempt, package/version not found, remove of a name not present in [dependencies]) — reuses the existing "suggest the exact fix" discipline every diagnostic in this backlog already follows.

Compat (Low): purely additive — no existing .cohdl source, [dependencies] entry, or cohdl.lock row is affected; a project that never publishes or adds from registry.cohdl.org (e.g. using only local/vendored packages) experiences zero change.

## Gradeability

Three-tier namespace validity is checked in two places, both structural and deterministic from the name's own shape: (1) locally, at cohdl add/cohdl publish time, before any network call; (2) authoritatively, server-side, on every publish request (the registry is the final arbiter — a client-side check is a fast-fail convenience, never a substitute for server-side enforcement). Content-hash verification on install is unchanged from RFC-029's own already-specified mechanism — this RFC does not add a second hash-checking code path, it makes the existing one resolve against a real server.

## AI-generatability

High. An AI author writing [dependencies] learns exactly one small, closed rule beyond RFC-029's existing exact-version grammar: a package name is either bare (official), @brand/name (manufacturer), or @contrib/name (community) — three shapes, each immediately legible, no memorized exception list. cohdl login/publish/add/remove/install/update are six small, single-purpose commands, each named for exactly one npm/cargo-equivalent action a model (or human) already has strong priors for. `cohdl search` is equally local and read-only, and its `--json` form lets an agent discover an importable part path and its exact owning package/version without scraping a web page or downloading multi-megabyte package sidecars.

## Alternatives

- **A single, flat, first-come-first-served namespace for every package (npm's original model)** — rejected: npm's own history is the cautionary tale here — a flat namespace means a manufacturer's real brand name can be squatted by an unrelated party before the manufacturer ever shows up, and there is no structural way to tell an official `std`-equivalent from an impersonation without an out-of-band check. This project's own recurring "make trust legible in the artifact itself, not a side-channel lookup" discipline (RFC-021's IPC-7351 naming, RFC-008's pin roles) argues directly against this.
- **Scoped packages for everyone, no reserved bare-name tier (npm's current **`@user/package`** model, no special "official" tier)** — rejected: this treats CoHDL's own standard library as just another `@cohdl/std`-shaped package, indistinguishable in trust from any other scope at a glance — but `std` is categorically different (every project implicitly depends on it, per RFC-029) and deserves a namespace tier that makes that categorical difference visible without reading metadata.
- **A single **`@verified/name`** tier merging manufacturer-official and community-contributed, distinguished only by an internal verified-badge flag** — rejected: this reproduces exactly the "trust requires a side-channel check" problem the three-tier scheme exists to avoid — a name's own text should say which trust tier it's in, not a flag a UI might or might not surface prominently.
- A single cohdl fetch <name> command instead of separate add/remove/install/update — this RFC's own earlier draft, corrected per direct instruction: a bare fetch conflates "declare a new dependency" (a [dependencies]-mutating act) with "just download this content" (a read-only act), and gives no dedicated, symmetric way to remove a dependency or to re-resolve an existing pin to a newer version. Four small, single-purpose verbs — matching npm's/cargo's own add/remove/install/update shape exactly — are more learnable and leave no gap in the surface (adding, removing, installing-from-scratch, and updating are all real, distinct, everyday actions).
- **Let this RFC also specify the server's technology stack** (a specific database, language, hosting provider) — explicitly rejected per Tony's direct instruction; the external contract (namespace rules, API shape, CLI commands) is stable, reviewable design that shouldn't be coupled to an implementation choice that can and should be made separately, by whoever builds it, without re-litigating this RFC.
- **A GitHub-releases-based registry (packages are just tagged git repos, no separate hosted index)** — considered: real, zero-additional-infrastructure precedent (some smaller ecosystems do this). Rejected: it can't offer a real search/discovery UI, can't independently verify a publish's content hash server-side (RFC-029's core trust guarantee depends on the registry itself, not the publisher, computing the authoritative hash), and can't enforce the three-tier namespace rules at all (git hosting has no concept of "this name is reserved").

## Compatibility

Purely additive. Every existing project using local/vendored packages (no `registry.cohdl.org` dependency) is completely unaffected — RFC-029's manifest/lock mechanism works identically whether a package's content came from the registry, a local path, or any other RFC-029-compatible source (this RFC does not restrict RFC-029's resolution to registry-only sources). No existing `[dependencies]` entry, `cohdl.lock` row, or `.cohdl` source file changes meaning.

**Depends on**: RFC-029 (Package dependency versioning) — this RFC is the hosting layer RFC-029's own Non-goals named as deferred future work. Reuses RFC-016's package-name grammar (extended, not replaced, for the `@brand/name`/`@contrib/name` shapes) and RFC-005's "readable label + verified identity" pattern (applied here as "readable namespace tier + server-verified publish").

## Tooling & operations

- `cohdl login` stores an auth token in a local, gitignored credentials file (analogous to `~/.npmrc`/`~/.cargo/credentials`) — never committed, never embedded in `cohdl.toml`/`cohdl.lock`.
- cohdl add/cohdl publish's pre-flight local namespace check (see Design) means a misconfigured package name fails fast with a clear diagnostic, before any upload/manifest edit — consistent with this project's "suggest the exact fix" diagnostic discipline.
- `cohdl check --json`/`cohdl build --json` gain no new fields from this RFC — registry interaction is a pre-compilation, CLI-level concern (like RFC-029's manifest/lock verification stage), not a compiler diagnostic. `cohdl search --json` is its own discovery-result document, not a diagnostic document and not RFC-010's verdict schema.
- The web UI at `registry.cohdl.org` (search, package pages, README rendering from a package's own `#[doc(...)]`-referenced content) is real, required, npm/crates.io-parity scope — but its exact page layout/design is implementation detail, not specified by this RFC.
- The registry maintains a bounded search index over API-doc sidecars rather than making each CLI client enumerate packages and download their documents. Each existing package's most-recent sidecar must be backfilled when this index is introduced; re-running `cohdl docs --publish` for that already-published package/version is the supported, idempotent backfill path.
- Error codes for CLI-level registry failures (login required, unverified-brand publish rejected, bare-name-not-owned publish rejected, package/version not found, remove of an absent dependency, network/registry-unreachable) reserve a new block, distinct from RFC-029's own manifest/lock-verification block, since these are a different kind of mistake (registry-interaction failures vs. local resolution/hash failures) per RFC-011's organizing principle.

## Teaching cost

Low. login/publish/add/remove/install/update map directly onto commands any npm or cargo user already knows by name and purpose. The three-tier namespace rule is one small, closed, immediately-legible-from-the-name fact — no special-casing beyond "which of three shapes does this name have."

## Failure modes

- **An author tries to publish to an unclaimed bare name** — rejected locally (before any network call) and, as the authoritative backstop, server-side too; the diagnostic names the reservation rule directly, not a generic "name taken" message that would wrongly suggest first-come-first-served applies.
- **An author tries to publish under **`@somebrand/...`** without being that brand's verified account** — same two-stage rejection, naming the verification requirement and (if the registry's web UI documents a request process) pointing at how to request verification.
- cohdl remove is given a name not currently in [dependencies] — a clear diagnostic naming the actual current dependency list, never a silent no-op.
- cohdl install/cohdl add/cohdl update run with no network access and no local cache — a clear, distinct diagnostic from a hash-mismatch failure (RFC-029's own already-specified error): "cannot reach registry.cohdl.org and no cached copy of <name>@<version> exists locally," never conflated with the hash-verification failure path, since they are different kinds of mistakes.
- cohdl search receives a query shorter than three characters after trimming — rejected locally as an invocation error before a network call, so the registry is never asked to perform an unbounded one- or two-character scan.
- cohdl search receives no matches — successful, with empty package/part lists; absence is not a registry failure and never becomes E1203.
- The registry has more matches than the response limit — it returns the bounded prefix and sets that result family's `has_more` to true; it never computes or exposes a total count. Both human and JSON renderings disclose the truncation rather than implying completeness.
- A locally-computed pre-publish hash and the registry's own server-computed hash disagree (e.g. a build artifact leaked into the package tarball) — the registry's computed hash is always authoritative for what cohdl.lock will later verify against; cohdl publish surfaces this as a warning showing both hashes, since the actual published (and hash-verified-on-install) content is whatever the server computed, not whatever the client assumed.

## Migration path

No existing .cohdl source, [dependencies] entry, or cohdl.lock row requires any change. A project currently depending on packages available only locally can adopt registry.cohdl.org incrementally, one dependency at a time, via cohdl add once the named package is actually published there — RFC-029's own resolution mechanism is agnostic to where content comes from, so there is no flag-day migration required.

## Decision

Accepted — 2026-07-24 (CLI surface corrected same day), amended 2026-08-24. registry.cohdl.org is specified as a real, hosted registry service with a closed three-tier namespace scheme (bare name = CoHDL official, @brand/name = verified manufacturer official, @contrib/name = open community), enforced both client-side (fast-fail) and server-side (authoritative). The cohdl CLI gains login/publish/add/remove/install/update, composing directly with RFC-029's already-Accepted exact-version/cohdl.lock/hash-verification mechanism without modifying it. (Revision note: the original acceptance proposed a single cohdl fetch command; Tony corrected this same day to the four-verb add/remove/install/update surface, matching npm/cargo's own dependency-management shape — this reflects the corrected design.) The 2026-08-24 amendment adds the read-only `cohdl search QUERY [--json]` verb and promotes `GET /search` from unspecified browser-search implementation detail to a stable package-and-public-part discovery contract, with the part index derived from API-doc sidecars. The registry's server-side technology stack remains explicitly out of scope, per direct instruction — this RFC specifies the external contract only. Recorded as DR-036 and its 2026-08-24 amendment (see note 7). Language Specification (note 10) documents the namespace, dependency-management, and search surfaces.
