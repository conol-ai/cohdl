import { describe, expect, it } from "vitest";
import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { PartCard, SupplierBindings, formatDate, formatSize, safeSupplierUrl } from "../src/ui/components";
import { packageSearchUrl, partSearchUrl, type SupplierBinding } from "../src/ui/api";
import { parseCatalogueSearch, parseCatalogueUrlSearch, parseClassInput, stringifyCatalogueSearch } from "../src/ui/catalogue-model";

describe("registry UI formatting", () => {
  it("formats artifact sizes for compact metadata", () => {
    expect(formatSize(0)).toBe("0 B");
    expect(formatSize(1023)).toBe("1023 B");
    expect(formatSize(1024)).toBe("1.0 KB");
    expect(formatSize(15 * 1024 * 1024)).toBe("15 MB");
  });

  it("formats valid dates and preserves invalid source values", () => {
    expect(formatDate("2026-07-31T02:09:55.699Z")).toMatch(/Jul.*2026|2026.*Jul/);
    expect(formatDate("not-a-date")).toBe("not-a-date");
  });
});

function binding(overrides: Partial<SupplierBinding> = {}): SupplierBinding {
  return {
    supplier: "acme<script>",
    pn: 'A-1<"&>',
    at: ["1mm", "-2mm"],
    rotate: 180,
    links: [{ label: "Product <safe>", url: "https://example.com/A-1?q=1&x=2" }],
    catalog: { class: "resistor<0402>", stock: 0, snapshot_date: "2026-10-28T01:02:03.000Z" },
    ...overrides,
  };
}

describe("supplier part views", () => {
  it("renders PN, placement mapping, links, class, snapshot date and zero stock", () => {
    const html = renderToStaticMarkup(React.createElement(SupplierBindings, { suppliers: [binding()] }));
    for (const text of ["acme&lt;script&gt;", "A-1&lt;&quot;&amp;&gt;", "(1mm, -2mm)", "180°",
      "resistor&lt;0402&gt;", "2026-10-28T01:02:03.000Z", ">0</td>"]) {
      expect(html).toContain(text);
    }
    expect(html).toContain('href="https://example.com/A-1?q=1&amp;x=2"');
    expect(html).toContain('rel="noopener noreferrer"');
    expect(html).toContain('target="_blank"');
    expect(html).not.toContain("<script>");
  });

  it("renders null catalog and missing suppliers without confusing them with zero stock", () => {
    const html = renderToStaticMarkup(React.createElement(SupplierBindings, {
      suppliers: [binding({ catalog: null, links: [] })],
    }));
    expect(html).toContain("A-1");
    expect(html).not.toContain("resistor");
    expect(html).not.toContain("2026-10-28");
    expect(renderToStaticMarkup(React.createElement(SupplierBindings, {}))).toContain("No supplier bindings");
    const legacy = renderToStaticMarkup(React.createElement(PartCard, { part: {
      package: "p", tier: "official", version: "1.0.0", fq: "p::P", name: "P",
      device: "p::D", intent: null, manufacturer: null, mpn: null, primary: true,
    } }));
    expect(legacy).toContain("No supplier bindings");
  });

  it("accepts only credential-free http(s) anchors and escapes every rendered label", () => {
    for (const url of ["javascript:alert(1)", "data:text/html,x", "https://user@example.com/x",
      "https://user:pass@example.com/x", "//example.com/x", "https://example.com/a b"]) {
      expect(safeSupplierUrl(url)).toBeNull();
    }
    expect(safeSupplierUrl("http://example.com/x")).toBe("http://example.com/x");
    const unsafe = binding({ links: [
      { label: "bad protocol", url: "javascript:alert(1)" },
      { label: "bad credentials", url: "https://user@example.com/x" },
      { label: "safe <label>", url: "https://example.com/safe" },
    ] });
    const html = renderToStaticMarkup(React.createElement(SupplierBindings, { suppliers: [unsafe] }));
    expect(html).not.toContain("javascript:");
    expect(html).not.toContain("user@example.com");
    expect(html).not.toContain("bad protocol");
    expect(html).toContain("safe &lt;label&gt;");
    expect(html).toContain('href="https://example.com/safe"');
    expect(renderToStaticMarkup(React.createElement(SupplierBindings, {
      suppliers: [binding({ links: undefined as unknown as [] })],
    }))).not.toContain("<a");
  });
});

describe("catalogue supplier filters", () => {
  it("keeps supplier and repeatable classes in the URL state", () => {
    const state = parseCatalogueSearch({ q: " resistor ", kind: "part", supplier: "acme",
      class: ["resistor", "capacitor", "resistor"] });
    expect(state.class).toEqual(["resistor", "capacitor"]);
    const url = stringifyCatalogueSearch(state);
    expect(url).toBe("?q=resistor&kind=part&supplier=acme&class=resistor&class=capacitor");
    const decoded = parseCatalogueUrlSearch(url);
    expect(parseCatalogueSearch(decoded)).toEqual(state);
    expect(parseClassInput(" resistor, capacitor,resistor ")).toEqual(["resistor", "capacitor"]);
  });

  it("drops classes without a supplier and preserves legacy package URL state", () => {
    expect(parseCatalogueSearch({ kind: "part", class: ["resistor"] }).class).toBeUndefined();
    expect(stringifyCatalogueSearch(parseCatalogueSearch({ q: "usb", tier: "official", sort: "name" })))
      .toBe("?q=usb&tier=official&sort=name");
    expect(stringifyCatalogueSearch({ view: "api", item: "p::P", version: "1.0.0" }))
      .toBe("?view=api&item=p%3A%3AP&version=1.0.0");
    expect(parseCatalogueSearch(parseCatalogueUrlSearch("?kind=part&supplier=acme&class=0402&class=true")))
      .toMatchObject({ class: ["0402", "true"] });
  });

  it("uses stable kind=part search while package search keeps its original endpoint", () => {
    const part = new URL(partSearchUrl("P / 1", "acme", ["resistor", "capacitor"]), "https://registry.test");
    expect(part.pathname).toBe("/search");
    expect(part.searchParams.get("kind")).toBe("part");
    expect(part.searchParams.get("q")).toBe("P / 1");
    expect(part.searchParams.get("supplier")).toBe("acme");
    expect(part.searchParams.getAll("class")).toEqual(["resistor", "capacitor"]);
    expect(partSearchUrl("resistor")).toBe("/search?q=resistor&kind=part");
    expect(packageSearchUrl("resistor", "official", "name"))
      .toBe("/api/search?q=resistor&tier=official&sort=name");
    expect(packageSearchUrl("resistor")).toBe("/api/search?q=resistor");
  });
});
