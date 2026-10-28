import type { Env } from "./auth";
import { normalizeManufacturer, normalizeMpn } from "./suppliers";

const JSON_HEADERS = { "Content-Type": "application/json", "Cache-Control": "public, max-age=60" };
const MAX_VALUES = 100;
const MAX_VALUE_BYTES = 256;

export interface ManufacturerKeyStore {
  canonicalByAliasKey(keys: string[]): Promise<ReadonlyMap<string, string>>;
}

export async function resolveManufacturerKeys(
  values: string[], store: ManufacturerKeyStore,
): Promise<string[]> {
  const normalized = values.map(normalizeManufacturer);
  const aliases = await store.canonicalByAliasKey([...new Set(normalized)]);
  return normalized.map((key) => normalizeManufacturer(aliases.get(key) ?? key));
}

export function manufacturerKeyStore(db: D1Database): ManufacturerKeyStore {
  return {
    async canonicalByAliasKey(keys) {
      const canonical = new Map<string, string>();
      for (let offset = 0; offset < keys.length; offset += MAX_VALUES) {
        const chunk = keys.slice(offset, offset + MAX_VALUES);
        const result = await db.prepare(`SELECT alias_key, canonical FROM manufacturer_aliases
          WHERE alias_key IN (${chunk.map(() => "?").join(", ")})`).bind(...chunk)
          .all<{ alias_key: string; canonical: string }>();
        for (const row of result.results) canonical.set(row.alias_key, row.canonical);
      }
      return canonical;
    },
  };
}

function json(status: number, body: unknown, headers: Record<string, string> = {}): Response {
  return new Response(JSON.stringify(body), { status, headers: { ...JSON_HEADERS, ...headers } });
}

function validValue(value: string, normalize: (value: string) => string): boolean {
  return value.length > 0 && new TextEncoder().encode(value).length <= MAX_VALUE_BYTES &&
    !/\p{Cc}/u.test(value) && normalize(value).length > 0;
}

export interface ManufacturerNormalizeDependencies { store: ManufacturerKeyStore }

export async function handleManufacturerNormalize(
  request: Request, url: URL, deps: ManufacturerNormalizeDependencies,
): Promise<Response> {
  if (request.method !== "GET") return json(405, { error: "method not allowed" }, { Allow: "GET" });
  const keys = [...new Set(url.searchParams.keys())];
  if (keys.some((key) => key !== "mfr" && key !== "mpn")) {
    return json(400, { error: "only mfr and mpn query parameters are allowed" });
  }
  const mfr = url.searchParams.getAll("mfr");
  const mpn = url.searchParams.getAll("mpn");
  if (mfr.length + mpn.length === 0) return json(400, { error: "provide at least one mfr or mpn" });
  if (mfr.length > MAX_VALUES || mpn.length > MAX_VALUES) {
    return json(400, { error: "at most 100 values per parameter are allowed" });
  }
  if (mfr.some((value) => !validValue(value, normalizeManufacturer)) ||
      mpn.some((value) => !validValue(value, normalizeMpn))) {
    return json(400, { error: "values must be non-empty, at most 256 UTF-8 bytes, and free of control characters" });
  }
  try {
    const manufacturerKeys = await resolveManufacturerKeys(mfr, deps.store);
    return json(200, {
      mfr: mfr.map((value, index) => ({ value, key: manufacturerKeys[index] })),
      mpn: mpn.map((value) => ({ value, key: normalizeMpn(value) })),
    });
  } catch {
    return json(500, { error: "internal error" });
  }
}

export async function manufacturerNormalizeApi(env: Env, request: Request, url: URL): Promise<Response> {
  return handleManufacturerNormalize(request, url, { store: manufacturerKeyStore(env.DB) });
}