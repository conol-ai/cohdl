/**
 * cohdl.org — project site Worker.
 *
 * Two jobs: enforce transport and content security on every response, and
 * accept waitlist signups at POST /api/waitlist. Everything else is served
 * from Workers Assets.
 *
 * The signup endpoint answers both fetch() (JSON) and a plain <form> POST
 * (form-encoded, answered with a redirect), so the waitlist works with
 * JavaScript disabled.
 */

interface Env {
  DB: D1Database;
  ASSETS: Fetcher;
  /** Salt for the stored IP hash. Set with `wrangler secret put IP_SALT`. */
  IP_SALT?: string;
}

/** Signups allowed per IP per hour — generous for a shared office, useless for a script. */
const RATE_LIMIT = 5;
const RATE_WINDOW_MS = 60 * 60 * 1000;

const MAX_EMAIL_LEN = 254;
const MAX_SOURCE_LEN = 64;
const MAX_UA_LEN = 256;

/**
 * Pragmatic address check: one @, no whitespace or delimiters that indicate a
 * header-injection or list-paste attempt, and a dotted domain. Deliberately
 * not RFC 5322 — that grammar accepts addresses no signup form should.
 */
const EMAIL_RE = /^[^\s@,;:<>()[\]\\"]+@[^\s@,;:<>()[\]\\"]+\.[^\s@,;:<>()[\]\\"]{2,}$/;

const SECURITY_HEADERS: Record<string, string> = {
  "strict-transport-security": "max-age=31536000; includeSubDomains; preload",
  "x-content-type-options": "nosniff",
  "referrer-policy": "strict-origin-when-cross-origin",
  "x-frame-options": "DENY",
  "permissions-policy": "geolocation=(), microphone=(), camera=(), interest-cohort=()",
  "content-security-policy": [
    "default-src 'none'",
    "script-src 'self'",
    "style-src 'self'",
    "img-src 'self' data:",
    "font-src 'self'",
    "connect-src 'self'",
    "form-action 'self'",
    "base-uri 'none'",
    "frame-ancestors 'none'",
  ].join("; "),
};

/**
 * Loopback hosts are exempt from the https redirect and from HSTS: sending
 * `Strict-Transport-Security` for localhost pins EVERY local project's port to
 * https in the developer's browser. Deployed traffic always carries the zone's
 * own hostname, so this exempts nothing in production. (Same rule as the
 * registry Worker.)
 */
function isLoopbackHost(hostname: string): boolean {
  return (
    hostname === "localhost" ||
    hostname.endsWith(".localhost") ||
    hostname === "127.0.0.1" ||
    hostname === "[::1]" ||
    hostname === "::1"
  );
}

function withSecurityHeaders(response: Response): Response {
  const out = new Response(response.body, response);
  for (const [key, value] of Object.entries(SECURITY_HEADERS)) {
    out.headers.set(key, value);
  }
  return out;
}

function json(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      "content-type": "application/json; charset=utf-8",
      "cache-control": "no-store",
    },
  });
}

async function sha256Hex(input: string): Promise<string> {
  const digest = await crypto.subtle.digest("SHA-256", new TextEncoder().encode(input));
  return [...new Uint8Array(digest)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

/** Read the submitted fields from either a JSON body or a form POST. */
async function readSubmission(
  request: Request,
): Promise<{ email: string; source: string; honeypot: string; wantsJson: boolean }> {
  const contentType = request.headers.get("content-type") ?? "";
  let raw: Record<string, unknown> = {};
  let wantsJson = true;

  if (contentType.includes("application/json")) {
    raw = (await request.json().catch(() => ({}))) as Record<string, unknown>;
  } else {
    const form = await request.formData().catch(() => null);
    if (form) {
      raw = Object.fromEntries(form.entries());
      // A bare <form> POST has no fetch() wrapper to read JSON, so answer it
      // with a redirect instead.
      wantsJson = false;
    }
  }

  const str = (v: unknown) => (typeof v === "string" ? v : "");
  return {
    email: str(raw.email).trim().toLowerCase().slice(0, MAX_EMAIL_LEN + 1),
    source: str(raw.source).trim().slice(0, MAX_SOURCE_LEN),
    // Bots fill every field they find; humans never see this one.
    honeypot: str(raw.company).trim(),
    wantsJson,
  };
}

async function handleWaitlist(request: Request, env: Env, url: URL): Promise<Response> {
  if (request.method !== "POST") {
    return json(405, { ok: false, error: "method_not_allowed" });
  }

  // Same-origin only: a cross-site form should not be able to drive this.
  const origin = request.headers.get("origin");
  if (origin && new URL(origin).host !== url.host) {
    return json(403, { ok: false, error: "cross_origin" });
  }

  const { email, source, honeypot, wantsJson } = await readSubmission(request);

  const reply = (status: number, ok: boolean, code: string, message: string): Response => {
    if (wantsJson) return json(status, ok ? { ok, message } : { ok, error: code, message });
    // No-JS path: bounce back to the page with the outcome in the query string.
    const back = new URL("/", url);
    back.searchParams.set(ok ? "joined" : "error", ok ? "1" : code);
    return new Response(null, { status: 303, headers: { location: back.toString() } });
  };

  // Silently accept the honeypot so the bot records a success and moves on.
  if (honeypot) {
    return reply(200, true, "ok", "You're on the list.");
  }

  if (!email || email.length > MAX_EMAIL_LEN || !EMAIL_RE.test(email)) {
    return reply(400, false, "invalid_email", "That doesn't look like an email address.");
  }

  const ip = request.headers.get("cf-connecting-ip") ?? "";
  const ipHash = ip ? await sha256Hex(`${env.IP_SALT ?? "cohdl"}:${ip}`) : null;
  const now = new Date();

  try {
    if (ipHash) {
      const since = new Date(now.getTime() - RATE_WINDOW_MS).toISOString();
      const recent = await env.DB.prepare(
        "SELECT COUNT(*) AS n FROM waitlist WHERE ip_hash = ? AND created_at > ?",
      )
        .bind(ipHash, since)
        .first<{ n: number }>();
      if ((recent?.n ?? 0) >= RATE_LIMIT) {
        return reply(429, false, "rate_limited", "Too many signups from here. Try again later.");
      }
    }

    await env.DB.prepare(
      `INSERT INTO waitlist (email, created_at, source, ip_hash, user_agent)
       VALUES (?, ?, ?, ?, ?)
       ON CONFLICT(email) DO NOTHING`,
    )
      .bind(
        email,
        now.toISOString(),
        source || null,
        ipHash,
        (request.headers.get("user-agent") ?? "").slice(0, MAX_UA_LEN) || null,
      )
      .run();
  } catch {
    return reply(500, false, "storage", "Couldn't save that just now. Please try again.");
  }

  // Identical answer whether or not the address was already stored — a
  // waitlist should not double as an address-enumeration oracle.
  return reply(200, true, "ok", "You're on the list. We'll email you when CoHDL opens up.");
}

export default {
  async fetch(request: Request, env: Env): Promise<Response> {
    const url = new URL(request.url);
    const local = isLoopbackHost(url.hostname);

    // Enforce HTTPS here rather than relying on a zone-level rule.
    if (url.protocol === "http:" && !local) {
      url.protocol = "https:";
      return Response.redirect(url.toString(), 301);
    }

    let response: Response;
    if (url.pathname === "/api/waitlist") {
      response = await handleWaitlist(request, env, url);
    } else if (url.pathname.startsWith("/api/")) {
      response = json(404, { ok: false, error: "not_found" });
    } else {
      response = await env.ASSETS.fetch(request);
    }

    return local ? response : withSecurityHeaders(response);
  },
} satisfies ExportedHandler<Env>;
