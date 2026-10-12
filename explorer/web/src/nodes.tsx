// Custom React Flow node renderers: compact overview parts, mini passives,
// SCH pin-level ICs, region frames, and the to-scale footprint preview.

import { useLayoutEffect } from 'react'
import { Handle, Position, useUpdateNodeInternals, type NodeProps } from '@xyflow/react'
import type { FootprintGeo } from './model'
import { shortName } from './model'
import type { GNode } from './transform'
import { netWireColor, railColor, selectNet } from './palette'
import { passiveTerminals } from './handles'
import { FootprintPreview } from './FootprintGeometry'
import { DETAILED_WIDTH, DETAILED_HEADER, DETAILED_ROW, detailedPinY } from './deviceGeometry'
export { DETAILED_WIDTH, DETAILED_HEADER, DETAILED_ROW, detailedPinY } from './deviceGeometry'
import { PinTarget, type PinIdentity } from './PinTarget'
import { partPresentation } from './partPresentation'
import { Star } from 'lucide-react'
import { GroupIndexContent } from './GroupIndexContent'
import type { GroupIndexEntry } from './groupIndex'

function Mark({ marked }: { marked: unknown }) { return marked ? <Star className="node-important" size={13} fill="currentColor" aria-label="Important" /> : null }
function CenterHandles() { return <><Handle type="target" position={Position.Left} style={{ opacity: 0 }} /><Handle type="source" position={Position.Right} style={{ opacity: 0 }} /></> }

/** Logical boundaries have named Pin ports and no physical pin numbers. */
export function BoundaryNode({ id, data, selected }: NodeProps) {
  const n = data.g as GNode
  const updateInternals = useUpdateNodeInternals()
  useLayoutEffect(() => { updateInternals(id) }, [id, n, updateInternals])
  const dark = data.dark as boolean
  const hlPins = (data.hlPins ?? []) as string[]
  const compact = !!data.compact
  if (compact && data.index) return <div className="system-group-node" style={{ position: 'relative', width: n.width, height: n.height, boxSizing: 'border-box', border: `2px solid ${selected ? '#f59e0b' : data.hl ? '#22d3ee' : '#64748b'}`, borderRadius: 6, background: dark ? '#22262e' : '#fff', color: dark ? '#e5e7eb' : '#1f2937', opacity: data.dim ? .18 : 1 }}>
    <Mark marked={data.marked} /><CenterHandles />
    <GroupIndexContent entry={data.index as GroupIndexEntry} onOpen={data.onGroup as (group: string) => void} />
  </div>
  return (
    <div style={{
      position: 'relative',
      width: n.width, height: n.height, boxSizing: 'border-box',
      border: `2px ${n.kind === 'port' ? 'dashed' : 'solid'} ${selected ? '#f59e0b' : data.hl ? '#22d3ee' : '#64748b'}`,
      borderRadius: 6, background: dark ? '#22262e' : '#ffffff',
      color: dark ? '#e5e7eb' : '#1f2937', opacity: data.dim ? 0.18 : 1,
    }}>
      <Mark marked={data.marked} />
      {!!data.neutral && <CenterHandles />}
      <div title={n.title} style={{ padding: compact ? '4px 10px' : '6px 10px', fontSize: compact ? 22 : 12, lineHeight: compact ? '22px' : undefined, fontWeight: 700, overflow: 'hidden', overflowWrap: 'anywhere', whiteSpace: compact ? 'normal' : 'nowrap' }}>
        {n.title}{n.kind === 'subdesign' && ' ↗'}
        <div style={{ fontSize: compact ? 14 : 10, lineHeight: compact ? '16px' : undefined, fontWeight: 400, color: dark ? '#9ca3af' : '#6b7280' }}>{n.sub}</div>
      </div>
      {compact && <div style={{ padding: '0 10px', fontSize: 12, lineHeight: '14px' }}>{new Set(n.ports?.map((p) => p.net)).size} networks</div>}
      {(compact ? data.neutral ? [] : (n.ports ?? []).filter(p => p.net === data.traceNet) : n.ports ?? []).map((p, index) => (
        <div key={p.name} role={!compact && p.net ? 'button' : undefined} tabIndex={!compact && p.net ? 0 : undefined} title={p.member ? `${p.member.instance_path}.${p.member.logical_pin} (${p.member.numbers.join(', ')}) · ${p.net}` : p.name}
          onKeyDown={(e) => { if (p.net && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); selectNet(p.net) } }} onClick={() => p.net && selectNet(p.net)} style={{
          position: 'absolute', top: compact ? 56 + (index + 0.5) * 28 / (n.ports?.length || 1) : 46 + index * 18, left: 0, right: 0,
          height: compact ? 0 : 18, padding: compact ? 0 : '0 10px', fontSize: 10,
          color: hlPins.includes(p.name) ? '#06b6d4' : undefined,
          cursor: p.net ? 'pointer' : 'default',
          display: 'flex', justifyContent: 'space-between', gap: 12,
        }}>
          {!data.neutral && <Handle id={`p:${p.name}`} type="target" position={Position.Left} style={{ width: 5, height: 5, background: '#8b5cf6' }} />}
          {!compact && <span style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minWidth: 0 }}>{p.member ? `${shortName(p.member.instance_path)}.${p.member.logical_pin} (${p.member.numbers.join(', ')})` : p.name}{p.obligation === 'required' ? ' *' : ''}</span>}
          {!compact && <span style={{ opacity: 0.75, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
            {p.net ?? 'open'}
          </span>}
          {!data.neutral && <Handle id={`p:${p.name}`} type="source" position={Position.Right} style={{ width: 5, height: 5, background: '#8b5cf6' }} />}
        </div>
      ))}
    </div>
  )
}

export function PartNode({ id, data, selected }: NodeProps) {
  const n = data.g as GNode
  const updateInternals = useUpdateNodeInternals()
  useLayoutEffect(() => { updateInternals(id) }, [id, n, updateInternals])
  const dark = data.dark as boolean
  if (n.kind === 'net') {
    const c = data.neutral ? (dark ? '#9ca3af' : '#64748b') : netWireColor(n.title, dark)
    return (
      <div
        role="button" tabIndex={0} aria-label={n.id.startsWith('bundle:') ? `Open ${n.title} bundle` : n.title}
        onClick={(e) => { e.stopPropagation(); n.id.startsWith('bundle:') ? (data.onBundle as (id: string) => void)?.(n.id) : selectNet(n.title) }}
        onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); n.id.startsWith('bundle:') ? (data.onBundle as (id: string) => void)?.(n.id) : selectNet(n.title) } }}
        style={{
          width: n.width, height: n.height, borderRadius: 11,
          border: `1.5px solid ${c}`, color: c, cursor: 'pointer',
          background: dark ? '#181c22' : '#fff',
          opacity: data.dim ? 0.25 : 1,
          fontSize: data.neutral ? 20 : 9.5, fontWeight: 700, display: 'flex',
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
    // Each logical terminal owns its side, independent of edge direction.
    return (
      <div
        style={{
          width: n.width, height: n.height, boxSizing: 'border-box',
          position: 'relative',
          border: `${hl ? 2 : 1.5}px solid ${selected ? '#f59e0b' : hl ? '#22d3ee' : border}`,
          borderRadius: 5, background: hl ? (dark ? '#0e3a44' : '#ecfeff') : bg,
          color: hl ? (dark ? '#a5f3fc' : '#0e7490') : fg,
          boxShadow: hl ? '0 0 0 3px #22d3ee55, 0 0 14px #22d3ee66' : undefined,
          opacity: data.dim ? 0.18 : 1,
          display: 'flex', alignItems: 'center', justifyContent: 'center',
          gap: 4, fontSize: 9, fontWeight: 600, padding: '0 4px',
        }}
      >
        <Mark marked={data.marked} />
        {!!data.neutral && <CenterHandles />}
        {!data.neutral && passiveTerminals(n).flatMap((p) => (['source', 'target'] as const).map((type) => (
          <Handle key={p.id + type} id={p.id} type={type}
            position={p.side === 'left' ? Position.Left : Position.Right}
            title={p.label} aria-label={p.label}
            style={{ top: p.y, width: 4, height: 4, background: border, border: 0 }} />
        )))}
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
        position: 'relative',
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
      <Mark marked={data.marked} />
      <Handle type="target" position={Position.Left} style={{ opacity: 0 }} />
      <Handle type="source" position={Position.Right} style={{ opacity: 0 }} />
      <div title={n.title} style={{ fontWeight: 700, fontSize: 11, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {n.title}
        {n.inst && n.pinsConnected === 0 && (
          <span style={{ color: '#ef4444', marginLeft: 5 }}>✕ unwired</span>
        )}
      </div>
      {n.sub && <div title={n.sub} style={{ color: dark ? '#9ca3af' : '#6b7280', fontSize: 9, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{n.sub}</div>}
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
            <div key={d} title={d} style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{d}</div>
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
export { FootprintPreview } from './FootprintGeometry'

const pinNetColor = (net: string) =>
  net.startsWith('GND') ? '#9ca3af' : /^V|VBUS|VSYS/.test(net) ? '#ef4444' : '#34d399'

export function detailedPins(n: GNode): { name: string; num: string; net: string; state: string }[] {
  if (!n.inst) return []
  const pins = n.inst.pins
    .map((p) => ({ name: p.logical, num: p.numbers.join(','), net: n.pinNets[p.logical] ?? '', role: p.role, state: p.nc ? 'NC' : p.connected ? 'wired' : 'open' }))
  const rank = (r: string) => (r.startsWith('power') ? 0 : 1)
  pins.sort((a, b) => rank(a.role) - rank(b.role) || a.name.localeCompare(b.name))
  return pins
}

export function detailedSize(n: GNode): { width: number; height: number } {
  if (!n.inst) return { width: n.width, height: n.height }
  const rows = Math.ceil(detailedPins(n).length / 2)
  return { width: DETAILED_WIDTH, height: DETAILED_HEADER + rows * DETAILED_ROW + 10 }
}

export function DetailedNode(props: NodeProps) {
  const { data, selected } = props
  const n = data.g as GNode
  const updateInternals = useUpdateNodeInternals()
  useLayoutEffect(() => { updateInternals(props.id) }, [props.id, n, data.pinOrder, updateInternals])
  if (!n.inst) return <PartNode {...props} />
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
  const dark = !!data.dark
  const presentation = partPresentation(n.inst, data.sourceValue as string | undefined)
  const subtitle = presentation.subtitle || n.sub
  const pinRow = (p: { name: string; num: string; net: string; state: string }, i: number, side: 'l' | 'r') => {
    const y = detailedPinY(i)
    const on = hlPins.has(p.name)
    const c = on ? '#22d3ee' : pinNetColor(p.net)
    return (
      <PinTarget key={side + p.name} identity={{ instance: n.id, pin: p.name, net: p.net }} selected={on} onActivate={(data.onPin as ((pin: PinIdentity) => void) | undefined) ?? (() => {})} className="device-pin-row" style={{ position: 'absolute', top: y - DETAILED_ROW / 2, height: DETAILED_ROW, [side === 'l' ? 'left' : 'right']: 2, width: DETAILED_WIDTH / 2 - 4, boxSizing: 'border-box', display: 'flex', flexDirection: side === 'l' ? 'row' : 'row-reverse', gap: 4, padding: '0 4px', fontSize: 12, alignItems: 'center', background: on ? '#22d3ee22' : undefined, borderRadius: 3 }}>
        <span style={{ color: on ? '#67e8f9' : '#6b7280', minWidth: 14, textAlign: side === 'l' ? 'left' : 'right' }}>{p.num}</span>
        <span title={p.name} style={{ color: on ? '#a5f3fc' : dark ? '#e5e7eb' : '#17212e', fontWeight: on ? 800 : 600, fontSize: 12, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', minWidth: 0 }}>{p.name}</span>
        <span
          title={`${p.name} (${p.num}): ${p.net || p.state}`} style={{ color: c, fontWeight: on ? 800 : 400, cursor: 'pointer', marginLeft: side === 'l' ? 'auto' : 0, marginRight: side === 'r' ? 'auto' : 0, maxWidth: 82, flexShrink: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}
        >
          {p.net || p.state}
        </span>
        {!data.neutral && <Handle id={'p:' + p.name} type="source" position={side === 'l' ? Position.Left : Position.Right} style={{ top: DETAILED_ROW / 2, opacity: 0, [side === 'l' ? 'left' : 'right']: -2 }} />}
        {!data.neutral && <Handle id={'p:' + p.name} type="target" position={side === 'l' ? Position.Left : Position.Right} style={{ top: DETAILED_ROW / 2, opacity: 0, [side === 'l' ? 'left' : 'right']: -2 }} />}
        <span
          style={{
            position: 'absolute', [side === 'l' ? 'left' : 'right']: on ? -7 : -5,
            top: on ? 3 : 5, width: on ? 10 : 6, height: on ? 10 : 6,
            borderRadius: 5, background: c,
            boxShadow: on ? '0 0 8px #22d3ee' : undefined,
          }}
        />
      </PinTarget>
    )
  }
  return (
    <div className="device-card" data-instance={n.id} style={{ width, height, background: dark ? '#22262e' : '#fff', opacity: data.dim ? 0.18 : 1, border: `${nodeHl ? 2.5 : 1.5}px solid ${selected ? '#f59e0b' : nodeHl ? '#22d3ee' : '#596475'}`, boxShadow: nodeHl ? '0 0 0 3px #22d3ee44' : undefined, borderRadius: 6, position: 'relative', boxSizing: 'border-box' }}>
      <Mark marked={data.marked} />
      {!!data.neutral && <CenterHandles />}
      <div className="detailed-heading device-header" title={`${presentation.title} ${subtitle}`} style={{ height: DETAILED_HEADER, boxSizing: 'border-box', background: dark ? '#161a20' : '#f1f5f9', borderRadius: '6px 6px 0 0', padding: '8px 10px', overflow: 'hidden' }}>
        <div style={{ color: dark ? '#f3f4f6' : '#17212e', fontWeight: 700, fontSize: 16, lineHeight: '22px', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{presentation.title}</div>
        <div style={{ display: 'flex', gap: 10, marginTop: 5 }}>
          <div className="device-footprint" style={{ width: 88, flexShrink: 0 }}><FootprintPreview geo={data.geo as FootprintGeo | undefined} dark={dark} width={88} height={64} /><div style={{ fontSize: 10, color: dark ? '#aeb6c2' : '#536170', textAlign: 'center' }}>Local footprint</div></div>
          <div style={{ minWidth: 0, color: dark ? '#c6cdd7' : '#374151', fontSize: 13, lineHeight: '18px', overflowWrap: 'anywhere', maxHeight: 76, overflow: 'hidden' }}><span>{subtitle || 'Identity unspecified'}</span>{!!data.suggested && <div style={{ color: dark ? '#7dd3fc' : '#0369a1', fontSize: 12 }}>Core suggestion</div>}</div>
        </div>
      </div>
      {left.map((p, i) => pinRow(p, i, 'l'))}
      {right.map((p, i) => pinRow(p, i, 'r'))}
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
      <div title={data.label as string} style={{ fontWeight: 700, fontSize: 13, color: dark ? '#94a3b8' : '#475569', padding: '9px 13px', overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>
        {data.label as string}
      </div>
    </div>
  )
}
