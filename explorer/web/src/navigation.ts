import type { ExplorerModel } from './model'
import { explorerNets } from './hierarchy'
import type { InspectionContext } from './inspection'
import { buildViewGraph, groupNodeId, type ViewConfig } from './views'
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
    tab: input.tab === 'bom' || input.tab === 'issues' || input.tab === 'proposals' ? input.tab : 'circuit',
  }
}

export function parseInspectionUrl(url: URL, model: ExplorerModel, config: ViewConfig | null): InspectionContext {
  const params = url.searchParams
  const index = (input: Record<string, unknown>) => {
    const context = normalizeContext({ ...input, mode: 'overview' }, model, config)
    const definition = config?.views.find(view => view.name === context.view)
    return { ...context, scope: !definition && model.subdesigns?.length ? '' : context.scope,
      collapsed: buildViewGraph(model, null, definition).regions?.regions ?? [] }
  }
  if (params.has('inspect')) {
    try {
      const input = JSON.parse(params.get('inspect')!)
      if (record(input) && input.version !== 2 && input.mode === 'overview' && !strings(input.groups).length && !strings(input.selection).length)
        return index({ ...input, viewport: null })
      const migrated = record(input) && input.version !== 2 && input.mode === 'overview'
        ? { ...input, mode: 'sch', connectionMode: 'individual', collapsed: [], viewport: null } : input
      return normalizeContext(migrated, model, config)
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
