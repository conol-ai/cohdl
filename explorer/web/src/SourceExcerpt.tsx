import { useEffect, useState } from 'react'
import { Code } from 'lucide-react'
import type { SrcSpan } from './model'
import { sourceUrl } from './documents'

interface Excerpt { path: string; line: number; text: string; sha256: string }

export function SourceExcerpt({ span }: { span: SrcSpan }) {
  const [open, setOpen] = useState(false)
  const [excerpt, setExcerpt] = useState<Excerpt | null>(null)
  const [error, setError] = useState('')
  useEffect(() => {
    setExcerpt(null); setError('')
    if (!open) return
    const controller = new AbortController()
    fetch(sourceUrl(span), { signal: controller.signal }).then(async r => {
      const body = await r.json()
      if (!r.ok) throw new Error(body.error ?? 'Source unavailable')
      if (typeof body.text !== 'string' || typeof body.sha256 !== 'string' || !Number.isSafeInteger(body.line)) throw new Error('Invalid source response')
      return body as Excerpt
    }).then(next => { if (!controller.signal.aborted) setExcerpt(next) }).catch(e => { if (!controller.signal.aborted) setError(String(e.message ?? e)) })
    return () => controller.abort()
  }, [span.file, span.line, open])
  return <details className="source-excerpt" onToggle={event => setOpen(event.currentTarget.open)}>
    <summary><Code size={14} /> Source {span.file}:{span.line}:{span.col}</summary>
    {open && (error ? <p role="alert">{error}</p> : excerpt ? <>
      <div className="document-hash">SHA-256 {excerpt.sha256}</div>
      <pre aria-label="CoHDL source excerpt">{excerpt.text.split('\n').map((line, index) => <div key={index} data-source-line={excerpt.line + index} data-current={excerpt.line + index === span.line}><span>{excerpt.line + index}</span> {line || ' '}</div>)}</pre>
    </> : <p role="status">Loading source...</p>)}
  </details>
}
