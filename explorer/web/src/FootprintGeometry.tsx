import { memo, useMemo } from 'react'
import type { FootprintGeo, FpPad, FpShape, Matrix } from './model'
import { footprintBounds, svgMatrix } from './boardGeometry'

/** Shape coordinates are local mm; SVG transforms preserve their handedness. */
export function Land({ shape, size, vertices, corner_radius, chamfer }: Pick<FpPad, 'shape' | 'size' | 'vertices' | 'corner_radius' | 'chamfer'>) {
  if (shape === 'polygon') return <polygon points={(vertices ?? []).map((p) => p.join(',')).join(' ')} />
  const w = size[0] ?? 0, h = size[1] ?? w
  if (shape === 'circle') return <circle r={w / 2} />
  if (shape === 'annulus') {
    const r = w / 2, inner = h / 2
    return <path fillRule="evenodd" d={`M ${-r} 0 A ${r} ${r} 0 1 0 ${r} 0 A ${r} ${r} 0 1 0 ${-r} 0 Z M ${-inner} 0 A ${inner} ${inner} 0 1 0 ${inner} 0 A ${inner} ${inner} 0 1 0 ${-inner} 0 Z`} />
  }
  if (chamfer) {
    const x = w / 2, y = h / 2, cut = chamfer.cut
    const points = [
      ...(chamfer.corner === 'top_left' ? [[-x, -y + cut], [-x + cut, -y]] : [[-x, -y]]),
      ...(chamfer.corner === 'top_right' ? [[x - cut, -y], [x, -y + cut]] : [[x, -y]]),
      ...(chamfer.corner === 'bottom_right' ? [[x, y - cut], [x - cut, y]] : [[x, y]]),
      ...(chamfer.corner === 'bottom_left' ? [[-x + cut, y], [-x, y - cut]] : [[-x, y]]),
    ]
    return <polygon points={points.map((p) => p.join(',')).join(' ')} />
  }
  return <rect x={-w / 2} y={-h / 2} width={w} height={h} rx={shape === 'oval' ? Math.min(w, h) / 2 : corner_radius ?? 0} />
}

function shapeMatrix(shape: FpPad): Matrix {
  if (shape.matrix) return shape.matrix
  const radians = (shape.rotate ?? 0) * Math.PI / 180
  return [Math.cos(radians), -Math.sin(radians), Math.sin(radians), Math.cos(radians)]
}

function perimeter(shape: FpShape, color: string) {
  return <g transform={`translate(${shape.x} ${shape.y})`} fill="none" stroke={color} strokeWidth={0.08} strokeDasharray="0.3 0.2"><Land shape={shape.shape as FpPad['shape']} size={shape.size} /></g>
}

function validShape(shape: FpShape & { vertices?: [number, number][]; matrix?: Matrix }) {
  if (![shape.x, shape.y, ...(shape.matrix ?? [])].every(Number.isFinite)) return false
  if (shape.shape === 'polygon') return (shape.vertices?.length ?? 0) >= 3 && shape.vertices!.every(p => p.every(Number.isFinite))
  return shape.size.length > 0 && shape.size.every(value => Number.isFinite(value) && value > 0)
}

export const FootprintPreview = memo(function FootprintPreview({ geo, dark, width = 300, height = 200 }: { geo?: FootprintGeo; dark: boolean; width?: number; height?: number }) {
  const prepared = useMemo(() => {
    if (!geo) return null
    const shapes = [...geo.pads, ...geo.mount_holes, ...[geo.courtyard, geo.window].filter((shape): shape is FpShape => !!shape)]
    if (!shapes.length || !shapes.every(validShape)) return null
    const normalized = { ...geo, pads: geo.pads.map(p => ({ ...p, matrix: shapeMatrix(p) })) }
    const bounds = footprintBounds(normalized)
    if (![bounds.x, bounds.y, bounds.width, bounds.height].every(Number.isFinite)) return null
    const margin = Math.max(bounds.width, bounds.height) * 0.08 + 0.1
    return { geo: normalized, viewBox: `${bounds.x - margin} ${bounds.y - margin} ${bounds.width + margin * 2} ${bounds.height + margin * 2}` }
  }, [geo])
  const background = dark ? '#181c22' : '#fff'
  if (!prepared) return <div className="footprint-unavailable" role="img" aria-label="Local footprint unavailable" style={{ width, height, display: 'grid', placeItems: 'center', fontSize: 11, color: dark ? '#aeb6c2' : '#536170', textAlign: 'center' }}>Footprint unavailable</div>
  return <svg className="local-footprint" role="img" aria-label="Actual local footprint; not board placement" width={width} height={height} viewBox={prepared.viewBox} style={{ display: 'block', flexShrink: 0 }}>
    <title>Actual local footprint; not board placement</title>
    {prepared.geo.courtyard && perimeter(prepared.geo.courtyard, dark ? '#2dd4bf' : '#0d9488')}
    {prepared.geo.window && perimeter(prepared.geo.window, '#94a3b8')}
    {prepared.geo.pads.map((p, index) => <g key={index} transform={svgMatrix(p.matrix!, [p.x, p.y])} fill="#c9973f">
      <title>{`Pad ${p.number}`}</title><Land {...p} />
      {!!p.drill?.length && <g fill={background}><Land shape={p.drill.length === 1 ? 'circle' : 'oval'} size={p.drill} /></g>}
      {(p.size[0] ?? 0) > 0 && <text fill={dark ? '#111' : '#fff'} fontSize={(p.size[0] ?? 0) * 0.4} textAnchor="middle" dominantBaseline="central">{p.number}</text>}
    </g>)}
    {prepared.geo.mount_holes.map((h, index) => <g key={index} transform={`translate(${h.x} ${h.y})`} fill={background} stroke={h.plated ? '#c9973f' : '#94a3b8'} strokeWidth={0.08} data-plating={h.plated === null ? 'source_unspecified' : h.plated ? 'plated' : 'non_plated'}>
      <title>{h.plated === null ? 'Hole plating: source unspecified' : h.plated ? 'Plated hole' : 'Non-plated hole'}</title><Land shape={h.shape as FpPad['shape']} size={h.size} />
    </g>)}
  </svg>
})
