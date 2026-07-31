// registry.cohdl.org — the RFC-030 registry Worker.
//
// External contract endpoints (the shapes the cohdl CLI speaks):
//   GET  /packages/{name}                → { name, versions: ["X.Y.Z", …] }
//   GET  /packages/{name}/{ver}          → { name, version, hash, size, published_at,
//                                            description, license, repository, docs }
//   GET  /packages/{name}/{ver}.tar      → the package content (from R2)
//   POST /packages/{name}/{ver}          → publish (Bearer token; server recomputes
//                                          the RFC-029 hash — authoritative)
//   POST /login                          → token verify → { account, official, brands }
//
// Web-UI endpoints (/api/*): signup/session/tokens/search/recent/package/doc.
// Everything else falls through to Workers Assets (the SPA).

import {
  Env,
  accountForSession,
  accountForToken,
  createSession,
  hashPassword,
  sessionIdForRequest,
  sha256Hex,
  verifiedBrands,
  verifyPassword,
} from "./auth";
import { adminApi, hasJsonContentType, validWriteOrigin } from "./admin";
import { docContentType, docPaths, validDocPath } from "./docs";
import { packageContentHash } from "./hash";
import { metadataRejection, parsePackageManifest } from "./manifest";
import { nameTier, publishRejection } from "./namespace";
import { readTar } from "./tar";

const JSON_HEADERS = { "Content-Type": "application/json" };
const SPA_CONTENT_SECURITY_POLICY = [
  "default-src 'self'",
  "base-uri 'none'",
  "object-src 'none'",
  "frame-ancestors 'none'",
  "form-action 'self'",
  "script-src 'self' https://www.google.com/recaptcha/ https://www.gstatic.com/recaptcha/",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: https:",
  "font-src 'self' data:",
  "connect-src 'self' https://www.google.com/recaptcha/",
  "frame-src https://www.google.com/recaptcha/ https://recaptcha.google.com/recaptcha/",
].join("; ");

function json(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS, ...headers } });
}

/// A `versions` row as the metadata-bearing queries select it.
interface VersionRow {
  version?: string;
  hash?: string;
  size?: number;
  r2_key?: string;
  published_at?: string;
  description: string | null;
  license: string | null;
  repository: string | null;
  docs: string | null;
}

/// The `[package]` metadata a version carries, in the shape every API
/// returns it: absent keys stay null, `docs` is always an array. Rows
/// published before the metadata columns existed read as null — the same
/// shape as a manifest that simply declares nothing.
function manifestMeta(row: VersionRow) {
  let docs: string[] = [];
  if (row.docs) {
    try {
      const parsed: unknown = JSON.parse(row.docs);
      if (Array.isArray(parsed)) docs = parsed.filter((d): d is string => typeof d === "string");
    } catch {
      docs = [];
    }
  }
  return {
    description: row.description ?? null,
    license: row.license ?? null,
    repository: row.repository ?? null,
    docs,
  };
}

/// `/packages/<name…>[/<version>[.tar]]` — the name may contain one `/`
/// (scoped `@scope/name`), so parse from the RIGHT: a trailing segment that
/// starts with a digit is the version.
function parsePackagePath(pathname: string): { name: string; version?: string; tar: boolean } | null {
  const rest = decodeURIComponent(pathname).replace(/^\/packages\//, "");
  if (!rest || rest === pathname) return null;
  const tar = rest.endsWith(".tar");
  const stem = tar ? rest.slice(0, -4) : rest;
  const slash = stem.lastIndexOf("/");
  if (slash >= 0) {
    const last = stem.slice(slash + 1);
    if (/^\d/.test(last)) {
      return { name: stem.slice(0, slash), version: last, tar };
    }
  }
  if (tar) return null; // a tarball URL always names a version
  return { name: stem, tar: false };
}

const EXACT_VERSION = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/;

/// Production responses are HTTPS-only and cannot be embedded by another
/// origin. The frame guard matters especially for the cookie-authenticated
/// admin UI: a same-site sibling must not be able to clickjack real controls.
export function withProductionSecurity(resp: Response): Response {
  const out = new Response(resp.body, resp);
  out.headers.set("Strict-Transport-Security", "max-age=31536000; includeSubDomains");
  out.headers.set("X-Frame-Options", "DENY");
  const csp = out.headers.get("Content-Security-Policy");
  out.headers.set(
    "Content-Security-Policy",
    csp
      ? `${csp}; frame-ancestors 'none'; object-src 'none'; base-uri 'none'`
      : SPA_CONTENT_SECURITY_POLICY,
  );
  return out;
}

/// A local dev server (`npm run dev`, `wrangler dev`) is reached over plain
/// http on a loopback host. Neither the https redirect nor HSTS may apply
/// there: the redirect makes dev unusable, and an HSTS header on `localhost`
/// pins EVERY local project's port to https in the developer's browser.
/// Deployed traffic always carries the zone's own hostname, so this exempts
/// nothing in production.
function isLoopbackHost(hostname: string): boolean {
  return (
    hostname === "localhost" ||
    hostname.endsWith(".localhost") ||
    hostname === "127.0.0.1" ||
    hostname === "[::1]" ||
    hostname === "::1"
  );
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const local = isLoopbackHost(url.hostname);
    // https-only: permanent-redirect any plain-http request.
    if (url.protocol === "http:" && !local) {
      url.protocol = "https:";
      return Response.redirect(url.toString(), 301);
    }
    const resp = await route(env, request, url);
    return local ? resp : withProductionSecurity(resp);
  },
} satisfies ExportedHandler<Env>;

async function route(env: Env, request: Request, url: URL): Promise<Response> {
  const { pathname } = url;
  try {
    if (pathname === "/login" && request.method === "POST") {
      return await cliLogin(env, request);
    }
    // `/packages` (and its harmless trailing-slash form) is the web
    // catalogue. Only a non-empty suffix belongs to the CLI/package API.
    if (pathname.startsWith("/packages/") && pathname !== "/packages/") {
      return await packages(env, request, pathname);
    }
    if (pathname === "/api/admin" || pathname.startsWith("/api/admin/")) {
      return await adminApi(env, request, url);
    }
    if (pathname.startsWith("/api/")) {
      return await api(env, request, url);
    }
  } catch (e) {
    return json(500, { error: `internal error: ${e instanceof Error ? e.message : e}` });
  }
  // Everything else: the SPA via the assets binding.
  return env.ASSETS.fetch(request);
}

// ---------------------------------------------------------------------------
// CLI contract
// ---------------------------------------------------------------------------

async function cliLogin(env: Env, request: Request): Promise<Response> {
  const account = await accountForToken(env, request);
  if (!account) return json(401, { error: "bad token" });
  return json(200, {
    account: account.email,
    official: account.is_official === 1,
    brands: await verifiedBrands(env, account.id),
  });
}

async function packages(env: Env, request: Request, pathname: string): Promise<Response> {
  const parsed = parsePackagePath(pathname);
  if (!parsed) return json(404, { error: "not found" });
  const { name, version, tar } = parsed;

  if (request.method === "GET") {
    if (!version) {
      const rows = await env.DB.prepare(
        "SELECT version FROM versions WHERE name = ? ORDER BY published_at ASC",
      )
        .bind(name)
        .all<{ version: string }>();
      if (rows.results.length === 0) return json(404, { error: "not found" });
      return json(200, { name, versions: rows.results.map((r) => r.version) });
    }
    const row = await env.DB.prepare(
      `SELECT hash, size, r2_key, published_at, description, license, repository, docs
         FROM versions WHERE name = ? AND version = ?`,
    )
      .bind(name, version)
      .first<VersionRow>();
    if (!row) return json(404, { error: "not found" });
    if (tar) {
      const obj = await env.PKG.get(row.r2_key!);
      if (!obj) return json(404, { error: "content missing" });
      return new Response(obj.body, {
        headers: { "Content-Type": "application/x-tar", "Content-Length": String(row.size) },
      });
    }
    return json(200, {
      name,
      version,
      hash: row.hash,
      size: row.size,
      published_at: row.published_at,
      ...manifestMeta(row),
    });
  }

  if (request.method === "POST") {
    if (!version || tar) return json(400, { error: "publish to /packages/{name}/{version}" });
    if (!EXACT_VERSION.test(version)) {
      return json(400, { error: `\`${version}\` is not an exact X.Y.Z version` });
    }
    const account = await accountForToken(env, request);
    if (!account) return json(401, { error: "login required" });

    // Three-tier namespace: the server is the final arbiter (RFC-030).
    const grants = {
      isOfficial: account.is_official === 1,
      verifiedBrands: await verifiedBrands(env, account.id),
    };
    const rejection = publishRejection(name, grants);
    if (rejection) return json(403, { error: rejection });

    const existing = await env.DB.prepare("SELECT owner_account FROM packages WHERE name = ?")
      .bind(name)
      .first<{ owner_account: number }>();
    if (existing && existing.owner_account !== account.id) {
      return json(403, { error: `\`${name}\` is owned by another account` });
    }
    const dup = await env.DB.prepare("SELECT 1 FROM versions WHERE name = ? AND version = ?")
      .bind(name, version)
      .first();
    if (dup) {
      return json(409, {
        error: `\`${name} ${version}\` is already published — a version is one immutable identity; publish a new version`,
      });
    }

    const body = new Uint8Array(await request.arrayBuffer());
    let files: Map<string, Uint8Array>;
    try {
      files = readTar(body);
    } catch (e) {
      return json(400, { error: `bad package archive: ${e instanceof Error ? e.message : e}` });
    }
    if (![...files.keys()].some((f) => f.endsWith(".cohdl"))) {
      return json(400, { error: "the package contains no .cohdl files" });
    }

    // The manifest inside the archive is the sole identity authority
    // (RFC-029) — the URL must agree with what the package declares about
    // itself, and its `[package]` metadata is what the web UI displays.
    const manifestFile = files.get("cohdl.toml");
    if (!manifestFile) {
      return json(400, { error: "the package has no cohdl.toml manifest" });
    }
    const manifest = parsePackageManifest(new TextDecoder().decode(manifestFile));
    if (manifest.name !== name) {
      return json(400, {
        error: `the manifest declares \`[package] name = "${manifest.name ?? ""}"\` but this publishes \`${name}\` — the manifest is the sole identity authority`,
      });
    }
    if (manifest.version !== version) {
      return json(400, {
        error: `the manifest declares \`[package] version = "${manifest.version ?? ""}"\` but this publishes \`${version}\` — the manifest is the sole version authority`,
      });
    }
    const metaRejection = metadataRejection(manifest);
    if (metaRejection) return json(400, { error: metaRejection });
    // The server's own hash is the authoritative identity (RFC-030): the
    // publisher's local computation is never trusted.
    const hash = await packageContentHash(files);
    // RFC-017 documents the package ships, as an index over the tar (the
    // tar in R2 stays the only copy of the bytes).
    const docs = docPaths(files);

    const r2Key = `pkg/${name}/${version}.tar`;
    await env.PKG.put(r2Key, body);
    const now = new Date().toISOString();
    const tierInfo = nameTier(name);
    const tier = "tier" in tierInfo ? tierInfo.tier : "contrib";
    if (!existing) {
      await env.DB.prepare(
        "INSERT INTO packages (name, tier, owner_account, created_at) VALUES (?, ?, ?, ?)",
      )
        .bind(name, tier, account.id, now)
        .run();
    }
    await env.DB.prepare(
      `INSERT INTO versions
         (name, version, hash, size, r2_key, published_at, description, license, repository, docs)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    )
      .bind(
        name,
        version,
        hash,
        body.length,
        r2Key,
        now,
        manifest.description,
        manifest.license,
        manifest.repository,
        JSON.stringify(docs),
      )
      .run();
    return json(200, { name, version, hash, docs });
  }

  return json(405, { error: "method not allowed" });
}

// ---------------------------------------------------------------------------
// Web UI API
// ---------------------------------------------------------------------------

/// reCAPTCHA v3 verification (signup/sign-in only — never the CLI token
/// flow). Unconfigured secret = pass-through, so the service works before
/// the dashboard variables exist.
async function recaptchaOk(
  env: Env,
  request: Request,
  token: string | undefined,
  action: string,
): Promise<boolean> {
  if (!env.RECAPTCHA_SECRET_KEY) return true;
  if (!token) return false;
  const form = new URLSearchParams({
    secret: env.RECAPTCHA_SECRET_KEY,
    response: token,
    remoteip: request.headers.get("CF-Connecting-IP") ?? "",
  });
  const resp = await fetch("https://www.google.com/recaptcha/api/siteverify", {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: form.toString(),
  });
  if (!resp.ok) return false;
  const data = (await resp.json()) as { success?: boolean; score?: number; action?: string };
  return data.success === true && data.action === action && (data.score ?? 0) >= 0.5;
}

/// Cookie-authenticated browser mutations must be non-simple, exact-origin
/// JSON requests. This blocks cross-origin form/no-cors writes and same-site
/// sibling origins from driving a user's registry session.
export function webJsonWriteRejection(request: Request, url: URL): Response | null {
  if (!validWriteOrigin(request, url)) {
    return json(403, { error: "same-origin request required" });
  }
  if (!hasJsonContentType(request)) {
    return json(415, { error: "Content-Type application/json required" });
  }
  return null;
}

async function api(env: Env, request: Request, url: URL): Promise<Response> {
  const path = url.pathname;

  if (path === "/api/config" && request.method === "GET") {
    return json(200, { recaptcha_site_key: env.RECAPTCHA_SITE_KEY ?? null });
  }

  if (path === "/api/signup" && request.method === "POST") {
    const rejection = webJsonWriteRejection(request, url);
    if (rejection) return rejection;
    let body: {
      email?: string;
      password?: string;
      recaptcha?: string;
    };
    try {
      body = await request.json();
    } catch {
      return json(400, { error: "request body must be valid JSON" });
    }
    const { email, password, recaptcha } = body;
    if (!(await recaptchaOk(env, request, recaptcha, "signup"))) {
      return json(403, { error: "reCAPTCHA verification failed — reload the page and try again" });
    }
    if (!email || !password || password.length < 8) {
      return json(400, { error: "email and a password of at least 8 characters are required" });
    }
    const exists = await env.DB.prepare("SELECT 1 FROM accounts WHERE email = ?")
      .bind(email)
      .first();
    if (exists) return json(409, { error: "an account with that email already exists" });
    const now = new Date().toISOString();
    await env.DB.prepare(
      "INSERT INTO accounts (email, password_hash, is_official, created_at) VALUES (?, ?, 0, ?)",
    )
      .bind(email, await hashPassword(password), now)
      .run();
    return sessionResponse(env, email);
  }

  if (path === "/api/session" && request.method === "POST") {
    const rejection = webJsonWriteRejection(request, url);
    if (rejection) return rejection;
    let body: {
      email?: string;
      password?: string;
      recaptcha?: string;
    };
    try {
      body = await request.json();
    } catch {
      return json(400, { error: "request body must be valid JSON" });
    }
    const { email, password, recaptcha } = body;
    if (!(await recaptchaOk(env, request, recaptcha, "login"))) {
      return json(403, { error: "reCAPTCHA verification failed — reload the page and try again" });
    }
    const row = await env.DB.prepare("SELECT id, password_hash FROM accounts WHERE email = ?")
      .bind(email ?? "")
      .first<{ id: number; password_hash: string }>();
    if (!row || !password || !(await verifyPassword(password, row.password_hash))) {
      return json(401, { error: "bad email or password" });
    }
    return sessionResponse(env, email!);
  }

  if (path === "/api/session" && request.method === "DELETE") {
    const rejection = webJsonWriteRejection(request, url);
    if (rejection) return rejection;
    const sessionId = sessionIdForRequest(request);
    if (sessionId) await env.SESSIONS.delete(`session:${sessionId}`);
    return json(
      200,
      { signed_out: true },
      {
        "Cache-Control": "no-store",
        "Set-Cookie":
          "__Host-session=; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=0",
      },
    );
  }

  if (path === "/api/me" && request.method === "GET") {
    const account = await accountForSession(env, request);
    if (!account) {
      return json(401, { error: "not signed in" }, { "Cache-Control": "no-store" });
    }
    return json(
      200,
      {
        account: account.email,
        official: account.is_official === 1,
        brands: await verifiedBrands(env, account.id),
      },
      { "Cache-Control": "no-store" },
    );
  }

  if (path === "/api/tokens" && request.method === "POST") {
    const rejection = webJsonWriteRejection(request, url);
    if (rejection) return rejection;
    const account = await accountForSession(env, request);
    if (!account) return json(401, { error: "not signed in" });
    const raw = [...crypto.getRandomValues(new Uint8Array(24))]
      .map((b) => b.toString(16).padStart(2, "0"))
      .join("");
    const token = `cohdl_${raw}`;
    await env.DB.prepare(
      "INSERT INTO tokens (token_hash, account_id, created_at) VALUES (?, ?, ?)",
    )
      .bind(await sha256Hex(token), account.id, new Date().toISOString())
      .run();
    // Shown exactly once; only the hash is stored.
    return json(200, { token }, { "Cache-Control": "no-store" });
  }

  if (path === "/api/search" && request.method === "GET") {
    const q = (url.searchParams.get("q") ?? "").trim();
    const like = `%${q}%`;
    const requestedTier = url.searchParams.get("tier");
    const tier =
      requestedTier === "official" || requestedTier === "brand" || requestedTier === "contrib"
        ? requestedTier
        : "";
    const orderBy = url.searchParams.get("sort") === "name" ? "p.name ASC" : "updated DESC";
    // Descriptions live only on `versions` (one immutable fact per version);
    // anything package-level derives from the newest version by subquery —
    // both what a hit displays and what a hit matches on.
    const totalRow = await env.DB.prepare(
      `SELECT COUNT(*) AS total
         FROM packages p
        WHERE (p.name LIKE ?
           OR (SELECT description FROM versions WHERE name = p.name ORDER BY published_at DESC LIMIT 1) LIKE ?)
          AND (? = '' OR p.tier = ?)`,
    )
      .bind(like, like, tier, tier)
      .first<{ total: number }>();
    const rows = await env.DB.prepare(
      `SELECT p.name, p.tier, MAX(v.published_at) AS updated,
              (SELECT version FROM versions WHERE name = p.name ORDER BY published_at DESC LIMIT 1) AS latest,
              (SELECT description FROM versions WHERE name = p.name ORDER BY published_at DESC LIMIT 1) AS description
       FROM packages p JOIN versions v ON v.name = p.name
       WHERE (p.name LIKE ?
          OR (SELECT description FROM versions WHERE name = p.name ORDER BY published_at DESC LIMIT 1) LIKE ?)
         AND (? = '' OR p.tier = ?)
       GROUP BY p.name ORDER BY ${orderBy} LIMIT 50`,
    )
      .bind(like, like, tier, tier)
      .all();
    const total = Number(totalRow?.total ?? 0);
    return json(200, {
      results: rows.results,
      total,
      truncated: total > rows.results.length,
    });
  }

  if (path === "/api/recent" && request.method === "GET") {
    const rows = await env.DB.prepare(
      `SELECT v.name, v.version, v.published_at, v.description, p.tier
       FROM versions v JOIN packages p ON p.name = v.name
       WHERE v.version = (
         SELECT newest.version
           FROM versions newest
          WHERE newest.name = v.name
          ORDER BY newest.published_at DESC, newest.version DESC
          LIMIT 1
       )
       ORDER BY v.published_at DESC LIMIT 10`,
    ).all();
    return json(200, { results: rows.results });
  }

  if (path.startsWith("/api/packages/") && request.method === "GET") {
    const name = decodeURIComponent(path.slice("/api/packages/".length));
    const pkg = await env.DB.prepare("SELECT name, tier, created_at FROM packages WHERE name = ?")
      .bind(name)
      .first();
    if (!pkg) return json(404, { error: "not found" });
    const versions = await env.DB.prepare(
      `SELECT version, hash, size, published_at, description, license, repository, docs
         FROM versions WHERE name = ? ORDER BY published_at DESC`,
    )
      .bind(name)
      .all<VersionRow>();
    return json(200, {
      ...pkg,
      versions: versions.results.map((v) => ({
        version: v.version,
        hash: v.hash,
        size: v.size,
        published_at: v.published_at,
        ...manifestMeta(v),
      })),
    });
  }

  // One file out of a published version's immutable tar, for the web UI's
  // document rendering: `?pkg=<name>&version=<X.Y.Z>&path=<p>`.
  //
  // Any file the archive contains is servable, not just the `#[doc]`-declared
  // ones — a rendered document's own figures are relative paths that were
  // never declared themselves, and the whole tar is public at
  // `/packages/{name}/{ver}.tar` regardless. The `docs` list decides what the
  // UI *presents* as a document; this endpoint just serves package bytes,
  // sandboxed and never content-sniffed.
  if (path === "/api/doc" && request.method === "GET") {
    const pkg = url.searchParams.get("pkg") ?? "";
    const version = url.searchParams.get("version") ?? "";
    const docPath = url.searchParams.get("path") ?? "";
    if (!pkg || !EXACT_VERSION.test(version) || !docPath || !validDocPath(docPath)) {
      return json(400, { error: "need ?pkg=<name>&version=<X.Y.Z>&path=<package-relative path>" });
    }
    const row = await env.DB.prepare("SELECT r2_key FROM versions WHERE name = ? AND version = ?")
      .bind(pkg, version)
      .first<{ r2_key: string }>();
    if (!row) return json(404, { error: "not found" });
    const obj = await env.PKG.get(row.r2_key);
    if (!obj) return json(404, { error: "content missing" });
    let content: Uint8Array | undefined;
    try {
      content = readTar(new Uint8Array(await obj.arrayBuffer())).get(docPath);
    } catch (e) {
      return json(500, { error: `bad package archive: ${e instanceof Error ? e.message : e}` });
    }
    if (!content) {
      return json(404, { error: `\`${pkg} ${version}\` contains no file \`${docPath}\`` });
    }
    // The tar reader hands back a view into the archive; the response body is
    // that view's own bytes.
    const body = content.buffer.slice(
      content.byteOffset,
      content.byteOffset + content.byteLength,
    ) as ArrayBuffer;
    return new Response(body, {
      headers: {
        "Content-Type": docContentType(docPath),
        "Content-Security-Policy": "sandbox",
        "X-Content-Type-Options": "nosniff",
        // A published version is immutable, so its documents are too.
        "Cache-Control": "public, max-age=31536000, immutable",
      },
    });
  }

  return json(404, { error: "not found" });
}

async function sessionResponse(env: Env, email: string): Promise<Response> {
  const row = await env.DB.prepare("SELECT id, is_official FROM accounts WHERE email = ?")
    .bind(email)
    .first<{ id: number; is_official: number }>();
  const session = await createSession(env, row!.id);
  return json(
    200,
    {
      account: email,
      official: row!.is_official === 1,
      brands: await verifiedBrands(env, row!.id),
    },
    {
      "Cache-Control": "no-store",
      "Set-Cookie": `__Host-session=${session}; HttpOnly; Secure; SameSite=Strict; Path=/; Max-Age=${7 * 24 * 3600}`,
    },
  );
}
