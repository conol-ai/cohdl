import { existsSync, readdirSync, realpathSync } from 'node:fs'
import { join } from 'node:path'
import { check, confined, DOCUMENT_LIMIT, JSON_LIMIT, load, read, relativePath, sha } from './io.mjs'

const INPUT_LIMIT = 2048
const TOTAL_LIMIT = 256 * 1024 * 1024
const compilerFile = (name) => /\.(?:cohdl|dxf)$/.test(name) || ['cohdl.toml', 'cohdl.lock', 'design.lock'].includes(name)

function walk(root, directory, paths) {
  const path = confined(root, directory, { missing: true })
  if (!existsSync(path)) return
  for (const item of readdirSync(path, { withFileTypes: true })) {
    const entry = `${directory}/${item.name}`
    check(!item.isSymbolicLink(), `Symlink input rejected: ${entry}`)
    relativePath(entry)
    if (item.isDirectory()) walk(root, entry, paths)
    else if (compilerFile(item.name)) paths.add(entry)
    check(paths.size <= INPUT_LIMIT, 'Too many compiler input files')
  }
}

export function registry(root) {
  const value = load(root, 'explorer/documents.json', true)
  if (value === null) return null
  check(value.schema === 1 && Array.isArray(value.documents) && value.documents.length <= 512 && Array.isArray(value.constraints) && value.constraints.length <= 4096, 'Invalid or oversized document registry')
  const ids = new Set()
  for (const document of value.documents) {
    check(typeof document.id === 'string' && /^[A-Za-z0-9][A-Za-z0-9_-]{0,63}$/.test(document.id) && !ids.has(document.id), 'Invalid or duplicate document identity')
    ids.add(document.id)
    check(['pdf', 'markdown'].includes(document.kind), 'Unknown document kind')
    check(document.path === null || typeof document.path === 'string', 'Invalid document path')
    if (document.path === null) continue
    relativePath(document.path)
    check(document.path.endsWith(document.kind === 'pdf' ? '.pdf' : '.md'), 'Document media extension mismatch')
  }
  return value
}

export function snapshot(project, design = null) {
  const root = realpathSync(project)
  const paths = new Set(['cohdl.toml', 'cohdl.lock', 'design.lock', 'explorer/documents.json'])
  for (const directory of ['src', 'deps']) walk(root, directory, paths)
  const views = confined(root, 'views', { missing: true })
  if (existsSync(views)) {
    for (const item of readdirSync(views, { withFileTypes: true })) {
      check(!item.isSymbolicLink(), 'Symlink view rejected')
      if (item.name.endsWith('.view.json')) paths.add(`views/${item.name}`)
    }
  }
  if (design !== null) {
    check(typeof design === 'string' && /^[A-Za-z_][A-Za-z0-9_]*$/.test(design), 'Invalid design name')
    paths.add(`views/${design}.view.json`)
  }
  const documents = registry(root)
  const assets = new Set(documents?.documents.flatMap((d) => d.path === null ? [] : [d.path]) ?? [])
  for (const path of assets) paths.add(path)
  check(paths.size <= INPUT_LIMIT, 'Too many snapshot files')
  let total = 0
  let inputTotal = 0
  return [...paths].sort().map((path) => {
    const bytes = read(root, path, assets.has(path) ? DOCUMENT_LIMIT : JSON_LIMIT, true)
    total += bytes?.length ?? 0
    if (!assets.has(path)) inputTotal += bytes?.length ?? 0
    check(total <= TOTAL_LIMIT && inputTotal <= JSON_LIMIT, 'Snapshot byte budget exceeded')
    return { path, sha256: bytes === null ? null : sha(bytes) }
  })
}

export function fresh(project, bundle, expected = bundle.snapshot) {
  check(Array.isArray(expected) && expected.length <= INPUT_LIMIT, 'Invalid baseline snapshot')
  const current = snapshot(project, bundle.design)
  check(JSON.stringify(current) === JSON.stringify(expected), 'Stale source, view, config or document snapshot')
  return current
}
