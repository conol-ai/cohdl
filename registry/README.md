# registry.cohdl.org

The CoHDL package registry (RFC-030) — API + web UI in one Cloudflare
Worker.

RFC-030 deliberately specifies only the registry's *external contract*
(namespace rules, API shape, CLI verbs). This implementation's stack, per
direct instruction: **all Cloudflare services** —

| Service | Role |
|---|---|
| Workers | the API + UI host (one worker, `run_worker_first` API routes) |
| D1 | the index: accounts, brands, tokens, packages, versions |
| R2 | package content (uncompressed tar archives) |
| KV | web sessions |
| Workers Assets | the SPA (single-page-application fallback) |

The web UI is **React + Vite + TanStack** (Router for the SPA routes,
Query for data), with the **Vite Plus** toolchain (`vite-plus`, plus its
constituent `vitest`/`oxlint`) as devDependencies.

## The contract (what `cohdl` speaks)

- `GET /packages/{name}` → `{ name, versions: [...] }`
- `GET /packages/{name}/{ver}` → `{ name, version, hash, size,
  published_at, description, license, repository, docs }`
- `GET /packages/{name}/{ver}.tar` → the content (R2)
- `POST /packages/{name}/{ver}` → publish (Bearer token). The server
  **re-computes the RFC-029 content hash itself** — its hash is the
  authoritative identity every consumer's `cohdl.lock` verifies
  (`test/hash.test.ts` pins a cross-language vector against the Rust
  implementation). It also re-reads `cohdl.toml` **from inside the
  archive**: a declared name or version that disagrees with the URL is
  refused, because the manifest is the sole identity authority.
- `PUT /packages/{name}/{ver}/docs` → upload the version's API-docs
  sidecar (`cohdl docs --publish` / the post-publish step; Bearer token,
  package owner only, the version must already be published). The body is
  the `schema_version: 1` JSON of `docs/apidocs.md`, at most 16 MiB; the
  server validates only the envelope (UTF-8 JSON, top-level object,
  `schema_version`, `package.name`/`package.version` matching the URL) —
  deep schema validation stays with the emitter, and the UI renders every
  field as inert text/SVG. Search projection safely skips malformed,
  non-public, non-part, and foreign items rather than rejecting an otherwise
  valid sidecar. Unlike the tar the sidecar is a derived,
  re-generatable view, **not** identity: re-uploading replaces it (last
  write wins), e.g. after a compiler upgrade.
- `GET /search?q={term}&kind={all|package|part}&limit={1..50}&offset={0..10000}`
  → the stable, unauthenticated CLI discovery contract. `q` is trimmed,
  contains no Unicode control characters, is at least three Unicode scalar
  values and at most 128 UTF-8 bytes. `kind` defaults to `all`, `limit` to
  20, and `offset` to 0. Package and part results are paged independently;
  each response family has `results` and `has_more`, never a total count.
  The exact fields are pinned in RFC-030. This endpoint is deliberately
  separate from the browser's existing package-only `/api/search`, whose
  response stays unchanged.
  Package names are searched in full; descriptions use and return a bounded
  prefix of 1024 Unicode scalar values.

## What a package can say about itself

Beyond `name` and `version`, `[package]` takes `license`, `description`,
and `repository` — recorded per published version (a version is one
immutable identity, so its metadata is too; anything package-level derives
from the newest version). `cohdl publish` echoes them, naming any optional
key the manifest omits.

**`license` is required to publish.** A package a design can pin is a
package whose terms that design's owner must be able to read, so a version
declaring none is refused — by the server (400) and by the CLI's pre-flight,
so a license-less package is never packed or uploaded at all. The value is
not checked against a license list: proprietary and custom terms are
legitimate, and `""` counts as silence. `description` and `repository`
stay optional.

Documents come from the source itself: every RFC-017 `#[doc("path")]`
reference is indexed at publish (`docs.ts`, mirroring `parse.rs`'s
package-relative path grammar) and rendered on the package page —
Markdown and text inline, anything else served for download.
`GET /api/doc?pkg&version&path` serves one file out of the immutable tar
with `Content-Security-Policy: sandbox` and `nosniff`; figures referenced
from inside a document resolve the same way. Rendering goes through a
Markdown subset that emits React elements only (`markdown.tsx`) — no raw
HTML, no `dangerouslySetInnerHTML`, URLs limited to http/https/mailto and
same-version relative paths.

The API-docs sidecar is served by `GET /api/apidocs?pkg&version` —
public, `Content-Type: application/json`, `nosniff`, and a ten-minute
public cache (the sidecar is replaceable, so never `immutable`); 404 when
that version has none. Each version row in `GET /api/packages/{name}`
carries `api_docs: true/false` (derived from D1's `api_docs_size`), the
UI's cue to offer the API explorer.

The same sidecar feeds part discovery. Only the most-recently-published
version is searchable, and only package-local `items` with `kind: "part"`
and `pub: true` are indexed — never private declarations or `foreign`
dependency items. Search covers the owning package name, symbol/FQ path,
device, intent, arguments/variant, and every primary or alternate AVL field
name/value within fixed projection budgets, so ordinary primary and alternate
manufacturer/MPN queries resolve. The FQ must belong to the server-derived
module root for the uploading package and end in its declared short name;
malformed or pathological excess projection data is skipped while the raw
sidecar remains available.
Uploading docs for the most-recently-published version atomically replaces
its rows; publishing a newer version clears the older index, and an
older-version docs upload never displaces the current index.

“Most-recently-published” is literal publication chronology, not the
greatest semantic version used by unversioned `cohdl add`/`cohdl update`.
Every package and part result therefore carries the exact searchable version.

After `migrations/0003-part-search.sql` (the content-addressed sidecar pointer
plus the `part_search` FTS5 trigram table) is deployed, each package's
most-recent existing R2 sidecar needs an explicit
backfill because D1 cannot derive its contents by migration alone.
Re-run `cohdl docs --publish` for each package's newest version; the sidecar
upload is idempotent and does not change the immutable package tar or hash.

- `POST /login` → token check → `{ account, official, brands }` (the CLI
  stores the grants for local publish pre-flight).

Namespace enforcement is server-side and structural (`namespace.ts`):
bare = official account only, `@brand/…` = verified brand rows in D1
(human-gated — managed by the official account), `@contrib/…` = any
authenticated account. Published versions are immutable (409 on re-publish).

## Official administration

An account whose D1 `accounts.is_official` value is `1` gets an **Admin**
link and the `/admin` dashboard. The dashboard can search accounts, create
or re-enable a verified manufacturer-brand claim, and revoke verification.
Every dashboard API independently re-resolves the web session and checks
`is_official`; hiding the link is only a UI convenience.

Brand management is deliberately conservative:

- a grant authorizes the target account to publish the entire
  `@brand/…` namespace, not just one package;
- granting a brand already claimed by another account returns a conflict
  and never transfers ownership;
- revocation sets `verified = 0` but preserves the claim, so another
  account cannot silently take the namespace;
- the dashboard cannot create another official account or move a brand
  between accounts. Those exceptional trust changes remain direct
  operator actions in D1.

Cookie-authenticated admin writes require same-origin JSON requests in
addition to the session cookie's `HttpOnly`, `Secure`, and `SameSite=Lax`
attributes. Admin responses are never cached.

## Develop / test / deploy

```sh
npm install
npm test              # vitest: hash vector, tar reader, namespace rules
npm run typecheck
npm run dev           # vite dev (workerd-backed via @cloudflare/vite-plugin)

wrangler d1 create cohdl-registry           # then paste the id into wrangler.jsonc
wrangler kv namespace create SESSIONS      # likewise
npm run db:init                             # fresh DB: the whole schema
npm run db:migrate                         # existing DB: additive column adds
npm run db:migrate:0002                    # …and the api-docs column
npm run db:migrate:0003                    # …and sidecar pointer + part-search FTS index
npm run deploy
```

`db:init` is idempotent (`CREATE TABLE IF NOT EXISTS`) and therefore does
**not** add columns to a table that already exists — an already-deployed
DB takes the one-shot `migrations/` file instead. Local dev is reached
over plain http on loopback, which is the one place the worker skips its
https redirect and HSTS header.

The official account: sign up via the UI, then set `is_official = 1` on
its D1 row (there is exactly one; never self-service).

Standalone like `editors/vscode` — the compiler crate has zero
dependencies on this directory.
