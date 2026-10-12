import type { ExplorerModel } from './model'
import { coreCandidates, partPresentation } from './partPresentation'

export interface GroupIndexEntry {
  name: string
  parts: number
  cores: { path: string; label: string }[]
  interfaces: string[]
  rails: string[]
}

export function groupIndex(model: ExplorerModel, instanceRegions: ReadonlyMap<string, string>, sourceValues: ReadonlyMap<string, string> = new Map()): Map<string, GroupIndexEntry> {
  const groups = new Map<string, GroupIndexEntry>()
  for (const instance of model.instances) {
    const name = instanceRegions.get(instance.path) ?? 'Ungrouped'
    const entry = groups.get(name) ?? { name, parts: 0, cores: [], interfaces: [], rails: [] }
    entry.parts++
    groups.set(name, entry)
  }
  for (const candidate of coreCandidates(model, sourceValues)) {
    const identity = partPresentation(candidate.instance, sourceValues.get(candidate.instance.path))
    groups.get(instanceRegions.get(candidate.instance.path) ?? 'Ungrouped')!.cores.push({ path: candidate.instance.path, label: `${identity.reference} ${identity.subtitle || identity.category}` })
  }
  const rails = new Set(model.derived.rails)
  for (const net of model.nets) {
    const destinations = new Set(net.members.map(member => instanceRegions.get(member.instance_path) ?? 'Ungrouped'))
    if (destinations.size < 2) continue
    for (const name of destinations) {
      const entry = groups.get(name)
      if (!entry) continue
      const names = net.is_gnd || rails.has(net.name) ? entry.rails : entry.interfaces
      if (!names.includes(net.name)) names.push(net.name)
    }
  }
  return groups
}
