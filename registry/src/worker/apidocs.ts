// Package API documentation (docs/apidocs.md) — the `cohdl docs` sidecar.
//
// The docs artifact is a derived, re-generatable view of a published
// version: unlike the tar it is NOT identity (RFC-030 binds the content
// hash to the tar alone), so the owner may replace it — last write wins,
// e.g. after a compiler upgrade. The server validates only the envelope
// (UTF-8 JSON, top-level object, `schema_version` 1, `package.name` and
// `package.version` matching the URL, at most 16 MiB). Deep schema
// validation is deliberately not re-implemented server-side: the UI
// renders every field as inert text/SVG (no HTML path exists), so a
// malformed field can garble one page, never run. Search derives a bounded
// best-effort index from recognizable public `part` items; malformed deep
// fields are skipped and never make the otherwise-valid sidecar fail.

import { accountForToken, type Account, type Env } from "./auth";
import {
  extractPartSearchRows,
  partSearchChunks,
  type PartSearchIndexRow,
} from "./search";

const JSON_HEADERS = { "Content-Type": "application/json" };

export const APIDOCS_MAX_BYTES = 16 * 1024 * 1024;

/// Where a version's api-docs sidecar lives in R2 — beside, never inside,
/// the `pkg/` tar tree (the tar stays the sole identity).
export function apidocsKey(name: string, version: string): string {
  return `apidocs/${name}/${version}.json`;
}

/// New uploads use immutable, content-addressed objects. The key is fixed
/// length rather than embedding the package name: R2 keys are limited to
/// 1024 bytes, while a valid package name can itself be long. The document
/// envelope already contains and validates package name + version. The D1
/// version row points at one of these keys in the same transaction that
/// replaces the search index, so concurrent PUTs cannot leave R2 bytes from
/// upload B with searchable rows from upload A. `apidocsKey` remains the
/// legacy fallback for sidecars written before migration 0003.
export function apidocsContentKey(hash: string): string {
  return `apidocs/sha256/${hash}.json`;
}

export type ApidocsVerdict =
  | { ok: true; size: number }
  | { ok: false; status: 400 | 413; error: string };

type ParsedApidocsVerdict =
  | { ok: true; size: number; document: Record<string, unknown> }
  | { ok: false; status: 400 | 413; error: string };

/// The envelope check: raw upload bytes against the URL's name/version.
/// Everything beyond the envelope is the emitter's contract, not ours.
function parseApidocs(body: Uint8Array, name: string, version: string): ParsedApidocsVerdict {
  if (body.length > APIDOCS_MAX_BYTES) {
    return {
      ok: false,
      status: 413,
      error: `api docs are ${body.length} bytes — the limit is 16 MiB (${APIDOCS_MAX_BYTES})`,
    };
  }
  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true }).decode(body);
  } catch {
    return { ok: false, status: 400, error: "api docs must be valid UTF-8" };
  }
  let doc: unknown;
  try {
    doc = JSON.parse(text);
  } catch {
    return { ok: false, status: 400, error: "api docs must be valid JSON" };
  }
  if (typeof doc !== "object" || doc === null || Array.isArray(doc)) {
    return { ok: false, status: 400, error: "api docs must be a top-level JSON object" };
  }
  if (Reflect.get(doc, "schema_version") !== 1) {
    return { ok: false, status: 400, error: "api docs must declare `schema_version` 1" };
  }
  const pkgField: unknown = Reflect.get(doc, "package");
  const pkg =
    typeof pkgField === "object" && pkgField !== null && !Array.isArray(pkgField) ? pkgField : null;
  const declaredName: unknown = pkg ? Reflect.get(pkg, "name") : undefined;
  const declaredVersion: unknown = pkg ? Reflect.get(pkg, "version") : undefined;
  if (declaredName !== name) {
    return {
      ok: false,
      status: 400,
      error: `the api docs declare \`package.name = "${typeof declaredName === "string" ? declaredName : ""}"\` but this uploads docs for \`${name}\` — the document and the URL must agree`,
    };
  }
  if (declaredVersion !== version) {
    return {
      ok: false,
      status: 400,
      error: `the api docs declare \`package.version = "${typeof declaredVersion === "string" ? declaredVersion : ""}"\` but this uploads docs for \`${version}\` — the document and the URL must agree`,
    };
  }
  return { ok: true, size: body.length, document: doc as Record<string, unknown> };
}

/// Public envelope validator retained as the small contract tests and callers
/// expect. The PUT path uses `parseApidocs` directly so a 16 MiB sidecar is
/// never decoded and parsed twice merely to derive its search rows.
export function validateApidocs(body: Uint8Array, name: string, version: string): ApidocsVerdict {
  const verdict = parseApidocs(body, name, version);
  return verdict.ok
    ? { ok: true, size: verdict.size }
    : { ok: false, status: verdict.status, error: verdict.error };
}

export interface ApidocsStore {
  /// The owning account id of a `packages` row, or null when unpublished.
  packageOwner(name: string): Promise<number | null>;
  versionExists(name: string, version: string): Promise<boolean>;
  /// Store the validated bytes and flag the version row. Byte-preserving:
  /// what was uploaded is exactly what `/api/apidocs` later serves.
  put(
    name: string,
    version: string,
    body: Uint8Array,
    parts: PartSearchIndexRow[],
  ): Promise<void>;
}

export interface ApidocsDependencies {
  tokenAccount(request: Request): Promise<Account | null>;
  store: ApidocsStore;
}

export interface D1Mutation {
  sql: string;
  bindings: unknown[];
}

/// The D1 half of a docs upload as plain data, both to keep the batch
/// construction reviewable and to pin its latest-version guards in tests.
export function partSearchMutations(
  name: string,
  version: string,
  size: number,
  r2Key: string,
  parts: PartSearchIndexRow[],
): D1Mutation[] {
  const latestVersion = `(SELECT version FROM versions
                            WHERE name = ?
                            ORDER BY published_at DESC, version DESC
                            LIMIT 1)`;
  return [
    {
      sql: `UPDATE versions
               SET api_docs_size = ?, api_docs_r2_key = ?
             WHERE name = ? AND version = ?`,
      bindings: [size, r2Key, name, version],
    },
    {
      sql: `DELETE FROM part_search
             WHERE package_name = ?
               AND ? = ${latestVersion}`,
      bindings: [name, version, name],
    },
    ...partSearchChunks(parts).map((chunk) => ({
      sql: `INSERT INTO part_search
              (package_name, package_version, fq, name, device, intent, searchable, avl_json)
            SELECT json_extract(value, '$.package_name'),
                   json_extract(value, '$.package_version'),
                   json_extract(value, '$.fq'),
                   json_extract(value, '$.name'),
                   json_extract(value, '$.device'),
                   json_extract(value, '$.intent'),
                   json_extract(value, '$.searchable'),
                   json_extract(value, '$.avl_json')
              FROM json_each(?)
             WHERE ? = ${latestVersion}`,
      bindings: [chunk, version, name],
    })),
  ];
}

type ApidocsBodyResult =
  | { ok: true; body: Uint8Array<ArrayBuffer> }
  | { ok: false; error: string };

async function readApidocsBody(request: Request): Promise<ApidocsBodyResult> {
  const declared = request.headers.get("Content-Length");
  if (declared !== null && /^\d+$/.test(declared) && Number(declared) > APIDOCS_MAX_BYTES) {
    return { ok: false, error: "api docs exceed the 16 MiB upload limit" };
  }
  if (!request.body) return { ok: true, body: new Uint8Array() };

  const reader = request.body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > APIDOCS_MAX_BYTES) {
        await reader.cancel("api docs upload exceeds 16 MiB");
        return { ok: false, error: "api docs exceed the 16 MiB upload limit" };
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }

  const body = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    body.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return { ok: true, body };
}

async function sha256Hex(body: Uint8Array): Promise<string> {
  const bytes = body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength) as ArrayBuffer;
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  return [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function apidocsJson(status: number, body: unknown): Response {
  return new Response(JSON.stringify(body), { status, headers: JSON_HEADERS });
}

function notPublished(name: string, version: string): string {
  return `\`${name} ${version}\` is not published — publish the version before uploading api docs`;
}

/// `PUT /packages/{name}/{version}/docs` — owner-authenticated upload.
/// Replacing an existing upload is allowed (the sidecar is not identity).
export async function handleApidocsPut(
  request: Request,
  name: string,
  version: string,
  deps: ApidocsDependencies,
): Promise<Response> {
  const account = await deps.tokenAccount(request);
  if (!account) return apidocsJson(401, { error: "login required" });
  const owner = await deps.store.packageOwner(name);
  if (owner === null) return apidocsJson(404, { error: notPublished(name, version) });
  if (owner !== account.id) {
    return apidocsJson(403, { error: `\`${name}\` is owned by another account` });
  }
  if (!(await deps.store.versionExists(name, version))) {
    return apidocsJson(404, { error: notPublished(name, version) });
  }
  const read = await readApidocsBody(request);
  if (!read.ok) return apidocsJson(413, { error: read.error });
  const body = read.body;
  const verdict = parseApidocs(body, name, version);
  if (!verdict.ok) return apidocsJson(verdict.status, { error: verdict.error });
  const parts = extractPartSearchRows(verdict.document, name, version);
  await deps.store.put(name, version, body, parts);
  return apidocsJson(200, { name, version, size: verdict.size });
}

function d1r2Store(env: Env): ApidocsStore {
  return {
    async packageOwner(name: string): Promise<number | null> {
      const row = await env.DB.prepare("SELECT owner_account FROM packages WHERE name = ?")
        .bind(name)
        .first<{ owner_account: number }>();
      return row?.owner_account ?? null;
    },

    async versionExists(name: string, version: string): Promise<boolean> {
      return (
        (await env.DB.prepare("SELECT 1 AS found FROM versions WHERE name = ? AND version = ?")
          .bind(name, version)
          .first<{ found: number }>()) !== null
      );
    },

    async put(
      name: string,
      version: string,
      body: Uint8Array,
      parts: PartSearchIndexRow[],
    ): Promise<void> {
      const hash = await sha256Hex(body);
      const key = apidocsContentKey(hash);
      await env.PKG.put(key, body, {
        httpMetadata: { contentType: "application/json" },
      });

      // D1's batch is transactional. The version guard is repeated on every
      // mutation so a late upload for an older version can store its docs but
      // can never displace the newest release's searchable parts.
      const statements = partSearchMutations(name, version, body.length, key, parts).map(
        ({ sql, bindings }) => env.DB.prepare(sql).bind(...bindings),
      );
      await env.DB.batch(statements);
    },
  };
}

export async function apidocsPut(
  env: Env,
  request: Request,
  name: string,
  version: string,
): Promise<Response> {
  return handleApidocsPut(request, name, version, {
    tokenAccount: (candidate) => accountForToken(env, candidate),
    store: d1r2Store(env),
  });
}
