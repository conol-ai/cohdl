// Wire direction is a graph-layout choice, never an electrical pin identity.
// Keep passive terminals fixed even when one net has both incoming and
// outgoing edges at the same part (for example an RF matching network).
import type { GEdge, GNode } from './transform'

export const pinHandleId = (pin: string) => `p:${pin}`

export function passiveTerminals(n: GNode) {
  if (n.kind !== 'passive' || !n.inst) return []
  const pins = n.inst.pins
  const leftCount = Math.ceil(pins.length / 2)
  return pins.map((p, index) => {
    const left = index < leftCount
    const count = left ? leftCount : pins.length - leftCount
    const row = left ? index : index - leftCount
    return {
      pin: p.logical, id: pinHandleId(p.logical),
      side: left ? 'left' as const : 'right' as const,
      x: left ? 0 : n.width, y: n.height * (row + 1) / (count + 1),
      label: `${p.logical} (${p.numbers.join(',')}): ${n.pinNets[p.logical] ?? (p.nc ? 'nc' : 'unconnected')}`,
    }
  })
}

/** Named anchors supported by the active renderer, shared by both modes. */
export function wireHandles(e: GEdge, nodes: ReadonlyMap<string, GNode>, detailed: boolean) {
  const handle = (id: string, pin?: string) => {
    const n = nodes.get(id)
    if (!n || !pin) return undefined
    if (n.ports || (detailed && n.kind === 'ic')) return pinHandleId(pin)
    return passiveTerminals(n).find((p) => p.pin === pin)?.id
  }
  return { sourceHandle: handle(e.source, e.sourcePin), targetHandle: handle(e.target, e.targetPin) }
}
