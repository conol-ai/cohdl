import { MapPin } from 'lucide-react'
import type { Diag, ExplorerModel } from './model'
import { shortName } from './model'
import { inspectionIssues, type SearchHit } from './search'
import './InspectionLists.css'
import { partPresentation } from './partPresentation'

interface InspectionListsProps {
  model: ExplorerModel
  tab: 'bom' | 'issues'
  onLocate: (hit: SearchHit) => void
  onIssue: (issue: Diag) => void
  query?: string; onQuery?: (query: string) => void; selected?: string; sourceValues?: ReadonlyMap<string, string>
}

function sourceValue(value: string | undefined) {
  if (value === undefined) return <span className="source-unknown">Unknown</span>
  if (value === '') return <span className="source-unknown">Not specified</span>
  return value
}

export function InspectionLists({ model, tab, onLocate, onIssue, query = '', onQuery, selected, sourceValues }: InspectionListsProps) {
  if (tab === 'bom') return <section className="inspection-lists" aria-label="BOM">
    <h3>BOM <span>{model.instances.length}</span></h3>
    {onQuery && <input className="group-search" aria-label="Search BOM" value={query} onChange={event => onQuery(event.target.value)} placeholder="Search BOM" />}
    {!model.instances.length ? <p className="inspection-empty">No parts</p> : <table className="inspection-list">
      <thead><tr><th scope="col">Part</th><th scope="col">MPN / Value</th><th scope="col">Footprint</th></tr></thead>
      <tbody>{model.instances.filter(instance => !query || `${partPresentation(instance, sourceValues?.get(instance.path)).title} ${partPresentation(instance, sourceValues?.get(instance.path)).subtitle}`.toLowerCase().includes(query.toLowerCase())).map((instance) => {
        const identity = partPresentation(instance, sourceValues?.get(instance.path))
        const label = identity.reference
        return <tr key={instance.path} data-instance={instance.path} aria-selected={selected === instance.path}>
          <td><button className="locate-link" title={`Locate ${identity.title}`} aria-label={`Locate ${label}`}
            onClick={() => onLocate({ kind: 'part', id: instance.path, label, sub: identity.subtitle })}>
            <MapPin size={13} aria-hidden="true" /><span>{label}</span>
          </button>{instance.designator === '' && <div className="source-unknown">Reference not specified</div>}
            <div className="source-secondary">{identity.category}</div></td>
          <td>{sourceValue(instance.part?.mpn)}
            {identity.value && <div className="source-secondary">{identity.value}</div>}
            {instance.specs.length > 0 && <div className="source-secondary">{instance.specs.map((spec) => `${spec.name} = ${spec.value}`).join(' · ')}</div>}
          </td>
          <td title={instance.part?.footprint}>{sourceValue(instance.part?.footprint === undefined ? undefined : shortName(instance.part.footprint))}</td>
        </tr>
      })}</tbody>
    </table>}
  </section>
  const issues = inspectionIssues(model)
  return <section className="inspection-lists" aria-label="Issues">
    <h3>Issues <span>{issues.length}</span></h3>
    {!issues.length && <p className="inspection-empty">No issues</p>}
    {issues.map((issue, index) => <article className="inspection-issue" key={index} data-code={issue.code} tabIndex={0}>
      <strong data-severity={issue.severity}>{issue.severity} · {issue.code}</strong>
      <p>{issue.message}</p>
      {issue.span ? <button className="locate-link" title={`Locate ${issue.code} source`} aria-label={`Locate ${issue.code} source`}
        onClick={() => onIssue(issue)}>
        <MapPin size={13} aria-hidden="true" /><span>{issue.span.file}:{issue.span.line}:{issue.span.col}</span>
      </button> : <div className="source-unknown">Source location unavailable</div>}
    </article>)}
  </section>
}
