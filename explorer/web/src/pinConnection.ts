import type { ExplorerModel, Pin } from './model'

export function pinConnection(model: ExplorerModel, instance: string, pin: Pin, visible: ReadonlySet<string>, trace: string) {
  const net = model.nets.find(n => n.members.some(m => m.instance_path === instance && m.logical_pin === pin.logical))
  if (!net) return { net: undefined, state: pin.nc ? 'NC' : pin.connected ? 'membership unavailable' : 'open', hidden: 0 }
  const hidden = net.members.filter(m => !visible.has(m.instance_path)).length
  const state = trace === net.name ? 'exact trace' : model.derived.rails.includes(net.name) ? 'simplified rail'
    : net.members.length === 1 ? 'single endpoint' : hidden ? 'outside range' : 'bundled connection'
  return { net, state, hidden }
}
