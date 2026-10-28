import type { SearchRow } from "./api";
import { defaultParseSearch } from "@tanstack/react-router";

export type CatalogueSort = "updated" | "name";

export interface CatalogueSearchState {
  q?: string;
  tier?: SearchRow["tier"];
  sort?: CatalogueSort;
  kind?: "part";
  supplier?: string;
  class?: string[];
}

function classValues(value: unknown): string[] | undefined {
  const values = Array.isArray(value) ? value : [value];
  const classes = [...new Set(values.filter((item): item is string => typeof item === "string")
    .map((item) => item.trim()).filter(Boolean))];
  return classes.length > 0 ? classes : undefined;
}

/// Defaults are omitted from shareable URLs. A class is meaningful only with
/// its supplier, matching the stable `/search` contract.
export function parseCatalogueSearch(search: Record<string, unknown>): CatalogueSearchState {
  const supplier = typeof search.supplier === "string" && /^[a-z][a-z0-9_]{0,31}$/.test(search.supplier)
    ? search.supplier
    : undefined;
  return {
    q: typeof search.q === "string" && search.q.trim() ? search.q.trim() : undefined,
    tier: search.tier === "official" || search.tier === "brand" || search.tier === "contrib"
      ? search.tier
      : undefined,
    sort: search.sort === "name" ? "name" : undefined,
    kind: search.kind === "part" ? "part" : undefined,
    supplier,
    class: supplier ? classValues(search.class) : undefined,
  };
}

export function parseClassInput(value: string): string[] | undefined {
  return classValues(value.split(","));
}

/// TanStack's default encoder serializes arrays as JSON. The registry API and
/// RFC-034 instead use repeatable `class` parameters, so encode this route
/// explicitly while retaining its normal scalar state.
export function stringifyCatalogueSearch(search: object): string {
  const params = new URLSearchParams();
  const state = search as Record<string, unknown>;
  for (const [key, value] of Object.entries(state)) {
    if (value === undefined) continue;
    if (key === "class") {
      if (typeof state.supplier === "string" && state.supplier) {
        for (const category of Array.isArray(value) ? value : [value]) {
          if (typeof category === "string" && category) params.append(key, category);
        }
      }
      continue;
    }
    let encoded = String(value);
    if (typeof value === "object" && value !== null) {
      encoded = JSON.stringify(value);
    } else if (typeof value === "string") {
      // Match TanStack's default handling of JSON-looking scalar strings.
      try { JSON.parse(value); encoded = JSON.stringify(value); } catch { /* plain string */ }
    }
    params.set(key, encoded);
  }
  return params.size > 0 ? `?${params.toString()}` : "";
}

/// qss coerces numeric-looking values. Restore raw class strings after the
/// default parse so classes such as `0402` survive a URL round trip.
export function parseCatalogueUrlSearch(search: string): Record<string, unknown> {
  const parsed = defaultParseSearch(search) as Record<string, unknown>;
  const classes = new URLSearchParams(search).getAll("class");
  if (classes.length > 0) parsed.class = classes;
  return parsed;
}