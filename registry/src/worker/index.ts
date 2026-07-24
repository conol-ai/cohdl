// registry.cohdl.org — the RFC-030 registry Worker.
//
// External contract endpoints (the shapes the cohdl CLI speaks):
//   GET  /packages/{name}                → { name, versions: ["X.Y.Z", …] }
//   GET  /packages/{name}/{ver}          → { name, version, hash, size, published_at }
//   GET  /packages/{name}/{ver}.tar      → the package content (from R2)
//   POST /packages/{name}/{ver}          → publish (Bearer token; server recomputes
//                                          the RFC-029 hash — authoritative)
//   POST /login                          → token verify → { account, official, brands }
//
// Web-UI endpoints (/api/*): signup/session/tokens/search/recent/package.
// Everything else falls through to Workers Assets (the SPA).

import {
  Env,
  accountForSession,
  accountForToken,
  createSession,
  hashPassword,
  sha256Hex,
  verifiedBrands,
  verifyPassword,
} from "./auth";
import { packageContentHash } from "./hash";
import { nameTier, publishRejection } from "./namespace";
import { readTar } from "./tar";

const JSON_HEADERS = { "Content-Type": "application/json" };

function json(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS, ...headers } });
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

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const { pathname } = url;
    try {
      if (pathname === "/login" && request.method === "POST") {
        return await cliLogin(env, request);
      }
      if (pathname.startsWith("/packages/")) {
        return await packages(env, request, pathname);
      }
      if (pathname.startsWith("/api/")) {
        return await api(env, request, url);
      }
    } catch (e) {
      return json(500, { error: `internal error: ${e instanceof Error ? e.message : e}` });
    }
    // Everything else: the SPA (Workers Assets serves it; run_worker_first
    // keeps this fetch handler out of the way for asset routes).
    return new Response("not found", { status: 404 });
  },
} satisfies ExportedHandler<Env>;

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
      "SELECT hash, size, r2_key, published_at FROM versions WHERE name = ? AND version = ?",
    )
      .bind(name, version)
      .first<{ hash: string; size: number; r2_key: string; published_at: string }>();
    if (!row) return json(404, { error: "not found" });
    if (tar) {
      const obj = await env.PKG.get(row.r2_key);
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
    // The server's own hash is the authoritative identity (RFC-030): the
    // publisher's local computation is never trusted.
    const hash = await packageContentHash(files);

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
      "INSERT INTO versions (name, version, hash, size, r2_key, published_at) VALUES (?, ?, ?, ?, ?, ?)",
    )
      .bind(name, version, hash, body.length, r2Key, now)
      .run();
    return json(200, { name, version, hash });
  }

  return json(405, { error: "method not allowed" });
}

// ---------------------------------------------------------------------------
// Web UI API
// ---------------------------------------------------------------------------

async function api(env: Env, request: Request, url: URL): Promise<Response> {
  const path = url.pathname;

  if (path === "/api/signup" && request.method === "POST") {
    const { email, password } = await request.json<{ email?: string; password?: string }>();
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
    const { email, password } = await request.json<{ email?: string; password?: string }>();
    const row = await env.DB.prepare("SELECT id, password_hash FROM accounts WHERE email = ?")
      .bind(email ?? "")
      .first<{ id: number; password_hash: string }>();
    if (!row || !password || !(await verifyPassword(password, row.password_hash))) {
      return json(401, { error: "bad email or password" });
    }
    return sessionResponse(env, email!);
  }

  if (path === "/api/me" && request.method === "GET") {
    const account = await accountForSession(env, request);
    if (!account) return json(401, { error: "not signed in" });
    return json(200, {
      account: account.email,
      official: account.is_official === 1,
      brands: await verifiedBrands(env, account.id),
    });
  }

  if (path === "/api/tokens" && request.method === "POST") {
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
    return json(200, { token });
  }

  if (path === "/api/search" && request.method === "GET") {
    const q = (url.searchParams.get("q") ?? "").trim();
    const rows = await env.DB.prepare(
      `SELECT p.name, p.tier, MAX(v.published_at) AS updated,
              (SELECT version FROM versions WHERE name = p.name ORDER BY published_at DESC LIMIT 1) AS latest
       FROM packages p JOIN versions v ON v.name = p.name
       WHERE p.name LIKE ? GROUP BY p.name ORDER BY updated DESC LIMIT 50`,
    )
      .bind(`%${q}%`)
      .all();
    return json(200, { results: rows.results });
  }

  if (path === "/api/recent" && request.method === "GET") {
    const rows = await env.DB.prepare(
      `SELECT v.name, v.version, v.published_at, p.tier
       FROM versions v JOIN packages p ON p.name = v.name
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
      "SELECT version, hash, size, published_at FROM versions WHERE name = ? ORDER BY published_at DESC",
    )
      .bind(name)
      .all();
    return json(200, { ...pkg, versions: versions.results });
  }

  return json(404, { error: "not found" });
}

async function sessionResponse(env: Env, email: string): Promise<Response> {
  const row = await env.DB.prepare("SELECT id FROM accounts WHERE email = ?")
    .bind(email)
    .first<{ id: number }>();
  const session = await createSession(env, row!.id);
  return json(
    200,
    { account: email },
    {
      "Set-Cookie": `session=${session}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age=${7 * 24 * 3600}`,
    },
  );
}
