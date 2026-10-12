import type { ExplorerModel, Instance } from './model'
import type { DocumentRegistry } from './documents'

function identityCategory(identity: string): string | undefined {
  if (/^(STM32|SF32LB|ESP32|RP20|RP23|CH32)/i.test(identity)) return 'Microcontroller'
  if (/^(W25Q|GD25)/i.test(identity)) return 'Memory'
  if (/^(TPD[1246]E|ESD[0-9])/i.test(identity)) return 'ESD protection'
  if (/^1N4148/i.test(identity)) return 'Diode'
  if (/^AO340[01]/i.test(identity)) return 'MOSFET'
  if (/^S8050/i.test(identity)) return 'Transistor'
  if (/^(TPS|RT61)/i.test(identity)) return 'Power IC'
  return undefined
}

export function partPresentation(instance: Instance, sourceValue = '') {
  const electrical = [
    ['resistance', 'Resistor'], ['capacitance', 'Capacitor'], ['inductance', 'Inductor'],
  ] as const
  const declared = electrical.find(([name]) => instance.specs.some(s => s.name === name))
  const trait = electrical.find(([, category]) => instance.impl_traits.some(t => t.split('::').at(-1) === category))
  const declaredCategory = instance.impl_traits.map(t => t.split('::').at(-1)).find(t => ['IC', 'Microcontroller', 'Memory', 'MotorDriver', 'Amplifier', 'Diode', 'MOSFET', 'Transistor', 'Connector', 'Crystal'].includes(t ?? ''))
  const category = declared?.[1] ?? trait?.[1] ?? declaredCategory ?? identityCategory(instance.part?.mpn ?? '') ?? identityCategory(sourceValue) ?? 'Component'
  const value = instance.specs.find(s => s.name === declared?.[0])?.value ?? instance.specs.find(s => s.name === 'value')?.value ?? sourceValue
  const reference = instance.designator?.trim() && !/^component_\d+$/i.test(instance.designator) ? instance.designator : 'Component'
  const mpn = instance.part?.mpn ?? ''
  return { reference, category, value, mpn, title: `${reference} ${category}`, subtitle: [...new Set([value, mpn].filter(Boolean))].join(' · ') }
}

export function sourcePartValues(registry: DocumentRegistry | null): Map<string, string> {
  const values = new Map<string, string>()
  for (const document of registry?.documents ?? []) {
    if (!['original-schematic', 'original-pcb'].includes(document.id) || document.status === 'invalid') continue
    for (const applicability of document.applicability) {
      if (applicability.relation !== 'exact' || !applicability.facts?.value) continue
      for (const path of applicability.instances) if (!values.has(path)) values.set(path, applicability.facts.value)
    }
  }
  return values
}

/** Suggestions are derived evidence; Important remains the only confirmed authority. */
export function coreCandidates(model: ExplorerModel, sourceValues: ReadonlyMap<string, string> = new Map()) {
  const rails = new Set(model.derived.rails)
  return model.instances.flatMap(instance => {
    if (['Resistor', 'Capacitor', 'Inductor'].includes(partPresentation(instance).category)) return []
    const mpn = instance.part?.mpn ?? ''
    const sourceValue = sourceValues.get(instance.path) ?? ''
    const identity = mpn || sourceValue
    const family = identity.match(/^(STM32|SF32LB|ESP32|RP20|RP23|W25Q|GD25|DRV|TAS|TLV|TPS|RT61|CH32|CH34|BMI|ICM|MAX)/i)?.[1]
    const category = instance.impl_traits.find(t => ['IC', 'Microcontroller', 'Memory', 'MotorDriver', 'Amplifier'].includes(t.split('::').at(-1) ?? ''))?.split('::').at(-1)
    if (!family && !category) return []
    const signals = model.nets.filter(n => !n.is_gnd && !rails.has(n.name) && n.members.some(m => m.instance_path === instance.path)).length
    return [{ instance, signals, evidence: `${family ? `${mpn ? 'MPN' : 'Source value'} ${identity}` : `Declared ${category}`} · ${signals} signal networks` }]
  }).sort((a, b) => b.signals - a.signals || a.instance.path.localeCompare(b.instance.path))
}
