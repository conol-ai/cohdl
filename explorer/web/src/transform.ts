// Display-rule engine (spec R1-R4): ExplorerModel -> graph nodes/edges.
//
// R1  Two-terminal parts do not occupy layout nodes: series parts become
//     edge relays; rail-to-rail parts aggregate into per-rail-combo nodes.
// R2  Rail nets (GND / voltage-annotated / high fan-out) render as stub tags
//     on each node instead of drawn wires.
// R3  Non-rail nets with >3 endpoints render as net-label badges, not wires.
// R4  Only connected pins count toward node size; unused pins collapse.

import type { ExplorerModel, Instance } from './model'
import { shortName } from './model'

export interface GNode {
  id: string
  kind: 'ic' | 'agg' | 'passive' | 'net'
  title: string
  sub: string
  railTags: string[]
  netLabels: string[]
  decors: string[]
  pinsConnected: number
  pinsTotal: number
  /** logical pin -> net name (connected pins only). */
  pinNets: Record<string, string>
  inst?: Instance
  aggMembers?: string[]
  width: number
  height: number
}

export interface GEdge {
  id: string
  source: string
  target: string
  net: string
  nets: string[]
  label: string
  relay?: string // series two-terminal instance path riding this edge
  sourcePin?: string
  targetPin?: string
  /** dashed attachment edge (bypass group -> host IC), not a signal wire. */
  dashed?: boolean
}

export interface Graph {
  nodes: GNode[]
  edges: GEdge[]
  railSet: Set<string>
  /** instance path -> node id that represents it (self, or aggregate). */
  location: Map<string, string>
}

const spec = (i: Instance, name: string): string | undefined =>
  i.specs.find((s) => s.name === name)?.value

export function buildGraph(m: ExplorerModel): Graph {
  const railSet = new Set(m.derived.rails)
  const twoT = new Set(m.derived.two_terminal)
  const inst = new Map(m.instances.map((i) => [i.path, i]))

  // Nets touching each instance (connected pins only).
  const instNets = new Map<string, string[]>()
  const pinNet = new Map<string, Record<string, string>>()
  for (const n of m.nets)
    for (const mem of n.members) {
      const l = instNets.get(mem.instance_path) ?? []
      if (!l.includes(n.name)) l.push(n.name)
      instNets.set(mem.instance_path, l)
      const pn = pinNet.get(mem.instance_path) ?? {}
      pn[mem.logical_pin] = n.name
      pinNet.set(mem.instance_path, pn)
    }

  const nodes: GNode[] = []
  const edges: GEdge[] = []
  const location = new Map<string, string>()
  const aggs = new Map<string, string[]>() // group key -> member paths
  const bypassOf = new Map(m.derived.bypasses.map((b) => [b.cap, b.target]))

  // ---- classify two-terminal instances (R1, unified): every 2T part with a
  // non-rail end renders as ONE compact mini node, always wired — series
  // parts sit mid-wire with an edge per side, rail taps hang off one wire.
  // Only all-rail 2T parts (decoupling/pulls) aggregate into group boxes.
  for (const p of twoT) {
    const nets = instNets.get(p) ?? []
    const nonRail = nets.filter((n) => !railSet.has(n))
    if (nonRail.length === 0 && nets.length > 0) {
      // Host-anchored group when a #[bypass] fact names the target IC;
      // otherwise fall back to the rail-combination bucket.
      const key = bypassOf.has(p)
        ? `host:${bypassOf.get(p)}`
        : [...nets].sort().join(' + ')
      aggs.set(key, [...(aggs.get(key) ?? []), p])
    }
  }

  // ---- nodes
  const mkNode = (i: Instance, kind: GNode['kind']): GNode => {
    const nets = instNets.get(i.path) ?? []
    const rails = nets.filter((n) => railSet.has(n))
    const connected = i.pins.filter((p) => p.connected).length
    const val = spec(i, 'resistance') ?? spec(i, 'capacitance') ?? spec(i, 'inductance')
    // Minis show "R16 100kohm" — one uniform compact style for every
    // passive; the full device/MPN lives in the sidebar.
    const title =
      kind === 'passive'
        ? `${i.designator ?? ''} ${val ?? shortName(i.device_fq)}`.trim()
        : `${i.designator ?? ''} ${shortName(i.device_fq)}`.trim()
    const subBits = kind === 'passive' ? [] : [i.part?.mpn, val].filter(Boolean)
    const h = kind === 'ic' ? Math.max(54, 38 + Math.min(connected, 12) * 5) : kind === 'passive' ? 26 : 42
    const w =
      kind === 'ic'
        ? Math.max(150, title.length * 7 + 26)
        : kind === 'passive'
          ? Math.max(64, title.length * 6 + 18)
          : 118
    return {
      id: i.path,
      kind,
      title,
      sub: subBits.join(' · '),
      railTags: rails,
      netLabels: [],
      decors: [],
      pinsConnected: connected,
      pinsTotal: i.pins.length,
      pinNets: pinNet.get(i.path) ?? {},
      inst: i,
      width: w,
      height: h,
    }
  }

  for (const i of m.instances) {
    const aggKey = [...aggs].find(([, mem]) => mem.includes(i.path))?.[0]
    if (aggKey) continue // represented by the aggregate node
    const kind: GNode['kind'] = twoT.has(i.path) ? 'passive' : 'ic'
    const n = mkNode(i, kind)
    nodes.push(n)
    location.set(i.path, n.id)
  }
  const hostEdges: { agg: string; host: string }[] = []
  for (const [key, members] of aggs) {
    const id = `agg:${key}`
    const labels = members.map((p) => {
      const i = inst.get(p)!
      const v = spec(i, 'capacitance') ?? spec(i, 'resistance') ?? ''
      return `${i.designator ?? shortName(i.device_fq)} ${v}`.trim()
    })
    const isHost = key.startsWith('host:')
    const hostPath = isHost ? key.slice(5) : undefined
    const hostInst = hostPath ? inst.get(hostPath) : undefined
    const rails = [
      ...new Set(members.flatMap((p) => instNets.get(p) ?? [])),
    ].filter((n) => railSet.has(n))
    nodes.push({
      id,
      kind: 'agg',
      title: isHost
        ? `${hostInst?.designator ?? shortName(hostPath ?? '')} decoupling`
        : key,
      sub: isHost ? `bypass ×${members.length}` : `bypass/pull ×${members.length}`,
      railTags: rails,
      netLabels: [],
      decors: labels,
      pinsConnected: 0,
      pinsTotal: 0,
      pinNets: {},
      aggMembers: members,
      width: 150,
      height: Math.min(130, 50 + Math.min(labels.length, 6) * 12),
    })
    for (const p of members) location.set(p, id)
    if (hostPath) hostEdges.push({ agg: id, host: hostPath })
  }

  // ---- edges (non-rail nets)
  const nodeIds = new Set(nodes.map((n) => n.id))
  const edgeSeen = new Set<string>()
  const pinFor = (nodeId: string, netName: string): string | undefined => {
    const node = nodes.find((x) => x.id === nodeId)
    if (!node) return undefined
    for (const [p, nn] of Object.entries(node.pinNets)) if (nn === netName) return p
    return undefined
  }
  const pushEdge = (a: string, b: string, net: string, label: string, relay?: string) => {
    if (!nodeIds.has(a) || !nodeIds.has(b) || a === b) return
    const key = `${[a, b].sort().join('|')}|${net}`
    if (edgeSeen.has(key)) return
    edgeSeen.add(key)
    const netA = net.includes('\u21c4') ? net.split('\u21c4')[0] : net
    const netB = net.includes('\u21c4') ? net.split('\u21c4')[1] : net
    edges.push({
      id: `e${edges.length}`, source: a, target: b, net, nets: [net], label, relay,
      sourcePin: pinFor(a, netA), targetPin: pinFor(b, netB),
    })
  }

  for (const n of m.nets) {
    if (railSet.has(n.name)) continue
    const eps: string[] = []
    for (const mem of n.members) {
      const loc = location.get(mem.instance_path)
      if (loc && loc !== '(edge)' && !eps.includes(loc)) eps.push(loc)
    }
    if (eps.length <= 1) continue
    if (eps.length > 3) {
      // R3: star junction — a small net node every endpoint wires to, so
      // every part shows a real connection (EDA net-tie convention).
      const jid = `net:${n.name}`
      if (!nodeIds.has(jid)) {
        nodes.push({
          id: jid, kind: 'net', title: n.name, sub: '', railTags: [],
          netLabels: [], decors: [], pinsConnected: 0, pinsTotal: 0,
          pinNets: {}, width: Math.max(54, n.name.length * 7 + 14), height: 22,
        })
        nodeIds.add(jid)
      }
      for (const id of eps) pushEdge(id, jid, n.name, '')
      continue
    }
    for (let k = 0; k + 1 < eps.length; k++) pushEdge(eps[k], eps[k + 1], n.name, n.name)
  }

  for (const { agg, host } of hostEdges) {
    const hostNode = location.get(host)
    if (hostNode && hostNode !== '(edge)')
      edges.push({
        id: `e${edges.length}`,
        source: agg,
        target: hostNode,
        net: `bypass:${host}`,
        nets: [],
        label: '',
        dashed: true,
      })
  }
  return { nodes, edges, railSet, location }
}

/** Focus (spec I1): the conductive subgraph between selected node ids. */
export function focusSubset(g: Graph, selected: string[]): Set<string> {
  if (selected.length < 2) return new Set()
  const adj = new Map<string, { to: string; edge: GEdge }[]>()
  for (const e of g.edges) {
    adj.set(e.source, [...(adj.get(e.source) ?? []), { to: e.target, edge: e }])
    adj.set(e.target, [...(adj.get(e.target) ?? []), { to: e.source, edge: e }])
  }
  const keep = new Set<string>()
  // BFS between every selected pair; keep all nodes on any shortest path.
  for (let i = 0; i < selected.length; i++)
    for (let j = i + 1; j < selected.length; j++) {
      const [src, dst] = [selected[i], selected[j]]
      const prev = new Map<string, string | null>([[src, null]])
      const q = [src]
      while (q.length) {
        const cur = q.shift()!
        if (cur === dst) break
        for (const { to } of adj.get(cur) ?? [])
          if (!prev.has(to)) {
            prev.set(to, cur)
            q.push(to)
          }
      }
      if (prev.has(dst)) {
        let cur: string | null = dst
        while (cur) {
          keep.add(cur)
          cur = prev.get(cur) ?? null
        }
      }
    }
  return keep
}
