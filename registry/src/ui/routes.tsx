// Code-based TanStack Router route tree. The package route is a splat —
// scoped names (`@sparkfun/power`) contain a `/`.

import React, { useState } from "react";
import {
  Link,
  Outlet,
  createRootRoute,
  createRoute,
  useNavigate,
} from "@tanstack/react-router";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  del,
  docUrl,
  post,
  put,
  useAdminAccounts,
  useConfig,
  useDocText,
  useMe,
  usePackage,
  useRecent,
  useSearch,
  type AdminAccount,
  type VersionRow,
} from "./api";
import { Markdown } from "./markdown";

// reCAPTCHA v3 (loaded on demand when the registry has a site key
// configured; see /api/config). Keys live in the Cloudflare dashboard.
declare global {
  interface Window {
    grecaptcha?: {
      ready(cb: () => void): void;
      execute(siteKey: string, opts: { action: string }): Promise<string>;
    };
  }
}

let recaptchaLoaded: Promise<void> | null = null;
function loadRecaptcha(siteKey: string): Promise<void> {
  recaptchaLoaded ??= new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = `https://www.google.com/recaptcha/api.js?render=${encodeURIComponent(siteKey)}`;
    s.onload = () => window.grecaptcha!.ready(resolve);
    s.onerror = () => reject(new Error("could not load reCAPTCHA"));
    document.head.appendChild(s);
  });
  return recaptchaLoaded;
}

async function recaptchaToken(siteKey: string | null | undefined, action: string) {
  if (!siteKey) return undefined;
  await loadRecaptcha(siteKey);
  return window.grecaptcha!.execute(siteKey, { action });
}

// ---------------------------------------------------------------------------

function TierBadge({ tier }: { tier: string }) {
  const label = tier === "official" ? "official" : tier === "brand" ? "manufacturer" : "community";
  return <span className={`tier tier-${tier}`}>{label}</span>;
}

function Layout() {
  const [q, setQ] = useState("");
  const navigate = useNavigate();
  const me = useMe();
  return (
    <div className="shell">
      <header>
        <Link to="/" className="logo">
          <span className="logo-mark">⌁</span> registry.cohdl.org
        </Link>
        <form
          onSubmit={(e) => {
            e.preventDefault();
            navigate({ to: "/", search: { q } });
          }}
        >
          <input
            name="q"
            aria-label="search packages"
            placeholder="search packages…"
            value={q}
            onChange={(e) => setQ(e.target.value)}
          />
        </form>
        <nav>
          <Link to="/docs">Docs</Link>
          {me.data?.official && <Link to="/admin">Admin</Link>}
          <Link to="/account">Account</Link>
        </nav>
      </header>
      <main>
        <Outlet />
      </main>
      <footer>
        the CoHDL package registry — RFC-030 · three-tier namespace: bare = official ·
        @brand/name = verified manufacturer · @contrib/name = community
      </footer>
    </div>
  );
}

const rootRoute = createRootRoute({ component: Layout });

// ---------------------------------------------------------------------------

function Home() {
  const { q } = homeRoute.useSearch();
  const search = useSearch(q ?? "");
  const recent = useRecent();
  return (
    <>
      <section className="hero">
        <h1>Packages you can pin to the byte.</h1>
        <p>
          Every published version is content-hashed server-side; <code>cohdl.lock</code>{" "}
          verifies it on every build. Exact versions only — no ranges, ever.
        </p>
        <pre className="snippet">cohdl add @contrib/your-first-package</pre>
      </section>
      {q ? (
        <section>
          <h2>Results for “{q}”</h2>
          <ul className="pkg-list">
            {(search.data?.results ?? []).map((r) => (
              <li key={r.name}>
                <Link to="/package/$" params={{ _splat: r.name }}>
                  {r.name}
                </Link>{" "}
                <TierBadge tier={r.tier} /> <span className="ver">{r.latest}</span>
                {r.description && <div className="pkg-blurb">{r.description}</div>}
              </li>
            ))}
            {search.data?.results.length === 0 && <li className="muted">nothing published under that name</li>}
          </ul>
        </section>
      ) : (
        <section>
          <h2>Recently published</h2>
          <ul className="pkg-list">
            {(recent.data?.results ?? []).map((r) => (
              <li key={`${r.name}@${r.version}`}>
                <Link to="/package/$" params={{ _splat: r.name }}>
                  {r.name}
                </Link>{" "}
                <TierBadge tier={r.tier} /> <span className="ver">{r.version}</span>
                {r.description && <div className="pkg-blurb">{r.description}</div>}
              </li>
            ))}
            {recent.data?.results.length === 0 && (
              <li className="muted">nothing published yet — be the first: `cohdl publish`</li>
            )}
          </ul>
        </section>
      )}
    </>
  );
}

const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/",
  validateSearch: (s: Record<string, unknown>): { q?: string } =>
    typeof s.q === "string" && s.q ? { q: s.q } : {},
  component: Home,
});

// ---------------------------------------------------------------------------

/// Renderable in place (text) vs. linked (a datasheet PDF, a figure).
function isTextDoc(path: string) {
  return /\.(md|markdown|txt)$/i.test(path);
}

/// A README-ish document sorts first — it is what a visitor came to read.
function pickFirstDoc(docs: string[]): string | null {
  if (docs.length === 0) return null;
  const readme = docs.find((d) => /(^|\/)readme\.(md|markdown|txt)$/i.test(d));
  return readme ?? docs.find(isTextDoc) ?? docs[0];
}

/// The RFC-017 documents a version ships, rendered from the package's own
/// archive. Relative links and figures resolve within the same version.
function Documents({ name, version }: { name: string; version: VersionRow }) {
  const docs = version.docs;
  const [selected, setSelected] = useState<string | null>(null);
  const active = selected ?? pickFirstDoc(docs);
  const text = useDocText(name, version.version, active && isTextDoc(active) ? active : null);
  if (docs.length === 0) return null;

  // A document's own relative references are relative to ITS directory.
  const dir = active && active.includes("/") ? active.slice(0, active.lastIndexOf("/") + 1) : "";
  const resolve = (path: string) => {
    const joined = `${dir}${path}`.split("/").reduce<string[]>((acc, seg) => {
      if (seg === "." || seg === "") return acc;
      if (seg === "..") {
        acc.pop();
        return acc;
      }
      acc.push(seg);
      return acc;
    }, []);
    return joined.length > 0 ? docUrl(name, version.version, joined.join("/")) : null;
  };

  // A document that links to a sibling document switches documents here
  // rather than sending the reader to raw Markdown.
  const onLinkClick = (e: React.MouseEvent) => {
    const anchor = (e.target as HTMLElement).closest("a");
    const href = anchor?.getAttribute("href");
    if (!href?.startsWith("/api/doc?")) return;
    const target = new URLSearchParams(href.slice(href.indexOf("?") + 1)).get("path");
    if (target && docs.includes(target) && isTextDoc(target)) {
      e.preventDefault();
      setSelected(target);
    }
  };

  return (
    <section onClick={onLinkClick}>
      <h2>
        Documents <span className="muted doc-ver">of {version.version}</span>
      </h2>
      {docs.length > 1 && (
        <nav className="doc-tabs">
          {docs.map((d) => (
            <button
              key={d}
              className={d === active ? "doc-tab doc-tab-on" : "doc-tab"}
              onClick={() => setSelected(d)}
            >
              {d}
            </button>
          ))}
        </nav>
      )}
      {active && !isTextDoc(active) ? (
        <p>
          <a href={docUrl(name, version.version, active)} rel="noopener noreferrer" target="_blank">
            open {active}
          </a>{" "}
          <span className="muted">— not a text document, so it is served, not rendered</span>
        </p>
      ) : text.isLoading ? (
        <p className="muted">loading {active}…</p>
      ) : text.isError ? (
        <p className="error">{(text.error as Error).message}</p>
      ) : (
        <Markdown source={text.data ?? ""} resolve={resolve} />
      )}
    </section>
  );
}

function PackagePage() {
  const { _splat } = packageRoute.useParams();
  const name = _splat ?? "";
  const pkg = usePackage(name);
  if (pkg.isLoading) return <p className="muted">loading…</p>;
  if (pkg.isError || !pkg.data) return <p className="muted">`{name}` is not published here.</p>;
  const latest = pkg.data.versions[0];
  const repo = latest?.repository;
  // Only http(s) repository links are turned into anchors; anything else is
  // shown as the literal text the publisher wrote.
  const repoHref = repo && /^https?:\/\//i.test(repo) ? repo : null;
  return (
    <>
      <h1>
        {name} <TierBadge tier={pkg.data.tier} />
      </h1>
      {latest?.description && <p className="pkg-desc">{latest.description}</p>}
      {(latest?.license || repo) && (
        <p className="muted pkg-meta">
          {latest?.license && <>license {latest.license}</>}
          {latest?.license && repo && " · "}
          {repo &&
            (repoHref ? (
              <a href={repoHref} rel="noopener noreferrer nofollow" target="_blank">
                {repo}
              </a>
            ) : (
              <>{repo}</>
            ))}
        </p>
      )}
      <pre className="snippet">
        cohdl add {name}
        {latest ? `@${latest.version}` : ""}
      </pre>
      {latest && <Documents name={name} version={latest} />}
      <h2>Versions</h2>
      <table>
        <thead>
          <tr>
            <th>version</th>
            <th>published</th>
            <th>size</th>
            <th>content hash (what cohdl.lock verifies)</th>
          </tr>
        </thead>
        <tbody>
          {pkg.data.versions.map((v) => (
            <tr key={v.version}>
              <td>{v.version}</td>
              <td>{v.published_at.slice(0, 10)}</td>
              <td>{v.size} B</td>
              <td>
                <code className="hash">{v.hash}</code>
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </>
  );
}

const packageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/package/$",
  component: PackagePage,
});

// ---------------------------------------------------------------------------

function Account() {
  const me = useMe();
  const config = useConfig();
  const qc = useQueryClient();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [token, setToken] = useState<string | null>(null);

  const auth = useMutation({
    mutationFn: async () => {
      const action = mode === "signup" ? "signup" : "login";
      const recaptcha = await recaptchaToken(config.data?.recaptcha_site_key, action);
      return post<{ account: string }>(mode === "signup" ? "/api/signup" : "/api/session", {
        email,
        password,
        recaptcha,
      });
    },
    onSuccess: (data) => {
      // Flip to the signed-in view immediately from the mutation's own
      // result; the invalidate refreshes grants in the background. (The
      // original invalidate-only flow could leave the form up silently.)
      qc.setQueryData(["me"], { account: data.account, official: false, brands: [] });
      qc.invalidateQueries({ queryKey: ["me"] });
    },
  });
  const mintToken = useMutation({
    mutationFn: () => post<{ token: string }>("/api/tokens", {}),
    onSuccess: (d) => setToken(d.token),
  });

  if (me.data) {
    return (
      <>
        <h1>{me.data.account}</h1>
        <p>
          {me.data.official ? "CoHDL official account. " : ""}
          {me.data.brands.length > 0
            ? `Verified brands: ${me.data.brands.join(", ")}.`
            : "No verified brands — brand verification is human-gated; see Docs."}
        </p>
        <h2>CLI tokens</h2>
        <p className="muted">
          A token is shown exactly once; `cohdl login` stores it in ~/.cohdl/credentials.toml.
        </p>
        <button onClick={() => mintToken.mutate()} disabled={mintToken.isPending}>
          Create token
        </button>
        {token && <pre className="snippet">{token}</pre>}
      </>
    );
  }
  return (
    <>
      <h1>{mode === "signup" ? "Create an account" : "Sign in"}</h1>
      <form
        className="auth"
        onSubmit={(e) => {
          e.preventDefault();
          auth.mutate();
        }}
      >
        <input placeholder="email" value={email} onChange={(e) => setEmail(e.target.value)} />
        <input
          placeholder="password"
          type="password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <button type="submit" disabled={auth.isPending}>
          {mode === "signup" ? "Sign up" : "Sign in"}
        </button>
        {auth.isError && (
          <p className="error">
            {(auth.error as Error).message}
            {mode === "signup" && (auth.error as Error).message.includes("already exists") && (
              <> — <button type="button" className="linkish" onClick={() => setMode("signin")}>sign in instead</button></>
            )}
          </p>
        )}
      </form>
      <button className="linkish" onClick={() => setMode(mode === "signup" ? "signin" : "signup")}>
        {mode === "signup" ? "Have an account? Sign in" : "New here? Create an account"}
      </button>
    </>
  );
}

const accountRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/account",
  component: Account,
});

// ---------------------------------------------------------------------------

function AdminDashboard({ accountEmail }: { accountEmail: string }) {
  const qc = useQueryClient();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [brand, setBrand] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const accountsQuery = useAdminAccounts(search);
  const accounts = accountsQuery.data?.accounts ?? [];
  const selected =
    accounts.find((account) => account.id === selectedId) ??
    accounts.find((account) => account.email === accountEmail) ??
    accounts[0];

  const refreshGrants = () =>
    Promise.all([
      qc.invalidateQueries({ queryKey: ["admin", "accounts"] }),
      qc.invalidateQueries({ queryKey: ["me"] }),
    ]);

  const grant = useMutation({
    mutationFn: ({ account, scope }: { account: AdminAccount; scope: string }) =>
      put<unknown>(`/api/admin/accounts/${account.id}/brands`, { brand: scope }),
    onSuccess: async (_data, { account, scope }) => {
      setBrand("");
      setStatus(
        `Verified @${scope} for ${account.email}. The account may now publish every package under @${scope}/*.`,
      );
      await refreshGrants();
    },
  });

  const revoke = useMutation({
    mutationFn: ({ account, scope }: { account: AdminAccount; scope: string }) =>
      del<unknown>(`/api/admin/accounts/${account.id}/brands`, { brand: scope }),
    onSuccess: async (_data, { account, scope }) => {
      setStatus(
        `Revoked @${scope} verification from ${account.email}. The unverified claim is preserved.`,
      );
      await refreshGrants();
    },
  });

  const busy = grant.isPending || revoke.isPending;
  const verified = selected?.brands.filter((claim) => claim.verified) ?? [];
  const unverified = selected?.brands.filter((claim) => !claim.verified) ?? [];

  const clearFeedback = () => {
    setStatus(null);
    grant.reset();
    revoke.reset();
  };

  return (
    <>
      <h1>Registry administration</h1>
      <p className="muted">
        Manufacturer namespace verification is human-gated. Select an account, then grant or revoke
        its exact brand scope.
      </p>

      <section className="admin-card" aria-labelledby="account-heading">
        <h2 id="account-heading">Account</h2>
        <form
          className="admin-search"
          role="search"
          onSubmit={(e) => {
            e.preventDefault();
            clearFeedback();
            setSelectedId(null);
            setSearch(searchInput.trim());
          }}
        >
          <label htmlFor="admin-account-search">Find an account</label>
          <div className="admin-inline">
            <input
              id="admin-account-search"
              type="search"
              placeholder="email address"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              disabled={busy}
            />
            <button type="submit" disabled={busy || accountsQuery.isFetching}>
              {accountsQuery.isFetching ? "Searching…" : "Search"}
            </button>
            {search && (
              <button
                type="button"
                className="button-secondary"
                disabled={busy}
                onClick={() => {
                  clearFeedback();
                  setSearchInput("");
                  setSearch("");
                  setSelectedId(null);
                }}
              >
                Clear
              </button>
            )}
          </div>
        </form>

        {accountsQuery.isPending ? (
          <p className="muted">loading accounts…</p>
        ) : accountsQuery.isError ? (
          <p className="error" role="alert">
            {(accountsQuery.error as Error).message}
          </p>
        ) : accounts.length === 0 ? (
          <p className="muted">No accounts match this search.</p>
        ) : (
          <>
            {accountsQuery.data?.truncated && (
              <p className="field-help">
                Showing the first 100 accounts. Refine the email search to find another account.
              </p>
            )}
            <div className="admin-field">
              <label htmlFor="admin-account">Manage account</label>
              <select
                id="admin-account"
                value={selected?.id ?? ""}
                disabled={busy}
                onChange={(e) => {
                  clearFeedback();
                  setSelectedId(Number(e.target.value));
                }}
              >
                {accounts.map((account) => (
                  <option key={account.id} value={account.id}>
                    {account.email}
                    {account.official ? " — official" : ""}
                  </option>
                ))}
              </select>
            </div>
          </>
        )}
      </section>

      {selected && (
        <section className="admin-card" aria-labelledby="brands-heading" aria-busy={busy}>
          <div className="admin-card-heading">
            <div>
              <h2 id="brands-heading">Brand claims</h2>
              <p className="admin-account-email">
                {selected.email}{" "}
                {selected.official && <span className="tier tier-official">official</span>}
              </p>
            </div>
          </div>

          <form
            className="brand-grant"
            onSubmit={(e) => {
              e.preventDefault();
              const scope = brand.trim();
              if (!scope || !selected) return;
              clearFeedback();
              grant.mutate({ account: selected, scope });
            }}
          >
            <label htmlFor="admin-brand">Brand scope (without @)</label>
            <div className="scope-input">
              <span aria-hidden="true">@</span>
              <input
                id="admin-brand"
                value={brand}
                onChange={(e) => setBrand(e.target.value)}
                pattern="[A-Za-z0-9_][A-Za-z0-9_-]*"
                title="Use letters, digits, underscores, and hyphens; the first character cannot be a hyphen."
                autoCapitalize="none"
                autoComplete="off"
                spellCheck={false}
                required
                disabled={busy}
                aria-describedby="brand-scope-help brand-scope-warning"
              />
              <button type="submit" disabled={busy || !brand.trim()}>
                {grant.isPending ? "Granting…" : "Grant verification"}
              </button>
            </div>
            <p id="brand-scope-help" className="field-help">
              Enter the exact, case-sensitive scope, for example <code>espressif</code>.
            </p>
            <p id="brand-scope-warning" className="admin-warning">
              <strong>Namespace-wide access:</strong> a grant authorizes this account to publish
              every package under <code>@{brand.trim() || "brand"}/*</code>, not just one package.
            </p>
          </form>

          <div className="claim-group">
            <h3>Verified</h3>
            {verified.length === 0 ? (
              <p className="muted">No verified brands.</p>
            ) : (
              <ul className="brand-claims">
                {verified.map((claim) => (
                  <li key={claim.brand}>
                    <code>@{claim.brand}/*</code>
                    <button
                      type="button"
                      className="button-danger"
                      disabled={busy}
                      aria-label={`Revoke @${claim.brand} verification from ${selected.email}`}
                      onClick={() => {
                        const confirmed = window.confirm(
                          `Revoke @${claim.brand} verification from ${selected.email}? This immediately removes publish access, but preserves the unverified claim.`,
                        );
                        if (!confirmed) return;
                        clearFeedback();
                        revoke.mutate({ account: selected, scope: claim.brand });
                      }}
                    >
                      {revoke.isPending && revoke.variables?.scope === claim.brand
                        ? "Revoking…"
                        : "Revoke"}
                    </button>
                  </li>
                ))}
              </ul>
            )}
          </div>

          <div className="claim-group">
            <h3>Unverified claims</h3>
            {unverified.length === 0 ? (
              <p className="muted">No unverified claims.</p>
            ) : (
              <ul className="brand-claims brand-claims-inactive">
                {unverified.map((claim) => (
                  <li key={claim.brand}>
                    <code>@{claim.brand}/*</code>
                    <span className="muted">publishing disabled; claim preserved</span>
                  </li>
                ))}
              </ul>
            )}
          </div>

          {status && (
            <p role="status" className="success">
              {status}
            </p>
          )}
          {grant.isError && (
            <p role="alert" className="error">
              Could not grant verification: {(grant.error as Error).message}
            </p>
          )}
          {revoke.isError && (
            <p role="alert" className="error">
              Could not revoke verification: {(revoke.error as Error).message}
            </p>
          )}
        </section>
      )}
    </>
  );
}

function Admin() {
  const me = useMe();
  if (me.isPending) {
    return (
      <>
        <h1>Registry administration</h1>
        <p className="muted">checking account access…</p>
      </>
    );
  }
  if (!me.data) {
    return (
      <>
        <h1>Registry administration</h1>
        <p>
          <Link to="/account">Sign in</Link> with the CoHDL official account to continue.
        </p>
      </>
    );
  }
  if (!me.data.official) {
    return (
      <>
        <h1>Registry administration</h1>
        <p className="error" role="alert">
          Official account access is required. You are signed in as {me.data.account}.
        </p>
      </>
    );
  }
  return <AdminDashboard accountEmail={me.data.account} />;
}

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: Admin,
});

// ---------------------------------------------------------------------------

const MANIFEST_EXAMPLE = `[package]
name = "@contrib/your-name"
version = "1.0.0"
license = "MIT"                 # required
description = "One line: what the package gives a design."
repository = "https://github.com/you/your-name"`;

function Docs() {
  return (
    <>
      <h1>Publishing to registry.cohdl.org</h1>
      <ol className="docs">
        <li>
          Create an account and a CLI token on the <Link to="/account">Account</Link> page,
          then <code>cohdl login</code> and paste it.
        </li>
        <li>
          Name your package by trust tier (the name's shape IS its tier — RFC-030):
          <ul>
            <li>
              <code>@contrib/your-name</code> — open community namespace,
              first-come-first-served.
            </li>
            <li>
              <code>@yourbrand/name</code> — requires a{" "}
              <strong id="brands">verified manufacturer account</strong>: verification is
              human-gated; contact the registry maintainers with proof you represent the
              named company.
            </li>
            <li>
              bare names (<code>std</code>, <code>sensors</code>) are reserved for CoHDL's
              own packages — never first-come-first-served.
            </li>
          </ul>
        </li>
        <li>
          Describe the package in its own <code>cohdl.toml</code>. Alongside{" "}
          <code>name</code> and <code>version</code>, the <code>[package]</code> section
          takes three metadata keys — none of them affect a verdict or an emitted byte:
          <pre className="snippet">{MANIFEST_EXAMPLE}</pre>
          <strong>
            <code>license</code> is required
          </strong>{" "}
          — a package you can pin into a board you manufacture is a package whose terms
          you must be able to read, so a version that declares none is refused (any
          value is accepted, including proprietary terms; what the registry refuses is
          silence). <code>description</code> and <code>repository</code> are optional.
          All three are recorded per published version (a version is one immutable
          identity, so its metadata is too) and shown here.
        </li>
        <li>
          Ship documents with the package. Every RFC-017{" "}
          <code>#[doc("path")]</code> reference in your source — a README, a datasheet, an
          errata note — is indexed at publish and rendered on the package page; Markdown
          and text render inline, anything else is served for download. Paths are
          package-relative, so a document's own figures resolve too.
        </li>
        <li>
          <code>cohdl publish</code> from the package directory. The registry re-computes
          the RFC-029 content hash server-side — its hash is the authoritative identity
          every consumer's <code>cohdl.lock</code> will verify. It also re-reads your
          manifest from inside the archive: what a package declares about itself is what
          gets published, so a name or version that disagrees with the publish is refused.
        </li>
        <li>
          Consumers: <code>cohdl add {"<name>"}</code>, <code>cohdl install</code> on a
          fresh clone, <code>cohdl update</code> to move pins — exact versions only.
        </li>
      </ol>
    </>
  );
}

const docsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: Docs,
});

export const routeTree = rootRoute.addChildren([homeRoute, packageRoute, accountRoute, adminRoute, docsRoute]);
