// ExplorerModel v1 — mirror of extractor/src/model.rs (schema_version 1).

export interface ExplorerModel {
  schema_version: number
  design: string
  verdict: string
  instances: Instance[]
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
}

export interface FootprintGeo {
  pads: FpPad[]
  mount_holes: FpHole[]
  courtyard?: FpShape
  window?: FpShape
}

export interface FpPad {
  number: string
  shape: 'rect' | 'circle' | 'oval'
  x: number
  y: number
  size: number[]
  rotate: number
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
