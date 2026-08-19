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
// malformed field can garble one page, never run. The worker itself only
// stores and serves the bytes — it never interprets them.

import { accountForToken, type Account, type Env } from "./auth";

const JSON_HEADERS = { "Content-Type": "application/json" };

export const APIDOCS_MAX_BYTES = 16 * 1024 * 1024;

/// Where a version's api-docs sidecar lives in R2 — beside, never inside,
/// the `pkg/` tar tree (the tar stays the sole identity).
export function apidocsKey(name: string, version: string): string {
  return `apidocs/${name}/${version}.json`;
}

export type ApidocsVerdict =
  | { ok: true; size: number }
  | { ok: false; status: 400 | 413; error: string };

/// The envelope check: raw upload bytes against the URL's name/version.
/// Everything beyond the envelope is the emitter's contract, not ours.
export function validateApidocs(body: Uint8Array, name: string, version: string): ApidocsVerdict {
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
  return { ok: true, size: body.length };
}

export interface ApidocsStore {
  /// The owning account id of a `packages` row, or null when unpublished.
  packageOwner(name: string): Promise<number | null>;
  versionExists(name: string, version: string): Promise<boolean>;
  /// Store the validated bytes and flag the version row. Byte-preserving:
  /// what was uploaded is exactly what `/api/apidocs` later serves.
  put(name: string, version: string, body: Uint8Array): Promise<void>;
}

export interface ApidocsDependencies {
  tokenAccount(request: Request): Promise<Account | null>;
  store: ApidocsStore;
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
  const body = new Uint8Array(await request.arrayBuffer());
  const verdict = validateApidocs(body, name, version);
  if (!verdict.ok) return apidocsJson(verdict.status, { error: verdict.error });
  await deps.store.put(name, version, body);
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

    async put(name: string, version: string, body: Uint8Array): Promise<void> {
      await env.PKG.put(apidocsKey(name, version), body, {
        httpMetadata: { contentType: "application/json" },
      });
      await env.DB.prepare(
        "UPDATE versions SET api_docs_size = ? WHERE name = ? AND version = ?",
      )
        .bind(body.length, name, version)
        .run();
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
