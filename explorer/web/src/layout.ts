import ELK from 'elkjs/lib/elk.bundled.js'
import type { ElkNode, ElkExtendedEdge } from 'elkjs/lib/elk-api'
import type { Graph, GNode, GEdge } from './transform'
import type { RegionAssign } from './views'
import { compactOverview } from './compactLayout'
import { bundleLabelSize } from './connectivity'

const elk = new ELK()
type Point = { x: number; y: number }
export interface EdgeRoute {
  points: Point[]
  label: Point
}
export interface Positioned {
  /** Region children use parent-relative positions, matching React Flow. */
  positions: Map<string, Point>
  regionBoxes: Map<string, Point & { width: number; height: number }>
  /** Routes and labels use absolute graph coordinates, including container offsets. */
  edgeRoutes: Map<string, EdgeRoute>
}

const BASE = {
  'elk.algorithm': 'layered',
  'elk.direction': 'RIGHT',
  'elk.edgeRouting': 'ORTHOGONAL',
  'elk.spacing.nodeNode': '56',
  'elk.layered.spacing.nodeNodeBetweenLayers': '140',
  'elk.spacing.edgeNode': '32',
  'elk.spacing.edgeEdge': '18',
  'elk.layered.spacing.edgeEdgeBetweenLayers': '18',
  'elk.layered.spacing.edgeNodeBetweenLayers': '28',
}

const portId = (id: string, side: string) => JSON.stringify([id, side])
function elkNode(node: GNode): ElkNode {
  return {
    id: node.id, width: node.width, height: node.height,
    layoutOptions: { 'elk.portConstraints': 'FIXED_POS' },
    ports: [
      { id: portId(node.id, 'in'), x: 0, y: node.height / 2, width: 0, height: 0, layoutOptions: { 'elk.port.side': 'WEST' } },
      { id: portId(node.id, 'out'), x: node.width, y: node.height / 2, width: 0, height: 0, layoutOptions: { 'elk.port.side': 'EAST' } },
    ],
  }
}
function elkEdge(edge: GEdge): ElkExtendedEdge {
  const size = edge.id.startsWith('bundle:') ? bundleLabelSize(edge.label) : { width: edge.label.length * 7 + 12, height: 20 }
  return {
    id: edge.id, sources: [portId(edge.source, 'out')], targets: [portId(edge.target, 'in')],
    labels: edge.label ? [{ text: edge.label, width: size.width, height: size.height }] : [],
  }
}

export async function layout(g: Graph, regions?: RegionAssign, opts?: { compact?: boolean; aspectRatio?: number; columns?: number }): Promise<Positioned> {
  if (opts?.compact && !regions && g.nodes.every((node) => node.kind === 'net' || node.id.startsWith('group:'))
    && g.edges.every((edge) => edge.nets.length === 0 || edge.id.startsWith('bundle:')))
    return compactOverview(g, opts.aspectRatio ?? 1.6, opts.columns)
  const spacing = opts?.compact ? {
    ...BASE, 'elk.spacing.nodeNode': '30', 'elk.layered.spacing.nodeNodeBetweenLayers': '80',
    'elk.spacing.edgeNode': '20', 'elk.spacing.edgeEdge': '12',
    'elk.layered.spacing.edgeEdgeBetweenLayers': '12', 'elk.layered.spacing.edgeNodeBetweenLayers': '18',
  } : BASE
  const byRegion = new Map<string, GNode[]>()
  for (const node of g.nodes) {
    const name = regions?.byNode.get(node.id) ?? 'Other'
    const members = byRegion.get(name) ?? []
    members.push(node)
    byRegion.set(name, members)
  }
  const children = regions ? [...byRegion].map(([name, nodes]) => ({
    id: `region:${name}`,
    layoutOptions: { ...spacing, 'elk.padding': '[top=40,left=14,bottom=14,right=14]' },
    children: nodes.map(elkNode),
  })) : g.nodes.map(elkNode)
  const result = await elk.layout({
    id: 'root', layoutOptions: { ...spacing, 'elk.hierarchyHandling': 'INCLUDE_CHILDREN' },
    children, edges: g.edges.map(elkEdge),
  })
  return positioned(result, Boolean(regions))
}

function positioned(root: ElkNode, compound: boolean): Positioned {
  const positions: Positioned['positions'] = new Map()
  const regionBoxes: Positioned['regionBoxes'] = new Map()
  const edgeRoutes: Positioned['edgeRoutes'] = new Map()
  const offsets = new Map<string, Point>()
  function collect(node: ElkNode, parent: Point) {
    const absolute = { x: parent.x + (node.x ?? 0), y: parent.y + (node.y ?? 0) }
    offsets.set(node.id, absolute)
    if (node.id !== 'root') {
      if (compound && node.children) regionBoxes.set(node.id, { ...absolute, width: node.width ?? 0, height: node.height ?? 0 })
      else positions.set(node.id, { x: node.x ?? 0, y: node.y ?? 0 })
    }
    for (const child of node.children ?? []) collect(child, absolute)
  }
  collect(root, { x: 0, y: 0 })
  function routes(owner: ElkNode) {
    for (const edge of owner.edges ?? []) {
      // ELK keeps compound edges at root but sections can be region-local.
      const container = (edge as ElkExtendedEdge & { container?: string }).container ?? owner.id
      const offset = offsets.get(container)
      if (!offset) throw new Error(`Missing ELK edge container: ${container}`)
      const points = sectionPoints(edge).map((point) => ({ x: point.x + offset.x, y: point.y + offset.y }))
      const label = edge.labels?.[0]
      const center = label && label.x !== undefined && label.y !== undefined
        ? { x: label.x + (label.width ?? 0) / 2 + offset.x, y: label.y + (label.height ?? 0) / 2 + offset.y }
        : points[Math.floor(points.length / 2)]
      edgeRoutes.set(edge.id, { points, label: center })
    }
    for (const child of owner.children ?? []) routes(child)
  }
  routes(root)
  return { positions, regionBoxes, edgeRoutes }
}

function sectionPoints(edge: ElkExtendedEdge): Point[] {
  const remaining = [...(edge.sections ?? [])]
  if (!remaining.length) throw new Error(`ELK returned no orthogonal route for ${edge.id}`)
  const points: Point[] = []
  while (remaining.length) {
    const index = points.length ? remaining.findIndex((section) => section.startPoint.x === points[points.length - 1].x && section.startPoint.y === points[points.length - 1].y)
      : remaining.findIndex((section) => !section.incomingSections?.length)
    if (index < 0) throw new Error(`Disconnected ELK route for ${edge.id}`)
    const [section] = remaining.splice(index, 1)
    if (!points.length) points.push(section.startPoint)
    points.push(...(section.bendPoints ?? []), section.endPoint)
  }
  return points
}
