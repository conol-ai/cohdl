// The web UI's data layer: thin fetch wrappers + TanStack Query hooks.

import { useQuery } from "@tanstack/react-query";

async function get<T>(url: string): Promise<T> {
  const r = await fetch(url);
  if (!r.ok) {
    const data = (await r.json().catch(() => null)) as { error?: string } | null;
    throw new Error(data?.error ?? `HTTP ${r.status}`);
  }
  return r.json() as Promise<T>;
}

export async function post<T>(url: string, body: unknown): Promise<T> {
  const r = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = (await r.json().catch(() => null)) as (T & { error?: string }) | null;
  if (!r.ok) throw new Error(data?.error ?? `HTTP ${r.status}`);
  return data as T;
}

export interface SearchRow {
  name: string;
  tier: "official" | "brand" | "contrib";
  latest: string;
  updated: string;
  description: string | null;
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

/// URL of a published document's bytes — served sandboxed from the immutable
/// tar in R2.
export function docUrl(pkg: string, version: string, path: string): string {
  const q = new URLSearchParams({ pkg, version, path });
  return `/api/doc?${q.toString()}`;
}

export function useSearch(q: string) {
  return useQuery({
    queryKey: ["search", q],
    queryFn: () => get<{ results: SearchRow[] }>(`/api/search?q=${encodeURIComponent(q)}`),
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
    queryFn: () => get<{ account: string; official: boolean; brands: string[] }>("/api/me"),
  });
}
