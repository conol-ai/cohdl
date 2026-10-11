// ELK layered layout over the display graph.

import ELK from 'elkjs/lib/elk.bundled.js'
import type { Graph } from './transform'
import type { RegionAssign } from './views'

const elk = new ELK()

export interface Positioned {
  /** node id -> absolute position; for region children, relative to parent. */
  positions: Map<string, { x: number; y: number }>
  /** region id -> {x,y,width,height} (absolute). */
  regionBoxes: Map<string, { x: number; y: number; width: number; height: number }>
}

const BASE = {
  'elk.algorithm': 'layered',
  'elk.direction': 'RIGHT',
  'elk.spacing.nodeNode': '56',
  'elk.layered.spacing.nodeNodeBetweenLayers': '140',
  'elk.spacing.edgeNode': '32',
  'elk.spacing.edgeEdge': '18',
  'elk.layered.spacing.edgeEdgeBetweenLayers': '18',
  'elk.layered.spacing.edgeNodeBetweenLayers': '28',
}

export async function layout(
  g: Graph,
  regions?: RegionAssign,
  opts?: { compact?: boolean },
): Promise<Positioned> {
  // Fully folded overviews pack native regions; per-pin routing must not
  // stretch their summary blocks into a nearly unreadable vertical graph.
  const packed = opts?.compact && g.nodes.some((n) => n.id.startsWith('group:')) && g.nodes.every((n) => n.kind === 'net' || n.id.startsWith('group:'))
  const SP = packed
    ? { ...BASE, 'elk.algorithm': 'rectpacking', 'elk.aspectRatio': '2.2', 'elk.spacing.nodeNode': '16' }
    : opts?.compact
    ? {
        ...BASE,
        'elk.spacing.nodeNode': '30',
        'elk.layered.spacing.nodeNodeBetweenLayers': '80',
        'elk.spacing.edgeNode': '20',
        'elk.spacing.edgeEdge': '12',
        'elk.layered.spacing.edgeEdgeBetweenLayers': '12',
        'elk.layered.spacing.edgeNodeBetweenLayers': '18',
      }
    : BASE
  const positions = new Map<string, { x: number; y: number }>()
  const regionBoxes = new Map<string, { x: number; y: number; width: number; height: number }>()
  if (!regions) {
    const res = await elk.layout({
      id: 'root',
      layoutOptions: SP,
      children: g.nodes.map((n) => ({ id: n.id, width: n.width, height: n.height })),
      edges: g.edges.map((e) => ({ id: e.id, sources: [e.source], targets: [e.target] })),
    })
    for (const c of res.children ?? []) positions.set(c.id!, { x: c.x ?? 0, y: c.y ?? 0 })
    return { positions, regionBoxes }
  }
  // Compound layout: one ELK parent per region (G3).
  const byRegion = new Map<string, typeof g.nodes>()
  for (const n of g.nodes) {
    const r = regions.byNode.get(n.id) ?? 'Other'
    byRegion.set(r, [...(byRegion.get(r) ?? []), n])
  }
  const res = await elk.layout({
    id: 'root',
    layoutOptions: { ...SP, 'elk.hierarchyHandling': 'INCLUDE_CHILDREN' },
    children: [...byRegion.entries()].map(([r, ns]) => ({
      id: `region:${r}`,
      layoutOptions: {
        ...SP,
        ...(packed ? { 'elk.algorithm': 'box', 'elk.spacing.nodeNode': '10' } : {}),
        'elk.padding': packed ? '[top=32,left=10,bottom=10,right=10]' : '[top=40,left=14,bottom=14,right=14]',
      },
      children: ns.map((n) => ({ id: n.id, width: n.width, height: n.height })),
    })),
    edges: g.edges.map((e) => ({ id: e.id, sources: [e.source], targets: [e.target] })),
  })
  for (const rc of res.children ?? []) {
    regionBoxes.set(rc.id!, {
      x: rc.x ?? 0,
      y: rc.y ?? 0,
      width: rc.width ?? 100,
      height: rc.height ?? 100,
    })
    for (const c of rc.children ?? []) positions.set(c.id!, { x: c.x ?? 0, y: c.y ?? 0 })
  }
  return { positions, regionBoxes }
}
