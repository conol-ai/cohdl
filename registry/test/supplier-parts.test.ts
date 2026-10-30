import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Env } from "../src/worker/auth";
import worker from "../src/worker/index";
import { manufacturerKeyStore } from "../src/worker/manufacturers";
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
    expect(body.parts[0].catalog.mpn_key).toBe("m100");
    expect(body.parts[0].catalog.manufacturer_key).toBe("example");
    expect(env.ASSETS.fetch).not.toHaveBeenCalled();
    const lookup = statements.find(({ sql }) => sql.includes("pn IN"));
    expect(lookup?.sql).toContain("pn IN (?, ?, ?)");
    expect(lookup?.values).toEqual(["acme", "P-4", "missing", "P-1"]);
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
    const lookup = statements.find(({ sql }) => sql.includes("mpn_key IN"));
    expect(lookup?.sql).toContain("mpn_key IN (?, ?)");
    expect(lookup?.values).toEqual(["acme", "m100", "m-100"]);
    expect((await fetchParts("/suppliers/acme/parts?mpn=unknown")).body).toEqual({
      supplier: "acme", snapshot_date: "2026-10-28T00:00:00.000Z", parts: [{ mpn: "unknown", catalog: [] }],
    });
  });

  it("returns duplicate PN and normalized-duplicate MPN inputs independently in order", async () => {
    const pn = await fetchParts("/suppliers/acme/parts?pn=P-1,P-1,missing,P-1");
    expect(pn.response.status).toBe(200);
    expect(pn.body.parts.map((part: any) => [part.pn, part.catalog === "unknown" ? "unknown" : part.catalog.pn])).toEqual([
      ["P-1", "P-1"], ["P-1", "P-1"], ["missing", "unknown"], ["P-1", "P-1"],
    ]);

    const mpn = await fetchParts("/suppliers/acme/parts?mpn=M100,m%20100,M100,M-100");
    expect(mpn.response.status).toBe(200);
    expect(mpn.body.parts.map((part: any) => [part.mpn, part.catalog.map((row: any) => row.pn)])).toEqual([
      ["M100", ["P-4", "P-1", "P-5", "P-2", "P-6"]],
      ["m 100", ["P-4", "P-1", "P-5", "P-2", "P-6"]],
      ["M100", ["P-4", "P-1", "P-5", "P-2", "P-6"]],
      ["M-100", ["P-3"]],
    ]);
  });

  it("queries non-NFC, untrimmed, and Cf-containing PNs exactly", async () => {
    const nonNfc = "P-E\u0301";
    const untrimmed = " P-1 ";
    const withCf = "P-\u200f1";
    const nfc = "P-É";
    const values = [nonNfc, untrimmed, withCf, nfc];
    const missing = new URLSearchParams({ pn: values.join(",") });
    const unknown = await fetchParts(`/suppliers/acme/parts?${missing}`);
    expect(unknown.response.status).toBe(200);
    expect(unknown.body.parts).toEqual(values.map((pn) => ({ pn, catalog: "unknown" })));

    await supplierStore(d1).replaceSnapshot(snapshot([
      catalogRow(nonNfc, "M-nonNfc"),
      catalogRow(untrimmed, "M-untrimmed"),
      catalogRow(withCf, "M-cf"),
      catalogRow(nfc, "M-nfc"),
    ]));
    const parameters = new URLSearchParams({ pn: [...values, nonNfc].join(",") });
    const exact = await fetchParts(`/suppliers/acme/parts?${parameters}`);
    expect(exact.response.status).toBe(200);
    expect(exact.body.parts.map((part: any) => [part.pn, part.catalog === "unknown" ? "unknown" : part.catalog.pn])).toEqual([
      [nonNfc, nonNfc], [untrimmed, untrimmed], [withCf, withCf], [nfc, nfc], [nonNfc, nonNfc],
    ]);
  });

  it("normalizes non-NFC, untrimmed, and Cf-containing MPN inputs", async () => {
    await supplierStore(d1).replaceSnapshot(snapshot([
      catalogRow("P-unicode", "é100"),
      catalogRow("P-cf", "é\u200f100"),
    ]));
    const mpn = ["e\u0301 100", " E\u0301\u200f100 "];
    const parameters = new URLSearchParams({ mpn: mpn.join(",") });
    const { response, body } = await fetchParts(`/suppliers/acme/parts?${parameters}`);
    expect(response.status).toBe(200);
    expect(body.parts).toEqual([
      { mpn: mpn[0], catalog: [expect.objectContaining({ pn: "P-unicode", mpn_key: "é100" })] },
      { mpn: mpn[1], catalog: [expect.objectContaining({ pn: "P-cf", mpn_key: "é\u200f100" })] },
    ]);
  });

  it.each(["pn", "mpn"])("accepts 100 duplicate %s values and rejects 101", async (kind) => {
    const values = Array.from({ length: 100 }, () => kind === "pn" ? "P-1" : "M100");
    const ok = await fetchParts(`/suppliers/acme/parts?${kind}=${values.join(",")}`);
    expect(ok.response.status).toBe(200);
    expect(ok.body.parts).toHaveLength(100);
    expect(ok.body.parts.every((part: any) => part[kind] === values[0])).toBe(true);
    expect(statements.filter(({ sql }) => sql.includes("INSERT")).length).toBe(0);
    const lookups = statements.filter(({ sql }) => sql.includes(kind === "pn" ? "SELECT pn" : "mpn_key IN"));
    expect(lookups.map(({ values }) => values.length)).toEqual([100, 2]);
    const tooMany = await fetchParts(`/suppliers/acme/parts?${kind}=${[...values, values[0]].join(",")}`);
    expect(tooMany.response.status).toBe(400);
  });

  it.each([
    "/suppliers/acme/parts",
    "/suppliers/acme/parts?pn=P-1&mpn=M100",
    "/suppliers/acme/parts?pn=P-1&extra=x",
    "/suppliers/acme/parts?pn=P-1,",
    "/suppliers/acme/parts?pn=,P-1",
    "/suppliers/acme/parts?pn=P-1%00",
    "/suppliers/acme/parts?pn=P-1%0AP-2",
    "/suppliers/acme/parts?mpn=M100,%20%20",
  ])("rejects empty, control-character, or ambiguous query %s", async (path) => {
    expect((await fetchParts(path)).response.status).toBe(400);
  });

  it("rejects supplier query values over 256 UTF-8 bytes", async () => {
    const boundary = "é".repeat(128);
    for (const kind of ["pn", "mpn"]) {
      const accepted = await fetchParts(`/suppliers/acme/parts?${new URLSearchParams({ [kind]: boundary })}`);
      expect(accepted.response.status).toBe(200);
      expect(accepted.body.parts[0][kind]).toBe(boundary);
      expect((await fetchParts(`/suppliers/acme/parts?${new URLSearchParams({ [kind]: `${boundary}é` })}`)).response.status).toBe(400);
    }
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

  it("adds normalized MPN and alias-resolved manufacturer keys without changing catalog fields", async () => {
    await supplierStore(d1).putAlias("example semis", "Example Semis", "Éxample, Manufacturing");
    const row = catalogRow("P-alias", " É  100 ", {
      manufacturer: "Example Semis", package: "SOT-23", class: "connector", stock: 42,
      links: [{ label: "Datasheet", url: "https://example.com/datasheet" }],
      fetched_at: "2026-10-28T01:02:03.000Z",
    });
    await supplierStore(d1).replaceSnapshot(snapshot([row]));
    const { response, body } = await fetchParts("/suppliers/acme/parts?pn=P-alias");
    expect(response.status).toBe(200);
    expect(body).toEqual({ supplier: "acme", snapshot_date: row.fetched_at, parts: [{ pn: row.pn, catalog: {
      pn: row.pn, mpn: row.mpn, mpn_key: "é100", manufacturer: row.manufacturer,
      manufacturer_key: "éxample, manufacturing", package: row.package, class: row.class,
      stock: row.stock, links: row.links, fetched_at: row.fetched_at,
    } }] });
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

describe("public manufacturer normalization Worker route", () => {
  async function fetchNormalize(parameters: URLSearchParams | string, method = "GET") {
    const query = typeof parameters === "string" ? parameters : parameters.toString();
    const request = new Request(`https://registry.cohdl.org/manufacturers/normalize${query ? `?${query}` : ""}`, { method });
    const response = await worker.fetch(request, env);
    return { response, body: await response.json() as any };
  }

  it("maps aliases and canonical names in both directions with Unicode, whitespace, commas, order, and duplicates", async () => {
    await supplierStore(d1).putAlias("example semis", "Example Semis", "Éxample, Manufacturing");
    const parameters = new URLSearchParams();
    const mfr = [" EXAMPLE   Semis ", "ÉXAMPLE,  MANUFACTURING", "Example Semis", "Unknown   Manufacturer"];
    const mpn = [" É  100 ", "M-100"];
    for (const value of mfr) parameters.append("mfr", value);
    for (const value of mpn) parameters.append("mpn", value);
    const { response, body } = await fetchNormalize(parameters);
    expect(response.status).toBe(200);
    expect(response.headers.get("Cache-Control")).toBe("public, max-age=60");
    expect(body).toEqual({
      mfr: [
        { value: mfr[0], key: "éxample, manufacturing" },
        { value: mfr[1], key: "éxample, manufacturing" },
        { value: mfr[2], key: "éxample, manufacturing" },
        { value: mfr[3], key: "unknown manufacturer" },
      ],
      mpn: [{ value: mpn[0], key: "é100" }, { value: mpn[1], key: "m-100" }],
    });
    const aliases = statements.filter(({ sql }) => sql.includes("FROM manufacturer_aliases"));
    expect(aliases).toHaveLength(1);
    expect(aliases[0].sql).toContain("alias_key IN (?, ?, ?)");
    expect(aliases[0].values).toEqual(["example semis", "éxample, manufacturing", "unknown manufacturer"]);
    expect(env.ASSETS.fetch).not.toHaveBeenCalled();
  });

  it("returns an empty manufacturer array for MPN-only input without an alias query", async () => {
    const { response, body } = await fetchNormalize(new URLSearchParams([["mpn", " M 100 "]]));
    expect(response.status).toBe(200);
    expect(body).toEqual({ mfr: [], mpn: [{ value: " M 100 ", key: "m100" }] });
    expect(statements.filter(({ sql }) => sql.includes("manufacturer_aliases"))).toHaveLength(0);
  });

  it("normalizes duplicate, non-NFC, untrimmed, and Cf-containing values independently", async () => {
    await supplierStore(d1).putAlias("example semis", "Example Semis", "Éxample, Manufacturing");
    const mfr = [" EXAMPLE Semis ", " EXAMPLE Semis ", " E\u0301XAMPLE,\u200f MANUFACTURING "];
    const mpn = [" É 100 ", "e\u0301\u200f100", " É 100 "];
    const parameters = new URLSearchParams();
    for (const value of mfr) parameters.append("mfr", value);
    for (const value of mpn) parameters.append("mpn", value);
    const { response, body } = await fetchNormalize(parameters);
    expect(response.status).toBe(200);
    expect(body).toEqual({
      mfr: [
        { value: mfr[0], key: "éxample, manufacturing" },
        { value: mfr[1], key: "éxample, manufacturing" },
        { value: mfr[2], key: "éxample,\u200f manufacturing" },
      ],
      mpn: [
        { value: mpn[0], key: "é100" },
        { value: mpn[1], key: "é\u200f100" },
        { value: mpn[2], key: "é100" },
      ],
    });
  });

  it("enforces the per-side value limit and accepts exactly 256 UTF-8 bytes", async () => {
    const boundary = "é".repeat(128);
    const accepted = await fetchNormalize(new URLSearchParams([["mfr", boundary]]));
    expect(accepted.response.status).toBe(200);
    expect(accepted.body.mfr[0]).toEqual({ value: boundary, key: boundary });
    expect((await fetchNormalize(new URLSearchParams([["mfr", `${boundary}é`]]))).response.status).toBe(400);
    for (const kind of ["mfr", "mpn"]) {
      const values = Array.from({ length: 100 }, () => [`${kind}`, "duplicate"] as [string, string]);
      expect((await fetchNormalize(new URLSearchParams(values))).response.status).toBe(200);
      expect((await fetchNormalize(new URLSearchParams([...values, [kind, "duplicate"]]))).response.status).toBe(400);
    }
  });

  it.each([
    "", "mfr=", "mpn=", "mfr=%20%20", "mpn=%09", "mfr=ok&extra=x",
    "mfr=bad%00value", "mpn=bad%0Avalue",
  ])("rejects empty or invalid input %s", async (query) => {
    const { response, body } = await fetchNormalize(query);
    expect(response.status).toBe(400);
    expect(body).toEqual({ error: expect.any(String) });
  });

  it("rejects non-GET methods and fails closed without exposing persistence errors", async () => {
    const method = await fetchNormalize("mfr=Example", "POST");
    expect(method.response.status).toBe(405);
    expect(method.response.headers.get("Allow")).toBe("GET");
    const broken = { DB: { prepare() { throw new Error("secret alias failure"); } }, ASSETS: env.ASSETS } as unknown as Env;
    const response = await worker.fetch(new Request("https://registry.cohdl.org/manufacturers/normalize?mfr=Example"), broken);
    expect(response.status).toBe(500);
    expect(await response.json()).toEqual({ error: "internal error" });
  });

  it("chunks batched alias IN lookups at 100 bindings without N+1 queries", async () => {
    const keys = Array.from({ length: 101 }, (_, index) => `manufacturer-${index}`);
    const put = db.prepare("INSERT INTO manufacturer_aliases (alias_key, alias, canonical) VALUES (?, ?, ?)");
    for (const key of keys) put.run(key, key, `Canonical ${key}`);
    statements.length = 0;
    const resolved = await manufacturerKeyStore(d1).canonicalByAliasKey(keys);
    expect(resolved.get("manufacturer-0")).toBe("Canonical manufacturer-0");
    expect(resolved.get("manufacturer-100")).toBe("Canonical manufacturer-100");
    expect(statements.map(({ values }) => values.length)).toEqual([100, 1]);
    expect(statements.every(({ sql }) => sql.includes("alias_key IN"))).toBe(true);
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
