// Custom React Flow node renderers: compact overview parts, mini passives,
// SCH pin-level ICs, region frames, and the to-scale footprint preview.

import { Handle, Position, type NodeProps } from '@xyflow/react'
import type { FootprintGeo } from './model'
import { shortName } from './model'
import type { GNode } from './transform'
import { netWireColor, railColor, selectNet } from './palette'

export function PartNode({ data, selected }: NodeProps) {
  const n = data.g as GNode
  const dark = data.dark as boolean
  if (n.kind === 'net') {
    const c = netWireColor(n.title, dark)
    return (
      <div
        onClick={() => selectNet(n.title)}
        style={{
          width: n.width, height: n.height, borderRadius: 11,
          border: `1.5px solid ${c}`, color: c, cursor: 'pointer',
          background: dark ? '#181c22' : '#fff',
          opacity: data.dim ? 0.25 : 1,
          fontSize: 9.5, fontWeight: 700, display: 'flex',
          alignItems: 'center', justifyContent: 'center', boxSizing: 'border-box',
        }}
      >
        <Handle type="target" position={Position.Left} style={{ opacity: 0 }} />
        <Handle type="source" position={Position.Right} style={{ opacity: 0 }} />
        {n.title}
      </div>
    )
  }
  const border =
    n.kind === 'agg' ? '#059669' : n.kind === 'passive' ? (dark ? '#4b5563' : '#9ca3af') : '#2563eb'
  const bg = dark ? (data.dim ? '#1a1d24' : '#22262e') : data.dim ? '#f3f4f6' : '#ffffff'
  const fg = dark ? '#e5e7eb' : '#111827'
  const hl = data.hl as boolean
  if (n.kind === 'passive') {
    // Uniform mini style for every R/C/L: one compact chip riding its wire.
    return (
      <div
        style={{
          width: n.width, height: n.height, boxSizing: 'border-box',
          border: `${hl ? 2 : 1.5}px solid ${selected ? '#f59e0b' : hl ? '#22d3ee' : border}`,
          borderRadius: 5, background: hl ? (dark ? '#0e3a44' : '#ecfeff') : bg,
          color: hl ? (dark ? '#a5f3fc' : '#0e7490') : fg,
          boxShadow: hl ? '0 0 0 3px #22d3ee55, 0 0 14px #22d3ee66' : undefined,
          opacity: data.dim ? 0.18 : 1,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          gap: 4, fontSize: 9, fontWeight: 600, padding: '0 4px',
        }}
      >
        <Handle type="target" position={Position.Left} style={{ opacity: 0 }} />
        <Handle type="source" position={Position.Right} style={{ opacity: 0 }} />
        <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
          {n.title}
          {n.inst && n.pinsConnected === 0 && <span style={{ color: '#ef4444' }}> ✕</span>}
        </span>
        {n.railTags.map((r) => (
          <span
            key={r}
            onClick={(e) => {
              e.stopPropagation()
              selectNet(r)
            }}
            style={{
              background: railColor(r), color: '#fff', borderRadius: 3,
              padding: '0 3px', fontSize: 7.5, cursor: 'pointer', flexShrink: 0,
            }}
          >
            {r}
          </span>
        ))}
      </div>
    )
  }
  return (
    <div
      style={{
        width: n.width,
        minHeight: n.height,
        border: `${hl ? 2.5 : 2}px solid ${selected ? '#f59e0b' : hl ? '#22d3ee' : border}`,
        borderRadius: 7,
        background: hl ? (dark ? '#0e3a44' : '#ecfeff') : bg,
        boxShadow: hl ? '0 0 0 3px #22d3ee55, 0 0 16px #22d3ee66' : undefined,
        opacity: data.dim ? 0.18 : 1,
        fontSize: 10,
        padding: '5px 7px',
        boxSizing: 'border-box',
        color: fg,
      }}
    >
      <Handle type="target" position={Position.Left} style={{ opacity: 0 }} />
      <Handle type="source" position={Position.Right} style={{ opacity: 0 }} />
      <div style={{ fontWeight: 700, fontSize: 11 }}>
        {n.title}
        {n.inst && n.pinsConnected === 0 && (
          <span style={{ color: '#ef4444', marginLeft: 5 }}>✕ unwired</span>
        )}
      </div>
      {n.sub && <div style={{ color: dark ? '#9ca3af' : '#6b7280', fontSize: 9 }}>{n.sub}</div>}
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 3, marginTop: 2 }}>
        {n.railTags.map((r) => (
          <span
            key={r}
            onClick={(e) => {
              e.stopPropagation()
              selectNet(r)
            }}
            style={{
              background: railColor(r),
              color: '#fff',
              borderRadius: 3,
              padding: '0 4px',
              fontSize: 8.5,
              cursor: 'pointer',
            }}
          >
            {r}
          </span>
        ))}
        {n.netLabels.map((l) => (
          <span
            key={l}
            onClick={(e) => {
              e.stopPropagation()
              selectNet(l)
            }}
            style={{
              border: `1px solid ${netWireColor(l, dark)}`,
              color: netWireColor(l, dark),
              borderRadius: 3,
              padding: '0 3px',
              fontSize: 8.5,
              cursor: 'pointer',
            }}
          >
            {l}
          </span>
        ))}
      </div>
      {n.decors.length > 0 && (
        <div style={{ marginTop: 2, color: dark ? '#9ca3af' : '#374151', fontSize: 8.5 }}>
          {n.decors.slice(0, 5).map((d) => (
            <div key={d}>{d}</div>
          ))}
          {n.decors.length > 5 && <div>… +{n.decors.length - 5}</div>}
        </div>
      )}
      {n.kind === 'ic' && n.pinsTotal > n.pinsConnected && (
        <div style={{ color: dark ? '#4b5563' : '#9ca3af', fontSize: 8.5, marginTop: 1 }}>
          +{n.pinsTotal - n.pinsConnected} unused
        </div>
      )}
    </div>
  )
}

// ---------- footprint preview (sidebar) ----------
// Renders the RFC-018 pad geometry to scale: courtyard dashed, copper pads,
// PTH drills, mount holes. CoHDL authors +y-down — same frame as SVG.
export function FootprintPreview({ geo, dark }: { geo: FootprintGeo; dark: boolean }) {
  const rad = (d: number) => (d * Math.PI) / 180
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity
  const grow = (x: number, y: number, hw: number, hh: number) => {
    minX = Math.min(minX, x - hw); maxX = Math.max(maxX, x + hw)
    minY = Math.min(minY, y - hh); maxY = Math.max(maxY, y + hh)
  }
  for (const p of geo.pads) {
    const [a, b] = p.shape === 'circle' ? [p.size[0] ?? 0, p.size[0] ?? 0] : [p.size[0] ?? 0, p.size[1] ?? 0]
    const c = Math.abs(Math.cos(rad(p.rotate))), s = Math.abs(Math.sin(rad(p.rotate)))
    grow(p.x, p.y, (a * c + b * s) / 2, (a * s + b * c) / 2)
  }
  for (const h of geo.mount_holes) {
    const [a, b] = h.size.length === 1 ? [h.size[0], h.size[0]] : [h.size[0], h.size[1] ?? h.size[0]]
    grow(h.x, h.y, a / 2, b / 2)
  }
  for (const sh of [geo.courtyard, geo.window]) {
    if (!sh) continue
    const [a, b] = sh.size.length === 1 ? [sh.size[0], sh.size[0]] : [sh.size[0], sh.size[1] ?? sh.size[0]]
    grow(sh.x, sh.y, a / 2, b / 2)
  }
  if (!isFinite(minX)) return null
  const pad = 0.4
  minX -= pad; minY -= pad; maxX += pad; maxY += pad
  const w = maxX - minX, h = maxY - minY
  const scale = Math.min(300 / w, 200 / h)
  const W = w * scale, H = h * scale
  const X = (x: number) => (x - minX) * scale
  const Y = (y: number) => (y - minY) * scale
  const copper = '#c9973f'
  const court = dark ? '#2dd4bf' : '#0d9488'
  const fontPx = (p: { size: number[] }) => Math.min(10, Math.max(0, (p.size[0] ?? 0) * scale * 0.42))
  const shapeEl = (sh: NonNullable<typeof geo.courtyard>, color: string, dash: string) =>
    sh.shape === 'circle' ? (
      <circle cx={X(sh.x)} cy={Y(sh.y)} r={((sh.size[0] ?? 0) / 2) * scale} fill="none" stroke={color} strokeDasharray={dash} strokeWidth={1} />
    ) : (
      <rect x={X(sh.x - (sh.size[0] ?? 0) / 2)} y={Y(sh.y - (sh.size[1] ?? 0) / 2)} width={(sh.size[0] ?? 0) * scale} height={(sh.size[1] ?? 0) * scale} fill="none" stroke={color} strokeDasharray={dash} strokeWidth={1} />
    )
  // 1mm scale bar, bottom-left
  const barMm = w > 12 ? 5 : 1
  return (
    <svg width={W} height={H + 16} style={{ display: 'block', margin: '6px 0 2px' }}>
      {geo.courtyard && shapeEl(geo.courtyard, court, '4 3')}
      {geo.window && shapeEl(geo.window, dark ? '#64748b' : '#94a3b8', '2 2')}
      {geo.pads.map((p, i) => {
        const num = fontPx(p) >= 5 && (
          <text x={X(p.x)} y={Y(p.y)} fill={dark ? '#111' : '#fff'} fontSize={fontPx(p)} textAnchor="middle" dominantBaseline="central" fontWeight={700}>
            {p.number}
          </text>
        )
        if (p.shape === 'circle') {
          return (
            <g key={i}>
              <circle cx={X(p.x)} cy={Y(p.y)} r={((p.size[0] ?? 0) / 2) * scale} fill={copper} />
              {p.drill && p.drill.length > 0 && (
                <circle cx={X(p.x)} cy={Y(p.y)} r={(p.drill[0] / 2) * scale} fill={dark ? '#181c22' : '#fff'} />
              )}
              {num}
            </g>
          )
        }
        const pw = (p.size[0] ?? 0) * scale, ph = (p.size[1] ?? 0) * scale
        return (
          <g key={i} transform={p.rotate ? `rotate(${p.rotate} ${X(p.x)} ${Y(p.y)})` : undefined}>
            <rect x={X(p.x) - pw / 2} y={Y(p.y) - ph / 2} width={pw} height={ph} rx={p.shape === 'oval' ? Math.min(pw, ph) / 2 : 0.5} fill={copper} />
            {p.drill && p.drill.length > 0 && (
              <circle cx={X(p.x)} cy={Y(p.y)} r={(p.drill[0] / 2) * scale} fill={dark ? '#181c22' : '#fff'} />
            )}
            {num}
          </g>
        )
      })}
      {geo.mount_holes.map((mh, i) => (
        <g key={'mh' + i}>
          <circle cx={X(mh.x)} cy={Y(mh.y)} r={((mh.size[0] ?? 0) / 2) * scale} fill="none" stroke={dark ? '#6b7280' : '#9ca3af'} strokeWidth={1.5} />
          <circle cx={X(mh.x)} cy={Y(mh.y)} r={((mh.size[0] ?? 0) / 2) * scale * 0.7} fill={dark ? '#181c22' : '#fff'} stroke="none" />
        </g>
      ))}
      <line x1={2} y1={H + 8} x2={2 + barMm * scale} y2={H + 8} stroke={dark ? '#9ca3af' : '#6b7280'} strokeWidth={2} />
      <text x={2 + barMm * scale + 4} y={H + 11} fill={dark ? '#9ca3af' : '#6b7280'} fontSize={9}>
        {barMm}mm
      </text>
    </svg>
  )
}

const pinNetColor = (net: string) =>
  net.startsWith('GND') ? '#9ca3af' : /^V|VBUS|VSYS/.test(net) ? '#ef4444' : '#34d399'

export function detailedPins(n: GNode): { name: string; num: string; net: string }[] {
  if (!n.inst) return []
  const pins = n.inst.pins
    .filter((p) => p.connected)
    .map((p) => ({ name: p.logical, num: p.numbers.join(','), net: n.pinNets[p.logical] ?? '', role: p.role }))
  const rank = (r: string) => (r.startsWith('power') ? 0 : 1)
  pins.sort((a, b) => rank(a.role) - rank(b.role) || a.name.localeCompare(b.name))
  return pins
}

const ROW = 18
export function detailedSize(n: GNode): { width: number; height: number } {
  if (n.kind !== 'ic' || !n.inst) return { width: n.width, height: n.height }
  const rows = Math.ceil(detailedPins(n).length / 2)
  return { width: 300, height: 40 + rows * ROW + 10 }
}

export function DetailedNode(props: NodeProps) {
  const { data, selected } = props
  const n = data.g as GNode
  if (n.kind !== 'ic' || !n.inst) return <PartNode {...props} />
  const pins = detailedPins(n)
  const rows = Math.ceil(pins.length / 2)
  // Layout-aware pin placement (side + row order face the counterpart node)
  // when the app computed it; alphabetical half-split otherwise.
  const order = data.pinOrder as { l: string[]; r: string[] } | undefined
  const byName = new Map(pins.map((p) => [p.name, p]))
  const left = order
    ? order.l.map((nm) => byName.get(nm)).filter((p): p is NonNullable<typeof p> => !!p)
    : pins.slice(0, rows)
  const right = order
    ? order.r.map((nm) => byName.get(nm)).filter((p): p is NonNullable<typeof p> => !!p)
    : pins.slice(rows)
  const { width, height } = detailedSize(n)
  const hlPins = new Set((data.hlPins as string[] | undefined) ?? [])
  const nodeHl = data.hl as boolean
  const pinRow = (p: { name: string; num: string; net: string }, i: number, side: 'l' | 'r') => {
    const y = 40 + i * ROW + ROW / 2
    const on = hlPins.has(p.name)
    const c = on ? '#22d3ee' : pinNetColor(p.net)
    return (
      <div key={side + p.name} style={{ position: 'absolute', top: y - 8, [side === 'l' ? 'left' : 'right']: 6, width: 138, display: 'flex', flexDirection: side === 'l' ? 'row' : 'row-reverse', gap: 4, fontSize: 9, alignItems: 'center', background: on ? '#22d3ee22' : undefined, borderRadius: 3 }}>
        <span style={{ color: on ? '#67e8f9' : '#6b7280', minWidth: 14, textAlign: side === 'l' ? 'left' : 'right' }}>{p.num}</span>
        <span style={{ color: on ? '#a5f3fc' : '#e5e7eb', fontWeight: on ? 800 : 600, fontSize: 10 }}>{p.name}</span>
        <span
          onClick={(e) => {
            e.stopPropagation()
            selectNet(p.net)
          }}
          style={{ color: c, fontWeight: on ? 800 : 400, cursor: 'pointer', marginLeft: side === 'l' ? 'auto' : 0, marginRight: side === 'r' ? 'auto' : 0, maxWidth: 62, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}
        >
          {p.net}
        </span>
        <Handle id={'p:' + p.name} type="source" position={side === 'l' ? Position.Left : Position.Right} style={{ top: 8, opacity: 0, [side === 'l' ? 'left' : 'right']: -8 }} />
        <Handle id={'p:' + p.name} type="target" position={side === 'l' ? Position.Left : Position.Right} style={{ top: 8, opacity: 0, [side === 'l' ? 'left' : 'right']: -8 }} />
        <span
          style={{
            position: 'absolute', [side === 'l' ? 'left' : 'right']: on ? -7 : -5,
            top: on ? 3 : 5, width: on ? 10 : 6, height: on ? 10 : 6,
            borderRadius: 5, background: c,
            boxShadow: on ? '0 0 8px #22d3ee' : undefined,
          }}
        />
      </div>
    )
  }
  return (
    <div style={{ width, height, background: data.dim ? '#1a1d24' : nodeHl ? '#12303a' : '#22262e', opacity: data.dim ? 0.18 : 1, border: `${nodeHl ? 2.5 : 1.5}px solid ${selected ? '#f59e0b' : nodeHl ? '#22d3ee' : '#3b4252'}`, boxShadow: nodeHl ? '0 0 0 3px #22d3ee44, 0 0 18px #22d3ee66' : undefined, borderRadius: 6, position: 'relative', boxSizing: 'border-box' }}>
      <div style={{ background: '#161a20', borderRadius: '6px 6px 0 0', padding: '6px 10px', display: 'flex', gap: 8, alignItems: 'baseline' }}>
        <span style={{ color: '#60a5fa', fontWeight: 700, fontSize: 12 }}>{n.inst.designator}</span>
        <span style={{ color: '#f3f4f6', fontWeight: 700, fontSize: 12 }}>{shortName(n.inst.device_fq)}</span>
        <span style={{ color: '#6b7280', fontSize: 10 }}>{n.inst.part?.mpn}</span>
      </div>
      {left.map((p, i) => pinRow(p, i, 'l'))}
      {right.map((p, i) => pinRow(p, i, 'r'))}
      {n.pinsTotal > n.pinsConnected && (
        <div style={{ position: 'absolute', bottom: 2, left: 10, color: '#4b5563', fontSize: 8 }}>+{n.pinsTotal - n.pinsConnected} unused</div>
      )}
    </div>
  )
}

export function RegionNode({ data }: NodeProps) {
  const dark = data.dark as boolean
  return (
    <div
      style={{
        width: data.width as number,
        height: data.height as number,
        border: `1.5px dashed ${dark ? '#3b4252' : '#94a3b8'}`,
        borderRadius: 10,
        background: dark ? '#1a1d2455' : '#f8fafc99',
        boxSizing: 'border-box',
      }}
    >
      <div style={{ fontWeight: 700, fontSize: 13, color: dark ? '#94a3b8' : '#475569', padding: '9px 13px' }}>
        {data.label as string}
      </div>
    </div>
  )
}
