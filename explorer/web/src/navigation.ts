import type { ExplorerModel } from './model'
import { explorerNets } from './hierarchy'
import type { InspectionContext } from './inspection'
import { buildViewGraph, groupNodeId, type ViewConfig, type ViewDef } from './views'
import { coreCandidates } from './partPresentation'
import { validViewport, type CanvasSize, type BoardViewportSnapshot } from './viewport'
import type { ViewportSnapshot } from './useCanvasFit'

const record = (value: unknown): value is Record<string, unknown> => !!value && typeof value === 'object' && !Array.isArray(value)
const strings = (value: unknown) => Array.isArray(value) ? [...new Set(value.filter((v): v is string => typeof v === 'string'))] : []
const validSize = (value: unknown): value is CanvasSize => record(value) && [value.width, value.height].every((v) => typeof v === 'number' && Number.isFinite(v) && v > 0 && v <= 1000000)
const canvasSnapshot = (value: unknown): ViewportSnapshot | null => record(value) && validViewport(value.viewport) && Math.abs(value.viewport.x) <= 1000000000 && Math.abs(value.viewport.y) <= 1000000000 && validSize(value.size) && typeof value.autoFit === 'boolean'
  ? { viewport: value.viewport, size: value.size, autoFit: value.autoFit } : null
const boardSnapshot = (value: unknown): BoardViewportSnapshot | null => {
  if (!record(value) || !record(value.view) || !validSize(value.size) || typeof value.autoFit !== 'boolean') return null
  const v = value.view
  if (![v.x, v.y, v.width, v.height].every((n) => typeof n === 'number' && Number.isFinite(n) && Math.abs(n) <= 1000000000) || Number(v.width) <= 0 || Number(v.height) <= 0) return null
  return { view: { x: Number(v.x), y: Number(v.y), width: Number(v.width), height: Number(v.height) }, size: value.size, autoFit: value.autoFit }
}

export function normalizeContext(value: unknown, model: ExplorerModel, config: ViewConfig | null): InspectionContext {
  const input = record(value) ? value : {}
  const views = config?.views.filter(v => v.name !== 'Source pages') ?? []
  const view = typeof input.view === 'string' && input.view !== 'Source pages' && (!config || !input.view || views.some((v) => v.name === input.view)) ? input.view : views.find(v => v.name === 'Functional')?.name ?? views[0]?.name ?? ''
  const definition = config?.views.find((v) => v.name === view)
  const flat = buildViewGraph(model, null, definition)
  const names = flat.regions?.regions ?? []
  const scope = input.scope === '' || (typeof input.scope === 'string' && model.subdesigns?.some((s) => s.path === input.scope)) ? input.scope : null
  const scoped = scope === null ? flat : buildViewGraph(model, scope, definition)
  const parts = new Set([...model.instances.map((i) => i.path), ...scoped.graph.nodes.map((n) => n.id), ...flat.graph.nodes.map((n) => n.id), ...(model.subdesigns ?? []).map((s) => s.path), ...names.map((n) => groupNodeId(view, n))])
  if (!config) for (const group of [...strings(input.groups), ...strings(input.collapsed)]) parts.add(groupNodeId(view, group))
  const selection = strings(input.selection).filter((id) => parts.has(id))
  const pin = typeof input.pin === 'string' && selection.length === 1 && model.instances.find((i) => i.path === selection[0])?.pins.some((p) => p.logical === input.pin) ? input.pin : ''
  return {
    mode: input.mode === 'overview' || input.mode === 'layout' ? input.mode : 'sch',
    connectionMode: input.connectionMode === 'bundled' ? 'bundled' : 'individual',
    scope,
    view, groups: strings(input.groups).filter((g) => !config || names.includes(g)), collapsed: strings(input.collapsed).filter((g) => !config || names.includes(g)),
    selection, pin, net: typeof input.net === 'string' && explorerNets(model).some((n) => n.name === input.net) ? input.net : '',
    viewport: canvasSnapshot(input.viewport), boardViewport: boardSnapshot(input.boardViewport),
    layoutFrame: input.layoutFrame === 'board' ? 'board' : 'local', layoutSide: input.layoutSide === 'top' || input.layoutSide === 'bottom' ? input.layoutSide : 'both',
    // Legacy issue/proposal URLs intentionally fall back to the core circuit view.
    tab: input.tab === 'bom' ? 'bom' : 'circuit',
  }
}

export function defaultInsideGroups(model: ExplorerModel, definition?: ViewDef, sourceValues: ReadonlyMap<string, string> = new Map()): string[] {
  const graph = buildViewGraph(model, null, definition)
  const nonempty = graph.regions?.regions.filter(name => [...graph.instanceRegions.values()].includes(name)) ?? []
  const core = coreCandidates(model, sourceValues).find(candidate => nonempty.includes(graph.instanceRegions.get(candidate.instance.path) ?? ''))
  const preferred = core ? graph.instanceRegions.get(core.instance.path) : nonempty[0]
  return preferred ? [preferred] : []
}

export function parseInspectionUrl(url: URL, model: ExplorerModel, config: ViewConfig | null, sourceValues: ReadonlyMap<string, string> = new Map()): InspectionContext {
  const params = url.searchParams
  const index = (input: Record<string, unknown>) => {
    const context = normalizeContext({ ...input, mode: 'sch' }, model, config)
    const definition = config?.views.find(view => view.name === context.view)
    return { ...context, scope: !definition && model.subdesigns?.length ? '' : context.scope,
      groups: defaultInsideGroups(model, definition, sourceValues), collapsed: [] }
  }
  if (params.has('inspect')) {
    try {
      const input = JSON.parse(params.get('inspect')!)
      if (record(input) && typeof input.view === 'string' && input.view && config && !config.views.some(view => view.name === input.view)) return index({})
      return normalizeContext(input, model, config)
    } catch { return index({}) }
  }
  const input = { mode: params.get('mode'), scope: params.has('scope') ? params.get('scope') === 'all' ? null : params.get('scope') : null,
    view: params.has('view') ? params.get('view') : undefined, selection: params.get('select')?.split(',') ?? [], net: params.get('net') }
  return !params.has('mode') && !params.has('select') && !params.has('net') && !params.has('scope') ? index(input) : normalizeContext(input, model, config)
}

export function inspectionUrl(url: URL, context: InspectionContext): URL {
  const next = new URL(url)
  for (const name of ['mode', 'scope', 'view', 'select', 'net']) next.searchParams.delete(name)
  next.searchParams.set('inspect', JSON.stringify({ ...context, version: 2 }))
  return next
}

export const contextKey = (context: Pick<InspectionContext, 'mode' | 'scope' | 'view'>) => JSON.stringify([context.mode, context.scope, context.view])
