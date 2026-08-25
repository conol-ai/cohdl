// Code-based TanStack Router route tree. The package route is a splat —
// scoped names (`@sparkfun/power`) contain a `/`.

import React, { useEffect, useRef, useState } from "react";
import {
  Link,
  Outlet,
  createRootRoute,
  createRoute,
  useNavigate,
  useRouterState,
  type ErrorComponentProps,
} from "@tanstack/react-router";
import { useMutation, useQueryClient } from "@tanstack/react-query";
import {
  del,
  docUrl,
  isUnauthorized,
  post,
  put,
  useAdminAccounts,
  useApiDocs,
  useConfig,
  useDocText,
  useMe,
  usePackage,
  useRecent,
  useSearch,
  type AdminAccount,
  type Me,
  type SearchRow,
  type VersionRow,
} from "./api";
import { ApiExplorer } from "./apidocs";
import {
  BrandMark,
  CommandBox,
  CopyButton,
  Icon,
  LoadingRows,
  PackageCard,
  StatePanel,
  TierBadge,
  formatDate,
  formatSize,
} from "./components";
import { Markdown } from "./markdown";
import { recaptchaToken } from "./recaptcha";
import {
  AdminComponentRequestsPage,
  ComponentRequestPage,
} from "./component-requests";

// ---------------------------------------------------------------------------

function usePageTitle(title: string) {
  useEffect(() => {
    document.title = `${title} · CoHDL Registry`;
  }, [title]);
}

function GlobalSearch() {
  const routeQuery = useRouterState({
    select: (state) => {
      // Only the catalogue's own `q` belongs in this box — the package
      // page's API explorer reuses the `q` param for its item filter, and
      // mirroring that here would read as a site-wide search in flight.
      const search = state.location.search as Record<string, unknown>;
      const mirrors = state.location.pathname === "/packages" || state.location.pathname === "/";
      return mirrors && typeof search.q === "string" ? search.q : "";
    },
  });
  const [q, setQ] = useState(routeQuery);
  const navigate = useNavigate();
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => setQ(routeQuery), [routeQuery]);
  useEffect(() => {
    const focusSearch = (event: KeyboardEvent) => {
      if (
        event.key === "/" &&
        !(event.target instanceof HTMLInputElement) &&
        !(event.target instanceof HTMLTextAreaElement) &&
        !(event.target instanceof HTMLSelectElement)
      ) {
        event.preventDefault();
        input.current?.focus();
      }
    };
    window.addEventListener("keydown", focusSearch);
    return () => window.removeEventListener("keydown", focusSearch);
  }, []);

  return (
    <form
      className="global-search"
      role="search"
      aria-label="Site package search"
      onSubmit={(event) => {
        event.preventDefault();
        navigate({
          to: "/packages",
          search: { q: q.trim() || undefined },
        });
      }}
    >
      <Icon name="search" size={18} />
      <input
        ref={input}
        name="q"
        aria-label="Search packages"
        placeholder="Search components, footprints, libraries…"
        value={q}
        onChange={(event) => setQ(event.target.value)}
      />
      <kbd aria-hidden="true">/</kbd>
      <button type="submit" aria-label="Submit package search">
        Search
      </button>
    </form>
  );
}

function Layout() {
  const me = useMe();
  const pathname = useRouterState({ select: (state) => state.location.pathname });
  const main = useRef<HTMLElement>(null);
  const previousPath = useRef(pathname);

  useEffect(() => {
    if (previousPath.current !== pathname) {
      main.current?.focus({ preventScroll: true });
      previousPath.current = pathname;
    }
  }, [pathname]);

  return (
    <>
      <a className="skip-link" href="#main">
        Skip to content
      </a>
      <div className="spectral-rule" aria-hidden="true" />
      <header className="site-header">
        <div className="utility-bar">
          <div className="shell utility-inner">
            <span>
              <span className="signal-dot" aria-hidden="true" />
              Content-addressed hardware libraries
            </span>
            <span className="utility-proof">Exact versions · server-verified hashes</span>
          </div>
        </div>
        <div className="shell header-main">
          <Link to="/" className="wordmark" activeOptions={{ exact: true }}>
            <BrandMark />
            <span>
              <strong>CoHDL</strong>
              <small>Registry</small>
            </span>
          </Link>
          <GlobalSearch />
          <nav className="primary-nav" aria-label="Primary navigation">
            <Link
              to="/packages"
              activeProps={{ className: "is-active", "aria-current": "page" }}
            >
              Packages
            </Link>
            <Link to="/docs" activeProps={{ className: "is-active", "aria-current": "page" }}>
              Docs
            </Link>
            {me.data?.official && (
              <Link
                to="/admin"
                activeProps={{ className: "is-active", "aria-current": "page" }}
              >
                Admin
              </Link>
            )}
            <Link
              to="/account"
              className="account-link"
              activeProps={{
                className: "account-link is-active",
                "aria-current": "page",
              }}
            >
              <span className="account-orb" aria-hidden="true">
                {(me.data?.account ?? "A").slice(0, 1).toUpperCase()}
              </span>
              <span>{me.data ? "Account" : "Sign in"}</span>
            </Link>
          </nav>
        </div>
      </header>
      <main ref={main} id="main" className="shell main-content" tabIndex={-1}>
        <Outlet />
      </main>
      <footer className="site-footer">
        <div className="shell footer-grid">
          <div className="footer-brand">
            <BrandMark size={30} />
            <div>
              <strong>CoHDL Registry</strong>
              <p>Trusted hardware libraries, immutable by design.</p>
            </div>
          </div>
          <nav aria-label="Footer navigation">
            <Link to="/packages">Packages</Link>
            <Link to="/request">Request a component</Link>
            <Link to="/docs">Publishing guide</Link>
            <Link to="/account">Account</Link>
            <a href="https://github.com/conol-ai/cohdl" rel="noopener noreferrer" target="_blank">
              GitHub <Icon name="external" size={13} />
            </a>
          </nav>
        </div>
        <div className="shell footer-bottom">
          <span>Official · verified manufacturer · community</span>
          <span>Every release is exact-pinned and content-hashed.</span>
        </div>
      </footer>
    </>
  );
}

function NotFound() {
  usePageTitle("Not found");
  return (
    <div className="narrow-page">
      <StatePanel title="That route does not exist" icon="search">
        Check the URL or return to the package catalogue.
        <div className="state-inline-action">
          <Link className="button button-primary" to="/packages">
            Explore packages
          </Link>
        </div>
      </StatePanel>
    </div>
  );
}

/// The router's defaultErrorComponent (wired in main.tsx): any exception a
/// route throws while rendering lands in this panel instead of unmounting
/// the whole React root to a blank page.
export function RouteErrorPanel({ error, reset }: ErrorComponentProps) {
  return (
    <div className="narrow-page">
      <StatePanel
        tone="error"
        icon="shield"
        title="This page failed to render"
        action={
          <div className="state-action-row">
            <button className="button button-secondary" onClick={reset}>
              Try again
            </button>
            <button className="button button-ghost" onClick={() => window.location.reload()}>
              Reload the page
            </button>
          </div>
        }
      >
        {error instanceof Error ? error.message : String(error)}
      </StatePanel>
    </div>
  );
}

const rootRoute = createRootRoute({ component: Layout, notFoundComponent: NotFound });

// ---------------------------------------------------------------------------

function Home() {
  usePageTitle("Hardware libraries, verified");
  const { q: legacyQuery } = homeRoute.useSearch();
  const navigate = useNavigate();
  const [heroQuery, setHeroQuery] = useState("");
  const recent = useRecent();

  useEffect(() => {
    if (legacyQuery) {
      navigate({
        to: "/packages",
        search: { q: legacyQuery },
        replace: true,
      });
    }
  }, [legacyQuery, navigate]);

  if (legacyQuery) return <LoadingRows />;

  const seenPackages = new Set<string>();
  const recentPackages: SearchRow[] = (recent.data?.results ?? [])
    .filter((pkg) => {
      if (seenPackages.has(pkg.name)) return false;
      seenPackages.add(pkg.name);
      return true;
    })
    .map((pkg) => ({
      name: pkg.name,
      tier: pkg.tier as SearchRow["tier"],
      latest: pkg.version,
      updated: pkg.published_at,
      description: pkg.description,
    }));

  return (
    <>
      <section className="hero">
        <div className="hero-grid" aria-hidden="true" />
        <div className="hero-glow hero-glow-one" aria-hidden="true" />
        <div className="hero-glow hero-glow-two" aria-hidden="true" />
        <div className="hero-content">
          <p className="eyebrow">
            <span className="eyebrow-node" aria-hidden="true" />
            The CoHDL package registry
          </p>
          <h1>
            Hardware building blocks,
            <span> verified down to the byte.</span>
          </h1>
          <p className="hero-lede">
            Discover trusted component libraries, footprints, parts, and documents. Every exact
            release is hashed by the registry and locked into your design.
          </p>
          <form
            className="hero-search"
            role="search"
            aria-label="Homepage package search"
            onSubmit={(event) => {
              event.preventDefault();
              navigate({
                to: "/packages",
                search: { q: heroQuery.trim() || undefined },
              });
            }}
          >
            <Icon name="search" size={21} />
            <input
              aria-label="Search the CoHDL package catalogue"
              placeholder="Try “esp32”, “qfn”, or “power”…"
              value={heroQuery}
              onChange={(event) => setHeroQuery(event.target.value)}
            />
            <button type="submit">Explore packages</button>
          </form>
          <div className="hero-actions">
            <Link className="button button-primary" to="/packages">
              Browse the catalogue <Icon name="arrow" size={17} />
            </Link>
            <Link className="button button-ghost" to="/docs">
              Publishing guide
            </Link>
          </div>
        </div>
        <div className="hero-console" aria-label="Example package install">
          <div className="console-top">
            <span className="console-lights" aria-hidden="true">
              <i />
              <i />
              <i />
            </span>
            <span>design terminal</span>
            <span className="console-state">verified</span>
          </div>
          <div className="console-body">
            <p>
              <span className="prompt">›</span> cohdl add passive@0.2.0
            </p>
            <p className="console-muted">resolving exact package identity…</p>
            <p>
              <span className="console-check">✓</span> content hash verified
            </p>
            <p>
              <span className="console-check">✓</span> lockfile updated
            </p>
            <div className="console-hash">
              <Icon name="hash" size={15} />
              <span>sha256:7d4c…91af</span>
            </div>
          </div>
        </div>
      </section>

      <section className="trust-strip" aria-label="Registry guarantees">
        <article>
          <Icon name="shield" />
          <div>
            <strong>Trust is structural</strong>
            <span>Official, verified manufacturer, and community namespaces stay distinct.</span>
          </div>
        </article>
        <article>
          <Icon name="hash" />
          <div>
            <strong>Server-hashed releases</strong>
            <span>The registry computes the authoritative content identity itself.</span>
          </div>
        </article>
        <article>
          <Icon name="cube" />
          <div>
            <strong>Exact versions only</strong>
            <span>No floating ranges. One version always means one immutable artifact.</span>
          </div>
        </article>
      </section>

      <section className="content-section">
        <div className="section-heading">
          <div>
            <p className="eyebrow">Fresh from the registry</p>
            <h2>Recently published</h2>
          </div>
          <Link className="text-link" to="/packages">
            Browse catalogue
            <Icon name="arrow" size={16} />
          </Link>
        </div>
        {recent.isPending ? (
          <LoadingRows />
        ) : recent.isError ? (
          <StatePanel
            tone="error"
            icon="search"
            title="Could not load recent packages"
            action={
              <button className="button button-secondary" onClick={() => recent.refetch()}>
                Try again
              </button>
            }
          >
            The registry did not return its latest releases.
          </StatePanel>
        ) : recentPackages.length === 0 ? (
          <StatePanel title="The registry is ready for its first package">
            Follow the publishing guide to ship an immutable release.
          </StatePanel>
        ) : (
          <div className="package-stack">
            {recentPackages.slice(0, 6).map((pkg) => (
              <PackageCard key={pkg.name} pkg={pkg} />
            ))}
          </div>
        )}
      </section>
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

function ComponentRequestRoute() {
  usePageTitle("Request a component");
  const { part } = requestRoute.useSearch();
  return <ComponentRequestPage initialPart={part} />;
}

const requestRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/request",
  validateSearch: (search: Record<string, unknown>): { part?: string } =>
    typeof search.part === "string" && search.part ? { part: search.part } : {},
  component: ComponentRequestRoute,
});

// ---------------------------------------------------------------------------

type TierFilter = "all" | SearchRow["tier"];
type CatalogueSort = "updated" | "name";

function Catalogue() {
  const searchState = catalogRoute.useSearch();
  const navigate = useNavigate();
  const query = searchState.q ?? "";
  const tier = searchState.tier ?? "all";
  const sort = searchState.sort ?? "updated";
  const [input, setInput] = useState(query);
  const packages = useSearch(query, tier === "all" ? undefined : tier, sort);
  usePageTitle(query ? `Search: ${query}` : "Packages");

  useEffect(() => setInput(query), [query]);

  const visible = packages.data?.results ?? [];

  const updateSearch = (next: {
    q?: string | null;
    tier?: TierFilter;
    sort?: CatalogueSort;
  }) => {
    const nextTier = next.tier ?? tier;
    const nextSort = next.sort ?? sort;
    return navigate({
      to: "/packages",
      search: {
        q: next.q === null ? undefined : (next.q ?? (input.trim() || undefined)),
        tier: nextTier === "all" ? undefined : nextTier,
        sort: nextSort === "updated" ? undefined : nextSort,
      },
    });
  };

  return (
    <div className="catalogue-page">
      <div className="page-heading page-heading-row catalogue-page-heading">
        <div>
          <p className="eyebrow">Package catalogue</p>
          <h1>{query ? `Search results for “${query}”` : "Explore hardware libraries"}</h1>
          <p>
            Find exact-pinned component libraries across official, manufacturer, and community
            namespaces.
          </p>
        </div>
        <Link className="button button-secondary" to="/request">
          Request a component
        </Link>
      </div>

      <div className="catalogue-toolbar">
        <form
          className="catalogue-search"
          role="search"
          aria-label="Package catalogue search"
          onSubmit={(event) => {
            event.preventDefault();
            updateSearch({ q: input.trim() || null });
          }}
        >
          <Icon name="search" size={18} />
          <input
            aria-label="Search package catalogue"
            value={input}
            onChange={(event) => setInput(event.target.value)}
            placeholder="Search package names and descriptions"
          />
          {query && (
            <button
              type="button"
              className="clear-search"
              onClick={() => {
                setInput("");
                updateSearch({ q: null });
              }}
            >
              Clear
            </button>
          )}
          <button type="submit" className="button button-primary">
            Search
          </button>
        </form>
        <div className="catalogue-controls">
          <div className="filter-pills" aria-label="Filter by trust tier">
            {(
              [
                ["all", "All"],
                ["official", "Official"],
                ["brand", "Manufacturers"],
                ["contrib", "Community"],
              ] as const
            ).map(([value, label]) => (
              <button
                type="button"
                key={value}
                aria-pressed={tier === value}
                onClick={() => updateSearch({ tier: value })}
              >
                {label}
              </button>
            ))}
          </div>
          <label className="sort-control">
            <span>Sort</span>
            <select
              value={sort}
              onChange={(event) => updateSearch({ sort: event.target.value as CatalogueSort })}
            >
              <option value="updated">Recently updated</option>
              <option value="name">Name</option>
            </select>
          </label>
        </div>
      </div>

      {packages.isPending ? (
        <LoadingRows count={5} />
      ) : packages.isError ? (
        <StatePanel
          tone="error"
          title="Package search is unavailable"
          icon="search"
          action={
            <button className="button button-secondary" onClick={() => packages.refetch()}>
              Try again
            </button>
          }
        >
          The registry could not complete this search.
        </StatePanel>
      ) : visible.length === 0 ? (
        <StatePanel
          title={query ? `No packages match “${query}”` : "No packages in this trust tier"}
          icon="search"
          action={
            <div className="state-action-row">
              {query && tier === "all" && (
                <Link
                  className="button button-primary"
                  to="/request"
                  search={{ part: query }}
                >
                  Request this component
                </Link>
              )}
              <button
                className="button button-secondary"
                onClick={() => {
                  setInput("");
                  navigate({ to: "/packages", search: {} });
                }}
              >
                Clear filters
              </button>
            </div>
          }
        >
          Try another name, description, or namespace.
        </StatePanel>
      ) : (
        <>
          <div className="results-summary" aria-live="polite">
            Showing {visible.length}
            {(packages.data?.total ?? visible.length) > visible.length
              ? ` of ${packages.data?.total}`
              : ""}{" "}
            {(packages.data?.total ?? visible.length) === 1 ? "package" : "packages"}
            {tier !== "all" ? ` in ${tier === "brand" ? "verified manufacturers" : tier}` : ""}
          </div>
          <div className="package-stack">
            {visible.map((pkg) => (
              <PackageCard key={pkg.name} pkg={pkg} />
            ))}
          </div>
          {packages.data?.truncated && (
            <p className="results-note">
              Refine the search to see beyond the first 50 matching packages.
            </p>
          )}
        </>
      )}
    </div>
  );
}

const catalogRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/packages",
  validateSearch: (
    search: Record<string, unknown>,
  ): { q?: string; tier?: SearchRow["tier"]; sort?: CatalogueSort } => {
    const q = typeof search.q === "string" && search.q.trim() ? search.q.trim() : undefined;
    const tier =
      search.tier === "official" || search.tier === "brand" || search.tier === "contrib"
        ? search.tier
        : undefined;
    const sort = search.sort === "name" ? "name" : undefined;
    return { q, tier, sort };
  },
  component: Catalogue,
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
  const active = selected && docs.includes(selected) ? selected : pickFirstDoc(docs);
  const text = useDocText(name, version.version, active && isTextDoc(active) ? active : null);

  useEffect(() => setSelected(null), [version.version]);

  if (docs.length === 0) {
    return (
      <StatePanel title="No package documents in this release" icon="document">
        The package metadata and immutable artifact are still available in the release details.
      </StatePanel>
    );
  }

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
    <div className={`document-layout${docs.length === 1 ? " document-layout-single" : ""}`}>
      {docs.length > 1 && (
        <nav className="doc-navigator" aria-label={`Documents in ${name} ${version.version}`}>
          <div className="doc-nav-heading">
            <span>Included documents</span>
            <span>{docs.length}</span>
          </div>
          {docs.map((d) => (
            <button
              type="button"
              key={d}
              className={d === active ? "doc-link doc-link-active" : "doc-link"}
              onClick={() => setSelected(d)}
              aria-pressed={d === active}
            >
              <Icon name="document" size={16} />
              <span>{d}</span>
            </button>
          ))}
        </nav>
      )}
      <article className="published-document" onClick={onLinkClick}>
        {active && !isTextDoc(active) ? (
          <div className="binary-document">
            <span className="binary-document-icon">
              <Icon name="document" size={28} />
            </span>
            <div>
              <p className="eyebrow">Published document</p>
              <h3>{active}</h3>
              <p>This file is served directly from the immutable {version.version} artifact.</p>
              <a
                className="button button-primary"
                href={docUrl(name, version.version, active)}
                rel="noopener noreferrer"
                target="_blank"
              >
                Open document <Icon name="external" size={15} />
              </a>
            </div>
          </div>
        ) : text.isPending ? (
          <div className="document-loading" role="status">
            <span className="skeleton skeleton-title" />
            <span className="skeleton skeleton-copy" />
            <span className="skeleton skeleton-copy" />
            <span className="sr-only">Loading {active}…</span>
          </div>
        ) : text.isError ? (
          <StatePanel
            tone="error"
            title={`Could not load ${active}`}
            action={
              <button className="button button-secondary" onClick={() => text.refetch()}>
                Try again
              </button>
            }
          >
            {(text.error as Error).message}
          </StatePanel>
        ) : (
          <Markdown source={text.data ?? ""} resolve={resolve} />
        )}
      </article>
    </div>
  );
}

function packageTarUrl(name: string, version: string): string {
  const encodedName = name.split("/").map(encodeURIComponent).join("/");
  return `/packages/${encodedName}/${encodeURIComponent(version)}.tar`;
}

function safeRepositoryUrl(value: string | null): URL | null {
  if (!value) return null;
  try {
    const url = new URL(value);
    if (
      (url.protocol !== "https:" && url.protocol !== "http:") ||
      url.username ||
      url.password
    ) {
      return null;
    }
    return url;
  } catch {
    return null;
  }
}

function PackagePage() {
  const { _splat } = packageRoute.useParams();
  const search = packageRoute.useSearch();
  const name = _splat ?? "";
  const pkg = usePackage(name);
  const navigate = useNavigate();
  const [tab, setTab] = useState<"overview" | "versions">("overview");
  const overviewTab = useRef<HTMLButtonElement>(null);
  const versionsTab = useRef<HTMLButtonElement>(null);
  const apiTab = useRef<HTMLButtonElement>(null);
  usePageTitle(name || "Package");

  const latest = pkg.data?.versions[0];
  const requested = search.version;
  const version =
    pkg.data?.versions.find((candidate) => candidate.version === requested) ?? latest;
  // A version row that says `api_docs: false` needs no probe; anything else
  // (true, or a row predating the flag) asks the endpoint — a 404 resolves
  // to `null`, the normal "no docs" state.
  const apiDocs = useApiDocs(
    name,
    version && version.api_docs !== false ? version.version : undefined,
  );

  useEffect(() => {
    if (
      search.version &&
      pkg.data &&
      !pkg.data.versions.some((candidate) => candidate.version === search.version)
    ) {
      navigate({
        to: "/package/$",
        params: { _splat: name },
        search: {},
        replace: true,
      });
    }
  }, [name, navigate, pkg.data, search.version]);

  if (pkg.isPending) {
    return (
      <div className="package-page">
        <div className="package-heading-skeleton">
          <span className="skeleton skeleton-icon" />
          <span>
            <span className="skeleton skeleton-title" />
            <span className="skeleton skeleton-copy" />
          </span>
        </div>
        <LoadingRows count={2} />
      </div>
    );
  }
  if (pkg.isError || !pkg.data) {
    const missing = (pkg.error as Error | null)?.message === "not found";
    return (
      <div className="narrow-page">
        <StatePanel
          tone={missing ? "neutral" : "error"}
          icon={missing ? "cube" : "search"}
          title={missing ? `“${name}” is not published here` : "Could not load this package"}
          action={
            missing ? (
              <Link className="button button-secondary" to="/packages">
                Search the catalogue
              </Link>
            ) : (
              <button className="button button-secondary" onClick={() => pkg.refetch()}>
                Try again
              </button>
            )
          }
        >
          {missing
            ? "Check the package name or explore the registry."
            : "The registry returned an error while loading package metadata."}
        </StatePanel>
      </div>
    );
  }

  if (!version) {
    return (
      <StatePanel title="This package has no published versions">
        Package metadata exists, but there is no release artifact to inspect.
      </StatePanel>
    );
  }
  const repo = version.repository;
  // Only credential-free http(s) URLs become anchors. Showing the parsed
  // hostname keeps a deceptive user-info prefix from masquerading as a
  // trusted repository host.
  const repoUrl = safeRepositoryUrl(repo);
  const install = `cohdl add ${name}@${version.version}`;
  const docsCount = version.docs.length;
  const isLatest = version.version === latest?.version;
  // The API tab is URL-driven (`view=api`, or an `item` deep link); the
  // other two keep their local tab state untouched.
  const apiSelected = search.view === "api" || !!search.item;
  const activeTab: "overview" | "versions" | "api" = apiSelected ? "api" : tab;
  const showApiTab = version.api_docs === true || !!apiDocs.data || apiSelected;
  const versionSearch = { version: isLatest ? undefined : version.version };
  const selectTab = (next: "overview" | "versions" | "api") => {
    if (next === "api") {
      if (!apiSelected) {
        navigate({
          to: "/package/$",
          params: { _splat: name },
          search: { ...versionSearch, view: "api" },
        });
      }
      return;
    }
    setTab(next);
    if (apiSelected) {
      navigate({ to: "/package/$", params: { _splat: name }, search: versionSearch });
    }
  };
  const trustCopy =
    pkg.data.tier === "official"
      ? "Published from CoHDL’s official namespace."
      : pkg.data.tier === "brand"
        ? "The manufacturer namespace is verified by the registry."
        : "Published in the open community namespace.";

  return (
    <div className="package-page">
      <nav className="breadcrumbs" aria-label="Breadcrumb">
        <Link to="/packages">Packages</Link>
        <span aria-hidden="true">/</span>
        <span>{name}</span>
      </nav>

      <header className="package-identity">
        <span className="package-identity-glyph">
          <Icon name="cube" size={29} />
        </span>
        <div>
          <div className="package-kicker">
            <TierBadge tier={pkg.data.tier} />
            <span>v{version.version}</span>
            {!isLatest && <span className="version-note">historical release</span>}
          </div>
          <h1>{name}</h1>
          <p>{version.description ?? "No package description has been published for this release."}</p>
        </div>
      </header>

      {requested && !pkg.data.versions.some((candidate) => candidate.version === requested) && (
        <StatePanel title={`Version ${requested} was not found`}>
          Showing the latest release instead.
        </StatePanel>
      )}

      <div
        className="package-tabs"
        role="tablist"
        aria-label="Package sections"
        onKeyDown={(event) => {
          if (!["ArrowLeft", "ArrowRight", "Home", "End"].includes(event.key)) return;
          event.preventDefault();
          const order: ("overview" | "versions" | "api")[] = showApiTab
            ? ["overview", "versions", "api"]
            : ["overview", "versions"];
          const current = order.indexOf(activeTab);
          const nextIndex =
            event.key === "Home"
              ? 0
              : event.key === "End"
                ? order.length - 1
                : event.key === "ArrowRight"
                  ? Math.min(current + 1, order.length - 1)
                  : Math.max(current - 1, 0);
          const next = order[nextIndex];
          selectTab(next);
          (next === "overview" ? overviewTab : next === "versions" ? versionsTab : apiTab)
            .current?.focus();
        }}
      >
        <button
          ref={overviewTab}
          id="package-overview-tab"
          type="button"
          role="tab"
          aria-selected={activeTab === "overview"}
          aria-controls="package-overview-panel"
          tabIndex={activeTab === "overview" ? 0 : -1}
          onClick={() => selectTab("overview")}
        >
          Overview
          <span>{docsCount > 0 ? `${docsCount} ${docsCount === 1 ? "doc" : "docs"}` : "metadata"}</span>
        </button>
        <button
          ref={versionsTab}
          id="package-versions-tab"
          type="button"
          role="tab"
          aria-selected={activeTab === "versions"}
          aria-controls="package-versions-panel"
          tabIndex={activeTab === "versions" ? 0 : -1}
          onClick={() => selectTab("versions")}
        >
          Versions <span>{pkg.data.versions.length}</span>
        </button>
        {showApiTab && (
          <button
            ref={apiTab}
            id="package-api-tab"
            type="button"
            role="tab"
            aria-selected={activeTab === "api"}
            aria-controls="package-api-panel"
            tabIndex={activeTab === "api" ? 0 : -1}
            onClick={() => selectTab("api")}
          >
            API
            <span>
              {apiDocs.data
                ? `${(apiDocs.data.items ?? []).length} ${
                    (apiDocs.data.items ?? []).length === 1 ? "item" : "items"
                  }`
                : "explorer"}
            </span>
          </button>
        )}
      </div>

      <div className="package-layout">
        <div className="package-primary">
          {activeTab === "overview" ? (
            <section
              id="package-overview-panel"
              className="content-panel"
              role="tabpanel"
              aria-labelledby="package-overview-tab"
              tabIndex={0}
            >
              <div className="panel-heading">
                <div>
                  <p className="eyebrow">Published with {version.version}</p>
                  <h2>Documentation</h2>
                </div>
                <span className="panel-count">
                  {docsCount} {docsCount === 1 ? "file" : "files"}
                </span>
              </div>
              <Documents name={name} version={version} />
            </section>
          ) : activeTab === "api" ? (
            <section
              id="package-api-panel"
              className="content-panel"
              role="tabpanel"
              aria-labelledby="package-api-tab"
              tabIndex={0}
            >
              <div className="panel-heading">
                <div>
                  <p className="eyebrow">Extracted from {version.version}</p>
                  <h2>API documentation</h2>
                </div>
                {apiDocs.data && (
                  <span className="panel-count">{(apiDocs.data.items ?? []).length} items</span>
                )}
              </div>
              <ApiExplorer
                name={name}
                version={version.version}
                versionParam={isLatest ? undefined : version.version}
                query={apiDocs}
                search={{ item: search.item, q: search.q, kind: search.kind }}
              />
            </section>
          ) : (
            <section
              id="package-versions-panel"
              className="content-panel"
              role="tabpanel"
              aria-labelledby="package-versions-tab"
              tabIndex={0}
            >
              <div className="panel-heading">
                <div>
                  <p className="eyebrow">Immutable release history</p>
                  <h2>Versions</h2>
                </div>
                <span className="panel-count">{pkg.data.versions.length} total</span>
              </div>
              <div className="table-wrap">
                <table className="versions-table">
                  <caption className="sr-only">Published versions of {name}</caption>
                  <thead>
                    <tr>
                      <th scope="col">Version</th>
                      <th scope="col">Published</th>
                      <th scope="col">Size</th>
                      <th scope="col">Content hash</th>
                    </tr>
                  </thead>
                  <tbody>
                    {pkg.data.versions.map((candidate) => (
                      <tr key={candidate.version}>
                        <td data-label="Version">
                          <Link
                            to="/package/$"
                            params={{ _splat: name }}
                            search={{
                              version:
                                candidate.version === latest?.version
                                  ? undefined
                                  : candidate.version,
                            }}
                            onClick={() => setTab("overview")}
                          >
                            {candidate.version}
                          </Link>
                          {candidate.version === latest?.version && (
                            <span className="latest-badge">latest</span>
                          )}
                        </td>
                        <td data-label="Published">
                          <time dateTime={candidate.published_at}>
                            {formatDate(candidate.published_at)}
                          </time>
                        </td>
                        <td data-label="Size">{formatSize(candidate.size)}</td>
                        <td data-label="Content hash">
                          <code className="hash-short">{candidate.hash.slice(0, 20)}…</code>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            </section>
          )}
        </div>

        <aside className="package-sidebar" aria-label="Package details">
          <section className="sidebar-panel install-panel">
            <p className="sidebar-label">Install exact release</p>
            <CommandBox command={install} compact />
            <p className="sidebar-hint">
              This exact identity is recorded in <code>cohdl.lock</code>.
            </p>
          </section>

          <section className="sidebar-panel trust-panel">
            <div className="trust-panel-heading">
              <span className="trust-icon">
                <Icon name="shield" size={20} />
              </span>
              <div>
                <strong>Registry trust</strong>
                <TierBadge tier={pkg.data.tier} />
              </div>
            </div>
            <p>{trustCopy}</p>
          </section>

          <section className="sidebar-panel">
            <div className="sidebar-panel-heading">
              <strong>Release details</strong>
              <span>{isLatest ? "Latest" : "Selected"}</span>
            </div>
            <label className="version-select">
              <span>Version</span>
              <select
                value={version.version}
                onChange={(event) => {
                  setTab("overview");
                  navigate({
                    to: "/package/$",
                    params: { _splat: name },
                    search: {
                      version:
                        event.target.value === latest?.version ? undefined : event.target.value,
                    },
                  });
                }}
              >
                {pkg.data.versions.map((candidate) => (
                  <option value={candidate.version} key={candidate.version}>
                    {candidate.version}
                    {candidate.version === latest?.version ? " — latest" : ""}
                  </option>
                ))}
              </select>
            </label>
            <dl className="metadata-list">
              <div>
                <dt>Published</dt>
                <dd>
                  <time dateTime={version.published_at}>{formatDate(version.published_at)}</time>
                </dd>
              </div>
              <div>
                <dt>Artifact</dt>
                <dd>{formatSize(version.size)}</dd>
              </div>
              <div>
                <dt>License</dt>
                <dd>{version.license ?? "Not declared"}</dd>
              </div>
              <div>
                <dt>Documents</dt>
                <dd>{docsCount}</dd>
              </div>
            </dl>
            <div className="sidebar-links">
              <a
                href={packageTarUrl(name, version.version)}
                className="sidebar-link"
                download
              >
                Download artifact <Icon name="arrow" size={15} />
              </a>
              {repo &&
                (repoUrl ? (
                  <a
                    href={repoUrl.toString()}
                    className="sidebar-link"
                    rel="noopener noreferrer nofollow"
                    target="_blank"
                    title={repoUrl.toString()}
                  >
                    Source · {repoUrl.host} <Icon name="external" size={14} />
                  </a>
                ) : (
                  <span className="sidebar-repository">{repo}</span>
                ))}
            </div>
          </section>

          <section className="sidebar-panel integrity-panel">
            <div className="integrity-heading">
              <span>
                <Icon name="hash" size={17} /> Content integrity
              </span>
              <CopyButton value={version.hash} compact label="Copy hash" />
            </div>
            <code>{version.hash}</code>
            <p>Computed by the registry from the published artifact.</p>
          </section>
        </aside>
      </div>
    </div>
  );
}

const packageRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/package/$",
  // API-explorer state is shareable: `view=api` (or a present `item`)
  // selects the API tab, `item` deep-links one declaration, `q`/`kind`
  // filter. Defaults are canonically omitted.
  validateSearch: (
    search: Record<string, unknown>,
  ): { version?: string; view?: "api"; item?: string; q?: string; kind?: string } => ({
    version:
      typeof search.version === "string" && search.version ? search.version : undefined,
    view: search.view === "api" ? "api" : undefined,
    item: typeof search.item === "string" && search.item ? search.item : undefined,
    q: typeof search.q === "string" && search.q ? search.q : undefined,
    kind: typeof search.kind === "string" && search.kind ? search.kind : undefined,
  }),
  component: PackagePage,
});

// ---------------------------------------------------------------------------

function Account() {
  const me = useMe();
  const config = useConfig();
  const qc = useQueryClient();
  const navigate = useNavigate();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [mode, setMode] = useState<"signin" | "signup">("signin");
  const [token, setToken] = useState<string | null>(null);
  usePageTitle(me.data ? "Account" : mode === "signup" ? "Create account" : "Sign in");

  const auth = useMutation({
    mutationFn: async () => {
      const action = mode === "signup" ? "signup" : "login";
      const recaptcha = await recaptchaToken(config.data?.recaptcha_site_key, action);
      return post<Me>(mode === "signup" ? "/api/signup" : "/api/session", {
        email,
        password,
        recaptcha,
      });
    },
    onSuccess: (data) => {
      qc.setQueryData(["me"], data);
      setPassword("");
    },
  });
  const mintToken = useMutation({
    mutationFn: () => post<{ token: string }>("/api/tokens", {}),
    onMutate: () => setToken(null),
    onSuccess: (d) => setToken(d.token),
  });
  const logout = useMutation({
    mutationFn: () => del<{ signed_out: boolean }>("/api/session", {}),
    onSuccess: async () => {
      setToken(null);
      qc.removeQueries({ queryKey: ["me"] });
      await navigate({ to: "/" });
    },
  });

  if (me.isPending) {
    return (
      <div className="account-page">
        <div className="page-heading">
          <p className="eyebrow">Registry identity</p>
          <h1>Account</h1>
        </div>
        <LoadingRows count={2} />
      </div>
    );
  }

  if (me.isError && !isUnauthorized(me.error)) {
    return (
      <div className="narrow-page">
        <StatePanel
          tone="error"
          icon="shield"
          title="Could not check your registry session"
          action={
            <button className="button button-secondary" onClick={() => me.refetch()}>
              Try again
            </button>
          }
        >
          Your account state is unchanged. The registry could not reach the session service.
        </StatePanel>
      </div>
    );
  }

  if (me.data) {
    const identityTier = me.data.official ? "official" : me.data.brands.length > 0 ? "brand" : "contrib";
    return (
      <div className="account-page">
        <div className="page-heading page-heading-row">
          <div>
            <p className="eyebrow">Registry identity</p>
            <h1>Account</h1>
            <p>Manage publishing access and command-line credentials.</p>
          </div>
          <button
            className="button button-ghost"
            type="button"
            disabled={logout.isPending}
            onClick={() => logout.mutate()}
          >
            {logout.isPending ? "Signing out…" : "Sign out"}
          </button>
        </div>

        <section className="identity-card">
          <span className="identity-avatar" aria-hidden="true">
            {me.data.account.slice(0, 1).toUpperCase()}
          </span>
          <div className="identity-main">
            <span className="identity-label">Signed in as</span>
            <h2>{me.data.account}</h2>
            <TierBadge tier={identityTier} />
          </div>
          <div className="identity-proof">
            <Icon name="shield" size={21} />
            <span>
              <strong>Session protected</strong>
              <small>Web credentials never become CLI tokens.</small>
            </span>
          </div>
        </section>

        {logout.isError && (
          <StatePanel tone="error" title="Could not sign out">
            {(logout.error as Error).message}
          </StatePanel>
        )}

        <div className="account-grid">
          <section className="content-panel token-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Command-line access</p>
                <h2>CLI tokens</h2>
              </div>
              <Icon name="terminal" size={23} />
            </div>
            <p>
              Create a separate credential for <code>cohdl publish</code>. The raw token is shown
              once; the registry stores only its hash.
            </p>
            <div className="token-actions">
              <button
                className="button button-primary"
                type="button"
                onClick={() => mintToken.mutate()}
                disabled={mintToken.isPending}
              >
                {mintToken.isPending ? "Creating token…" : "Create CLI token"}
              </button>
            </div>
            {mintToken.isError && (
              <StatePanel tone="error" title="Could not create a token">
                {(mintToken.error as Error).message}
              </StatePanel>
            )}
            {token && (
              <div className="token-reveal" role="status">
                <div className="token-warning">
                  <Icon name="shield" size={18} />
                  <span>Copy this token now. It cannot be shown again.</span>
                </div>
                <CommandBox command={token} label="New CLI token" />
                <CommandBox command="cohdl login" label="Then authenticate the CLI" compact />
              </div>
            )}
          </section>

          <section className="content-panel scope-panel">
            <div className="panel-heading">
              <div>
                <p className="eyebrow">Publish authority</p>
                <h2>Namespaces</h2>
              </div>
              <Icon name="shield" size={23} />
            </div>
            {me.data.official && (
              <div className="scope-row">
                <TierBadge tier="official" />
                <div>
                  <strong>Bare package names</strong>
                  <p>May publish official libraries such as <code>std</code>.</p>
                </div>
              </div>
            )}
            {me.data.brands.map((brand) => (
              <div className="scope-row" key={brand}>
                <TierBadge tier="brand" />
                <div>
                  <strong>@{brand}/*</strong>
                  <p>Verified manufacturer namespace.</p>
                </div>
              </div>
            ))}
            <div className="scope-row">
              <TierBadge tier="contrib" />
              <div>
                <strong>@contrib/*</strong>
                <p>Available to every authenticated publisher.</p>
              </div>
            </div>
            {!me.data.official && me.data.brands.length === 0 && (
              <p className="panel-footnote">
                Manufacturer verification is human-gated. See the{" "}
                <Link to="/docs">publishing guide</Link> for the trust model.
              </p>
            )}
          </section>
        </div>
      </div>
    );
  }

  return (
    <div className="auth-page">
      <section className="auth-intro">
        <p className="eyebrow">Publisher access</p>
        <h1>{mode === "signup" ? "Start publishing hardware libraries." : "Welcome back."}</h1>
        <p>
          One registry identity connects the web dashboard and separately issued CLI credentials.
        </p>
        <ul className="auth-benefits">
          <li>
            <Icon name="hash" size={19} />
            Server-computed content identities
          </li>
          <li>
            <Icon name="shield" size={19} />
            Structural namespace trust
          </li>
          <li>
            <Icon name="cube" size={19} />
            Immutable exact releases
          </li>
        </ul>
      </section>

      <section className="auth-card">
        <div className="auth-card-heading">
          <BrandMark size={38} />
          <div>
            <p className="eyebrow">{mode === "signup" ? "Create account" : "Registry sign in"}</p>
            <h2>{mode === "signup" ? "Create your identity" : "Access your account"}</h2>
          </div>
        </div>
        <form
          className="auth-form"
          onSubmit={(event) => {
            event.preventDefault();
            auth.mutate();
          }}
        >
          <label htmlFor="account-email">Email address</label>
          <input
            id="account-email"
            name="email"
            type="email"
            autoComplete="email"
            placeholder="you@company.com"
            required
            value={email}
            onChange={(event) => setEmail(event.target.value)}
          />
          <label htmlFor="account-password">Password</label>
          <input
            id="account-password"
            name="password"
            type="password"
            autoComplete={mode === "signup" ? "new-password" : "current-password"}
            minLength={mode === "signup" ? 8 : undefined}
            placeholder={mode === "signup" ? "At least 8 characters" : "Your password"}
            required
            value={password}
            onChange={(event) => setPassword(event.target.value)}
          />
          {mode === "signup" && (
            <p className="field-help">Use at least 8 characters. Your password is never a CLI token.</p>
          )}
          <button
            className="button button-primary auth-submit"
            type="submit"
            disabled={auth.isPending || config.isPending || config.isError}
          >
            {auth.isPending
              ? mode === "signup"
                ? "Creating account…"
                : "Signing in…"
              : mode === "signup"
                ? "Create account"
                : "Sign in"}
          </button>
          {auth.isError && (
            <StatePanel tone="error" title={mode === "signup" ? "Could not create account" : "Could not sign in"}>
              {(auth.error as Error).message}
            </StatePanel>
          )}
          {config.isError && (
            <StatePanel tone="error" title="Sign-in configuration is unavailable">
              Reload the page and try again.
            </StatePanel>
          )}
        </form>
        <div className="auth-switch">
          <span>{mode === "signup" ? "Already have an account?" : "New to CoHDL Registry?"}</span>
          <button
            type="button"
            className="text-button"
            onClick={() => {
              auth.reset();
              setMode(mode === "signup" ? "signin" : "signup");
            }}
          >
            {mode === "signup" ? "Sign in" : "Create an account"}
          </button>
        </div>
      </section>
    </div>
  );
}

const accountRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/account",
  component: Account,
});

// ---------------------------------------------------------------------------

interface RevokeTarget {
  account: AdminAccount;
  scope: string;
}

function RevokeDialog({
  target,
  busy,
  error,
  onCancel,
  onConfirm,
}: {
  target: RevokeTarget | null;
  busy: boolean;
  error?: string;
  onCancel(): void;
  onConfirm(target: RevokeTarget): void;
}) {
  const dialog = useRef<HTMLDialogElement>(null);

  useEffect(() => {
    const element = dialog.current;
    if (!element) return;
    if (target && !element.open) element.showModal();
    if (!target && element.open) element.close();
  }, [target]);

  return (
    <dialog
      ref={dialog}
      className="confirm-dialog"
      aria-labelledby="revoke-dialog-title"
      aria-describedby="revoke-dialog-description"
      onCancel={(event) => {
        event.preventDefault();
        if (!busy) onCancel();
      }}
      onClose={() => {
        if (target && !busy) onCancel();
      }}
    >
      {target && (
        <div className="dialog-content">
          <span className="dialog-icon">
            <Icon name="shield" size={24} />
          </span>
          <p className="eyebrow">Confirm privilege change</p>
          <h2 id="revoke-dialog-title">Revoke @{target.scope}/*?</h2>
          <p id="revoke-dialog-description">
            This immediately removes publish access from <strong>{target.account.email}</strong>.
            The unverified claim remains reserved to this account.
          </p>
          {error && (
            <p className="dialog-error" role="alert">
              {error}
            </p>
          )}
          <div className="dialog-actions">
            <button
              type="button"
              className="button button-secondary"
              disabled={busy}
              autoFocus
              onClick={onCancel}
            >
              Keep verification
            </button>
            <button
              type="button"
              className="button button-danger"
              disabled={busy}
              onClick={() => onConfirm(target)}
            >
              {busy ? "Revoking…" : "Revoke verification"}
            </button>
          </div>
        </div>
      )}
    </dialog>
  );
}

function AdminDashboard({ accountEmail }: { accountEmail: string }) {
  const qc = useQueryClient();
  const [searchInput, setSearchInput] = useState("");
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<number | null>(null);
  const [brand, setBrand] = useState("");
  const [status, setStatus] = useState<string | null>(null);
  const [revokeTarget, setRevokeTarget] = useState<RevokeTarget | null>(null);
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
      setRevokeTarget(null);
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

  const chooseAccount = (account: AdminAccount, focus = false) => {
    clearFeedback();
    setBrand("");
    setRevokeTarget(null);
    setSelectedId(account.id);
    if (focus) {
      requestAnimationFrame(() => {
        document.getElementById(`admin-account-${account.id}`)?.focus();
      });
    }
  };

  return (
    <div className="admin-page">
      <div className="page-heading page-heading-row">
        <div>
          <p className="eyebrow">Official operations</p>
          <h1>Registry administration</h1>
          <p>Review publisher accounts and manage manufacturer namespace verification.</p>
        </div>
        <span className="operator-badge">
          <Icon name="shield" size={17} />
          Official session
        </span>
      </div>

      <div className="admin-local-nav" aria-label="Registry administration sections">
        <Link className="is-active" to="/admin" aria-current="page">
          Publishers
        </Link>
        <Link to="/admin/requests">Component requests</Link>
      </div>

      <div className="admin-workspace">
        <section className="content-panel account-browser" aria-labelledby="account-heading">
          <div className="panel-heading">
            <div>
              <p className="eyebrow">Publishers</p>
              <h2 id="account-heading">Accounts</h2>
            </div>
            <span className="panel-count">{accounts.length}</span>
          </div>
          <form
            className="admin-search"
            role="search"
            aria-label="Publisher account search"
            onSubmit={(event) => {
              event.preventDefault();
              clearFeedback();
              setBrand("");
              setSelectedId(null);
              setSearch(searchInput.trim());
            }}
          >
            <div className="admin-search-box">
              <Icon name="search" size={17} />
              <input
                id="admin-account-search"
                aria-label="Find an account by email"
                type="search"
                placeholder="Search email…"
                value={searchInput}
                onChange={(event) => setSearchInput(event.target.value)}
                disabled={busy}
              />
              <button type="submit" disabled={busy || accountsQuery.isFetching}>
                {accountsQuery.isFetching ? "…" : "Go"}
              </button>
            </div>
            {search && (
              <button
                type="button"
                className="text-button admin-clear"
                disabled={busy}
                onClick={() => {
                  clearFeedback();
                  setBrand("");
                  setSearchInput("");
                  setSearch("");
                  setSelectedId(null);
                }}
              >
                Clear “{search}”
              </button>
            )}
          </form>

          {accountsQuery.isPending ? (
            <div className="account-list-loading" aria-label="Loading accounts">
              <span className="skeleton skeleton-copy" />
              <span className="skeleton skeleton-copy" />
              <span className="skeleton skeleton-copy" />
            </div>
          ) : accountsQuery.isError ? (
            <StatePanel
              tone="error"
              title="Could not load accounts"
              action={
                <button className="button button-secondary" onClick={() => accountsQuery.refetch()}>
                  Try again
                </button>
              }
            >
              {(accountsQuery.error as Error).message}
            </StatePanel>
          ) : accounts.length === 0 ? (
            <StatePanel icon="search" title="No accounts match this search" />
          ) : (
            <>
              {accountsQuery.data?.truncated && (
                <p className="field-help">
                  Showing the first 100 accounts. Refine the email search for another publisher.
                </p>
              )}
              <div
                className="account-list"
                role="listbox"
                aria-label="Registry accounts"
                aria-orientation="vertical"
              >
                {accounts.map((account, index) => (
                  <button
                    id={`admin-account-${account.id}`}
                    type="button"
                    role="option"
                    key={account.id}
                    className={
                      account.id === selected?.id ? "account-row account-row-active" : "account-row"
                    }
                    aria-selected={account.id === selected?.id}
                    tabIndex={account.id === selected?.id ? 0 : -1}
                    disabled={busy}
                    onClick={() => chooseAccount(account)}
                    onKeyDown={(event) => {
                      if (!["ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
                      event.preventDefault();
                      const nextIndex =
                        event.key === "Home"
                          ? 0
                          : event.key === "End"
                            ? accounts.length - 1
                            : event.key === "ArrowDown"
                              ? (index + 1) % accounts.length
                              : (index - 1 + accounts.length) % accounts.length;
                      chooseAccount(accounts[nextIndex], true);
                    }}
                  >
                    <span className="account-row-avatar" aria-hidden="true">
                      {account.email.slice(0, 1).toUpperCase()}
                    </span>
                    <span className="account-row-main">
                      <strong>{account.email}</strong>
                      <small>
                        {account.brands.length} {account.brands.length === 1 ? "claim" : "claims"}
                      </small>
                    </span>
                    {account.official && <TierBadge tier="official" />}
                  </button>
                ))}
              </div>
            </>
          )}
        </section>

        {selected ? (
          <section
            className="content-panel claims-panel"
            aria-labelledby="brands-heading"
            aria-busy={busy}
          >
            <div className="selected-account">
              <span className="identity-avatar" aria-hidden="true">
                {selected.email.slice(0, 1).toUpperCase()}
              </span>
              <div>
                <p className="eyebrow">Managing publisher</p>
                <h2 id="brands-heading">{selected.email}</h2>
                <p>
                  Joined {formatDate(selected.created_at)}
                  {selected.official && (
                    <>
                      {" "}
                      · <TierBadge tier="official" />
                    </>
                  )}
                </p>
              </div>
            </div>

            <form
              className="brand-grant"
              onSubmit={(event) => {
                event.preventDefault();
                const scope = brand.trim();
                if (!scope || !selected) return;
                clearFeedback();
                grant.mutate({ account: selected, scope });
              }}
            >
              <label htmlFor="admin-brand">Grant manufacturer namespace</label>
              <div className="scope-input">
                <span aria-hidden="true">@</span>
                <input
                  id="admin-brand"
                  value={brand}
                  onChange={(event) => setBrand(event.target.value)}
                  pattern="[A-Za-z0-9_][A-Za-z0-9_-]*"
                  title="Use letters, digits, underscores, and hyphens; the first character cannot be a hyphen."
                  placeholder="manufacturer"
                  autoCapitalize="none"
                  autoComplete="off"
                  spellCheck={false}
                  required
                  disabled={busy}
                  aria-describedby="brand-scope-help brand-scope-warning"
                />
                <button
                  className="button button-primary"
                  type="submit"
                  disabled={busy || !brand.trim()}
                >
                  {grant.isPending ? "Granting…" : "Grant verification"}
                </button>
              </div>
              <p id="brand-scope-help" className="field-help">
                Exact and case-sensitive, for example <code>espressif</code>.
              </p>
              <div id="brand-scope-warning" className="admin-warning">
                <Icon name="shield" size={18} />
                <p>
                  <strong>Verify authority out of band.</strong> Account email addresses are
                  self-declared and are not proof of manufacturer ownership.{" "}
                  <strong>Namespace-wide access.</strong> This authorizes every package under{" "}
                  <code>@{brand.trim() || "brand"}/*</code>, not just one library.
                </p>
              </div>
            </form>

            <div className="claim-group">
              <div className="claim-heading">
                <h3>Verified namespaces</h3>
                <span>{verified.length}</span>
              </div>
              {verified.length === 0 ? (
                <p className="empty-line">No verified manufacturer namespaces.</p>
              ) : (
                <ul className="brand-claims">
                  {verified.map((claim) => (
                    <li key={claim.brand}>
                      <span className="claim-identity">
                        <span className="claim-status claim-status-active" aria-hidden="true" />
                        <span>
                          <code>@{claim.brand}/*</code>
                          <small>Publishing enabled</small>
                        </span>
                      </span>
                      <button
                        type="button"
                        className="button button-danger button-small"
                        disabled={busy}
                        onClick={() => {
                          clearFeedback();
                          setRevokeTarget({ account: selected, scope: claim.brand });
                        }}
                      >
                        Revoke
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            <div className="claim-group">
              <div className="claim-heading">
                <h3>Reserved, inactive claims</h3>
                <span>{unverified.length}</span>
              </div>
              {unverified.length === 0 ? (
                <p className="empty-line">No inactive claims.</p>
              ) : (
                <ul className="brand-claims brand-claims-inactive">
                  {unverified.map((claim) => (
                    <li key={claim.brand}>
                      <span className="claim-identity">
                        <span className="claim-status" aria-hidden="true" />
                        <span>
                          <code>@{claim.brand}/*</code>
                          <small>Publishing disabled; ownership preserved</small>
                        </span>
                      </span>
                      <button
                        type="button"
                        className="button button-secondary button-small"
                        disabled={busy}
                        onClick={() => {
                          clearFeedback();
                          grant.mutate({ account: selected, scope: claim.brand });
                        }}
                      >
                        {grant.isPending && grant.variables?.scope === claim.brand
                          ? "Re-verifying…"
                          : "Re-verify"}
                      </button>
                    </li>
                  ))}
                </ul>
              )}
            </div>

            {status && (
              <StatePanel tone="success" icon="check" title="Registry permission updated">
                {status}
              </StatePanel>
            )}
            {grant.isError && (
              <StatePanel tone="error" title="Could not grant verification">
                {(grant.error as Error).message}
              </StatePanel>
            )}
            {revoke.isError && (
              <StatePanel tone="error" title="Could not revoke verification">
                {(revoke.error as Error).message}
              </StatePanel>
            )}
          </section>
        ) : (
          <StatePanel icon="search" title="Select a publisher account">
            Search the registry and choose an account to manage its namespace claims.
          </StatePanel>
        )}
      </div>

      <RevokeDialog
        target={revokeTarget}
        busy={revoke.isPending}
        error={revoke.isError ? (revoke.error as Error).message : undefined}
        onCancel={() => setRevokeTarget(null)}
        onConfirm={({ account, scope }) => {
          clearFeedback();
          revoke.mutate({ account, scope });
        }}
      />
    </div>
  );
}

function Admin() {
  const me = useMe();
  usePageTitle("Registry administration");
  if (me.isPending) {
    return (
      <div className="admin-page">
        <div className="page-heading">
          <p className="eyebrow">Official operations</p>
          <h1>Registry administration</h1>
        </div>
        <LoadingRows count={2} />
      </div>
    );
  }
  if (me.isError && !isUnauthorized(me.error)) {
    return (
      <div className="narrow-page">
        <StatePanel
          tone="error"
          icon="shield"
          title="Could not verify the official session"
          action={
            <button className="button button-secondary" onClick={() => me.refetch()}>
              Try again
            </button>
          }
        >
          No administration controls are available until the registry can verify your session.
        </StatePanel>
      </div>
    );
  }
  if (!me.data) {
    return (
      <div className="narrow-page">
        <StatePanel
          icon="shield"
          title="Sign in with an official account"
          action={
            <Link className="button button-primary" to="/account">
              Go to sign in
            </Link>
          }
        >
          Registry administration is available only through a protected official web session.
        </StatePanel>
      </div>
    );
  }
  if (!me.data.official) {
    return (
      <div className="narrow-page">
        <StatePanel tone="error" icon="shield" title="Official account access is required">
          You are signed in as {me.data.account}, which does not have registry administration
          authority.
        </StatePanel>
      </div>
    );
  }
  return <AdminDashboard accountEmail={me.data.account} />;
}

const adminRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin",
  component: Admin,
});

function AdminComponentRequestRoute() {
  usePageTitle("Component requests");
  return <AdminComponentRequestsPage />;
}

const adminRequestsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/admin/requests",
  component: AdminComponentRequestRoute,
});

// ---------------------------------------------------------------------------

const MANIFEST_EXAMPLE = `[package]
name = "@contrib/your-name"
version = "1.0.0"
license = "MIT"                 # required
description = "One line: what the package gives a design."
repository = "https://github.com/you/your-name"`;

function Docs() {
  usePageTitle("Publishing guide");
  return (
    <div className="docs-page">
      <div className="page-heading docs-hero">
        <p className="eyebrow">Publisher documentation</p>
        <h1>Ship a trusted hardware library.</h1>
        <p>
          From registry identity to immutable release: the complete path for publishing a CoHDL
          package.
        </p>
      </div>

      <div className="docs-layout">
        <aside className="docs-rail">
          <span>On this page</span>
          <nav aria-label="Publishing guide sections">
            <a href="#access">1. Get access</a>
            <a href="#namespaces">2. Choose a namespace</a>
            <a href="#manifest">3. Describe the package</a>
            <a href="#documents">4. Include documents</a>
            <a href="#publish">5. Publish</a>
            <a href="#consume">6. Consume</a>
          </nav>
          <div className="docs-rail-note">
            <Icon name="shield" size={18} />
            <span>Trust comes from the package name’s structure, not a self-declared flag.</span>
          </div>
        </aside>

        <div className="docs-content">
          <section id="access" className="docs-step">
            <span className="step-number">01</span>
            <div>
              <p className="eyebrow">Registry identity</p>
              <h2>Create access</h2>
              <p>
                Create an account and mint a one-time CLI token from the{" "}
                <Link to="/account">Account</Link> page. The registry stores only the token hash.
              </p>
              <CommandBox command="cohdl login" />
            </div>
          </section>

          <section id="namespaces" className="docs-step">
            <span className="step-number">02</span>
            <div>
              <p className="eyebrow">Structural trust</p>
              <h2>Choose a namespace</h2>
              <p>
                The package name itself communicates who is authorized to publish it. These tiers
                cannot be overridden by manifest metadata.
              </p>
              <div className="namespace-grid">
                <article>
                  <TierBadge tier="official" />
                  <code>std</code>
                  <p>Bare names are reserved for CoHDL’s official libraries.</p>
                </article>
                <article id="brands">
                  <TierBadge tier="brand" />
                  <code>@yourbrand/name</code>
                  <p>Requires human-verified authority for the named manufacturer.</p>
                </article>
                <article>
                  <TierBadge tier="contrib" />
                  <code>@contrib/your-name</code>
                  <p>Open to every authenticated community publisher.</p>
                </article>
              </div>
            </div>
          </section>

          <section id="manifest" className="docs-step">
            <span className="step-number">03</span>
            <div>
              <p className="eyebrow">Package identity</p>
              <h2>Describe the release</h2>
              <p>
                Your <code>cohdl.toml</code> is the identity authority inside the archive. Its name
                and exact version must match the publish request.
              </p>
              <pre className="code-panel">
                <code>{MANIFEST_EXAMPLE}</code>
              </pre>
              <div className="docs-callout">
                <Icon name="document" size={19} />
                <p>
                  <strong>License is required.</strong> Description and repository are optional,
                  but strong metadata makes a library easier to evaluate and reuse.
                </p>
              </div>
            </div>
          </section>

          <section id="documents" className="docs-step">
            <span className="step-number">04</span>
            <div>
              <p className="eyebrow">Design context</p>
              <h2>Include documents and datasheets</h2>
              <p>
                Every RFC-017 <code>#[doc("path")]</code> reference is indexed with the release.
                Markdown and text render inline; PDFs, figures, errata, and other files remain
                downloadable from the immutable package artifact.
              </p>
              <p>
                Paths are package-relative, so README figures and document-to-document links
                continue to resolve within the same exact version.
              </p>
            </div>
          </section>

          <section id="publish" className="docs-step">
            <span className="step-number">05</span>
            <div>
              <p className="eyebrow">Immutable release</p>
              <h2>Publish from the package directory</h2>
              <CommandBox command="cohdl publish" />
              <p>
                The registry re-reads the manifest, enforces namespace authority, and computes the
                authoritative content hash itself. A published version can never be replaced.
              </p>
              <div className="proof-row">
                <span>
                  <Icon name="check" size={16} /> Identity matched
                </span>
                <span>
                  <Icon name="check" size={16} /> Namespace authorized
                </span>
                <span>
                  <Icon name="check" size={16} /> Content hashed
                </span>
              </div>
            </div>
          </section>

          <section id="consume" className="docs-step">
            <span className="step-number">06</span>
            <div>
              <p className="eyebrow">Reproducible design</p>
              <h2>Consume the exact identity</h2>
              <CommandBox command={"cohdl add <name>@<version>"} />
              <p>
                Run <code>cohdl install</code> on a fresh clone. Use <code>cohdl update</code> when
                you intentionally want to move pins. CoHDL accepts exact versions only—never
                floating ranges.
              </p>
              <Link className="button button-primary" to="/packages">
                Explore published packages <Icon name="arrow" size={16} />
              </Link>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}

const docsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: "/docs",
  component: Docs,
});

export const routeTree = rootRoute.addChildren([
  homeRoute,
  requestRoute,
  catalogRoute,
  packageRoute,
  accountRoute,
  adminRoute,
  adminRequestsRoute,
  docsRoute,
]);
