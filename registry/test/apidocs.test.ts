import { describe, expect, it, vi } from "vitest";
import {
  APIDOCS_BUFFER_MAX_BYTES,
  APIDOCS_MAX_BYTES,
  apidocsContentKey,
  apidocsKey,
  apidocsSizeWithinLimit,
  handleApidocsPut,
  partSearchMutations,
  validateApidocs,
  type ApidocsDependencies,
  type ApidocsLimits,
  type ApidocsStore,
} from "../src/worker/apidocs";
import type { Account } from "../src/worker/auth";

const OWNER: Account = { id: 1, email: "owner@example.com", is_official: 0 };
const OTHER: Account = { id: 2, email: "other@example.com", is_official: 0 };

function doc(name = "passive", version = "1.0.0"): Uint8Array<ArrayBuffer> {
  return new TextEncoder().encode(
    JSON.stringify({ schema_version: 1, package: { name, version }, items: [] }),
  );
}

describe("apidocsKey", () => {
  it("stores sidecars beside, never inside, the pkg/ tar tree", () => {
    expect(apidocsKey("passive", "1.0.0")).toBe("apidocs/passive/1.0.0.json");
  });

  it("keeps a scoped name's @scope/ segment", () => {
    expect(apidocsKey("@st/stm32", "0.1.0")).toBe("apidocs/@st/stm32/0.1.0.json");
  });

  it("gives new uploads immutable content-addressed keys", () => {
    expect(apidocsContentKey("abc123")).toBe("apidocs/sha256/abc123.json");
    expect(
      new TextEncoder().encode(apidocsContentKey("a".repeat(64))).length,
    ).toBeLessThan(1024);
  });
});

describe("validateApidocs", () => {
  it("accepts a valid document and reports its byte size", () => {
    const body = doc();
    expect(validateApidocs(body, "passive", "1.0.0")).toEqual({
      ok: true,
      size: body.length,
    });
  });

  it("accepts a scoped package name", () => {
    const body = doc("@st/stm32", "0.1.0");
    expect(validateApidocs(body, "@st/stm32", "0.1.0")).toEqual({
      ok: true,
      size: body.length,
    });
  });

  it("refuses bytes that are not valid UTF-8", () => {
    const verdict = validateApidocs(new Uint8Array([0x7b, 0xff, 0x7d]), "p", "1.0.0");
    expect(verdict).toEqual({ ok: false, status: 400, error: "api docs must be valid UTF-8" });
  });

  it("refuses bytes that are not valid JSON", () => {
    const verdict = validateApidocs(new TextEncoder().encode("{"), "p", "1.0.0");
    expect(verdict).toEqual({ ok: false, status: 400, error: "api docs must be valid JSON" });
  });

  it("refuses any non-object top level", () => {
    for (const top of [[{ schema_version: 1 }], "docs", 3, true, null]) {
      const body = new TextEncoder().encode(JSON.stringify(top));
      const verdict = validateApidocs(body, "p", "1.0.0");
      expect(verdict).toEqual({
        ok: false,
        status: 400,
        error: "api docs must be a top-level JSON object",
      });
    }
  });

  it("refuses every schema_version except the number 1", () => {
    for (const schema_version of [2, 0, "1", 1.5, null, undefined, [1]]) {
      const body = new TextEncoder().encode(
        JSON.stringify({ schema_version, package: { name: "p", version: "1.0.0" } }),
      );
      const verdict = validateApidocs(body, "p", "1.0.0");
      expect(verdict).toEqual({
        ok: false,
        status: 400,
        error: "api docs must declare `schema_version` 1",
      });
    }
  });

  it("refuses a package.name that disagrees with the URL", () => {
    const verdict = validateApidocs(doc("other", "1.0.0"), "passive", "1.0.0");
    expect(verdict).toEqual({
      ok: false,
      status: 400,
      error:
        'the api docs declare `package.name = "other"` but this uploads docs for `passive` — the document and the URL must agree',
    });
  });

  it("refuses a package.version that disagrees with the URL", () => {
    const verdict = validateApidocs(doc("passive", "2.0.0"), "passive", "1.0.0");
    expect(verdict).toEqual({
      ok: false,
      status: 400,
      error:
        'the api docs declare `package.version = "2.0.0"` but this uploads docs for `1.0.0` — the document and the URL must agree',
    });
  });

  it("names the mismatch without echoing a non-string value", () => {
    const body = new TextEncoder().encode(
      JSON.stringify({ schema_version: 1, package: { name: 7, version: "1.0.0" } }),
    );
    const verdict = validateApidocs(body, "passive", "1.0.0");
    expect(verdict.ok).toBe(false);
    if (!verdict.ok) expect(verdict.error).toContain('`package.name = ""`');

    const missing = new TextEncoder().encode(JSON.stringify({ schema_version: 1 }));
    const noPkg = validateApidocs(missing, "passive", "1.0.0");
    expect(noPkg.ok).toBe(false);
    if (!noPkg.ok) expect(noPkg.error).toContain('`package.name = ""`');
  });

  it("keeps a 16 MB fully parsed threshold under the 200 MB application cap", () => {
    expect(APIDOCS_BUFFER_MAX_BYTES).toBe(16_000_000);
    expect(APIDOCS_MAX_BYTES).toBe(200_000_000);
    // Prove both inclusive numeric boundaries without allocating either a
    // 16 MB legacy document or a 200 MB streaming document in the test.
    expect(apidocsSizeWithinLimit(APIDOCS_MAX_BYTES)).toBe(true);
    expect(apidocsSizeWithinLimit(APIDOCS_MAX_BYTES + 1)).toBe(false);
  });
});

// The PUT route follows the admin.ts store-injection pattern, so the
// authorization ladder is testable without D1/R2.
interface Harness {
  deps: ApidocsDependencies;
  store: {
    packageOwner: ReturnType<typeof vi.fn<ApidocsStore["packageOwner"]>>;
    versionExists: ReturnType<typeof vi.fn<ApidocsStore["versionExists"]>>;
    put: ReturnType<typeof vi.fn<ApidocsStore["put"]>>;
    putStream: ReturnType<typeof vi.fn<ApidocsStore["putStream"]>>;
  };
  streamed: Uint8Array<ArrayBuffer>[];
}

function harness(
  account: Account | null = OWNER,
  facts: { owner?: number | null; exists?: boolean } = {},
  limits?: ApidocsLimits,
): Harness {
  const streamed: Uint8Array<ArrayBuffer>[] = [];
  const store = {
    packageOwner: vi
      .fn<ApidocsStore["packageOwner"]>()
      .mockResolvedValue(facts.owner === undefined ? OWNER.id : facts.owner),
    versionExists: vi
      .fn<ApidocsStore["versionExists"]>()
      .mockResolvedValue(facts.exists ?? true),
    put: vi.fn<ApidocsStore["put"]>().mockResolvedValue(undefined),
    putStream: vi
      .fn<ApidocsStore["putStream"]>()
      .mockImplementation(async (_name, _version, body) => {
        streamed.push(new Uint8Array(await new Response(body).arrayBuffer()));
      }),
  };
  return {
    store,
    streamed,
    deps: { tokenAccount: vi.fn().mockResolvedValue(account), store, limits },
  };
}

function putRequest(body: Uint8Array<ArrayBuffer> | string): Request {
  return new Request("https://registry.cohdl.org/packages/passive/1.0.0/docs", {
    method: "PUT",
    headers: {
      Authorization: "Bearer cohdl_token",
      "Content-Type": "application/json",
    },
    body,
  });
}

const STREAM_LIMITS: ApidocsLimits = {
  maxBytes: 2_048,
  bufferMaxBytes: 32,
  prefixMaxBytes: 512,
};

function canonicalDoc(name = "passive", version = "1.0.0"): Uint8Array<ArrayBuffer> {
  return new TextEncoder().encode(
    JSON.stringify({
      schema_version: 1,
      generator: "cohdl test",
      package: { name, version, root: "passive" },
      dependencies: [],
      items: [],
      impls: [],
      foreign: [],
    }),
  );
}

async function bodySha256(body: Uint8Array<ArrayBuffer>): Promise<string> {
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", body));
  return [...digest].map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

async function streamingRequest(
  body: Uint8Array<ArrayBuffer>,
  headers: Record<string, string> = {},
): Promise<Request> {
  return new Request("https://registry.cohdl.org/packages/passive/1.0.0/docs", {
    method: "PUT",
    headers: {
      Authorization: "Bearer cohdl_token",
      "Content-Type": "application/json",
      "Content-Length": String(body.byteLength),
      "X-CoHDL-Api-Docs-Schema": "1",
      "X-CoHDL-Api-Docs-SHA256": await bodySha256(body),
      ...headers,
    },
    body,
  });
}

async function invoke(h: Harness, body: Uint8Array<ArrayBuffer> | string = doc()) {
  const response = await handleApidocsPut(putRequest(body), "passive", "1.0.0", h.deps);
  return { response, body: (await response.json()) as Record<string, unknown> };
}

describe("handleApidocsPut", () => {
  it("requires a resolvable bearer token before touching the store", async () => {
    const h = harness(null);
    const { response } = await invoke(h);
    expect(response.status).toBe(401);
    expect(h.store.packageOwner).not.toHaveBeenCalled();
    expect(h.store.put).not.toHaveBeenCalled();
  });

  it("returns 404 for a package that was never published", async () => {
    const h = harness(OWNER, { owner: null });
    const { response, body } = await invoke(h);
    expect(response.status).toBe(404);
    expect(body.error).toContain("not published");
    expect(h.store.put).not.toHaveBeenCalled();
  });

  it("returns 403 for an account that does not own the package", async () => {
    const h = harness(OTHER);
    const { response, body } = await invoke(h);
    expect(response.status).toBe(403);
    expect(body.error).toBe("`passive` is owned by another account");
    expect(h.store.versionExists).not.toHaveBeenCalled();
    expect(h.store.put).not.toHaveBeenCalled();
  });

  it("returns 404 for a version that is not published", async () => {
    const h = harness(OWNER, { exists: false });
    const { response, body } = await invoke(h);
    expect(response.status).toBe(404);
    expect(body.error).toBe(
      "`passive 1.0.0` is not published — publish the version before uploading api docs",
    );
    expect(h.store.put).not.toHaveBeenCalled();
  });

  it("rejects an invalid body without storing anything", async () => {
    const h = harness();
    const { response, body } = await invoke(h, JSON.stringify({ schema_version: 2 }));
    expect(response.status).toBe(400);
    expect(body.error).toBe("api docs must declare `schema_version` 1");
    expect(h.store.put).not.toHaveBeenCalled();
  });

  it("rejects a declared oversized upload before buffering its body", async () => {
    const h = harness();
    const request = putRequest("{}");
    request.headers.set("Content-Length", String(APIDOCS_MAX_BYTES + 1));
    const response = await handleApidocsPut(request, "passive", "1.0.0", h.deps);
    expect(response.status).toBe(413);
    expect(await response.json()).toEqual({ error: "api docs exceed the 200 MB upload limit" });
    expect(h.store.put).not.toHaveBeenCalled();
    expect(h.store.putStream).not.toHaveBeenCalled();
  });

  it("streams a compact canonical document above the buffered threshold", async () => {
    const h = harness(OWNER, {}, STREAM_LIMITS);
    const uploaded = canonicalDoc();
    const sha256 = await bodySha256(uploaded);
    expect(uploaded.byteLength).toBeGreaterThan(STREAM_LIMITS.bufferMaxBytes);

    const response = await handleApidocsPut(
      await streamingRequest(uploaded),
      "passive",
      "1.0.0",
      h.deps,
    );
    expect(response.status).toBe(200);
    expect(await response.json()).toEqual({
      name: "passive",
      version: "1.0.0",
      size: uploaded.byteLength,
    });
    expect(h.store.put).not.toHaveBeenCalled();
    expect(h.store.putStream).toHaveBeenCalledTimes(1);
    const [name, version, _stream, size, suppliedSha256] = h.store.putStream.mock.calls[0];
    expect(name).toBe("passive");
    expect(version).toBe("1.0.0");
    expect(size).toBe(uploaded.byteLength);
    expect(suppliedSha256).toBe(sha256);
    expect(h.streamed).toHaveLength(1);
    expect([...h.streamed[0]]).toEqual([...uploaded]);
  });

  it("requires both streaming metadata headers", async () => {
    const uploaded = canonicalDoc();
    const missingSchema = harness(OWNER, {}, STREAM_LIMITS);
    const noSchema = await streamingRequest(uploaded, {
      "X-CoHDL-Api-Docs-Schema": "2",
    });
    let response = await handleApidocsPut(
      noSchema,
      "passive",
      "1.0.0",
      missingSchema.deps,
    );
    expect(response.status).toBe(400);
    expect(await response.json()).toEqual({
      error: "large api docs need `X-CoHDL-Api-Docs-Schema: 1`",
    });
    expect(missingSchema.store.putStream).not.toHaveBeenCalled();

    const badHash = harness(OWNER, {}, STREAM_LIMITS);
    response = await handleApidocsPut(
      await streamingRequest(uploaded, { "X-CoHDL-Api-Docs-SHA256": "A".repeat(64) }),
      "passive",
      "1.0.0",
      badHash.deps,
    );
    expect(response.status).toBe(400);
    expect(await response.json()).toEqual({
      error: "large api docs need `X-CoHDL-Api-Docs-SHA256` as 64 lowercase hex digits",
    });
    expect(badHash.store.putStream).not.toHaveBeenCalled();
  });

  it("validates the actual compact prefix rather than trusting its headers", async () => {
    const h = harness(OWNER, {}, STREAM_LIMITS);
    const wrongPackage = canonicalDoc("other");
    const response = await handleApidocsPut(
      await streamingRequest(wrongPackage),
      "passive",
      "1.0.0",
      h.deps,
    );
    expect(response.status).toBe(400);
    const body = (await response.json()) as { error: string };
    expect(body.error).toContain('`package.name = "other"`');
    expect(h.store.putStream).not.toHaveBeenCalled();
  });

  it("bounds canonical-prefix inspection independently of the total cap", async () => {
    const limits = { ...STREAM_LIMITS, prefixMaxBytes: 24 };
    const h = harness(OWNER, {}, limits);
    const uploaded = canonicalDoc();
    const response = await handleApidocsPut(
      await streamingRequest(uploaded),
      "passive",
      "1.0.0",
      h.deps,
    );
    expect(response.status).toBe(400);
    expect(await response.json()).toEqual({
      error: "large api docs must reach the canonical `items` opener within 24 bytes",
    });
    expect(h.store.putStream).not.toHaveBeenCalled();
  });

  it("does not buffer an undeclared document past the legacy threshold", async () => {
    const h = harness(OWNER, {}, STREAM_LIMITS);
    const response = await handleApidocsPut(
      putRequest(canonicalDoc()),
      "passive",
      "1.0.0",
      h.deps,
    );
    expect(response.status).toBe(413);
    expect(await response.json()).toEqual({
      error: "api docs larger than 32 bytes require streaming metadata",
    });
    expect(h.store.put).not.toHaveBeenCalled();
    expect(h.store.putStream).not.toHaveBeenCalled();
  });

  it("stores the exact uploaded bytes and reports name/version/size", async () => {
    const h = harness();
    const uploaded = doc();
    const { response, body } = await invoke(h, uploaded);
    expect(response.status).toBe(200);
    expect(response.headers.get("Content-Type")).toBe("application/json");
    expect(body).toEqual({ name: "passive", version: "1.0.0", size: uploaded.length });
    expect(h.store.put).toHaveBeenCalledTimes(1);
    const [name, version, stored, parts] = h.store.put.mock.calls[0];
    expect(name).toBe("passive");
    expect(version).toBe("1.0.0");
    expect([...stored]).toEqual([...uploaded]);
    expect(parts).toEqual([]);
  });

  it("passes only local public parts, with primary and alternate AVL data, to storage", async () => {
    const uploaded = new TextEncoder().encode(
      JSON.stringify({
        schema_version: 1,
        package: { name: "passive", version: "1.0.0" },
        items: [
          {
            fq: "passive::parts::C100N",
            name: "C100N",
            kind: "part",
            pub: true,
            part: {
              device: "passive::devices::MLCC",
              primary: { fields: [{ name: "mpn", value: "PRIMARY" }] },
              alts: [{ fields: [{ name: "mpn", value: "ALTERNATE" }] }],
            },
          },
          {
            fq: "passive::parts::PRIVATE",
            name: "PRIVATE",
            kind: "part",
            pub: false,
            part: { device: "passive::devices::MLCC", primary: { fields: [] } },
          },
        ],
        foreign: [
          {
            fq: "dep::parts::FOREIGN",
            name: "FOREIGN",
            kind: "part",
            pub: true,
            part: { device: "dep::Device", primary: { fields: [] } },
          },
        ],
      }),
    );
    const h = harness();
    expect((await invoke(h, uploaded)).response.status).toBe(200);
    const indexed = h.store.put.mock.calls[0][3];
    expect(indexed).toHaveLength(1);
    expect(indexed[0].fq).toBe("passive::parts::C100N");
    expect(JSON.parse(indexed[0].avl_json)).toEqual([
      { primary: true, manufacturer: null, mpn: "PRIMARY" },
      { primary: false, manufacturer: null, mpn: "ALTERNATE" },
    ]);
  });

  it("lets the owner replace an existing upload (last write wins)", async () => {
    // The handler never checks for a prior sidecar — a second PUT for the
    // same version validates and stores just like the first.
    const h = harness();
    expect((await invoke(h)).response.status).toBe(200);
    expect((await invoke(h)).response.status).toBe(200);
    expect(h.store.put).toHaveBeenCalledTimes(2);
  });
});

describe("part search D1 mutation plan", () => {
  const indexed = {
    package_name: "passive",
    package_version: "1.0.0",
    fq: "passive::parts::C100N",
    name: "C100N",
    device: "passive::devices::MLCC",
    intent: null,
    searchable: "C100N PRIMARY ALTERNATE",
    avl_json: "[]",
  };

  it("updates metadata and atomically guards delete and insert by newest version", () => {
    const key = "apidocs/sha256/hash.json";
    const mutations = partSearchMutations("passive", "1.0.0", 1234, key, [indexed]);
    expect(mutations).toHaveLength(3);
    expect(mutations[0].sql).toContain("api_docs_size = ?, api_docs_r2_key = ?");
    expect(mutations[0].bindings).toEqual([1234, key, "passive", "1.0.0"]);
    for (const mutation of mutations.slice(1)) {
      expect(mutation.sql).toContain("SELECT version FROM versions");
      expect(mutation.sql).toContain("ORDER BY published_at DESC, version DESC");
      expect(mutation.bindings.slice(-2)).toEqual(["1.0.0", "passive"]);
    }
    expect(mutations[1].sql).toContain("DELETE FROM part_search");
    expect(mutations[2].sql).toContain("INSERT INTO part_search");
    expect(mutations[2].sql).toContain("FROM json_each(?)");
    expect(JSON.parse(mutations[2].bindings[0] as string)).toEqual([indexed]);
  });

  it("still clears newest-version rows when a valid sidecar has no indexable parts", () => {
    const mutations = partSearchMutations(
      "passive",
      "2.0.0",
      99,
      "apidocs/sha256/hash.json",
      [],
    );
    expect(mutations).toHaveLength(2);
    expect(mutations[1].sql).toContain("DELETE FROM part_search");
  });
});
