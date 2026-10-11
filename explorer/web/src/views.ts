// View/region config (spec G1): display-only partitioning loaded from
// views/<design>.view.json. Member rules: designator ("U2"), explicit
// path ("path:Pico2::mcu"), or aggregate key ("agg:GND + V3V3").
// Unmatched nodes fall into an implicit "其他" region.

import { buildGraph, reconnectGraph, type Graph, type GNode } from './transform'
import type { ExplorerModel, Instance } from './model'
import { projectScope } from './hierarchy'

export interface ViewDef {
  name: string
  regions: { name: string; members: string[] }[]
}

export interface ViewConfig {
  schema_version: number
  design: string
  views: ViewDef[]
}

export interface RegionAssign {
  /** node id -> region name (regions are `region:<view>:<name>`). */
  byNode: Map<string, string>
  regions: string[]
}

const record = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value)

function checkedName(value: unknown, where: string, seen: Set<string>): string {
  if (typeof value !== 'string' || !value.trim() || value !== value.trim() || seen.has(value))
    throw new Error(`Invalid view config: ${where} must have a unique, nonempty trimmed name`)
  seen.add(value)
  return value
}

export function parseViewConfig(value: unknown, design: string): ViewConfig {
  const fail = (detail: string): never => { throw new Error(`Invalid view config: ${detail}`) }
  if (!record(value) || value.schema_version !== 1) return fail('schema_version must be 1')
  if (value.design !== design) return fail(`design must match ${design}`)
  if (!Array.isArray(value.views) || !value.views.length) return fail('views must be a nonempty array')
  const names = new Set<string>()
  const views = value.views.map((view, index): ViewDef => {
    if (!record(view)) return fail(`view ${index} must be an object`)
    const name = checkedName(view.name, 'view', names)
    if (!Array.isArray(view.regions)) return fail(`view ${name} regions must be an array`)
    const regions = new Set<string>()
    return { name, regions: view.regions.map((region) => {
      if (!record(region)) return fail(`region in ${name} must be an object`)
      const regionName = checkedName(region.name, 'region', regions)
      if (regionName.toLowerCase() === 'other') return fail('Other is reserved for unmatched members')
      if (!Array.isArray(region.members)) return fail(`region ${regionName} members must be an array`)
      const members = region.members.map((member) => {
        if (typeof member !== 'string' || !member.trim() || member !== member.trim())
          return fail(`region ${regionName} members must be nonempty trimmed strings`)
        if (member.includes(':') && !/^(path|agg):\S/.test(member)) return fail(`invalid member rule ${member}`)
        if (member.startsWith('agg:host:') && !/^agg:host:\S/.test(member)) return fail(`invalid host member rule ${member}`)
        return member
      })
      return { name: regionName, members }
    }) }
  })
  return { schema_version: 1, design, views }
}

export function assignRegions(g: Graph, view: ViewDef): RegionAssign {
  const byNode = new Map<string, string>()
  const match = (n: GNode, rule: string): boolean => {
    if (rule.startsWith('path:')) return n.id === rule.slice(5)
    if (rule.startsWith('agg:')) return n.id === `agg:${rule.slice(4)}`
    // designator rule
    return n.inst?.designator === rule
  }
  for (const r of view.regions)
    for (const n of g.nodes)
      if (!byNode.has(n.id) && r.members.some((m) => match(n, m))) byNode.set(n.id, r.name)
  const rest = g.nodes.filter((n) => !byNode.has(n.id))
  const regions = view.regions.map((r) => r.name)
  if (rest.length > 0) {
    regions.push('Other')
    for (const n of rest) byNode.set(n.id, 'Other')
  }
  return { byNode, regions }
}

export interface ViewGraph {
  graph: Graph
  regions?: RegionAssign
  instanceRegions: Map<string, string>
  view?: ViewDef
}

/** Classify real instances before aggregation so groups cannot swallow each other. */
export function buildViewGraph(model: ExplorerModel, scope: string | null, view?: ViewDef): ViewGraph {
  const original = buildGraph(model, null)
  const instanceRegions = new Map<string, string>()
  if (!view) return { graph: scope === null ? original : buildGraph(model, scope), instanceRegions }
  const matches = (instance: Instance, rule: string) => rule.startsWith('path:')
    ? instance.path === rule.slice(5)
    : rule.startsWith('agg:') ? original.location.get(instance.path) === rule : instance.designator === rule
  for (const instance of model.instances)
    instanceRegions.set(instance.path, view.regions.find((r) => r.members.some((rule) => matches(instance, rule)))?.name ?? 'Other')
  const graph = buildGraph(model, scope, instanceRegions)
  const regions = assignRegions(graph, view)
  for (const node of graph.nodes) {
    const member = node.inst?.path ?? node.aggMembers?.[0]
    if (member) regions.byNode.set(node.id, instanceRegions.get(member) ?? 'Other')
  }
  regions.regions = [...view.regions.map((r) => r.name), ...([...regions.byNode.values()].includes('Other') ? ['Other'] : [])]
  return { graph, regions, instanceRegions, view }
}

/** Real instance paths represented by a component, aggregate or hierarchy block. */
export function representedInstances(model: ExplorerModel, graph: Graph): Set<string> {
  const blocks = graph.nodes.flatMap((n) => n.subdesign ? [n.subdesign.path] : [])
  return new Set(model.instances.filter((i) => graph.location.has(i.path) || blocks.some((p) => i.path.startsWith(`${p}::`))).map((i) => i.path))
}

export function projectGroupUnion(model: ExplorerModel, scope: string | null, full: ViewGraph, selected: readonly string[]) {
  // Functional groups represent physical members across hierarchy boundaries.
  if (selected.length && scope !== null) return projectGroupUnion(model, null, buildViewGraph(model, null, full.view), selected)
  const keep = new Set(full.graph.nodes.filter((n) => !selected.length || selected.includes(full.regions?.byNode.get(n.id) ?? 'Other')).map((n) => n.id))
  const location = new Map([...full.graph.location].filter(([, id]) => keep.has(id)))
  const graph = reconnectGraph({ ...full.graph, nodes: full.graph.nodes.filter((n) => keep.has(n.id)), location }, projectScope(model, scope).model.nets)
  const regions = full.regions ? { ...full.regions, byNode: new Map(full.regions.byNode) } : undefined
  // Junctions are routing infrastructure; attach them to a represented endpoint's group.
  for (const node of graph.nodes.filter((n) => n.kind === 'net')) {
    const edge = graph.edges.find((e) => e.source === node.id || e.target === node.id)
    const endpoint = edge && (edge.source === node.id ? edge.target : edge.source)
    if (endpoint && regions) regions.byNode.set(node.id, regions.byNode.get(endpoint) ?? 'Other')
  }
  const represented = representedInstances(model, graph)
  const counts = {
    parts: represented.size, totalParts: model.instances.length,
    nets: new Set(model.nets.filter((n) => n.members.some((m) => represented.has(m.instance_path))).map((n) => n.name)).size,
    totalNets: new Set(model.nets.map((n) => n.name)).size,
  }
  return { graph, regions, counts, represented }
}
