export interface LayoutRect { x: number; y: number; width: number; height: number }

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
