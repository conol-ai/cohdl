import { useEffect, useRef, useState } from 'react'
import { createRoot } from 'react-dom/client'
import { flushSync } from 'react-dom'
import { ReactFlow, type ReactFlowInstance } from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useCanvasFit } from '../src/useCanvasFit'

const nodes = [{ id: 'actual-flow', position: { x: 0, y: 0 }, data: { label: 'Camera regression' } }]
const bounds = { x: 0, y: 0, width: 600, height: 400 }
function Fixture() {
  const canvas = useRef<HTMLDivElement>(null)
  const flow = useRef<ReactFlowInstance | null>(null)
  const [height, setHeight] = useState(520)
  const [revision, setRevision] = useState(0)
  const fit = useCanvasFit(canvas, flow, true)
  useEffect(() => { fit.setBounds(bounds, fit.generation) }, [revision, fit.setBounds, fit.generation])
  Object.assign(window, {
    captureCamera: fit.capture,
    async setManualCamera() { await flow.current!.setViewport({ x: 273, y: -246, zoom: 2 }, { duration: 0 }); fit.manual(); return fit.capture() },
    queueCameraRestore(snapshot: ReturnType<typeof fit.capture>) {
      fit.prepare(snapshot)
      const captured = fit.capture()
      flushSync(() => setRevision(value => value + 1))
      return captured
    },
    resizeAndRestore(nextHeight = 496) {
      flushSync(() => setHeight(nextHeight))
      const snapshot = fit.capture()
      fit.prepare(snapshot)
      flushSync(() => setRevision(value => value + 1))
      return snapshot
    },
  })
  return <div ref={canvas} style={{ width: 950, height }}><ReactFlow defaultNodes={nodes} onInit={instance => { flow.current = instance; fit.onReady() }} minZoom={0.002} maxZoom={2} /></div>
}
createRoot(document.getElementById('root')!).render(<Fixture />)
