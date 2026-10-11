export interface LayoutRect { x: number; y: number; width: number; height: number }
export interface CircuitViewport { x: number; y: number; zoom: number }
export interface CanvasSize { width: number; height: number }
export interface BoardViewportSnapshot { view: LayoutRect; size: CanvasSize; autoFit: boolean }

/** SVG uses a centered, aspect-preserving viewBox; retain its actual screen scale. */
export function resizeBoardView(view: LayoutRect, previous: CanvasSize, next: CanvasSize): LayoutRect {
  if (previous.width === next.width && previous.height === next.height) return view
  const scale = Math.min(previous.width / view.width, previous.height / view.height)
  if (!(scale > 0) || !Number.isFinite(scale) || !next.width || !next.height) return view
  const width = next.width / scale, height = next.height / scale
  return { x: view.x + (view.width - width) / 2, y: view.y + (view.height - height) / 2, width, height }
}

export function validViewport(value: unknown): value is CircuitViewport {
  if (!value || typeof value !== 'object') return false
  const viewport = value as CircuitViewport
  return [viewport.x, viewport.y, viewport.zoom].every(Number.isFinite) && viewport.zoom >= 0.002 && viewport.zoom <= 2
}

export function resizeViewport(viewport: CircuitViewport, previous: CanvasSize, next: CanvasSize): CircuitViewport {
  return { x: viewport.x + (next.width - previous.width) / 2, y: viewport.y + (next.height - previous.height) / 2, zoom: viewport.zoom }
}

export function boundsOfRects(rects: Iterable<LayoutRect>): LayoutRect | null {
  let bounds: LayoutRect | null = null
  for (const rect of rects) {
    const right = rect.x + rect.width
    const bottom = rect.y + rect.height
    if (![rect.x, rect.y, rect.width, rect.height, right, bottom].every(Number.isFinite) || rect.width < 0 || rect.height < 0)
      throw new Error('Invalid layout bounds')
    if (!bounds) { bounds = { ...rect }; continue }
    const x = Math.min(bounds.x, rect.x)
    const y = Math.min(bounds.y, rect.y)
    bounds = { x, y, width: Math.max(bounds.x + bounds.width, right) - x, height: Math.max(bounds.y + bounds.height, bottom) - y }
    if (!Number.isFinite(bounds.width) || !Number.isFinite(bounds.height)) throw new Error('Invalid layout bounds')
  }
  return bounds
}
