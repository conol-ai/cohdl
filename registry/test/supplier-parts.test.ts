import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Env } from "../src/worker/auth";
import worker from "../src/worker/index";
import { supplierPartsStore } from "../src/worker/supplier-parts";
import { supplierStore, type CatalogRow } from "../src/worker/suppliers";

interface BoundStatement { sql: string; values: unknown[] }
function sqliteD1(db: DatabaseSync, statements: BoundStatement[] = []): D1Database {
  return {
    prepare(sql: string) {
      return { bind: (...values: unknown[]) => {
        const statement = { sql, values };
        statements.push(statement);
        return Object.assign({
          async all() { return { results: db.prepare(sql).all(...values as never[]) }; },
          async first() { return db.prepare(sql).get(...values as never[]) ?? null; },
          async run() { db.prepare(sql).run(...values as never[]); return { meta: {} }; },
        }, statement);
      } };
    },
    async batch(batch: BoundStatement[]) {
      db.exec("BEGIN");
      try {
        for (const { sql, values } of batch) db.prepare(sql).run(...values as never[]);
        db.exec("COMMIT");
      } catch (error) {
        db.exec("ROLLBACK");
        throw error;
      }
      return [];
    },
  } as unknown as D1Database;
}

const classes = ["resistor", "capacitor", "connector"];
const base: Omit<CatalogRow, "pn" | "mpn" | "mpn_key" | "manufacturer_key"> = {
  manufacturer: "Example", package: "0402", class: "resistor", stock: 0,
  links: [{ label: "Product", url: "https://example.com/part" }],
  fetched_at: "2026-10-27T00:00:00.000Z",
};
function catalogRow(pn: string, mpn: string, overrides: Partial<CatalogRow> = {}): CatalogRow {
  return { ...base, pn, mpn, mpn_key: mpn.toLowerCase().replace(/\s/g, ""),
    manufacturer_key: "example", ...overrides };
}
function snapshot(rows: CatalogRow[]) {
  return { supplier: "acme", display_name: "Acme", product_url_template: "https://example.com/{pn}",
    class_vocabulary: classes, rows };
}

let db: DatabaseSync;
let d1: D1Database;
let statements: BoundStatement[];
let env: Env;
beforeEach(async () => {
  db = new DatabaseSync(":memory:");
  db.exec(readFileSync(new URL("../migrations/0005-suppliers.sql", import.meta.url), "utf8"));
  statements = [];
  d1 = sqliteD1(db, statements);
  env = { DB: d1, ASSETS: { fetch: vi.fn(async () => new Response("asset")) } } as unknown as Env;
  await supplierStore(d1).replaceSnapshot(snapshot([
    catalogRow("P-1", "M 100", { stock: 0 }),
    catalogRow("P-2", "m100", { class: "capacitor", stock: 3 }),
    catalogRow("P-3", "M-100", { stock: 2 }),
    catalogRow("P-4", "M100", { stock: 7 }),
    catalogRow("P-5", "m 100", { class: "capacitor", stock: 7, fetched_at: "2026-10-28T00:00:00.000Z" }),
    catalogRow("P-6", "M100", { class: "connector", stock: 999 }),
  ]));
  statements.length = 0;
});
async function fetchParts(path: string, method = "GET") {
  const request = new Request(`https://registry.cohdl.org${path}`, { method });
  const response = await worker.fetch(request, env);
  return { response, body: await response.json() as any };
}

describe("public supplier parts Worker route", () => {
  it("serves login-free PN lookups in exact input order with unknown and snapshot date", async () => {
    const { response, body } = await fetchParts("/suppliers/acme/parts?pn=P-4,missing,P-1");
    expect(response.status).toBe(200);
    expect(body).toEqual({ supplier: "acme", snapshot_date: "2026-10-28T00:00:00.000Z", parts: [
      { pn: "P-4", catalog: expect.objectContaining({ pn: "P-4", mpn: "M100", stock: 7, links: base.links }) },
      { pn: "missing", catalog: "unknown" },
      { pn: "P-1", catalog: expect.objectContaining({ pn: "P-1", mpn: "M 100", stock: 0 }) },
    ] });
    expect(body.parts[0].catalog).not.toHaveProperty("mpn_key");
    expect(body.parts[0].catalog).not.toHaveProperty("manufacturer_key");
    expect(env.ASSETS.fetch).not.toHaveBeenCalled();
    expect(statements[1].sql).toContain("pn IN (?, ?, ?)");
    expect(statements[1].values).toEqual(["acme", "P-4", "missing", "P-1"]);
    expect((await fetchParts("/suppliers/acme/parts?pn=p-4")).body.parts).toEqual([{ pn: "p-4", catalog: "unknown" }]);
  });

  it("normalizes MPN case/whitespace and ranks class, in-stock, and higher stock", async () => {
    const { response, body } = await fetchParts("/suppliers/acme/parts?mpn=%20m%20%20100%20,M-100");
    expect(response.status).toBe(200);
    expect(body.parts.map((part: any) => part.catalog.map((row: any) => row.pn))).toEqual([
      ["P-4", "P-1", "P-5", "P-2", "P-6"], ["P-3"],
    ]);
    expect(body.snapshot_date).toBe("2026-10-28T00:00:00.000Z");
    expect(body.parts[0].catalog.every((row: any) => typeof row.fetched_at === "string")).toBe(true);
    expect(statements[1].sql).toContain("mpn_key IN (?, ?)");
    expect(statements[1].values).toEqual(["acme", "m100", "m-100"]);
    expect((await fetchParts("/suppliers/acme/parts?mpn=unknown")).body).toEqual({
      supplier: "acme", snapshot_date: "2026-10-28T00:00:00.000Z", parts: [{ mpn: "unknown", catalog: [] }],
    });
  });

  it("accepts 100 values with bounded multi-query lookup and rejects 101", async () => {
    const values = Array.from({ length: 100 }, (_, index) => `PN-${index}`);
    const ok = await fetchParts(`/suppliers/acme/parts?pn=${values.join(",")}`);
    expect(ok.response.status).toBe(200);
    expect(ok.body.parts).toHaveLength(100);
    expect(statements.filter(({ sql }) => sql.includes("INSERT")).length).toBe(0);
    const lookups = statements.filter(({ sql }) => sql.includes("SELECT pn"));
    expect(lookups.map(({ values }) => values.length)).toEqual([100, 2]);
    const tooMany = await fetchParts(`/suppliers/acme/parts?pn=${[...values, "extra"].join(",")}`);
    expect(tooMany.response.status).toBe(400);
  });

  it.each([
    "/suppliers/acme/parts",
    "/suppliers/acme/parts?pn=P-1&mpn=M100",
    "/suppliers/acme/parts?pn=P-1&pn=P-2",
    "/suppliers/acme/parts?pn=P-1&extra=x",
    "/suppliers/acme/parts?pn=P-1,P-1",
    "/suppliers/acme/parts?pn=P-1,",
    "/suppliers/acme/parts?pn=,P-1",
    "/suppliers/acme/parts?pn=P-1%00",
    "/suppliers/acme/parts?pn=P-1%0AP-2",
    "/suppliers/acme/parts?mpn=M100,%20%20",
    "/suppliers/acme/parts?mpn=M100,m%20100",
    "/suppliers/acme/parts?pn=%20P-1",
  ])("rejects duplicate, empty, non-canonical, or ambiguous query %s", async (path) => {
    expect((await fetchParts(path)).response.status).toBe(400);
  });

  it("validates supplier IDs, methods, and unknown suppliers", async () => {
    expect((await fetchParts("/suppliers/INVALID/parts?pn=P-1")).response.status).toBe(404);
    expect((await fetchParts("/suppliers/unknown/parts?pn=P-1")).response.status).toBe(404);
    const method = await fetchParts("/suppliers/acme/parts?pn=P-1", "POST");
    expect(method.response.status).toBe(405);
    expect(method.response.headers.get("Allow")).toBe("GET");
    const malformed = await fetchParts("/suppliers/acme/extra/parts?pn=P-1");
    expect(malformed.response.status).toBe(404);
    expect(malformed.body).toEqual({ error: "not found" });
    expect(env.ASSETS.fetch).not.toHaveBeenCalled();
  });

  it("returns a null snapshot date and unknown results for an empty catalog", async () => {
    await supplierStore(d1).replaceSnapshot(snapshot([]));
    const { response, body } = await fetchParts("/suppliers/acme/parts?pn=P-1");
    expect(response.status).toBe(200);
    expect(body).toEqual({ supplier: "acme", snapshot_date: null, parts: [{ pn: "P-1", catalog: "unknown" }] });
  });

  it("fails closed without exposing persistence errors", async () => {
    const broken = { DB: { prepare() { throw new Error("secret D1 failure"); } }, ASSETS: env.ASSETS } as unknown as Env;
    const response = await worker.fetch(new Request("https://registry.cohdl.org/suppliers/acme/parts?pn=P-1"), broken);
    expect(response.status).toBe(500);
    const text = await response.text();
    expect(JSON.parse(text)).toEqual({ error: "internal error" });
    expect(text).not.toContain("secret");
  });
});

describe("supplier parts real SQLite queries", () => {
  it("uses the primary key and normalized MPN indexes rather than temporary full scans", () => {
    for (const column of ["pn", "mpn_key"]) {
      const plan = db.prepare(`EXPLAIN QUERY PLAN SELECT pn FROM supplier_catalog WHERE supplier = ? AND ${column} = ?`).all() as { detail: string }[];
      expect(plan.some(({ detail }) => detail.includes("USING INDEX") || detail.includes("USING COVERING INDEX") ||
        detail.includes("USING PRIMARY KEY"))).toBe(true);
      expect(plan.some(({ detail }) => detail.includes("SCAN supplier_catalog"))).toBe(false);
    }
  });

  it("executes chunked snapshot replacement and reads back the real catalog", async () => {
    const rows = Array.from({ length: 19 }, (_, index) => catalogRow(`NEW-${index}`, `M-${index}`));
    await supplierStore(d1).replaceSnapshot(snapshot(rows));
    const result = await supplierPartsStore(d1).byPns("acme", ["NEW-18", "P-1"]);
    expect(result.map(({ pn }) => pn)).toEqual(["NEW-18"]);
    expect(db.prepare("SELECT COUNT(*) AS count FROM supplier_catalog").get()).toEqual({ count: 19 });
  });

  it("rolls back every chunked insert when the single batch fails", async () => {
    db.exec(`CREATE TRIGGER reject_bad_pn BEFORE INSERT ON supplier_catalog
      WHEN NEW.pn = 'FAIL' BEGIN SELECT RAISE(ABORT, 'rejected'); END`);
    const rows = Array.from({ length: 10 }, (_, index) => catalogRow(`NEW-${index}`, `M-${index}`));
    rows.push(catalogRow("FAIL", "M-FAIL"));
    await expect(supplierStore(d1).replaceSnapshot(snapshot(rows))).rejects.toThrow("rejected");
    expect(db.prepare("SELECT COUNT(*) AS count FROM supplier_catalog").get()).toEqual({ count: 6 });
    expect((await supplierPartsStore(d1).byPns("acme", ["P-1", "NEW-0"])).map(({ pn }) => pn)).toEqual(["P-1"]);
  });
});
