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
- `GET /packages/{name}/{ver}` → `{ name, version, hash, size, published_at }`
- `GET /packages/{name}/{ver}.tar` → the content (R2)
- `POST /packages/{name}/{ver}` → publish (Bearer token). The server
  **re-computes the RFC-029 content hash itself** — its hash is the
  authoritative identity every consumer's `cohdl.lock` verifies
  (`test/hash.test.ts` pins a cross-language vector against the Rust
  implementation).
- `POST /login` → token check → `{ account, official, brands }` (the CLI
  stores the grants for local publish pre-flight).

Namespace enforcement is server-side and structural (`namespace.ts`):
bare = official account only, `@brand/…` = verified brand rows in D1
(human-gated — set `brands.verified` manually), `@contrib/…` = any
authenticated account. Published versions are immutable (409 on
re-publish).

## Develop / test / deploy

```sh
npm install
npm test              # vitest: hash vector, tar reader, namespace rules
npm run typecheck
npm run dev           # vite dev (workerd-backed via @cloudflare/vite-plugin)

wrangler d1 create cohdl-registry           # then paste the id into wrangler.jsonc
wrangler kv namespace create SESSIONS      # likewise
npm run db:init
npm run deploy
```

The official account: sign up via the UI, then set `is_official = 1` on
its D1 row (there is exactly one; never self-service).

Standalone like `editors/vscode` — the compiler crate has zero
dependencies on this directory.
