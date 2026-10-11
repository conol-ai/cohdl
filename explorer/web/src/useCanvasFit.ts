import { useCallback, useEffect, useRef, type RefObject, type MutableRefObject } from 'react'
import type { ReactFlowInstance } from '@xyflow/react'
import type { LayoutRect } from './viewport'

export function useCanvasFit(canvas: RefObject<HTMLDivElement>, flow: MutableRefObject<ReactFlowInstance | null>, active: boolean) {
  const bounds = useRef<LayoutRect | null>(null)
  const frame = useRef<number>(0)
  const fit = useCallback(() => {
    cancelAnimationFrame(frame.current)
    frame.current = requestAnimationFrame(() => {
      if (bounds.current && canvas.current?.clientWidth && canvas.current.clientHeight)
        void flow.current?.fitBounds(bounds.current, { duration: 0, padding: 0.16 })
    })
  }, [canvas, flow])
  const setBounds = useCallback((value: LayoutRect | null) => { bounds.current = value; fit() }, [fit])
  useEffect(() => {
    if (!active) return
    const observer = new ResizeObserver(fit)
    if (canvas.current) observer.observe(canvas.current)
    return () => { observer.disconnect(); cancelAnimationFrame(frame.current) }
  }, [canvas, fit, active])
  return { fit, setBounds }
}
