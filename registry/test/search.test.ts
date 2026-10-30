import { readFileSync } from "node:fs";
import { DatabaseSync } from "node:sqlite";
import { describe, expect, it, vi } from "vitest";
import type { Env } from "../src/worker/auth";
import worker from "../src/worker/index";
import {
  MAX_PART_SEARCH_INDEX_BYTES,
  MAX_PART_SEARCH_CHUNKS,
  PART_SEARCH_CHUNK_BYTES,
  extractPartSearchRows,
  handleSearch,
  parseSearchParams,
  partSearchChunks,
  quoteFtsPhrase,
  searchApi,
  type IndexedPartSearchRow,
  type PackageSearchRow,
  type SearchDependencies,
  type SearchStore,
} from "../src/worker/search";

interface BoundStatement { sql: string; values: unknown[] }
function sqliteD1(db: DatabaseSync, statements: BoundStatement[] = []): D1Database {
  return {
    prepare(sql: string) {
      return { bind: (...values: unknown[]) => {
        statements.push({ sql, values });
        return {
          async all() { return { results: db.prepare(sql).all(...values as never[]) }; },
          async first() { return db.prepare(sql).get(...values as never[]) ?? null; },
        };
      } };
    },
  } as unknown as D1Database;
}

function sqliteSearch(parts: IndexedPartSearchRow[] = []) {
  const db = new DatabaseSync(":memory:");
  db.exec(readFileSync(new URL("../schema.sql", import.meta.url), "utf8"));
  db.prepare("INSERT INTO accounts (email, password_hash, created_at) VALUES ('a@example.com', 'x', 'now')").run();
  db.prepare("INSERT INTO packages (name, tier, owner_account, created_at) VALUES ('p', 'official', 1, 'now')").run();
  db.prepare("INSERT INTO suppliers VALUES ('acme', 'Acme', 'https://example.com/products/{pn}', '[\"resistor\",\"capacitor\",\"connector\"]')").run();
  db.prepare("INSERT INTO suppliers VALUES ('globex', 'Globex', 'https://example.net/{pn}', '[\"resistor\"]')").run();
  for (const part of parts) {
    db.prepare("INSERT INTO part_search (package_name, package_version, fq, name, device, intent, searchable, avl_json) VALUES ('p', ?, ?, ?, ?, ?, ?, ?)")
      .run(part.version, part.fq, part.name, part.device, part.intent,
        `${part.fq}\n${JSON.parse(part.avl_json).flatMap((entry: any) =>
          [entry.manufacturer, entry.mpn, ...(entry.suppliers ?? []).map((b: any) => b.pn)]).join("\n")}`,
        part.avl_json);
  }
  const statements: BoundStatement[] = [];
  return { db, statements, d1: sqliteD1(db, statements) };
}

function catalogInsert(db: DatabaseSync, supplier: string, pn: string, category: string, stock = 3) {
  db.prepare("INSERT INTO supplier_catalog (supplier, pn, mpn, mpn_key, manufacturer_key, manufacturer, package, \"class\", stock, links, fetched_at) VALUES (?, ?, ?, ?, 'm', 'M', '0402', ?, ?, '[{\"label\":\"Datasheet\",\"url\":\"https://example.com/datasheet\"}]', '2026-10-28T01:02:03.000Z')")
    .run(supplier, pn, `MPN-${pn}`, `mpn-${pn}`.toLowerCase(), category, stock);
}

async function sqliteInvoke(d1: D1Database, path: string) {
  const req = request(path);
  const response = await searchApi({ DB: d1 } as Env, req, new URL(req.url));
  return { response, body: await response.json() as any };
}

interface Harness {
  deps: SearchDependencies;
  store: {
    searchPackages: ReturnType<typeof vi.fn<SearchStore["searchPackages"]>>;
    searchParts: ReturnType<typeof vi.fn<SearchStore["searchParts"]>>;
    catalog: ReturnType<typeof vi.fn<SearchStore["catalog"]>>;
  };
  logs: Record<string, unknown>[];
}

function harness(
  packages: PackageSearchRow[] = [],
  parts: IndexedPartSearchRow[] = [],
): Harness {
  const store = {
    searchPackages: vi.fn<SearchStore["searchPackages"]>().mockResolvedValue(packages),
    searchParts: vi.fn<SearchStore["searchParts"]>().mockResolvedValue(parts),
    catalog: vi.fn<SearchStore["catalog"]>().mockResolvedValue(new Map()),
  };
  const logs: Record<string, unknown>[] = [];
  return {
    store,
    logs,
    deps: { store, log: (entry) => logs.push(entry) },
  };
}

function request(path: string, method = "GET"): Request {
  return new Request(`https://registry.cohdl.org${path}`, { method });
}

async function invoke(path: string, h: Harness, method = "GET") {
  const req = request(path, method);
  const response = await handleSearch(req, new URL(req.url), h.deps);
  return { response, body: (await response.json()) as any };
}

function packageRow(index: number): PackageSearchRow {
  return {
    name: `pkg${index}`,
    tier: "official",
    latest: "1.0.0",
    description: index === 0 ? null : `Package ${index}`,
    updated: `2026-08-${String(index + 1).padStart(2, "0")}T00:00:00.000Z`,
  };
}

function partRow(
  avl: unknown = [
    { primary: true, manufacturer: "Yageo", mpn: "RC0402-primary" },
    { primary: false, manufacturer: "Vishay", mpn: "ALT-needle-42" },
  ],
): IndexedPartSearchRow {
  return {
    package: "passive",
    tier: "official",
    version: "1.2.0",
    fq: "passive::resistors::R_0402",
    name: "R_0402",
    device: "passive::devices::Resistor",
    intent: null,
    avl_json: typeof avl === "string" ? avl : JSON.stringify(avl),
  };
}

describe("stable search request parsing", () => {
  it("validates supplier/class filters without changing package-only behavior", async () => {
    expect(parseSearchParams(new URL("https://registry.cohdl.org/search?q=usb&supplier=acme&class=resistor&class=capacitor"))).toEqual({
      ok: true, value: { query: "usb", kind: "all", limit: 20, offset: 0,
        supplier: "acme", classes: ["resistor", "capacitor"] },
    });
    for (const path of [
      "/search?q=usb&class=resistor", "/search?q=usb&supplier=ACME",
      "/search?q=usb&supplier=acme&supplier=globex", "/search?q=usb&supplier=acme&class=",
      `/search?q=usb&supplier=acme&${Array.from({ length: 97 }, (_, i) => `class=c${i}`).join("&")}`,
    ]) expect(parseSearchParams(new URL(`https://registry.cohdl.org${path}`)).ok).toBe(false);
    const h = harness([packageRow(0)]);
    await invoke("/search?q=usb&kind=package&supplier=acme&class=resistor", h);
    expect(h.store.searchParts).not.toHaveBeenCalled();
  });
  it("trims the query and applies bounded defaults", () => {
    expect(parseSearchParams(new URL("https://registry.cohdl.org/search?q=%20stm32%20"))).toEqual({
      ok: true,
      value: { query: "stm32", kind: "all", limit: 20, offset: 0, supplier: null, classes: [] },
    });
  });

  it("uses the same Unicode whitespace and BOM trimming as the Rust CLI", () => {
    const query = `\uFEFF\u0085stm32\u2003\uFEFF`;
    expect(
      parseSearchParams(
        new URL(`https://registry.cohdl.org/search?q=${encodeURIComponent(query)}`),
      ),
    ).toEqual({
      ok: true,
      value: { query: "stm32", kind: "all", limit: 20, offset: 0, supplier: null, classes: [] },
    });
  });

  it("accepts all kinds and their numeric boundaries", () => {
    expect(
      parseSearchParams(
        new URL("https://registry.cohdl.org/search?q=usb&kind=part&limit=1&offset=10000"),
      ),
    ).toEqual({
      ok: true,
      value: { query: "usb", kind: "part", limit: 1, offset: 10_000, supplier: null, classes: [] },
    });
    expect(
      parseSearchParams(
        new URL("https://registry.cohdl.org/search?q=qfn&kind=package&limit=50&offset=0"),
      ).ok,
    ).toBe(true);
  });

  it("requires 3 Unicode scalars, not 3 UTF-16 code units", () => {
    for (const q of ["", "a", "ab", "😀😀"]) {
      const parsed = parseSearchParams(
        new URL(`https://registry.cohdl.org/search?q=${encodeURIComponent(q)}`),
      );
      expect(parsed.ok, q).toBe(false);
    }
    expect(
      parseSearchParams(
        new URL(`https://registry.cohdl.org/search?q=${encodeURIComponent("😀😀😀")}`),
      ).ok,
    ).toBe(true);
  });

  it("rejects controls and queries over 128 UTF-8 bytes", () => {
    const control = parseSearchParams(
      new URL(`https://registry.cohdl.org/search?q=${encodeURIComponent("abc\u0007")}`),
    );
    expect(control).toEqual({ ok: false, error: "q must not contain control characters" });
    const long = parseSearchParams(
      new URL(`https://registry.cohdl.org/search?q=${encodeURIComponent("é".repeat(65))}`),
    );
    expect(long).toEqual({ ok: false, error: "q must be at most 128 UTF-8 bytes" });
  });

  it("rejects unknown kinds and noncanonical or out-of-range pages", () => {
    for (const path of [
      "/search?q=usb&kind=parts",
      "/search?q=usb&limit=0",
      "/search?q=usb&limit=51",
      "/search?q=usb&limit=01",
      "/search?q=usb&offset=-1",
      "/search?q=usb&offset=10001",
      "/search?q=usb&offset=1.5",
    ]) {
      expect(parseSearchParams(new URL(`https://registry.cohdl.org${path}`)).ok, path).toBe(false);
    }
  });

  it("quotes an FTS phrase without allowing operators to become syntax", () => {
    expect(quoteFtsPhrase("stm32")).toBe('"stm32"');
    expect(quoteFtsPhrase('foo" OR bar')).toBe('"foo"" OR bar"');
  });
});

describe("supplier bindings in stable part search", () => {
  it("projects supplier bindings and makes their PN searchable", () => {
    const row = extractPartSearchRows({ items: [{ fq: "p::P", name: "P", kind: "part", pub: true,
      part: { device: "p::D", primary: { fields: [], suppliers: [
        { supplier: "acme", pn: "SKU-needle-9", at: ["-1.2mm", "0mm"], rotate: 180 },
      ] } } }] }, "p", "1.0.0")[0];
    expect(row.searchable).toContain("SKU-needle-9");
    expect(JSON.parse(row.avl_json)[0].suppliers).toEqual([
      { supplier: "acme", pn: "SKU-needle-9", at: ["-1.2mm", "0mm"], rotate: 180 },
    ]);
  });

  it("bounds bindings per AVL and drops malformed bindings without truncating fields", () => {
    const suppliers = Array.from({ length: 18 }, (_, i) => ({ supplier: `s${i}`,
      pn: `PN-${i}`, at: ["0mm", "0mm"], rotate: 0 }));
    suppliers[3] = { ...suppliers[3], rotate: 360 };
    const row = extractPartSearchRows({ items: [{ fq: "p::P", name: "P", kind: "part", pub: true,
      part: { device: "p::D", primary: { fields: [], suppliers },
        alts: [{ fields: [], suppliers: [{ supplier: "acme", pn: "ALT-1", at: ["0mm", "0mm"], rotate: 0 }] }] } }] },
      "p", "1.0.0")[0];
    const avl = JSON.parse(row.avl_json);
    expect(avl[0].suppliers).toHaveLength(15);
    expect(avl[0].suppliers.map((b: any) => b.supplier)).not.toContain("s3");
    expect(avl[1].suppliers).toHaveLength(1);
  });

  it("retains the existing cumulative and per-part projection bounds with suppliers", () => {
    const suppliers = Array.from({ length: 16 }, (_, i) => ({ supplier: `s${i}`,
      pn: `PN-${i}-${"x".repeat(110)}`, at: ["0mm", "0mm"], rotate: 0 }));
    const rows = extractPartSearchRows({ items: Array.from({ length: 40 }, (_, i) => ({
      fq: `p::P_${i}`, name: `P_${i}`, kind: "part", pub: true,
      part: { device: "p::D", primary: { fields: [], suppliers },
        alts: Array.from({ length: 255 }, () => ({ fields: [], suppliers })) },
    })) }, "p", "1.0.0");
    expect(partSearchChunks(rows).flatMap((chunk) => JSON.parse(chunk))).toEqual(rows);
    for (const row of rows) {
      expect(new TextEncoder().encode(JSON.stringify(row)).length).toBeLessThanOrEqual(PART_SEARCH_CHUNK_BYTES - 2);
      expect(JSON.parse(row.avl_json)).toHaveLength(256);
    }
    const bytes = rows.reduce((sum, row) => sum + new TextEncoder().encode(JSON.stringify(row)).length, 2);
    expect(bytes).toBeLessThanOrEqual(MAX_PART_SEARCH_INDEX_BYTES);
  });

  it("enriches through real D1 and leaves missing catalogs as null", async () => {
    const binding = { supplier: "acme", pn: "P / 1", at: ["1mm", "-2mm"], rotate: 90 };
    const { db, d1, statements } = sqliteSearch([partRow([{ primary: true, manufacturer: "M", mpn: "M-1",
      suppliers: [binding, { ...binding, supplier: "globex" }] }])]);
    catalogInsert(db, "acme", "P / 1", "resistor", 7);
    const { response, body } = await sqliteInvoke(d1, "/search?q=resistor&kind=part");
    expect(response.status).toBe(200);
    expect(body.parts.results[0].suppliers).toEqual([
      { ...binding, links: [{ label: "Product", url: "https://example.com/products/P%20%2F%201" },
        { label: "Datasheet", url: "https://example.com/datasheet" }],
        catalog: { class: "resistor", stock: 7, snapshot_date: "2026-10-28T01:02:03.000Z" } },
      { ...binding, supplier: "globex", links: [], catalog: null },
    ]);
    expect(statements.filter((s) => s.sql.includes("FROM supplier_catalog"))).toHaveLength(1);
    expect(statements.at(-1)?.values).toEqual(["acme", "P / 1", "globex", "P / 1"]);
  });

  it("searches PN through real FTS and filters bindings and catalog before pagination", async () => {
    const binding = (pn: string, supplier = "acme") => ({ supplier, pn, at: ["0mm", "0mm"], rotate: 0 });
    const rows = [
      partRow([{ primary: true, manufacturer: "M", mpn: "M-1", suppliers: [binding("WRONG-CLASS"), binding("GOOD-1", "globex")] }]),
      partRow([{ primary: true, manufacturer: "M", mpn: "M-1", suppliers: [binding("NO-CATALOG")] }]),
      partRow([{ primary: true, manufacturer: "M", mpn: "M-1", suppliers: [binding("GOOD-2"), binding("OTHER-CLASS")] }]),
      partRow([{ primary: true, manufacturer: "M", mpn: "M-1", suppliers: [binding("GOOD-3")] }]),
    ].map((row, i) => ({ ...row, fq: `p::P_${i}`, name: `P_${i}`, device: `p::resistor_${i}` }));
    const { db, d1, statements } = sqliteSearch(rows);
    for (const [pn, category] of [["WRONG-CLASS", "connector"], ["NO-CATALOG", "connector"],
      ["GOOD-2", "resistor"], ["OTHER-CLASS", "capacitor"], ["GOOD-3", "capacitor"]]) {
      catalogInsert(db, "acme", pn, category);
    }
    catalogInsert(db, "globex", "GOOD-1", "resistor");
    let result = await sqliteInvoke(d1, "/search?q=resistor&kind=part&supplier=acme&class=resistor&class=capacitor&limit=1");
    expect(result.body.parts.results.map((row: any) => row.fq)).toEqual(["p::P_3"]);
    expect(result.body.parts.has_more).toBe(true);
    result = await sqliteInvoke(d1, "/search?q=resistor&kind=part&supplier=acme&class=resistor&class=capacitor&limit=1&offset=1");
    expect(result.body.parts.results.map((row: any) => row.fq)).toEqual(["p::P_2"]);
    expect(result.body.parts.has_more).toBe(false);
    expect(statements[0].sql.indexOf("json_each")).toBeLessThan(statements[0].sql.indexOf("LIMIT"));
    result = await sqliteInvoke(d1, "/search?q=GOOD-3&kind=part&supplier=acme");
    expect(result.body.parts.results.map((row: any) => row.fq)).toEqual(["p::P_3"]);
    result = await sqliteInvoke(d1, "/search?q=resistor&kind=part&supplier=acme");
    expect(result.body.parts.results.map((row: any) => row.fq)).toContain("p::P_0");
    result = await sqliteInvoke(d1, "/search?q=NO-CATALOG&kind=part&supplier=acme");
    expect(result.body.parts.results.map((row: any) => row.fq)).toEqual(["p::P_1"]);
    result = await sqliteInvoke(d1, "/search?q=NO-CATALOG&kind=part&supplier=acme&class=resistor");
    expect(result.body.parts.results).toEqual([]);
  });

  it("chunks tuple IN enrichment below 100 bindings without per-row lookups", async () => {
    const suppliers = Array.from({ length: 50 }, (_, i) => ({ supplier: "acme", pn: `PN-${i}`,
      at: ["0mm", "0mm"], rotate: 0 }));
    const { db, d1, statements } = sqliteSearch([partRow(suppliers.map((binding, index) => ({
      primary: index === 0, manufacturer: "M", mpn: `M-${index}`, suppliers: [binding],
    })))]);
    for (const { pn } of suppliers) catalogInsert(db, "acme", pn, "resistor");
    const { response, body } = await sqliteInvoke(d1, "/search?q=resistor&kind=part");
    expect(response.status).toBe(200);
    expect(body.parts.results[0].suppliers).toHaveLength(50);
    expect(body.parts.results[0].suppliers.every((b: any) => b.catalog !== null)).toBe(true);
    const lookups = statements.filter((s) => s.sql.includes("FROM supplier_catalog c"));
    expect(lookups).toHaveLength(2);
    expect(lookups.map((s) => s.values.length)).toEqual([98, 2]);
    expect(lookups.flatMap((s) => s.values)).toEqual(suppliers.flatMap((b) => [b.supplier, b.pn]));
  });

  it("keeps the package-only web route independent of part supplier metadata", async () => {
    const { d1, statements } = sqliteSearch();
    const env = { DB: d1, ASSETS: { fetch: vi.fn() } } as unknown as Env;
    const response = await worker.fetch(new Request("https://registry.cohdl.org/api/search?q=p&supplier=acme"), env);
    expect(response.status).toBe(200);
    expect(statements).toHaveLength(2);
    expect(statements.every((s) => !s.sql.includes("part_search") && !s.sql.includes("supplier_catalog"))).toBe(true);
  });
});

describe("stable search response", () => {
  it("rejects non-GET methods at the stable route", async () => {
    const h = harness();
    const { response, body } = await invoke("/search?q=usb", h, "POST");
    expect(response.status).toBe(405);
    expect(response.headers.get("Allow")).toBe("GET");
    expect(response.headers.get("Cache-Control")).toBe("no-store");
    expect(body).toEqual({ error: "method not allowed" });
    expect(h.store.searchPackages).not.toHaveBeenCalled();
  });

  it("returns the exact flat contract and chooses a matching alternate AVL", async () => {
    const h = harness([packageRow(0)], [partRow()]);
    const { response, body } = await invoke("/search?q=needle", h);
    expect(response.status).toBe(200);
    expect(response.headers.get("Cache-Control")).toBe("public, max-age=30");
    expect(h.store.searchPackages).toHaveBeenCalledWith("needle", 21, 0);
    expect(h.store.searchParts).toHaveBeenCalledWith('"needle"', 21, 0, null, []);
    expect(body).toEqual({
      query: "needle",
      packages: { results: [packageRow(0)], has_more: false },
      parts: {
        results: [
          {
            package: "passive",
            tier: "official",
            version: "1.2.0",
            fq: "passive::resistors::R_0402",
            name: "R_0402",
            device: "passive::devices::Resistor",
            intent: null,
            manufacturer: "Vishay",
            mpn: "ALT-needle-42",
            primary: false,
            suppliers: [],
          },
        ],
        has_more: false,
      },
    });
  });

  it("falls back to the primary AVL when the hit came from another field", async () => {
    const h = harness([], [partRow()]);
    const { body } = await invoke("/search?q=resistor", h);
    expect(body.parts.results[0]).toMatchObject({
      manufacturer: "Yageo",
      mpn: "RC0402-primary",
      primary: true,
    });
  });

  it("degrades malformed or empty internal AVL metadata safely", async () => {
    for (const avl of ["{", {}, []]) {
      const h = harness([], [partRow(avl)]);
      const { response, body } = await invoke("/search?q=resistor", h);
      expect(response.status).toBe(200);
      expect(body.parts.results[0]).toMatchObject({
        manufacturer: null,
        mpn: null,
        primary: true,
      });
    }
  });

  it("uses limit+1 for independent truncation and applies the shared offset", async () => {
    const h = harness(
      Array.from({ length: 3 }, (_, index) => packageRow(index)),
      [partRow(), partRow(), partRow()],
    );
    const { body } = await invoke("/search?q=resistor&limit=2&offset=7", h);
    expect(h.store.searchPackages).toHaveBeenCalledWith("resistor", 3, 7);
    expect(h.store.searchParts).toHaveBeenCalledWith('"resistor"', 3, 7, null, []);
    expect(body.packages.results).toHaveLength(2);
    expect(body.packages.has_more).toBe(true);
    expect(body.parts.results).toHaveLength(2);
    expect(body.parts.has_more).toBe(true);
  });

  it("does not query the unrequested kind and returns its stable empty section", async () => {
    const packagesOnly = harness([packageRow(0)], [partRow()]);
    const packageResponse = await invoke("/search?q=passive&kind=package", packagesOnly);
    expect(packagesOnly.store.searchParts).not.toHaveBeenCalled();
    expect(packageResponse.body.parts).toEqual({ results: [], has_more: false });

    const partsOnly = harness([packageRow(0)], [partRow()]);
    const partResponse = await invoke("/search?q=resistor&kind=part", partsOnly);
    expect(partsOnly.store.searchPackages).not.toHaveBeenCalled();
    expect(partResponse.body.packages).toEqual({ results: [], has_more: false });
  });

  it("does not expose a store error or query text in a 500", async () => {
    const h = harness();
    h.store.searchPackages.mockRejectedValue(
      new Error("sensitive SQL detail for user-query-secret"),
    );
    const { response, body } = await invoke("/search?q=secret", h);
    expect(response.status).toBe(500);
    expect(response.headers.get("Cache-Control")).toBe("no-store");
    expect(body).toEqual({ error: "search is temporarily unavailable" });
    expect(JSON.stringify(body)).not.toContain("sensitive");
    expect(h.logs).toEqual([{ event: "registry_search_error", error: "Error" }]);
    expect(JSON.stringify(h.logs)).not.toContain("secret");
  });

  it("does not cache validation errors", async () => {
    const h = harness();
    const { response } = await invoke("/search?q=ab", h);
    expect(response.status).toBe(400);
    expect(response.headers.get("Cache-Control")).toBe("no-store");
  });
});

describe("API-doc part search extraction", () => {
  const primary = {
    fields: [
      { name: "mfr", value: "Samsung" },
      { name: "mpn", value: "CL05B104KO5NNNC" },
      { name: "supplier_sku", value: "C1525" },
    ],
  };
  const alt = {
    fields: [
      { name: "mfr", value: "Murata" },
      { name: "mpn", value: "GRM155R71C104KA88D" },
    ],
  };

  it("indexes only local public parts, including every primary/alt field", () => {
    const document = {
      items: [
        {
          fq: "passive::parts::C_100N",
          name: "C_100N",
          kind: "part",
          pub: true,
          intent: "General decoupling",
          part: {
            device: "passive::devices::MLCC",
            args: ["100nF", "16V"],
            variant: "C0402",
            primary,
            alts: [alt],
          },
        },
        {
          fq: "passive::parts::PRIVATE",
          name: "PRIVATE",
          kind: "part",
          pub: false,
          part: { device: "passive::devices::MLCC", primary },
        },
        { fq: "passive::devices::MLCC", name: "MLCC", kind: "device", pub: true },
      ],
      foreign: [
        {
          fq: "dependency::parts::FOREIGN",
          name: "FOREIGN",
          kind: "part",
          pub: true,
          part: { device: "dependency::Device", primary },
        },
      ],
    };
    const rows = extractPartSearchRows(document, "passive", "1.0.0");
    expect(rows).toHaveLength(1);
    expect(rows[0]).toMatchObject({
      package_name: "passive",
      package_version: "1.0.0",
      fq: "passive::parts::C_100N",
      name: "C_100N",
      device: "passive::devices::MLCC",
      intent: "General decoupling",
    });
    for (const value of [
      "Samsung",
      "CL05B104KO5NNNC",
      "supplier_sku",
      "C1525",
      "Murata",
      "GRM155R71C104KA88D",
      "100nF",
      "16V",
      "C0402",
    ]) {
      expect(rows[0].searchable).toContain(value);
    }
    expect(JSON.parse(rows[0].avl_json)).toEqual([
      { primary: true, manufacturer: "Samsung", mpn: "CL05B104KO5NNNC", suppliers: [] },
      { primary: false, manufacturer: "Murata", mpn: "GRM155R71C104KA88D", suppliers: [] },
    ]);
  });

  it("ignores malformed declarations and deterministically deduplicates fq", () => {
    const valid = {
      fq: "p::P",
      name: "P",
      kind: "part",
      pub: true,
      part: { device: "p::D", primary },
    };
    const rows = extractPartSearchRows(
      {
        items: [
          null,
          {},
          { ...valid, fq: null },
          { ...valid, part: null },
          valid,
          { ...valid, name: "duplicate must not replace first" },
        ],
      },
      "p",
      "1.0.0",
    );
    expect(rows).toHaveLength(1);
    expect(rows[0].name).toBe("P");
  });

  it("rejects forged foreign FQs and a name that is not the FQ tail", () => {
    const base = {
      name: "P",
      kind: "part",
      pub: true,
      part: { device: "p::D", primary },
    };
    const rows = extractPartSearchRows(
      {
        items: [
          { ...base, fq: "std::Forged" },
          { ...base, fq: "p::Actual", name: "Alias" },
          { ...base, fq: "p::nested::P" },
        ],
      },
      "p",
      "1.0.0",
    );
    expect(rows.map((row) => row.fq)).toEqual(["p::nested::P"]);
  });

  it("derives the same sanitized module root for scoped package names", () => {
    const rows = extractPartSearchRows(
      {
        items: [
          {
            fq: "st_stm32::mcu::Chip",
            name: "Chip",
            kind: "part",
            pub: true,
            part: { device: "st_stm32::mcu::Device", primary },
          },
          {
            fq: "st::stm32::Forged",
            name: "Forged",
            kind: "part",
            pub: true,
            part: { device: "st_stm32::mcu::Device", primary },
          },
        ],
      },
      "@st/stm32",
      "1.0.0",
    );
    expect(rows.map((row) => row.fq)).toEqual(["st_stm32::mcu::Chip"]);
  });

  it("normalizes lone UTF-16 surrogates before indexing", () => {
    const rows = extractPartSearchRows(
      {
        items: [
          {
            fq: "p::P\uD800",
            name: "P\uD800",
            kind: "part",
            pub: true,
            intent: "intent\uDFFF",
            part: {
              device: "p::D\uD800",
              primary: { fields: [{ name: "mfr", value: "Acme\uD800" }] },
            },
          },
        ],
      },
      "p",
      "1.0.0",
    );
    expect(rows).toHaveLength(1);
    expect(JSON.stringify(rows)).not.toMatch(/\\u(?:d[89ab][0-9a-f]{2}|d[c-f][0-9a-f]{2})/i);
    expect(rows[0]).toMatchObject({ fq: "p::P�", name: "P�", device: "p::D�", intent: "intent�" });
    expect(JSON.parse(rows[0].avl_json)[0].manufacturer).toBe("Acme�");
  });

  it("truncates a huge source field before UTF-8 normalization", () => {
    const rows = extractPartSearchRows(
      {
        items: [
          {
            fq: "p::P",
            name: "P",
            kind: "part",
            pub: true,
            intent: "x".repeat(4 * 1024 * 1024),
            part: { device: "p::D", primary },
          },
        ],
      },
      "p",
      "1.0.0",
    );
    expect(rows).toHaveLength(1);
    expect(rows[0].intent).toHaveLength(2_048);
  });

  it("adds an explicit primary identity even when the payload lacks one", () => {
    const rows = extractPartSearchRows(
      {
        items: [
          {
            fq: "p::P",
            name: "P",
            kind: "part",
            pub: true,
            part: { device: "p::D", alts: [alt] },
          },
        ],
      },
      "p",
      "1.0.0",
    );
    expect(JSON.parse(rows[0].avl_json)[0]).toEqual({
      primary: true,
      manufacturer: null,
      mpn: null,
      suppliers: [],
    });
  });

  it("chunks rows as complete JSON arrays without losing order", () => {
    const row = extractPartSearchRows(
      {
        items: [
          {
            fq: "p::P",
            name: "P",
            kind: "part",
            pub: true,
            part: { device: "p::D", primary },
          },
        ],
      },
      "p",
      "1.0.0",
    )[0];
    const many = Array.from({ length: 2_000 }, (_, index) => ({
      ...row,
      fq: `p::P_${index}_${"x".repeat(300)}`,
      name: `P_${index}`,
    }));
    const chunks = partSearchChunks(many);
    expect(chunks.length).toBeGreaterThan(1);
    expect(chunks.every((chunk) => new TextEncoder().encode(chunk).length <= PART_SEARCH_CHUNK_BYTES)).toBe(
      true,
    );
    expect(chunks.flatMap((chunk) => JSON.parse(chunk))).toEqual(many);
  });

  it("refuses a caller-supplied row that violates the single-chunk invariant", () => {
    const oversized = {
      package_name: "p",
      package_version: "1.0.0",
      fq: "p::P",
      name: "P",
      device: "p::D",
      intent: null,
      searchable: "x".repeat(PART_SEARCH_CHUNK_BYTES),
      avl_json: "[]",
    };
    expect(() => partSearchChunks([oversized])).toThrow("exceeds the D1 JSON chunk bound");
  });

  it("bounds cumulative extraction before rows and chunks amplify memory", () => {
    const items = Array.from({ length: 6_000 }, (_, index) => ({
      fq: `p::P_${index}`,
      name: `P_${index}`,
      kind: "part",
      pub: true,
      intent: `bulk-${index}-${"x".repeat(1_024)}`,
      part: { device: "p::D", primary },
    }));
    const rows = extractPartSearchRows({ items }, "p", "1.0.0");
    const chunks = partSearchChunks(rows);
    const conceptualArrayBytes =
      2 +
      rows.reduce(
        (total, row, index) =>
          total + new TextEncoder().encode(JSON.stringify(row)).length + (index === 0 ? 0 : 1),
        0,
      );
    expect(rows.length).toBeLessThan(items.length);
    expect(conceptualArrayBytes).toBeLessThanOrEqual(MAX_PART_SEARCH_INDEX_BYTES);
    expect(chunks.length).toBeLessThanOrEqual(MAX_PART_SEARCH_CHUNKS);
    expect(chunks.flatMap((chunk) => JSON.parse(chunk))).toEqual(rows);
  });

  it("caps insert chunks below the per-invocation D1 query budget", () => {
    const bulky = Array.from({ length: MAX_PART_SEARCH_CHUNKS + 5 }, (_, index) => ({
      package_name: "p",
      package_version: "1.0.0",
      fq: `p::P_${index}`,
      name: `P_${index}`,
      device: "p::D",
      intent: null,
      searchable: "x".repeat(PART_SEARCH_CHUNK_BYTES / 2),
      avl_json: "[]",
    }));
    const chunks = partSearchChunks(bulky);
    expect(chunks).toHaveLength(MAX_PART_SEARCH_CHUNKS);
    expect(chunks.flatMap((chunk) => JSON.parse(chunk))).toEqual(
      bulky.slice(0, MAX_PART_SEARCH_CHUNKS),
    );
  });
});
