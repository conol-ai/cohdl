// Package API documentation (docs/apidocs.md) — the `cohdl docs` sidecar.
//
// The docs artifact is a derived, re-generatable view of a published
// version: unlike the tar it is NOT identity (RFC-030 binds the content
// hash to the tar alone), so the owner may replace it — last write wins,
// e.g. after a compiler upgrade. Documents through 16 MB retain the original
// fully parsed/indexed envelope-validation path. Larger documents use the
// canonical emitter prefix plus an R2-verified SHA-256 checksum, up to the
// 200 MB application cap, so the Worker never materializes a document larger
// than its isolate memory.
// Deep schema validation is deliberately not re-implemented server-side:
// the UI renders every field as inert text/SVG (no HTML path exists), so a
// malformed field can garble one page, never run. Search derives a bounded
// best-effort index from recognizable public `part` items on the fully
// parsed path; the streaming path deliberately publishes an empty index.

import { accountForToken, type Account, type Env } from "./auth";
import {
  extractPartSearchRows,
  partSearchChunks,
  type PartSearchIndexRow,
} from "./search";

const JSON_HEADERS = { "Content-Type": "application/json" };

export const APIDOCS_MAX_BYTES = 200_000_000;
// Keep this decimal threshold just below the compact STM32 catalog
// (currently about 16.77 MB): parsing that highly nested document exceeds the
// Worker's 128 MB isolate even though its wire bytes fit under the old cap.
export const APIDOCS_BUFFER_MAX_BYTES = 16_000_000;
export const APIDOCS_PREFIX_MAX_BYTES = 64 * 1024;

export interface ApidocsLimits {
  maxBytes: number;
  bufferMaxBytes: number;
  prefixMaxBytes: number;
}

const DEFAULT_LIMITS: ApidocsLimits = {
  maxBytes: APIDOCS_MAX_BYTES,
  bufferMaxBytes: APIDOCS_BUFFER_MAX_BYTES,
  prefixMaxBytes: APIDOCS_PREFIX_MAX_BYTES,
};

/// Numeric boundary check kept separate so tests never allocate a 200 MB
/// value merely to prove the inclusive application cap.
export function apidocsSizeWithinLimit(
  size: number,
  maximum: number = APIDOCS_MAX_BYTES,
): boolean {
  return Number.isSafeInteger(size) && size >= 0 && size <= maximum;
}

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
  if (body.length > APIDOCS_BUFFER_MAX_BYTES) {
    return {
      ok: false,
      status: 413,
      error: `api docs are ${body.length} bytes — the fully parsed limit is 16 MB (${APIDOCS_BUFFER_MAX_BYTES})`,
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
/// expect. The buffered PUT path uses `parseApidocs` directly so a 16 MB sidecar is
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
    body: Uint8Array<ArrayBuffer>,
    parts: PartSearchIndexRow[],
  ): Promise<void>;
  /// Store a large canonical-emitter document without buffering it. Its
  /// checksum is supplied to R2, which rejects any byte mismatch. This path
  /// cannot safely materialize `items`, so it atomically publishes no part
  /// rows (and therefore clears any stale latest-version rows).
  putStream(
    name: string,
    version: string,
    body: ReadableStream<Uint8Array>,
    size: number,
    sha256: string,
  ): Promise<void>;
}

export interface ApidocsDependencies {
  tokenAccount(request: Request): Promise<Account | null>;
  store: ApidocsStore;
  limits?: ApidocsLimits;
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

async function readApidocsBody(
  request: Request,
  maximum: number,
): Promise<ApidocsBodyResult> {
  if (!request.body) return { ok: true, body: new Uint8Array() };

  const reader = request.body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      size += value.byteLength;
      if (size > maximum) {
        await reader.cancel("api docs upload exceeds the fully parsed limit");
        return {
          ok: false,
          error:
            maximum === APIDOCS_BUFFER_MAX_BYTES
              ? "api docs larger than 16 MB require a canonical Content-Length and streaming metadata"
              : `api docs larger than ${maximum} bytes require streaming metadata`,
        };
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

type DeclaredLengthVerdict =
  | { ok: true; size: number | null }
  | { ok: false; error: string };

function declaredContentLength(request: Request): DeclaredLengthVerdict {
  const raw = request.headers.get("Content-Length");
  if (raw === null) return { ok: true, size: null };
  if (!/^(0|[1-9]\d*)$/.test(raw)) {
    return { ok: false, error: "api docs need a canonical decimal Content-Length" };
  }
  const size = Number(raw);
  if (!Number.isSafeInteger(size)) {
    return { ok: false, error: "api docs Content-Length is too large" };
  }
  return { ok: true, size };
}

const CANONICAL_START = new TextEncoder().encode('{"schema_version":1,"generator":');
const CANONICAL_ITEMS_OPENER = new TextEncoder().encode(',"items":[');

function bytesStartWith(body: Uint8Array, prefix: Uint8Array): boolean {
  if (body.byteLength < prefix.byteLength) return false;
  for (let index = 0; index < prefix.byteLength; index += 1) {
    if (body[index] !== prefix[index]) return false;
  }
  return true;
}

function findBytes(body: Uint8Array, needle: Uint8Array): number {
  const last = body.byteLength - needle.byteLength;
  outer: for (let start = 0; start <= last; start += 1) {
    for (let offset = 0; offset < needle.byteLength; offset += 1) {
      if (body[start + offset] !== needle[offset]) continue outer;
    }
    return start;
  }
  return -1;
}

type PrefixVerdict = { ok: true } | { ok: false; error: string };

function canonicalPrefixEnvelope(
  prefix: Uint8Array,
  name: string,
  version: string,
): PrefixVerdict {
  if (!bytesStartWith(prefix, CANONICAL_START)) {
    return { ok: false, error: "large api docs must use the canonical CoHDL JSON prefix" };
  }
  const opener = findBytes(prefix, CANONICAL_ITEMS_OPENER);
  if (opener < 0) {
    return { ok: false, error: "large api docs are missing the canonical `items` opener" };
  }
  const throughOpener = opener + CANONICAL_ITEMS_OPENER.byteLength;
  let text: string;
  try {
    text = new TextDecoder("utf-8", { fatal: true }).decode(prefix.subarray(0, throughOpener));
  } catch {
    return { ok: false, error: "api docs must be valid UTF-8" };
  }
  let document: unknown;
  try {
    // Close the just-opened, still-empty `items` array and root object. This
    // validates the complete canonical envelope without touching the large
    // declaration payload which follows the opener.
    document = JSON.parse(`${text}]\n}`);
  } catch {
    return { ok: false, error: "large api docs must have a valid canonical JSON envelope" };
  }
  if (typeof document !== "object" || document === null || Array.isArray(document)) {
    return { ok: false, error: "api docs must be a top-level JSON object" };
  }
  const keys = Object.keys(document);
  if (
    keys.length !== 5 ||
    keys[0] !== "schema_version" ||
    keys[1] !== "generator" ||
    keys[2] !== "package" ||
    keys[3] !== "dependencies" ||
    keys[4] !== "items" ||
    typeof Reflect.get(document, "generator") !== "string" ||
    !Array.isArray(Reflect.get(document, "dependencies")) ||
    !Array.isArray(Reflect.get(document, "items"))
  ) {
    return { ok: false, error: "large api docs must use the canonical CoHDL JSON envelope" };
  }
  if (Reflect.get(document, "schema_version") !== 1) {
    return { ok: false, error: "api docs must declare `schema_version` 1" };
  }
  const pkgField: unknown = Reflect.get(document, "package");
  const pkg =
    typeof pkgField === "object" && pkgField !== null && !Array.isArray(pkgField)
      ? pkgField
      : null;
  const declaredName: unknown = pkg ? Reflect.get(pkg, "name") : undefined;
  const declaredVersion: unknown = pkg ? Reflect.get(pkg, "version") : undefined;
  if (declaredName !== name) {
    return {
      ok: false,
      error: `the api docs declare \`package.name = "${typeof declaredName === "string" ? declaredName : ""}"\` but this uploads docs for \`${name}\` — the document and the URL must agree`,
    };
  }
  if (declaredVersion !== version) {
    return {
      ok: false,
      error: `the api docs declare \`package.version = "${typeof declaredVersion === "string" ? declaredVersion : ""}"\` but this uploads docs for \`${version}\` — the document and the URL must agree`,
    };
  }
  return { ok: true };
}

async function validateCanonicalPrefix(
  body: ReadableStream<Uint8Array>,
  name: string,
  version: string,
  maximum: number,
): Promise<PrefixVerdict> {
  const reader = body.getReader();
  const chunks: Uint8Array[] = [];
  let size = 0;
  try {
    while (size < maximum) {
      const { done, value } = await reader.read();
      if (done) break;
      const remaining = maximum - size;
      const kept = value.byteLength > remaining ? value.subarray(0, remaining) : value;
      chunks.push(kept);
      size += kept.byteLength;

      const prefix = new Uint8Array(size);
      let offset = 0;
      for (const chunk of chunks) {
        prefix.set(chunk, offset);
        offset += chunk.byteLength;
      }
      const opener = findBytes(prefix, CANONICAL_ITEMS_OPENER);
      if (opener >= 0) {
        // Register cancellation immediately so this tee branch does not queue
        // the remaining document while the sibling branch streams to R2. Its
        // promise resolves only once the sibling finishes, so do not await it.
        void reader.cancel("canonical api-docs prefix validated").catch(() => undefined);
        return canonicalPrefixEnvelope(prefix, name, version);
      }
      if (value.byteLength > remaining) break;
    }
    void reader.cancel("canonical api-docs prefix exceeds its bound").catch(() => undefined);
    return {
      ok: false,
      error:
        maximum === APIDOCS_PREFIX_MAX_BYTES
          ? "large api docs must reach the canonical `items` opener within 64 KiB"
          : `large api docs must reach the canonical \`items\` opener within ${maximum} bytes`,
    };
  } finally {
    reader.releaseLock();
  }
}

async function sha256Hex(body: Uint8Array<ArrayBuffer>): Promise<string> {
  const bytes =
    body.byteOffset === 0 && body.byteLength === body.buffer.byteLength
      ? body.buffer
      : body.buffer.slice(body.byteOffset, body.byteOffset + body.byteLength);
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
  const limits = deps.limits ?? DEFAULT_LIMITS;
  const declared = declaredContentLength(request);
  if (!declared.ok) return apidocsJson(400, { error: declared.error });
  if (declared.size !== null && !apidocsSizeWithinLimit(declared.size, limits.maxBytes)) {
    return apidocsJson(413, {
      error:
        limits.maxBytes === APIDOCS_MAX_BYTES
          ? "api docs exceed the 200 MB upload limit"
          : `api docs exceed the ${limits.maxBytes}-byte upload limit`,
    });
  }

  if (declared.size !== null && declared.size > limits.bufferMaxBytes) {
    if (request.headers.get("X-CoHDL-Api-Docs-Schema") !== "1") {
      return apidocsJson(400, {
        error: "large api docs need `X-CoHDL-Api-Docs-Schema: 1`",
      });
    }
    const sha256 = request.headers.get("X-CoHDL-Api-Docs-SHA256") ?? "";
    if (!/^[0-9a-f]{64}$/.test(sha256)) {
      return apidocsJson(400, {
        error: "large api docs need `X-CoHDL-Api-Docs-SHA256` as 64 lowercase hex digits",
      });
    }
    if (!request.body) {
      return apidocsJson(400, { error: "large api docs need a request body" });
    }

    const [prefixBody, storageBody] = request.body.tee();
    const prefix = await validateCanonicalPrefix(
      prefixBody,
      name,
      version,
      limits.prefixMaxBytes,
    );
    if (!prefix.ok) {
      await storageBody.cancel("invalid canonical api-docs prefix").catch(() => undefined);
      return apidocsJson(400, { error: prefix.error });
    }

    // Large documents stay byte-preserved and checksum-verified, but are not
    // materialized for part projection. `putStream` atomically publishes an
    // empty part set, clearing stale rows if this is the newest version.
    await deps.store.putStream(name, version, storageBody, declared.size, sha256);
    return apidocsJson(200, { name, version, size: declared.size });
  }

  const read = await readApidocsBody(request, limits.bufferMaxBytes);
  if (!read.ok) return apidocsJson(413, { error: read.error });
  const body = read.body;
  if (declared.size !== null && declared.size !== body.byteLength) {
    return apidocsJson(400, {
      error: `api docs Content-Length declares ${declared.size} bytes but received ${body.byteLength}`,
    });
  }
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
      body: Uint8Array<ArrayBuffer>,
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

    async putStream(
      name: string,
      version: string,
      body: ReadableStream<Uint8Array>,
      size: number,
      sha256: string,
    ): Promise<void> {
      const key = apidocsContentKey(sha256);
      // A plain TransformStream loses the request's known length. R2 accepts
      // this FixedLengthStream directly, enforces the declared byte count,
      // and independently verifies the client-computed SHA-256 while bytes
      // flow through without entering the JS heap as one contiguous value.
      const fixed = new FixedLengthStream(size);
      const write = body.pipeTo(fixed.writable);
      const upload = env.PKG.put(key, fixed.readable, {
        httpMetadata: { contentType: "application/json" },
        sha256,
      });
      await Promise.all([write, upload]);

      // No full `items` tree exists on this path. The same transactional
      // mutation plan therefore updates the sidecar pointer and deliberately
      // clears the newest version's derived part rows.
      const statements = partSearchMutations(name, version, size, key, []).map(
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
