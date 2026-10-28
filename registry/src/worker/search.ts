// Stable registry search contract for `cohdl search`.
//
// Unlike `/api/search` (the web catalogue's implementation-private package
// search), this endpoint is a CLI contract. Package matches come from D1's
// ordinary package/version rows; part matches come from a bounded FTS5
// trigram index derived from the latest release's public API-doc `items`.

import type { Env } from "./auth";

const JSON_HEADERS = {
  "Content-Type": "application/json",
  "Cache-Control": "no-store",
};

const DEFAULT_LIMIT = 20;
const MAX_LIMIT = 50;
const MAX_OFFSET = 10_000;
const MAX_QUERY_BYTES = 128;
const MIN_QUERY_SCALARS = 3;

const MAX_ID_BYTES = 512;
const MAX_INTENT_BYTES = 2_048;
const MAX_PACKAGE_DESCRIPTION_BYTES = 4_096;
const MAX_SEARCHABLE_BYTES = 256 * 1024;
const MAX_AVL_JSON_BYTES = 1024 * 1024;
const MAX_INDEXED_PARTS = 50_000;
const MAX_SCANNED_ITEMS = 100_000;
const MAX_ARGS_PER_PART = 128;
const MAX_AVL_ENTRIES_PER_PART = 256;
const MAX_FIELDS_PER_AVL = 64;
export const MAX_SUPPLIERS_PER_AVL = 16;
const MAX_SUPPLIER_PN_BYTES = 128;
// match + supplier + 96 classes + limit + offset = D1's 100 bindings.
const MAX_SEARCH_CLASSES = 96;
const MAX_CLASS_BYTES = 128;
// Each key binds supplier and PN; 49 pairs stay below D1's 100-binding limit.
const CATALOG_LOOKUP_ROWS = 49;
const MAX_LINK_BYTES = 2048;
export const PART_SEARCH_CHUNK_BYTES = 512 * 1024;
// Four MiB retains the current largest real package (`passive`, 8,972 public
// parts) while preventing an accepted sidecar from multiplying into
// tens of MiB of derived JS objects before chunking. Eight half-MiB inserts
// also leave ample room beneath D1 Free's 50-query invocation ceiling.
export const MAX_PART_SEARCH_INDEX_BYTES = 4 * 1024 * 1024;
export const MAX_PART_SEARCH_CHUNKS = 8;

export type SearchKind = "all" | "package" | "part";

export interface PackageSearchRow {
  name: string;
  tier: "official" | "brand" | "contrib";
  latest: string;
  description: string | null;
  updated: string;
}

export interface IndexedPartSearchRow {
  package: string;
  tier: "official" | "brand" | "contrib";
  version: string;
  fq: string;
  name: string;
  device: string;
  intent: string | null;
  avl_json: string;
}

export interface PartSearchIndexRow {
  package_name: string;
  package_version: string;
  fq: string;
  name: string;
  device: string;
  intent: string | null;
  searchable: string;
  avl_json: string;
}

export interface SearchStore {
  searchPackages(query: string, take: number, offset: number): Promise<PackageSearchRow[]>;
  searchParts(match: string, take: number, offset: number, supplier: string | null, classes: string[]): Promise<IndexedPartSearchRow[]>;
  catalog(keys: readonly IndexedSupplier[]): Promise<Map<string, SupplierCatalogRow>>;
}

export interface SearchDependencies {
  store: SearchStore;
  log(entry: Record<string, unknown>): void;
}

interface SearchParams {
  query: string;
  kind: SearchKind;
  limit: number;
  offset: number;
  supplier: string | null;
  classes: string[];
}

type SearchParamsResult =
  | { ok: true; value: SearchParams }
  | { ok: false; error: string };

interface IndexedAvl {
  primary: boolean;
  manufacturer: string | null;
  mpn: string | null;
  suppliers: IndexedSupplier[];
}

interface IndexedSupplier {
  supplier: string;
  pn: string;
  at: [string, string];
  rotate: number;
}

interface PublicCatalog {
  class: string;
  stock: number;
  snapshot_date: string;
}

interface PublicSupplier extends IndexedSupplier {
  links: { label: string; url: string }[];
  catalog: PublicCatalog | null;
}

export interface PublicPartSearchRow {
  package: string;
  tier: "official" | "brand" | "contrib";
  version: string;
  fq: string;
  name: string;
  device: string;
  intent: string | null;
  manufacturer: string | null;
  mpn: string | null;
  primary: boolean;
  suppliers: PublicSupplier[];
}

interface SupplierCatalogRow extends PublicCatalog {
  supplier: string;
  pn: string;
  product_url_template: string | null;
  links: { label: string; url: string }[];
}

function searchJson(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...JSON_HEADERS, ...headers },
  });
}

function canonicalInteger(
  raw: string | null,
  fallback: number,
  minimum: number,
  maximum: number,
): number | null {
  if (raw === null) return fallback;
  if (!/^(0|[1-9]\d*)$/.test(raw)) return null;
  const value = Number(raw);
  return Number.isSafeInteger(value) && value >= minimum && value <= maximum ? value : null;
}

export function parseSearchParams(url: URL): SearchParamsResult {
  // Rust's `char::is_whitespace` follows Unicode White_Space, while browser
  // `String.trim()` has a slightly different legacy set. Spell out the
  // shared contract and include BOM at the edges so the echoed query is
  // byte-for-byte identical to what the CLI validated and sent.
  const query = (url.searchParams.get("q") ?? "").replace(
    /^[\p{White_Space}\uFEFF]+|[\p{White_Space}\uFEFF]+$/gu,
    "",
  );
  if ([...query].length < MIN_QUERY_SCALARS) {
    return { ok: false, error: "q must contain at least 3 characters" };
  }
  if (/\p{Cc}/u.test(query)) {
    return { ok: false, error: "q must not contain control characters" };
  }
  if (new TextEncoder().encode(query).length > MAX_QUERY_BYTES) {
    return { ok: false, error: "q must be at most 128 UTF-8 bytes" };
  }

  const requestedKind = url.searchParams.get("kind") ?? "all";
  if (requestedKind !== "all" && requestedKind !== "package" && requestedKind !== "part") {
    return { ok: false, error: "kind must be one of all, package, or part" };
  }

  const limit = canonicalInteger(url.searchParams.get("limit"), DEFAULT_LIMIT, 1, MAX_LIMIT);
  if (limit === null) {
    return { ok: false, error: "limit must be a canonical integer from 1 to 50" };
  }
  const offset = canonicalInteger(url.searchParams.get("offset"), 0, 0, MAX_OFFSET);
  if (offset === null) {
    return { ok: false, error: "offset must be a canonical integer from 0 to 10000" };
  }

  const suppliers = url.searchParams.getAll("supplier");
  if (suppliers.length > 1 || (suppliers.length === 1 && !/^[a-z][a-z0-9_]{0,31}$/.test(suppliers[0]))) {
    return { ok: false, error: "supplier must be one valid supplier ID" };
  }
  const classes = url.searchParams.getAll("class");
  if (classes.length > 0 && suppliers.length === 0) {
    return { ok: false, error: "class requires supplier" };
  }
  if (classes.length > MAX_SEARCH_CLASSES || classes.some((value) =>
    value.length === 0 || value.normalize("NFC") !== value || value.trim() !== value ||
    /[\p{Cc}\p{Cf}]/u.test(value) || new TextEncoder().encode(value).length > MAX_CLASS_BYTES)) {
    return { ok: false, error: "at most 96 bounded, normalized classes are allowed" };
  }

  return { ok: true, value: { query, kind: requestedKind, limit, offset,
    supplier: suppliers[0] ?? null, classes: [...new Set(classes)] } };
}

/// Turn untrusted user text into one literal FTS phrase. Doubling a quote is
/// FTS5's in-phrase escape, so operators in the query never become syntax.
export function quoteFtsPhrase(query: string): string {
  return `"${query.replaceAll('"', '""')}"`;
}

function emitLog(deps: SearchDependencies, entry: Record<string, unknown>): void {
  try {
    deps.log(entry);
  } catch {
    // Search failure reporting must never replace the response itself.
  }
}

function validSupplier(value: unknown): value is string {
  return typeof value === "string" && /^[a-z][a-z0-9_]{0,31}$/.test(value);
}

function validPn(value: unknown): value is string {
  return typeof value === "string" && value.trim().length > 0 &&
    !/[\p{Cc}\p{Cf}]/u.test(value) &&
    new TextEncoder().encode(normalizeUtf8(value)).length <= MAX_SUPPLIER_PN_BYTES;
}

function validOffset(value: unknown): value is string {
  return typeof value === "string" && value.length > 0 && value.length <= 64 &&
    /^-?(?:0|[1-9]\d*)(?:\.\d+)?mm$/.test(value);
}

function parseSuppliers(value: unknown): IndexedSupplier[] {
  if (!Array.isArray(value)) return [];
  const suppliers: IndexedSupplier[] = [];
  const seen = new Set<string>();
  for (const candidate of value.slice(0, MAX_SUPPLIERS_PER_AVL)) {
    if (!isRecord(candidate) || !validSupplier(candidate.supplier) || !validPn(candidate.pn) ||
        seen.has(candidate.supplier) || !Array.isArray(candidate.at) || candidate.at.length !== 2 ||
        !validOffset(candidate.at[0]) || !validOffset(candidate.at[1]) ||
        !Number.isSafeInteger(candidate.rotate) || (candidate.rotate as number) < 0 ||
        (candidate.rotate as number) > 359) continue;
    seen.add(candidate.supplier);
    suppliers.push({ supplier: candidate.supplier, pn: normalizeUtf8(candidate.pn),
      at: [candidate.at[0], candidate.at[1]], rotate: candidate.rotate as number });
  }
  return suppliers;
}

function parseAvl(raw: string): IndexedAvl[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch {
    return [];
  }
  if (!Array.isArray(parsed)) return [];
  const avl: IndexedAvl[] = [];
  for (const value of parsed) {
    if (!isRecord(value)) continue;
    const manufacturer = boundedString(value.manufacturer, MAX_ID_BYTES);
    const mpn = boundedString(value.mpn, MAX_ID_BYTES);
    avl.push({ primary: value.primary === true, manufacturer, mpn,
      suppliers: parseSuppliers(value.suppliers) });
  }
  return avl;
}

function publicPart(row: IndexedPartSearchRow, query: string): Omit<PublicPartSearchRow, "suppliers"> & { suppliers: IndexedSupplier[] } {
  const avl = parseAvl(row.avl_json);
  const needle = query.toLowerCase();
  const matching = avl.find(
    (entry) =>
      entry.manufacturer?.toLowerCase().includes(needle) === true ||
      entry.mpn?.toLowerCase().includes(needle) === true,
  );
  const selected = matching ?? avl.find((entry) => entry.primary) ?? avl[0] ?? null;
  const byIdentity = new Map<string, IndexedSupplier>();
  for (const entry of avl) {
    for (const binding of entry.suppliers) {
      const key = `${binding.supplier}\0${binding.pn}`;
      if (!byIdentity.has(key)) byIdentity.set(key, binding);
    }
  }
  return {
    package: row.package,
    tier: row.tier,
    version: row.version,
    fq: row.fq,
    name: row.name,
    device: row.device,
    intent: row.intent,
    manufacturer: selected?.manufacturer ?? null,
    mpn: selected?.mpn ?? null,
    primary: selected?.primary ?? true,
    suppliers: [...byIdentity.values()],
  };
}

function catalogKey(supplier: string, pn: string): string {
  return JSON.stringify([supplier, pn]);
}

function safeLinks(value: unknown): { label: string; url: string }[] {
  if (!Array.isArray(value)) return [];
  const result: { label: string; url: string }[] = [];
  const labels = new Set<string>();
  for (const item of value.slice(0, 16)) {
    if (!isRecord(item) || typeof item.label !== "string" || item.label.length === 0 ||
        item.label.length > 128 || item.label.normalize("NFC") !== item.label ||
        item.label.trim() !== item.label || /[\p{Cc}\p{Cf}]/u.test(item.label) ||
        labels.has(item.label) || !safeUrl(item.url)) continue;
    labels.add(item.label);
    result.push({ label: item.label, url: item.url });
  }
  return result;
}

function safeUrl(value: unknown): value is string {
  if (typeof value !== "string" || value.length > MAX_LINK_BYTES || /\s/u.test(value) ||
      new TextEncoder().encode(value).length > MAX_LINK_BYTES) return false;
  try {
    const url = new URL(value);
    return (url.protocol === "https:" || url.protocol === "http:") && !url.username && !url.password;
  } catch { return false; }
}

function productLink(template: string | null, pn: string): { label: string; url: string }[] {
  if (template === null || template.length > MAX_LINK_BYTES || /\s/u.test(template) ||
      (template.match(/\{pn\}/g)?.length ?? 0) !== 1) return [];
  const url = template.replaceAll("{pn}", encodeURIComponent(pn));
  return safeUrl(url) ? [{ label: "Product", url }] : [];
}

function enrichPart(
  part: Omit<PublicPartSearchRow, "suppliers"> & { suppliers: IndexedSupplier[] },
  catalog: Map<string, SupplierCatalogRow>,
): PublicPartSearchRow {
  return { ...part, suppliers: part.suppliers.map((binding) => {
    const row = catalog.get(catalogKey(binding.supplier, binding.pn));
    if (!row) return { ...binding, links: [], catalog: null };
    return { ...binding, links: [...productLink(row.product_url_template, binding.pn), ...row.links],
      catalog: { class: row.class, stock: row.stock, snapshot_date: row.snapshot_date } };
  }) };
}

export async function handleSearch(
  request: Request,
  url: URL,
  deps: SearchDependencies,
): Promise<Response> {
  if (request.method !== "GET") {
    return searchJson(405, { error: "method not allowed" }, { Allow: "GET" });
  }
  const parsed = parseSearchParams(url);
  if (!parsed.ok) return searchJson(400, { error: parsed.error });
  const { query, kind, limit, offset, supplier, classes } = parsed.value;
  const take = limit + 1;

  try {
    const [packageRows, partRows] = await Promise.all([
      kind === "part" ? Promise.resolve([]) : deps.store.searchPackages(query, take, offset),
      kind === "package"
        ? Promise.resolve([])
        : deps.store.searchParts(quoteFtsPhrase(query), take, offset, supplier, classes),
    ]);
    const visibleParts = partRows.slice(0, limit).map((row) => publicPart(row, query));
    const lookupKeys = [...new Map(visibleParts.flatMap((part) => part.suppliers)
      .map((binding) => [catalogKey(binding.supplier, binding.pn), binding])).values()];
    const catalog = await deps.store.catalog(lookupKeys);
    return searchJson(
      200,
      {
        query,
        packages: {
          results: packageRows.slice(0, limit).map((row) => ({
            ...row,
            description:
              row.description === null
                ? null
                : utf8Prefix(row.description, MAX_PACKAGE_DESCRIPTION_BYTES),
          })),
          has_more: packageRows.length > limit,
        },
        parts: {
          results: visibleParts.map((part) => enrichPart(part, catalog)),
          has_more: partRows.length > limit,
        },
      },
      { "Cache-Control": "public, max-age=30" },
    );
  } catch (error) {
    emitLog(deps, {
      event: "registry_search_error",
      error: error instanceof Error ? error.name : "UnknownError",
    });
    return searchJson(500, { error: "search is temporarily unavailable" });
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function normalizeUtf8(value: string): string {
  // JSON.parse accepts lone UTF-16 surrogates. TextEncoder replaces them with
  // U+FFFD; decode that byte sequence now so D1 and the strict Rust client see
  // one canonical, valid Unicode string.
  return new TextDecoder().decode(new TextEncoder().encode(value));
}

function utf8Prefix(value: string, maximum: number): string {
  // Every UTF-16 code unit contributes at least one byte once normalized to
  // UTF-8. Code units beyond `maximum` therefore cannot affect a prefix of
  // `maximum` bytes; slice first so one hostile multi-megabyte field cannot
  // allocate multiple full-size encoder/decoder temporaries merely to return
  // a 2 KiB value. A high surrogate cut at the boundary normalizes to U+FFFD,
  // which the byte-boundary pass below necessarily excludes.
  const candidate = value.length > maximum ? value.slice(0, maximum) : value;
  const normalized = normalizeUtf8(candidate);
  const bytes = new TextEncoder().encode(normalized);
  if (bytes.length <= maximum) return normalized;
  let end = maximum;
  // If the first excluded byte is a UTF-8 continuation, the boundary sits
  // inside a scalar. Walk back to its leading byte and exclude it whole.
  while (end > 0 && (bytes[end] & 0xc0) === 0x80) end -= 1;
  return new TextDecoder().decode(bytes.subarray(0, end));
}

function boundedString(value: unknown, maximum: number): string | null {
  if (typeof value !== "string") return null;
  const bounded = utf8Prefix(value, maximum);
  return bounded.length > 0 ? bounded : null;
}

function boundedIdentity(value: unknown): string | null {
  if (typeof value !== "string" || value.length === 0 || value.length > MAX_ID_BYTES) return null;
  const normalized = normalizeUtf8(value);
  return new TextEncoder().encode(normalized).length <= MAX_ID_BYTES ? normalized : null;
}

function fieldsForAvl(value: unknown): { avl: IndexedAvl; searchable: string[] } | null {
  if (!isRecord(value)) return null;
  const fields = Array.isArray(value.fields) ? value.fields : [];
  let manufacturer: string | null = null;
  let mpn: string | null = null;
  const searchable: string[] = [];
  for (const field of fields.slice(0, MAX_FIELDS_PER_AVL)) {
    if (!isRecord(field)) continue;
    const name = boundedString(field.name, 128);
    const fieldValue = boundedString(
      field.value,
      name === "mfr" || name === "mpn" ? MAX_ID_BYTES : 4_096,
    );
    if (!name || !fieldValue) continue;
    searchable.push(name, fieldValue);
    if (name === "mfr") manufacturer = fieldValue;
    if (name === "mpn") mpn = fieldValue;
  }
  return {
    avl: { primary: false, manufacturer, mpn, suppliers: [] },
    searchable,
  };
}

function supplierProjection(value: unknown): { suppliers: IndexedSupplier[]; pns: string[] } {
  const suppliers = parseSuppliers(isRecord(value) ? value.suppliers : undefined);
  return { suppliers, pns: suppliers.map((binding) => binding.pn) };
}

function packageRoot(packageName: string): string {
  const unscoped = packageName.startsWith("@") ? packageName.slice(1) : packageName;
  let root = [...unscoped]
    .map((character) => (/[A-Za-z0-9_]/.test(character) ? character : "_"))
    .join("");
  if (root.length === 0 || /^[0-9]/.test(root)) root = `_${root}`;
  return root;
}

/// Derive bounded, inert search rows from an already envelope-validated API
/// document. Only this package's public `items` participate: `foreign` is
/// render support supplied by dependencies, never an item this package owns.
export function extractPartSearchRows(
  document: unknown,
  packageName: string,
  packageVersion: string,
): PartSearchIndexRow[] {
  if (!isRecord(document) || !Array.isArray(document.items)) return [];
  const byFq = new Map<string, PartSearchIndexRow>();
  const root = packageRoot(packageName);
  const encoder = new TextEncoder();
  let projectedBytes = 2; // one conceptual JSON array: []
  let projectedChunkBytes = 2;
  let projectedChunkRows = 0;
  let projectedChunkCount = 0;
  let scanned = 0;
  for (const item of document.items) {
    if (scanned >= MAX_SCANNED_ITEMS) break;
    scanned += 1;
    if (byFq.size >= MAX_INDEXED_PARTS) break;
    if (!isRecord(item) || item.pub !== true || item.kind !== "part" || !isRecord(item.part)) {
      continue;
    }
    const fq = boundedIdentity(item.fq);
    const name = boundedIdentity(item.name);
    const device = boundedIdentity(item.part.device);
    if (!fq || !name || !device || byFq.has(fq)) continue;
    // `items` is only envelope-validated, so derive ownership from the URL's
    // already-validated package name. A publisher must never advertise an
    // actionable `std::...` or another package's path as its own result.
    if (!fq.startsWith(`${root}::`) || fq.slice(fq.lastIndexOf("::") + 2) !== name) continue;
    const intent = boundedString(item.intent, MAX_INTENT_BYTES);

    const entries: { value: unknown; primary: boolean }[] = [
      { value: item.part.primary, primary: true },
    ];
    if (Array.isArray(item.part.alts)) {
      for (const value of item.part.alts.slice(0, MAX_AVL_ENTRIES_PER_PART - 1)) {
        entries.push({ value, primary: false });
      }
    }
    const avl: IndexedAvl[] = [];
    // fq/name/device/intent have dedicated FTS columns; this catch-all adds
    // discovery keys that do not have public response columns of their own.
    const searchable: string[] = [];
    let searchableBytes = 0;
    const addSearchable = (value: string) => {
      const delimiter = searchable.length === 0 ? 0 : 1;
      const remaining = MAX_SEARCHABLE_BYTES - searchableBytes - delimiter;
      if (remaining <= 0) return;
      const bounded = utf8Prefix(value, remaining);
      if (!bounded) return;
      searchable.push(bounded);
      searchableBytes += encoder.encode(bounded).length + delimiter;
    };
    addSearchable(packageName);
    if (Array.isArray(item.part.args)) {
      for (const arg of item.part.args.slice(0, MAX_ARGS_PER_PART)) {
        const value = boundedString(arg, 4_096);
        if (value) addSearchable(value);
      }
    }
    const variant = boundedString(item.part.variant, MAX_ID_BYTES);
    if (variant) addSearchable(variant);
    for (const entry of entries) {
      const extracted = fieldsForAvl(entry.value);
      const projected = supplierProjection(entry.value);
      if (!extracted) continue;
      extracted.avl.primary = entry.primary;
      extracted.avl.suppliers = projected.suppliers;
      avl.push(extracted.avl);
      for (const value of extracted.searchable) addSearchable(value);
      for (const pn of projected.pns) addSearchable(pn);
    }
    if (!avl.some((entry) => entry.primary)) {
      avl.unshift({ primary: true, manufacturer: null, mpn: null, suppliers: [] });
    }
    const avlJson = JSON.stringify(avl);
    if (encoder.encode(avlJson).length > MAX_AVL_JSON_BYTES) continue;
    const row: PartSearchIndexRow = {
      package_name: packageName,
      package_version: packageVersion,
      fq,
      name,
      device,
      intent,
      searchable: utf8Prefix(searchable.join("\n"), MAX_SEARCHABLE_BYTES),
      avl_json: avlJson,
    };
    // One json_each bind must stay within the advertised chunk bound even
    // for a hostile-but-envelope-valid document containing one enormous
    // part. Normal emitter output is far smaller; an unindexable declaration
    // remains available in the stored sidecar, it simply cannot poison PUT.
    const encodedBytes = encoder.encode(JSON.stringify(row)).length;
    if (encodedBytes > PART_SEARCH_CHUNK_BYTES - 2) {
      continue;
    }
    const projectedDelimiter = byFq.size === 0 ? 0 : 1;
    if (projectedBytes + projectedDelimiter + encodedBytes > MAX_PART_SEARCH_INDEX_BYTES) {
      break;
    }
    const chunkDelimiter = projectedChunkRows === 0 ? 0 : 1;
    if (
      projectedChunkRows > 0 &&
      projectedChunkBytes + chunkDelimiter + encodedBytes > PART_SEARCH_CHUNK_BYTES
    ) {
      if (projectedChunkCount >= MAX_PART_SEARCH_CHUNKS) break;
      projectedChunkCount += 1;
      projectedChunkBytes = 2;
      projectedChunkRows = 0;
    }
    if (projectedChunkRows === 0 && projectedChunkCount === 0) {
      projectedChunkCount = 1;
    }
    projectedChunkBytes += encodedBytes + (projectedChunkRows === 0 ? 0 : 1);
    projectedChunkRows += 1;
    projectedBytes += projectedDelimiter + encodedBytes;
    byFq.set(fq, row);
  }
  return [...byFq.values()];
}

/// JSON-array chunks small enough to bind safely to D1. `json_each(?)` turns
/// each chunk into many rows in one statement, keeping a large (9k-part)
/// package well below the per-invocation query limit.
export function partSearchChunks(rows: PartSearchIndexRow[]): string[] {
  const encoder = new TextEncoder();
  const chunks: string[] = [];
  let current: string[] = [];
  let bytes = 2; // []
  for (const row of rows) {
    const encoded = JSON.stringify(row);
    const encodedBytes = encoder.encode(encoded).length;
    if (encodedBytes > PART_SEARCH_CHUNK_BYTES - 2) {
      throw new Error("part search row exceeds the D1 JSON chunk bound");
    }
    const rowBytes = encodedBytes + (current.length === 0 ? 0 : 1);
    if (current.length > 0 && bytes + rowBytes > PART_SEARCH_CHUNK_BYTES) {
      chunks.push(`[${current.join(",")}]`);
      if (chunks.length >= MAX_PART_SEARCH_CHUNKS) return chunks;
      current = [];
      bytes = 2;
    }
    current.push(encoded);
    bytes += encodedBytes + (current.length === 1 ? 0 : 1);
  }
  if (current.length > 0 && chunks.length < MAX_PART_SEARCH_CHUNKS) {
    chunks.push(`[${current.join(",")}]`);
  }
  return chunks;
}

function d1SearchStore(db: D1Database): SearchStore {
  return {
    async searchPackages(query: string, take: number, offset: number) {
      // D1 follows SQLite's built-in `lower()`: ASCII folds, while non-ASCII
      // code points compare literally. Package names are ASCII by grammar;
      // RFC-030 states the same bounded behavior for free-text descriptions.
      const rows = await db
        .prepare(
          `WITH ranked_versions AS (
             SELECT name, version, published_at, description,
                    ROW_NUMBER() OVER (
                      PARTITION BY name ORDER BY published_at DESC, version DESC
                    ) AS position
               FROM versions
           )
           SELECT p.name, p.tier, v.version AS latest,
                  substr(v.description, 1, 1024) AS description,
                  v.published_at AS updated
             FROM packages p
            JOIN ranked_versions v ON v.name = p.name AND v.position = 1
            WHERE instr(lower(p.name), lower(?)) > 0
               OR instr(lower(coalesce(substr(v.description, 1, 1024), '')), lower(?)) > 0
            ORDER BY CASE
                       WHEN lower(p.name) = lower(?) THEN 0
                       WHEN instr(lower(p.name), lower(?)) = 1 THEN 1
                       ELSE 2
                     END,
                     v.published_at DESC, p.name ASC
            LIMIT ? OFFSET ?`,
        )
        .bind(query, query, query, query, take, offset)
        .all<PackageSearchRow>();
      return rows.results.map((row) => ({
        ...row,
        description:
          row.description === null
            ? null
            : utf8Prefix(row.description, MAX_PACKAGE_DESCRIPTION_BYTES),
      }));
    },

    async searchParts(match: string, take: number, offset: number, supplier: string | null, classes: string[]) {
      const values: unknown[] = [match];
      let filter = "";
      if (supplier !== null) {
        values.push(supplier);
        filter += ` AND EXISTS (
          SELECT 1 FROM json_each(part_search.avl_json) AS avl,
                        json_each(json_extract(avl.value, '$.suppliers')) AS binding
           WHERE json_extract(binding.value, '$.supplier') = ?`;
        if (classes.length > 0) {
          values.push(...classes);
          filter += ` AND EXISTS (SELECT 1 FROM supplier_catalog
             WHERE supplier_catalog.supplier = json_extract(binding.value, '$.supplier')
               AND supplier_catalog.pn = json_extract(binding.value, '$.pn')
               AND supplier_catalog.class IN (${classes.map(() => "?").join(", ")}))`;
        }
        filter += ")";
      }
      const rows = await db
        .prepare(
          `WITH filtered_parts AS MATERIALIZED (
             SELECT part_search.*, bm25(part_search) AS rank
               FROM part_search
              WHERE part_search MATCH ?${filter}
           )
           SELECT filtered_parts.package_name, p.tier,
                  filtered_parts.package_version, filtered_parts.fq,
                  filtered_parts.name, filtered_parts.device,
                  filtered_parts.intent, filtered_parts.avl_json
             FROM filtered_parts
             JOIN packages p ON p.name = filtered_parts.package_name
            ORDER BY filtered_parts.rank, filtered_parts.package_name ASC,
                     filtered_parts.fq ASC
            LIMIT ? OFFSET ?`,
        )
        .bind(...values, take, offset)
        .all<{
          package_name: string;
          tier: "official" | "brand" | "contrib";
          package_version: string;
          fq: string;
          name: string;
          device: string;
          intent: string | null;
          avl_json: string;
        }>();
      return rows.results.map((row) => ({
        package: row.package_name,
        tier: row.tier,
        version: row.package_version,
        fq: row.fq,
        name: row.name,
        device: row.device,
        intent: row.intent,
        avl_json: row.avl_json,
      }));
    },

    async catalog(keys: readonly IndexedSupplier[]) {
      const catalog = new Map<string, SupplierCatalogRow>();
      for (let offset = 0; offset < keys.length; offset += CATALOG_LOOKUP_ROWS) {
        const chunk = keys.slice(offset, offset + CATALOG_LOOKUP_ROWS);
        const result = await db.prepare(`SELECT c.supplier, c.pn, c.class, c.stock,
              c.links, c.fetched_at, s.product_url_template
             FROM supplier_catalog c
             LEFT JOIN suppliers s ON s.id = c.supplier
            WHERE (c.supplier, c.pn) IN (${chunk.map(() => "(?, ?)").join(", ")})
            ORDER BY c.supplier ASC, c.pn ASC`)
          .bind(...chunk.flatMap((binding) => [binding.supplier, binding.pn])).all();
        for (const row of result.results) {
          const supplier = String(row.supplier);
          const pn = String(row.pn);
          if (!validSupplier(supplier) || !validPn(pn) || typeof row.class !== "string" ||
              row.class.length > MAX_CLASS_BYTES || !Number.isSafeInteger(Number(row.stock)) ||
              Number(row.stock) < 0 || typeof row.fetched_at !== "string" || row.fetched_at.length > 64) continue;
          let links: { label: string; url: string }[] = [];
          try { links = safeLinks(JSON.parse(String(row.links))); } catch { /* malformed rows expose no links */ }
          catalog.set(catalogKey(supplier, pn), { supplier, pn, class: row.class,
            stock: Number(row.stock), snapshot_date: row.fetched_at,
            product_url_template: typeof row.product_url_template === "string" ? row.product_url_template : null,
            links });
        }
      }
      return catalog;
    },
  };
}

export function searchApi(env: Env, request: Request, url: URL): Promise<Response> {
  return handleSearch(request, url, {
    store: d1SearchStore(env.DB),
    log: (entry) => console.error(JSON.stringify(entry)),
  });
}
