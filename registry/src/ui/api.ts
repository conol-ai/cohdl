// The web UI's data layer: thin fetch wrappers + TanStack Query hooks.

import { useQuery } from "@tanstack/react-query";

export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

async function get<T>(url: string): Promise<T> {
  const r = await fetch(url);
  if (!r.ok) {
    const data = (await r.json().catch(() => null)) as { error?: string } | null;
    throw new ApiError(data?.error ?? `HTTP ${r.status}`, r.status);
  }
  return r.json() as Promise<T>;
}

async function write<T>(method: "POST" | "PUT" | "DELETE", url: string, body: unknown): Promise<T> {
  const r = await fetch(url, {
    method,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = (await r.json().catch(() => null)) as (T & { error?: string }) | null;
  if (!r.ok) throw new ApiError(data?.error ?? `HTTP ${r.status}`, r.status);
  return data as T;
}

export function isUnauthorized(error: unknown): boolean {
  return error instanceof ApiError && error.status === 401;
}

export function post<T>(url: string, body: unknown): Promise<T> {
  return write<T>("POST", url, body);
}

export function put<T>(url: string, body: unknown): Promise<T> {
  return write<T>("PUT", url, body);
}

export function del<T>(url: string, body: unknown): Promise<T> {
  return write<T>("DELETE", url, body);
}

export interface SearchRow {
  name: string;
  tier: "official" | "brand" | "contrib";
  latest: string;
  updated: string;
  description: string | null;
}

export interface SearchResponse {
  results: SearchRow[];
  total: number;
  truncated: boolean;
}

export interface RecentRow {
  name: string;
  version: string;
  published_at: string;
  tier: string;
  description: string | null;
}

/// One published version, with the `[package]` metadata its own manifest
/// declared and the RFC-017 documents it ships.
export interface VersionRow {
  version: string;
  hash: string;
  size: number;
  published_at: string;
  description: string | null;
  license: string | null;
  repository: string | null;
  docs: string[];
}

export interface PackageDetail {
  name: string;
  tier: string;
  created_at: string;
  versions: VersionRow[];
}

export interface AdminBrandClaim {
  brand: string;
  verified: boolean;
}

export interface AdminAccount {
  id: number;
  email: string;
  official: boolean;
  created_at: string;
  brands: AdminBrandClaim[];
}

export interface Me {
  account: string;
  official: boolean;
  brands: string[];
}

/// URL of a published document's bytes — served sandboxed from the immutable
/// tar in R2.
export function docUrl(pkg: string, version: string, path: string): string {
  const q = new URLSearchParams({ pkg, version, path });
  return `/api/doc?${q.toString()}`;
}

export function useSearch(
  q: string,
  tier?: SearchRow["tier"],
  sort: "updated" | "name" = "updated",
) {
  const params = new URLSearchParams({ q });
  if (tier) params.set("tier", tier);
  if (sort !== "updated") params.set("sort", sort);
  return useQuery({
    queryKey: ["search", q, tier ?? "all", sort],
    queryFn: () => get<SearchResponse>(`/api/search?${params.toString()}`),
  });
}

export function useRecent() {
  return useQuery({
    queryKey: ["recent"],
    queryFn: () => get<{ results: RecentRow[] }>("/api/recent"),
  });
}

export function usePackage(name: string) {
  return useQuery({
    queryKey: ["package", name],
    queryFn: () => get<PackageDetail>(`/api/packages/${encodeURIComponent(name)}`),
    retry: (failureCount, error) =>
      !(error instanceof ApiError && error.status === 404) && failureCount < 2,
  });
}

/// A document's text, for the Markdown renderer. Only ever called for text
/// documents (`.md`/`.txt`); binary ones are linked, not rendered.
export function useDocText(pkg: string, version: string | undefined, path: string | null) {
  return useQuery({
    queryKey: ["doc", pkg, version, path],
    enabled: !!version && !!path,
    staleTime: Infinity, // a published version is immutable
    queryFn: async () => {
      const r = await fetch(docUrl(pkg, version!, path!));
      if (!r.ok) throw new Error(`could not load \`${path}\` (HTTP ${r.status})`);
      return r.text();
    },
  });
}

export function useConfig() {
  return useQuery({
    queryKey: ["config"],
    staleTime: Infinity,
    queryFn: () => get<{ recaptcha_site_key: string | null }>("/api/config"),
  });
}

export function useMe() {
  return useQuery({
    queryKey: ["me"],
    retry: false,
    queryFn: () => get<Me>("/api/me"),
  });
}

export function useAdminAccounts(q: string) {
  return useQuery({
    queryKey: ["admin", "accounts", q],
    queryFn: () =>
      get<{ accounts: AdminAccount[]; truncated: boolean }>(
        `/api/admin/accounts?q=${encodeURIComponent(q)}`,
      ),
  });
}
