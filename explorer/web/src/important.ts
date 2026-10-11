const MAX_MARKS_BYTES = 65536
const MAX_MARKS = 1024

export function marksKey(project: string, design: string): string | null {
  return /^[a-f0-9]{64}$/.test(project) && design.length <= 2048 ? `cohdl:important:1:${project}:${encodeURIComponent(design)}` : null
}

export function readMarks(storage: Pick<Storage, 'getItem'>, key: string | null, allowed: ReadonlySet<string>): Set<string> {
  if (!key) return new Set()
  try {
    const text = storage.getItem(key)
    if (!text || text.length > MAX_MARKS_BYTES || new TextEncoder().encode(text).byteLength > MAX_MARKS_BYTES) return new Set()
    const parsed: unknown = JSON.parse(text)
    if (!Array.isArray(parsed) || parsed.length > MAX_MARKS) return new Set()
    return new Set(parsed.filter((p): p is string => typeof p === 'string' && p.length <= 2048 && allowed.has(p)))
  } catch { return new Set() }
}

export function writeMarks(storage: Pick<Storage, 'setItem'>, key: string | null, marks: ReadonlySet<string>): boolean {
  if (!key || marks.size > MAX_MARKS) return false
  try {
    const text = JSON.stringify([...marks])
    if (text.length > MAX_MARKS_BYTES || new TextEncoder().encode(text).byteLength > MAX_MARKS_BYTES) return false
    storage.setItem(key, text)
    return true
  } catch { return false }
}
