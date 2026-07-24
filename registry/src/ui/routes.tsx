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
import { post, usePackage, useMe, useRecent, useSearch } from "./api";

// ---------------------------------------------------------------------------

function TierBadge({ tier }: { tier: string }) {
  const label = tier === "official" ? "official" : tier === "brand" ? "manufacturer" : "community";
  return <span className={`tier tier-${tier}`}>{label}</span>;
}

function Layout() {
  const [q, setQ] = useState("");
  const navigate = useNavigate();
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
            placeholder="search packages…"
            value={q}
            onChange={(e) => setQ(e.target.value)}
          />
        </form>
        <nav>
          <Link to="/docs">Docs</Link>
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

function PackagePage() {
  const { _splat } = packageRoute.useParams();
  const name = _splat ?? "";
  const pkg = usePackage(name);
  if (pkg.isLoading) return <p className="muted">loading…</p>;
  if (pkg.isError || !pkg.data) return <p className="muted">`{name}` is not published here.</p>;
  const latest = pkg.data.versions[0];
  return (
    <>
      <h1>
        {name} <TierBadge tier={pkg.data.tier} />
      </h1>
      <pre className="snippet">
        cohdl add {name}
        {latest ? `@${latest.version}` : ""}
      </pre>
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
  const qc = useQueryClient();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [token, setToken] = useState<string | null>(null);

  const auth = useMutation({
    mutationFn: () =>
      post<{ account: string }>(mode === "signup" ? "/api/signup" : "/api/session", {
        email,
        password,
      }),
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
          <code>cohdl publish</code> from the package directory. The registry re-computes
          the RFC-029 content hash server-side — its hash is the authoritative identity
          every consumer's <code>cohdl.lock</code> will verify.
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

export const routeTree = rootRoute.addChildren([homeRoute, packageRoute, accountRoute, docsRoute]);
