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
}

export interface RecentRow {
  name: string;
  version: string;
  published_at: string;
  tier: string;
}

export interface PackageDetail {
  name: string;
  tier: string;
  created_at: string;
  versions: { version: string; hash: string; size: number; published_at: string }[];
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

export function useMe() {
  return useQuery({
    queryKey: ["me"],
    retry: false,
    queryFn: () => get<{ account: string; official: boolean; brands: string[] }>("/api/me"),
  });
}
