import { describe, expect, it, vi } from "vitest";
import {
  handleAdminComponentRequests,
  handlePublicComponentRequest,
  parseComponentRequestBody,
  readBoundedJson,
  type AdminComponentRequestDependencies,
  type ComponentRequestRow,
  type ComponentRequestStore,
  type PublicComponentRequestDependencies,
} from "../src/worker/component-requests";
import type { Account } from "../src/worker/auth";

const OFFICIAL: Account = { id: 1, email: "official@example.com", is_official: 1 };
const MEMBER: Account = { id: 2, email: "member@example.com", is_official: 0 };
const ROW: ComponentRequestRow = {
  id: 7,
  manufacturer: "Texas Instruments",
  part_number: "TPS63070RNMR",
  datasheet_url: "https://www.ti.com/lit/ds/symlink/tps63070.pdf",
  description: "Battery-powered board",
  status: "open",
  request_count: 3,
  created_at: "2026-08-24T00:00:00.000Z",
  last_requested_at: "2026-08-25T00:00:00.000Z",
  updated_at: "2026-08-25T00:00:00.000Z",
  resolved_at: null,
};

function store() {
  return {
    submit: vi.fn<ComponentRequestStore["submit"]>().mockResolvedValue({ duplicate: false }),
    list: vi.fn<ComponentRequestStore["list"]>().mockResolvedValue({
      requests: [ROW],
      truncated: false,
    }),
    setStatus: vi.fn<ComponentRequestStore["setStatus"]>().mockResolvedValue({
      request: { ...ROW, status: "resolved", resolved_at: "2026-08-25T01:00:00.000Z" },
      changed: true,
    }),
  };
}

function publicHarness(overrides: Partial<PublicComponentRequestDependencies> = {}) {
  const componentStore = store();
  const logs: Record<string, unknown>[] = [];
  const deps: PublicComponentRequestDependencies = {
    store: componentStore,
    rateLimit: vi.fn().mockResolvedValue(true),
    verifyHuman: vi.fn().mockResolvedValue("ok"),
    now: () => "2026-08-25T01:00:00.000Z",
    log: (entry) => logs.push(entry),
    ...overrides,
  };
  return { deps, store: componentStore, logs };
}

function adminHarness(
  actor: Account | null = OFFICIAL,
  overrides: Partial<AdminComponentRequestDependencies> = {},
) {
  const componentStore = store();
  const logs: Record<string, unknown>[] = [];
  const deps: AdminComponentRequestDependencies = {
    store: componentStore,
    sessionAccount: vi.fn().mockResolvedValue(actor),
    now: () => "2026-08-25T01:00:00.000Z",
    log: (entry) => logs.push(entry),
    ...overrides,
  };
  return { deps, store: componentStore, logs };
}

function publicRequest(
  body: unknown = {
    manufacturer: "Texas Instruments",
    part_number: "TPS63070RNMR",
    datasheet_url: "https://www.ti.com/lit/ds/symlink/tps63070.pdf",
    description: "Battery-powered board",
    recaptcha: "token",
  },
  options: { method?: string; origin?: string | null; contentType?: string | null } = {},
): Request {
  const method = options.method ?? "POST";
  const headers = new Headers();
  if (options.origin !== null) {
    headers.set("Origin", options.origin ?? "https://registry.cohdl.org");
  }
  if (options.contentType !== null) {
    headers.set("Content-Type", options.contentType ?? "application/json");
  }
  return new Request("https://registry.cohdl.org/api/component-requests", {
    method,
    headers,
    body:
      method === "GET" || method === "HEAD"
        ? undefined
        : typeof body === "string"
          ? body
          : JSON.stringify(body),
  });
}

function adminRequest(
  method: string,
  path: string,
  body: unknown = { status: "resolved" },
  options: { origin?: string | null; contentType?: string | null } = {},
): Request {
  const headers = new Headers();
  if (options.origin !== null) {
    headers.set("Origin", options.origin ?? "https://registry.cohdl.org");
  }
  if (options.contentType !== null) {
    headers.set("Content-Type", options.contentType ?? "application/json");
  }
  return new Request(`https://registry.cohdl.org${path}`, {
    method,
    headers,
    body: method === "GET" ? undefined : typeof body === "string" ? body : JSON.stringify(body),
  });
}

async function body(response: Response): Promise<any> {
  expect(response.headers.get("Cache-Control")).toBe("no-store");
  return response.json();
}

describe("component request validation", () => {
  it("normalizes display and canonical identity while preserving punctuation", () => {
    const parsed = parseComponentRequestBody({
      manufacturer: "  Acme\t Semiconductor  ",
      part_number: " TPS-123-A ",
      datasheet_url: "https://example.com/datasheet.pdf",
      description: "  first line\r\nsecond line  ",
    });
    expect(parsed).toEqual({
      ok: true,
      input: {
        manufacturer: "Acme Semiconductor",
        manufacturer_key: "acme semiconductor",
        part_number: "TPS-123-A",
        part_number_key: "tps-123-a",
        datasheet_url: "https://example.com/datasheet.pdf",
        description: "first line\nsecond line",
      },
    });
  });

  it("requires the three requested fields, HTTPS, exact keys, and bounded optional copy", () => {
    for (const candidate of [
      {},
      {
        manufacturer: "Acme",
        part_number: "A1",
        datasheet_url: "http://example.com/a.pdf",
      },
      {
        manufacturer: "Acme",
        part_number: "A1",
        datasheet_url: "https://user:pass@example.com/a.pdf",
      },
      {
        manufacturer: "Acme",
        part_number: "A1",
        datasheet_url: "https://example.com/a.pdf",
        extra: true,
      },
      {
        manufacturer: "Acme\u200f",
        part_number: "A1",
        datasheet_url: "https://example.com/a.pdf",
      },
      {
        manufacturer: "Acme",
        part_number: "A1",
        datasheet_url: "https://example.com/a.pdf",
        description: "x".repeat(2001),
      },
    ]) {
      expect(parseComponentRequestBody(candidate).ok, JSON.stringify(candidate)).toBe(false);
    }
  });

  it("streams and enforces the body cap even without Content-Length", async () => {
    const small = publicRequest({ value: "ok" });
    expect(await readBoundedJson(small, 100)).toEqual({ ok: true, value: { value: "ok" } });

    const large = publicRequest({ value: "x".repeat(200) });
    expect(await readBoundedJson(large, 100)).toEqual({ ok: false, reason: "too_large" });
  });
});

describe("public component request endpoint", () => {
  it("allows POST only and requires exact-origin JSON", async () => {
    for (const [request, status] of [
      [publicRequest(undefined, { method: "GET" }), 405],
      [publicRequest(undefined, { origin: null }), 403],
      [publicRequest(undefined, { origin: "https://evil.example" }), 403],
      [publicRequest(undefined, { contentType: "text/plain" }), 415],
    ] as const) {
      const h = publicHarness();
      const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
      expect(response.status).toBe(status);
      await body(response);
      expect(h.store.submit).not.toHaveBeenCalled();
    }
  });

  it("rate limits before parsing, verification, or D1", async () => {
    const h = publicHarness({ rateLimit: vi.fn().mockResolvedValue(false) });
    const request = publicRequest("{");
    const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
    expect(response.status).toBe(429);
    expect(response.headers.get("Retry-After")).toBe("60");
    expect(h.deps.verifyHuman).not.toHaveBeenCalled();
    expect(h.store.submit).not.toHaveBeenCalled();
  });

  it("returns field errors without invoking verification or D1", async () => {
    const h = publicHarness();
    const request = publicRequest({ manufacturer: "", part_number: "", datasheet_url: "no" });
    const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
    expect(response.status).toBe(400);
    expect(await body(response)).toMatchObject({
      fields: {
        manufacturer: expect.any(String),
        part_number: expect.any(String),
        datasheet_url: expect.any(String),
      },
    });
    expect(h.deps.verifyHuman).not.toHaveBeenCalled();
    expect(h.store.submit).not.toHaveBeenCalled();
  });

  it("fails closed when verification is unavailable or rejected", async () => {
    for (const [verification, status] of [
      ["unavailable", 503],
      ["failed", 403],
    ] as const) {
      const h = publicHarness({ verifyHuman: vi.fn().mockResolvedValue(verification) });
      const request = publicRequest();
      const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
      expect(response.status).toBe(status);
      expect(h.store.submit).not.toHaveBeenCalled();
    }
  });

  it("returns distinct new and duplicate success status without exposing an id", async () => {
    for (const [duplicate, status] of [
      [false, 202],
      [true, 200],
    ] as const) {
      const h = publicHarness();
      h.store.submit.mockResolvedValue({ duplicate });
      const request = publicRequest();
      const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
      expect(response.status).toBe(status);
      expect(await body(response)).toEqual({ ok: true, duplicate });
      expect(h.store.submit).toHaveBeenCalledWith(
        expect.objectContaining({
          manufacturer: "Texas Instruments",
          part_number: "TPS63070RNMR",
        }),
        "2026-08-25T01:00:00.000Z",
      );
      expect(h.logs).toEqual([
        expect.objectContaining({ event: "component_request_submitted", duplicate }),
      ]);
    }
  });

  it("returns a generic 500 and logs only the exception class", async () => {
    const h = publicHarness();
    h.store.submit.mockRejectedValue(new Error("secret D1 detail"));
    const request = publicRequest();
    const response = await handlePublicComponentRequest(request, new URL(request.url), h.deps);
    expect(response.status).toBe(500);
    expect(await body(response)).toEqual({ error: "internal error" });
    expect(JSON.stringify(h.logs)).not.toContain("secret D1 detail");
  });
});

describe("official component request administration", () => {
  it("requires an official web session", async () => {
    for (const [actor, status] of [
      [null, 401],
      [MEMBER, 403],
    ] as const) {
      const h = adminHarness(actor);
      const request = adminRequest("GET", "/api/admin/component-requests");
      const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
      expect(response.status).toBe(status);
      expect(h.store.list).not.toHaveBeenCalled();
    }
  });

  it("lists sanitized rows using bounded filters and sorting", async () => {
    const h = adminHarness();
    const request = adminRequest(
      "GET",
      "/api/admin/component-requests?status=resolved&sort=newest&q=%20TPS%20",
    );
    const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
    expect(response.status).toBe(200);
    expect(await body(response)).toEqual({ requests: [ROW], truncated: false });
    expect(h.store.list).toHaveBeenCalledWith("resolved", "newest", "tps");
  });

  it("rejects invalid list parameters before D1", async () => {
    for (const query of ["status=pending", "sort=popular", `q=${"x".repeat(300)}`]) {
      const h = adminHarness();
      const request = adminRequest("GET", `/api/admin/component-requests?${query}`);
      const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
      expect(response.status).toBe(400);
      expect(h.store.list).not.toHaveBeenCalled();
    }
  });

  it("requires a canonical id, exact-origin JSON, and an exact status body", async () => {
    for (const [request, status] of [
      [adminRequest("PUT", "/api/admin/component-requests/01"), 400],
      [adminRequest("PUT", "/api/admin/component-requests/7", undefined, { origin: null }), 403],
      [adminRequest("PUT", "/api/admin/component-requests/7", undefined, { contentType: "text/plain" }), 415],
      [adminRequest("PUT", "/api/admin/component-requests/7", { status: "open", extra: true }), 400],
      [adminRequest("PUT", "/api/admin/component-requests/7", { status: "pending" }), 400],
    ] as const) {
      const h = adminHarness();
      const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
      expect(response.status).toBe(status);
      expect(h.store.setStatus).not.toHaveBeenCalled();
    }
  });

  it("resolves idempotently and writes a structured audit event", async () => {
    const h = adminHarness();
    const request = adminRequest("PUT", "/api/admin/component-requests/7");
    const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
    expect(response.status).toBe(200);
    expect(await body(response)).toMatchObject({ changed: true, request: { id: 7, status: "resolved" } });
    expect(h.store.setStatus).toHaveBeenCalledWith(
      7,
      "resolved",
      OFFICIAL.id,
      "2026-08-25T01:00:00.000Z",
    );
    expect(h.logs).toEqual([
      expect.objectContaining({
        event: "component_request_status_changed",
        request_id: 7,
        previous_status: "open",
        status: "resolved",
        actor_account_id: 1,
      }),
    ]);
    expect(JSON.stringify(h.logs)).not.toContain("Battery-powered");
  });

  it("returns 404 for a missing request", async () => {
    const h = adminHarness();
    h.store.setStatus.mockResolvedValue(null);
    const request = adminRequest("PUT", "/api/admin/component-requests/999");
    const response = await handleAdminComponentRequests(request, new URL(request.url), h.deps);
    expect(response.status).toBe(404);
  });
});
