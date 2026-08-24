import { describe, expect, it, vi } from "vitest";
import {
  MAX_PART_SEARCH_INDEX_BYTES,
  MAX_PART_SEARCH_CHUNKS,
  PART_SEARCH_CHUNK_BYTES,
  extractPartSearchRows,
  handleSearch,
  parseSearchParams,
  partSearchChunks,
  quoteFtsPhrase,
  type IndexedPartSearchRow,
  type PackageSearchRow,
  type SearchDependencies,
  type SearchStore,
} from "../src/worker/search";

interface Harness {
  deps: SearchDependencies;
  store: {
    searchPackages: ReturnType<typeof vi.fn<SearchStore["searchPackages"]>>;
    searchParts: ReturnType<typeof vi.fn<SearchStore["searchParts"]>>;
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
  it("trims the query and applies bounded defaults", () => {
    expect(parseSearchParams(new URL("https://registry.cohdl.org/search?q=%20stm32%20"))).toEqual({
      ok: true,
      value: { query: "stm32", kind: "all", limit: 20, offset: 0 },
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
      value: { query: "stm32", kind: "all", limit: 20, offset: 0 },
    });
  });

  it("accepts all kinds and their numeric boundaries", () => {
    expect(
      parseSearchParams(
        new URL("https://registry.cohdl.org/search?q=usb&kind=part&limit=1&offset=10000"),
      ),
    ).toEqual({
      ok: true,
      value: { query: "usb", kind: "part", limit: 1, offset: 10_000 },
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
    expect(h.store.searchParts).toHaveBeenCalledWith('"needle"', 21, 0);
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
    expect(h.store.searchParts).toHaveBeenCalledWith('"resistor"', 3, 7);
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
      { primary: true, manufacturer: "Samsung", mpn: "CL05B104KO5NNNC" },
      { primary: false, manufacturer: "Murata", mpn: "GRM155R71C104KA88D" },
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
