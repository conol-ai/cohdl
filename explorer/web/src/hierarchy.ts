// Scope is a projection of checked connectivity. Never infer a public port
// from a descendant's physical pin: only the compiler's port mapping crosses
// a subdesign boundary. null is the explicit flattened "All parts" view.
import type { ExplorerModel, Net, Subdesign, SubdesignPort } from './model'
import { shortName } from './model'

export interface Boundary {
  path: string
  kind: 'subdesign' | 'port'
  title: string
  sub: string
  ports: SubdesignPort[]
  subdesign?: Subdesign
}

export function ownerScope(m: ExplorerModel, path: string): string {
  return enclosingScope(path, new Set((m.subdesigns ?? []).map((s) => s.path)))
}

function enclosingScope(path: string, scopes: ReadonlySet<string>): string {
  for (let end = path.lastIndexOf('::'); end >= 0; end = path.lastIndexOf('::', end - 1)) {
    const parent = path.slice(0, end)
    if (scopes.has(parent)) return parent
  }
  return ''
}

/** Physical nets plus port-only classes, for global net inspection/search. */
export function explorerNets(m: ExplorerModel): Net[] {
  const nets = new Map(m.nets.map((n) => [n.name, n]))
  for (const s of m.subdesigns ?? [])
    for (const p of s.ports)
      if (p.net && !nets.has(p.net))
        nets.set(p.net, { name: p.net, is_gnd: false, members: [], span: s.span })
  return [...nets.values()]
}

export function projectScope(m: ExplorerModel, scope: string | null) {
  const subs = m.subdesigns ?? []
  if (scope === null || subs.length === 0) return { model: m, boundaries: [] as Boundary[] }
  const current = subs.find((s) => s.path === scope)
  const children = subs.filter((s) => (s.parent ?? '') === scope)
  const scopes = new Set(subs.map((s) => s.path))
  const instances = m.instances.filter((i) => enclosingScope(i.path, scopes) === scope)
  const visible = new Set(instances.map((i) => i.path))
  const boundaries: Boundary[] = children.map((s) => ({
    path: s.path, kind: 'subdesign', title: shortName(s.path),
    sub: shortName(s.definition), subdesign: s,
    // An internal-only optional net is not an external connection.
    ports: s.ports.map((p) => ({ ...p, net: p.connected ? p.net : undefined })),
  }))
  for (const p of current?.ports ?? []) {
    boundaries.push({
      path: `port:${current!.path}:${p.name}`, kind: 'port', title: p.name,
      sub: `${p.obligation} Pin · ${p.connected ? 'connected outside' : 'open outside'}`,
      ports: [p],
    })
  }
  const nets = new Map<string, Net>(m.nets.map((n) => [n.name, {
    ...n, members: n.members.filter((mem) => visible.has(mem.instance_path)),
  }]))
  for (const b of boundaries) {
    for (const p of b.ports) {
      if (!p.net) continue
      // A port-only equivalence class has no manufacturing net, but is
      // still a real logical connection in the hierarchy.
      const net = nets.get(p.net) ?? {
        name: p.net, is_gnd: false, members: [],
        span: (b.subdesign ?? current)!.span,
      }
      net.members.push({ instance_path: b.path, logical_pin: p.name, numbers: [] })
      nets.set(p.net, net)
    }
  }
  return {
    model: { ...m, instances, nets: [...nets.values()].filter((n) => n.members.length > 0) },
    boundaries,
  }
}
