import { useEffect, useMemo, useRef, useState } from 'react'
import type { ExplorerModel, FpPad, FpShape } from './model'
import { shortName } from './model'
import { boundsOf, corners, footprintBounds, gridStep, identity, layoutScope, outlineBounds, outlinePath, placedBounds, svgMatrix, type Bounds, type Point } from './boardGeometry'
import './BoardLayout.css'

const topColor = '#f59e0b'
const bottomColor = '#60a5fa'
const mm = (n: number) => Number(n.toFixed(4)).toString()

/** Shape coordinates are local mm; SVG transforms preserve their handedness. */
function Land({ shape, size, corner_radius, chamfer }: Pick<FpPad, 'shape' | 'size' | 'corner_radius' | 'chamfer'>) {
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

function Perimeter({ shape, color }: { shape: FpShape; color: string }) {
  return <g transform={`translate(${shape.x} ${shape.y})`} fill="none" stroke={color} strokeWidth={0.08}>
    {shape.shape === 'circle' ? <circle r={shape.size[0] / 2} />
      : <rect x={-shape.size[0] / 2} y={-shape.size[1] / 2} width={shape.size[0]} height={shape.size[1]} />}
  </g>
}

export function BoardLayout({ model, scope, frame, onFrame, onScope, selected, selectedNet, focus, onSelect, onNet }: {
  model: ExplorerModel
  scope: string | null
  frame: 'board' | 'local'
  onFrame: (frame: 'board' | 'local') => void
  onScope: (path: string) => void
  selected: string[]
  selectedNet: string
  focus: { id?: string; net?: string } | null
  onSelect: (paths: string[]) => void
  onNet: (name: string) => void
}) {
  const [side, setSide] = useState<'both' | 'top' | 'bottom'>('both')
  const [labels, setLabels] = useState(true)
  const [padNumbers, setPadNumbers] = useState(false)
  const scoped = useMemo(() => layoutScope(model, scope, frame), [model, scope, frame])
  const local = scoped.frame === 'local'
  const subdesign = model.subdesigns?.find((s) => s.path === scope)
  const localLayouts = model.subdesigns?.filter((s) => s.local_placements?.length && (!scope || s.path === scope || s.path.startsWith(`${scope}::`))) ?? []
  const byPath = useMemo(() => new Map(scoped.instances.map((i) => [i.path, i])), [scoped])
  const placements = useMemo(() => scoped.placements.filter((p) => side === 'both' || p.side === side), [scoped, side])
  const geometry = useMemo(() => new Map(placements.map((p) => {
    const i = byPath.get(p.instance)!
    const fp = i.part?.footprint ? model.footprints[i.part.footprint] : undefined
    return [p.instance, { fp, box: placedBounds(p, fp) }]
  })), [model, byPath, placements])
  const fitted = useMemo(() => {
    const pts = [...geometry.values()].flatMap(({ box }) => corners(box))
    // A board outline belongs only to the board coordinate frame.
    pts.push(...outlineBounds(scoped.outline))
    const b = boundsOf(pts)
    const pad = Math.max(2, Math.max(b.width, b.height) * 0.12)
    return { x: b.x - pad, y: b.y - pad, width: b.width + pad * 2, height: b.height + pad * 2 }
  }, [geometry, scoped.outline])
  const [view, setView] = useState<Bounds>(fitted)
  useEffect(() => { setView(fitted) }, [fitted])
  // Global search can reveal a component even if its side was filtered out.
  useEffect(() => {
    const p = scoped.placements.find((p) => p.instance === focus?.id)
    if (p) setSide((current) => current !== 'both' && current !== p.side ? 'both' : current)
  }, [focus, scoped])
  useEffect(() => {
    const p = placements.find((p) => p.instance === focus?.id)
    if (!p) return
    const inst = byPath.get(p.instance)
    const fp = inst?.part?.footprint ? model.footprints[inst.part.footprint] : undefined
    const box = placedBounds(p, fp)
    const span = Math.max(12, box.width * 2, box.height * 2)
    setView({ x: box.x + box.width / 2 - span / 2, y: box.y + box.height / 2 - span / 2, width: span, height: span })
  }, [fitted, focus, placements, byPath, model.footprints])
  const svg = useRef<SVGSVGElement>(null)
  const drag = useRef<{ x: number; y: number; view: Bounds; scale: number; moved: boolean } | null>(null)
  const screenPoint = (x: number, y: number): Point => {
    const matrix = svg.current?.getScreenCTM()
    if (!matrix) return [0, 0]
    const p = new DOMPoint(x, y).matrixTransform(matrix.inverse())
    return [p.x, p.y]
  }
  const zoom = (factor: number, center: Point = [view.x + view.width / 2, view.y + view.height / 2]) => {
    if (view.width * factor < 0.1 || view.width * factor > Math.max(fitted.width * 20, 1000)) return
    setView({ x: center[0] + (view.x - center[0]) * factor, y: center[1] + (view.y - center[1]) * factor, width: view.width * factor, height: view.height * factor })
  }
  // React delegates wheel listeners passively; this canvas owns wheel zoom.
  useEffect(() => {
    const el = svg.current
    if (!el) return
    const wheel = (event: WheelEvent) => {
      event.preventDefault()
      zoom(event.deltaY > 0 ? 1.15 : 1 / 1.15, screenPoint(event.clientX, event.clientY))
    }
    el.addEventListener('wheel', wheel, { passive: false })
    return () => el.removeEventListener('wheel', wheel)
  }, [view, fitted])
  const step = gridStep(view.width)
  const netPads = useMemo(() => {
    const pads = new Map<string, Set<string>>()
    for (const member of model.nets.find((n) => n.name === selectedNet)?.members ?? []) {
      const numbers = pads.get(member.instance_path) ?? new Set<string>()
      member.numbers.forEach((n) => numbers.add(n))
      pads.set(member.instance_path, numbers)
    }
    return pads
  }, [model, selectedNet])
  const selectedPlacements = selected.map((id) => scoped.placements.find((p) => p.instance === id)).filter((p) => !!p)
  const constraintButton = (name: string) => <button key={name} onClick={() => onNet(name)} style={{ border: 0, background: 'none', color: '#67e8f9', cursor: 'pointer', padding: '2px 3px' }}>{name}</button>
  const panel = { background: '#181c22ed', color: '#d1d5db', border: '1px solid #374151', borderRadius: 6, padding: '6px 10px', fontSize: 11 }

  return <div className="board-layout" style={{ height: '100%', background: '#111820', color: '#e5e7eb' }}>
    <div style={{ position: 'absolute', top: 132, left: 12, right: 12, zIndex: 4, display: 'flex', gap: 8, flexWrap: 'wrap', alignItems: 'center', ...panel }}>
      <b>{local ? 'Local layout' : 'Board layout'} · mm</b>
      {subdesign?.local_placements !== undefined && <label>Coordinates <select aria-label="Layout coordinates" value={scoped.frame} onChange={(e) => onFrame(e.target.value as 'board' | 'local')}>
        <option value="local">Subdesign local</option><option value="board">Board coordinates</option>
      </select></label>}
      <label>Components <select aria-label="Component side" value={side} onChange={(e) => setSide(e.target.value as typeof side)}>
        <option value="both">Both sides</option><option value="top">Top</option><option value="bottom">Bottom</option>
      </select></label>
      <label><input type="checkbox" checked={labels} onChange={(e) => setLabels(e.target.checked)} /> Labels</label>
      <label><input type="checkbox" checked={padNumbers} onChange={(e) => setPadNumbers(e.target.checked)} /> Pad numbers</label>
      <span><span style={{ color: topColor }}>● top</span> · <span style={{ color: bottomColor }}>● bottom</span></span>
      <span>{placements.length} {local ? 'locally placed' : 'placed'} · {scoped.unplaced.length} without placement</span>
    </div>
    <svg ref={svg} role="group" aria-label="Physical component layout in millimetres" viewBox={`${view.x} ${view.y} ${view.width} ${view.height}`}
      style={{ position: 'absolute', left: 0, top: 178, width: '100%', height: 'calc(100% - 224px)', touchAction: 'none', cursor: 'grab' }}
      onPointerDown={(e) => {
        if ((e.target as Element).closest('[data-instance]')) return
        drag.current = { x: e.clientX, y: e.clientY, view, scale: svg.current?.getScreenCTM()?.a ?? 1, moved: false }
        e.currentTarget.setPointerCapture(e.pointerId)
      }}
      onPointerMove={(e) => {
        const d = drag.current
        if (!d) return
        const dx = e.clientX - d.x, dy = e.clientY - d.y
        if (Math.abs(dx) + Math.abs(dy) > 3) d.moved = true
        setView({ ...d.view, x: d.view.x - dx / d.scale, y: d.view.y - dy / d.scale })
      }}
      onPointerUp={() => { if (drag.current && !drag.current.moved) onSelect([]); drag.current = null }}
      onPointerCancel={() => { drag.current = null }}>
      <defs><pattern id="board-mm-grid" width={step} height={step} patternUnits="userSpaceOnUse"><path d={`M ${step} 0 L 0 0 0 ${step}`} stroke="#334155" strokeWidth={0.5} vectorEffect="non-scaling-stroke" fill="none" /></pattern></defs>
      <rect x={view.x - view.width} y={view.y - view.height} width={view.width * 3} height={view.height * 3} fill="url(#board-mm-grid)" />
      <path d={`M ${view.x} 0 H ${view.x + view.width} M 0 ${view.y} V ${view.y + view.height}`} stroke="#64748b" strokeWidth={1} vectorEffect="non-scaling-stroke" strokeDasharray="3 4" />
      <text x={0.4} y={-0.4} fontSize={Math.max(0.5, view.width / 140)} fill="#94a3b8">0, 0</text>
      {scoped.outline && <path data-board-outline d={outlinePath(scoped.outline)} stroke="#34d399" fill="#064e3b22" strokeWidth={1.5} vectorEffect="non-scaling-stroke" />}
      {placements.map((p) => {
        const inst = byPath.get(p.instance)!, { fp, box } = geometry.get(p.instance)!
        const selectedPart = selected.includes(p.instance)
        const lit = netPads.has(p.instance)
        const color = selectedPart || lit ? '#22d3ee' : p.side === 'bottom' ? bottomColor : topColor
        const local = footprintBounds(fp)
        return <g key={p.instance} className="board-layout__component" data-instance={p.instance} role="button" tabIndex={0}
          aria-label={`${inst.designator ?? shortName(p.instance)} at ${p.at_mm.join(', ')} mm, rotate ${p.rotate}, ${p.side}`}
          onClick={(e) => { e.stopPropagation(); onSelect(e.metaKey || e.ctrlKey ? selectedPart ? selected.filter((x) => x !== p.instance) : [...selected, p.instance] : [p.instance]) }}
          onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect([p.instance]) } }}
          style={{ cursor: 'pointer', opacity: selectedNet && !lit ? 0.25 : 1 }}>
          <title>{p.instance}{'\n'}{p.at_mm.join(', ')} mm · {p.rotate}° · {p.side}{!fp ? '\nFootprint geometry unavailable; origin shown' : ''}</title>
          <g transform={svgMatrix(p.matrix, p.at)}>
            <rect x={local.x} y={local.y} width={local.width} height={local.height} fill={selectedPart ? '#22d3ee22' : '#ffffff04'} stroke={selectedPart ? '#22d3ee' : 'none'} strokeWidth={2} vectorEffect="non-scaling-stroke" />
            {fp?.courtyard && <Perimeter shape={fp.courtyard} color={color} />}
            {fp?.window && <Perimeter shape={fp.window} color="#34d399" />}
            {fp?.pads.map((pad, index) => {
              const bottomPad = (pad.layer === 'bottom_copper') !== (p.side === 'bottom')
              const fill = netPads.get(p.instance)?.has(pad.number) ? '#22d3ee' : pad.pth ? '#d6b96c' : bottomPad ? bottomColor : topColor
              return <g key={index} transform={svgMatrix(pad.matrix ?? identity, [pad.x, pad.y])} fill={fill}>
                <Land {...pad} />
                {!!pad.drill?.length && <g fill="#111820"><Land shape={pad.drill.length === 1 ? 'circle' : 'oval'} size={pad.drill} /></g>}
                {padNumbers && <text fill="#111827" textAnchor="middle" dominantBaseline="central" fontSize={Math.min(0.7, pad.size[0] * 0.65)}>{pad.number}</text>}
              </g>
            })}
            {fp?.mount_holes.map((hole, index) => <g key={index} transform={`translate(${hole.x} ${hole.y})`} fill="#111820" stroke={hole.plated ? '#d6b96c' : '#94a3b8'} strokeWidth={0.08}><Land shape={hole.shape as FpPad['shape']} size={hole.size} /></g>)}
            <path d="M -0.3 0 H 0.3 M 0 -0.3 V 0.3" stroke={color} strokeWidth={0.08} />
            {!fp && <circle r={0.7} fill="none" stroke={color} strokeWidth={0.08} strokeDasharray="0.2 0.15" />}
            <rect className="board-layout__focus-ring" aria-hidden="true"
              x={local.x - 0.08} y={local.y - 0.08} width={local.width + 0.16} height={local.height + 0.16}
              fill="none" stroke="#67e8f9" strokeWidth={2} strokeDasharray="4 3" vectorEffect="non-scaling-stroke" />
          </g>
          {(labels || selectedPart) && <text x={p.at[0]} y={box.y - 0.5} textAnchor="middle" fill={color} fontSize={Math.max(0.55, Math.min(1.2, view.width / 110))} paintOrder="stroke" stroke="#111820" strokeWidth={0.15}>{inst.designator ?? shortName(p.instance)}</text>}
        </g>
      })}
    </svg>
    <div style={{ position: 'absolute', top: 180, right: 12, zIndex: 3, maxWidth: 300, maxHeight: '45%', overflow: 'auto', ...panel }}>
      {local && <div style={{ marginBottom: 6 }}>Local defaults for <b>{shortName(scope ?? '')}</b>. Board coordinates include the parent placement and any outer overrides.</div>}
      {!local && model.layout?.outline_error && <div style={{ color: '#fbbf24', marginBottom: 6 }}>Outline unavailable: {model.layout.outline_error}</div>}
      {!local && !model.layout && <div>{model.layout === undefined ? 'This snapshot has no layout data. Re-extract the project to view placements.' : 'No board layout facts declared.'}</div>}
      {!local && model.layout && !model.layout.outline_source && <div style={{ color: '#9ca3af' }}>No board outline declared</div>}
      {!local && !scoped.placements.length && localLayouts.length > 0 && <div style={{ margin: '6px 0' }}>
        <div>Local layouts are available:</div>
        {localLayouts.map((s) => <button key={s.path} onClick={() => onScope(s.path)} style={{ margin: 2 }}>{shortName(s.path)} · {s.local_placements!.length} parts</button>)}
      </div>}
      <details><summary style={{ cursor: 'pointer' }}>Without {local ? 'local' : 'board'} placement ({scoped.unplaced.length})</summary>
        <p>{local ? 'These parts have no placement relative to this subdesign. A nested subdesign may have its own local layout.' : 'These parts have no resolved board coordinate. Open a subdesign to inspect its local defaults.'}</p>
        {scoped.unplaced.map((i) => <div key={i.path}><button onClick={() => onSelect([i.path])} title={i.path} style={{ border: 0, background: 'none', color: '#93c5fd', cursor: 'pointer', padding: 3 }}>{i.designator} {shortName(i.path)}</button>{i.placement_hint && <small> · {i.placement_hint}</small>}</div>)}
      </details>
      <details style={{ marginTop: 5 }}><summary style={{ cursor: 'pointer' }}>Layout constraints</summary>
        {!model.layout?.net_classes.length && !model.layout?.diff_pairs.length && !model.layout?.length_matches.length && <p>No net constraints declared</p>}
        {model.layout?.net_classes.map((c) => <div key={c.name}><b>{c.name}</b>: {c.nets.map(constraintButton)}</div>)}
        {model.layout?.diff_pairs.map((d) => <div key={d.p + d.n}>Differential pair: {constraintButton(d.p)} / {constraintButton(d.n)}</div>)}
        {model.layout?.length_matches.map((l, index) => <div key={index}>Length match: {l.nets.map(constraintButton)} {l.tolerance && `± ${l.tolerance}`}</div>)}
      </details>
    </div>
    <div style={{ position: 'absolute', bottom: 10, left: 12, right: 12, display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap', ...panel }}>
      <button onClick={() => setView(fitted)}>{local ? 'Fit local layout' : 'Fit board'}</button><button aria-label="Zoom into layout" onClick={() => zoom(0.8)}>+</button><button aria-label="Zoom out of layout" onClick={() => zoom(1.25)}>−</button>
      <span>Grid {mm(step)} mm · X → Y ↓ · viewed from top</span>
      <span style={{ color: '#9ca3af' }}>Pan to inspect · ⌘/Ctrl-click two parts to measure origins</span>
      {selectedPlacements.length === 2 && (() => {
        const dx = selectedPlacements[1].at[0] - selectedPlacements[0].at[0]
        const dy = selectedPlacements[1].at[1] - selectedPlacements[0].at[1]
        return <b style={{ color: '#67e8f9' }}>Distance {mm(Math.hypot(dx, dy))} mm · ΔX {mm(dx)} · ΔY {mm(dy)}</b>
      })()}
    </div>
  </div>
}
