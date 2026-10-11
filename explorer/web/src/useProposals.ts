import { useEffect, useState } from 'react'
import type { ExplorerModel } from './model'
import { parseProposalDetail, parseProposalList, proposalUrl, type ProposalDetail, type ProposalResource, type ProposalSummary } from './proposals'

async function response(url: string, signal: AbortSignal): Promise<unknown> {
  const result = await fetch(url, { signal })
  const body: unknown = await result.json()
  if (!result.ok) throw new Error(typeof body === 'object' && body && 'error' in body ? String(body.error) : `Proposal request failed (${result.status})`)
  return body
}
const message = (reason: unknown) => reason instanceof Error ? reason.message : String(reason)

export function useProposals(model: ExplorerModel | null, enabled: boolean): ProposalResource {
  const [runs, setRuns] = useState<ProposalSummary[]>([])
  const [selected, select] = useState('')
  const [detail, setDetail] = useState<ProposalDetail | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [detailLoading, setDetailLoading] = useState(false)
  const [detailError, setDetailError] = useState('')
  const [revision, refresh] = useState(0)
  useEffect(() => {
    if (!model || !enabled) return
    const controller = new AbortController()
    setLoading(true)
    response('/api/proposals', controller.signal).then(parseProposalList).then(next => {
      if (controller.signal.aborted) return
      setRuns(next); setError('')
      select(previous => next.some(r => r.id === previous) ? previous : next[0]?.id ?? '')
    }).catch(reason => { if (!controller.signal.aborted) { setRuns([]); select(''); setError(message(reason)) } })
      .finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [model, enabled, revision])
  useEffect(() => {
    if (!enabled) return
    setDetail(null); setDetailError('')
    if (!selected) { setDetailLoading(false); return }
    const controller = new AbortController()
    setDetailLoading(true)
    response(proposalUrl(selected), controller.signal).then(parseProposalDetail).then(next => {
      if (!controller.signal.aborted) { if (next.run.id !== selected) throw new Error('Proposal detail ID mismatch'); setDetail(next) }
    }).catch(reason => { if (!controller.signal.aborted) setDetailError(message(reason)) })
      .finally(() => { if (!controller.signal.aborted) setDetailLoading(false) })
    return () => controller.abort()
  }, [model, enabled, selected, revision])
  return { runs, selected, detail, loading, error, detailLoading, detailError, select, refresh: () => refresh(v => v + 1) }
}
