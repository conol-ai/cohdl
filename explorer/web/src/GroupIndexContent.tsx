import { ArrowRight } from 'lucide-react'
import type { GroupIndexEntry } from './groupIndex'

export function GroupIndexContent({ entry, onOpen }: { entry: GroupIndexEntry; onOpen: (group: string) => void }) {
  const cores = entry.cores.map(core => core.label).join(' · ')
  const interfaces = entry.interfaces.join(' · ')
  return <div className="group-index-identity" data-group={entry.name} style={{ padding: '8px 12px', height: '100%', boxSizing: 'border-box' }}>
    <div title={entry.name} style={{ fontSize: 20, lineHeight: '24px', fontWeight: 700, whiteSpace: 'nowrap', textOverflow: 'ellipsis', overflow: 'hidden' }}>{entry.name}</div>
    <div className="group-index-count" style={{ fontSize: 12, lineHeight: '18px', opacity: .8 }}>{entry.parts} parts</div>
    <div className="group-index-cores" title={cores || 'No identified core'} style={{ fontSize: 13, lineHeight: '18px', height: 36, overflow: 'hidden', overflowWrap: 'anywhere' }}>{cores || 'No identified core'}</div>
    <div className="group-index-interfaces" title={interfaces || (entry.rails.length ? entry.rails.join(' · ') : 'No crossing signals')} style={{ fontSize: 12, lineHeight: '18px', whiteSpace: 'nowrap', textOverflow: 'ellipsis', overflow: 'hidden', opacity: .8 }}>{interfaces || (entry.rails.length ? entry.rails.join(' · ') : 'No crossing signals')}</div>
    <button className="group-entry nodrag nopan" aria-label={`View inside ${entry.name}`} onPointerDown={event => event.stopPropagation()} onKeyDown={event => event.stopPropagation()} onClick={event => { event.stopPropagation(); onOpen(entry.name) }} style={{ display: 'flex', alignItems: 'center', gap: 5, padding: '3px 0', fontSize: 12, lineHeight: '18px', color: 'inherit', background: 'transparent', border: 0, cursor: 'pointer', fontWeight: 600 }}>View inside <ArrowRight size={14} /></button>
  </div>
}
