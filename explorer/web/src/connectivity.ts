import type { ExplorerModel, Net } from './model'
import type { Graph, GNode } from './transform'
import { explorerNets } from './hierarchy'

export interface ConnectionBundle {
  id: string
  objects: string[]
  nets: Net[]
}

export const bundleLabelSize = (label: string) => ({ width: label.length * 12 + 16, height: 32, fontSize: 20 })

/** Neutral display projection; the input graph remains the exact trace authority. */
export function bundleConnections(graph: Graph, model: ExplorerModel): {
  graph: Graph
  bundles: Map<string, ConnectionBundle>
} {
  const nodes = graph.nodes.filter((node) => node.kind !== 'net')
  const visible = new Set(nodes.map((node) => node.id))
  const edges = graph.edges.filter((edge) => edge.nets.length === 0 && visible.has(edge.source) && visible.has(edge.target))
  const grouped = new Map<string, ConnectionBundle>()
  const seenNets = new Set<string>()
  for (const net of explorerNets(model)) {
    if (graph.railSet.has(net.name) || net.is_gnd || seenNets.has(net.name)) continue
    seenNets.add(net.name)
    const objects = visibleObjects(net, graph, nodes, visible)
    if (objects.length < 2) continue
    const id = `bundle:${encodeURIComponent(JSON.stringify(objects))}`
    const bundle = grouped.get(id) ?? { id, objects, nets: [] }
    bundle.nets.push(net)
    grouped.set(id, bundle)
  }
  const bundles = new Map<string, ConnectionBundle>()
  for (const bundle of grouped.values()) {
    const { id, objects, nets } = bundle
    const label = `${nets.length} ${nets.length === 1 ? 'net' : 'nets'}`
    const names = nets.map((net) => net.name)
    bundles.set(id, bundle)
    if (objects.length === 2) {
      edges.push({ id, source: objects[0], target: objects[1], net: names[0], nets: names, label })
      continue
    }
    nodes.push({
      id, kind: 'net', title: label, sub: '', railTags: [], netLabels: [], decors: [],
      pinsConnected: 0, pinsTotal: 0, pinNets: {}, width: bundleLabelSize(label).width, height: bundleLabelSize(label).height,
    })
    for (const [index, object] of objects.entries()) {
      const spoke = `${id}:${index}`
      edges.push({ id: spoke, source: object, target: id, net: names[0], nets: names, label: '' })
      bundles.set(spoke, bundle)
    }
  }
  return { graph: { ...graph, nodes, edges }, bundles }
}

function visibleObjects(net: Net, graph: Graph, nodes: GNode[], visible: Set<string>): string[] {
  const objects = new Set<string>()
  for (const member of net.members) {
    const object = graph.location.get(member.instance_path)
    if (object && visible.has(object)) objects.add(object)
  }
  // Only checked visible boundary ports represent hidden hierarchy endpoints.
  for (const node of nodes)
    if (node.ports?.some((port) => port.net === net.name)) objects.add(node.id)
  return [...objects].sort()
}
