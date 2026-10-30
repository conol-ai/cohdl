// Official-account maintenance for supplier catalog snapshots and aliases.
import { accountForSession, type Account, type Env } from "./auth";
import { hasJsonContentType, validWriteOrigin } from "./admin";
import { readBoundedJson } from "./component-requests";

const JSON_HEADERS = { "Content-Type": "application/json", "Cache-Control": "no-store" };
export const SUPPLIER_BODY_LIMIT = 4 * 1024 * 1024;
export const MAX_CATALOG_ROWS = 10_000;
// D1 permits 100 bound parameters per statement; catalog rows need 11 each.
const CATALOG_INSERT_ROWS = Math.floor(100 / 11);

export interface CatalogLink { label: string; url: string }
export interface CatalogRow {
  pn: string; mpn: string; mpn_key: string; manufacturer_key: string;
  manufacturer: string; package: string; class: string;
  stock: number; links: CatalogLink[]; fetched_at: string;
}
export interface Snapshot {
  supplier: string; display_name: string; product_url_template: string;
  class_vocabulary: string[]; rows: CatalogRow[];
}
export interface SupplierAdminStore {
  replaceSnapshot(snapshot: Snapshot): Promise<void>;
  putAlias(aliasKey: string, alias: string, canonical: string): Promise<void>;
  deleteAlias(aliasKey: string): Promise<boolean>;
}
export interface SupplierAdminDependencies {
  sessionAccount(request: Request): Promise<Account | null>;
  store: SupplierAdminStore;
  log(entry: Record<string, unknown>): void;
  now(): string;
}
type Parsed<T> = { ok: true; value: T } | { ok: false; error: string };

function json(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS, ...headers } });
}
function object(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
function exact(value: unknown, keys: string[]): value is Record<string, unknown> {
  return object(value) && Object.keys(value).length === keys.length &&
    keys.every((key) => Object.hasOwn(value, key));
}
function text(value: unknown, maxBytes: number): value is string {
  return typeof value === "string" && value.normalize("NFC") === value &&
    value.trim() === value && value.length > 0 && !/\p{Cc}|\p{Cf}/u.test(value) &&
    new TextEncoder().encode(value).length <= maxBytes;
}
export function normalizeManufacturer(value: string): string {
  return value.normalize("NFC").toLocaleLowerCase("und").replace(/\s+/gu, " ").trim();
}
export function normalizeMpn(value: string): string {
  return value.normalize("NFC").toLocaleLowerCase("und").replace(/\s+/gu, "");
}
function supplierId(value: string): boolean { return /^[a-z][a-z0-9_]{0,31}$/.test(value); }
function httpUrl(value: unknown, template = false): value is string {
  if (typeof value !== "string" || value.length > 2048 || /\s/u.test(value)) return false;
  if (template && (value.match(/\{pn\}/g)?.length ?? 0) !== 1) return false;
  let url: URL;
  try { url = new URL(template ? value.replaceAll("{pn}", "PN") : value); } catch { return false; }
  return (url.protocol === "https:" || url.protocol === "http:") && !url.username && !url.password &&
    new TextEncoder().encode(value).length <= 2048;
}
function timestamp(value: unknown): value is string {
  if (typeof value !== "string" || !/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d{3})?Z$/.test(value)) return false;
  const parsed = new Date(value);
  return !Number.isNaN(parsed.getTime()) && (value.length === 20
    ? parsed.toISOString() === `${value.slice(0, -1)}.000Z` : parsed.toISOString() === value);
}
function vocabulary(value: unknown): value is string[] {
  return Array.isArray(value) && value.length > 0 && value.length <= 256 &&
    value.every((item) => text(item, 128)) && new Set(value).size === value.length;
}
function links(value: unknown): Parsed<CatalogLink[]> {
  if (!Array.isArray(value) || value.length > 16) return { ok: false, error: "links must contain at most 16 entries" };
  const labels = new Set<string>();
  const result: CatalogLink[] = [];
  for (const item of value) {
    if (!exact(item, ["label", "url"]) || !text(item.label, 128) || labels.has(item.label) || !httpUrl(item.url)) {
      return { ok: false, error: "each link needs a unique label and a bounded HTTP(S) URL without credentials" };
    }
    labels.add(item.label); result.push({ label: item.label, url: item.url });
  }
  return { ok: true, value: result };
}

export function parseSnapshot(value: unknown, supplier: string): Parsed<Snapshot> {
  if (!supplierId(supplier)) return { ok: false, error: "invalid supplier ID" };
  if (!exact(value, ["display_name", "product_url_template", "classes", "parts"])) {
    return { ok: false, error: "snapshot must contain exactly display_name, product_url_template, classes, and parts" };
  }
  if (!text(value.display_name, 256) || !httpUrl(value.product_url_template, true) || !vocabulary(value.classes)) {
    return { ok: false, error: "invalid supplier display name, product URL template, or class vocabulary" };
  }
  if (!Array.isArray(value.parts) || value.parts.length > MAX_CATALOG_ROWS) {
    return { ok: false, error: `parts must contain at most ${MAX_CATALOG_ROWS} rows` };
  }
  const pns = new Set<string>();
  const rows: CatalogRow[] = [];
  const fields = ["pn", "mpn", "manufacturer", "package", "class", "stock", "links", "fetched_at"];
  for (let index = 0; index < value.parts.length; index++) {
    const item = value.parts[index];
    if (!exact(item, fields) || !text(item.pn, 256) || !text(item.mpn, 256) ||
        !text(item.manufacturer, 256) || !text(item.package, 256) ||
        !text(item.class, 128) || !value.classes.includes(item.class)) {
      return { ok: false, error: `parts[${index}] has invalid or unknown fields` };
    }
    if (pns.has(item.pn)) return { ok: false, error: `parts[${index}] duplicates PN ${item.pn}` };
    if (!Number.isSafeInteger(item.stock) || (item.stock as number) < 0) {
      return { ok: false, error: `parts[${index}] stock must be a non-negative safe integer` };
    }
    if (!timestamp(item.fetched_at)) return { ok: false, error: `parts[${index}] has an invalid UTC timestamp` };
    const parsedLinks = links(item.links);
    if (!parsedLinks.ok) return { ok: false, error: `parts[${index}]: ${parsedLinks.error}` };
    pns.add(item.pn);
    rows.push({ pn: item.pn, mpn: item.mpn, mpn_key: normalizeMpn(item.mpn),
      manufacturer_key: normalizeManufacturer(item.manufacturer), manufacturer: item.manufacturer,
      package: item.package, class: item.class, stock: item.stock as number,
      links: parsedLinks.value, fetched_at: item.fetched_at });
  }
  return { ok: true, value: { supplier, display_name: value.display_name,
    product_url_template: value.product_url_template, class_vocabulary: value.classes, rows } };
}
function decode(value: string): string | null { try { return decodeURIComponent(value); } catch { return null; } }
function route(pathname: string): { supplier: string } | { alias: string } | null {
  let match = pathname.match(/^\/api\/admin\/suppliers\/([^/]+)\/snapshot$/);
  if (match) { const supplier = decode(match[1]); return supplier === null ? null : { supplier }; }
  match = pathname.match(/^\/api\/admin\/manufacturer-aliases\/([^/]+)$/);
  if (match) { const alias = decode(match[1]); return alias === null ? null : { alias }; }
  return null;
}

export async function handleSupplierAdmin(
  request: Request, url: URL, deps: SupplierAdminDependencies,
): Promise<Response> {
  let actor: Account | null = null;
  try {
    actor = await deps.sessionAccount(request);
    if (!actor) return json(401, { error: "not signed in" });
    if (actor.is_official !== 1) return json(403, { error: "official account required" });
    const path = route(url.pathname);
    if (!path) return json(404, { error: "not found" });
    if ("supplier" in path && request.method !== "PUT") return json(405, { error: "method not allowed" }, { Allow: "PUT" });
    if ("alias" in path && request.method !== "PUT" && request.method !== "DELETE") {
      return json(405, { error: "method not allowed" }, { Allow: "PUT, DELETE" });
    }
    if (!validWriteOrigin(request, url)) return json(403, { error: "same-origin request required" });
    if (!hasJsonContentType(request)) return json(415, { error: "Content-Type application/json required" });
    const decoded = await readBoundedJson(request, SUPPLIER_BODY_LIMIT);
    if (!decoded.ok) return decoded.reason === "too_large"
      ? json(413, { error: "request body is too large" }) : json(400, { error: "request body must be valid JSON" });
    if ("supplier" in path) {
      const parsed = parseSnapshot(decoded.value, path.supplier);
      if (!parsed.ok) return json(400, { error: parsed.error });
      await deps.store.replaceSnapshot(parsed.value);
      deps.log({ event: "supplier_snapshot_replaced", supplier: path.supplier, rows: parsed.value.rows.length, actor_account_id: actor.id, at: deps.now() });
      return json(200, { ok: true, supplier: path.supplier, rows: parsed.value.rows.length });
    }
    if (request.method === "PUT") {
      if (!exact(decoded.value, ["canonical"]) || !text(decoded.value.canonical, 256) || !text(path.alias, 256)) {
        return json(400, { error: "body must be exactly {canonical: <manufacturer>}" });
      }
      const aliasKey = normalizeManufacturer(path.alias);
      if (!aliasKey) return json(400, { error: "alias is empty after normalization" });
      await deps.store.putAlias(aliasKey, path.alias, decoded.value.canonical);
      deps.log({ event: "manufacturer_alias_put", alias_key: aliasKey, actor_account_id: actor.id, at: deps.now() });
      return json(200, { ok: true, alias_key: aliasKey });
    }
    if (!exact(decoded.value, [])) return json(400, { error: "body must be exactly {}" });
    const deleted = await deps.store.deleteAlias(normalizeManufacturer(path.alias));
    if (!deleted) return json(404, { error: "manufacturer alias not found" });
    deps.log({ event: "manufacturer_alias_deleted", alias_key: normalizeManufacturer(path.alias), actor_account_id: actor.id, at: deps.now() });
    return json(200, { ok: true, deleted: true });
  } catch (error) {
    try { deps.log({ event: "supplier_admin_error", actor_account_id: actor?.id ?? null, error: error instanceof Error ? error.name : "UnknownError", at: deps.now() }); } catch { /* ignored */ }
    return json(500, { error: "internal error" });
  }
}

export function supplierStore(db: D1Database): SupplierAdminStore {
  return {
    async replaceSnapshot(snapshot) {
      const statements: D1PreparedStatement[] = [
        db.prepare(`INSERT INTO suppliers (id, display_name, product_url_template, class_vocabulary) VALUES (?, ?, ?, ?)
          ON CONFLICT(id) DO UPDATE SET display_name = excluded.display_name,
          product_url_template = excluded.product_url_template, class_vocabulary = excluded.class_vocabulary`)
          .bind(snapshot.supplier, snapshot.display_name, snapshot.product_url_template, JSON.stringify(snapshot.class_vocabulary)),
        db.prepare("DELETE FROM supplier_catalog WHERE supplier = ?").bind(snapshot.supplier),
      ];
      const values = "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";
      for (let offset = 0; offset < snapshot.rows.length; offset += CATALOG_INSERT_ROWS) {
        const rows = snapshot.rows.slice(offset, offset + CATALOG_INSERT_ROWS);
        const bindings: (string | number)[] = [];
        for (const row of rows) bindings.push(snapshot.supplier, row.pn, row.mpn, row.mpn_key,
          row.manufacturer_key, row.manufacturer, row.package, row.class, row.stock, JSON.stringify(row.links), row.fetched_at);
        statements.push(db.prepare(`INSERT INTO supplier_catalog
          (supplier, pn, mpn, mpn_key, manufacturer_key, manufacturer, package, class, stock, links, fetched_at)
          VALUES ${rows.map(() => values).join(", ")}`).bind(...bindings));
      }
      await db.batch(statements);
    },
    async putAlias(aliasKey, alias, canonical) {
      await db.prepare(`INSERT INTO manufacturer_aliases (alias_key, alias, canonical) VALUES (?, ?, ?)
        ON CONFLICT(alias_key) DO UPDATE SET alias = excluded.alias, canonical = excluded.canonical`)
        .bind(aliasKey, alias, canonical).run();
    },
    async deleteAlias(aliasKey) {
      const result = await db.prepare("DELETE FROM manufacturer_aliases WHERE alias_key = ?").bind(aliasKey).run();
      return (result.meta.changes ?? 0) > 0;
    },
  };
}
export async function supplierAdminApi(env: Env, request: Request, url: URL): Promise<Response> {
  return handleSupplierAdmin(request, url, {
    sessionAccount: (req) => accountForSession(env, req), store: supplierStore(env.DB),
    log: (entry) => console.log(JSON.stringify(entry)), now: () => new Date().toISOString(),
  });
}
