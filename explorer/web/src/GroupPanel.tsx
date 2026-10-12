import { useEffect, useRef, useState } from 'react'
import { ChevronDown, ChevronRight, FoldVertical, UnfoldVertical, X } from 'lucide-react'

export function GroupPanel({ names, selected, collapsed, onToggle, onAll, onCollapsed, layout, open, onClose }: {
  names: string[]; selected: string[]; collapsed: string[]; onToggle: (name: string) => void
  onAll: () => void; onCollapsed: (names: string[]) => void; layout: boolean; open: boolean; onClose: () => void
}) {
  const [search, setSearch] = useState('')
  const panel = useRef<HTMLElement>(null)
  const previousFocus = useRef<HTMLElement | null>(null)
  useEffect(() => {
    if (!open) return
    previousFocus.current = document.activeElement as HTMLElement
    panel.current?.querySelector<HTMLInputElement>('input[type=search]')?.focus()
    return () => previousFocus.current?.focus()
  }, [open])
  const shown = names.filter(name => name.toLocaleLowerCase().includes(search.trim().toLocaleLowerCase()))
  return <aside ref={panel} id="group-panel" className={`group-panel${open ? ' is-open' : ''}`} aria-label="Groups" role={open ? 'dialog' : undefined} aria-modal={open || undefined} onKeyDown={event => {
    if (event.key === 'Escape' && open) onClose()
    if (event.key !== 'Tab' || !open) return
    const elements = [...panel.current!.querySelectorAll<HTMLElement>('button:not(:disabled), input')]
    const first = elements[0], last = elements[elements.length - 1]
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus() }
    if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus() }
  }}>
    <div className="group-panel-heading"><h2>Groups</h2><button className="icon-tool group-drawer-close" aria-label="Close groups" onClick={onClose}><X size={18} /></button></div>
    <input className="group-search" type="search" aria-label="Search groups" placeholder="Search groups" value={search} onChange={event => setSearch(event.target.value)} />
    <div className="group-selection-summary" role="status">{selected.length ? `${selected.length} selected · ${selected.join(' + ')}` : 'All groups'}</div>
    <div className="group-panel-actions"><button className="context-command" disabled={!selected.length} onClick={onAll}>Show all groups</button>
      {!layout && <span className="group-bulk-tools"><button className="collapse-tool" title="Collapse all groups" aria-label="Collapse all groups" disabled={names.every(name => collapsed.includes(name))} onClick={() => onCollapsed(names)}><FoldVertical size={16} /></button><button className="collapse-tool" title="Expand all groups" aria-label="Expand all groups" disabled={!collapsed.length} onClick={() => onCollapsed([])}><UnfoldVertical size={16} /></button></span>}
    </div>
    <fieldset className="group-selector"><legend className="visually-hidden">Group selection</legend>{shown.map(name => <div className="group-control" data-selected={selected.includes(name)} key={name}><label><input type="checkbox" checked={selected.includes(name)} onChange={() => onToggle(name)} /><span>{name}</span></label>
      {!layout && <button className="collapse-tool" aria-label={`${collapsed.includes(name) ? 'Expand' : 'Collapse'} ${name}`} title={`${collapsed.includes(name) ? 'Expand' : 'Collapse'} ${name}`} aria-expanded={!collapsed.includes(name)} onClick={() => onCollapsed(collapsed.includes(name) ? collapsed.filter(group => group !== name) : [...collapsed, name])}>{collapsed.includes(name) ? <ChevronRight size={16} /> : <ChevronDown size={16} />}</button>}
    </div>)}{!shown.length && <div className="group-no-results">No matching groups</div>}</fieldset>
  </aside>
}
