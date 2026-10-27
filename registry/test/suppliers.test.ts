import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { describe, expect, it, vi } from "vitest";
import type { Account } from "../src/worker/auth";
import {
  handleSupplierAdmin,
  normalizeManufacturer,
  parseSnapshot,
  type SupplierAdminDependencies,
  type SupplierAdminStore,
} from "../src/worker/suppliers";

const OFFICIAL: Account = { id: 1, email: "official@example.com", is_official: 1 };
const MEMBER: Account = { id: 2, email: "member@example.com", is_official: 0 };
const ROW = {
  pn: "P-001",
  mpn: "M-100",
  manufacturer: "Example Semiconductor",
  package: "0402",
  class: "capacitor",
  stock: 12,
  links: [{ label: "Product", url: "https://parts.example.com/P-001" }],
  fetched_at: "2026-10-28T00:00:00.000Z",
};

function snapshot(parts: unknown[] = [ROW]) {
  return {
    display_name: "Example Parts",
    product_url_template: "https://parts.example.com/{pn}",
    classes: ["capacitor", "resistor"],
    parts,
  };
}

function harness(actor: Account | null = OFFICIAL) {
  const store = {
    replaceSnapshot: vi.fn<SupplierAdminStore["replaceSnapshot"]>().mockResolvedValue(),
    putAlias: vi.fn<SupplierAdminStore["putAlias"]>().mockResolvedValue(),
    deleteAlias: vi.fn<SupplierAdminStore["deleteAlias"]>().mockResolvedValue(true),
  };
  const deps: SupplierAdminDependencies = {
    sessionAccount: vi.fn().mockResolvedValue(actor),
    store,
    log: vi.fn(),
    now: () => "2026-10-28T01:00:00.000Z",
  };
  return { deps, store };
}

function request(
  path: string,
  body: unknown = snapshot(),
  options: {
    method?: string;
    actor?: Account | null;
    origin?: string | null;
    contentType?: string | null;
  } = {},
) {
  const method = options.method ?? "PUT";
  const headers = new Headers();
  if (options.origin !== null) headers.set("Origin", options.origin ?? "https://registry.cohdl.org");
  if (options.contentType !== null) headers.set("Content-Type", options.contentType ?? "application/json");
  return new Request(`https://registry.cohdl.org${path}`, {
    method,
    headers,
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

async function invoke(req: Request, h: ReturnType<typeof harness>) {
  const response = await handleSupplierAdmin(req, new URL(req.url), h.deps);
  expect(response.headers.get("Cache-Control")).toBe("no-store");
  return { response, body: await response.json() as any };
}

describe("0005 suppliers migration", () => {
  it("creates the catalog, normalized lookup keys, and aliases in both schema entry points", () => {
    for (const file of ["migrations/0005-suppliers.sql", "schema.sql"]) {
      const db = new DatabaseSync(":memory:");
      db.exec(readFileSync(new URL(`../${file}`, import.meta.url), "utf8"));
      if (file === "schema.sql") {
        db.exec("INSERT INTO accounts (email, password_hash, created_at) VALUES ('a@example.com', 'x', 'now')");
        db.exec("INSERT INTO packages (name, tier, owner_account, created_at) VALUES ('@acme/x', 'brand', 1, 'now')");
        db.exec("INSERT INTO part_search (package_name, package_version, fq, name, searchable) VALUES ('@acme/x', '1.0.0', 'x', 'x', 'x')");
      }
      db.exec("INSERT INTO suppliers VALUES ('acme', 'Acme', 'https://parts.example.com/{pn}', '[\"capacitor\"]')");
      db.prepare("INSERT INTO supplier_catalog (supplier, pn, mpn, mpn_key, manufacturer_key, manufacturer, package, \"class\", stock, links, fetched_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").run(
        "acme", "P-001", "M 100", "m 100", "example semiconductor", "Example Semiconductor", "0402", "capacitor", 3, "[]", ROW.fetched_at,
      );
      db.prepare("INSERT INTO manufacturer_aliases VALUES (?, ?, ?)").run("example semis", "Example Semis", "Example Semiconductor");
      expect(db.prepare("SELECT mpn_key, manufacturer_key, stock FROM supplier_catalog").get()).toEqual({
        mpn_key: "m 100", manufacturer_key: "example semiconductor", stock: 3,
      });
      expect(db.prepare("SELECT canonical FROM manufacturer_aliases WHERE alias_key = ?").get("example semis")).toEqual({ canonical: "Example Semiconductor" });
      expect(() => db.prepare("INSERT INTO supplier_catalog VALUES ('acme', 'P-001', 'x', 'x', 'x', 'x', 'x', 'capacitor', 1, '[]', ?)").run(ROW.fetched_at)).toThrow();
      expect(() => db.prepare("INSERT INTO supplier_catalog VALUES ('acme', 'P-002', 'x', 'x', 'x', 'x', 'x', 'capacitor', -1, '[]', ?)").run(ROW.fetched_at)).toThrow();
      db.close();
    }
  });
});

describe("snapshot parsing", () => {
  it("keeps part numbers exact, derives normalized keys, and permits an empty snapshot", () => {
    const parsed = parseSnapshot(snapshot(), "acme");
    expect(parsed.ok).toBe(true);
    if (!parsed.ok) return;
    expect(parsed.value.supplier).toBe("acme");
    expect(parsed.value.rows[0]).toMatchObject({ pn: "P-001", mpn_key: "m-100", manufacturer_key: "example semiconductor" });
    expect(parseSnapshot(snapshot([]), "acme")).toMatchObject({ ok: true, value: { supplier: "acme", rows: [] } });
  });

  it.each([
    ["duplicate PN", snapshot([ROW, ROW])],
    ["case-distinct PN remains valid", snapshot([ROW, { ...ROW, pn: "p-001" }])],
    ["unknown class", snapshot([{ ...ROW, class: "connector" }])],
    ["negative stock", snapshot([{ ...ROW, stock: -1 }])],
    ["fractional stock", snapshot([{ ...ROW, stock: 1.5 }])],
    ["unsafe stock", snapshot([{ ...ROW, stock: Number.MAX_SAFE_INTEGER + 1 }])],
    ["invalid timestamp", snapshot([{ ...ROW, fetched_at: "2026-02-30T00:00:00.000Z" }])],
    ["non-UTC timestamp", snapshot([{ ...ROW, fetched_at: "2026-10-28T00:00:00+00:00" }])],
    ["javascript link", snapshot([{ ...ROW, links: [{ label: "bad", url: "javascript:alert(1)" }] }])],
    ["credential link", snapshot([{ ...ROW, links: [{ label: "bad", url: "https://user:pass@example.com/" }] }])],
    ["extra row field", snapshot([{ ...ROW, extra: true }])],
    ["control character", snapshot([{ ...ROW, manufacturer: "Bad\u0000Name" }])],
    ["blank PN", snapshot([{ ...ROW, pn: "  " }])],
  ])("rejects or precisely accepts %s", (_name, value) => {
    const result = parseSnapshot(value, "acme");
    if (_name === "case-distinct PN remains valid") expect(result.ok).toBe(true);
    else expect(result.ok).toBe(false);
  });

  it("requires a bounded HTTP(S) product template with a PN placeholder", () => {
    expect(parseSnapshot({ ...snapshot(), product_url_template: "https://parts.example.com/product" }, "acme").ok).toBe(false);
    expect(parseSnapshot({ ...snapshot(), product_url_template: "file:///tmp/{pn}" }, "acme").ok).toBe(false);
    expect(parseSnapshot({ ...snapshot([]), classes: [] }, "acme").ok).toBe(false);
    expect(parseSnapshot(snapshot(), "Bad-ID").ok).toBe(false);
  });
});

describe("D1 supplier store", () => {
  it("replaces each snapshot in one atomic batch, including empty snapshots", async () => {
    const { supplierStore } = await import("../src/worker/suppliers");
    const statements: { sql: string; values: unknown[] }[] = [];
    const batch = vi.fn().mockResolvedValue([]);
    const db = {
      prepare: (sql: string) => ({ bind: (...values: unknown[]) => {
        statements.push({ sql, values });
        return { sql, values };
      } }),
      batch,
    } as unknown as D1Database;
    const parsed = parseSnapshot(snapshot(), "acme");
    expect(parsed.ok).toBe(true);
    if (!parsed.ok) return;
    await supplierStore(db).replaceSnapshot(parsed.value);
    expect(batch).toHaveBeenCalledTimes(1);
    expect(statements.map(({ sql }) => sql)).toHaveLength(3);
    expect(statements[0].sql).toContain("ON CONFLICT(id) DO UPDATE");
    expect(statements[1]).toEqual({ sql: "DELETE FROM supplier_catalog WHERE supplier = ?", values: ["acme"] });
    expect(statements[2].values).toContain("P-001");
    statements.length = 0;
    batch.mockClear();
    const empty = parseSnapshot(snapshot([]), "acme");
    expect(empty.ok).toBe(true);
    if (!empty.ok) return;
    await supplierStore(db).replaceSnapshot(empty.value);
    expect(batch).toHaveBeenCalledTimes(1);
    expect(statements).toHaveLength(2);
  });
});

describe("official supplier administration", () => {
  it("requires a web-session official account before parsing or mutating", async () => {
    for (const [actor, status] of [[null, 401], [MEMBER, 403]] as const) {
      const h = harness(actor);
      const { response } = await invoke(request("/api/admin/suppliers/acme/snapshot"), h);
      expect(response.status).toBe(status);
      expect(h.store.replaceSnapshot).not.toHaveBeenCalled();
      expect(h.deps.sessionAccount).toHaveBeenCalled();
    }
  });

  it("enforces same-origin JSON, methods, and bounded bodies", async () => {
    const h = harness();
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", snapshot(), { origin: "https://evil.example" }), h)).response.status).toBe(403);
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", snapshot(), { contentType: "text/plain" }), h)).response.status).toBe(415);
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", snapshot(), { method: "POST" }), h)).response.status).toBe(405);
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", "{"), h)).response.status).toBe(400);
    const oversized = `"${"x".repeat(512 * 1024)}"`;
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", oversized), h)).response.status).toBe(413);
    expect(h.store.replaceSnapshot).not.toHaveBeenCalled();
  });

  it("validates a complete snapshot before replacing the supplier exactly once", async () => {
    const h = harness();
    const valid = await invoke(request("/api/admin/suppliers/acme/snapshot"), h);
    expect(valid.response.status).toBe(200);
    expect(valid.body).toEqual({ ok: true, supplier: "acme", rows: 1 });
    expect(h.store.replaceSnapshot).toHaveBeenCalledTimes(1);
    const saved = h.store.replaceSnapshot.mock.calls[0][0];
    expect(saved.supplier).toBe("acme");
    h.store.replaceSnapshot.mockClear();
    const invalid = await invoke(request("/api/admin/suppliers/acme/snapshot", snapshot([{ ...ROW, stock: -1 }])), h);
    expect(invalid.response.status).toBe(400);
    expect(h.store.replaceSnapshot).not.toHaveBeenCalled();
  });

  it("allows an empty snapshot and fails closed when persistence fails", async () => {
    const h = harness();
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot", snapshot([])), h)).response.status).toBe(200);
    expect(h.store.replaceSnapshot).toHaveBeenCalledWith(expect.objectContaining({ supplier: "acme", rows: [] }));
    h.store.replaceSnapshot.mockRejectedValueOnce(new Error("batch failed"));
    expect((await invoke(request("/api/admin/suppliers/acme/snapshot"), h)).response.status).toBe(500);
  });

  it("puts and deletes aliases by their Unicode case/whitespace-normalized key", async () => {
    const h = harness();
    const put = await invoke(request("/api/admin/manufacturer-aliases/Example%20%20SEMIS", { canonical: "Example Semiconductor" }), h);
    expect(put.response.status).toBe(200);
    expect(put.body.alias_key).toBe("example semis");
    expect(h.store.putAlias).toHaveBeenCalledWith("example semis", "Example  SEMIS", "Example Semiconductor");
    const del = await invoke(request("/api/admin/manufacturer-aliases/example%20semis", {}, { method: "DELETE" }), h);
    expect(del.body).toEqual({ ok: true, deleted: true });
    expect(h.store.deleteAlias).toHaveBeenCalledWith("example semis");
    expect(normalizeManufacturer("  EXAMPLE\t Semiconductor  ")).toBe("example semiconductor");
  });

  it("rejects malformed aliases without a mutation", async () => {
    const h = harness();
    expect((await invoke(request("/api/admin/manufacturer-aliases/acme", {}), h)).response.status).toBe(400);
    expect((await invoke(request("/api/admin/manufacturer-aliases/%20", { canonical: "Acme" }), h)).response.status).toBe(400);
    expect((await invoke(request("/api/admin/manufacturer-aliases/acme", {}, { method: "POST" }), h)).response.status).toBe(405);
    expect(h.store.putAlias).not.toHaveBeenCalled();
    expect(h.store.deleteAlias).not.toHaveBeenCalled();
  });
});
