import type { BoardOutline, ExplorerModel, FootprintGeo, FpShape, Matrix, Placement } from './model'

export type Point = [number, number]
export interface Bounds { x: number; y: number; width: number; height: number }
export const identity: Matrix = [1, 0, 0, 1]

// Coefficients are supplied by the compiler's fixed-point trig. Geometry is
// never laid out or rotated afresh by ELK or platform trigonometry here.
export function transformPoint([x, y]: Point, [a, b, c, d]: Matrix, at: Point = [0, 0]): Point {
  return [a * x + c * y + at[0], b * x + d * y + at[1]]
}
export const svgMatrix = (m: Matrix, at: Point = [0, 0]) => `matrix(${[...m, ...at].join(' ')})`

export function boundsOf(points: Point[]): Bounds {
  if (!points.length) return { x: -10, y: -10, width: 20, height: 20 }
  let x = Infinity, y = Infinity, right = -Infinity, bottom = -Infinity
  for (const [px, py] of points) {
    x = Math.min(x, px); y = Math.min(y, py)
    right = Math.max(right, px); bottom = Math.max(bottom, py)
  }
  return { x, y, width: Math.max(0.1, right - x), height: Math.max(0.1, bottom - y) }
}
export function corners(b: Bounds): Point[] {
  return [[b.x, b.y], [b.x + b.width, b.y], [b.x + b.width, b.y + b.height], [b.x, b.y + b.height]]
}

export function footprintBounds(fp?: FootprintGeo): Bounds {
  if (!fp) return { x: -0.8, y: -0.8, width: 1.6, height: 1.6 }
  const points: Point[] = []
  const shapes: (FpShape & { matrix?: Matrix })[] = [...fp.pads, ...fp.mount_holes, ...[fp.courtyard, fp.window].filter((s) => !!s)]
  for (const shape of shapes) {
    const round = shape.shape === 'circle' || shape.shape === 'annulus'
    const w = shape.size[0] ?? 0
    const h = round ? w : shape.size[1] ?? w
    const matrix = shape.matrix ?? identity
    for (const pt of corners({ x: -w / 2, y: -h / 2, width: w, height: h }))
      points.push(transformPoint(pt, matrix, [shape.x, shape.y]))
  }
  return points.length ? boundsOf(points) : footprintBounds()
}

export function placedBounds(p: Placement, fp?: FootprintGeo): Bounds {
  return boundsOf(corners(footprintBounds(fp)).map((point) => transformPoint(point, p.matrix, p.at)))
}

// DXF's clockwise flag is mathematical (+y-up). Numeric coordinates pass
// through unchanged, so its SVG sweep flag is reversed in the +y-down view.
export function outlinePath(outline: BoardOutline): string {
  let from = outline.start
  const chunks = [`M ${from.join(' ')}`]
  for (const seg of outline.segments) {
    if (seg.type === 'line') chunks.push(`L ${seg.to.join(' ')}`)
    else {
      const start = Math.atan2(from[1] - seg.center[1], from[0] - seg.center[0])
      const end = Math.atan2(seg.to[1] - seg.center[1], seg.to[0] - seg.center[0])
      const tau = Math.PI * 2
      const sweep = ((seg.clockwise ? start - end : end - start) + tau) % tau || tau
      const r = Math.hypot(from[0] - seg.center[0], from[1] - seg.center[1])
      chunks.push(`A ${r} ${r} 0 ${sweep > Math.PI ? 1 : 0} ${seg.clockwise ? 0 : 1} ${seg.to.join(' ')}`)
    }
    from = seg.to
  }
  return chunks.join(' ') + ' Z'
}

export function outlineBounds(outline?: BoardOutline | null): Point[] {
  if (!outline) return []
  const points = [outline.start]
  let from = outline.start
  for (const seg of outline.segments) {
    points.push(seg.to)
    if (seg.type === 'arc') {
      const r = Math.hypot(from[0] - seg.center[0], from[1] - seg.center[1])
      // Conservative for major arcs too; endpoints alone can clip the edge.
      points.push([seg.center[0] - r, seg.center[1] - r], [seg.center[0] + r, seg.center[1] + r])
    }
    from = seg.to
  }
  return points
}

export function layoutScope(m: ExplorerModel, scope: string | null, frame: 'board' | 'local' = 'board') {
  const subdesign = m.subdesigns?.find((s) => s.path === scope)
  const local = frame === 'local' && subdesign?.local_placements !== undefined
  const instances = m.instances.filter((i) => !scope || i.path.startsWith(`${scope}::`))
  const ids = new Set(instances.map((i) => i.path))
  const placements = (local ? subdesign.local_placements! : m.layout?.placements ?? []).filter((p) => ids.has(p.instance))
  const placed = new Set(placements.map((p) => p.instance))
  return {
    instances, placements, unplaced: instances.filter((i) => !placed.has(i.path)),
    frame: local ? 'local' as const : 'board' as const,
    outline: local ? undefined : m.layout?.board_outline,
  }
}

export function gridStep(width: number): number {
  const magnitude = 10 ** Math.floor(Math.log10(Math.max(width / 15, 0.000001)))
  return [1, 2, 5, 10].map((v) => v * magnitude).find((v) => v >= width / 15) ?? magnitude * 10
}
