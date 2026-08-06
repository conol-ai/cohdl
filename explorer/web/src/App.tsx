import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import {
  applyEdgeChanges,
  applyNodeChanges,
  Background,
  Controls,
  ReactFlow,
  type Edge,
  type EdgeChange,
  type Node,
  type NodeChange,
  Position,
  type ReactFlowInstance,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import type { ExplorerModel, Instance, Net } from './model'
import { shortName } from './model'
import { buildGraph, focusSubset, type GNode, type Graph } from './transform'
import { layout } from './layout'
import { assignRegions, type ViewConfig } from './views'
import { netWireColor, railColor } from './palette'
import {
  DetailedNode,
  FootprintPreview,
  PartNode,
  RegionNode,
  detailedPins,
  detailedSize,
} from './nodes'
import { LaneEdge } from './edges'
import { toPng } from 'html-to-image'

const nodeTypes = { part: PartNode, region: RegionNode, detailed: DetailedNode }
const edgeTypes = { lane: LaneEdge }

// ---------- app ----------
export default function App() {
  const [model, setModel] = useState<ExplorerModel | null>(null)
  const [graph, setGraph] = useState<Graph | null>(null)
  const [nodes, setNodes] = useState<Node[]>([])
  const [edges, setEdges] = useState<Edge[]>([])
  const [sel, setSel] = useState<string[]>([])
  const [selNet, setSelNet] = useState<string>('')
  const [err, setErr] = useState<string>('')
  const [viewCfg, setViewCfg] = useState<ViewConfig | null>(null)
  const [activeView, setActiveView] = useState<string>('')
  const [regionFocus, setRegionFocus] = useState<string>('')
  const [photoUrl, setPhotoUrl] = useState<string>('')
  const [query, setQuery] = useState<string>('')
  const [moved, setMoved] = useState(false)
  /** Node id -> position produced by the layout engine (reset target). */
  const pristine = useRef<Map<string, { x: number; y: number }>>(new Map())
  /** Latest resetLayout, so the key handler binds once. */
  const resetLayoutRef = useRef<(() => void) | null>(null)
  const [mode, setMode] = useState<'overview' | 'sch'>(
    new URLSearchParams(location.search).get('mode') === 'sch' ? 'sch' : 'overview',
  )
  const dark = mode === 'sch'

  const params = new URLSearchParams(location.search)
  const src = params.get('model') ?? '/rpi-pico2.json'

  // Live: prefer /api/model + SSE unless ?model= is explicit; ?nolive=1 for headless shots.
  useEffect(() => {
    let es: EventSource | null = null
    const nolive = params.has('nolive')
    const explicitModel = params.has('model')
    const loadApi = () =>
      fetch('/api/model')
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error('no api'))))
        .then(setModel)
    const start = explicitModel ? Promise.reject(new Error('static')) : loadApi()
    start
      .then(() => {
        if (nolive) return
        es = new EventSource('/api/events')
        let last = 0
        es.onmessage = (ev) => {
          const v = JSON.parse(ev.data).version
          if (v !== last) {
            last = v
            loadApi().catch(() => {})
          }
        }
      })
      .catch(() =>
        fetch(src)
          .then((r) => r.json())
          .then(setModel)
          .catch((e) => setErr(String(e))),
      )
    return () => es?.close()
  }, [src])

  useEffect(() => {
    if (!model) return
    fetch(`/views/${model.design}.view.json`)
      .then((r) => (r.ok ? r.json() : null))
      .then((c) => {
        setViewCfg(c)
        const pv = params.get('view')
        if (c && pv && c.views.some((v: any) => v.name === pv)) setActiveView(pv)
      })
      .catch(() => setViewCfg(null))
  }, [model])

  // rail/net chip clicks from inside custom nodes
  useEffect(() => {
    const h = (e: Event) => {
      setSel([])
      setSelNet((e as CustomEvent).detail as string)
    }
    window.addEventListener('explorer-select-net', h)
    return () => window.removeEventListener('explorer-select-net', h)
  }, [])

  // Keyboard: Esc clears selection, R restores the computed layout.
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null
      if (el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA')) return
      if (e.key === 'Escape') {
        setSel([])
        setSelNet('')
        setNodes((ns) => ns.map((n) => ({ ...n, selected: false })))
      } else if (e.key === 'r' || e.key === 'R') resetLayoutRef.current?.()
    }
    window.addEventListener('keydown', h)
    return () => window.removeEventListener('keydown', h)
  }, [])

  const viewDef = viewCfg?.views.find((v) => v.name === activeView)

  useEffect(() => setRegionFocus(''), [activeView, mode])

  useEffect(() => {
    if (!model) return
    const g = buildGraph(model)
    if (mode === 'sch')
      for (const n of g.nodes) {
        const sz = detailedSize(n)
        n.width = sz.width
        n.height = sz.height
      }
    const regions = viewDef ? assignRegions(g, viewDef) : undefined
    // Single-region focus: keep only that region's nodes (Split view).
    let gg = g
    if (regions && regionFocus) {
      const keep = new Set(
        g.nodes.filter((n) => regions.byNode.get(n.id) === regionFocus).map((n) => n.id),
      )
      gg = {
        ...g,
        nodes: g.nodes.filter((n) => keep.has(n.id)),
        edges: g.edges.filter((e) => keep.has(e.source) && keep.has(e.target)),
      }
    }
    setGraph(gg)
    const layoutRegions = regionFocus ? undefined : regions
    layout(gg, layoutRegions, { compact: mode === 'overview' }).then(({ positions, regionBoxes }) => {
      const preset = params.get('select')?.split(',') ?? []
      // SCH mode: each connected pin picks the side facing its counterpart
      // node (capped at the row count so node height stays fixed), then rows
      // within a side sort by the counterpart's y so wires run near-straight.
      // Wire-less pins (rail stubs) sink to the bottom rows.
      const orderByNode: Record<string, { l: string[]; r: string[] }> = {}
      if (mode === 'sch') {
        const box = (id: string) => {
          const p = positions.get(id)
          const nn = gg.nodes.find((x) => x.id === id)
          return p
            ? { cx: p.x + (nn?.width ?? 0) / 2, cy: p.y + (nn?.height ?? 0) / 2 }
            : null
        }
        for (const n of gg.nodes) {
          if (n.kind !== 'ic' || !n.inst) continue
          const me = box(n.id)
          if (!me) continue
          const pref = new Map<string, { side: 'l' | 'r'; cy: number }>()
          for (const e of gg.edges) {
            const [pin, other] =
              e.source === n.id && e.sourcePin
                ? [e.sourcePin, e.target]
                : e.target === n.id && e.targetPin
                  ? [e.targetPin, e.source]
                  : [undefined, '']
            if (!pin || pref.has(pin)) continue
            const ob = box(other)
            if (ob) pref.set(pin, { side: ob.cx < me.cx ? 'l' : 'r', cy: ob.cy })
          }
          const pins = detailedPins(n)
          const cap = Math.ceil(pins.length / 2)
          const lefts: { name: string; cy: number }[] = []
          const rights: { name: string; cy: number }[] = []
          for (const p of pins) {
            const pr = pref.get(p.name)
            let s: 'l' | 'r' = pr?.side ?? (lefts.length <= rights.length ? 'l' : 'r')
            if (s === 'l' && lefts.length >= cap) s = 'r'
            if (s === 'r' && rights.length >= cap) s = 'l'
            ;(s === 'l' ? lefts : rights).push({
              name: p.name,
              cy: pr?.cy ?? Number.MAX_SAFE_INTEGER,
            })
          }
          const byCy = (a: { name: string; cy: number }[]) =>
            a.slice().sort((x, y) => x.cy - y.cy).map((x) => x.name)
          orderByNode[n.id] = { l: byCy(lefts), r: byCy(rights) }
        }
      }
      setSel(preset.filter((p) => gg.nodes.some((n) => n.id === p)))
      const regionNodes: Node[] = [...regionBoxes.entries()].map(([id, b]) => ({
        id,
        type: 'region',
        position: { x: b.x, y: b.y },
        // Explicit dimensions: skip async measurement, which strands
        // extent:'parent' children in the hidden "uninitialized" state.
        width: b.width,
        height: b.height,
        data: { label: id.slice('region:'.length), width: b.width, height: b.height, dark },
        draggable: false,
        selectable: false,
        zIndex: -1,
      }))
      // Pristine layout snapshot — "Reset layout" restores exactly this.
      pristine.current = new Map([
        ...regionNodes.map((r) => [r.id, { ...r.position }] as const),
        ...gg.nodes.map((n) => [n.id, { ...(positions.get(n.id) ?? { x: 0, y: 0 }) }] as const),
      ])
      setMoved(false)
      setNodes([
        ...regionNodes,
        ...gg.nodes.map((n) => ({
          id: n.id,
          type: mode === 'sch' && n.kind === 'ic' ? 'detailed' : 'part',
          // Overview only: explicit dims + static handle coordinates (the
          // v12 SSR mechanism) make nodes AND edges render without any async
          // measurement — kills the cold-start "parts but no wires" race and
          // the extent:'parent' hidden stranding. SCH must stay on DOM
          // measurement or pin-handle edge anchoring breaks.
          ...(mode === 'sch'
            ? {}
            : {
                width: n.width,
                height: n.height,
                handles: [
                  { type: 'target' as const, position: Position.Left, x: 0, y: n.height / 2, width: 2, height: 2 },
                  { type: 'source' as const, position: Position.Right, x: n.width, y: n.height / 2, width: 2, height: 2 },
                ],
              }),
          position: positions.get(n.id) ?? { x: 0, y: 0 },
          ...(layoutRegions
            ? { parentId: `region:${layoutRegions.byNode.get(n.id) ?? 'Other'}`, extent: 'parent' as const }
            : {}),
          data: { g: n, dim: false, hl: false, dark, pinOrder: orderByNode[n.id] },
          selected: preset.includes(n.id),
        })),
      ])
      // Lane allocation: edges sharing a corridor (same node pair) spread
      // their vertical jogs evenly across the gap; singletons take a
      // net-hashed lane so unrelated corridors rarely coincide either.
      const lanes = new Array<number>(gg.edges.length).fill(0.5)
      {
        const corridors = new Map<string, number[]>()
        gg.edges.forEach((e, i) => {
          const k = [e.source, e.target].sort().join('~')
          corridors.set(k, [...(corridors.get(k) ?? []), i])
        })
        for (const idxs of corridors.values()) {
          if (idxs.length === 1) {
            let hsh = 0
            const s = gg.edges[idxs[0]].net
            for (let c = 0; c < s.length; c++) hsh = (hsh * 31 + s.charCodeAt(c)) >>> 0
            lanes[idxs[0]] = 0.3 + (hsh % 41) / 100
          } else {
            idxs.forEach((ei, j) => {
              lanes[ei] = 0.12 + (0.76 * (j + 1)) / (idxs.length + 1)
            })
          }
        }
      }
      // Pin-level anchors exist only on DetailedNode (ic); naming a handle
      // on any other node kind makes React Flow drop the edge silently.
      const kindOf = new Map(gg.nodes.map((n) => [n.id, n.kind]))
      // Absolute node bottoms feed the detour router (region children carry
      // parent-relative positions).
      const bottomOf = new Map(
        gg.nodes.map((n) => {
          const p = positions.get(n.id)
          if (!p) return [n.id, 0] as const
          const rb = layoutRegions
            ? regionBoxes.get(`region:${layoutRegions.byNode.get(n.id) ?? 'Other'}`)
            : undefined
          const h = mode === 'sch' && n.kind === 'ic' ? detailedSize(n).height : n.height
          return [n.id, (rb?.y ?? 0) + p.y + h] as const
        }),
      )
      setEdges(
        gg.edges.map((e, i) => {
          const wire = e.dashed ? (dark ? '#4b5563' : '#94a3b8') : netWireColor(e.net, dark)
          return {
            id: e.id,
            source: e.source,
            target: e.target,
            ...(mode === 'sch' && e.sourcePin && kindOf.get(e.source) === 'ic'
              ? { sourceHandle: 'p:' + e.sourcePin }
              : {}),
            ...(mode === 'sch' && e.targetPin && kindOf.get(e.target) === 'ic'
              ? { targetHandle: 'p:' + e.targetPin }
              : {}),
            label: e.label,
            type: 'lane',
            data: {
              wire,
              net: e.net,
              dashed: e.dashed,
              lane: lanes[i],
              // Detour corridor track: 14 distinct lanes so wires sharing the
              // area under a node never stack on one line.
              dlane: ((i * 5) % 14) / 14,
              sBot: bottomOf.get(e.source),
              tBot: bottomOf.get(e.target),
              labelBg: dark ? '#111318d9' : '#ffffffd9',
            },
            style: {
              stroke: wire,
              strokeWidth: e.dashed ? 1.1 : 1.5,
              ...(e.dashed ? { strokeDasharray: '4 3' } : {}),
            },
            labelStyle: { fontSize: 9, fill: dark ? '#c9d1d9' : '#374151' },
            labelBgStyle: dark ? { fill: '#111318', fillOpacity: 0.85 } : undefined,
          }
        }),
      )
    })
  }, [model, viewCfg, activeView, mode, regionFocus])

  // Focus paths (multi-select) + net highlight
  useEffect(() => {
    if (!graph) return
    const keep = focusSubset(graph, sel)
    const active = keep.size > 0
    const netNodes = new Set<string>()
    const netPins = new Map<string, string[]>()
    if (selNet && model) {
      const net = model.nets.find((x) => x.name === selNet)
      for (const mem of net?.members ?? []) {
        const loc = graph.location.get(mem.instance_path)
        if (loc && loc !== '(edge)') {
          netNodes.add(loc)
          netPins.set(loc, [...(netPins.get(loc) ?? []), mem.logical_pin])
        }
      }
    }
    setNodes((ns) =>
      ns.map((n) => ({
        ...n,
        data: {
          ...n.data,
          // A selected net dims everything off it, so its members pop.
          dim: (active && !keep.has(n.id)) || (!!selNet && !netNodes.has(n.id)),
          hl: netNodes.has(n.id),
          hlPins: netPins.get(n.id),
        },
      })),
    )
    setEdges((es) =>
      es.map((e) => {
        const onPath = !active || (keep.has(e.source) && keep.has(e.target))
        const isNet = selNet && (e.data?.net as string) === selNet
        const wire = (e.data?.wire as string) ?? '#2563eb'
        return {
          ...e,
          zIndex: isNet ? 10 : 0,
          style: {
            ...e.style,
            stroke: isNet
              ? '#22d3ee'
              : selNet
                ? dark ? '#232830' : '#eceff3'
                : onPath ? (active ? '#f43f5e' : wire) : dark ? '#2a2f3a' : '#e5e7eb',
            strokeWidth: isNet ? 3.2 : onPath && active ? 2.6 : (e.data?.dashed as boolean) ? 1.1 : 1.5,
            filter: isNet ? 'drop-shadow(0 0 5px #22d3ee)' : undefined,
          },
          labelStyle: { fontSize: 9, fill: isNet ? '#22d3ee' : onPath ? wire : '#6b7280' },
        }
      }),
    )
  }, [sel, selNet, graph])

  const onSelectionChange = useCallback(({ nodes: sn }: { nodes: Node[] }) => {
    setSel(sn.map((n) => n.id))
    if (sn.length > 0) setSelNet('')
  }, [])

  // Controlled flow (React Flow v12): change handlers are mandatory, or
  // clicks/selection/measurements are silently dropped.
  const onNodesChange = useCallback((changes: NodeChange[]) => {
    if (changes.some((c) => c.type === 'position')) setMoved(true)
    setNodes((ns) => applyNodeChanges(changes, ns))
  }, [])

  /** One-click undo of any manual dragging: positions back to the computed
   *  layout, selection cleared, viewport re-fitted. */
  const resetLayout = useCallback(() => {
    setNodes((ns) =>
      ns.map((n) => ({
        ...n,
        position: pristine.current.get(n.id) ?? n.position,
        selected: false,
        dragging: false,
      })),
    )
    setSel([])
    setSelNet('')
    setQuery('')
    setMoved(false)
    setTimeout(() => rf.current?.fitView({ duration: 600, padding: 0.1 }), 30)
  }, [])
  resetLayoutRef.current = resetLayout
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((es) => applyEdgeChanges(changes, es)),
    [],
  )

  const selNode: GNode | undefined = useMemo(
    () => (sel.length === 1 && graph ? graph.nodes.find((n) => n.id === sel[0]) : undefined),
    [sel, graph],
  )
  const selInst: Instance | undefined = selNode?.inst
  /** Members of a selected decoupling/pull aggregate, resolved to instances. */
  const aggInsts: Instance[] = useMemo(() => {
    if (!selNode?.aggMembers || !model) return []
    const byPath = new Map(model.instances.map((i) => [i.path, i]))
    return selNode.aggMembers.map((p) => byPath.get(p)).filter((i): i is Instance => !!i)
  }, [selNode, model])

  // Probe part photo quietly (fetch 404s don't spam the console like <img> does).
  useEffect(() => {
    setPhotoUrl('')
    const mpn = selInst?.part?.mpn
    if (!mpn) return
    let alive = true
    const url = `/api/photo?mpn=${encodeURIComponent(mpn)}`
    fetch(url).then((r) => {
      if (alive && r.ok) setPhotoUrl(url)
    }).catch(() => {})
    return () => {
      alive = false
    }
  }, [selInst])

  const selNetObj: Net | undefined = useMemo(
    () => (selNet ? model?.nets.find((n) => n.name === selNet) : undefined),
    [selNet, model],
  )

  // ---- search: designator / device / MPN / net -> select + fly to it
  const rf = useRef<ReactFlowInstance | null>(null)
  const nodeCenter = useCallback(
    (id: string) => {
      const n = nodes.find((x) => x.id === id)
      if (!n) return null
      const parent = n.parentId ? nodes.find((x) => x.id === n.parentId) : undefined
      const w = (n.width as number) ?? (n.measured?.width ?? 150)
      const h = (n.height as number) ?? (n.measured?.height ?? 60)
      return {
        x: (parent?.position.x ?? 0) + n.position.x + w / 2,
        y: (parent?.position.y ?? 0) + n.position.y + h / 2,
      }
    },
    [nodes],
  )
  const flyTo = useCallback(
    (id: string) => {
      const c = nodeCenter(id)
      if (c) rf.current?.setCenter(c.x, c.y, { zoom: 1.15, duration: 600 })
    },
    [nodeCenter],
  )
  const locateNode = useCallback(
    (id: string) => {
      setSelNet('')
      setSel([id])
      setNodes((ns) => ns.map((n) => ({ ...n, selected: n.id === id })))
      flyTo(id)
    },
    [flyTo],
  )
  const locateNet = useCallback(
    (name: string) => {
      setSel([])
      setNodes((ns) => ns.map((n) => ({ ...n, selected: false })))
      setSelNet(name)
      const first = model?.nets
        .find((n) => n.name === name)
        ?.members.map((mm) => graph?.location.get(mm.instance_path))
        .find((l) => l && l !== '(edge)')
      if (first) flyTo(first)
    },
    [model, graph, flyTo],
  )

  const hits = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q || !graph || !model) return [] as { id: string; kind: 'part' | 'net'; label: string; sub: string }[]
    const out: { id: string; kind: 'part' | 'net'; label: string; sub: string }[] = []
    for (const n of graph.nodes) {
      if (n.kind === 'net') continue
      const i = n.inst
      const hay = [
        i?.designator,
        i ? shortName(i.device_fq) : n.title,
        i?.part?.mpn,
        n.title,
        ...(n.aggMembers ?? []),
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase()
      if (hay.includes(q))
        out.push({
          id: n.id,
          kind: 'part',
          label: n.title,
          sub: i?.part?.mpn ?? (n.aggMembers ? `${n.aggMembers.length} caps` : ''),
        })
    }
    for (const n of model.nets)
      if (n.name.toLowerCase().includes(q))
        out.push({ id: n.name, kind: 'net', label: n.name, sub: `${n.members.length} pins` })
    const rank = (h: { label: string }) => (h.label.toLowerCase().startsWith(q) ? 0 : 1)
    return out.sort((a, b) => rank(a) - rank(b)).slice(0, 12)
  }, [query, graph, model])

  const chip = (label: string, on: boolean, onClick: () => void, color = '#2563eb') => (
    <button
      key={label}
      onClick={onClick}
      style={{
        fontSize: 11,
        padding: '2px 10px',
        borderRadius: 12,
        border: `1px solid ${on ? color : '#d1d5db'}`,
        background: on ? color : dark ? '#22262e' : '#fff',
        color: on ? '#fff' : dark ? '#c9d1d9' : '#374151',
        cursor: 'pointer',
      }}
    >
      {label}
    </button>
  )

  if (err) return <div style={{ padding: 20, color: '#b91c1c' }}>Failed to load: {err}</div>
  if (!model) return <div style={{ padding: 20 }}>Loading…</div>

  const panelBg = dark ? '#181c22' : '#fff'
  const panelFg = dark ? '#e5e7eb' : '#111827'

  return (
    <div style={{ display: 'flex', height: '100vh', fontFamily: 'system-ui', background: dark ? '#111318' : '#fff' }}>
      <div style={{ flex: 1, position: 'relative' }}>
        <div
          style={{
            position: 'absolute',
            zIndex: 10,
            background: dark ? '#181c22ee' : '#ffffffee',
            color: panelFg,
            padding: '6px 12px',
            borderRadius: 8,
            margin: 8,
            fontSize: 12,
            border: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
          }}
        >
          <b>{model.design}</b> · {model.instances.length} parts · {model.nets.length} nets ·{' '}
          {model.verdict}
          {(model as any).live_error && (
            <div style={{ color: '#f87171', maxWidth: 480 }}>
              ⚠ source currently fails to compile (showing last good state)
            </div>
          )}
          {model.verdict !== 'pass' &&
            (() => {
              const errs = model.diagnostics.filter((d) => d.severity === 'error')
              const e0 = errs[0]
              return e0 ? (
                <div style={{ color: '#f87171', maxWidth: 480, marginTop: 3 }}>
                  ⚠ {errs.length} error{errs.length > 1 ? 's' : ''} — {e0.code}: {e0.message.slice(0, 90)}
                  {e0.span && (
                    <span style={{ color: '#9ca3af' }}>
                      {' '}({e0.span.file.split('/').pop()}:{e0.span.line})
                    </span>
                  )}
                </div>
              ) : null
            })()}
          {sel.length >= 2 && <span style={{ color: '#f43f5e' }}> · trace: {sel.length} parts</span>}
          {selNet && <span style={{ color: '#0ea5e9' }}> · net: {selNet}</span>}
          <span style={{ color: '#9ca3af' }}> (drag/⌘-click parts to trace, Esc to reset)</span>
          <div style={{ marginTop: 5, position: 'relative' }}>
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && hits[0]) {
                  hits[0].kind === 'net' ? locateNet(hits[0].id) : locateNode(hits[0].id)
                  setQuery('')
                } else if (e.key === 'Escape') setQuery('')
              }}
              placeholder="Search parts, MPN or nets…  (Enter = jump)"
              style={{
                width: 300, fontSize: 11, padding: '4px 8px', borderRadius: 6,
                border: `1px solid ${dark ? '#2a2f3a' : '#d1d5db'}`,
                background: dark ? '#22262e' : '#fff', color: panelFg, outline: 'none',
              }}
            />
            {hits.length > 0 && (
              <div
                style={{
                  position: 'absolute', top: 28, left: 0, width: 300, maxHeight: 260,
                  overflow: 'auto', zIndex: 20, borderRadius: 6,
                  border: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
                  background: dark ? '#181c22' : '#fff',
                  boxShadow: '0 6px 20px #0006',
                }}
              >
                {hits.map((h) => (
                  <div
                    key={h.kind + h.id}
                    onClick={() => {
                      h.kind === 'net' ? locateNet(h.id) : locateNode(h.id)
                      setQuery('')
                    }}
                    style={{
                      padding: '4px 8px', cursor: 'pointer', display: 'flex',
                      justifyContent: 'space-between', gap: 8, fontSize: 11,
                      borderBottom: `1px solid ${dark ? '#22262e' : '#f3f4f6'}`,
                    }}
                  >
                    <span style={{ color: h.kind === 'net' ? '#22d3ee' : panelFg }}>
                      {h.kind === 'net' ? '⎯ ' : '▢ '}
                      {h.label}
                    </span>
                    <span style={{ color: '#6b7280' }}>{h.sub}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
          <div style={{ marginTop: 5, display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            <button
              onClick={() => {
                const el = document.querySelector('.react-flow') as HTMLElement
                if (!el) return
                toPng(el, {
                  backgroundColor: dark ? '#111318' : '#ffffff',
                  pixelRatio: 2,
                }).then((url) => {
                  const a = document.createElement('a')
                  a.download = `${model.design}-${mode}${activeView ? '-' + activeView : ''}${regionFocus ? '-' + regionFocus : ''}.png`
                  a.href = url
                  a.click()
                })
              }}
              style={{ fontSize: 11, padding: '2px 10px', borderRadius: 12, border: '1px solid #059669', background: dark ? '#22262e' : '#fff', color: '#10b981', cursor: 'pointer' }}
            >
              Export PNG
            </button>
            <button
              onClick={resetLayout}
              title="Undo any manual moves: restore the computed layout and refit the view (R)"
              style={{
                fontSize: 11, padding: '2px 10px', borderRadius: 12, cursor: 'pointer',
                border: `1px solid ${moved ? '#f59e0b' : dark ? '#2a2f3a' : '#d1d5db'}`,
                background: moved ? '#f59e0b' : dark ? '#22262e' : '#fff',
                color: moved ? '#fff' : dark ? '#c9d1d9' : '#374151',
              }}
            >
              ⟲ Reset layout
            </button>
            {chip(mode === 'sch' ? 'SCH view' : 'Overview', true, () => setMode(mode === 'sch' ? 'overview' : 'sch'))}
            {viewCfg &&
              ['', ...viewCfg.views.map((v) => v.name)].map((v) =>
                chip(v || 'All', activeView === v, () => setActiveView(v), '#0e7490'),
              )}
          </div>
          {viewDef && (
            <div style={{ marginTop: 5, display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
              <span style={{ color: '#9ca3af', fontSize: 10 }}>regions:</span>
              {chip('Combined', regionFocus === '', () => setRegionFocus(''), '#475569')}
              {viewDef.regions.map((r) =>
                chip(r.name, regionFocus === r.name, () => setRegionFocus(r.name), '#475569'),
              )}
            </div>
          )}
        </div>
        <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          onInit={(i) => (rf.current = i)}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onSelectionChange={onSelectionChange}
          onEdgeClick={(_, e) => {
            if (!(e.data?.dashed as boolean)) {
              setSel([])
              setSelNet((e.data?.net as string) ?? '')
            }
          }}
          onPaneClick={() => {
            setSelNet('')
          }}
          minZoom={0.08}
          fitView
          selectionOnDrag
          panOnDrag={[1, 2]}
          panOnScroll
          zoomOnScroll={false}
          zoomOnPinch
          colorMode={dark ? 'dark' : 'light'}
          proOptions={{ hideAttribution: true }}
        >
          <Background color={dark ? '#2a2f3a' : undefined} />
          <Controls />
        </ReactFlow>
      </div>
      {(selInst || selNetObj || aggInsts.length > 0) && (
        <div
          style={{
            width: 330,
            borderLeft: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
            padding: 14,
            overflow: 'auto',
            fontSize: 12,
            background: panelBg,
            color: panelFg,
          }}
        >
          {aggInsts.length > 0 && selNode && (
            <>
              <h3 style={{ margin: '0 0 2px' }}>{selNode.title}</h3>
              <div style={{ color: '#6b7280', marginBottom: 6 }}>
                {selNode.title.includes('decoupling')
                  ? 'Decoupling group — capacitors bridging power rails at this IC. Grouped because both ends sit on rails, so there is no signal wire to draw; the dashed line marks the host.'
                  : 'Rail-to-rail group — bypass/pull parts across the same rail pair.'}
              </div>
              <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 8 }}>
                {selNode.railTags.map((r) => (
                  <span
                    key={r}
                    onClick={() => locateNet(r)}
                    style={{ background: railColor(r), color: '#fff', borderRadius: 3, padding: '1px 5px', fontSize: 10, cursor: 'pointer' }}
                  >
                    {r}
                  </span>
                ))}
              </div>
              <table style={{ width: '100%', borderCollapse: 'collapse' }}>
                <thead>
                  <tr style={{ color: '#6b7280', textAlign: 'left' }}>
                    <th>part</th>
                    <th>value</th>
                    <th>MPN</th>
                  </tr>
                </thead>
                <tbody>
                  {aggInsts.map((i) => (
                    <tr key={i.path} style={{ borderTop: `1px solid ${dark ? '#22262e' : '#f3f4f6'}` }}>
                      <td>
                        <b>{i.designator}</b>
                      </td>
                      <td>{i.specs.map((s) => s.value).join(' ')}</td>
                      <td style={{ color: '#6b7280' }}>{i.part?.mpn}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
              <div style={{ color: '#6b7280', marginTop: 8, fontSize: 11 }}>
                source {aggInsts[0].span.file}:{aggInsts[0].span.line}
              </div>
            </>
          )}
          {selInst && (
            <>
              {photoUrl && (
                <img src={photoUrl} style={{ width: '100%', borderRadius: 8, marginBottom: 8 }} />
              )}
              <h3 style={{ margin: '0 0 4px' }}>
                {selInst.designator} {shortName(selInst.device_fq)}
              </h3>
              <div style={{ color: '#6b7280' }}>{selInst.device_fq}</div>
              {selInst.part && (
                <div style={{ margin: '6px 0' }}>
                  <b>{selInst.part.mfr}</b> {selInst.part.mpn}
                  {selInst.part.footprint && (
                    <div style={{ color: '#6b7280' }}>{shortName(selInst.part.footprint)}</div>
                  )}
                  {selInst.part.footprint && model.footprints?.[selInst.part.footprint] && (
                    <>
                      <FootprintPreview geo={model.footprints[selInst.part.footprint]} dark={dark} />
                      <div style={{ color: '#6b7280', fontSize: 10 }}>
                        {model.footprints[selInst.part.footprint].pads.length} pads
                        {(() => {
                          const c = model.footprints[selInst.part.footprint].courtyard
                          return c && c.size.length >= 2 ? ` · courtyard ${c.size[0]} × ${c.size[1]} mm` : ''
                        })()}
                        {model.footprints[selInst.part.footprint].pads.some((p) => p.pth)
                          ? ' · through-hole'
                          : ' · SMD'}
                      </div>
                    </>
                  )}
                </div>
              )}
              {selInst.impl_traits.length > 0 && (
                <div style={{ color: '#6b7280', fontSize: 11 }}>
                  {selInst.impl_traits.map(shortName).join(' · ')}
                </div>
              )}
              {selInst.specs.length > 0 && (
                <div style={{ marginTop: 4 }}>
                  {selInst.specs.map((s) => `${s.name} = ${s.value}`).join(' · ')}
                </div>
              )}
              {selInst.placement_hint && (
                <div style={{ marginTop: 4, color: '#0e7490' }}>📍 {selInst.placement_hint}</div>
              )}
              <div style={{ color: '#6b7280', margin: '6px 0' }}>
                source {selInst.span.file}:{selInst.span.line}
              </div>
              {selInst.docs
                .filter((d) => /\.(png|jpe?g|webp|gif)$/i.test(d.name))
                .map((d) => (
                  <img
                    key={d.name}
                    src={`/api/file?p=${encodeURIComponent(d.abs)}`}
                    style={{ width: '100%', borderRadius: 8, margin: '6px 0' }}
                  />
                ))}
              {selInst.docs
                .filter((d) => /\.pdf$/i.test(d.name))
                .map((d) => (
                  <div key={d.name}>
                    <a
                      href={`/api/file?p=${encodeURIComponent(d.abs)}`}
                      target="_blank"
                      style={{ color: '#2563eb' }}
                    >
                      📄 {d.name.split('/').pop()} ↗
                    </a>
                  </div>
                ))}
              {selInst.part?.mpn && (
                <div style={{ margin: '6px 0', display: 'flex', gap: 10 }}>
                  <a
                    href={`https://www.google.com/search?q=${encodeURIComponent(selInst.part.mpn + ' datasheet')}`}
                    target="_blank"
                    style={{ color: '#0ea5e9' }}
                  >
                    Datasheet ↗
                  </a>
                  <a
                    href={`https://octopart.com/search?q=${encodeURIComponent(selInst.part.mpn)}`}
                    target="_blank"
                    style={{ color: '#0ea5e9' }}
                  >
                    Octopart ↗
                  </a>
                </div>
              )}
              <table style={{ borderCollapse: 'collapse', marginTop: 8, width: '100%' }}>
                <thead>
                  <tr style={{ textAlign: 'left', color: '#6b7280' }}>
                    <th>pin</th>
                    <th>#</th>
                    <th>role</th>
                    <th>state</th>
                  </tr>
                </thead>
                <tbody>
                  {selInst.pins.map((p) => (
                    <tr key={p.logical} style={{ borderTop: `1px solid ${dark ? '#2a2f3a' : '#f3f4f6'}` }}>
                      <td>{p.logical}</td>
                      <td>{p.numbers.join(',')}</td>
                      <td>{p.role}</td>
                      <td style={{ color: p.connected ? '#10b981' : p.nc ? '#9ca3af' : '#d97706' }}>
                        {p.connected ? 'wired' : p.nc ? 'nc' : 'unused'}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </>
          )}
          {selNetObj && !selInst && (
            <>
              <h3 style={{ margin: '0 0 4px', color: '#0ea5e9' }}>net {selNetObj.name}</h3>
              <div style={{ color: '#6b7280' }}>
                {selNetObj.is_gnd ? 'ground' : selNetObj.voltage ? `rail ${selNetObj.voltage}` : 'signal'} ·{' '}
                {selNetObj.members.length} pins
              </div>
              <table style={{ borderCollapse: 'collapse', marginTop: 8, width: '100%' }}>
                <thead>
                  <tr style={{ textAlign: 'left', color: '#6b7280' }}>
                    <th>part</th>
                    <th>pin</th>
                    <th>#</th>
                  </tr>
                </thead>
                <tbody>
                  {selNetObj.members.map((m, i) => {
                    const inst = model.instances.find((x) => x.path === m.instance_path)
                    return (
                      <tr key={i} style={{ borderTop: `1px solid ${dark ? '#2a2f3a' : '#f3f4f6'}` }}>
                        <td>
                          <b>{inst?.designator ?? shortName(m.instance_path)}</b>{' '}
                          <span style={{ color: '#6b7280' }}>
                            {inst ? shortName(inst.device_fq) : ''}
                          </span>
                        </td>
                        <td>{m.logical_pin}</td>
                        <td>{m.numbers.join(',')}</td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </>
          )}
        </div>
      )}
    </div>
  )
}
