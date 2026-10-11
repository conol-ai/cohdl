import type { Instance } from './model'
import type { DocumentRegistry } from './documents'

export function partPresentation(instance: Instance, sourceValue = '') {
  const electrical = [
    ['resistance', 'Resistor'], ['capacitance', 'Capacitor'], ['inductance', 'Inductor'],
  ] as const
  const declared = electrical.find(([name]) => instance.specs.some(s => s.name === name))
  const trait = electrical.find(([, category]) => instance.impl_traits.some(t => t.split('::').at(-1) === category))
  const category = declared?.[1] ?? trait?.[1] ?? 'Component'
  const value = instance.specs.find(s => s.name === declared?.[0])?.value ?? instance.specs.find(s => s.name === 'value')?.value ?? sourceValue
  const reference = instance.designator ?? 'Component'
  const mpn = instance.part?.mpn ?? ''
  return { reference, category, value, mpn, title: `${reference} ${category}`, subtitle: [value, mpn].filter(Boolean).join(' · ') }
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
