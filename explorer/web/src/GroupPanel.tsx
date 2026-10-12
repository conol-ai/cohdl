import { useEffect, useRef, useState, type ReactNode } from 'react'
import { ChevronDown, ChevronRight, FoldVertical, UnfoldVertical, X } from 'lucide-react'
import type { ViewDef } from './views'
import { groupCheck, groupNavigation, memberCount, toggleGroupSelection } from './groupTree'
import './GroupPanel.css'

function GroupCheckbox({ name, checked, mixed = false, onChange }: { name: string; checked: boolean; mixed?: boolean; onChange: () => void }) {
  const ref = useRef<HTMLInputElement>(null)
  useEffect(() => { if (ref.current) ref.current.indeterminate = mixed }, [mixed])
  return <input ref={ref} type="checkbox" aria-label={name} aria-checked={mixed ? 'mixed' : checked} checked={checked} onChange={onChange} />
}

export function GroupPanel({ names, selected, collapsed, onToggle, onAll, onCollapsed, layout, open, onClose, navigation, groupMembers, onSelection, children }: {
  names: string[]; selected: string[]; collapsed: string[]; onToggle: (name: string) => void
  onAll: () => void; onCollapsed: (names: string[]) => void; layout: boolean; open: boolean; onClose: () => void
  navigation?: ViewDef['navigation']; groupMembers?: ReadonlyMap<string, ReadonlySet<string>>; onSelection?: (names: string[]) => void
  children?: ReactNode
}) {
  const [search, setSearch] = useState('')
  const [folded, setFolded] = useState<string[]>([])
  const panel = useRef<HTMLElement>(null)
  const previousFocus = useRef<HTMLElement | null>(null)
  useEffect(() => {
    if (!open) return
    previousFocus.current = document.activeElement as HTMLElement
    panel.current?.querySelector<HTMLInputElement>('input[type=search]')?.focus()
    return () => previousFocus.current?.focus()
  }, [open])
  const shown = groupNavigation(names, navigation, search)
  const count = (groups: string[]) => groupMembers && <span className="group-member-count" title="Unique components">{memberCount(groups, groupMembers)}</span>
  const leaf = (name: string, parent?: string) => <div className={`group-control${parent ? ' group-child' : ''}`} data-group={name} data-parent={parent} data-selected={selected.includes(name)} key={name}>
    <label><GroupCheckbox name={name} checked={selected.includes(name)} onChange={() => onToggle(name)} /><span>{name}</span>{count([name])}</label>
    {!layout && <button className="collapse-tool" aria-label={`${collapsed.includes(name) ? 'Expand' : 'Collapse'} ${name}`} title={`${collapsed.includes(name) ? 'Expand' : 'Collapse'} ${name}`} aria-expanded={!collapsed.includes(name)} onClick={() => onCollapsed(collapsed.includes(name) ? collapsed.filter(group => group !== name) : [...collapsed, name])}>{collapsed.includes(name) ? <ChevronRight size={16} /> : <ChevronDown size={16} />}</button>}
  </div>
  const fold = (name: string, hide: boolean) => setFolded(current => hide ? [...new Set([...current, name])] : current.filter(value => value !== name))
  return <aside ref={panel} id="group-panel" className={`group-panel${open ? ' is-open' : ''}`} aria-label="Groups" role={open ? 'dialog' : undefined} aria-modal={open || undefined} onKeyDown={event => {
    if (event.key === 'Escape' && open) onClose()
    if (event.key !== 'Tab' || !open) return
    const elements = [...panel.current!.querySelectorAll<HTMLElement>('button:not(:disabled), input')]
    const first = elements[0], last = elements[elements.length - 1]
    if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus() }
    if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus() }
  }}>
    <div className="group-panel-heading"><h2>Groups</h2><button className="icon-tool group-drawer-close" title="Close groups" aria-label="Close groups" onClick={onClose}><X size={18} /></button></div>
    {children}
    <input className="group-search" type="search" aria-label="Search groups" placeholder="Search groups" value={search} onChange={event => setSearch(event.target.value)} />
    <div className="group-selection-summary" role="status">{selected.length ? `${selected.length} selected · ${selected.join(' + ')}` : 'All groups'}</div>
    <div className="group-panel-actions"><button className="context-command" disabled={!selected.length} onClick={onAll}>Show all groups</button>
      {!layout && <span className="group-bulk-tools"><button className="collapse-tool" title="Collapse all groups" aria-label="Collapse all groups" disabled={names.every(name => collapsed.includes(name))} onClick={() => onCollapsed(names)}><FoldVertical size={16} /></button><button className="collapse-tool" title="Expand all groups" aria-label="Expand all groups" disabled={!collapsed.length} onClick={() => onCollapsed([])}><UnfoldVertical size={16} /></button></span>}
    </div>
    <fieldset className="group-selector"><legend className="visually-hidden">Group selection</legend>{shown.map(entry => {
      if (!entry.parent) return leaf(entry.name)
      const groups = navigation!.find(parent => parent.name === entry.name)!.groups.filter(name => names.includes(name))
      const check = groupCheck(selected, groups)
      const expanded = !!search.trim() || !folded.includes(entry.name)
      return <section className="group-branch" aria-label={entry.name} key={entry.name}>
        <div className="group-control group-parent" data-selected={check.checked}>
          <button className="collapse-tool group-tree-toggle" aria-label={`${expanded ? 'Hide' : 'Show'} ${entry.name} groups`} title={`${expanded ? 'Hide' : 'Show'} ${entry.name} groups`} aria-expanded={expanded} onClick={() => fold(entry.name, expanded)} onKeyDown={event => {
            if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') { event.preventDefault(); fold(entry.name, event.key === 'ArrowLeft') }
          }}>{expanded ? <ChevronDown size={16} /> : <ChevronRight size={16} />}</button>
          <label><GroupCheckbox name={`${entry.name} groups`} {...check} onChange={() => onSelection?.(toggleGroupSelection(selected, groups))} /><span>{entry.name}</span>{count(groups)}</label>
        </div>
        {expanded && <div className="group-children">{entry.groups.map(name => leaf(name, entry.name))}</div>}
      </section>
    })}{!shown.length && <div className="group-no-results">No matching groups</div>}</fieldset>
  </aside>
}
