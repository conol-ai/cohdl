# cohdl.org

The CoHDL project site: landing page, documentation, blog, and use cases.
(The pre-release waitlist form is gone from the landing page since the v0.3.0
open-source release; the Worker endpoint and D1 table remain so the collected
addresses can be exported for the promised one-time announcement.)

Hand-authored static pages plus one small Worker — no framework and no build
step. The Worker fronts every request so HTTPS, HSTS and the CSP are enforced
in code rather than in zone settings, and anything that is not `/api/*` falls
through to Workers Assets (`auto-trailing-slash`, so `/docs/` serves
`docs/index.html`).

```
public/
  index.html         landing: hero, principles, pipeline, case study, install
  docs/              docs index + getting-started, language, cli, packages,
                     layout, errors, editors (one directory per page)
  docs/spec/         the language specification — GENERATED, do not hand-edit
  docs/rfcs/         RFC index + one page per RFC — GENERATED, do not hand-edit
  blog/              blog index, one directory per post, Atom feed.xml
  use-cases/         use-case index + openmicrokbd/ case study
  img/openmicrokbd/  case-study photos (WebP, from the openmicrokbd repo, MIT)
  css/home.css       isolated landing-page visual system and responsive layout
  css/style.css      shared docs/blog/use-case tokens, chrome and code highlighting
  css/prose.css      long-form styles for docs/blog/use-case pages
  js/home.js         progressive motion, navigation, pipeline scroll stack and copy control
  js/main.js         waitlist progressive enhancement (no longer referenced
                     by the landing page; kept with the endpoint)
  logo.svg, og.png   landing mark and social preview card
  404.html, favicon.svg, robots.txt, sitemap.xml
src/worker.ts        security headers + POST /api/waitlist
schema.sql           the D1 waitlist table
```

The long-form pages use the registry's dark canvas, accent, grid and compact
chrome. The landing page has its own expanded marketing system in `home.css`
and `home.js`; keeping those files isolated prevents its light/dark chapters
and motion treatments from leaking into documentation. Every long-form page
copies the same masthead/footer chrome from `docs/index.html`; there is no
templating, so a chrome change means editing each page (deliberate — the page
count is small and the deploy story stays trivial).

Editing notes:

- New pages copy the `<head>` pattern, masthead and footer of an existing
  page, set their own title/description/canonical, and put `aria-current` on
  their section's nav link.
- CoHDL code samples are hand-highlighted with `c-kw`/`c-ty`/`c-at`/`c-unit`/
  `c-cm`/`c-str` spans; terminal output stays unhighlighted.
- A new blog post gets its own directory, an entry at the top of
  `blog/index.html`, and an `<entry>` in `blog/feed.xml`.
- Add new pages to `sitemap.xml`.
- `docs/spec/` and `docs/rfcs/` are generated verbatim from `docs/design/` by
  `site/tools/gen_design_docs.py` (chrome template + curated summaries live in
  the script). After a design-repo re-extract, re-run it and commit the diff;
  it fails loudly on any Markdown construct it does not handle. A new RFC
  means adding its filename and one-line summary to the script's lists.

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

The `cohdl-site` D1 database already exists and its id is in `wrangler.jsonc`
(wrangler provisioned it on 2026-08-12), but it is still empty:

```sh
npm run db:init                          # create the waitlist table remotely
npx wrangler secret put IP_SALT          # any long random string
npm run deploy
```

Recreating it from scratch elsewhere means `npx wrangler d1 create cohdl-site`
and pasting the printed id over `database_id`.

`IP_SALT` is optional but recommended: it salts the SHA-256 that stands in for
the client IP. Without it a constant fallback is used, which still avoids
storing raw addresses but is weaker against an offline guess of which IPs
signed up.

The `routes` block claims `cohdl.org` and `www.cohdl.org` as custom domains,
so the zone must exist on the Cloudflare account; wrangler provisions the DNS
records and certificates itself.

## The waitlist (retired form, live data)

The pre-release landing page carried a signup form; it was removed with the
v0.3.0 open-source release, so nothing on the site posts to
`POST /api/waitlist` any longer. The endpoint, `js/main.js` and the D1 table
are kept as they were — the endpoint still behaves correctly if hit directly
(strict shape check, idempotent insert, no enumeration oracle, honeypot,
5/IP/hour rate limit, salted IP hash) — because the collected addresses are
still owed their one promised announcement email.

Reading the list for that send:

```sh
npm run waitlist:count
npm run waitlist:export     # email, created_at, source as JSON
```

The `source` column holds the `?ref=…` attribution captured while the form
was live. Once the announcement has gone out, the endpoint, `js/main.js`,
`schema.sql` and the D1 binding can all be deleted together.

## CI

The root workflow's `site` job typechecks the Worker and builds its bundle
with `wrangler deploy --dry-run`. Deploys are manual and need no credentials
in CI.
