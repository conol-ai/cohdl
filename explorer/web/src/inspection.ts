import type { ExplorerModel, Net } from './model'
import type { ViewGraph } from './views'
import { representedInstances } from './views'
import type { ViewportSnapshot } from './useCanvasFit'
import type { BoardViewportSnapshot } from './viewport'
import type { LayoutSide } from './boardGeometry'

export interface InspectionContext {
  mode: 'overview' | 'sch' | 'layout'
  connectionMode: 'individual' | 'bundled'
  scope: string | null
  view: string
  groups: string[]
  selection: string[]
  net: string
  viewport: ViewportSnapshot | null
  boardViewport: BoardViewportSnapshot | null
  layoutFrame: 'board' | 'local'
  layoutSide: LayoutSide
  collapsed: string[]
  pin: string
  tab: 'circuit' | 'bom'
}

export function netEndpoints(model: ExplorerModel, net: Net, full: ViewGraph, visible: ViewGraph['graph'] | ReadonlySet<string>) {
  const represented = 'nodes' in visible ? representedInstances(model, visible) : visible
  return net.members.map((member) => ({
    ...member,
    group: full.instanceRegions.get(member.instance_path) ?? 'Other',
    visible: represented.has(member.instance_path),
  }))
}
