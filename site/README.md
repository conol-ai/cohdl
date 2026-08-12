# cohdl.org

The CoHDL project site: a pre-launch page and the waitlist that collects
addresses until the compiler's source is public.

A static page plus one small Worker — no framework and no build step. The
Worker fronts every request so HTTPS, HSTS and the CSP are enforced in code
rather than in zone settings, and anything that is not `/api/*` falls through
to Workers Assets.

```
public/          the page — index.html, css/, js/, favicon, robots, sitemap
src/worker.ts    security headers + POST /api/waitlist
schema.sql       the D1 waitlist table
```

The visual system is the registry's: same dark canvas, accent, grid and brand
mark, so cohdl.org and registry.cohdl.org read as one product.

## Local development

```sh
npm install
npm run db:init:local     # create the waitlist table in the local D1
npm run dev               # https://localhost:8788
```

`npm run dev` deliberately serves **https**. `wrangler dev` presents requests
using the hostname from the `routes` block (`cohdl.org`), not `localhost`, so
the Worker's loopback exemption never applies locally and a plain-http dev
server would be caught by the http→https redirect and loop. Your browser will
warn about wrangler's self-signed certificate; that is expected.

## First deploy

The D1 database has to exist before the first deploy, and its id has to be
written into `wrangler.jsonc`:

```sh
npx wrangler d1 create cohdl-site        # copy the printed database_id
#   -> paste it over REPLACE_WITH_D1_DATABASE_ID in wrangler.jsonc
npm run db:init                          # create the table remotely
npx wrangler secret put IP_SALT          # any long random string
npm run deploy
```

`IP_SALT` is optional but recommended: it salts the SHA-256 that stands in for
the client IP. Without it a constant fallback is used, which still avoids
storing raw addresses but is weaker against an offline guess of which IPs
signed up.

The `routes` block claims `cohdl.org` and `www.cohdl.org` as custom domains,
so the zone must exist on the Cloudflare account; wrangler provisions the DNS
records and certificates itself.

## The waitlist

`POST /api/waitlist` accepts either JSON (`{"email": "…"}`) or a plain form
post, so the form works with JavaScript disabled — without JS the Worker
answers with a redirect carrying the outcome in the query string, and
`js/main.js` submits in place when it is available.

What it does with a submission:

- trims and lower-cases the address, and rejects anything that fails a
  deliberately strict shape check (no whitespace or list delimiters);
- stores it with `ON CONFLICT DO NOTHING`, so a repeat signup is idempotent;
- answers identically whether or not the address was already stored — a
  waitlist should not double as an address-enumeration oracle;
- silently accepts honeypot submissions, so a bot records success and leaves;
- allows 5 signups per IP per hour;
- keeps a salted hash of the IP, never the address itself.

Reading the list:

```sh
npm run waitlist:count
npm run waitlist:export     # email, created_at, source as JSON
```

`?ref=…` on the landing page is carried into the `source` column, so a link
shared somewhere specific can be attributed later.

## CI

The root workflow's `site` job typechecks the Worker and builds its bundle
with `wrangler deploy --dry-run`. Deploys are manual and need no credentials
in CI.
