import { useEffect, useState } from 'react'
import { parseDocumentRegistry, type DocumentRegistry, type DocumentResource } from './documents'
import type { ExplorerModel } from './model'

export function useDocuments(model: ExplorerModel | null): DocumentResource {
  const [registry, setRegistry] = useState<DocumentRegistry | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [refresh, setRefresh] = useState(0)
  useEffect(() => {
    if (!model) return
    const controller = new AbortController()
    setLoading(true)
    fetch('/api/documents', { signal: controller.signal }).then(async response => {
      const body: unknown = await response.json()
      if (!response.ok) throw new Error(response.status === 404 ? 'No document registry for this project' : typeof body === 'object' && body !== null && 'error' in body ? String(body.error) : `Document request failed (${response.status})`)
      return parseDocumentRegistry(body)
    }).then(next => { if (!controller.signal.aborted) { setRegistry(next); setError('') } }).catch(reason => {
      if (!controller.signal.aborted) { setRegistry(null); setError(String(reason instanceof Error ? reason.message : reason)) }
    }).finally(() => { if (!controller.signal.aborted) setLoading(false) })
    return () => controller.abort()
  }, [model, refresh])
  return { registry, loading, error, refresh: () => setRefresh(v => v + 1) }
}
