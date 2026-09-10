// ExplorerModel v1 — mirror of extractor/src/model.rs (schema_version 1).

export interface ExplorerModel {
  schema_version: number
  design: string
  verdict: string
  instances: Instance[]
  /** Additive v1 metadata. Older snapshots have no retained boundaries. */
  subdesigns?: Subdesign[]
  nets: Net[]
  nc: { instance_path: string; logical_pin: string }[]
  diagnostics: Diag[]
  derived: {
    two_terminal: string[]
    rails: string[]
    fn_groups: { name: string; members: string[] }[]
    bypasses: { cap: string; target: string }[]
  }
  footprints: Record<string, FootprintGeo>
  /** Absent on old snapshots; null when no layout facts were declared. */
  layout?: DesignLayout | null
}

export type Matrix = [number, number, number, number]
export interface Placement {
  instance: string
  at: [number, number]
  at_mm: [string, string]
  rotate: number
  side: 'top' | 'bottom'
  matrix: Matrix
}

export interface BoardOutline {
  source: string
  start: [number, number]
  segments: ({ type: 'line'; to: [number, number] } | {
    type: 'arc'; to: [number, number]; center: [number, number]; clockwise: boolean
  })[]
}

export interface DesignLayout {
  placements: Placement[]
  board_outline: BoardOutline | null
  outline_source?: string
  outline_error?: string
  net_classes: { name: string; nets: string[] }[]
  diff_pairs: { p: string; n: string }[]
  length_matches: { nets: string[]; tolerance: string | null }[]
  placement_hints: { designator: string; instance: string; hint: string }[]
}

export interface Subdesign {
  path: string
  definition: string
  parent?: string
  span: SrcSpan
  ports: SubdesignPort[]
  /** Defaults in this scope's local frame, independent of its board anchor. */
  local_placements?: Placement[]
}

export interface SubdesignPort {
  name: string
  obligation: string
  net?: string
  connected: boolean
}

export interface FootprintGeo {
  pads: FpPad[]
  mount_holes: FpHole[]
  courtyard?: FpShape
  window?: FpShape
}

export interface FpPad {
  number: string
  shape: 'rect' | 'circle' | 'oval' | 'annulus'
  x: number
  y: number
  size: number[]
  rotate: number
  matrix?: Matrix
  layer?: string
  corner_radius?: number
  chamfer?: { corner: string; cut: number }
  drill?: number[]
  pth: boolean
}

export interface FpHole {
  shape: string
  x: number
  y: number
  size: number[]
  plated: boolean
}

export interface FpShape {
  shape: string
  x: number
  y: number
  size: number[]
}

export interface Instance {
  path: string
  device_fq: string
  variant?: string
  designator?: string
  part?: { fq: string; mfr?: string; mpn?: string; footprint?: string }
  impl_traits: string[]
  placement_hint?: string
  specs: { name: string; value: string }[]
  span: SrcSpan
  docs: { name: string; abs: string }[]
  pins: Pin[]
}

export interface Pin {
  logical: string
  numbers: string[]
  role: string
  obligation: string
  connected: boolean
  nc: boolean
}

export interface Net {
  name: string
  voltage?: string
  is_gnd: boolean
  members: { instance_path: string; logical_pin: string; numbers: string[] }[]
  span: SrcSpan
}

export interface Diag {
  code: string
  severity: string
  message: string
  span?: SrcSpan
}

export interface SrcSpan {
  file: string
  line: number
  col: number
}

export const shortName = (fq: string): string => {
  const parts = fq.split('::')
  return parts[parts.length - 1]
}
