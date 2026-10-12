import { useCallback, useEffect, useRef, useState } from 'react'
import type { ExplorerModel } from './model'
import type { InspectionContext } from './inspection'
import type { ViewConfig } from './views'
import { contextKey, inspectionUrl, normalizeContext, parseInspectionUrl } from './navigation'

export function useInspectionNavigation({ model, config, ready, busy, signature, read, apply, sourceValues }: {
  model: ExplorerModel | null; config: ViewConfig | null; ready: boolean; busy: boolean; signature: string
  read: () => InspectionContext; apply: (context: InspectionContext) => void
  sourceValues?: ReadonlyMap<string, string>
}) {
  const latest = useRef({ model, config, read, apply, ready, busy, sourceValues })
  latest.current = { model, config, read, apply, ready, busy, sourceValues }
  const contexts = useRef(new Map<string, InspectionContext>())
  const pending = useRef<InspectionContext | null>(null)
  const initialized = useRef(false)
  const generation = useRef(0)
  const [revision, setRevision] = useState(0)
  const [canBack, setCanBack] = useState(!!history.state?.explorer)
  const restore = useCallback((context: InspectionContext) => {
    generation.current++
    pending.current = context
    latest.current.apply(context)
    setRevision((v) => v + 1)
  }, [])
  const sync = useCallback(() => {
    if (!initialized.current || pending.current || !latest.current.ready || latest.current.busy) return
    const context = latest.current.read()
    contexts.current.set(contextKey(context), context)
    const next = inspectionUrl(new URL(location.href), context).href
    if (next !== location.href) history.replaceState(history.state, '', next)
  }, [])
  const navigate = useCallback((patch: Partial<InspectionContext>) => {
    const { model: currentModel, config: currentConfig } = latest.current
    if (!currentModel || !initialized.current || !latest.current.ready) return
    const current = latest.current.read()
    const source = pending.current ? { ...pending.current,
      layoutSide: contextKey(current) === contextKey(pending.current) ? current.layoutSide : pending.current.layoutSide } : current
    contexts.current.set(contextKey(source), source)
    history.replaceState(history.state, '', inspectionUrl(new URL(location.href), source))
    const key = contextKey({ ...source, ...patch })
    const target = key === contextKey(source) ? source : contexts.current.get(key) ?? { ...source, viewport: null, boardViewport: null }
    const next = normalizeContext({ ...target, ...patch }, currentModel, currentConfig)
    history.pushState({ explorer: true }, '', inspectionUrl(new URL(location.href), next))
    setCanBack(true)
    restore(next)
  }, [restore])
  useEffect(() => {
    if (!ready || !model) return
    const current = initialized.current ? pending.current ?? latest.current.read() : parseInspectionUrl(new URL(location.href), model, config, latest.current.sourceValues)
    initialized.current = true
    restore(normalizeContext(current, model, config))
  }, [ready, model, config, restore])
  useEffect(() => {
    const pop = () => {
      const { model: currentModel, config: currentConfig } = latest.current
      if (!currentModel || !initialized.current) return
      setCanBack(!!history.state?.explorer)
      if (!pending.current) {
        const current = latest.current.read()
        contexts.current.set(contextKey(current), current)
      }
      restore(parseInspectionUrl(new URL(location.href), currentModel, currentConfig, latest.current.sourceValues))
    }
    addEventListener('popstate', pop)
    return () => removeEventListener('popstate', pop)
  }, [restore])
  useEffect(() => {
    if (!initialized.current || busy || !ready || (!config && pending.current?.view)) return
    const currentGeneration = generation.current
    let frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => {
        if (currentGeneration !== generation.current || !latest.current.ready || latest.current.busy) return
        pending.current = null
        sync()
      })
    })
    return () => cancelAnimationFrame(frame)
  }, [busy, ready, config, signature, revision, sync])
  return { navigate, sync, canBack, back: () => history.back() }
}
