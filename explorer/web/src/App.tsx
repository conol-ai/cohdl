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
import { explorerNets, ownerScope } from './hierarchy'
import { buildGraph, focusSubset, type GNode, type Graph } from './transform'
import { layout } from './layout'
import { assignRegions, type ViewConfig } from './views'
import { netWireColor, railColor } from './palette'
import {
  DetailedNode,
  BoundaryNode,
  FootprintPreview,
  PartNode,
  RegionNode,
  detailedPins,
  detailedSize,
} from './nodes'
import { LaneEdge } from './edges'
import { toPng } from 'html-to-image'
import { BoardLayout } from './BoardLayout'
import { passiveTerminals, wireHandles } from './handles'
import { layoutScope } from './boardGeometry'

const nodeTypes = { part: PartNode, region: RegionNode, detailed: DetailedNode, boundary: BoundaryNode }
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
  const [layoutFrame, setLayoutFrame] = useState<'board' | 'local'>('local')
  const [scope, setScope] = useState<string | null>(() => {
    const value = new URLSearchParams(location.search).get('scope')
    return value === 'all' ? null : value ?? ''
  })
  const [jump, setJump] = useState<{ id?: string; net?: string } | null>(null)
  const allNets = useMemo(() => model ? explorerNets(model) : [], [model])
  const currentSubdesign = model?.subdesigns?.find((s) => s.path === scope)
  const navigateScope = useCallback((path: string | null, target?: { id?: string; net?: string }) => {
    setScope(path)
    setActiveView('')
    setRegionFocus('')
    setSel([])
    setSelNet('')
    setQuery('')
    setJump(target ?? null)
  }, [])

  // A live edit can remove or rename the open use site.
  useEffect(() => {
    if (model && scope && !model.subdesigns?.some((s) => s.path === scope)) navigateScope('')
  }, [model, scope, navigateScope])
  /** Node id -> position produced by the layout engine (reset target). */
  const pristine = useRef<Map<string, { x: number; y: number }>>(new Map())
  /** Latest resetLayout, so the key handler binds once. */
  const resetLayoutRef = useRef<(() => void) | null>(null)
  const [mode, setMode] = useState<'overview' | 'sch' | 'layout'>(() => {
    const value = new URLSearchParams(location.search).get('mode')
    return value === 'sch' || value === 'layout' ? value : 'overview'
  })
  const dark = mode !== 'overview'

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
    let cancelled = false
    fetch(`/views/${model.design}.view.json`)
      .then((r) => (r.ok ? r.json() : null))
      .then((c) => {
        if (cancelled) return
        setViewCfg(c)
        const pv = params.get('view')
        if (c && pv && c.views.some((v: any) => v.name === pv)) {
          setActiveView(pv)
          setScope(null)
        }
      })
      .catch(() => { if (!cancelled) setViewCfg(null) })
    return () => { cancelled = true }
  }, [model?.design])

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
    if (mode === 'layout') {
      setGraph(null)
      if (jump?.id) setSel([jump.id])
      if (jump?.net) setSelNet(jump.net)
      return
    }
    let cancelled = false
    const g = buildGraph(model, scope)
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
      if (cancelled) return
      const target = jump?.id ? gg.location.get(jump.id) ?? jump.id : undefined
      const preset = target ? [target] : params.get('select')?.split(',') ?? []
      setSelNet(jump?.net ?? '')
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
          type: n.ports ? 'boundary' : mode === 'sch' && n.kind === 'ic' ? 'detailed' : 'part',
          // Explicit dimensions also preserve reused nodes across scope/live
          // changes: an unchanged DOM size may not fire ResizeObserver again.
          // Pin-level renderers refresh their own DOM handle measurements.
          width: n.width,
          height: n.height,
          ...(n.ports || (mode === 'sch' && n.kind === 'ic')
            ? {}
            : {
                handles: n.kind === 'passive' ? passiveTerminals(n).flatMap((p) => (['source', 'target'] as const).map((type) => ({
                  id: p.id, type, position: p.side === 'left' ? Position.Left : Position.Right,
                  x: p.x, y: p.y, width: 4, height: 4,
                }))) : [
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
      const nodesById = new Map(gg.nodes.map((n) => [n.id, n]))
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
            ...wireHandles(e, nodesById, mode === 'sch'),
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
      // Fit the completed layout, not React Flow's previous node store.
      // A scope change can otherwise fit stale bounds before its new nodes
      // arrive, leaving the new schematic zoomed in and mostly offscreen.
      const boxes = gg.nodes.map((n) => {
        const p = positions.get(n.id) ?? { x: 0, y: 0 }
        const region = layoutRegions
          ? regionBoxes.get(`region:${layoutRegions.byNode.get(n.id) ?? 'Other'}`)
          : undefined
        return { x: p.x + (region?.x ?? 0), y: p.y + (region?.y ?? 0), width: n.width, height: n.height }
      })
      boxes.push(...regionBoxes.values())
      if (boxes.length) {
        const x = Math.min(...boxes.map((b) => b.x))
        const y = Math.min(...boxes.map((b) => b.y))
        const width = Math.max(...boxes.map((b) => b.x + b.width)) - x
        const height = Math.max(...boxes.map((b) => b.y + b.height)) - y
        requestAnimationFrame(() => {
          if (!cancelled) rf.current?.fitBounds({ x, y, width, height }, { duration: 250, padding: 0.2 })
        })
      }
    }).catch((e) => {
      if (!cancelled) setErr(`Layout: ${String(e)}`)
    })
    return () => { cancelled = true }
  }, [model, viewCfg, activeView, mode, regionFocus, scope, jump])

  // Focus paths (multi-select) + net highlight
  useEffect(() => {
    if (!graph) return
    const keep = focusSubset(graph, sel)
    const active = keep.size > 0
    const netNodes = new Set<string>()
    const netPins = new Map<string, string[]>()
    if (selNet) {
      for (const n of graph.nodes) {
        const pins = Object.entries(n.pinNets).filter(([, net]) => net === selNet).map(([pin]) => pin)
        if (pins.length || n.railTags.includes(selNet) || (n.kind === 'net' && n.title === selNet)) {
          netNodes.add(n.id)
          netPins.set(n.id, pins)
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
  const selInst: Instance | undefined = mode === 'layout'
    ? model?.instances.find((i) => sel.length === 1 && i.path === sel[0]) : selNode?.inst
  const physicalScope = useMemo(() => model ? layoutScope(model, scope, layoutFrame) : undefined, [model, scope, layoutFrame])
  const selectedBoardPlacement = model?.layout?.placements.find((p) => p.instance === selInst?.path)
  const selectedPlacement = mode === 'layout' ? physicalScope?.placements.find((p) => p.instance === selInst?.path) : selectedBoardPlacement
  const localPlacementView = mode === 'layout' && physicalScope?.frame === 'local'
  const localPlacementScope = model && selInst ? model.subdesigns?.find((s) => s.path === ownerScope(model, selInst.path) && s.local_placements?.some((p) => p.instance === selInst.path)) : undefined
  const inspectedSubdesign = selNode?.subdesign ?? currentSubdesign
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
    () => (selNet ? allNets.find((n) => n.name === selNet) : undefined),
    [selNet, allNets],
  )

  /** Selected instance's logical pin -> net, for the clickable pin table. */
  const pinNetOf = useMemo(() => {
    const m = new Map<string, string>()
    if (!selInst || !model) return m
    for (const n of model.nets)
      for (const mem of n.members)
        if (mem.instance_path === selInst.path) m.set(mem.logical_pin, n.name)
    return m
  }, [selInst, model])

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
      if (mode === 'layout') {
        if (model && scope && !id.startsWith(`${scope}::`)) navigateScope(ownerScope(model, id), { id })
        else { setSel([id]); setSelNet(''); setJump({ id }) }
        return
      }
      const loc = graph?.location.get(id) ?? id
      if (!nodes.some((n) => n.id === loc) && model) {
        navigateScope(ownerScope(model, id), { id })
        return
      }
      setSelNet('')
      setSel([loc])
      setNodes((ns) => ns.map((n) => ({ ...n, selected: n.id === loc })))
      flyTo(loc)
    },
    [flyTo, nodes, graph, model, navigateScope, mode, scope],
  )
  const locateNet = useCallback(
    (name: string) => {
      if (mode === 'layout') {
        setSel([])
        setSelNet(name)
        setJump(null)
        return
      }
      const first = graph?.nodes.find((n) => Object.values(n.pinNets).includes(name) || n.railTags.includes(name))
      if (!first && model) {
        const member = allNets.find((n) => n.name === name)?.members[0]
        const owner = member ? ownerScope(model, member.instance_path)
          : model.subdesigns?.find((s) => s.ports.some((p) => p.net === name))?.path ?? ''
        navigateScope(owner, { net: name })
        return
      }
      setSel([])
      setNodes((ns) => ns.map((n) => ({ ...n, selected: false })))
      setSelNet(name)
      if (first) flyTo(first.id)
    },
    [model, allNets, graph, flyTo, navigateScope, mode],
  )

  const hits = useMemo(() => {
    const q = query.trim().toLowerCase()
    if (!q || !model) return [] as { id: string; kind: 'part' | 'net' | 'subdesign'; label: string; sub: string }[]
    const out: { id: string; kind: 'part' | 'net' | 'subdesign'; label: string; sub: string }[] = []
    for (const s of model.subdesigns ?? [])
      if ([s.path, s.definition, ...s.ports.map((p) => p.name)].join(' ').toLowerCase().includes(q))
        out.push({ id: s.path, kind: 'subdesign', label: s.path, sub: shortName(s.definition) })
    for (const i of model.instances)
      if ([i.path, i.designator, i.device_fq, i.part?.mpn].join(' ').toLowerCase().includes(q))
        out.push({ id: i.path, kind: 'part', label: `${i.designator ?? ''} ${shortName(i.device_fq)}`.trim(), sub: i.path })
    for (const n of allNets)
      if (n.name.toLowerCase().includes(q))
        out.push({ id: n.name, kind: 'net', label: n.name, sub: `${n.members.length} pins` })
    const rank = (h: { label: string }) => (h.label.toLowerCase().startsWith(q) ? 0 : 1)
    return out.sort((a, b) => rank(a) - rank(b)).slice(0, 12)
  }, [query, model, allNets])

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
          {!!model.subdesigns?.length && <span> · {model.subdesigns.length} subdesigns</span>}
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
          {sel.length >= 2 && <span style={{ color: '#f43f5e' }}> · {mode === 'layout' ? 'selected' : 'trace'}: {sel.length} parts</span>}
          {selNet && <span style={{ color: '#0ea5e9' }}> · net: {selNet}</span>}
          <span style={{ color: '#9ca3af' }}> {mode === 'layout' ? '(select a part to inspect its placement)' : '(drag/⌘-click parts to trace, Esc to reset)'}</span>
          <div style={{ marginTop: 5, position: 'relative' }}>
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && hits[0]) {
                  hits[0].kind === 'net' ? locateNet(hits[0].id)
                    : hits[0].kind === 'subdesign' ? navigateScope(hits[0].id) : locateNode(hits[0].id)
                  setQuery('')
                } else if (e.key === 'Escape') setQuery('')
              }}
              placeholder="Search parts, paths, subdesigns or nets…"
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
                      h.kind === 'net' ? locateNet(h.id)
                        : h.kind === 'subdesign' ? navigateScope(h.id) : locateNode(h.id)
                      setQuery('')
                    }}
                    style={{
                      padding: '4px 8px', cursor: 'pointer', display: 'flex',
                      justifyContent: 'space-between', gap: 8, fontSize: 11,
                      borderBottom: `1px solid ${dark ? '#22262e' : '#f3f4f6'}`,
                    }}
                  >
                    <span style={{ color: h.kind === 'net' ? '#22d3ee' : panelFg }}>
                      {h.kind === 'net' ? '⎯ ' : h.kind === 'subdesign' ? '◇ ' : '▢ '}
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
                const el = document.querySelector(mode === 'layout' ? '.board-layout' : '.react-flow') as HTMLElement
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
            {mode !== 'layout' && <button
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
            </button>}
            {chip('Overview', mode === 'overview', () => setMode('overview'))}
            {chip('SCH view', mode === 'sch', () => setMode('sch'))}
            {chip('Layout', mode === 'layout', () => { setMode('layout'); setActiveView(''); setRegionFocus('') }, '#047857')}
            {viewCfg && mode !== 'layout' &&
              ['', ...viewCfg.views.map((v) => v.name)].map((v) =>
                chip(v || 'No regions', activeView === v, () => { setActiveView(v); setScope(null); setJump(null) }, '#0e7490'),
              )}
          </div>
          {!!model.subdesigns?.length && (
            <div style={{ marginTop: 6, display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
              <label htmlFor="hierarchy-scope">Hierarchy</label>
              <select id="hierarchy-scope" value={scope ?? 'all'} onChange={(e) => navigateScope(e.target.value === 'all' ? null : e.target.value)}
                style={{ maxWidth: 300, fontSize: 11, background: panelBg, color: panelFg }}>
                <option value="">{model.design}</option>
                {model.subdesigns.map((s) => <option key={s.path} value={s.path}>{s.path}</option>)}
                <option value="all">All parts (flattened)</option>
              </select>
              {chip(model.design, scope === '', () => navigateScope(''), '#7c3aed')}
              {(model.subdesigns ?? []).filter((s) => scope === s.path || scope?.startsWith(`${s.path}::`)).map((s) => (
                chip(shortName(s.path), scope === s.path, () => navigateScope(s.path), '#7c3aed')
              ))}
              <span style={{ color: '#9ca3af', fontSize: 10 }}>{mode === 'layout' ? localPlacementView ? `${shortName(scope ?? '')} local coordinates` : 'Resolved board coordinates' : 'Double-click a subdesign to open'}</span>
            </div>
          )}
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
        {mode === 'layout' ? <BoardLayout model={model} scope={scope} frame={layoutFrame} onFrame={setLayoutFrame} selected={sel} selectedNet={selNet} focus={jump}
          onScope={(path) => { setLayoutFrame('local'); navigateScope(path) }}
          onSelect={(paths) => { setSel(paths); setSelNet(''); setJump(null) }} onNet={locateNet} /> : <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          onInit={(i) => (rf.current = i)}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onSelectionChange={onSelectionChange}
          onNodeDoubleClick={(_, n) => {
            const subdesign = (n.data.g as GNode | undefined)?.subdesign
            if (subdesign) navigateScope(subdesign.path)
          }}
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
        </ReactFlow>}
      </div>
      {(selInst || selNetObj || aggInsts.length > 0 || inspectedSubdesign) && (
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
          {inspectedSubdesign && (
            <div style={{ marginBottom: 16 }}>
              <h3 style={{ margin: '0 0 4px', color: '#8b5cf6' }}>{shortName(inspectedSubdesign.path)}</h3>
              <div style={{ overflowWrap: 'anywhere' }}>{inspectedSubdesign.path}</div>
              <div style={{ color: '#6b7280', margin: '4px 0' }}>subdesign {inspectedSubdesign.definition}</div>
              <div style={{ color: '#6b7280', margin: '4px 0' }}>
                {model.instances.filter((i) => i.path.startsWith(`${inspectedSubdesign.path}::`)).length} contained parts
                {' · '}{inspectedSubdesign.ports.length} ports
              </div>
              {scope !== inspectedSubdesign.path && <button onClick={() => navigateScope(inspectedSubdesign.path)}>Open subdesign</button>}
              <div style={{ color: '#6b7280', margin: '6px 0', overflowWrap: 'anywhere' }}>
                source {inspectedSubdesign.span.file}:{inspectedSubdesign.span.line}
              </div>
              <table style={{ width: '100%', borderCollapse: 'collapse', textAlign: 'left' }}>
                <thead><tr><th>port</th><th>obligation</th><th>outside</th></tr></thead>
                <tbody>{inspectedSubdesign.ports.map((p) => (
                  <tr key={p.name} style={{ borderTop: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}` }}>
                    <td style={{ padding: '5px 0' }}>
                      <b>{p.name}</b> <span style={{ color: '#6b7280' }}>Pin</span>
                      {p.net && <div><button onClick={() => locateNet(p.net!)} style={{ border: 0, padding: 0, background: 'none', color: '#0891b2', cursor: 'pointer', textAlign: 'left', overflowWrap: 'anywhere' }}>{p.net}</button></div>}
                    </td>
                    <td>{p.obligation}</td>
                    <td style={{ color: p.connected ? '#059669' : p.obligation === 'required' ? '#dc2626' : '#6b7280' }}>{p.connected ? 'wired' : 'open'}</td>
                  </tr>
                ))}</tbody>
              </table>
              {(model.subdesigns ?? []).filter((s) => s.parent === inspectedSubdesign.path).map((s) => (
                <div key={s.path} style={{ marginTop: 6 }}><button onClick={() => navigateScope(s.path)}>{shortName(s.path)} ↗</button></div>
              ))}
            </div>
          )}
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
              <div style={{ margin: '4px 0', overflowWrap: 'anywhere' }}>{selInst.path}</div>
              <div style={{ margin: '8px 0', padding: 8, borderRadius: 6, background: dark ? '#102c29' : '#ecfdf5' }}>
                <b>{localPlacementView ? 'Local placement' : 'Board placement'}</b>
                {selectedPlacement ? <>
                  <div>X {selectedPlacement.at_mm[0]} mm · Y {selectedPlacement.at_mm[1]} mm</div>
                  <div>{selectedPlacement.rotate}° · {selectedPlacement.side} side</div>
                </> : <div>{localPlacementView ? 'No placement in this local frame' : model.layout === undefined ? 'Layout data unavailable in this snapshot' : 'No resolved board placement'}</div>}
                {localPlacementView && <div style={{ marginTop: 4, color: '#9ca3af' }}>{selectedBoardPlacement ? 'Board placement available in Board coordinates' : 'Not yet placed on the board'}</div>}
                {mode !== 'layout' && (selectedBoardPlacement || localPlacementScope) && <button onClick={() => {
                  setLayoutFrame(selectedBoardPlacement ? 'board' : 'local')
                  setMode('layout')
                  navigateScope(localPlacementScope?.path ?? scope, { id: selInst.path })
                }}>Show in layout</button>}
              </div>
              {ownerScope(model, selInst.path) && <button onClick={() => navigateScope(ownerScope(model, selInst.path), { id: selInst.path })}>Open containing subdesign</button>}
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
              <div style={{ color: '#6b7280', marginTop: 8, fontSize: 11 }}>
                click a pin to light its net on the board
              </div>
              <table style={{ borderCollapse: 'collapse', marginTop: 2, width: '100%' }}>
                <thead>
                  <tr style={{ textAlign: 'left', color: '#6b7280' }}>
                    <th>pin</th>
                    <th>#</th>
                    <th>net</th>
                    <th>state</th>
                  </tr>
                </thead>
                <tbody>
                  {selInst.pins.map((p) => {
                    const net = pinNetOf.get(p.logical)
                    const on = !!net && net === selNet
                    return (
                      <tr
                        key={p.logical}
                        onClick={() => net && setSelNet(on ? '' : net)}
                        style={{
                          borderTop: `1px solid ${dark ? '#2a2f3a' : '#f3f4f6'}`,
                          cursor: net ? 'pointer' : 'default',
                          background: on ? '#22d3ee22' : undefined,
                          color: on ? '#22d3ee' : undefined,
                          fontWeight: on ? 700 : 400,
                        }}
                      >
                        <td>{p.logical}</td>
                        <td>{p.numbers.join(',')}</td>
                        <td style={{ color: on ? '#22d3ee' : net ? netWireColor(net, dark) : '#6b7280' }}>
                          {net ?? '—'}
                        </td>
                        <td style={{ color: on ? '#22d3ee' : p.connected ? '#10b981' : p.nc ? '#9ca3af' : '#d97706' }}>
                          {p.connected ? 'wired' : p.nc ? 'nc' : 'unused'}
                        </td>
                      </tr>
                    )
                  })}
                </tbody>
              </table>
            </>
          )}
          {selNetObj && (
            <>
              <h3
                style={{
                  margin: selInst ? '14px 0 4px' : '0 0 4px',
                  paddingTop: selInst ? 10 : 0,
                  borderTop: selInst ? `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}` : undefined,
                  color: '#22d3ee',
                }}
              >
                net {selNetObj.name}
              </h3>
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
                          <button onClick={() => locateNode(m.instance_path)} title={m.instance_path} style={{ border: 0, padding: 0, background: 'none', color: '#0891b2', cursor: 'pointer', fontWeight: 700 }}>{inst?.designator ?? shortName(m.instance_path)}</button>{' '}
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
              {(model.subdesigns ?? []).filter((s) => s.ports.some((p) => p.net === selNet)).map((s) => (
                <div key={s.path} style={{ marginTop: 6 }}>
                  <button onClick={() => navigateScope(s.path, { net: selNet })}>{s.path} ↗</button>
                  <span style={{ marginLeft: 4 }}>{s.ports.filter((p) => p.net === selNet).map((p) => p.name).join(', ')}</span>
                </div>
              ))}
            </>
          )}
        </div>
      )}
    </div>
  )
}
