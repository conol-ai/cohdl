// Accounts, API tokens (CLI), and sessions (web UI). Tokens are stored as
// sha256 hex — a leaked D1 row never yields a usable bearer token.

export interface Env {
  DB: D1Database;
  PKG: R2Bucket;
  SESSIONS: KVNamespace;
  /// Workers Assets (the SPA) — the worker fronts all routes for the
  /// https/HSTS enforcement and forwards non-API requests here.
  ASSETS: Fetcher;
  /// reCAPTCHA v3 keys, assigned in the Cloudflare dashboard (the site key
  /// as a plaintext variable — it is public and the UI fetches it via
  /// /api/config; the secret as a Worker secret). Both unset = reCAPTCHA
  /// disabled (local dev / pre-configuration).
  RECAPTCHA_SITE_KEY?: string;
  RECAPTCHA_SECRET_KEY?: string;
}

const PBKDF2_ITERS = 100_000;

function hex(buf: ArrayBuffer): string {
  return [...new Uint8Array(buf)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

export async function sha256Hex(text: string): Promise<string> {
  return hex(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(text)));
}

export async function hashPassword(password: string): Promise<string> {
  const salt = crypto.getRandomValues(new Uint8Array(16));
  const key = await derive(password, salt);
  return `pbkdf2$${PBKDF2_ITERS}$${hex(salt.buffer)}$${hex(key)}`;
}

export async function verifyPassword(password: string, stored: string): Promise<boolean> {
  const [scheme, iters, saltHex, hashHex] = stored.split("$");
  if (scheme !== "pbkdf2" || Number(iters) !== PBKDF2_ITERS) return false;
  const salt = new Uint8Array(saltHex.match(/../g)!.map((b) => parseInt(b, 16)));
  const key = await derive(password, salt);
  return hex(key) === hashHex;
}

async function derive(password: string, salt: Uint8Array): Promise<ArrayBuffer> {
  const material = await crypto.subtle.importKey(
    "raw",
    new TextEncoder().encode(password),
    "PBKDF2",
    false,
    ["deriveBits"],
  );
  return crypto.subtle.deriveBits(
    { name: "PBKDF2", hash: "SHA-256", salt: salt as BufferSource, iterations: PBKDF2_ITERS },
    material,
    256,
  );
}

export interface Account {
  id: number;
  email: string;
  is_official: number;
}

export function sessionIdForRequest(request: Request): string | null {
  const cookie = request.headers.get("Cookie") ?? "";
  return cookie.match(/(?:^|;\s*)__Host-session=([a-f0-9]{64})(?:;|$)/)?.[1] ?? null;
}

/// Resolve a `Bearer <token>` header to its account, or null.
export async function accountForToken(env: Env, request: Request): Promise<Account | null> {
  const auth = request.headers.get("Authorization") ?? "";
  const token = auth.startsWith("Bearer ") ? auth.slice(7).trim() : "";
  if (!token) return null;
  const tokenHash = await sha256Hex(token);
  return env.DB.prepare(
    "SELECT a.id, a.email, a.is_official FROM tokens t JOIN accounts a ON a.id = t.account_id WHERE t.token_hash = ?",
  )
    .bind(tokenHash)
    .first<Account>();
}

/// Resolve the web session cookie to its account, or null.
export async function accountForSession(env: Env, request: Request): Promise<Account | null> {
  const sessionId = sessionIdForRequest(request);
  if (!sessionId) return null;
  const accountId = await env.SESSIONS.get(`session:${sessionId}`);
  if (!accountId) return null;
  return env.DB.prepare("SELECT id, email, is_official FROM accounts WHERE id = ?")
    .bind(Number(accountId))
    .first<Account>();
}

export async function createSession(env: Env, accountId: number): Promise<string> {
  const id = hex(crypto.getRandomValues(new Uint8Array(32)).buffer);
  await env.SESSIONS.put(`session:${id}`, String(accountId), {
    expirationTtl: 7 * 24 * 3600,
  });
  return id;
}

export async function verifiedBrands(env: Env, accountId: number): Promise<string[]> {
  const rows = await env.DB.prepare(
    "SELECT brand FROM brands WHERE account_id = ? AND verified = 1",
  )
    .bind(accountId)
    .all<{ brand: string }>();
  return rows.results.map((r) => r.brand);
}
