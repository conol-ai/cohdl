// View/region config (spec G1): display-only partitioning loaded from
// views/<design>.view.json. Member rules: designator ("U2"), explicit
// path ("path:Pico2::mcu"), or aggregate key ("agg:GND + V3V3").
// Unmatched nodes fall into an implicit "其他" region.

import type { Graph, GNode } from './transform'

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
