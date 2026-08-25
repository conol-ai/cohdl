// The web UI's data layer: thin fetch wrappers + TanStack Query hooks.

import { useQuery } from "@tanstack/react-query";

export class ApiError extends Error {
  readonly status: number;
  readonly fields?: Record<string, string>;

  constructor(message: string, status: number, fields?: Record<string, string>) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.fields = fields;
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
  const data = (await r.json().catch(() => null)) as
    | (T & { error?: string; fields?: Record<string, string> })
    | null;
  if (!r.ok) throw new ApiError(data?.error ?? `HTTP ${r.status}`, r.status, data?.fields);
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
/// declared and the RFC-017 documents it ships. `api_docs` says whether an
/// API documentation sidecar (docs/apidocs.md) has been uploaded for it.
export interface VersionRow {
  version: string;
  hash: string;
  size: number;
  published_at: string;
  description: string | null;
  license: string | null;
  repository: string | null;
  docs: string[];
  api_docs: boolean;
}

export interface PackageDetail {
  name: string;
  tier: string;
  created_at: string;
  versions: VersionRow[];
}

// ---------------------------------------------------------------------------
// Package API documentation — the schema_version 1 document of
// docs/apidocs.md, produced by the Rust emitter and served byte-for-byte.
// Every string in it is publisher-derived content: it must only ever render
// as React text or SVG attributes, never as HTML.

export type ApiDocsKind = "trait" | "device" | "fn" | "part" | "pad" | "footprint" | "design";

export interface ApiDocsPackage {
  name: string;
  version: string;
  root: string;
  description?: string;
  license?: string;
  repository?: string;
}

export interface ApiDocsDependency {
  name: string;
  version: string;
  root: string;
}

export interface TraitPinDoc {
  name: string;
  obligation: "required" | "optional";
}

export interface TraitSpecDoc {
  name: string;
  type: string;
}

export interface TraitDoc {
  super_traits?: string[];
  designator_prefix?: string;
  pins?: TraitPinDoc[];
  specs?: TraitSpecDoc[];
}

export interface GenericDoc {
  name: string;
  bound: { unit?: string; traits?: string[] };
  default?: string;
}

export type PinRole = "input" | "output" | "bidirectional" | "passive" | "power_in" | "power_out";

export interface DevicePinDoc {
  name: string;
  obligation: "required" | "optional";
  numbers: string[];
  role: PinRole;
}

export interface SpecFieldDoc {
  name: string;
  /// Either `value` (literal, preserved source text) or `generic`
  /// (parameter name) — never both.
  value?: string;
  generic?: string;
}

export interface DeviceDoc {
  generics?: GenericDoc[];
  variants?: string[];
  designator_prefix: string;
  /// One entry (no `variant` key) for a variant-less device; one entry per
  /// variant, in declaration order, otherwise.
  pins?: { variant?: string; pins: DevicePinDoc[] }[];
  specs?: { variant?: string; fields: SpecFieldDoc[] }[];
}

export interface FnParamDoc {
  name: string;
  type: { kind: string; name?: string; traits?: string[] };
}

export interface InstDoc {
  name: string;
  type: string;
  args?: string[];
  variant?: string;
}

export interface FnDoc {
  generics?: GenericDoc[];
  params?: FnParamDoc[];
  insts?: InstDoc[];
  calls?: string[];
  nets: number;
}

export interface DesignDoc {
  insts?: InstDoc[];
  calls?: string[];
  nets: number;
}

export interface AvlEntryDoc {
  fields: { name: string; value: string }[];
  footprint?: string;
}

export interface PartDoc {
  device: string;
  args?: string[];
  variant?: string;
  primary: AvlEntryDoc;
  alts?: AvlEntryDoc[];
}

export type PadShapeName = "rect" | "circle" | "oval" | "annulus";

/// All geometry values are canonical mm decimal strings (`geom::mm`).
export interface PadDoc {
  shape?: PadShapeName;
  /// Arity follows the shape: circle 1; rect/oval 2; annulus
  /// `[outer_diameter, inner_diameter]`.
  size?: string[];
  layer?: string;
  plating?: string;
  drill?: { round?: string; slot?: string[] };
  chamfer?: { corner: string; cut: string };
  corner_radius?: string;
  mask_expansion?: string;
  paste?: "none" | { rect?: string[]; segmented_annulus?: string[] };
}

export interface FootprintPadDoc {
  number: string;
  pad: string;
  x: string;
  y: string;
  rotate?: number;
}

export interface MountHoleDoc {
  number: string;
  plating: string;
  shape: string;
  x: string;
  y: string;
  diameter?: string;
  size?: string[];
}

export interface OutlineDoc {
  shape: string;
  at: string[];
  size: string[];
}

export interface MarkerDoc {
  kind: string;
  pad?: string;
  cathode_pin?: string;
  shape: string;
}

export type SilkDoc =
  | { kind: "line"; from: string[]; to: string[]; width: string }
  | { kind: "circle"; at: string[]; radius: string; width?: string; fill?: boolean }
  | {
      kind: "arc";
      at: string[];
      radius: string;
      start_angle: number;
      end_angle: number;
      width: string;
    }
  | { kind: "polygon"; points: string[][]; width?: string; fill?: boolean };

export interface FootprintDoc {
  placeholder: boolean;
  pads?: FootprintPadDoc[];
  mount_holes?: MountHoleDoc[];
  courtyard?: OutlineDoc;
  window?: OutlineDoc;
  silkscreen_ref?: { at: string[] };
  markers?: MarkerDoc[];
  /// The EXPANDED graphics list (`emit::silk::graphics`) — semantic markers
  /// already reduced to checked primitives.
  silk?: SilkDoc[];
}

interface ApiDocsItemBase {
  fq: string;
  name: string;
  pub: boolean;
  module: string;
  file: string;
  line: number;
  intent?: string;
  docs?: string[];
}

/// One `items`/`foreign` entry: the common keys plus exactly one payload key
/// named after the kind.
export type ApiDocsItem = ApiDocsItemBase &
  (
    | { kind: "trait"; trait: TraitDoc }
    | { kind: "device"; device: DeviceDoc }
    | { kind: "fn"; fn: FnDoc }
    | { kind: "part"; part: PartDoc }
    | { kind: "pad"; pad: PadDoc }
    | { kind: "footprint"; footprint: FootprintDoc }
    | { kind: "design"; design: DesignDoc }
  );

export interface ApiDocsImpl {
  trait: string;
  device: string;
  file: string;
  line: number;
  pin_map?: { role: string; pin: string }[];
  spec_map?: { field: string; spec: string }[];
}

export interface ApiDocs {
  schema_version: number;
  generator: string;
  package: ApiDocsPackage;
  dependencies?: ApiDocsDependency[];
  items?: ApiDocsItem[];
  impls?: ApiDocsImpl[];
  foreign?: ApiDocsItem[];
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

export interface RegistryConfig {
  recaptcha_site_key: string | null;
  component_requests_enabled: boolean;
}

export interface ComponentRequestResponse {
  ok: true;
  duplicate: boolean;
}

export type ComponentRequestStatus = "open" | "resolved";
export type ComponentRequestSort = "requested" | "newest";

export interface ComponentRequestRow {
  id: number;
  manufacturer: string;
  part_number: string;
  datasheet_url: string;
  description: string | null;
  status: ComponentRequestStatus;
  request_count: number;
  created_at: string;
  last_requested_at: string;
  updated_at: string;
  resolved_at: string | null;
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

/// A version's API documentation sidecar. `null` data means the version has
/// no docs uploaded (a normal state, not an error — the endpoint 404s).
/// Pass `version: undefined` to skip fetching entirely, e.g. when the
/// version row already says `api_docs: false`.
export function useApiDocs(pkg: string, version: string | undefined) {
  return useQuery({
    queryKey: ["apidocs", pkg, version],
    enabled: !!pkg && !!version,
    staleTime: 10 * 60 * 1000, // matches the endpoint's Cache-Control max-age
    retry: (failureCount, error) =>
      !(error instanceof ApiError && error.status === 404) && failureCount < 2,
    queryFn: async (): Promise<ApiDocs | null> => {
      const params = new URLSearchParams({ pkg, version: version! });
      const r = await fetch(`/api/apidocs?${params.toString()}`);
      if (r.status === 404) return null;
      if (!r.ok) {
        const data = (await r.json().catch(() => null)) as { error?: string } | null;
        throw new ApiError(data?.error ?? `HTTP ${r.status}`, r.status);
      }
      return r.json() as Promise<ApiDocs>;
    },
  });
}

export function useConfig() {
  return useQuery({
    queryKey: ["config"],
    staleTime: Infinity,
    queryFn: () => get<RegistryConfig>("/api/config"),
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

export function useAdminComponentRequests(
  status: ComponentRequestStatus | "all",
  sort: ComponentRequestSort,
  q: string,
) {
  const params = new URLSearchParams({ status, sort });
  if (q) params.set("q", q);
  return useQuery({
    queryKey: ["admin", "component-requests", status, sort, q],
    queryFn: () =>
      get<{ requests: ComponentRequestRow[]; truncated: boolean }>(
        `/api/admin/component-requests?${params.toString()}`,
      ),
  });
}
