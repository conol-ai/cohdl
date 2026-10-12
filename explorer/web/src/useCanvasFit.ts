import { useCallback, useEffect, useRef, type RefObject, type MutableRefObject } from 'react'
import type { ReactFlowInstance } from '@xyflow/react'
import { resizeViewport, validViewport, type CanvasSize, type CircuitViewport, type LayoutRect } from './viewport'

function intersects(rect: LayoutRect, viewport: CircuitViewport, size: CanvasSize) {
  return rect.x * viewport.zoom + viewport.x < size.width && (rect.x + rect.width) * viewport.zoom + viewport.x > 0
    && rect.y * viewport.zoom + viewport.y < size.height && (rect.y + rect.height) * viewport.zoom + viewport.y > 0
}

export interface ViewportSnapshot { viewport: CircuitViewport; size: CanvasSize; autoFit: boolean }

export function useCanvasFit(canvas: RefObject<HTMLDivElement>, flow: MutableRefObject<ReactFlowInstance | null>, active: boolean) {
  const bounds = useRef<LayoutRect | null>(null)
  const frame = useRef<number>(0)
  const autoFit = useRef(true)
  const pending = useRef<ViewportSnapshot | null | undefined>(undefined)
  const resizeSource = useRef<ViewportSnapshot | null>(null)
  const focus = useRef<LayoutRect | null>(null)
  const widthFit = useRef(false)
  const primary = useRef<LayoutRect | null>(null)
  const content = useRef<LayoutRect[]>([])
  const selectedTarget = useRef<LayoutRect | null>(null)
  const setSelectionTarget = useCallback((rect: LayoutRect | null) => { selectedTarget.current = rect }, [])
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
      const restore = (saved: ViewportSnapshot) => {
        const viewport = resizeViewport(saved.viewport, saved.size, next)
        const visible = content.current.some(rect => intersects(rect, viewport, next))
        const target = selectedTarget.current
        if (visible && target && !intersects(target, viewport, next)) {
          if ((target.x + target.width) * viewport.zoom + viewport.x <= 0) viewport.x = 16 - target.x * viewport.zoom
          else if (target.x * viewport.zoom + viewport.x >= next.width) viewport.x = next.width - 16 - (target.x + target.width) * viewport.zoom
          if ((target.y + target.height) * viewport.zoom + viewport.y <= 0) viewport.y = 16 - target.y * viewport.zoom
          else if (target.y * viewport.zoom + viewport.y >= next.height) viewport.y = next.height - 16 - (target.y + target.height) * viewport.zoom
          void flow.current!.setViewport(viewport, { duration: 0 })
        } else if (visible || !content.current.length) void flow.current!.setViewport(viewport, { duration: 0 })
        else { autoFit.current = true; focus.current = selectedTarget.current ?? primary.current }
      }
      if (pending.current) {
        const saved = pending.current
        autoFit.current = saved.autoFit
        if (!saved.autoFit && validViewport(saved.viewport))
          restore(saved)
      } else if (focus.current) {
        autoFit.current = false
        void flow.current.fitBounds(focus.current, { duration: 0, padding: 0.2 })
      } else if (!autoFit.current && resizeSource.current) {
        const saved = resizeSource.current
        restore(saved)
      }
      resizeSource.current = null
      pending.current = undefined
      if (autoFit.current) {
        if (focus.current) void flow.current.fitBounds(focus.current, { duration: 0, padding: 0.2 })
        else if (widthFit.current) {
          const zoom = Math.min(2, next.width * 0.9 / bounds.current.width)
          void flow.current.setViewport({ x: (next.width - bounds.current.width * zoom) / 2 - bounds.current.x * zoom,
            y: 16 - bounds.current.y * zoom, zoom }, { duration: 0 })
        } else {
          const b = bounds.current
          const zoom = Math.min(2, Math.max(content.current.length <= 14 ? 1 : 0.9, Math.min(next.width / (b.width * 1.32), next.height / (b.height * 1.32))))
          const target = selectedTarget.current ?? primary.current ?? b
          const fits = b.width * zoom <= next.width && b.height * zoom <= next.height
          void flow.current.setViewport({ x: fits ? (next.width - b.width * zoom) / 2 - b.x * zoom : 24 - target.x * zoom, y: fits ? (next.height - b.height * zoom) / 2 - b.y * zoom : 20 - target.y * zoom, zoom }, { duration: 0 })
        }
      }
      focus.current = null
      size.current = next
    })
  }, [canvas, flow, currentSize])
  const fit = useCallback(() => { autoFit.current = false; widthFit.current = false; pending.current = undefined; resizeSource.current = null; focus.current = bounds.current; apply() }, [apply])
  const prepare = useCallback((saved?: ViewportSnapshot | null) => {
    cancelAnimationFrame(frame.current)
    generation.current++
    resizeSource.current = null
    focus.current = null
    selectedTarget.current = null
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
  const setBounds = useCallback((value: LayoutRect | null, expected: number, target: LayoutRect | null = null, fitWidth = false, rects: LayoutRect[] = [], primaryTarget: LayoutRect | null = null, selected: LayoutRect | null = null) => {
    if (expected !== generation.current) return
    bounds.current = value; content.current = rects.length ? rects : value ? [value] : []; primary.current = primaryTarget; selectedTarget.current = selected; focus.current = autoFit.current ? target : null; widthFit.current = fitWidth; apply()
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
  return { fit, setBounds, setSelectionTarget, prepare, capture, manual, onMoveStart, onReady: apply, generation: generation.current }
}
