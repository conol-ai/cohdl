import type { Graph, GEdge } from './transform'
import type { Positioned, EdgeRoute } from './layout'

type Point = { x: number; y: number }
type LabelBox = Point & { width: number }
const COLUMN_GAP = 96
const ROW_GAP = 76
const LABEL_TRACK = 22

/** Folded modules occupy fixed cells; all wire turns stay in reserved gutters. */
export function compactOverview(graph: Graph, aspectRatio: number): Positioned {
  const positions: Positioned['positions'] = new Map()
  const regionBoxes: Positioned['regionBoxes'] = new Map()
  const edgeRoutes: Positioned['edgeRoutes'] = new Map()
  if (!graph.nodes.length) return { positions, regionBoxes, edgeRoutes }
  const width = Math.max(...graph.nodes.map((node) => node.width))
  const height = Math.max(...graph.nodes.map((node) => node.height))
  const ratio = Number.isFinite(aspectRatio) && aspectRatio > 0 ? Math.min(4, Math.max(0.25, aspectRatio)) : 1.6
  const columns = Math.min(graph.nodes.length, Math.ceil(Math.sqrt(graph.nodes.length * ratio * (height + ROW_GAP) / (width + COLUMN_GAP))))
  const rows = Math.ceil(graph.nodes.length / columns)
  const cells = new Map<string, { column: number; row: number }>()
  graph.nodes.forEach((node, index) => {
    const column = index % columns, row = Math.floor(index / columns)
    cells.set(node.id, { column, row })
    positions.set(node.id, {
      x: COLUMN_GAP / 2 + column * (width + COLUMN_GAP) + (width - node.width) / 2,
      y: ROW_GAP + row * (height + ROW_GAP) + (height - node.height) / 2,
    })
  })
  const nodes = new Map(graph.nodes.map((node) => [node.id, node]))
  const labels: LabelBox[] = []
  for (const edge of graph.edges) {
    const source = nodes.get(edge.source), target = nodes.get(edge.target)
    const sourceCell = cells.get(edge.source), targetCell = cells.get(edge.target)
    const sp = positions.get(edge.source), tp = positions.get(edge.target)
    if (!source || !target || !sourceCell || !targetCell || !sp || !tp) throw new Error(`Unknown compact edge endpoint: ${edge.id}`)
    const start = { x: sp.x + source.width, y: sp.y + source.height / 2 }
    const end = { x: tp.x, y: tp.y + target.height / 2 }
    const right = (sourceCell.column + 1) * (width + COLUMN_GAP)
    const left = targetCell.column * (width + COLUMN_GAP)
    const preferredRow = Math.max(1, Math.round((sourceCell.row + targetCell.row + 1) / 2))
    const label = gutterLabel(edge, right, left, preferredRow, rows, height, labels)
    const points = [start, { x: right, y: start.y }, { x: right, y: label.y },
      { x: left, y: label.y }, { x: left, y: end.y }, end]
    edgeRoutes.set(edge.id, { points, label } satisfies EdgeRoute)
  }
  return { positions, regionBoxes, edgeRoutes }
}

function gutterLabel(edge: GEdge, right: number, left: number, preferredRow: number, rows: number, height: number, occupied: LabelBox[]): Point {
  const rowIndices = Array.from({ length: Math.max(1, rows - 1) }, (_, index) => rows > 1 ? index + 1 : 0)
    .sort((a, b) => Math.abs(a - preferredRow) - Math.abs(b - preferredRow))
  const width = edge.label.length * 7 + 12
  const middle = (left + right) / 2
  const candidates = [middle]
  for (let offset = width + 8; offset <= Math.abs(left - right) / 2; offset += width + 8) candidates.push(middle - offset, middle + offset)
  for (const row of rowIndices) {
    const base = ROW_GAP / 2 + row * (height + ROW_GAP)
    for (const track of [0, -LABEL_TRACK, LABEL_TRACK]) {
      for (const x of candidates) {
        const point = { x, y: base + track }
        if (!edge.label) return point
        if (occupied.some((box) => Math.abs(box.x - x) < (box.width + width) / 2 + 4 && Math.abs(box.y - point.y) < LABEL_TRACK)) continue
        occupied.push({ ...point, width })
        return point
      }
    }
  }
  throw new Error(`No free compact count-label corridor: ${edge.id}`)
}
