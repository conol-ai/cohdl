import { useEffect, useRef, useState } from 'react'
import type { ExplorerModel } from './model'
import { marksKey, readMarks, writeMarks } from './important'

export function useImportant(model: ExplorerModel | null) {
  const [marks, setMarks] = useState<Set<string>>(new Set())
  const key = useRef<string | null>(null)
  const knownKey = useRef<string | null>(null)
  const edits = useRef(new Map<string, boolean>())
  const design = useRef<string | null>(null)
  useEffect(() => {
    if (!model) return
    let cancelled = false
    if (key.current) edits.current = new Map()
    key.current = null
    const allowed = new Set(model.instances.map(i => i.path))
    if (design.current !== model.design) { knownKey.current = null; edits.current.clear(); setMarks(new Set()); design.current = model.design }
    setMarks(current => new Set([...current].filter(p => allowed.has(p))))
    fetch('/api/project').then(r => r.ok ? r.json() : null).then(identity => {
      if (cancelled || identity?.schema !== 1 || identity?.design !== model.design || typeof identity?.id !== 'string') return
      const nextKey = marksKey(identity.id, model.design)
      if (!nextKey) return
      let saved = new Set<string>()
      try { saved = readMarks(localStorage, nextKey, allowed) } catch { /* Session marks remain usable. */ }
      const previousKey = knownKey.current
      knownKey.current = nextKey
      key.current = nextKey
      setMarks(current => {
        const next = previousKey === nextKey ? new Set(current) : saved
        for (const [path, marked] of edits.current) { if (marked) next.add(path); else next.delete(path) }
        edits.current.clear()
        try { writeMarks(localStorage, nextKey, next) } catch { /* Storage can be unavailable. */ }
        return next
      })
    }).catch(() => {})
    return () => { cancelled = true }
  }, [model])
  const toggle = (path: string) => setMarks(current => {
    const next = new Set(current)
    if (next.has(path)) next.delete(path)
    else if (next.size < 1024 && path.length <= 2048) next.add(path)
    if (!key.current) edits.current.set(path, next.has(path))
    try { writeMarks(localStorage, key.current, next) } catch { /* Storage can be unavailable. */ }
    return next
  })
  return { marks, toggle }
}
