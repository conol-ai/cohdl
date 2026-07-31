import { describe, expect, it, vi } from "vitest";
import {
  brandFromBody,
  canonicalAccountId,
  handleAdminRequest,
  parseBrandPath,
  validBrand,
  type AdminAccount,
  type AdminDependencies,
  type AdminStore,
  type BrandMutation,
} from "../src/worker/admin";
import type { Account } from "../src/worker/auth";

const OFFICIAL: Account = { id: 1, email: "official@example.com", is_official: 1 };
const MEMBER: Account = { id: 2, email: "member@example.com", is_official: 0 };

interface Harness {
  deps: AdminDependencies;
  store: {
    listAccounts: ReturnType<typeof vi.fn<AdminStore["listAccounts"]>>;
    accountExists: ReturnType<typeof vi.fn<AdminStore["accountExists"]>>;
    grantBrand: ReturnType<typeof vi.fn<AdminStore["grantBrand"]>>;
    revokeBrand: ReturnType<typeof vi.fn<AdminStore["revokeBrand"]>>;
  };
  logs: Record<string, unknown>[];
}

function harness(
  actor: Account | null = OFFICIAL,
  mutations: {
    accounts?: AdminAccount[];
    truncated?: boolean;
    exists?: boolean;
    grant?: BrandMutation;
    revoke?: BrandMutation;
  } = {},
): Harness {
  const store = {
    listAccounts: vi.fn<AdminStore["listAccounts"]>().mockResolvedValue({
      accounts: mutations.accounts ?? [],
      truncated: mutations.truncated ?? false,
    }),
    accountExists: vi
      .fn<AdminStore["accountExists"]>()
      .mockResolvedValue(mutations.exists ?? true),
    grantBrand: vi
      .fn<AdminStore["grantBrand"]>()
      .mockResolvedValue(mutations.grant ?? { kind: "ok", changed: true }),
    revokeBrand: vi
      .fn<AdminStore["revokeBrand"]>()
      .mockResolvedValue(mutations.revoke ?? { kind: "ok", changed: true }),
  };
  const logs: Record<string, unknown>[] = [];
  return {
    store,
    logs,
    deps: {
      sessionAccount: vi.fn().mockResolvedValue(actor),
      store,
      log: (entry) => logs.push(entry),
      now: () => "2026-07-31T00:00:00.000Z",
    },
  };
}

function request(
  method: string,
  path: string,
  options: {
    body?: unknown;
    origin?: string | null;
    contentType?: string | null;
    authorization?: string;
    cookie?: string;
  } = {},
): Request {
  const headers = new Headers();
  if (options.origin !== null) {
    headers.set("Origin", options.origin ?? "https://registry.cohdl.org");
  }
  if (options.contentType !== null) {
    headers.set("Content-Type", options.contentType ?? "application/json");
  }
  if (options.authorization) headers.set("Authorization", options.authorization);
  if (options.cookie) headers.set("Cookie", options.cookie);
  return new Request(`https://registry.cohdl.org${path}`, {
    method,
    headers,
    body:
      method === "GET" || method === "HEAD"
        ? undefined
        : typeof options.body === "string"
          ? options.body
          : JSON.stringify(options.body ?? { brand: "espressif" }),
  });
}

async function invoke(req: Request, h: Harness): Promise<{ response: Response; body: any }> {
  const response = await handleAdminRequest(req, new URL(req.url), h.deps);
  expect(response.headers.get("Cache-Control")).toBe("no-store");
  return { response, body: await response.json() };
}

describe("admin request parsing", () => {
  it("accepts canonical positive safe account IDs only", () => {
    expect(canonicalAccountId("1")).toBe(1);
    expect(canonicalAccountId("9007199254740991")).toBe(Number.MAX_SAFE_INTEGER);
    for (const bad of ["", "0", "01", "-1", "+1", "1.0", "9007199254740992"]) {
      expect(canonicalAccountId(bad), bad).toBeNull();
    }
  });

  it("matches only the account-brand mutation path", () => {
    expect(parseBrandPath("/api/admin/accounts/42/brands")).toEqual({
      matched: true,
      accountId: 42,
    });
    expect(parseBrandPath("/api/admin/accounts/00/brands")).toEqual({
      matched: true,
      accountId: null,
    });
    expect(parseBrandPath("/api/admin/accounts/42")).toEqual({ matched: false });
  });

  it("uses the namespace segment grammar and reserves contrib", () => {
    for (const brand of ["espressif", "RaspberryPi", "acme-semiconductor", "acme_2"]) {
      expect(validBrand(brand), brand).toBe(true);
      expect(brandFromBody({ brand }), brand).toBe(brand);
    }
    for (const brand of ["", " contrib", "contrib", "acme ", "a/b", "@acme"]) {
      expect(validBrand(brand), brand).toBe(false);
      expect(brandFromBody({ brand }), brand).toBeNull();
    }
    expect(brandFromBody({ brand: "acme", extra: true })).toBeNull();
    expect(brandFromBody(["acme"])).toBeNull();
  });
});

describe("admin authorization and listing", () => {
  it("uses the web session only and returns 401 without one", async () => {
    const h = harness(null);
    const { response } = await invoke(
      request("GET", "/api/admin/accounts", {
        authorization: "Bearer cohdl_cli_token",
        cookie: "__Host-session=not-resolved-by-the-session-store",
      }),
      h,
    );
    expect(response.status).toBe(401);
    expect(h.store.listAccounts).not.toHaveBeenCalled();
  });

  it("returns 403 for a signed-in nonofficial account", async () => {
    const h = harness(MEMBER);
    const { response } = await invoke(request("GET", "/api/admin/accounts"), h);
    expect(response.status).toBe(403);
    expect(h.store.listAccounts).not.toHaveBeenCalled();
  });

  it("lists at most 100 sanitized account rows and passes the trimmed query", async () => {
    const accounts = Array.from({ length: 101 }, (_, index) => ({
      id: index + 1,
      email: `user${index + 1}@example.com`,
      official: index === 0,
      created_at: "2026-07-30T00:00:00.000Z",
      brands:
        index === 0
          ? [
              { brand: "espressif", verified: true },
              { brand: "legacy", verified: false },
            ]
          : [],
    }));
    const h = harness(OFFICIAL, { accounts });
    const { response, body } = await invoke(
      request("GET", "/api/admin/accounts?q=%20ESP%20"),
      h,
    );
    expect(response.status).toBe(200);
    expect(h.store.listAccounts).toHaveBeenCalledWith("ESP");
    expect(body.accounts).toHaveLength(100);
    expect(body.truncated).toBe(true);
    expect(body.accounts[0]).toEqual(accounts[0]);
    expect(JSON.stringify(body)).not.toMatch(/password|token_hash/i);
  });
});

describe("admin brand mutations", () => {
  it("rejects a noncanonical target before touching the store", async () => {
    const h = harness();
    const { response } = await invoke(
      request("PUT", "/api/admin/accounts/01/brands"),
      h,
    );
    expect(response.status).toBe(400);
    expect(h.store.accountExists).not.toHaveBeenCalled();
  });

  it("requires exact same-origin JSON writes", async () => {
    const badOrigin = harness();
    expect(
      (
        await invoke(
          request("PUT", "/api/admin/accounts/2/brands", {
            origin: "https://evil.example",
          }),
          badOrigin,
        )
      ).response.status,
    ).toBe(403);
    expect(badOrigin.store.accountExists).not.toHaveBeenCalled();

    const absentOrigin = harness();
    expect(
      (
        await invoke(
          request("PUT", "/api/admin/accounts/2/brands", { origin: null }),
          absentOrigin,
        )
      ).response.status,
    ).toBe(403);

    const wrongType = harness();
    expect(
      (
        await invoke(
          request("DELETE", "/api/admin/accounts/2/brands", {
            contentType: "text/plain",
          }),
          wrongType,
        )
      ).response.status,
    ).toBe(415);
    expect(wrongType.store.accountExists).not.toHaveBeenCalled();
  });

  it("rejects malformed JSON and invalid or nonexact brand bodies", async () => {
    const malformed = harness();
    expect(
      (
        await invoke(
          request("PUT", "/api/admin/accounts/2/brands", { body: "{" }),
          malformed,
        )
      ).response.status,
    ).toBe(400);

    for (const body of [
      {},
      { brand: "contrib" },
      { brand: " espressif" },
      { brand: "espressif", verified: true },
    ]) {
      const h = harness();
      expect(
        (await invoke(request("PUT", "/api/admin/accounts/2/brands", { body }), h)).response
          .status,
      ).toBe(400);
      expect(h.store.accountExists).not.toHaveBeenCalled();
    }
  });

  it("returns 404 when the target account does not exist", async () => {
    const h = harness(OFFICIAL, { exists: false });
    const { response } = await invoke(
      request("PUT", "/api/admin/accounts/99/brands"),
      h,
    );
    expect(response.status).toBe(404);
    expect(h.store.grantBrand).not.toHaveBeenCalled();
    expect(h.logs[0]).toMatchObject({
      event: "admin_brand_mutation",
      outcome: "target_not_found",
      target_account_id: 99,
    });
  });

  it("grants and idempotently re-grants a same-owner brand", async () => {
    const changed = harness();
    const first = await invoke(request("PUT", "/api/admin/accounts/2/brands"), changed);
    expect(first.response.status).toBe(200);
    expect(first.body).toEqual({
      account_id: 2,
      brand: "espressif",
      verified: true,
      changed: true,
    });
    expect(changed.store.grantBrand).toHaveBeenCalledWith(2, "espressif");

    const unchanged = harness(OFFICIAL, {
      grant: { kind: "ok", changed: false },
    });
    const second = await invoke(request("PUT", "/api/admin/accounts/2/brands"), unchanged);
    expect(second.response.status).toBe(200);
    expect(second.body.changed).toBe(false);
  });

  it("never transfers a brand owned by another account", async () => {
    const h = harness(OFFICIAL, {
      grant: { kind: "conflict", ownerAccountId: 7 },
    });
    const { response, body } = await invoke(
      request("PUT", "/api/admin/accounts/2/brands"),
      h,
    );
    expect(response.status).toBe(409);
    expect(body.owner_account_id).toBe(7);
    expect(h.logs[0]).toMatchObject({ outcome: "ownership_conflict" });
  });

  it("revokes without releasing ownership and makes repeats idempotent", async () => {
    const changed = harness();
    const first = await invoke(
      request("DELETE", "/api/admin/accounts/2/brands"),
      changed,
    );
    expect(first.body).toEqual({
      account_id: 2,
      brand: "espressif",
      verified: false,
      changed: true,
    });
    expect(changed.store.revokeBrand).toHaveBeenCalledWith(2, "espressif");

    const repeated = harness(OFFICIAL, {
      revoke: { kind: "ok", changed: false },
    });
    const second = await invoke(
      request("DELETE", "/api/admin/accounts/2/brands"),
      repeated,
    );
    expect(second.response.status).toBe(200);
    expect(second.body.changed).toBe(false);
  });

  it("logs structured mutation facts but no request credentials", async () => {
    const h = harness();
    await invoke(
      request("PUT", "/api/admin/accounts/2/brands", {
        authorization: "Bearer secret-cli-token",
        cookie: `__Host-session=${"a".repeat(64)}`,
      }),
      h,
    );
    expect(h.logs[0]).toMatchObject({
      event: "admin_brand_mutation",
      action: "grant",
      actor_account_id: 1,
      target_account_id: 2,
      brand: "espressif",
      outcome: "success",
      changed: true,
    });
    expect(JSON.stringify(h.logs)).not.toMatch(/secret-cli-token|session|cookie/i);
  });

  it("returns a no-store 500 without exposing store errors", async () => {
    const h = harness();
    h.store.accountExists.mockRejectedValue(new Error("sensitive database detail"));
    const { response, body } = await invoke(
      request("PUT", "/api/admin/accounts/2/brands"),
      h,
    );
    expect(response.status).toBe(500);
    expect(body).toEqual({ error: "internal error" });
    expect(JSON.stringify(body)).not.toContain("sensitive");
  });
});
