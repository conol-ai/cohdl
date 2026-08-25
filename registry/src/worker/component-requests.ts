// Public component-library requests and their official-account review queue.

import { accountForSession, type Account, type Env } from "./auth";
import { hasJsonContentType, validWriteOrigin } from "./admin";
import { recaptchaOk } from "./recaptcha";

const JSON_HEADERS = {
  "Content-Type": "application/json",
  "Cache-Control": "no-store",
};
const PUBLIC_BODY_LIMIT = 12 * 1024;
const ADMIN_BODY_LIMIT = 1024;
const MAX_QUERY_BYTES = 256;
const STATUS_VALUES = ["open", "resolved"] as const;
const SORT_VALUES = ["requested", "newest"] as const;

export type ComponentRequestStatus = (typeof STATUS_VALUES)[number];
export type ComponentRequestSort = (typeof SORT_VALUES)[number];

export interface ComponentRequestInput {
  manufacturer: string;
  manufacturer_key: string;
  part_number: string;
  part_number_key: string;
  datasheet_url: string;
  description: string | null;
}

export interface ComponentRequestRow {
  id: number;
  manufacturer: string;
  part_number: string;
  datasheet_url: string;
  description: string | null;
  status: ComponentRequestStatus;
  request_count: number;
  created_at: string;
  last_requested_at: string;
  updated_at: string;
  resolved_at: string | null;
}

export interface ComponentRequestList {
  requests: ComponentRequestRow[];
  truncated: boolean;
}

export interface ComponentRequestStore {
  submit(
    input: ComponentRequestInput,
    now: string,
  ): Promise<{ duplicate: boolean }>;
  list(
    status: ComponentRequestStatus | "all",
    sort: ComponentRequestSort,
    query: string,
  ): Promise<ComponentRequestList>;
  setStatus(
    id: number,
    status: ComponentRequestStatus,
    actorAccountId: number,
    now: string,
  ): Promise<{ request: ComponentRequestRow; changed: boolean } | null>;
}

export type HumanVerification = "ok" | "failed" | "unavailable";

export interface PublicComponentRequestDependencies {
  store: ComponentRequestStore;
  rateLimit(request: Request): Promise<boolean>;
  verifyHuman(request: Request, token: string | undefined): Promise<HumanVerification>;
  now(): string;
  log(entry: Record<string, unknown>): void;
}

export interface AdminComponentRequestDependencies {
  store: ComponentRequestStore;
  sessionAccount(request: Request): Promise<Account | null>;
  now(): string;
  log(entry: Record<string, unknown>): void;
}

type ReadJsonResult =
  | { ok: true; value: unknown }
  | { ok: false; reason: "invalid" | "too_large" };

type ParsedPublicBody =
  | { ok: true; input: ComponentRequestInput; recaptcha?: string }
  | { ok: false; fields: Record<string, string> };

interface SubmitRow {
  request_count: number;
}

interface D1ComponentRequestRow extends ComponentRequestRow {
  status: ComponentRequestStatus;
}

function json(
  status: number,
  body: unknown,
  headers: Record<string, string> = {},
): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...JSON_HEADERS, ...headers },
  });
}

function safeLog(
  log: (entry: Record<string, unknown>) => void,
  entry: Record<string, unknown>,
): void {
  try {
    log(entry);
  } catch {
    // Logging must not turn a completed request into a 500.
  }
}

function utf8Length(value: string): number {
  return new TextEncoder().encode(value).length;
}

function scalarLength(value: string): number {
  return [...value].length;
}

function visibleField(value: unknown): string | null {
  if (typeof value !== "string") return null;
  const normalized = value
    .normalize("NFC")
    .trim()
    .replace(/[ \t\r\n\f]+/g, " ");
  if (/\p{Cc}|\p{Cf}/u.test(normalized)) return null;
  return normalized;
}

function canonicalField(value: string): string {
  return value.normalize("NFKC").toLowerCase();
}

function descriptionField(value: unknown): string | null | undefined {
  if (value === undefined || value === null) return null;
  if (typeof value !== "string") return undefined;
  const normalized = value.normalize("NFC").replace(/\r\n?/g, "\n").trim();
  if (!normalized) return null;
  for (const character of normalized) {
    const code = character.codePointAt(0)!;
    if (
      /\p{Cf}/u.test(character) ||
      (code < 0x20 && code !== 0x09 && code !== 0x0a) ||
      (code >= 0x7f && code <= 0x9f)
    ) {
      return undefined;
    }
  }
  return normalized;
}

function datasheetField(value: unknown): string | null {
  if (typeof value !== "string" || utf8Length(value) > 2048) return null;
  try {
    const parsed = new URL(value.trim());
    if (parsed.protocol !== "https:" || parsed.username || parsed.password || !parsed.hostname) {
      return null;
    }
    return parsed.toString();
  } catch {
    return null;
  }
}

function exactObject(value: unknown, allowed: ReadonlySet<string>): value is Record<string, unknown> {
  return (
    typeof value === "object" &&
    value !== null &&
    !Array.isArray(value) &&
    Object.keys(value).every((key) => allowed.has(key))
  );
}

export function parseComponentRequestBody(value: unknown): ParsedPublicBody {
  const allowed = new Set([
    "manufacturer",
    "part_number",
    "datasheet_url",
    "description",
    "recaptcha",
  ]);
  if (!exactObject(value, allowed)) {
    return { ok: false, fields: { form: "Submit only the fields shown in the form." } };
  }

  const fields: Record<string, string> = {};
  const manufacturer = visibleField(value.manufacturer);
  if (
    !manufacturer ||
    scalarLength(manufacturer) > 128 ||
    utf8Length(manufacturer) > 256
  ) {
    fields.manufacturer = "Enter a manufacturer name of 128 characters or fewer.";
  }
  const partNumber = visibleField(value.part_number);
  if (!partNumber || scalarLength(partNumber) > 128 || utf8Length(partNumber) > 256) {
    fields.part_number = "Enter a part number of 128 characters or fewer.";
  }
  const datasheetUrl = datasheetField(value.datasheet_url);
  if (!datasheetUrl) {
    fields.datasheet_url = "Enter a complete HTTPS datasheet or product-page URL.";
  }
  const description = descriptionField(value.description);
  if (
    description === undefined ||
    (description !== null &&
      (scalarLength(description) > 2000 || utf8Length(description) > 8192))
  ) {
    fields.description = "Keep the description to 2,000 characters or fewer.";
  }
  const recaptcha = value.recaptcha;
  if (recaptcha !== undefined && (typeof recaptcha !== "string" || recaptcha.length > 4096)) {
    fields.form = "The verification response is invalid. Reload the page and try again.";
  }
  if (Object.keys(fields).length > 0) return { ok: false, fields };

  return {
    ok: true,
    input: {
      manufacturer: manufacturer!,
      manufacturer_key: canonicalField(manufacturer!),
      part_number: partNumber!,
      part_number_key: canonicalField(partNumber!),
      datasheet_url: datasheetUrl!,
      description: description ?? null,
    },
    ...(typeof recaptcha === "string" ? { recaptcha } : {}),
  };
}

export async function readBoundedJson(
  request: Request,
  maximumBytes: number,
): Promise<ReadJsonResult> {
  const declared = request.headers.get("Content-Length");
  if (declared !== null) {
    const length = Number(declared);
    if (Number.isFinite(length) && length > maximumBytes) {
      return { ok: false, reason: "too_large" };
    }
  }
  if (!request.body) return { ok: false, reason: "invalid" };

  const reader = request.body.getReader();
  const chunks: Uint8Array[] = [];
  let total = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      total += value.byteLength;
      if (total > maximumBytes) {
        await reader.cancel();
        return { ok: false, reason: "too_large" };
      }
      chunks.push(value);
    }
  } catch {
    return { ok: false, reason: "invalid" };
  }

  const bytes = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  try {
    const source = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
    return { ok: true, value: JSON.parse(source) };
  } catch {
    return { ok: false, reason: "invalid" };
  }
}

export async function handlePublicComponentRequest(
  request: Request,
  url: URL,
  deps: PublicComponentRequestDependencies,
): Promise<Response> {
  if (request.method !== "POST") {
    return json(405, { error: "method not allowed" }, { Allow: "POST" });
  }
  if (!validWriteOrigin(request, url)) {
    return json(403, { error: "same-origin request required" });
  }
  if (!hasJsonContentType(request)) {
    return json(415, { error: "Content-Type application/json required" });
  }

  try {
    if (!(await deps.rateLimit(request))) {
      return json(
        429,
        { error: "too many requests; wait a minute and try again" },
        { "Retry-After": "60" },
      );
    }

    const decoded = await readBoundedJson(request, PUBLIC_BODY_LIMIT);
    if (!decoded.ok) {
      return decoded.reason === "too_large"
        ? json(413, { error: "request body is too large" })
        : json(400, { error: "request body must be valid JSON" });
    }
    const parsed = parseComponentRequestBody(decoded.value);
    if (!parsed.ok) {
      return json(400, { error: "check the highlighted fields", fields: parsed.fields });
    }

    const human = await deps.verifyHuman(request, parsed.recaptcha);
    if (human === "unavailable") {
      return json(503, { error: "component requests are temporarily unavailable" });
    }
    if (human !== "ok") {
      return json(403, { error: "reCAPTCHA verification failed; try submitting again" });
    }

    const result = await deps.store.submit(parsed.input, deps.now());
    safeLog(deps.log, {
      event: "component_request_submitted",
      duplicate: result.duplicate,
      cf_ray: request.headers.get("CF-Ray"),
    });
    return json(result.duplicate ? 200 : 202, {
      ok: true,
      duplicate: result.duplicate,
    });
  } catch (error) {
    safeLog(deps.log, {
      event: "component_request_error",
      error: error instanceof Error ? error.name : "UnknownError",
      cf_ray: request.headers.get("CF-Ray"),
    });
    return json(500, { error: "internal error" });
  }
}

function canonicalPositiveInteger(raw: string): number | null {
  if (!/^[1-9]\d*$/.test(raw)) return null;
  const value = Number(raw);
  return Number.isSafeInteger(value) ? value : null;
}

function adminRequestId(pathname: string): number | null | undefined {
  const match = pathname.match(/^\/api\/admin\/component-requests\/([^/]+)$/);
  if (!match) return undefined;
  return canonicalPositiveInteger(match[1]);
}

function statusFromBody(value: unknown): ComponentRequestStatus | null {
  if (!exactObject(value, new Set(["status"]))) return null;
  const status = value.status;
  return status === "open" || status === "resolved" ? status : null;
}

function listParameters(url: URL):
  | {
      ok: true;
      status: ComponentRequestStatus | "all";
      sort: ComponentRequestSort;
      query: string;
    }
  | { ok: false; error: string } {
  const rawStatus = url.searchParams.get("status") ?? "open";
  const status =
    rawStatus === "open" || rawStatus === "resolved" || rawStatus === "all"
      ? rawStatus
      : null;
  if (!status) return { ok: false, error: "status must be open, resolved, or all" };
  const rawSort = url.searchParams.get("sort") ?? "requested";
  const sort = rawSort === "requested" || rawSort === "newest" ? rawSort : null;
  if (!sort) return { ok: false, error: "sort must be requested or newest" };
  const displayQuery = (url.searchParams.get("q") ?? "").normalize("NFC").trim();
  if (utf8Length(displayQuery) > MAX_QUERY_BYTES || /\p{Cc}|\p{Cf}/u.test(displayQuery)) {
    return { ok: false, error: "search query is invalid or too long" };
  }
  return { ok: true, status, sort, query: canonicalField(displayQuery) };
}

export async function handleAdminComponentRequests(
  request: Request,
  url: URL,
  deps: AdminComponentRequestDependencies,
): Promise<Response> {
  let actor: Account | null = null;
  try {
    actor = await deps.sessionAccount(request);
    if (!actor) return json(401, { error: "not signed in" });
    if (actor.is_official !== 1) return json(403, { error: "official account required" });

    if (url.pathname === "/api/admin/component-requests") {
      if (request.method !== "GET") {
        return json(405, { error: "method not allowed" }, { Allow: "GET" });
      }
      const parameters = listParameters(url);
      if (!parameters.ok) return json(400, { error: parameters.error });
      const listed = await deps.store.list(
        parameters.status,
        parameters.sort,
        parameters.query,
      );
      return json(200, listed);
    }

    const id = adminRequestId(url.pathname);
    if (id === undefined) return json(404, { error: "not found" });
    if (id === null) return json(400, { error: "request id must be a positive integer" });
    if (request.method !== "PUT") {
      return json(405, { error: "method not allowed" }, { Allow: "PUT" });
    }
    if (!validWriteOrigin(request, url)) {
      return json(403, { error: "same-origin request required" });
    }
    if (!hasJsonContentType(request)) {
      return json(415, { error: "Content-Type application/json required" });
    }
    const decoded = await readBoundedJson(request, ADMIN_BODY_LIMIT);
    if (!decoded.ok) {
      return decoded.reason === "too_large"
        ? json(413, { error: "request body is too large" })
        : json(400, { error: "request body must be valid JSON" });
    }
    const status = statusFromBody(decoded.value);
    if (!status) return json(400, { error: "body must be exactly {\"status\":\"open|resolved\"}" });

    const updated = await deps.store.setStatus(id, status, actor.id, deps.now());
    if (!updated) return json(404, { error: "component request not found" });
    safeLog(deps.log, {
      event: "component_request_status_changed",
      request_id: id,
      previous_status: updated.changed ? (status === "resolved" ? "open" : "resolved") : status,
      status,
      changed: updated.changed,
      actor_account_id: actor.id,
      cf_ray: request.headers.get("CF-Ray"),
    });
    return json(200, updated);
  } catch (error) {
    safeLog(deps.log, {
      event: "admin_component_request_error",
      actor_account_id: actor?.id ?? null,
      error: error instanceof Error ? error.name : "UnknownError",
      cf_ray: request.headers.get("CF-Ray"),
    });
    return json(500, { error: "internal error" });
  }
}

function d1Store(db: D1Database): ComponentRequestStore {
  return {
    async submit(input: ComponentRequestInput, now: string): Promise<{ duplicate: boolean }> {
      const row = await db
        .prepare(
          `INSERT INTO component_requests
             (manufacturer, manufacturer_key, part_number, part_number_key,
              datasheet_url, description, status, request_count, created_at,
              last_requested_at, updated_at)
           VALUES (?, ?, ?, ?, ?, ?, 'open', 1, ?, ?, ?)
           ON CONFLICT(manufacturer_key, part_number_key) DO UPDATE SET
             request_count = component_requests.request_count + 1,
             last_requested_at = excluded.last_requested_at,
             updated_at = excluded.updated_at
           RETURNING request_count`,
        )
        .bind(
          input.manufacturer,
          input.manufacturer_key,
          input.part_number,
          input.part_number_key,
          input.datasheet_url,
          input.description,
          now,
          now,
          now,
        )
        .first<SubmitRow>();
      if (!row) throw new Error("component request upsert returned no row");
      return { duplicate: row.request_count > 1 };
    },

    async list(status, sort, query): Promise<ComponentRequestList> {
      const order =
        sort === "newest"
          ? "last_requested_at DESC, id DESC"
          : "request_count DESC, last_requested_at DESC, id DESC";
      const rows = await db
        .prepare(
          `SELECT id, manufacturer, part_number, datasheet_url, description, status,
                  request_count, created_at, last_requested_at, updated_at, resolved_at
             FROM component_requests
            WHERE (? = 'all' OR status = ?)
              AND (? = '' OR instr(manufacturer_key, ?) > 0
                          OR instr(part_number_key, ?) > 0)
            ORDER BY ${order}
            LIMIT 101`,
        )
        .bind(status, status, query, query, query)
        .all<D1ComponentRequestRow>();
      return {
        requests: rows.results.slice(0, 100),
        truncated: rows.results.length > 100,
      };
    },

    async setStatus(id, status, actorAccountId, now) {
      const updated = await db
        .prepare(
          `UPDATE component_requests
              SET status = ?,
                  updated_at = ?,
                  resolved_at = CASE WHEN ? = 'resolved' THEN ? ELSE NULL END,
                  resolved_by_account_id = CASE WHEN ? = 'resolved' THEN ? ELSE NULL END
            WHERE id = ? AND status <> ?
          RETURNING id, manufacturer, part_number, datasheet_url, description, status,
                    request_count, created_at, last_requested_at, updated_at, resolved_at`,
        )
        .bind(status, now, status, now, status, actorAccountId, id, status)
        .first<D1ComponentRequestRow>();
      if (updated) return { request: updated, changed: true };
      const existing = await db
        .prepare(
          `SELECT id, manufacturer, part_number, datasheet_url, description, status,
                  request_count, created_at, last_requested_at, updated_at, resolved_at
             FROM component_requests WHERE id = ?`,
        )
        .bind(id)
        .first<D1ComponentRequestRow>();
      return existing ? { request: existing, changed: false } : null;
    },
  };
}

function isLoopback(hostname: string): boolean {
  return (
    hostname === "localhost" ||
    hostname.endsWith(".localhost") ||
    hostname === "127.0.0.1" ||
    hostname === "[::1]" ||
    hostname === "::1"
  );
}

export async function componentRequestApi(
  env: Env,
  request: Request,
  url: URL,
): Promise<Response> {
  return handlePublicComponentRequest(request, url, {
    store: d1Store(env.DB),
    rateLimit: async (candidate) => {
      const key = candidate.headers.get("CF-Connecting-IP") ?? "unknown";
      const [source, route] = await Promise.all([
        env.COMPONENT_REQUEST_RATE_LIMITER.limit({ key }),
        env.COMPONENT_REQUEST_GLOBAL_RATE_LIMITER.limit({ key: "component-requests" }),
      ]);
      return source.success && route.success;
    },
    verifyHuman: async (candidate, token) => {
      if (!env.RECAPTCHA_SITE_KEY || !env.RECAPTCHA_SECRET_KEY) {
        return isLoopback(url.hostname) ? "ok" : "unavailable";
      }
      return (await recaptchaOk(env, candidate, token, "component_request", url.hostname))
        ? "ok"
        : "failed";
    },
    now: () => new Date().toISOString(),
    log: (entry) => console.log(JSON.stringify(entry)),
  });
}

export async function adminComponentRequestApi(
  env: Env,
  request: Request,
  url: URL,
): Promise<Response> {
  return handleAdminComponentRequests(request, url, {
    store: d1Store(env.DB),
    sessionAccount: (candidate) => accountForSession(env, candidate),
    now: () => new Date().toISOString(),
    log: (entry) => console.log(JSON.stringify(entry)),
  });
}
