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
import { focusSubset, type GNode, type Graph } from './transform'
import { layout } from './layout'
import { buildViewGraph, projectGroupUnion, collapseGroups, groupNodeId, parseViewConfig, groupMemberPaths, type ViewConfig } from './views'
import { netWireColor, railColor } from './palette'
import {
  DetailedNode,
  BoundaryNode,
  PartNode,
  RegionNode,
  detailedPins,
  detailedSize,
} from './nodes'
import { LaneEdge } from './edges'
import { toPng } from 'html-to-image'
import { BoardLayout, type BoardLayoutHandle } from './BoardLayout'
import { passiveTerminals, wireHandles } from './handles'
import { layoutScope, type LayoutSide } from './boardGeometry'
import { ArrowLeft, Download, FilterX, Maximize, RotateCcw, Share2, ChevronRight, Star, X, PanelLeft, PanelTop, MoreHorizontal } from 'lucide-react'
import { useCanvasFit } from './useCanvasFit'
import { GroupPanel } from './GroupPanel'
import { boundsOfRects, type BoardViewportSnapshot } from './viewport'
import type { InspectionContext } from './inspection'
import { searchCircuit, type SearchHit } from './search'
import { useInspectionNavigation } from './useInspectionNavigation'
import { InspectionLists } from './InspectionLists'
import { useDocuments } from './useDocuments'
import { ProvenanceInspector } from './ProvenanceInspector'
import { PartDetails } from './PartDetails'
import { useImportant } from './useImportant'
import { coreCandidates, partPresentation, sourcePartValues } from './partPresentation'
import { bundleConnections } from './connectivity'
import type { PinIdentity } from './PinTarget'
import { useProposals } from './useProposals'
import { ProposalReview } from './ProposalReview'
import { groupIndex } from './groupIndex'
import { OutsideDestinations } from './OutsideDestinations'
import { GroupIntentPanel } from './GroupIntentPanel'
import { defaultInsideGroups } from './navigation'
import './App.css'

const nodeTypes = { part: PartNode, region: RegionNode, detailed: DetailedNode, boundary: BoundaryNode }
const edgeTypes = { lane: LaneEdge }

// ---------- app ----------
export default function App() {
  const [model, setModel] = useState<ExplorerModel | null>(null)
  const documents = useDocuments(model)
  const sourceValues = useMemo(() => sourcePartValues(documents.registry), [documents.registry])
  const important = useImportant(model)
  const candidates = useMemo(() => model ? coreCandidates(model, sourceValues) : [], [model, sourceValues])
  const [showImportant, setShowImportant] = useState(false)
  const [bundleId, setBundleId] = useState('')
  const [graph, setGraph] = useState<Graph | null>(null)
  const [nodes, setNodes] = useState<Node[]>([])
  const [edges, setEdges] = useState<Edge[]>([])
  const [sel, setSel] = useState<string[]>(() => new URLSearchParams(location.search).get('select')?.split(',') ?? [])
  const [selNet, setSelNet] = useState<string>('')
  const [err, setErr] = useState<string>('')
  const [viewCfg, setViewCfg] = useState<ViewConfig | null>(null)
  const [candidateConfig, setCandidateConfig] = useState<ViewConfig | null>(null)
  const resolvedConfig = candidateConfig ?? viewCfg
  const candidateOrigin = useRef<InspectionContext | null>(null)
  const candidateTarget = useRef<Partial<InspectionContext> | null>(null)
  const sourceSignature = useMemo(() => JSON.stringify([model, viewCfg, documents.registry]), [model, viewCfg, documents.registry])
  const initialRequestedView = useRef((() => { try { return JSON.parse(new URLSearchParams(location.search).get('inspect') ?? '{}').view as string | undefined } catch { return undefined } })())
  const [viewError, setViewError] = useState('')
  const [viewsLoading, setViewsLoading] = useState(false)
  const [viewRetry, setViewRetry] = useState(0)
  const [layoutBusy, setLayoutBusy] = useState(false)
  const loadedViewDesign = useRef<string | null>(null)
  const canvasRef = useRef<HTMLDivElement>(null)
  const rf = useRef<ReactFlowInstance | null>(null)
  const boardCamera = useRef<BoardLayoutHandle | null>(null)
  const [boardRestore, setBoardRestore] = useState<BoardViewportSnapshot | null>(null)
  const [activeView, setActiveView] = useState<string>('')
  const [selectedGroups, setSelectedGroups] = useState<string[]>([])
  const [collapsedGroups, setCollapsedGroups] = useState<string[]>([])
  const [groupsOpen, setGroupsOpen] = useState(false)
  const [groupsVisible, setGroupsVisible] = useState(true)
  const [toolbarVisible, setToolbarVisible] = useState(true)
  const [bomQuery, setBomQuery] = useState('')
  const [connectionMode, setConnectionMode] = useState<InspectionContext['connectionMode']>('individual')
  const [selectedPin, setSelectedPin] = useState('')
  const [tab, setTab] = useState<InspectionContext['tab']>('circuit')
  const proposals = useProposals(model, tab === 'proposals')
  const [shareError, setShareError] = useState('')
  const [issueFocus, setIssueFocus] = useState('')
  const issueFocusRef = useRef<HTMLDivElement>(null)
  useEffect(() => { if (issueFocus) issueFocusRef.current?.focus() }, [issueFocus])
  const [query, setQuery] = useState<string>('')
  const [moved, setMoved] = useState(false)
  const [layoutFrame, setLayoutFrame] = useState<'board' | 'local'>('local')
  const [layoutSide, setLayoutSide] = useState<LayoutSide>('both')
  const [scope, setScope] = useState<string | null>(() => {
    const value = new URLSearchParams(location.search).get('scope')
    return value === 'all' ? null : value ?? ''
  })
  const [jump, setJump] = useState<{ id?: string; net?: string; selection?: string[]; locate?: boolean } | null>(null)
  const handledJump = useRef<typeof jump>(null)
  const selection = useRef({ parts: sel, net: selNet })
  selection.current = { parts: sel, net: selNet }
  const allNets = useMemo(() => model ? explorerNets(model) : [], [model])
  const currentSubdesign = model?.subdesigns?.find((s) => s.path === scope)
  /** Node id -> position produced by the layout engine (reset target). */
  const pristine = useRef<Map<string, { x: number; y: number }>>(new Map())
  /** Latest resetLayout, so the key handler binds once. */
  const resetLayoutRef = useRef<(() => void) | null>(null)
  const [mode, setMode] = useState<'overview' | 'sch' | 'layout'>(() => {
    const value = new URLSearchParams(location.search).get('mode')
    return value === 'sch' || value === 'layout' ? value : 'overview'
  })
  const dark = mode !== 'overview'
  const { fit, setBounds, setSelectionTarget, prepare, capture, manual, onMoveStart, onReady, generation: cameraGeneration } = useCanvasFit(canvasRef, rf, !!model && mode !== 'layout')
  const readContext = (): InspectionContext => ({ mode, connectionMode, scope, view: activeView, groups: selectedGroups, collapsed: collapsedGroups,
    selection: sel, pin: selectedPin, net: selNet, tab, layoutFrame, layoutSide,
    viewport: mode === 'layout' ? null : capture(), boardViewport: mode === 'layout' ? boardCamera.current?.capture() ?? null : null })
  const navigation = useInspectionNavigation({ model, config: resolvedConfig, sourceValues, ready: !!model && !documents.loading && !viewsLoading && loadedViewDesign.current === model.design, busy: layoutBusy,
    signature: JSON.stringify([mode, connectionMode, scope, activeView, selectedGroups, collapsedGroups, sel, selectedPin, selNet, tab, layoutFrame, layoutSide]), read: readContext,
    apply: (context) => {
      prepare(context.viewport)
      setBoardRestore(context.boardViewport)
      setMode(context.mode); setScope(context.scope); setActiveView(context.view)
      setConnectionMode(context.connectionMode)
      setSelectedGroups(context.groups); setCollapsedGroups(context.collapsed)
      setSel(context.selection); setSelectedPin(context.pin); setSelNet(context.net); setTab(context.tab)
      setLayoutFrame(context.layoutFrame); setLayoutSide(context.layoutSide)
      setJump({ selection: context.selection, net: context.net })
      setQuery(''); setBundleId('')
    } })
  useEffect(() => {
    if (!candidateTarget.current || viewsLoading || documents.loading || !model || loadedViewDesign.current !== model.design) return
    const target = candidateTarget.current
    candidateTarget.current = null
    navigation.navigate(target)
  }, [resolvedConfig, navigation.navigate, viewsLoading, documents.loading, model])
  useEffect(() => {
    if (candidateConfig && candidateOrigin.current) {
      setViewError(error => error || 'Temporary focus expired; review proposal again')
      candidateTarget.current = { ...candidateOrigin.current, viewport: null, boardViewport: null }
      candidateOrigin.current = null
    }
    setCandidateConfig(null)
  }, [sourceSignature])
  useEffect(() => {
    const requested = initialRequestedView.current
    if (!viewCfg || !requested) return
    if (!viewCfg.views.some(view => view.name === requested)) setViewError('Requested view is unavailable or temporary; showing a saved group inside. Review the proposal again to restore temporary focus.')
    initialRequestedView.current = undefined
  }, [viewCfg])
  const freshRange = (patch: Partial<InspectionContext>) => { setShowImportant(false); navigation.navigate({ ...patch, selection: [], pin: '', net: '', tab: 'circuit', viewport: null, boardViewport: null }) }
  const navigateScope = (path: string | null, target?: { id?: string; net?: string }) => target
    ? navigation.navigate({ scope: path, selection: target.id ? [target.id] : [], net: target.net ?? '', pin: '', viewport: null, boardViewport: null })
    : freshRange({ scope: path })
  const activatePin = useCallback((pin: PinIdentity) => {
    setSel([pin.instance]); setSelectedPin(pin.pin); setSelNet(pin.net); setBundleId('')
  }, [])
  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      const inspector = document.querySelector<HTMLElement>('.explorer-inspector')
      const network = inspector?.querySelector<HTMLElement>('.part-network')
      if (inspector) inspector.scrollTop = selectedPin && network ? inspector.scrollTop + network.getBoundingClientRect().top - inspector.getBoundingClientRect().top - 12 : 0
    })
    return () => cancelAnimationFrame(frame)
  }, [sel[0], selectedPin])

  const params = new URLSearchParams(location.search)
  const src = params.get('model') ?? '/rpi-pico2.json'
  useEffect(() => { setIssueFocus('') }, [model, mode, scope, activeView, selectedGroups, collapsedGroups, sel, selectedPin, selNet, tab])

  // Live: prefer /api/model + SSE unless ?model= is explicit; ?nolive=1 for headless shots.
  useEffect(() => {
    let es: EventSource | null = null
    let cancelled = false
    let generation = 0
    const nolive = params.has('nolive')
    const explicitModel = params.has('model')
    const loadApi = () => {
      const request = ++generation
      return fetch('/api/model')
        .then((r) => (r.ok ? r.json() : Promise.reject(new Error('no api'))))
        .then((next) => { if (!cancelled && request === generation) setModel(next) })
    }
    const start = explicitModel ? Promise.reject(new Error('static')) : loadApi()
    start
      .then(() => {
        if (nolive || cancelled) return
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
          .then((next) => { if (!cancelled) setModel(next) })
          .catch((e) => { if (!cancelled) setErr(String(e)) }),
      )
    return () => { cancelled = true; es?.close() }
  }, [src])

  useEffect(() => {
    if (!model) return
    let cancelled = false
    setViewsLoading(true)
    fetch(`/views/${encodeURIComponent(model.design)}.view.json`)
      .then(async (r) => {
        if (r.status === 404) throw new Error(`No group configuration for ${model.design}`)
        const body: unknown = await r.json()
        if (!r.ok) throw new Error(typeof body === 'object' && body !== null && 'error' in body ? String(body.error) : `View request failed (${r.status})`)
        return parseViewConfig(body, model.design)
      })
      .then((c) => {
        if (cancelled) return
        setViewError(error => error.startsWith('Requested view') || error.startsWith('Temporary focus') ? error : '')
        setViewCfg(c)
        setCandidateConfig(current => JSON.stringify(c) === JSON.stringify(viewCfg) ? current : null)
      })
      .catch((e) => {
        if (!cancelled) { setViewCfg(null); setViewError(String(e instanceof Error ? e.message : e)) }
      })
      .finally(() => { if (!cancelled) { loadedViewDesign.current = model.design; setViewsLoading(false) } })
    return () => { cancelled = true }
  }, [model, viewRetry])

  // rail/net chip clicks from inside custom nodes
  useEffect(() => {
    const h = (e: Event) => {
      setSel([])
      setSelectedPin(''); setBundleId('')
      setSelNet((e as CustomEvent).detail as string)
    }
    window.addEventListener('explorer-select-net', h)
    return () => window.removeEventListener('explorer-select-net', h)
  }, [])

  // Keyboard: Esc clears selection, R restores the computed layout.
  useEffect(() => {
    const h = (e: KeyboardEvent) => {
      const el = e.target as HTMLElement | null
      if (el && (['INPUT', 'TEXTAREA', 'SELECT'].includes(el.tagName) || el.isContentEditable)) return
      if (e.key === 'Escape') {
        setSel([])
        setSelNet('')
        setSelectedPin('')
        setNodes((ns) => ns.map((n) => ({ ...n, selected: false })))
      } else if (e.key === 'r' || e.key === 'R') resetLayoutRef.current?.()
    }
    window.addEventListener('keydown', h)
    return () => window.removeEventListener('keydown', h)
  }, [])

  const viewDef = resolvedConfig?.views.find((v) => v.name === activeView)
  const groupMembers = useMemo(() => model ? groupMemberPaths(model, viewDef) : new Map<string, Set<string>>(), [model, viewDef])
  const fullView = useMemo(() => model ? buildViewGraph(model, null, viewDef, { individual: mode === 'sch' }) : null, [model, viewDef, mode])
  const scopedView = useMemo(() => model ? (scope === null ? fullView : buildViewGraph(model, scope, viewDef, { individual: mode === 'sch' })) : null, [model, scope, viewDef, fullView, mode])
  const indexEntries = useMemo(() => model && fullView ? groupIndex(model, fullView.instanceRegions, sourceValues) : new Map(), [model, fullView, sourceValues])
  const enterGroup = useCallback((name: string) => {
    navigation.navigate({ mode: 'sch', scope: null, groups: [name], collapsed: [], selection: [], pin: '', net: '', tab: 'circuit', viewport: null, boardViewport: null })
  }, [navigation.navigate])
  const projection = useMemo(() => {
    if (!model || !scopedView || !fullView) return null
    const union = projectGroupUnion(model, scope, scopedView, selectedGroups)
    return mode === 'layout' ? union : collapseGroups(model, union, fullView, collapsedGroups)
  }, [model, scope, scopedView, selectedGroups, fullView, collapsedGroups, mode])
  const bundled = useMemo(() => model && projection ? bundleConnections(projection.graph, model) : null, [model, projection])
  const displayGraph = connectionMode === 'individual' ? projection?.graph : bundled?.graph
  const openBundle = useCallback((id: string) => { setBundleId(id); setSel([]); setSelectedPin('') }, [])

  useEffect(() => {
    if (!model || documents.loading || !projection || !displayGraph) return
    if (mode === 'layout') {
      setLayoutBusy(false)
      setGraph(null)
      if (jump?.id) setSel([jump.id])
      if (jump?.net) setSelNet(jump.net)
      return
    }
    let cancelled = false
    setLayoutBusy(true)
    setErr('')
    const neutral = connectionMode === 'bundled'
    const rank = (id: string) => {
      if (important.marks.has(id)) return -1
      const index = candidates.findIndex(c => c.instance.path === id)
      return index < 0 ? candidates.length : index
    }
    const nodeRank = (node: GNode) => Math.min(rank(node.id), ...(node.aggMembers ?? []).map(rank))
    const gg = { ...displayGraph, nodes: displayGraph.nodes.map((n) => ({ ...n })).sort((a, b) => nodeRank(a) - nodeRank(b)), edges: displayGraph.edges }
    for (const node of gg.nodes) if (node.inst) {
      const identity = partPresentation(node.inst, sourceValues.get(node.inst.path))
      node.title = node.kind === 'passive' ? `${identity.reference} ${identity.value || identity.category}` : identity.title
      node.sub = node.kind === 'passive' ? '' : identity.subtitle
    }
    if (mode === 'sch')
      for (const n of gg.nodes) {
        const sz = detailedSize(n)
        n.width = sz.width
        n.height = sz.height
      }
    if (mode === 'overview') for (const n of gg.nodes) {
      if (n.id.startsWith('group:')) { n.width = 300; n.height = 144 }
    }
    const layoutRegions = mode === 'overview' && (gg.nodes.length <= 14 || gg.nodes.every(n => n.id.startsWith('group:') || n.kind === 'net')) ? undefined : projection.regions
    const canvas = canvasRef.current?.getBoundingClientRect()
    const mobileOverview = mode === 'overview' && !layoutRegions && gg.nodes.length > 8 && !!canvas && canvas.width < 640
    layout(gg, layoutRegions, { compact: mode === 'overview', aspectRatio: canvas?.height ? canvas.width / canvas.height : 1.6, columns: mobileOverview ? 2 : undefined }).then(({ positions, regionBoxes, edgeRoutes }) => {
      if (cancelled) return
      setGraph(gg)
      const intent = jump !== handledJump.current ? jump : null
      const preset = selection.current.parts
      const trace = selection.current.net
      setSelNet(allNets.some((n) => n.name === trace) ? trace : '')
      handledJump.current = jump
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
          if (!n.inst) continue
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
      setSel(preset.filter((p) => gg.nodes.some((n) => n.id === (gg.location.get(p) ?? p))))
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
          type: n.ports ? 'boundary' : mode === 'sch' && n.inst ? 'detailed' : 'part',
          // Explicit dimensions also preserve reused nodes across scope/live
          // changes: an unchanged DOM size may not fire ResizeObserver again.
          // Pin-level renderers refresh their own DOM handle measurements.
          width: n.width,
          height: n.height,
          ...(n.ports || (mode === 'sch' && n.inst)
            ? {}
            : {
                handles: !neutral && n.kind === 'passive' ? passiveTerminals(n).flatMap((p) => (['source', 'target'] as const).map((type) => ({
                  id: p.id, type, position: p.side === 'left' ? Position.Left : Position.Right,
                  x: p.x, y: p.y, width: 4, height: 4,
                }))) : [
                  { type: 'target' as const, position: Position.Left, x: 0, y: n.height / 2, width: 2, height: 2 },
                  { type: 'source' as const, position: Position.Right, x: n.width, y: n.height / 2, width: 2, height: 2 },
                ],
              }),
          position: positions.get(n.id) ?? { x: 0, y: 0 },
          draggable: !neutral,
          ...(layoutRegions
            ? { parentId: `region:${layoutRegions.byNode.get(n.id) ?? 'Other'}`, extent: 'parent' as const }
            : {}),
          data: { g: n, dim: false, hl: false, dark, neutral, compact: mode === 'overview' && n.id.startsWith('group:'), index: indexEntries.get(n.title), onGroup: enterGroup, pinOrder: orderByNode[n.id], onPin: activatePin, onBundle: openBundle, geo: n.inst?.part?.footprint ? model.footprints[n.inst.part.footprint] : undefined, sourceValue: n.inst ? sourceValues.get(n.inst.path) : undefined, suggested: candidates.some(candidate => candidate.instance.path === n.id), marked: important.marks.has(n.id) || !!n.aggMembers?.some(p => important.marks.has(p)) },
          selected: preset.some((p) => (gg.location.get(p) ?? p) === n.id),
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
          const h = mode === 'sch' && n.inst ? detailedSize(n).height : n.height
          return [n.id, (rb?.y ?? 0) + p.y + h] as const
        }),
      )
      setEdges(
        gg.edges.map((e, i) => {
          const wire = neutral ? (dark ? '#9ca3af' : '#94a3b8') : e.dashed ? (dark ? '#4b5563' : '#94a3b8') : netWireColor(e.net, dark)
          return {
            id: e.id,
            source: e.source,
            target: e.target,
            ...(!neutral && !(mode === 'overview' && (e.source.startsWith('group:') || e.target.startsWith('group:'))) ? wireHandles(e, nodesById, mode === 'sch') : {}),
            label: e.label,
            type: 'lane',
            data: {
              wire,
              net: e.net,
              nets: e.nets,
              dashed: e.dashed,
              lane: lanes[i],
              // Detour corridor track: 14 distinct lanes so wires sharing the
              // area under a node never stack on one line.
              dlane: ((i * 5) % 14) / 14,
              sBot: bottomOf.get(e.source),
              tBot: bottomOf.get(e.target),
              labelBg: dark ? '#111318d9' : '#ffffffd9',
              ...(neutral && bundled?.bundles.has(e.id) ? { bundleId: e.id, onBundle: openBundle, route: edgeRoutes.get(e.id) } : {}),
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
      const targets = new Set(intent?.locate ? (intent.id ? [intent.id] : allNets.find((n) => n.name === intent.net)?.members.map((m) => m.instance_path) ?? []).map((p) => gg.location.get(p) ?? p) : [])
      if (intent?.locate && intent.net && !intent.id) for (const edge of gg.edges) { targets.add(edge.source); targets.add(edge.target) }
      const targetBounds = boundsOfRects(boxes.filter((_, index) => targets.has(gg.nodes[index].id)))
      const primaryIndex = gg.nodes.findIndex(n => nodeRank(n) < candidates.length)
      const primaryBox = boxes[primaryIndex < 0 ? 0 : primaryIndex] ?? null
      const selectedIndex = gg.nodes.findIndex(n => preset.some(path => (gg.location.get(path) ?? path) === n.id))
      setBounds(boundsOfRects([...boxes, ...regionBoxes.values()]), cameraGeneration, targetBounds, false, boxes, primaryBox, boxes[selectedIndex] ?? null)
      setLayoutBusy(false)
    }).catch((e) => {
      if (!cancelled) { setErr(`Layout: ${String(e)}`); setLayoutBusy(false) }
    })
    return () => { cancelled = true }
  }, [model, documents.loading, projection, displayGraph, mode, connectionMode, jump, setBounds, allNets, cameraGeneration, activatePin, openBundle, sourceValues, candidates, important.marks, indexEntries, enterGroup])

  useEffect(() => {
    setNodes(current => current.map(node => ({ ...node, data: { ...node.data, marked: important.marks.has(node.id) || !!(node.data.g as GNode | undefined)?.aggMembers?.some(p => important.marks.has(p)) } })))
  }, [important.marks])

  // Focus paths (multi-select) + net highlight
  useEffect(() => {
    if (!graph) return
    const keep = focusSubset(graph, sel.map((p) => graph.location.get(p) ?? p))
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
      for (const edge of graph.edges) if (edge.nets.includes(selNet)) {
        netNodes.add(edge.source)
        netNodes.add(edge.target)
      }
    }
    setNodes((ns) =>
      ns.map((n) => ({
        ...n,
        selected: sel.some(path => (graph.location.get(path) ?? path) === n.id),
        data: {
          ...n.data,
          traceNet: selNet,
          // A selected net dims everything off it, so its members pop.
          dim: (active && !keep.has(n.id)) || (!!selNet && !netNodes.has(n.id)),
          hl: netNodes.has(n.id),
          hlPins: selectedPin && sel.length === 1 && (graph.location.get(sel[0]) ?? sel[0]) === n.id ? [...(netPins.get(n.id) ?? []), selectedPin] : netPins.get(n.id),
        },
      })),
    )
    setEdges((es) =>
      es.map((e) => {
        const onPath = !active || (keep.has(e.source) && keep.has(e.target))
        const isNet = !!selNet && ((e.data?.net as string) === selNet || (e.data?.nets as string[] | undefined)?.includes(selNet))
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
  }, [sel, selNet, selectedPin, graph])

  // Controlled flow (React Flow v12): change handlers are mandatory, or
  // clicks/selection/measurements are silently dropped.
  const onNodesChange = useCallback((changes: NodeChange[]) => {
    if (changes.some((c) => c.type === 'position')) setMoved(true)
    const selected = changes.filter(change => change.type === 'select')
    if (selected.length) {
      const id = selected.find(change => change.selected)?.id
      const internal = id && !id.startsWith('bundle:') ? rf.current?.getInternalNode(id) : null
      setSelectionTarget(internal ? { ...internal.internals.positionAbsolute, width: internal.width ?? internal.measured.width ?? 0, height: internal.height ?? internal.measured.height ?? 0 } : null)
    }
    if (selected.length) setSel(current => {
      const next = current.filter(path => !selected.some(change => !change.selected && change.id === (graph?.location.get(path) ?? path)))
      for (const change of selected) if (change.selected && !next.some(path => (graph?.location.get(path) ?? path) === change.id)) next.push(change.id)
      return next
    })
    setNodes((ns) => applyNodeChanges(changes, ns))
  }, [graph, setSelectionTarget])

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
    setSelectedPin(''); setBundleId('')
    setQuery('')
    setMoved(false)
    fit()
  }, [fit])
  resetLayoutRef.current = resetLayout
  const onEdgesChange = useCallback(
    (changes: EdgeChange[]) => setEdges((es) => applyEdgeChanges(changes, es)),
    [],
  )

  const selNode: GNode | undefined = useMemo(
    () => (sel.length === 1 && graph ? graph.nodes.find((n) => n.id === (graph.location.get(sel[0]) ?? sel[0])) : undefined),
    [sel, graph],
  )
  const selInst: Instance | undefined = model?.instances.find((i) => sel.length === 1 && i.path === sel[0]) ?? selNode?.inst
  const evidenceGroups = selNode?.id === groupNodeId(activeView, selNode?.title ?? '') ? [selNode.title] : selectedGroups
  const physicalScope = useMemo(() => model ? layoutScope(model, scope, layoutFrame) : undefined, [model, scope, layoutFrame])
  const physicalPlacements = useMemo(() => physicalScope?.placements.filter((p) =>
    (!selectedGroups.length || projection?.represented.has(p.instance)) && (layoutSide === 'both' || p.side === layoutSide)) ?? [],
  [physicalScope, selectedGroups, projection, layoutSide])
  const selectedBoardPlacement = model?.layout?.placements.find((p) => p.instance === selInst?.path)
  const selectedPlacement = mode === 'layout' ? physicalScope?.placements.find((p) => p.instance === selInst?.path) : selectedBoardPlacement
  const localPlacementView = mode === 'layout' && physicalScope?.frame === 'local'
  const inspectedSubdesign = selNode?.subdesign ?? currentSubdesign
  /** Members of a selected decoupling/pull aggregate, resolved to instances. */
  const aggInsts: Instance[] = useMemo(() => {
    if (!selNode?.aggMembers || !model) return []
    const byPath = new Map(model.instances.map((i) => [i.path, i]))
    return selNode.aggMembers.map((p) => byPath.get(p)).filter((i): i is Instance => !!i)
  }, [selNode, model])


  const selNetObj: Net | undefined = useMemo(
    () => (selNet ? allNets.find((n) => n.name === selNet) : undefined),
    [selNet, allNets],
  )
  const visibleParts = useMemo(() => mode === 'layout' ? new Set(physicalPlacements.map(p => p.instance)) : projection?.represented ?? new Set<string>(), [mode, physicalPlacements, projection])
  const followEndpoint = (path: string, pin: string, net: string) => {
    locateTarget({ kind: 'pin', id: path, pin, label: path, sub: path, net }, tab === 'bom' ? 'bom' : 'circuit')
  }
  const insideScope = !viewDef && model?.subdesigns?.length ? '' : null
  const clearFilters = () => {
    freshRange({ mode: 'sch', scope: insideScope, groups: model ? defaultInsideGroups(model, viewDef, sourceValues) : [], collapsed: [], layoutSide: 'both' })
  }
  const toggleGroup = (name: string) => {
    const groups = selectedGroups.includes(name) ? selectedGroups.filter((g) => g !== name) : [...selectedGroups, name]
    freshRange({ mode: mode === 'layout' ? 'layout' : 'sch', scope: null, groups, collapsed: [] })
  }

  const locateTarget = (hit: SearchHit, targetTab: InspectionContext['tab'] = 'circuit') => {
    if (!model) return
    if (hit.kind === 'subdesign') { navigateScope(hit.id); return }
    if (hit.kind === 'net') {
      const net = allNets.find((n) => n.name === hit.id)
      const visible = net?.members.some((m) => mode === 'layout' ? physicalPlacements.some(placement => placement.instance === m.instance_path)
        : projection?.represented.has(m.instance_path) && !collapsedGroups.includes(fullView?.instanceRegions.get(m.instance_path) ?? 'Other'))
      const member = net?.members[0]
      const group = member && fullView?.instanceRegions.get(member.instance_path)
      navigation.navigate({ selection: [], pin: '', net: hit.id, tab: 'circuit', viewport: null, boardViewport: null,
        ...(!visible && member ? { scope: viewDef ? null : ownerScope(model, member.instance_path), groups: viewDef ? [group ?? 'Other'] : [], collapsed: collapsedGroups.filter((g) => g !== group) } : {}) })
      setJump({ net: hit.id, locate: true })
      return
    }
    const group = fullView?.instanceRegions.get(hit.id) ?? 'Other'
    const hidden = !projection?.represented.has(hit.id) || (mode !== 'layout' && collapsedGroups.includes(group))
    const placement = physicalScope?.placements.find((p) => p.instance === hit.id)
    navigation.navigate({ mode: mode === 'layout' ? 'layout' : 'sch', scope: viewDef ? null : ownerScope(model, hit.id),
      groups: hidden && viewDef ? [group] : selectedGroups, collapsed: collapsedGroups.filter((g) => g !== group),
      selection: [hit.id], pin: hit.pin ?? '', net: hit.net ?? '', tab: targetTab, viewport: null, boardViewport: null,
      layoutSide: mode === 'layout' && placement && layoutSide !== 'both' && placement.side !== layoutSide ? 'both' : layoutSide })
    setJump({ id: hit.id, net: hit.net ?? '', locate: true })
  }
  const locateNet = (name: string) => locateTarget({ kind: 'net', id: name, label: name, sub: name })
  const hits = useMemo(() => model ? searchCircuit(model, query, sourceValues).slice(0, 30) : [], [query, model, sourceValues])

  const chip = (label: string, on: boolean, onClick: () => void, color = '#2563eb') => (
    <button
      key={label}
      aria-pressed={on}
      onClick={onClick}
      style={{
        fontSize: 12,
        padding: '2px 10px',
        borderRadius: 4,
        border: `1px solid ${on ? color : '#d1d5db'}`,
        background: on ? color : dark ? '#22262e' : '#fff',
        color: on ? '#fff' : dark ? '#c9d1d9' : '#374151',
        cursor: 'pointer',
      }}
    >
      {label}
    </button>
  )

  if (err && !model) return <div role="alert" style={{ padding: 20, color: '#b91c1c' }}>Failed to load: {err}</div>
  if (!model) return <div style={{ padding: 20 }}>Loading…</div>

  const panelBg = dark ? '#181c22' : '#fff'
  const panelFg = dark ? '#e5e7eb' : '#111827'
  const groupsShown = matchMedia('(max-width: 680px)').matches ? groupsOpen : groupsVisible

  return (
    <div className={`explorer-app${dark ? ' dark' : ''}${groupsVisible ? '' : ' groups-hidden'}${tab === 'bom' ? ' has-bom' : ''}`}>
      <main className="explorer-main">
        <div className="panel-rail" aria-label="Panels">
          {viewDef && <button className="icon-tool" title={groupsShown ? 'Hide Groups' : 'Show Groups'} aria-label={groupsShown ? 'Hide Groups' : 'Show Groups'} aria-pressed={groupsShown} onClick={() => { const mobile = matchMedia('(max-width: 680px)').matches; setGroupsVisible(mobile ? true : !groupsVisible); setGroupsOpen(mobile ? !groupsOpen : false); if (mobile || !groupsVisible) requestAnimationFrame(() => document.querySelector<HTMLInputElement>('.group-panel .group-search')?.focus()) }}><PanelLeft size={18} /></button>}
          <button className="icon-tool" title={toolbarVisible ? 'Hide toolbar' : 'Show toolbar'} aria-label={toolbarVisible ? 'Hide toolbar' : 'Show toolbar'} aria-pressed={toolbarVisible} onClick={() => { setToolbarVisible(value => !value); if (!toolbarVisible) requestAnimationFrame(() => document.querySelector<HTMLInputElement>('[aria-label="Search circuit"]')?.focus()) }}><PanelTop size={18} /></button>
          <div className="inspection-tabs" role="tablist" aria-label="Inspection views">
            {(['circuit', 'bom'] as const).map((value, index, tabs) => <button key={value} role="tab" aria-selected={tab === value} tabIndex={tab === value ? 0 : -1}
              onClick={() => navigation.navigate({ tab: value })} onKeyDown={event => {
                const offset = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0
                if (offset || event.key === 'Home' || event.key === 'End') {
                  event.preventDefault()
                  const target = event.key === 'Home' ? 0 : event.key === 'End' ? tabs.length - 1 : (index + offset + tabs.length) % tabs.length
                  navigation.navigate({ tab: tabs[target] })
                  event.currentTarget.parentElement?.querySelectorAll<HTMLButtonElement>('button')[target].focus()
                }
              }}>{value === 'bom' ? 'BOM' : 'Circuit'}</button>)}
          </div>
          <details className="inspection-more"><summary aria-label="More inspection views" title="More inspection views"><MoreHorizontal size={18} /></summary><button onClick={() => navigation.navigate({ tab: 'issues' })}>Issues</button><button onClick={() => navigation.navigate({ tab: 'proposals' })}>Proposals</button></details>
          {(mode === 'overview' || collapsedGroups.length > 0) && <button className="context-command view-inside" onClick={() => { const groups = selectedGroups.length ? selectedGroups : defaultInsideGroups(model, viewDef, sourceValues); navigation.navigate({ mode: 'sch', scope, groups, collapsed: [], viewport: null, boardViewport: null }) }}><ChevronRight size={16} />View inside</button>}
        </div>
        <div
          className="explorer-toolbar"
          hidden={!toolbarVisible}
          style={{
            zIndex: 10,
            background: dark ? '#181c22ee' : '#ffffffee',
            color: panelFg,
            padding: '6px 12px',
            fontSize: 12,
            border: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
          }}
        >
          <b>{model.design}</b> · <span className="inspection-counts" data-parts={projection?.counts.parts ?? model.instances.length} data-total-parts={model.instances.length} data-nets={projection?.counts.nets ?? model.nets.length} data-total-nets={projection?.counts.totalNets ?? model.nets.length}>
            {projection?.counts.parts ?? model.instances.length}/{model.instances.length} parts · {projection?.counts.nets ?? model.nets.length}/{projection?.counts.totalNets ?? model.nets.length} nets
          </span> ·{' '}
          {model.verdict}
          {!!model.subdesigns?.length && <span> · {model.subdesigns.length} subdesigns</span>}
          {model.live_error && (
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
          <div style={{ marginTop: 5, position: 'relative' }}>
            <input
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && hits[0]) {
                  locateTarget(hits[0])
                  setQuery('')
                } else if (e.key === 'Escape') setQuery('')
              }}
              aria-label="Search circuit"
              placeholder="Search parts, pins or nets"
              style={{
                width: 'min(300px, 100%)', boxSizing: 'border-box', fontSize: 12, padding: '6px 8px', borderRadius: 4,
                border: `1px solid ${dark ? '#2a2f3a' : '#d1d5db'}`,
                background: dark ? '#22262e' : '#fff', color: panelFg, outline: 'none',
              }}
            />
            {query && <button className="icon-tool" title="Clear search" aria-label="Clear search" onClick={(e) => { setQuery(''); e.currentTarget.parentElement?.querySelector('input')?.focus() }}><X size={14} /></button>}
            {hits.length > 0 && (
              <div
                style={{
                  width: 'min(100%, 500px)', maxHeight: 200,
                  overflow: 'auto', zIndex: 20, borderRadius: 6,
                  border: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
                  background: dark ? '#181c22' : '#fff',
                  boxShadow: '0 6px 20px #0006',
                }}
              >
                {hits.map((h) => (
                  <button
                    type="button"
                    key={h.kind + h.id + (h.pin ?? '')}
                    data-search-kind={h.kind}
                    data-search-pin={h.pin}
                    onClick={() => {
                      locateTarget(h)
                      setQuery('')
                    }}
                    style={{
                      padding: '4px 8px', cursor: 'pointer', display: 'flex',
                      justifyContent: 'space-between', gap: 8, fontSize: 12,
                      width: '100%', border: 0, background: 'transparent', textAlign: 'left',
                      borderBottom: `1px solid ${dark ? '#22262e' : '#f3f4f6'}`,
                    }}
                  >
                    <span style={{ color: h.kind === 'net' ? '#22d3ee' : panelFg }}>
                      {h.kind === 'net' ? '⎯ ' : h.kind === 'subdesign' ? '◇ ' : '▢ '}
                      {h.label}
                    </span>
                    <span style={{ color: '#6b7280' }}>{h.sub}</span>
                  </button>
                ))}
              </div>
            )}
            {query.trim() && !hits.length && <div role="status">No results</div>}
          </div>
          <div style={{ marginTop: 5, display: 'flex', gap: 6, flexWrap: 'wrap' }}>
            <button
              className="icon-tool"
              title="Export PNG"
              aria-label="Export PNG"
              onClick={() => {
                const el = document.querySelector(mode === 'layout' ? '.board-layout' : '.react-flow') as HTMLElement
                if (!el) return
                toPng(el, {
                  backgroundColor: dark ? '#111318' : '#ffffff',
                  pixelRatio: 2,
                }).then((url) => {
                  const a = document.createElement('a')
                  a.download = `${model.design}-${mode}${activeView ? '-' + activeView : ''}${selectedGroups.length ? '-' + selectedGroups.join('-') : ''}.png`
                  a.href = url
                  a.click()
                })
              }}
            >
              <Download size={16} />
            </button>
            {mode !== 'layout' && <button className="icon-tool" onClick={fit} title="Fit circuit" aria-label="Fit circuit"><Maximize size={16} /></button>}
            <button className="context-command" onClick={clearFilters} title="Clear filters"><FilterX size={14} />Clear filters</button>
            <button className="context-command" onClick={navigation.back} disabled={!navigation.canBack} title="Back to previous inspection"><ArrowLeft size={14} />Back</button>
            <button className="icon-tool" title="Copy inspection link" aria-label="Copy inspection link" onClick={() => {
              navigation.sync()
              if (!navigator.clipboard) { setShareError(location.href); return }
              navigator.clipboard.writeText(location.href).then(() => setShareError(''), () => setShareError(location.href))
            }}><Share2 size={16} /></button>
            {mode !== 'layout' && <button
              className="icon-tool"
              onClick={resetLayout}
              title="Reset layout"
              aria-label="Reset layout"
              style={{ color: moved ? '#d97706' : undefined }}
            >
              <RotateCcw size={16} />
            </button>}
            {chip('Overview', mode === 'overview', () => freshRange({ mode: 'overview', scope, groups: selectedGroups, collapsed: collapsedGroups }))}
            {chip('SCH view', mode === 'sch', () => freshRange({ mode: 'sch', scope, groups: selectedGroups, collapsed: collapsedGroups }))}
            {mode !== 'layout' && <span className="connection-mode" role="group" aria-label="Wire mode">{(['individual', 'bundled'] as const).map(value => <button key={value} aria-pressed={connectionMode === value} onClick={() => navigation.navigate({ connectionMode: value, viewport: null })}>{value === 'individual' ? 'Individual' : 'Bundled'}</button>)}</span>}
            {resolvedConfig && mode !== 'layout' &&
              ['', ...resolvedConfig.views.filter(v => v.name !== 'Source pages').map((v) => v.name)].map((v) =>
                chip(v || 'No regions', activeView === v, () => freshRange({ view: v, groups: [], collapsed: mode === 'overview' ? resolvedConfig.views.find(view => view.name === v)?.regions.map(r => r.name) ?? [] : [] }), '#0e7490'),
              )}
          </div>
          <button className="context-command" aria-pressed={showImportant} onClick={() => setShowImportant(v => !v)}><Star size={14} />Important ({important.marks.size})</button>
          {selNet && <button className="context-command" onClick={() => { setSelNet(''); setSelectedPin(''); setBundleId('') }}><X size={14} />Clear trace</button>}
          {shareError && <input aria-label="Inspection link" readOnly value={shareError} onFocus={(e) => e.target.select()} style={{ width: '100%' }} />}
          {!!model.subdesigns?.length && (
            <div style={{ marginTop: 6, display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
              <label htmlFor="hierarchy-scope">Hierarchy</label>
              <select id="hierarchy-scope" value={scope ?? 'all'} onChange={(e) => navigateScope(e.target.value === 'all' ? null : e.target.value)}
                style={{ maxWidth: 300, fontSize: 12, background: panelBg, color: panelFg }}>
                <option value="">{model.design}</option>
                {model.subdesigns.map((s) => <option key={s.path} value={s.path}>{s.path}</option>)}
                <option value="all">All parts (flattened)</option>
              </select>
              {chip(model.design, scope === '', () => navigateScope(''), '#7c3aed')}
              {(model.subdesigns ?? []).filter((s) => scope === s.path || scope?.startsWith(`${s.path}::`)).map((s) => (
                chip(shortName(s.path), scope === s.path, () => navigateScope(s.path), '#7c3aed')
              ))}
              {mode === 'layout' && <span style={{ color: '#9ca3af', fontSize: 12 }}>{localPlacementView ? `${shortName(scope ?? '')} local coordinates` : 'Resolved board coordinates'}</span>}
            </div>
          )}
          <div className="inspection-range" aria-label="Inspection range">{selectedGroups.length ? `${selectedGroups.join(' + ')} · ${projection?.represented.size ?? 0} parts` : mode === 'overview' ? 'System index' : 'All groups'}</div>
          {viewsLoading && <div role="status">Loading groups...</div>}
          {viewError && <div role="alert">{viewError} <button className="context-command" onClick={() => setViewRetry(value => value + 1)}>Retry groups</button></div>}
        </div>
        <div className="explorer-workspace">
        {(viewDef || viewCfg?.views.length) && <GroupPanel names={fullView?.regions?.regions ?? []} selected={selectedGroups} collapsed={collapsedGroups} onToggle={toggleGroup} onAll={() => freshRange({ mode: 'sch', scope: null, groups: [], collapsed: [] })} onCollapsed={collapsed => freshRange({ collapsed })} layout={mode === 'layout'} open={groupsOpen} onClose={() => setGroupsOpen(false)} navigation={viewDef?.navigation} groupMembers={groupMembers} onSelection={groups => freshRange({ mode: 'sch', scope: null, groups, collapsed: [] })}>
          <GroupIntentPanel model={model} config={viewCfg} selection={{ schema: 1, instances: candidateConfig ? [...projection?.represented ?? []] : sel.filter(path => model.instances.some(instance => instance.path === path)), groups: selectedGroups.filter(region => viewCfg?.views.some(view => view.name === activeView && view.regions.some(group => group.name === region))).map(region => ({ view: activeView, region })) }}
            onFocus={(config, view, groups) => { setViewError(''); candidateOrigin.current ??= readContext(); candidateTarget.current = { mode: 'sch', scope: null, view, groups, collapsed: [], selection: [], pin: '', net: '', viewport: null, boardViewport: null }; setCandidateConfig(config) }}
            onCancel={() => { candidateTarget.current = candidateOrigin.current; candidateOrigin.current = null; setCandidateConfig(null) }}
            onSaved={operation => {
              if (operation === 'undo') {
                const definition = viewCfg?.views.find(view => view.name === 'Functional') ?? viewCfg?.views[0]
                candidateTarget.current = candidateOrigin.current ? { ...candidateOrigin.current, viewport: null, boardViewport: null } : { mode: 'sch', scope: null, view: definition?.name ?? '', groups: defaultInsideGroups(model, definition, sourceValues), collapsed: [], selection: [], pin: '', net: '', viewport: null, boardViewport: null }
                candidateOrigin.current = null
              }
              setViewRetry(value => value + 1)
            }} />
        </GroupPanel>}
        {groupsOpen && <button className="group-drawer-backdrop" aria-label="Close groups overlay" onClick={() => setGroupsOpen(false)} />}
        <div className="explorer-canvas" ref={canvasRef} aria-busy={layoutBusy || documents.loading}>
        {mode === 'layout' ? <BoardLayout key={JSON.stringify([scope, layoutFrame, selectedGroups])} model={model} scope={scope} frame={layoutFrame} onFrame={(frame) => navigation.navigate({ layoutFrame: frame, boardViewport: null })} selected={sel} selectedNet={selNet} focus={jump} cameraRef={boardCamera} restore={boardRestore} visiblePaths={selectedGroups.length ? projection?.represented : undefined} placements={physicalPlacements} side={layoutSide} onSide={setLayoutSide} onCamera={navigation.sync}
          onScope={(path) => { setLayoutFrame('local'); navigateScope(path) }}
          onSelect={(paths) => { setSel(paths); setSelNet(''); setJump(null) }} onNet={locateNet} /> : <ReactFlow
          nodes={nodes}
          edges={edges}
          nodeTypes={nodeTypes}
          edgeTypes={edgeTypes}
          onInit={(i) => { rf.current = i; onReady() }}
          onMoveStart={onMoveStart}
          onMoveEnd={navigation.sync}
          onNodesChange={onNodesChange}
          onEdgesChange={onEdgesChange}
          onNodeClick={(_, node) => {
            if (node.id.startsWith('bundle:')) { setSelectionTarget(null); openBundle(node.id); return }
            setSelNet(''); setSelectedPin(''); setBundleId('')
          }}
          onNodeDoubleClick={(_, n) => {
            const subdesign = (n.data.g as GNode | undefined)?.subdesign
            if (subdesign) navigateScope(subdesign.path)
          }}
          onEdgeClick={(_, e) => {
            if (e.data?.bundleId) { openBundle(String(e.data.bundleId)); return }
            if (!(e.data?.dashed as boolean)) {
              setSel([])
              setSelNet((e.data?.net as string) ?? '')
            }
          }}
          onPaneClick={() => {
            setSelectionTarget(null)
            setSelNet('')
          }}
          minZoom={0.002}
          selectionOnDrag
          panOnDrag={[1, 2]}
          panOnScroll
          zoomOnScroll={false}
          zoomOnPinch
          colorMode={dark ? 'dark' : 'light'}
          proOptions={{ hideAttribution: true }}
        >
          <Background color={dark ? '#2a2f3a' : undefined} />
          <Controls onZoomIn={manual} onZoomOut={manual} onFitView={fit} />
        </ReactFlow>}
        {mode !== 'layout' && (layoutBusy || documents.loading || err || !graph?.nodes.length) && <div className="canvas-state" role={err ? 'alert' : 'status'}>{err || (documents.loading ? 'Loading component information...' : layoutBusy ? 'Arranging circuit...' : 'No parts in this selection')}
          {!layoutBusy && !documents.loading && !err && !graph?.nodes.length && <button className="empty-recovery" onClick={clearFilters}>Clear filters</button>}
        </div>}
        </div>
        </div>
      </main>
      {tab === 'bom' && <div className="bom-panel"><InspectionLists model={model} tab="bom" query={bomQuery} onQuery={setBomQuery} selected={selInst?.path} sourceValues={sourceValues} onLocate={hit => locateTarget(hit, 'bom')} onIssue={() => {}} /></div>}
      {(showImportant || bundleId || tab !== 'circuit' || selInst || selNetObj || aggInsts.length > 0 || inspectedSubdesign) && (
        <div
          className="explorer-inspector"
          style={{
            borderLeft: `1px solid ${dark ? '#2a2f3a' : '#e5e7eb'}`,
            padding: 14,
            overflow: 'auto',
            fontSize: 12,
            background: panelBg,
            color: panelFg,
          }}
        >
          {!showImportant && !bundleId && <button className="icon-tool inspector-close" aria-label="Close details" title="Close details" onClick={() => { setSelectionTarget(null); setSel([]); setSelNet(''); setBundleId(''); setShowImportant(false); if (tab !== 'bom') setTab('circuit') }}><X size={16} /></button>}
          {showImportant && <section className="important-list" aria-label="Important parts"><div className="part-heading"><h3>Important</h3><button className="icon-tool" aria-label="Close important parts" onClick={() => setShowImportant(false)}><X size={16} /></button></div>
            {!important.marks.size && <p>No important parts</p>}
            {[...important.marks].map(path => { const instance = model.instances.find(i => i.path === path); if (!instance) return null; const identity = partPresentation(instance, sourceValues.get(path)); return <div className="important-item" key={path}><button className="locate-link" onClick={() => locateTarget({ kind: 'part', id: path, label: identity.title, sub: path })}>{identity.title}{identity.subtitle && ` · ${identity.subtitle}`}</button><button className="icon-tool" aria-label={`Unmark ${instance.designator ?? 'component'}`} onClick={() => important.toggle(path)}><Star size={14} fill="currentColor" /></button></div> })}
            <h4>Suggested core components</h4>
            {candidates.filter(c => !important.marks.has(c.instance.path)).map(c => { const identity = partPresentation(c.instance, sourceValues.get(c.instance.path)); return <div className="core-candidate" key={c.instance.path}><button className="locate-link" onClick={() => locateTarget({ kind: 'part', id: c.instance.path, label: identity.title, sub: c.instance.path })}>{identity.title}{identity.subtitle && ` · ${identity.subtitle}`}</button><small>Heuristic suggestion · {c.evidence}</small><button className="context-command" onClick={() => important.toggle(c.instance.path)}><Star size={14} />Confirm important</button></div> })}
          </section>}
          {bundleId && bundled?.bundles.get(bundleId) && <section className="bundle-inspector" aria-label="Connection bundle"><div className="part-heading"><h3>{bundled.bundles.get(bundleId)!.nets.length} {bundled.bundles.get(bundleId)!.nets.length === 1 ? 'network' : 'networks'}</h3><button className="icon-tool" aria-label="Close connection bundle" onClick={() => setBundleId('')}><X size={16} /></button></div>
            {bundled.bundles.get(bundleId)!.nets.map(net => <button key={net.name} className="bundle-network" onClick={() => { setSel([]); setSelectedPin(''); setSelNet(net.name) }}><strong>{net.name}</strong><span>{net.members.length} physical endpoints · {net.members.filter(m => !projection?.represented.has(m.instance_path)).length} outside range</span></button>)}
          </section>}
          {tab === 'proposals' ? <ProposalReview resource={proposals} documents={documents} onBack={() => navigation.navigate({ tab: 'circuit' })} /> : tab === 'issues' ? <>{issueFocus && <div role="status" ref={issueFocusRef} className="issue-source-focus" tabIndex={-1}>{issueFocus}</div>}<InspectionLists model={model} tab={tab} onLocate={locateTarget} onIssue={(issue) => {
            const target = issue.span && model.instances.find((i) => i.span.file === issue.span!.file && i.span.line === issue.span!.line && i.span.col === issue.span!.col)
            if (target) locateTarget({ kind: 'part', id: target.path, label: target.designator ?? target.path, sub: target.path })
            else {
              const net = issue.span && allNets.find((n) => n.span?.file === issue.span!.file && n.span.line === issue.span!.line && n.span.col === issue.span!.col)
              if (net) { locateNet(net.name); return }
              const subdesign = issue.span && model.subdesigns?.find((s) => s.span.file === issue.span!.file && s.span.line === issue.span!.line && s.span.col === issue.span!.col)
              if (subdesign) navigateScope(subdesign.path)
              else setIssueFocus(`${issue.span?.file}:${issue.span?.line}:${issue.span?.col} · ${issue.message} · No component is declared at this source location`)
            }
          }} /></> : <>
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
              <div style={{ color: '#6b7280', marginBottom: 6 }}>{aggInsts.length} represented parts</div>
              {evidenceGroups.length > 0 && <button className="context-command view-inside" onClick={() => freshRange({ mode: 'sch', scope: null, groups: evidenceGroups, collapsed: collapsedGroups.filter(g => !evidenceGroups.includes(g)) })}><ChevronRight size={16} />View inside</button>}
              {!!selNode.ports?.length && <table className="boundary-pin-list" style={{ width: '100%', borderCollapse: 'collapse', marginBottom: 10 }}>
                <thead><tr><th>Part / pin</th><th>#</th><th>Boundary net</th></tr></thead>
                <tbody>{selNode.ports.map((port) => <tr key={port.name} data-boundary-port={port.name}>
                  <td>{port.member && <button className="locate-link" title={port.displayName ?? port.member.logical_pin} onClick={() => locateTarget({ kind: 'pin', id: port.member!.instance_path, pin: port.member!.logical_pin, net: port.net, label: port.member!.logical_pin, sub: '' })}>
                    {port.displayName ?? `${model.instances.find((i) => i.path === port.member!.instance_path)?.designator || 'Component'}.${port.member.logical_pin}`}</button>}</td>
                  <td>{port.member?.numbers.join(', ')}</td><td><button className="locate-link" onClick={() => port.net && locateNet(port.net)}>{port.net}</button></td>
                </tr>)}</tbody>
              </table>}
              <div style={{ display: 'flex', gap: 4, flexWrap: 'wrap', marginBottom: 8 }}>
                {selNode.railTags.map((r) => (
                  <span
                    key={r}
                    onClick={() => locateNet(r)}
                    style={{ background: railColor(r), color: '#fff', borderRadius: 3, padding: '1px 5px', fontSize: 12, cursor: 'pointer' }}
                  >
                    {r}
                  </span>
                ))}
              </div>
              <div className="module-members">{aggInsts.map(i => { const identity = partPresentation(i, sourceValues.get(i.path)); const candidate = candidates.find(c => c.instance.path === i.path); return <div className="module-member" key={i.path}>
                <button className="locate-link" title={identity.title} onClick={() => locateTarget({ kind: 'part', id: i.path, label: identity.title, sub: identity.subtitle })}>{identity.title}</button><span>{identity.subtitle || 'MPN unspecified'}</span>
                {candidate && <small>{important.marks.has(i.path) ? 'Confirmed important' : 'Heuristic core suggestion'} · {candidate.evidence}</small>}
                <button className="icon-tool" title={important.marks.has(i.path) ? 'Unmark important' : 'Mark important'} aria-label={`${important.marks.has(i.path) ? 'Unmark' : 'Mark'} ${identity.reference} important`} onClick={() => important.toggle(i.path)}><Star size={14} fill={important.marks.has(i.path) ? 'currentColor' : 'none'} /></button>
              </div> })}</div>
              {evidenceGroups.length > 0 && <ProvenanceInspector resource={documents} view={activeView} groups={evidenceGroups} />}
              <div style={{ color: '#6b7280', marginTop: 8, fontSize: 12 }}>
                source {aggInsts[0].span.file}:{aggInsts[0].span.line}
              </div>
            </>
          )}
          {selInst && <PartDetails model={model} instance={selInst} dark={dark} resource={documents} view={activeView} groups={selectedGroups} connectionMode={mode === 'layout' ? 'individual' : connectionMode}
            visible={visibleParts} locations={mode === 'layout' ? undefined : projection?.graph.location} selectedPin={selectedPin} selectedNet={selNet} marked={important.marks.has(selInst.path)}
            instanceRegions={fullView?.instanceRegions} onGroup={group => navigation.navigate({ mode: 'sch', scope: null, groups: [group], collapsed: [], selection: [], pin: '', net: selNet, tab, viewport: null, boardViewport: null })} onEndpoint={followEndpoint}
            onMark={() => important.toggle(selInst.path)} onPin={activatePin} placement={<div className="part-placement">
              <b>{localPlacementView ? 'Local placement' : 'Board placement'}</b>
              {selectedPlacement ? <div>X {selectedPlacement.at_mm[0]} mm · Y {selectedPlacement.at_mm[1]} mm · {selectedPlacement.rotate}° · {selectedPlacement.side}</div> : <div>No resolved placement</div>}
            </div>} />}
          {selNetObj && !selInst && (
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
              <OutsideDestinations model={model} net={selNetObj} instanceRegions={fullView?.instanceRegions ?? new Map()} visible={visibleParts} sourceValues={sourceValues}
                onGroup={group => navigation.navigate({ mode: 'sch', scope: null, groups: [group], collapsed: [], selection: [], pin: '', net: selNetObj.name, viewport: null, boardViewport: null })} onEndpoint={followEndpoint} />
              {(model.subdesigns ?? []).filter((s) => s.ports.some((p) => p.net === selNet)).map((s) => (
                <div key={s.path} style={{ marginTop: 6 }}>
                  <button onClick={() => navigateScope(s.path, { net: selNet })}>{s.path} ↗</button>
                  <span style={{ marginLeft: 4 }}>{s.ports.filter((p) => p.net === selNet).map((p) => p.name).join(', ')}</span>
                </div>
              ))}
            </>
          )}
          </>}
        </div>
      )}
    </div>
  )
}
