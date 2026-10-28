import type { Env } from "./auth";
import { manufacturerKeyStore, resolveManufacturerKeys } from "./manufacturers";
import { normalizeMpn, type CatalogLink } from "./suppliers";

const JSON_HEADERS = { "Content-Type": "application/json", "Cache-Control": "public, max-age=60" };
const MAX_VALUES = 100;
const MAX_VALUE_BYTES = 256;
const QUERY_ROWS = 99; // supplier plus at most 99 lookup keys stays within D1's 100 bindings.

export interface PublicCatalogRow {
  pn: string;
  mpn: string;
  mpn_key: string;
  manufacturer: string;
  manufacturer_key: string;
  package: string;
  class: string;
  stock: number;
  links: CatalogLink[];
  fetched_at: string;
}
export interface SupplierPartsMetadata {
  class_vocabulary: string[];
  snapshot_date: string | null;
}
export interface SupplierPartsStore {
  metadata(supplier: string): Promise<SupplierPartsMetadata | null>;
  byPns(supplier: string, values: string[]): Promise<PublicCatalogRow[]>;
  byMpnKeys(supplier: string, values: string[]): Promise<PublicCatalogRow[]>;
}
export interface SupplierPartsDependencies { store: SupplierPartsStore }
type Query = { kind: "pn" | "mpn"; values: string[] };

function json(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS, ...headers } });
}
function supplierId(value: string): boolean { return /^[a-z][a-z0-9_]{0,31}$/.test(value); }
function decode(value: string): string | null { try { return decodeURIComponent(value); } catch { return null; } }
function validValue(value: string, kind: "pn" | "mpn"): boolean {
  return value.length > 0 && value.length <= MAX_VALUE_BYTES &&
    new TextEncoder().encode(value).length <= MAX_VALUE_BYTES &&
    !/[\p{Cc}\p{Cf}]/u.test(value) &&
    (kind === "mpn" || (value.normalize("NFC") === value && value.trim() === value));
}
function parseQuery(url: URL): { ok: true; value: Query } | { ok: false; error: string } {
  const keys = [...new Set(url.searchParams.keys())];
  if (keys.length !== 1 || (keys[0] !== "pn" && keys[0] !== "mpn") ||
      url.searchParams.getAll(keys[0]).length !== 1) {
    return { ok: false, error: "provide exactly one pn or mpn query parameter" };
  }
  const kind = keys[0];
  const raw = url.searchParams.get(kind) ?? "";
  if (raw.length > 100 * (MAX_VALUE_BYTES + 1)) return { ok: false, error: "too many query values" };
  const values = raw.split(",");
  if (values.length > MAX_VALUES) return { ok: false, error: "at most 100 values are allowed" };
  if (values.some((value) => !validValue(value, kind))) {
    return { ok: false, error: "values must be non-empty, bounded, and free of control characters" };
  }
  const normalized = kind === "mpn" ? values.map(normalizeMpn) : values;
  if (normalized.some((value) => value.length === 0) || new Set(normalized).size !== values.length) {
    return { ok: false, error: "values must be non-empty and unique" };
  }
  return { ok: true, value: { kind, values } };
}
function publicRow(row: D1Row): PublicCatalogRow {
  return {
    pn: String(row.pn), mpn: String(row.mpn), mpn_key: normalizeMpn(String(row.mpn)),
    manufacturer: String(row.manufacturer), manufacturer_key: "",
    package: String(row.package), class: String(row.class), stock: Number(row.stock),
    links: JSON.parse(String(row.links)) as CatalogLink[], fetched_at: String(row.fetched_at),
  };
}
type D1Row = Record<string, unknown>;

export async function handleSupplierParts(
  request: Request, url: URL, deps: SupplierPartsDependencies,
): Promise<Response> {
  const match = url.pathname.match(/^\/suppliers\/([^/]+)\/parts$/);
  const supplier = match ? decode(match[1]) : null;
  if (supplier === null || !supplierId(supplier)) return json(404, { error: "not found" });
  if (request.method !== "GET") return json(405, { error: "method not allowed" }, { Allow: "GET" });
  const query = parseQuery(url);
  if (!query.ok) return json(400, { error: query.error });
  try {
    const metadata = await deps.store.metadata(supplier);
    if (!metadata) return json(404, { error: "supplier not found" });
    if (query.value.kind === "pn") {
      const rows = await deps.store.byPns(supplier, query.value.values);
      const byPn = new Map(rows.map((row) => [row.pn, row]));
      return json(200, { supplier, snapshot_date: metadata.snapshot_date,
        parts: query.value.values.map((pn) => ({ pn, catalog: byPn.get(pn) ?? "unknown" })) });
    }
    const keys = query.value.values.map(normalizeMpn);
    const rows = await deps.store.byMpnKeys(supplier, keys);
    const classOrder = new Map(metadata.class_vocabulary.map((name, index) => [name, index]));
    const byMpn = new Map<string, PublicCatalogRow[]>(keys.map((key) => [key, []]));
    for (const row of rows) byMpn.get(normalizeMpn(row.mpn))?.push(row);
    const parts = query.value.values.map((mpn, index) => ({
      mpn,
      catalog: (byMpn.get(keys[index]) ?? []).sort((a, b) =>
        (classOrder.get(a.class) ?? Number.MAX_SAFE_INTEGER) - (classOrder.get(b.class) ?? Number.MAX_SAFE_INTEGER) ||
        Number(b.stock > 0) - Number(a.stock > 0) || b.stock - a.stock || a.pn.localeCompare(b.pn)),
    }));
    return json(200, { supplier, snapshot_date: metadata.snapshot_date, parts });
  } catch {
    return json(500, { error: "internal error" });
  }
}

const CATALOG_COLUMNS = "pn, mpn, manufacturer, package, class, stock, links, fetched_at";
async function lookup(db: D1Database, column: "pn" | "mpn_key", supplier: string, values: string[]): Promise<PublicCatalogRow[]> {
  const rows: PublicCatalogRow[] = [];
  for (let offset = 0; offset < values.length; offset += QUERY_ROWS) {
    const chunk = values.slice(offset, offset + QUERY_ROWS);
    const result = await db.prepare(`SELECT ${CATALOG_COLUMNS} FROM supplier_catalog
      WHERE supplier = ? AND ${column} IN (${chunk.map(() => "?").join(", ")})`)
      .bind(supplier, ...chunk).all();
    for (const row of result.results) rows.push(publicRow(row));
  }
  const manufacturerKeys = await resolveManufacturerKeys(
    rows.map((row) => row.manufacturer), manufacturerKeyStore(db),
  );
  return rows.map((row, index) => ({ ...row, manufacturer_key: manufacturerKeys[index] }));
}
export function supplierPartsStore(db: D1Database): SupplierPartsStore {
  return {
    async metadata(supplier) {
      const row = await db.prepare(`SELECT s.class_vocabulary, MAX(c.fetched_at) AS snapshot_date
        FROM suppliers s LEFT JOIN supplier_catalog c ON c.supplier = s.id
        WHERE s.id = ? GROUP BY s.id`).bind(supplier).first();
      if (!row) return null;
      return { class_vocabulary: JSON.parse(String(row.class_vocabulary)) as string[],
        snapshot_date: row.snapshot_date === null ? null : String(row.snapshot_date) };
    },
    byPns: (supplier, values) => lookup(db, "pn", supplier, values),
    byMpnKeys: (supplier, values) => lookup(db, "mpn_key", supplier, values),
  };
}
export async function supplierPartsApi(env: Env, request: Request, url: URL): Promise<Response> {
  return handleSupplierParts(request, url, { store: supplierPartsStore(env.DB) });
}
