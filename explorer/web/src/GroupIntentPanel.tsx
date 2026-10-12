import { useState } from 'react'
import { Eye, Save, Undo2, X, WandSparkles, Pencil, Trash2 } from 'lucide-react'
import type { ExplorerModel } from './model'
import { parseViewConfig, type ViewConfig } from './views'
import './GroupIntentPanel.css'

interface FocusGroup { view: string; name: string; instances: string[]; nets: { name: string; inside: unknown[]; outside: unknown[] }[] }
interface Focus { schema: number; run: string; config: ViewConfig; groups: FocusGroup[]; rationale: string; uncertainty: string; model_sha256: string }
interface Props {
  model: ExplorerModel
  config?: ViewConfig | null
  selection: { schema: 1; instances: string[]; groups: { view: string; region: string }[] }
  onFocus: (config: ViewConfig, view: string, groups: string[]) => void
  onCancel: () => void
  onSaved: () => void
}

async function action(request: Record<string, unknown>) {
  const response = await fetch('/api/group-actions', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-CoHDL-Group-Action': '1' }, body: JSON.stringify(request) })
  const value = await response.json()
  if (!response.ok || value.error) throw new Error(value.error ?? `Group action failed (${response.status})`)
  return value
}

export function GroupIntentPanel({ model, config, selection, onFocus, onCancel, onSaved }: Props) {
  const [intent, setIntent] = useState('')
  const [run, setRun] = useState('')
  const [focus, setFocus] = useState<Focus | null>(null)
  const [busy, setBusy] = useState('')
  const [error, setError] = useState('')
  const [saved, setSaved] = useState(false)
  const [target, setTarget] = useState('')
  const [activeView, setActiveView] = useState('')
  const inspect = async (id: string) => {
    const result = await action({ action: 'focus', run: id }) as Focus
    const parsed = parseViewConfig(result.config, model.design)
    const paths = new Set(model.instances.map(i => i.path))
    if (!Array.isArray(result.groups) || result.groups.some(g => !Array.isArray(g.instances) || g.instances.some(p => !paths.has(p)))) throw new Error('Group contains invalid or stale members')
    setFocus({ ...result, config: parsed })
    setActiveView(result.groups[0]?.view ?? '')
    return { ...result, config: parsed }
  }
  const perform = async (name: string, operation: () => Promise<void>) => {
    setBusy(name); setError('')
    try { await operation() } catch (e) { setError(e instanceof Error ? e.message : String(e)) }
    finally { setBusy('') }
  }
  const generate = (prefix = '') => perform('Generating group proposal', async () => {
    onCancel(); setFocus(null); setSaved(false)
    const result = await action({ action: 'propose', intent: `${prefix}${intent}`, selection })
    setRun(result.id)
    await inspect(result.id)
  })
  const groups = focus?.groups.filter(g => g.view === activeView) ?? []
  return <details className="group-intent">
    <summary>AI Groups</summary>
    <label>Viewing intent<textarea aria-label="Group viewing intent" value={intent} onChange={e => setIntent(e.target.value)} maxLength={16000} rows={3} /></label>
    <button type="button" disabled={!!busy || !intent.trim()} onClick={() => void generate()}><WandSparkles size={14} />Generate</button>
    {!!config?.views.length && <div className="group-edit-target"><label>Saved group<select aria-label="Saved group to edit" value={target} onChange={e => setTarget(e.target.value)}><option value="">Select group</option>{config.views.flatMap(v => v.regions.map(r => <option key={JSON.stringify([v.name, r.name])} value={JSON.stringify([v.name, r.name])}>{v.name} / {r.name}</option>))}</select></label>
      <button type="button" title="Propose edits to selected group" aria-label="Propose group edit" disabled={!!busy || !target || !intent.trim()} onClick={() => void generate(`Edit canonical group ${target}. Requested change: `)}><Pencil size={14} /></button>
      <button type="button" title="Propose deletion of selected group" aria-label="Propose group deletion" disabled={!!busy || !target} onClick={() => void generate(`Delete canonical group ${target}, updating navigation references and preserving registry applicability. Additional context: `)}><Trash2 size={14} /></button>
    </div>}
    <label>Proposal run<input aria-label="Group proposal run" value={run} onChange={e => { setRun(e.target.value); setFocus(null); setSaved(false) }} /></label>
    <button type="button" disabled={!!busy || !run} onClick={() => void perform('Reviewing group', async () => { await inspect(run) })}><Eye size={14} />Review</button>
    {busy && <p role="status">{busy}...</p>}{error && <p role="alert">{error}</p>}
    {focus && <section aria-label="Group proposal preview">
      <p>{focus.rationale}</p><p className="group-uncertainty">{focus.uncertainty}</p>
      {new Set(focus.groups.map(g => g.view)).size > 1 && <label>View<select value={activeView} onChange={e => setActiveView(e.target.value)}>{[...new Set(focus.groups.map(g => g.view))].map(v => <option key={v}>{v}</option>)}</select></label>}
      {!groups.length && <p>No added or changed member groups; review deletion or navigation changes before saving.</p>}
      {groups.map(g => <details key={JSON.stringify([g.view, g.name])}><summary>{g.name} · {g.instances.length} components · {g.nets.length} networks</summary>
        <ul>{g.instances.map(path => { const instance = model.instances.find(i => i.path === path); return <li key={path}>{instance?.designator || instance?.part?.mpn || 'Unreferenced component'}</li> })}</ul>
        <details><summary>Technical member identifiers</summary><pre>{g.instances.join('\n')}</pre></details>
        <ul>{g.nets.map(n => <li key={n.name}>{n.name} · {n.inside.length} inside / {n.outside.length} outside</li>)}</ul>
      </details>)}
      <details><summary>View changes</summary><pre>{JSON.stringify(focus.config, null, 2)}</pre></details>
      <div className="group-intent-actions"><button type="button" disabled={!!busy || saved || !groups.some(g => g.instances.length)} onClick={() => void perform('Focusing group', async () => { const latest = await inspect(run); const selected = latest.groups.filter(g => g.view === activeView); onFocus(latest.config, activeView, selected.map(g => g.name)) })}><Eye size={14} />Focus</button>
        <button type="button" title="Cancel temporary focus" aria-label="Cancel temporary focus" onClick={onCancel}><X size={14} /></button>
        <button type="button" disabled={!!busy || saved} onClick={() => void perform('Saving group', async () => { await action({ action: 'save', run }); setSaved(true); onSaved() })}><Save size={14} />Save</button>
        <button type="button" title="Undo saved group change" aria-label="Undo saved group change" disabled={!!busy || !saved} onClick={() => void perform('Undoing group', async () => { await action({ action: 'undo', run }); setSaved(false); setFocus(null); onSaved() })}><Undo2 size={14} /></button>
      </div>
    </section>}
  </details>
}
