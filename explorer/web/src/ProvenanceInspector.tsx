import { ExternalLink, RefreshCw } from 'lucide-react'
import { applies, documentHref, type Applicability, type Citation, type DocumentRecord, type DocumentResource } from './documents'
import './ProvenanceInspector.css'

function CitationLink({ document, citation }: { document: DocumentRecord | undefined; citation: Citation }) {
  const href = document && documentHref(document, citation.page)
  const label = `${document?.title ?? citation.document}${citation.page ? ` · PDF page ${citation.page}` : ''}${citation.section ? ` · ${citation.section}` : ''}`
  return href ? <a href={href} target="_blank" rel="noreferrer">{label}<ExternalLink size={12} /></a> : <span aria-disabled="true">{label} · Unavailable</span>
}

function Evidence({ document, applicability }: { document: DocumentRecord; applicability: Applicability[] }) {
  return <details className="document-evidence" data-document-id={document.id} open>
    <summary>{document.title}</summary>
    <div className="document-status">{document.kind === 'markdown' ? 'Local supporting Markdown' : 'Original PDF'} · {document.status === 'ready' ? 'Local hash verified' : document.status === 'missing' ? 'No local original' : 'Invalid local evidence'} · {document.review}</div>
    <div>Revision: {document.revision ?? 'Unknown'} · {document.kind === 'pdf' ? `PDF pages: ${document.pages ?? 'Unknown'}` : 'Section citations'}</div>
    {document.uncertainty && <p className="document-uncertainty">{document.uncertainty}</p>}
    {document.issues.map((issue, index) => <div key={index} className="document-issue">{issue}</div>)}
    {applicability.map((a, index) => <div className="document-applicability" key={index}>
      <strong>{a.relation === 'exact' ? 'Exact association' : 'Related part evidence'}</strong>
      <CitationLink document={document} citation={{ document: document.id, page: a.page, section: a.section }} />
      {a.page && document.pages === null && <div>Page count unknown; citation range not verified</div>}
      {a.uncertainty && <p className="document-uncertainty">{a.uncertainty}</p>}
      {a.facts && <dl>{Object.entries(a.facts).map(([key, value]) => <div key={key}><dt>{key}</dt><dd>{value === '' ? <span className="document-empty">Authored empty</span> : value}</dd></div>)}</dl>}
    </div>)}
    <details className="document-fingerprint"><summary>SHA-256</summary>
      <div className="document-hash">Expected {document.sha256 ?? 'Unknown'}</div>
      <div className="document-hash">Actual {document.actual_sha256 ?? 'Not locally verified'}</div>
    </details>
    {document.source_url && /^https?:\/\//.test(document.source_url) && <a href={document.source_url} target="_blank" rel="noreferrer">External source<ExternalLink size={12} /></a>}
  </details>
}

export function ProvenanceInspector({ resource, instance, view, groups }: { resource: DocumentResource; instance?: string; view: string; groups: string[] }) {
  const matches = (instances: string[], targets: { view: string; region: string }[]) => applies(instances, targets, instance, view, groups)
  const documents = resource.registry?.documents.map(document => ({ document, applicability: document.applicability.filter(a => matches(a.instances, a.groups)) })).filter(d => d.applicability.length) ?? []
  const constraints = resource.registry?.constraints.filter(c => matches(c.instances, c.groups)) ?? []
  return <section className="provenance-inspector" aria-label="Source evidence" aria-busy={resource.loading}>
    <div className="provenance-heading"><h3>Source evidence</h3><button type="button" className="icon-tool" aria-label="Refresh source evidence" title="Refresh source evidence" onClick={resource.refresh}><RefreshCw size={14} /></button></div>
    {resource.loading ? <p role="status">Checking documents...</p> : resource.error ? <p role="alert">{resource.error}</p> : <>
      {!documents.length && <p>No applicable document records</p>}
      {documents.map(({ document, applicability }) => <Evidence key={document.id} document={document} applicability={applicability} />)}
      {!!constraints.length && <h4>Recorded constraints</h4>}
      {constraints.map((constraint, index) => <div className="document-constraint" key={index}>
        <p>{constraint.text}</p>
        {constraint.uncertainty && <p className="document-uncertainty">{constraint.uncertainty}</p>}
        {constraint.issues.map((issue, i) => <div key={i} className="document-issue">{issue}</div>)}
        {constraint.citations.map((citation, i) => <CitationLink key={i} document={resource.registry?.documents.find(d => d.id === citation.document)} citation={citation} />)}
      </div>)}
    </>}
  </section>
}
