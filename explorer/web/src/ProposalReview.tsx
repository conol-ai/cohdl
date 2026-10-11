import { useState } from 'react'
import { ArrowLeft, Copy, ExternalLink, RefreshCw } from 'lucide-react'
import { documentHref, type DocumentResource } from './documents'
import type { Operation, ProcessEvidence, ProposalResource, Validation } from './proposals'
import { SourceExcerpt } from './SourceExcerpt'
import './ProposalReview.css'

function ProcessResult({ evidence }: { evidence: ProcessEvidence }) {
  return <dl className="proposal-fields">
    <dt>Command</dt><dd><code>{JSON.stringify(evidence.command)}</code></dd>
    <dt>Working directory</dt><dd>{evidence.cwd}</dd>
    <dt>Started</dt><dd>{evidence.started_at}</dd><dt>Finished</dt><dd>{evidence.finished_at ?? 'Unfinished'}</dd>
    <dt>Exit</dt><dd>{evidence.exit_code ?? 'Unknown'}{evidence.signal && ` · ${evidence.signal}`}</dd>
    {evidence.error && <><dt>Error</dt><dd role="alert">{evidence.error}</dd></>}
  </dl>
}
function CompilerResult({ validation, title }: { validation: Validation; title: string }) {
  const compiler = validation.compiler
  return <section className="proposal-section" aria-label={title}>
    <h4>{title}: {validation.decision}</h4>
    {validation.issues.map((issue, i) => <p className="proposal-issue" key={i}>{issue}</p>)}
    {compiler ? <><p>Compiler verdict: {compiler.result?.verdict ?? 'Unavailable'}</p><ProcessResult evidence={compiler} />
      {compiler.result?.diagnostics.map((diagnostic, i) => <pre className="proposal-diagnostic" key={i}>{JSON.stringify(diagnostic, null, 2)}</pre>)}</> : <p>No compiler result retained</p>}
  </section>
}
function OperationResult({ operation, title }: { operation: Operation; title: string }) {
  return <div className="proposal-operation"><h4>{title}: {operation.outcome}</h4><div>{operation.started_at}</div><div>{operation.finished_at ?? 'Unfinished'}</div>{operation.error && <p role="alert">{operation.error}</p>}</div>
}

export function ProposalReview({ resource, documents, onBack }: { resource: ProposalResource; documents: DocumentResource; onBack: () => void }) {
  const { detail, runs } = resource
  const run = detail?.run
  const proposal = run?.proposal
  const [copyError, setCopyError] = useState('')
  return <div className="proposal-review">
    <div className="proposal-heading"><h3>Proposals</h3><div>
      <button type="button" className="icon-tool" title="Refresh proposals" aria-label="Refresh proposals" onClick={resource.refresh}><RefreshCw size={16} /></button>
      <button type="button" className="icon-tool" title="Back to Circuit" aria-label="Back to Circuit" onClick={onBack}><ArrowLeft size={16} /></button>
    </div></div>
    {resource.loading && <p role="status">Loading proposals...</p>}
    {resource.error && <p role="alert">{resource.error}</p>}
    {!resource.loading && !resource.error && !runs.length && <p role="status">No proposals</p>}
    {!!runs.length && <label className="proposal-picker">Run<select aria-label="Proposal run" value={resource.selected} onChange={e => resource.select(e.target.value)}>
      {runs.map(r => <option key={r.id} value={r.id}>{r.title} · {r.status} · {r.freshness}</option>)}
    </select></label>}
    {resource.detailLoading && <p role="status">Loading proposal...</p>}
    {resource.detailError && <p role="alert">{resource.detailError}</p>}
    {detail && run && <>
      <div className="proposal-state" data-status={detail.status} data-freshness={detail.freshness}><strong>{detail.status}</strong><span>{detail.freshness}</span></div>
      <div className="proposal-id">{run.id}{run.kind && ` · ${run.kind}`}</div>
      {detail.issues.map((issue, i) => <p className="proposal-issue" key={i}>{issue}</p>)}
      {run.provider && <details className="proposal-section"><summary>Provider: {run.provider.engine}</summary><ProcessResult evidence={run.provider} />{run.provider.heartbeat_at && <p>Last heartbeat: {run.provider.heartbeat_at}</p>}</details>}
      {run.bundle && <details className="proposal-section"><summary>Input: {run.bundle.design}</summary><p>{run.bundle.selection.instances.length ? run.bundle.selection.instances.join(', ') : 'Complete circuit'}</p>
        {run.bundle.selection.groups.map(g => <p key={JSON.stringify(g)}>{g.view} / {g.region}</p>)}<pre>{JSON.stringify(run.bundle.snapshot, null, 2)}</pre></details>}
      {proposal && <>
        <div className="proposal-title"><h3>{proposal.title}</h3><button type="button" className="icon-tool" title="Copy proposal JSON" aria-label="Copy proposal JSON" onClick={() => {
          if (!navigator.clipboard) { setCopyError('Clipboard unavailable'); return }
          navigator.clipboard.writeText(JSON.stringify(proposal, null, 2)).then(() => setCopyError(''), () => setCopyError('Unable to copy proposal'))
        }}><Copy size={16} /></button></div>
        {copyError && <p role="alert">{copyError}</p>}
        <section className="proposal-section"><h4>Rationale</h4><p>{proposal.rationale}</p><h4>Uncertainty</h4><p>{proposal.uncertainty || 'None stated'}</p></section>
        <section className="proposal-section" aria-label="Proposal citations"><h4>Citations</h4>{proposal.citations.map((citation, i) => {
          const document = documents.registry?.documents.find(d => d.id === citation.document)
          const href = document && documentHref(document, citation.page)
          return <div className="proposal-citation" key={i}>
            {citation.document && (href ? <a href={href} target="_blank" rel="noreferrer">{document.title}{citation.page && ` · PDF page ${citation.page}`} <ExternalLink size={13} /></a>
              : <div>{document?.title ?? citation.document} · Document unavailable{citation.page && ` · PDF page ${citation.page}`}</div>)}
            {citation.section && <div>{citation.section}</div>}
            {citation.source && <><SourceExcerpt span={{ file: citation.source.path, line: citation.source.line, col: 1 }} expectedHash={citation.source.sha256} /><div className="document-hash">Cited SHA-256 {citation.source.sha256}</div></>}
          </div>
        })}</section>
        <section className="proposal-section" aria-label="Proposal changes"><h4>Changes ({proposal.changes.length})</h4>
          {!proposal.changes.length && <p>No file edits</p>}
          {proposal.changes.map(change => <div className="proposal-change" key={change.path}><h4>{change.path}</h4><div className="document-hash">Before SHA-256 {change.before_sha256 ?? 'Absent file'}</div>
            <h5>Old</h5><pre data-diff="old">{change.old}</pre><h5>New</h5><pre data-diff="new">{change.new}</pre></div>)}
        </section>
      </>}
      {run.validation && <CompilerResult validation={run.validation} title="Validation" />}
      {run.receipt && <section className="proposal-section" aria-label="Receipt history"><h4>Receipt history</h4><OperationResult title="Apply" operation={run.receipt.apply} />
        {run.receipt.revert && <OperationResult title="Revert" operation={run.receipt.revert} />}{run.receipt.apply_validation ? <CompilerResult title="Fresh apply validation" validation={run.receipt.apply_validation} /> : <p>Fresh apply validation unavailable</p>}
        <details><summary>Receipt evidence</summary><pre>{JSON.stringify(run.receipt, null, 2)}</pre></details></section>}
      {!!detail.artifacts.length && <section className="proposal-section"><h4>Retained output</h4>{detail.artifacts.map(artifact => <a key={artifact.name} href={artifact.url} target="_blank" rel="noreferrer">{artifact.name} <ExternalLink size={13} /></a>)}</section>}
    </>}
  </div>
}
