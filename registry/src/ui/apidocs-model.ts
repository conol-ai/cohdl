// Pure derivations for the package API explorer (docs/apidocs.md) — no
// React in here. Everything is deterministic data → data, unit-tested in
// test/apidocs-model.test.ts. Geometry parses the emitter's canonical mm
// strings with `Number(...)` for display-only SVG; nothing byte-stable is
// ever derived from these floats.

import type {
  ApiDocs,
  ApiDocsItem,
  ApiDocsKind,
  DeviceDoc,
  DevicePinDoc,
  FnDoc,
  FootprintDoc,
  PadDoc,
  PartDoc,
  SpecFieldDoc,
} from "./api";

/// Canonical-mm decimal string → number. Unparseable input degrades to 0 —
/// the safe direction for a preview (a wrong picture, never a crash).
export function mm(value: string | undefined): number {
  if (value === undefined) return 0;
  const n = Number(value);
  return Number.isFinite(n) ? n : 0;
}

/// Hostile-document guard. The server does not deep-validate the uploaded
/// docs JSON, so every array the document claims to carry may be missing —
/// or not an array at all. Anything that is not a real array degrades to
/// empty, in the same spirit as `mm`.
export function asArray<T>(value: T[] | undefined): T[] {
  return Array.isArray(value) ? value : [];
}

// --- navigation -------------------------------------------------------------

export interface ModuleGroup {
  module: string;
  items: ApiDocsItem[];
}

/// Items grouped by their `module`, groups sorted by module path, item order
/// (the document's fq order) preserved inside each group.
export function moduleGroups(items: ApiDocsItem[]): ModuleGroup[] {
  const byModule = new Map<string, ApiDocsItem[]>();
  for (const item of items) {
    const module = String(item?.module ?? "");
    const bucket = byModule.get(module);
    if (bucket) bucket.push(item);
    else byModule.set(module, [item]);
  }
  return [...byModule.entries()]
    .sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0))
    .map(([module, moduleItems]) => ({ module, items: moduleItems }));
}

export interface ItemFilter {
  q?: string;
  kind?: string;
  showPrivate?: boolean;
}

/// Case-insensitive substring match on `name` and `fq`, optional kind
/// narrowing, and the pub gate — the package's public API is the `pub` set.
export function filterItems(items: ApiDocsItem[], filter: ItemFilter): ApiDocsItem[] {
  const needle = (filter.q ?? "").trim().toLowerCase();
  return items.filter((item) => {
    if (!item) return false;
    if (!filter.showPrivate && !item.pub) return false;
    if (filter.kind && item.kind !== filter.kind) return false;
    if (!needle) return true;
    const name = String(item.name ?? "").toLowerCase();
    const fq = String(item.fq ?? "").toLowerCase();
    return name.includes(needle) || fq.includes(needle);
  });
}

/// Rail order for the kind list — the reader's likely path from "what is
/// this package" down to raw geometry.
export const KIND_ORDER: ApiDocsKind[] = [
  "design",
  "device",
  "part",
  "trait",
  "fn",
  "footprint",
  "pad",
];

export function kindCounts(items: ApiDocsItem[]): { kind: ApiDocsKind; count: number }[] {
  const counts = new Map<ApiDocsKind, number>();
  for (const item of items) counts.set(item.kind, (counts.get(item.kind) ?? 0) + 1);
  return KIND_ORDER.filter((kind) => counts.has(kind)).map((kind) => ({
    kind,
    count: counts.get(kind)!,
  }));
}

/// The last path segment (`passive::pads::P_0402` → `P_0402`).
export function shortName(fq: string): string {
  const path = String(fq ?? "");
  const idx = path.lastIndexOf("::");
  return idx === -1 ? path : path.slice(idx + 2);
}

export type FqTarget =
  | { kind: "local"; fq: string }
  | { kind: "dependency"; package: string; fq: string }
  | { kind: "plain"; fq: string };

/// Where an fq reference can link: this package's own explorer, a dependency
/// package's registry page (via the `dependencies` root map), or nowhere —
/// an unknown root renders as plain text.
export function classifyFq(fq: string, doc: ApiDocs): FqTarget {
  const path = String(fq ?? "");
  const sep = path.indexOf("::");
  if (sep === -1) return { kind: "local", fq: path }; // bare-named designs are local
  const root = path.slice(0, sep);
  if (root === doc.package?.root) return { kind: "local", fq: path };
  const dep = asArray(doc.dependencies).find((d) => d?.root === root);
  if (dep) return { kind: "dependency", package: dep.name, fq: path };
  return { kind: "plain", fq: path };
}

/// fq → item across `items` and `foreign` (foreign fills render-support
/// roles: pads for footprints, devices and footprints for parts).
export function itemsByFq(doc: ApiDocs): Map<string, ApiDocsItem> {
  const map = new Map<string, ApiDocsItem>();
  for (const item of [...asArray(doc.items), ...asArray(doc.foreign)]) {
    if (item) map.set(item.fq, item);
  }
  return map;
}

export function padsByFq(doc: ApiDocs): Map<string, PadDoc> {
  const map = new Map<string, PadDoc>();
  for (const item of [...asArray(doc.items), ...asArray(doc.foreign)]) {
    if (item?.kind === "pad") map.set(item.fq, item.pad);
  }
  return map;
}

/// This package's parts bound to a device — the reverse of `part.device`.
export function partsForDevice(items: ApiDocsItem[], deviceFq: string): ApiDocsItem[] {
  return items.filter((item) => item?.kind === "part" && item.part?.device === deviceFq);
}

/// The pin set an instantiation of `variant` sees (a variant-less device has
/// exactly one unnamed set).
export function pinsForVariant(device: DeviceDoc, variant: string | undefined): DevicePinDoc[] {
  const sets = asArray(device.pins);
  const hit = sets.find((s) => s?.variant === variant) ?? sets[0];
  return asArray(hit?.pins);
}

/// The merged spec view for `variant` (`spec_fields_for`, pre-merged by the
/// emitter).
export function specsForVariant(device: DeviceDoc, variant: string | undefined): SpecFieldDoc[] {
  const sets = asArray(device.specs);
  const hit = sets.find((s) => s?.variant === variant) ?? sets[0];
  return asArray(hit?.fields);
}

/// AVL table columns: field names in first-appearance order over the primary
/// entry then the alts.
export function avlColumns(part: PartDoc): string[] {
  const cols: string[] = [];
  for (const entry of [part.primary, ...asArray(part.alts)]) {
    for (const field of asArray(entry?.fields)) {
      const name = field?.name;
      if (name != null && !cols.includes(name)) cols.push(name);
    }
  }
  return cols;
}

/// One list-row line: the `#[intent]` when present, otherwise a small
/// derived fact. Deterministic — it only reads the item. A hostile item
/// missing its kind-named payload gets an empty summary, never a crash.
export function itemSummary(item: ApiDocsItem): string {
  if (item.intent) return item.intent;
  switch (item.kind) {
    case "device": {
      if (!item.device) return "";
      const variants = asArray(item.device.variants).length;
      if (variants > 0) return `${variants} package variant${variants === 1 ? "" : "s"}`;
      const pins = asArray(asArray(item.device.pins)[0]?.pins).length;
      return `${pins} pin${pins === 1 ? "" : "s"}`;
    }
    case "part": {
      if (!item.part) return "";
      const mpn = asArray(item.part.primary?.fields).find((f) => f?.name === "mpn")?.value;
      const device = shortName(item.part.device);
      return mpn ? `${device} · ${mpn}` : device;
    }
    case "trait": {
      if (!item.trait) return "";
      const pins = asArray(item.trait.pins).length;
      const specs = asArray(item.trait.specs).length;
      return `${pins} pin${pins === 1 ? "" : "s"} · ${specs} spec${specs === 1 ? "" : "s"}`;
    }
    case "fn": {
      if (!item.fn) return "";
      const params = asArray(item.fn.params).length;
      return `${params} parameter${params === 1 ? "" : "s"} · ${item.fn.nets} net${
        item.fn.nets === 1 ? "" : "s"
      }`;
    }
    case "pad": {
      if (!item.pad) return "";
      return [
        item.pad.shape,
        Array.isArray(item.pad.size) ? item.pad.size.join(" × ") : undefined,
        item.pad.plating,
      ]
        .filter(Boolean)
        .join(" · ");
    }
    case "footprint": {
      if (!item.footprint) return "";
      if (item.footprint.placeholder) return "placeholder";
      const pads = asArray(item.footprint.pads).length;
      return `${pads} pad${pads === 1 ? "" : "s"}`;
    }
    case "design": {
      if (!item.design) return "";
      const insts = asArray(item.design.insts).length;
      return `${insts} instance${insts === 1 ? "" : "s"} · ${item.design.nets} net${
        item.design.nets === 1 ? "" : "s"
      }`;
    }
  }
}

export function fnParamType(
  t: { kind: string; name?: string; traits?: string[] } | undefined,
): string {
  if (!t) return "?";
  if (t.kind === "pin") return "Pin";
  if (t.kind === "generic") return t.name ?? "?";
  if (t.kind === "impl") return `impl ${asArray(t.traits).join(" + ")}`;
  return t.kind;
}

/// A cohdl-style one-line signature for fn (and, with "design", design)
/// pages. Designs are bare-named blocks — no generics, no parameter list.
export function fnSignature(keyword: string, name: string, fn: FnDoc): string {
  if (keyword === "design") return `design ${name}`;
  const generics = asArray(fn.generics).map((g) => {
    const bound = g?.bound?.unit ?? asArray(g?.bound?.traits).join(" + ");
    const dflt = g?.default !== undefined ? ` = ${g?.default}` : "";
    return bound ? `${g?.name}: ${bound}${dflt}` : `${g?.name}${dflt}`;
  });
  const params = asArray(fn.params).map((p) => `${p?.name}: ${fnParamType(p?.type)}`);
  const genericPart = generics.length > 0 ? `<${generics.join(", ")}>` : "";
  return `${keyword} ${name}${genericPart}(${params.join(", ")})`;
}

/// The pad detail page's field table, formatted deterministically. mm values
/// stay the emitter's own canonical strings.
export function padFacts(pad: PadDoc): { name: string; value: string }[] {
  const facts: { name: string; value: string }[] = [];
  if (pad.shape) facts.push({ name: "Shape", value: pad.shape });
  if (Array.isArray(pad.size)) facts.push({ name: "Size", value: `${pad.size.join(" × ")} mm` });
  if (pad.layer) facts.push({ name: "Layer", value: pad.layer });
  if (pad.plating) facts.push({ name: "Plating", value: pad.plating });
  if (pad.drill?.round !== undefined) {
    facts.push({ name: "Drill", value: `round ${pad.drill.round} mm` });
  }
  const slot = pad.drill?.slot;
  if (Array.isArray(slot)) {
    facts.push({ name: "Drill", value: `slot ${slot.join(" × ")} mm` });
  }
  if (pad.chamfer) {
    facts.push({ name: "Chamfer", value: `${pad.chamfer.corner}, cut ${pad.chamfer.cut} mm` });
  }
  if (pad.corner_radius !== undefined) {
    facts.push({ name: "Corner radius", value: `${pad.corner_radius} mm` });
  }
  if (pad.mask_expansion !== undefined) {
    facts.push({ name: "Mask expansion", value: `${pad.mask_expansion} mm` });
  }
  if (pad.paste != null) {
    const rect = typeof pad.paste === "object" ? pad.paste.rect : undefined;
    const annulus = typeof pad.paste === "object" ? pad.paste.segmented_annulus : undefined;
    const paste =
      pad.paste === "none"
        ? "none"
        : Array.isArray(rect)
          ? `rect ${rect.join(" × ")} mm`
          : Array.isArray(annulus)
            ? `segmented annulus ${annulus.join(" / ")} mm`
            : "";
    if (paste) facts.push({ name: "Paste", value: paste });
  }
  return facts;
}

// --- symbol layout ----------------------------------------------------------

export interface SymbolSides {
  left: DevicePinDoc[];
  right: DevicePinDoc[];
  top: DevicePinDoc[];
  bottom: DevicePinDoc[];
}

/// A ground-looking pin name: `GND` anywhere in it (AGND, DGND, PGND…), or
/// exactly `VSS` / `EP`. Compared uppercase.
export function isGroundName(name: string): boolean {
  const upper = String(name ?? "").toUpperCase();
  return upper.includes("GND") || upper === "VSS" || upper === "EP";
}

/// Box-side assignment for the auto-drawn symbol — a tooling-layer
/// convention, not a language guarantee: input/passive left; output,
/// bidirectional, power_out right; power_in top, except ground-looking
/// names, which go bottom. Order within a side follows the pin set.
export function symbolSides(pins: DevicePinDoc[]): SymbolSides {
  const sides: SymbolSides = { left: [], right: [], top: [], bottom: [] };
  for (const pin of pins) {
    if (!pin) continue;
    switch (pin.role) {
      case "output":
      case "bidirectional":
      case "power_out":
        sides.right.push(pin);
        break;
      case "power_in":
        (isGroundName(pin.name) ? sides.bottom : sides.top).push(pin);
        break;
      default: // input, passive
        sides.left.push(pin);
    }
  }
  return sides;
}

// --- footprint geometry -----------------------------------------------------

export interface Bounds {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
}

/// Axis-aligned half-extents of a pad's copper before rotation. Circle and
/// annulus sizes lead with a diameter; rect/oval are `[w, h]`.
export function padHalfSize(pad: PadDoc): { hw: number; hh: number } {
  const size = pad.size ?? [];
  if (pad.shape === "circle" || pad.shape === "annulus") {
    const r = mm(size[0]) / 2;
    return { hw: r, hh: r };
  }
  return { hw: mm(size[0]) / 2, hh: mm(size[1]) / 2 };
}

/// Half-extents of the axis-aligned bounding box of a `2hw × 2hh` box
/// rotated by `deg` (the sign cannot matter for a bounding box).
export function rotatedHalfExtents(
  hw: number,
  hh: number,
  deg: number,
): { hw: number; hh: number } {
  const rad = (deg * Math.PI) / 180;
  const c = Math.abs(Math.cos(rad));
  const s = Math.abs(Math.sin(rad));
  return { hw: hw * c + hh * s, hh: hw * s + hh * c };
}

/// The 5-vertex outline of a rect pad with one 45° chamfered corner, in the
/// y-down authoring frame (`top_*` is −y). An unknown corner name degrades
/// to the plain rectangle.
export function chamferPoints(
  hw: number,
  hh: number,
  corner: string,
  cut: number,
): [number, number][] {
  const c = Math.min(cut, hw * 2, hh * 2);
  switch (corner) {
    case "top_left":
      return [
        [-hw, -hh + c],
        [-hw + c, -hh],
        [hw, -hh],
        [hw, hh],
        [-hw, hh],
      ];
    case "top_right":
      return [
        [-hw, -hh],
        [hw - c, -hh],
        [hw, -hh + c],
        [hw, hh],
        [-hw, hh],
      ];
    case "bottom_right":
      return [
        [-hw, -hh],
        [hw, -hh],
        [hw, hh - c],
        [hw - c, hh],
        [-hw, hh],
      ];
    case "bottom_left":
      return [
        [-hw, -hh],
        [hw, -hh],
        [hw, hh],
        [-hw + c, hh],
        [-hw, hh - c],
      ];
    default:
      return [
        [-hw, -hh],
        [hw, -hh],
        [hw, hh],
        [-hw, hh],
      ];
  }
}

/// The bounding box of everything a footprint draws — pads (rotation
/// included), mount holes, courtyard, window, silkscreen (stroke widths
/// included), the `REF**` anchor — grown by `margin` mm on every side.
/// An empty footprint yields a small default box.
export function footprintBounds(
  footprint: FootprintDoc,
  pads: ReadonlyMap<string, PadDoc>,
  margin = 1,
): Bounds {
  let b: Bounds | null = null;
  const include = (x0: number, y0: number, x1: number, y1: number) => {
    if (b === null) {
      b = { minX: x0, minY: y0, maxX: x1, maxY: y1 };
    } else {
      b.minX = Math.min(b.minX, x0);
      b.minY = Math.min(b.minY, y0);
      b.maxX = Math.max(b.maxX, x1);
      b.maxY = Math.max(b.maxY, y1);
    }
  };

  for (const placement of asArray(footprint.pads)) {
    const x = mm(placement?.x);
    const y = mm(placement?.y);
    const def = pads.get(placement?.pad);
    if (!def) {
      include(x - 0.5, y - 0.5, x + 0.5, y + 0.5); // unresolved-pad marker
      continue;
    }
    const half = padHalfSize(def);
    const { hw, hh } = rotatedHalfExtents(half.hw, half.hh, placement?.rotate ?? 0);
    include(x - hw, y - hh, x + hw, y + hh);
  }

  for (const hole of asArray(footprint.mount_holes)) {
    const x = mm(hole?.x);
    const y = mm(hole?.y);
    if (hole?.diameter !== undefined) {
      const r = mm(hole.diameter) / 2;
      include(x - r, y - r, x + r, y + r);
    } else {
      const hw = mm(hole?.size?.[0]) / 2;
      const hh = mm(hole?.size?.[1]) / 2;
      include(x - hw, y - hh, x + hw, y + hh);
    }
  }

  for (const outline of [footprint.courtyard, footprint.window]) {
    if (!outline) continue;
    const x = mm(outline.at?.[0]);
    const y = mm(outline.at?.[1]);
    if (outline.shape === "circle") {
      const r = mm(outline.size?.[0]) / 2;
      include(x - r, y - r, x + r, y + r);
    } else {
      const hw = mm(outline.size?.[0]) / 2;
      const hh = mm(outline.size?.[1]) / 2;
      include(x - hw, y - hh, x + hw, y + hh);
    }
  }

  for (const g of asArray(footprint.silk)) {
    if (!g) continue;
    switch (g.kind) {
      case "line": {
        const hw = mm(g.width) / 2;
        const x0 = mm(g.from?.[0]);
        const y0 = mm(g.from?.[1]);
        const x1 = mm(g.to?.[0]);
        const y1 = mm(g.to?.[1]);
        include(
          Math.min(x0, x1) - hw,
          Math.min(y0, y1) - hw,
          Math.max(x0, x1) + hw,
          Math.max(y0, y1) + hw,
        );
        break;
      }
      case "circle": {
        const r = mm(g.radius) + mm(g.width) / 2;
        include(mm(g.at?.[0]) - r, mm(g.at?.[1]) - r, mm(g.at?.[0]) + r, mm(g.at?.[1]) + r);
        break;
      }
      case "arc": {
        // Conservative: the arc's full circle. Fitting is the goal, not
        // tightness.
        const r = mm(g.radius) + mm(g.width) / 2;
        include(mm(g.at?.[0]) - r, mm(g.at?.[1]) - r, mm(g.at?.[0]) + r, mm(g.at?.[1]) + r);
        break;
      }
      case "polygon": {
        const hw = mm(g.width) / 2;
        for (const point of asArray(g.points)) {
          const x = mm(point?.[0]);
          const y = mm(point?.[1]);
          include(x - hw, y - hw, x + hw, y + hw);
        }
        break;
      }
      default:
        break;
    }
  }

  if (footprint.silkscreen_ref) {
    // Nominal REF** text extent around its anchor.
    const x = mm(footprint.silkscreen_ref.at?.[0]);
    const y = mm(footprint.silkscreen_ref.at?.[1]);
    include(x - 1.8, y - 0.6, x + 1.8, y + 0.6);
  }

  const box: Bounds = b ?? { minX: -1, minY: -1, maxX: 1, maxY: 1 };
  return {
    minX: box.minX - margin,
    minY: box.minY - margin,
    maxX: box.maxX + margin,
    maxY: box.maxY + margin,
  };
}

// --- pin numbers and signal names -------------------------------------------

function clamp(value: number, lo: number, hi: number): number {
  return Math.min(Math.max(value, lo), hi);
}

/// Gap between a pad's copper edge and its signal label, mm.
export const SIGNAL_GAP = 0.25;

/// Electrical number → pin name for one device variant. A pin's `numbers`
/// array fans out (one entry per physical pad); on a duplicate number the
/// first writer wins, matching the pin set's declaration order.
export function buildSignalMap(
  device: DeviceDoc,
  variant: string | undefined,
): Map<string, string> {
  const map = new Map<string, string>();
  for (const pin of pinsForVariant(device, variant)) {
    if (!pin) continue;
    const name = String(pin.name ?? "");
    if (!name) continue;
    for (const number of asArray(pin.numbers)) {
      const key = String(number ?? "");
      if (key && !map.has(key)) map.set(key, name);
    }
  }
  return map;
}

export interface FootprintSignals {
  map: Map<string, string>;
  /// The bound device's short name, for the caption.
  source: string;
  /// The bound device's fq, so the caption can link it.
  deviceFq: string;
}

/// The signal names a footprint page can show: parts are scanned sorted by
/// fq (deterministic — the document's own order is not contractual), and the
/// first part bound to `fpFq` (primary or any alt) whose device resolves —
/// in `items` first, then `foreign` — supplies the number → pin-name map for
/// its variant. No resolving part means no signals, not an error.
export function signalsForFootprint(
  fpFq: string,
  items: ApiDocsItem[],
  foreign: ApiDocsItem[],
): FootprintSignals | undefined {
  const target = String(fpFq ?? "");
  if (!target) return undefined;
  const parts = asArray(items)
    .filter((item) => item?.kind === "part" && !!item.part)
    .sort((a, b) => {
      const fa = String(a.fq ?? "");
      const fb = String(b.fq ?? "");
      return fa < fb ? -1 : fa > fb ? 1 : 0;
    });
  const deviceByFq = (fq: string): DeviceDoc | undefined => {
    for (const pool of [asArray(items), asArray(foreign)]) {
      for (const item of pool) {
        if (item?.kind === "device" && item.fq === fq && item.device) return item.device;
      }
    }
    return undefined;
  };
  for (const item of parts) {
    if (item?.kind !== "part") continue;
    const part = item.part;
    const bound = [part.primary, ...asArray(part.alts)].some(
      (entry) => entry?.footprint === target,
    );
    if (!bound) continue;
    const deviceFq = String(part.device ?? "");
    const device = deviceFq ? deviceByFq(deviceFq) : undefined;
    if (!device) continue;
    return {
      map: buildSignalMap(device, part.variant),
      source: shortName(deviceFq),
      deviceFq,
    };
  }
  return undefined;
}

/// A pad-number label, centred on the pad: font size follows the pad's short
/// dimension, and the text turns −90° to run along the pad's long axis when
/// the placement's effective height clearly dominates (the placement's own
/// rotation swaps the effective extents first).
export function padNumberLabel(
  pad: PadDoc,
  rotate: number,
): { fontSize: number; rotated: boolean } {
  const half = padHalfSize(pad);
  const fontSize = clamp(0.62 * Math.min(half.hw, half.hh) * 2, 0.1, 0.8);
  const eff = rotatedHalfExtents(half.hw, half.hh, rotate);
  return { fontSize, rotated: eff.hh > 1.4 * eff.hw };
}

/// ONE signal font size for the whole footprint — the median pad short
/// dimension keeps a lone thermal pad from inflating every label.
export function signalFontSize(
  footprint: FootprintDoc,
  pads: ReadonlyMap<string, PadDoc>,
): number {
  const dims: number[] = [];
  for (const placement of asArray(footprint.pads)) {
    const def = placement ? pads.get(placement.pad) : undefined;
    if (!def) continue;
    const half = padHalfSize(def);
    dims.push(Math.min(half.hw, half.hh) * 2);
  }
  dims.sort((a, b) => a - b);
  const median =
    dims.length === 0
      ? 0
      : dims.length % 2 === 1
        ? dims[(dims.length - 1) / 2]
        : (dims[dims.length / 2 - 1] + dims[dims.length / 2]) / 2;
  return clamp(0.55 * median, 0.2, 0.7);
}

export type SignalSide = "left" | "right" | "top" | "bottom";

/// Datasheet-pinout side for a pad: the dominant axis of the pad centre
/// relative to the footprint bbox centre, ties going to the horizontal side
/// (y-down frame: −y is `top`).
export function signalSide(dx: number, dy: number): SignalSide {
  if (Math.abs(dx) >= Math.abs(dy)) return dx < 0 ? "left" : "right";
  return dy < 0 ? "top" : "bottom";
}

export interface SignalLabel {
  number: string;
  text: string;
  side: SignalSide;
  x: number;
  y: number;
  anchor: "start" | "end";
  /// Vertical text (top/bottom sides): SVG `rotate(-90)` about the anchor
  /// point, reading bottom-to-top.
  rotated: boolean;
}

/// Signal labels beside a footprint's pads. Each electrical number is
/// labelled ONCE — on its first placement in source order; repeated
/// placements (an exposed pad's extra copper) get no duplicate. Left/right
/// labels are horizontal, top/bottom vertical, all offset `SIGNAL_GAP` past
/// the placement's rotation-aware copper extent. Sides are assigned
/// relative to the CENTROID of the pad placements (the compiler's own
/// outward-marker policy) — never the drawing bounds, which the REF**
/// anchor or silkscreen art can skew off the land pattern's centre.
export function signalLabels(
  footprint: FootprintDoc,
  pads: ReadonlyMap<string, PadDoc>,
  signals: ReadonlyMap<string, string>,
): SignalLabel[] {
  const placements = asArray(footprint.pads).filter(Boolean);
  let cx = 0;
  let cy = 0;
  if (placements.length > 0) {
    for (const p of placements) {
      cx += mm(p.x);
      cy += mm(p.y);
    }
    cx /= placements.length;
    cy /= placements.length;
  }
  const labels: SignalLabel[] = [];
  const seen = new Set<string>();
  for (const placement of asArray(footprint.pads)) {
    if (!placement) continue;
    const number = String(placement.number ?? "");
    if (!number || seen.has(number)) continue;
    seen.add(number);
    const text = signals.get(number);
    if (text === undefined) continue;
    const def = pads.get(placement.pad);
    if (!def) continue;
    const half = padHalfSize(def);
    const { hw, hh } = rotatedHalfExtents(half.hw, half.hh, placement.rotate ?? 0);
    const x = mm(placement.x);
    const y = mm(placement.y);
    const side = signalSide(x - cx, y - cy);
    switch (side) {
      case "left":
        labels.push({ number, text, side, x: x - hw - SIGNAL_GAP, y, anchor: "end", rotated: false });
        break;
      case "right":
        labels.push({ number, text, side, x: x + hw + SIGNAL_GAP, y, anchor: "start", rotated: false });
        break;
      case "top":
        labels.push({ number, text, side, x, y: y - hh - SIGNAL_GAP, anchor: "start", rotated: true });
        break;
      case "bottom":
        labels.push({ number, text, side, x, y: y + hh + SIGNAL_GAP, anchor: "end", rotated: true });
        break;
    }
  }
  return labels;
}

export interface SideLengths {
  left: number;
  right: number;
  top: number;
  bottom: number;
}

/// Extra viewBox margin per side for the signal labels: the longest label's
/// estimated extent (chars × fontSize × 0.62 for the mono advance) plus the
/// gap and one line height of slack. A side with no labels gets none.
export function signalMargins(
  longest: SideLengths,
  fontSize: number,
  gap: number,
): SideLengths {
  const extent = (chars: number) => (chars > 0 ? chars * fontSize * 0.62 + gap + fontSize : 0);
  return {
    left: extent(longest.left),
    right: extent(longest.right),
    top: extent(longest.top),
    bottom: extent(longest.bottom),
  };
}

// --- arcs and scale ---------------------------------------------------------

function fmtNum(n: number): string {
  const rounded = Math.round(n * 10000) / 10000;
  return String(rounded === 0 ? 0 : rounded); // never "-0"
}

/// A point on the circle at `deg` in the authoring frame: angles are
/// counter-clockwise positive with y down, so +90° sits at +y — below the
/// centre on screen, i.e. the sweep appears clockwise.
export function arcPoint(cx: number, cy: number, r: number, deg: number): [number, number] {
  const rad = (deg * Math.PI) / 180;
  return [cx + r * Math.cos(rad), cy + r * Math.sin(rad)];
}

/// SVG path for a silkscreen arc: the compiler sweeps MONOTONICALLY from
/// `start_angle` to `end_angle` (either bound may be the larger — see
/// `emit::silk`'s `start + (end - start) * t` interpolation), so the SVG
/// sweep flag follows the sign of the delta. A zero-length sweep is a
/// degenerate point and draws nothing; a full ±360° turn is drawn as two
/// half arcs because a single SVG arc cannot close on itself.
export function arcPath(
  cx: number,
  cy: number,
  r: number,
  startDeg: number,
  endDeg: number,
): string {
  const delta = endDeg - startDeg;
  if (delta === 0) return "";
  const [x0, y0] = arcPoint(cx, cy, r, startDeg);
  const rs = fmtNum(r);
  const sweep = delta > 0 ? 1 : 0;
  if (Math.abs(delta) === 360) {
    const [xm, ym] = arcPoint(cx, cy, r, startDeg + 180);
    return (
      `M ${fmtNum(x0)} ${fmtNum(y0)} ` +
      `A ${rs} ${rs} 0 0 ${sweep} ${fmtNum(xm)} ${fmtNum(ym)} ` +
      `A ${rs} ${rs} 0 0 ${sweep} ${fmtNum(x0)} ${fmtNum(y0)}`
    );
  }
  const [x1, y1] = arcPoint(cx, cy, r, endDeg);
  const large = Math.abs(delta) > 180 ? 1 : 0;
  return `M ${fmtNum(x0)} ${fmtNum(y0)} A ${rs} ${rs} 0 ${large} ${sweep} ${fmtNum(x1)} ${fmtNum(y1)}`;
}

/// Scale-bar length in mm: the largest 1–2–5 progression value at most a
/// quarter of the view width (floored at 0.01 mm).
export function scaleBarMm(viewWidth: number): number {
  const target = Math.max(viewWidth / 4, 0.01);
  let pick = 0.01;
  for (const exp of [-2, -1, 0, 1, 2]) {
    for (const mantissa of [1, 2, 5]) {
      const value = mantissa * 10 ** exp;
      if (value <= target) pick = value;
    }
  }
  return pick;
}
