import type { ExplorerModel, Pin } from './model'
import type { InspectionContext } from './inspection'

export function pinConnection(model: ExplorerModel, instance: string, pin: Pin, visible: ReadonlySet<string>, trace: string, locations?: ReadonlyMap<string, string>, connectionMode: InspectionContext['connectionMode'] = 'bundled') {
  const net = model.nets.find(n => n.members.some(m => m.instance_path === instance && m.logical_pin === pin.logical))
  if (!net) return { net: undefined, state: pin.nc ? 'NC' : pin.connected ? 'membership unavailable' : 'open', hidden: 0 }
  const hidden = net.members.filter(m => !visible.has(m.instance_path)).length
  const objects = new Set(net.members.filter(m => visible.has(m.instance_path)).map(m => locations?.get(m.instance_path) ?? m.instance_path))
  const state = trace === net.name ? 'exact trace' : model.derived.rails.includes(net.name) ? 'simplified rail'
    : net.members.length === 1 ? 'single endpoint' : hidden ? 'outside range' : objects.size < 2 ? 'internal connection' : `${connectionMode} connection`
  return { net, state, hidden }
}
