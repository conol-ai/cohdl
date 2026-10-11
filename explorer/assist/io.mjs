import { closeSync, constants, existsSync, fstatSync, fsyncSync, lstatSync, mkdirSync, openSync, readSync, realpathSync, renameSync, rmSync, writeFileSync } from 'node:fs'
import { createHash, randomUUID } from 'node:crypto'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'

export const JSON_LIMIT = 8 * 1024 * 1024
export const DOCUMENT_LIMIT = 64 * 1024 * 1024
export const sha = (bytes) => createHash('sha256').update(bytes).digest('hex')
export const now = () => new Date().toISOString()
export const check = (condition, message) => { if (!condition) throw new Error(message) }
export const hash = (value) => typeof value === 'string' && /^[a-f0-9]{64}$/.test(value)
export const object = (value) => value !== null && typeof value === 'object' && !Array.isArray(value)
export const text = (bytes) => new TextDecoder('utf-8', { fatal: true }).decode(bytes)

export function relativePath(value) {
  check(typeof value === 'string' && value.length > 0 && value.length <= 512 && /^[\x20-\x7e]+$/.test(value), 'Invalid bounded ASCII path')
  check(!isAbsolute(value) && !value.includes('\\') && !value.includes(':') && !value.includes('%'), 'Unsafe relative path')
  check(value.split('/').every((part) => part && part !== '..' && !part.startsWith('.') && !/^(?:env|secrets?|credentials?|id_rsa|id_ed25519)(?:[._-]|$)/i.test(part)), 'Hidden, secret or traversal path rejected')
  return value
}

export function confined(root, path, { missing = false } = {}) {
  relativePath(path)
  const base = realpathSync(root)
  let cursor = base
  for (const part of path.split('/')) {
    cursor = join(cursor, part)
    if (!existsSync(cursor)) {
      // existsSync follows broken links, so lstat still gets a chance to reject them.
      try { check(!lstatSync(cursor).isSymbolicLink(), 'Symlink path rejected') } catch (error) { if (error.code !== 'ENOENT') throw error }
      check(missing, `Missing input: ${path}`)
      continue
    }
    check(!lstatSync(cursor).isSymbolicLink(), `Symlink path rejected: ${path}`)
  }
  check(relative(base, cursor) === path, 'Path confinement failed')
  return cursor
}

export function read(root, path, limit = JSON_LIMIT, missing = false) {
  const file = confined(root, path, { missing })
  let fd
  try {
    fd = openSync(file, constants.O_RDONLY | constants.O_NOFOLLOW)
    const stat = fstatSync(fd)
    check(stat.isFile() && stat.size <= limit, `Input is not a bounded regular file: ${path}`)
    const chunks = []
    let size = 0
    while (size <= limit) {
      const chunk = Buffer.alloc(Math.min(65536, limit + 1 - size))
      const count = readSync(fd, chunk, 0, chunk.length, null)
      if (count === 0) return Buffer.concat(chunks, size)
      size += count
      check(size <= limit, `Input grew beyond byte limit: ${path}`)
      chunks.push(chunk.subarray(0, count))
    }
  } catch (error) {
    if (missing && error.code === 'ENOENT') return null
    throw error
  } finally { if (fd !== undefined) closeSync(fd) }
}

export function load(root, path, missing = false) {
  const bytes = read(root, path, JSON_LIMIT, missing)
  return bytes === null ? null : JSON.parse(text(bytes))
}

export function input(path, limit = JSON_LIMIT) {
  const absolute = resolve(path)
  return read(realpathSync(dirname(absolute)), absolute.slice(dirname(absolute).length + 1), limit)
}

export function atomic(root, path, bytes) {
  const target = confined(root, path, { missing: true })
  mkdirSync(dirname(target), { recursive: true })
  confined(root, path, { missing: true })
  const temporary = join(dirname(target), `.cohdl-assist-${randomUUID()}`)
  let fd
  try {
    fd = openSync(temporary, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL, 0o600)
    writeFileSync(fd, bytes)
    fsyncSync(fd)
    closeSync(fd)
    fd = undefined
    confined(root, path, { missing: true })
    renameSync(temporary, target)
  } finally {
    if (fd !== undefined) closeSync(fd)
    rmSync(temporary, { force: true })
  }
}

export const save = (root, path, value) => atomic(root, path, JSON.stringify(value, null, 2) + '\n')

export function output(path, bytes) {
  const absolute = resolve(path)
  const parent = realpathSync(dirname(absolute))
  check(!existsSync(absolute), `Refusing to overwrite output: ${path}`)
  atomic(parent, absolute.slice(dirname(absolute).length + 1), bytes)
}
