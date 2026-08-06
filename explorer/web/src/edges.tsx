// Orthogonal wire renderer: per-corridor lanes for forward wires, a shared
// lower corridor for backward ones so no segment crosses a node body.

import { BaseEdge, EdgeLabelRenderer, Position, type EdgeProps } from '@xyflow/react'

/** Orthogonal polyline with rounded corners, collinear points dropped. */
function roundedPath(pts: { x: number; y: number }[], radius = 6): string {
  const p = pts.filter(
    (q, i) => i === 0 || Math.abs(q.x - pts[i - 1].x) > 0.5 || Math.abs(q.y - pts[i - 1].y) > 0.5,
  )
  if (p.length < 2) return ''
  let d = `M ${p[0].x} ${p[0].y}`
  for (let i = 1; i < p.length - 1; i++) {
    const a = p[i - 1], b = p[i], c = p[i + 1]
    const r = Math.min(
      radius,
      Math.hypot(b.x - a.x, b.y - a.y) / 2,
      Math.hypot(c.x - b.x, c.y - b.y) / 2,
    )
    const ua = { x: Math.sign(b.x - a.x), y: Math.sign(b.y - a.y) }
    const uc = { x: Math.sign(c.x - b.x), y: Math.sign(c.y - b.y) }
    d += ` L ${b.x - ua.x * r} ${b.y - ua.y * r} Q ${b.x} ${b.y} ${b.x + uc.x * r} ${b.y + uc.y * r}`
  }
  const e = p[p.length - 1]
  return d + ` L ${e.x} ${e.y}`
}

// Orthogonal edge with lane separation: a forward wire turns at its own
// fraction of the horizontal gap; a backward/same-side wire detours BELOW
// both nodes instead of cutting through the IC body.
export function LaneEdge({
  id, sourceX, sourceY, targetX, targetY, sourcePosition, targetPosition,
  style, label, labelStyle, data,
}: EdgeProps) {
  const t = (data?.lane as number) ?? 0.5
  const STUB = 16
  const dirS = sourcePosition === Position.Left ? -1 : 1
  const dirT = targetPosition === Position.Left ? -1 : 1
  const ex = sourceX + dirS * STUB
  const nx = targetX + dirT * STUB
  // Forward = the two stubs face each other with room in between.
  const forward = dirS > 0 && dirT < 0 ? nx > ex : dirS < 0 && dirT > 0 ? nx < ex : false
  let path: string
  let lx: number
  let ly: number
  if (Math.abs(targetY - sourceY) < 2 && forward) {
    path = `M ${sourceX} ${sourceY} L ${targetX} ${targetY}`
    lx = (sourceX + targetX) / 2
    ly = sourceY
  } else if (forward) {
    const midX = ex + (nx - ex) * t
    path = roundedPath([
      { x: sourceX, y: sourceY },
      { x: midX, y: sourceY },
      { x: midX, y: targetY },
      { x: targetX, y: targetY },
    ])
    lx = midX
    ly = (sourceY + targetY) / 2
  } else {
    // Route under both nodes: out through each stub, along a shared lower
    // corridor, then up into the target — never across a node body.
    const base = Math.max((data?.sBot as number) ?? sourceY, (data?.tBot as number) ?? targetY)
    const by = base + 26 + ((data?.dlane as number) ?? t) * 220
    path = roundedPath([
      { x: sourceX, y: sourceY },
      { x: ex, y: sourceY },
      { x: ex, y: by },
      { x: nx, y: by },
      { x: nx, y: targetY },
      { x: targetX, y: targetY },
    ])
    lx = (ex + nx) / 2
    ly = by
  }
  const bg = (data?.labelBg as string) ?? '#ffffffd9'
  return (
    <>
      <BaseEdge id={id} path={path} style={style} />
      {label ? (
        <EdgeLabelRenderer>
          <div
            style={{
              position: 'absolute',
              transform: `translate(-50%, -50%) translate(${lx}px, ${ly}px)`,
              background: bg,
              padding: '0 3px',
              borderRadius: 3,
              pointerEvents: 'none',
              color: (labelStyle as { fill?: string } | undefined)?.fill,
              fontSize: 9,
            }}
          >
            {label}
          </div>
        </EdgeLabelRenderer>
      ) : null}
    </>
  )
}
