import { MapPin } from 'lucide-react'
import type { Diag, ExplorerModel } from './model'
import { shortName } from './model'
import { inspectionIssues, type SearchHit } from './search'
import './InspectionLists.css'

interface InspectionListsProps {
  model: ExplorerModel
  tab: 'bom' | 'issues'
  onLocate: (hit: SearchHit) => void
  onIssue: (issue: Diag) => void
}

function sourceValue(value: string | undefined) {
  if (value === undefined) return <span className="source-unknown">Unknown</span>
  if (value === '') return <span className="source-unknown">Not specified</span>
  return value
}

export function InspectionLists({ model, tab, onLocate, onIssue }: InspectionListsProps) {
  if (tab === 'bom') return <section className="inspection-lists" aria-label="BOM">
    <h3>BOM <span>{model.instances.length}</span></h3>
    {!model.instances.length ? <p className="inspection-empty">No parts</p> : <table className="inspection-list">
      <thead><tr><th scope="col">Part</th><th scope="col">MPN / Value</th><th scope="col">Footprint</th></tr></thead>
      <tbody>{model.instances.map((instance) => {
        const label = instance.designator || instance.path
        return <tr key={instance.path} data-instance={instance.path}>
          <td><button className="locate-link" title={`Locate ${label}: ${instance.path}`} aria-label={`Locate ${label}`}
            onClick={() => onLocate({ kind: 'part', id: instance.path, label, sub: instance.path })}>
            <MapPin size={13} aria-hidden="true" /><span>{label}</span>
          </button>{instance.designator === '' && <div className="source-unknown">Reference not specified</div>}
            <div className="source-secondary" title={instance.device_fq}>{shortName(instance.device_fq)}</div></td>
          <td>{sourceValue(instance.part?.mpn)}
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
