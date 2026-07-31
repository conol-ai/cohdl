// Official-account administration for registry.cohdl.org.
//
// These endpoints deliberately accept web sessions only. CLI bearer tokens
// authorize publishing, but never inherit account-administration authority.

import { accountForSession, type Account, type Env } from "./auth";
import { nameTier } from "./namespace";

const JSON_HEADERS = {
  "Content-Type": "application/json",
  "Cache-Control": "no-store",
};

export interface AdminBrand {
  brand: string;
  verified: boolean;
}

export interface AdminAccount {
  id: number;
  email: string;
  official: boolean;
  created_at: string;
  brands: AdminBrand[];
}

export interface AccountList {
  accounts: AdminAccount[];
  truncated: boolean;
}

export type BrandMutation =
  | { kind: "ok"; changed: boolean }
  | { kind: "conflict"; ownerAccountId: number };

export interface AdminStore {
  listAccounts(query: string): Promise<AccountList>;
  accountExists(accountId: number): Promise<boolean>;
  grantBrand(accountId: number, brand: string): Promise<BrandMutation>;
  revokeBrand(accountId: number, brand: string): Promise<BrandMutation>;
}

export interface AdminDependencies {
  sessionAccount(request: Request): Promise<Account | null>;
  store: AdminStore;
  log(entry: Record<string, unknown>): void;
  now(): string;
}

type BrandPath =
  | { matched: false }
  | { matched: true; accountId: number | null };

interface AccountBrandRow {
  id: number;
  email: string;
  is_official: number;
  created_at: string;
  brand: string | null;
  verified: number | null;
}

interface BrandClaimRow {
  account_id: number;
  verified: number;
}

function adminJson(
  status: number,
  body: unknown,
  headers: Record<string, string> = {},
): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...JSON_HEADERS, ...headers },
  });
}

/// Parse only the canonical positive-decimal account IDs accepted in an
/// administration URL: no zero, sign, leading zeroes, fraction, or unsafe
/// JavaScript integer.
export function canonicalAccountId(raw: string): number | null {
  if (!/^[1-9]\d*$/.test(raw)) return null;
  const id = Number(raw);
  return Number.isSafeInteger(id) ? id : null;
}

export function parseBrandPath(pathname: string): BrandPath {
  const match = pathname.match(/^\/api\/admin\/accounts\/([^/]+)\/brands$/);
  if (!match) return { matched: false };
  return { matched: true, accountId: canonicalAccountId(match[1]) };
}

/// Keep brand grants byte-for-byte compatible with the namespace classifier.
/// `contrib` is the shared community tier, not a manufacturer claim.
export function validBrand(value: unknown): value is string {
  if (typeof value !== "string" || value === "" || value.trim() !== value) return false;
  const tier = nameTier(`@${value}/_`);
  return !("error" in tier) && tier.tier === "brand" && tier.brand === value;
}

export function brandFromBody(value: unknown): string | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  const keys = Object.keys(value);
  if (keys.length !== 1 || keys[0] !== "brand") return null;
  const brand = Reflect.get(value, "brand");
  return validBrand(brand) ? brand : null;
}

export function validWriteOrigin(request: Request, url: URL): boolean {
  return request.headers.get("Origin") === url.origin;
}

export function hasJsonContentType(request: Request): boolean {
  const contentType = request.headers.get("Content-Type") ?? "";
  return contentType.split(";", 1)[0].trim().toLowerCase() === "application/json";
}

function emitLog(deps: AdminDependencies, entry: Record<string, unknown>): void {
  // Logging must never turn a completed privilege mutation into a 500.
  try {
    deps.log(entry);
  } catch {
    // Deliberately ignored.
  }
}

function mutationLog(
  deps: AdminDependencies,
  request: Request,
  actor: Account,
  action: "grant" | "revoke",
  targetAccountId: number,
  brand: string,
  outcome: string,
  changed?: boolean,
): void {
  emitLog(deps, {
    event: "admin_brand_mutation",
    action,
    actor_account_id: actor.id,
    target_account_id: targetAccountId,
    brand,
    outcome,
    ...(changed === undefined ? {} : { changed }),
    at: deps.now(),
    cf_ray: request.headers.get("CF-Ray"),
  });
}

export async function handleAdminRequest(
  request: Request,
  url: URL,
  deps: AdminDependencies,
): Promise<Response> {
  let actor: Account | null = null;
  try {
    actor = await deps.sessionAccount(request);
    if (!actor) return adminJson(401, { error: "not signed in" });
    if (actor.is_official !== 1) {
      return adminJson(403, { error: "official account required" });
    }

    if (url.pathname === "/api/admin/accounts") {
      if (request.method !== "GET") {
        return adminJson(405, { error: "method not allowed" }, { Allow: "GET" });
      }
      const query = (url.searchParams.get("q") ?? "").trim();
      const listed = await deps.store.listAccounts(query);
      const accounts = listed.accounts.slice(0, 100).map((account) => ({
        id: account.id,
        email: account.email,
        official: account.official,
        created_at: account.created_at,
        brands: account.brands.map(({ brand, verified }) => ({ brand, verified })),
      }));
      return adminJson(200, {
        accounts,
        truncated: listed.truncated || listed.accounts.length > 100,
      });
    }

    const parsed = parseBrandPath(url.pathname);
    if (!parsed.matched) return adminJson(404, { error: "not found" });
    if (request.method !== "PUT" && request.method !== "DELETE") {
      return adminJson(405, { error: "method not allowed" }, { Allow: "PUT, DELETE" });
    }
    if (parsed.accountId === null) {
      return adminJson(400, { error: "account id must be a canonical positive integer" });
    }
    if (!validWriteOrigin(request, url)) {
      return adminJson(403, { error: "same-origin request required" });
    }
    if (!hasJsonContentType(request)) {
      return adminJson(415, { error: "Content-Type application/json required" });
    }

    let body: unknown;
    try {
      body = await request.json();
    } catch {
      return adminJson(400, { error: "request body must be valid JSON" });
    }
    const brand = brandFromBody(body);
    if (!brand) {
      return adminJson(400, {
        error: "body must be exactly {\"brand\":\"<manufacturer>\"}; contrib is reserved",
      });
    }

    const accountId = parsed.accountId;
    const action = request.method === "PUT" ? "grant" : "revoke";
    if (!(await deps.store.accountExists(accountId))) {
      mutationLog(deps, request, actor, action, accountId, brand, "target_not_found");
      return adminJson(404, { error: "account not found" });
    }

    const result =
      action === "grant"
        ? await deps.store.grantBrand(accountId, brand)
        : await deps.store.revokeBrand(accountId, brand);
    if (result.kind === "conflict") {
      mutationLog(deps, request, actor, action, accountId, brand, "ownership_conflict");
      return adminJson(409, {
        error: `brand \`${brand}\` is assigned to another account`,
        owner_account_id: result.ownerAccountId,
      });
    }

    mutationLog(deps, request, actor, action, accountId, brand, "success", result.changed);
    return adminJson(200, {
      account_id: accountId,
      brand,
      verified: action === "grant",
      changed: result.changed,
    });
  } catch (error) {
    emitLog(deps, {
      event: "admin_request_error",
      actor_account_id: actor?.id ?? null,
      method: request.method,
      path: url.pathname,
      error: error instanceof Error ? error.name : "UnknownError",
      at: deps.now(),
      cf_ray: request.headers.get("CF-Ray"),
    });
    return adminJson(500, { error: "internal error" });
  }
}

function d1Store(db: D1Database): AdminStore {
  const claim = async (brand: string): Promise<BrandClaimRow | null> =>
    db
      .prepare("SELECT account_id, verified FROM brands WHERE brand = ?")
      .bind(brand)
      .first<BrandClaimRow>();

  return {
    async listAccounts(query: string): Promise<AccountList> {
      const rows = await db
        .prepare(
          `SELECT a.id, a.email, a.is_official, a.created_at, b.brand, b.verified
             FROM (
               SELECT id, email, is_official, created_at
                 FROM accounts
                WHERE ? = '' OR instr(lower(email), lower(?)) > 0
                ORDER BY id ASC
                LIMIT 101
             ) a
             LEFT JOIN brands b ON b.account_id = a.id
            ORDER BY a.id ASC, b.brand ASC`,
        )
        .bind(query, query)
        .all<AccountBrandRow>();

      const accounts: AdminAccount[] = [];
      let current: AdminAccount | null = null;
      for (const row of rows.results) {
        if (!current || current.id !== row.id) {
          current = {
            id: row.id,
            email: row.email,
            official: row.is_official === 1,
            created_at: row.created_at,
            brands: [],
          };
          accounts.push(current);
        }
        if (row.brand !== null) {
          current.brands.push({ brand: row.brand, verified: row.verified === 1 });
        }
      }
      return { accounts: accounts.slice(0, 100), truncated: accounts.length > 100 };
    },

    async accountExists(accountId: number): Promise<boolean> {
      return (
        (await db
          .prepare("SELECT 1 AS found FROM accounts WHERE id = ?")
          .bind(accountId)
          .first<{ found: number }>()) !== null
      );
    },

    async grantBrand(accountId: number, brand: string): Promise<BrandMutation> {
      let current = await claim(brand);
      if (!current) {
        const inserted = await db
          .prepare(
            "INSERT OR IGNORE INTO brands (brand, account_id, verified) VALUES (?, ?, 1)",
          )
          .bind(brand, accountId)
          .run();
        if (inserted.meta.changes > 0) return { kind: "ok", changed: true };
        // Another request claimed the primary-key row between our read and
        // insert. Re-read it and apply the same ownership rule.
        current = await claim(brand);
        if (!current) throw new Error("brand claim disappeared");
      }
      if (current.account_id !== accountId) {
        return { kind: "conflict", ownerAccountId: current.account_id };
      }
      if (current.verified === 1) return { kind: "ok", changed: false };
      const updated = await db
        .prepare(
          "UPDATE brands SET verified = 1 WHERE brand = ? AND account_id = ? AND verified = 0",
        )
        .bind(brand, accountId)
        .run();
      return { kind: "ok", changed: updated.meta.changes > 0 };
    },

    async revokeBrand(accountId: number, brand: string): Promise<BrandMutation> {
      const current = await claim(brand);
      if (!current) return { kind: "ok", changed: false };
      if (current.account_id !== accountId) {
        return { kind: "conflict", ownerAccountId: current.account_id };
      }
      if (current.verified !== 1) return { kind: "ok", changed: false };
      const updated = await db
        .prepare(
          "UPDATE brands SET verified = 0 WHERE brand = ? AND account_id = ? AND verified = 1",
        )
        .bind(brand, accountId)
        .run();
      return { kind: "ok", changed: updated.meta.changes > 0 };
    },
  };
}

export async function adminApi(env: Env, request: Request, url: URL): Promise<Response> {
  return handleAdminRequest(request, url, {
    sessionAccount: (candidate) => accountForSession(env, candidate),
    store: d1Store(env.DB),
    log: (entry) => console.log(JSON.stringify(entry)),
    now: () => new Date().toISOString(),
  });
}
