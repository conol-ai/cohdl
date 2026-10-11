import type { ReactNode } from 'react'

export interface PinIdentity { instance: string; pin: string; net: string }
export function PinTarget({ identity, selected, children, onActivate, style }: {
  identity: PinIdentity; selected: boolean; children: ReactNode
  onActivate: (identity: PinIdentity) => void; style?: React.CSSProperties
}) {
  return <div className="pin-target nodrag nopan" role="button" tabIndex={0} data-instance={identity.instance} data-pin={identity.pin}
    aria-label={`${identity.pin}${identity.net ? ` · ${identity.net}` : ''}`} aria-pressed={selected} style={style}
    onPointerDown={e => e.stopPropagation()} onClick={e => { e.stopPropagation(); onActivate(identity) }}
    onKeyDown={e => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); onActivate(identity) } }}>
    {children}
  </div>
}
