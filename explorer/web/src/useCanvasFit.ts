import { useCallback, useEffect, useRef, type RefObject, type MutableRefObject } from 'react'
import type { ReactFlowInstance } from '@xyflow/react'
import { resizeViewport, validViewport, type CanvasSize, type CircuitViewport, type LayoutRect } from './viewport'

export interface ViewportSnapshot { viewport: CircuitViewport; size: CanvasSize; autoFit: boolean }

export function useCanvasFit(canvas: RefObject<HTMLDivElement>, flow: MutableRefObject<ReactFlowInstance | null>, active: boolean) {
  const bounds = useRef<LayoutRect | null>(null)
  const frame = useRef<number>(0)
  const autoFit = useRef(true)
  const pending = useRef<ViewportSnapshot | null | undefined>(undefined)
  const resizeSource = useRef<ViewportSnapshot | null>(null)
  const focus = useRef<LayoutRect | null>(null)
  const widthFit = useRef(false)
  const generation = useRef(0)
  const size = useRef<CanvasSize>({ width: 0, height: 0 })
  const currentSize = useCallback(() => {
    const rect = canvas.current?.getBoundingClientRect()
    return { width: rect?.width ?? 0, height: rect?.height ?? 0 }
  }, [canvas])
  const apply = useCallback(() => {
    cancelAnimationFrame(frame.current)
    frame.current = requestAnimationFrame(() => {
      if (!bounds.current || !flow.current || !canvas.current?.clientWidth || !canvas.current.clientHeight) return
      const next = currentSize()
      if (pending.current) {
        const saved = pending.current
        autoFit.current = saved.autoFit
        if (!saved.autoFit && validViewport(saved.viewport))
          void flow.current.setViewport(resizeViewport(saved.viewport, saved.size, next), { duration: 0 })
      } else if (focus.current) {
        autoFit.current = false
        void flow.current.fitBounds(focus.current, { duration: 0, padding: 0.2 })
      } else if (!autoFit.current && resizeSource.current) {
        const saved = resizeSource.current
        void flow.current.setViewport(resizeViewport(saved.viewport, saved.size, next), { duration: 0 })
      }
      resizeSource.current = null
      focus.current = null
      pending.current = undefined
      if (autoFit.current) {
        if (widthFit.current) {
          const zoom = Math.min(2, next.width * 0.9 / bounds.current.width)
          void flow.current.setViewport({ x: (next.width - bounds.current.width * zoom) / 2 - bounds.current.x * zoom,
            y: 16 - bounds.current.y * zoom, zoom }, { duration: 0 })
        } else void flow.current.fitBounds(bounds.current, { duration: 0, padding: 0.16 })
      }
      size.current = next
    })
  }, [canvas, flow, currentSize])
  const fit = useCallback(() => { autoFit.current = true; widthFit.current = false; pending.current = undefined; resizeSource.current = null; focus.current = null; apply() }, [apply])
  const prepare = useCallback((saved?: ViewportSnapshot | null) => {
    cancelAnimationFrame(frame.current)
    generation.current++
    resizeSource.current = null
    focus.current = null
    pending.current = saved ?? null
    autoFit.current = saved?.autoFit ?? true
    bounds.current = null
  }, [])
  const capture = useCallback((): ViewportSnapshot | null => {
    const queued = pending.current ?? resizeSource.current
    if (queued) return { viewport: { ...queued.viewport }, size: { ...queued.size }, autoFit: queued.autoFit }
    const viewport = flow.current?.getViewport()
    const appliedSize = size.current.width && size.current.height ? size.current : currentSize()
    return viewport && validViewport(viewport) ? { viewport: { ...viewport }, size: { ...appliedSize }, autoFit: autoFit.current } : null
  }, [flow, currentSize])
  const manual = useCallback(() => { autoFit.current = false; pending.current = undefined; resizeSource.current = null; focus.current = null; size.current = currentSize(); cancelAnimationFrame(frame.current) }, [currentSize])
  const onMoveStart = useCallback((event: MouseEvent | TouchEvent | null) => {
    if (event) manual()
  }, [manual])
  const setBounds = useCallback((value: LayoutRect | null, expected: number, target: LayoutRect | null = null, fitWidth = false) => {
    if (expected !== generation.current) return
    bounds.current = value; focus.current = autoFit.current ? target : null; widthFit.current = fitWidth; apply()
  }, [apply])
  useEffect(() => {
    if (!active) return
    const observer = new ResizeObserver(() => {
      const next = currentSize()
      if (pending.current !== undefined || autoFit.current) { if (bounds.current) apply(); return }
      if (flow.current && size.current.width && size.current.height) {
        resizeSource.current ??= { viewport: flow.current.getViewport(), size: { ...size.current }, autoFit: false }
        apply()
      } else size.current = next
    })
    if (canvas.current) observer.observe(canvas.current)
    return () => { observer.disconnect(); cancelAnimationFrame(frame.current) }
  }, [canvas, flow, apply, active, currentSize])
  return { fit, setBounds, prepare, capture, manual, onMoveStart, onReady: apply, generation: generation.current }
}
