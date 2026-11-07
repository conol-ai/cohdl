import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { describe, expect, it } from "vitest";
import {
  ADMIN_TOKEN_TTL_MS,
  TOKEN_PREFIX,
  accountForToken,
  parseTokenRequest,
  sessionIdForRequest,
  sha256Hex,
  type Account,
  type Env,
} from "../src/worker/auth";

describe("web session cookie parsing", () => {
  const id = "a".repeat(64);

  it("extracts an exact lowercase hexadecimal session cookie", () => {
    const request = new Request("https://registry.cohdl.org/api/me", {
      headers: { Cookie: `theme=dark; __Host-session=${id}; preference=compact` },
    });
    expect(sessionIdForRequest(request)).toBe(id);
  });

  it("rejects missing, malformed, and overlong session identifiers", () => {
    for (const cookie of [
      "",
      "__Host-session=short",
      `__Host-session=${"A".repeat(64)}`,
      `__Host-session=${id}a`,
      `not-__Host-session=${id}`,
      `session=${id}`,
    ]) {
      const request = new Request("https://registry.cohdl.org/api/me", {
        headers: { Cookie: cookie },
      });
      expect(sessionIdForRequest(request), cookie).toBeNull();
    }
  });
});

describe("token scopes (migration 0006)", () => {
  const PRE_0006 = `CREATE TABLE accounts (id INTEGER PRIMARY KEY AUTOINCREMENT, email TEXT NOT NULL UNIQUE,
      password_hash TEXT NOT NULL, is_official INTEGER NOT NULL DEFAULT 0, created_at TEXT NOT NULL);
    CREATE TABLE tokens (token_hash TEXT PRIMARY KEY, account_id INTEGER NOT NULL REFERENCES accounts(id),
      created_at TEXT NOT NULL);`;
  const migration = readFileSync(new URL("../migrations/0006-token-scopes.sql", import.meta.url), "utf8");

  /// A D1-shaped wrapper over node:sqlite: prepare → bind → first.
  function d1(sqlite: DatabaseSync): Env {
    return {
      DB: {
        prepare: (sql: string) => ({
          bind: (...values: (string | number | null)[]) => ({
            first: async () => (sqlite.prepare(sql).get(...values) as unknown) ?? null,
          }),
        }),
      },
    } as unknown as Env;
  }
  function bearer(token: string): Request {
    return new Request("https://registry.cohdl.org/x", { headers: { Authorization: `Bearer ${token}` } });
  }
  async function seed(sqlite: DatabaseSync, token: string, account: number, scope?: string, expires?: string | null) {
    const hash = await sha256Hex(token);
    if (scope === undefined) {
      sqlite.prepare("INSERT INTO tokens (token_hash, account_id, created_at) VALUES (?, ?, 'now')").run(hash, account);
    } else {
      sqlite.prepare("INSERT INTO tokens (token_hash, account_id, created_at, scope, expires_at) VALUES (?, ?, 'now', ?, ?)")
        .run(hash, account, scope, expires ?? null);
    }
  }

  it("keeps every pre-0006 token a working publish token", async () => {
    const sqlite = new DatabaseSync(":memory:");
    sqlite.exec(PRE_0006);
    sqlite.exec("INSERT INTO accounts (email, password_hash, is_official, created_at) VALUES ('o@x', 'x', 1, 'now')");
    await seed(sqlite, "cohdl_old", 1);
    sqlite.exec(migration);
    expect(sqlite.prepare("SELECT scope, expires_at FROM tokens").get()).toEqual({ scope: "publish", expires_at: null });
    const env = d1(sqlite);
    expect(await accountForToken(env, bearer("cohdl_old"))).toMatchObject({ id: 1 });
    expect(await accountForToken(env, bearer("cohdl_old"), "admin")).toBeNull();
    expect(() => sqlite.exec("INSERT INTO tokens (token_hash, account_id, created_at, scope) VALUES ('h', 1, 'now', 'root')")).toThrow();
  });

  it("resolves a token only for its own scope and only before it expires", async () => {
    for (const schema of ["migrated", "schema.sql"]) {
      const sqlite = new DatabaseSync(":memory:");
      if (schema === "migrated") { sqlite.exec(PRE_0006); sqlite.exec(migration); } else {
        sqlite.exec(readFileSync(new URL("../schema.sql", import.meta.url), "utf8"));
      }
      sqlite.exec("INSERT INTO accounts (email, password_hash, is_official, created_at) VALUES ('o@x', 'x', 1, 'now')");
      await seed(sqlite, "cohdl_pub", 1, "publish", null);
      await seed(sqlite, "cohdl_admin_live", 1, "admin", "2026-12-01T00:00:00.000Z");
      await seed(sqlite, "cohdl_admin_dead", 1, "admin", "2026-11-01T00:00:00.000Z");
      const env = d1(sqlite);
      const now = new Date("2026-11-07T00:00:00.000Z");
      expect(await accountForToken(env, bearer("cohdl_pub"), "publish", now), schema).toMatchObject({ id: 1 });
      expect(await accountForToken(env, bearer("cohdl_pub"), "admin", now), schema).toBeNull();
      expect(await accountForToken(env, bearer("cohdl_admin_live"), "admin", now), schema).toMatchObject({ id: 1, is_official: 1 });
      expect(await accountForToken(env, bearer("cohdl_admin_live"), "publish", now), schema).toBeNull();
      expect(await accountForToken(env, bearer("cohdl_admin_dead"), "admin", now), schema).toBeNull();
      expect(await accountForToken(env, bearer("cohdl_admin_live"), "admin", new Date("2026-12-01T00:00:00.000Z")), schema).toBeNull();
      expect(await accountForToken(env, new Request("https://registry.cohdl.org/x"), "admin", now), schema).toBeNull();
    }
  });
});

describe("token requests", () => {
  const OFFICIAL: Account = { id: 1, email: "o@x", is_official: 1 };
  const MEMBER: Account = { id: 2, email: "m@x", is_official: 0 };

  it("mints publish tokens for anyone and admin tokens only for official accounts", () => {
    expect(parseTokenRequest({}, MEMBER)).toEqual({ ok: true, scope: "publish" });
    expect(parseTokenRequest({ scope: "publish" }, MEMBER)).toEqual({ ok: true, scope: "publish" });
    expect(parseTokenRequest({ scope: "admin" }, OFFICIAL)).toEqual({ ok: true, scope: "admin" });
    expect(parseTokenRequest({ scope: "admin" }, MEMBER)).toMatchObject({ ok: false, status: 403 });
  });

  it("refuses unknown scopes and extra fields", () => {
    for (const body of [{ scope: "root" }, { scope: "admin", ttl: 99 }, [], null, "admin", { scope: 1 }]) {
      expect(parseTokenRequest(body, OFFICIAL), JSON.stringify(body)).toMatchObject({ ok: false, status: 400 });
    }
  });

  it("gives each scope its own token prefix", () => {
    expect(TOKEN_PREFIX.publish).toBe("cohdl_");
    expect(TOKEN_PREFIX.admin).toBe("cohdl_admin_");
    expect(ADMIN_TOKEN_TTL_MS).toBe(30 * 24 * 3600 * 1000);
  });
});
