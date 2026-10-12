import type { Diag, ExplorerModel } from './model'
import { shortName } from './model'
import { explorerNets } from './hierarchy'
import { partPresentation } from './partPresentation'

export interface SearchHit { kind: 'part' | 'net' | 'pin' | 'subdesign'; id: string; label: string; sub: string; pin?: string; net?: string }

export function searchCircuit(model: ExplorerModel, query: string, sourceValues: ReadonlyMap<string, string> = new Map()): SearchHit[] {
  const terms = query.trim().toLowerCase().split(/[\s.]+/).filter(Boolean)
  if (!terms.length) return []
  const matches = (text: string) => terms.every((term) => text.toLowerCase().includes(term))
  const identityOf = (instance: ExplorerModel['instances'][number]) => `${instance.designator ?? ''} ${instance.path} ${instance.device_fq} ${partPresentation(instance, sourceValues.get(instance.path)).subtitle}`.toLowerCase()
  const target = terms.at(-1) ?? ''
  const prefix = terms.slice(0, -1)
  const qualified = terms.length > 1 && model.instances.some((i) => prefix.every((t) => identityOf(i).includes(t)) &&
    ([i.designator?.toLowerCase(), i.path.toLowerCase()].includes(prefix.join(' ')) || /^[0-9]+$/.test(target) || i.pins.some((p) => p.logical.toLowerCase().includes(target) || p.numbers.some((n) => n.toLowerCase() === target))))
  const hits: SearchHit[] = []
  for (const sub of model.subdesigns ?? [])
    if (matches(`${sub.path} ${sub.definition}`)) hits.push({ kind: 'subdesign', id: sub.path, label: shortName(sub.path), sub: sub.path })
  for (const instance of model.instances) {
    const identity = identityOf(instance)
    const presentation = partPresentation(instance, sourceValues.get(instance.path))
    if (!qualified && matches(identity)) hits.push({ kind: 'part', id: instance.path, label: `${presentation.reference} ${presentation.subtitle || presentation.category}`, sub: instance.path })
    for (const pin of instance.pins) {
      if (qualified) {
        const target = terms.at(-1)!
        if (!terms.slice(0, -1).every((t) => identity.includes(t)) || !(pin.numbers.some((n) => n.toLowerCase() === target) || (!/^[0-9]+$/.test(target) && pin.logical.toLowerCase().includes(target)))) continue
      } else if (!matches(`${identity} ${pin.logical} ${pin.numbers.join(' ')}`)) continue
      const net = model.nets.find((n) => n.members.some((m) => m.instance_path === instance.path && m.logical_pin === pin.logical))?.name
      hits.push({ kind: 'pin', id: instance.path, pin: pin.logical, net,
        label: `${instance.designator ?? shortName(instance.path)}.${pin.logical} (${pin.numbers.join(', ')})`, sub: net ?? (pin.nc ? 'NC' : 'Unconnected') })
    }
  }
  for (const net of explorerNets(model)) if (matches(net.name)) hits.push({ kind: 'net', id: net.name, net: net.name, label: net.name, sub: `${net.members.length} pins` })
  const q = query.trim().toLowerCase()
  return hits.sort((a, b) => Number(!a.label.toLowerCase().startsWith(q)) - Number(!b.label.toLowerCase().startsWith(q)))
}

export function inspectionIssues(model: ExplorerModel): Diag[] {
  return [...(model.live_error ? [{ code: 'LIVE_SOURCE', severity: 'error', message: model.live_error }] : []), ...model.diagnostics]
}
