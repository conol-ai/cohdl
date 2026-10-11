import { closeSync, constants, existsSync, lstatSync, mkdirSync, openSync, readFileSync, rmSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { randomUUID } from 'node:crypto'
import { check, confined, now } from './io.mjs'

function retireDeadLock(path) {
  check(!lstatSync(path).isSymbolicLink() && lstatSync(path).isFile() && lstatSync(path).size <= 4096, 'Invalid mutation lock')
  const previous = JSON.parse(readFileSync(path, 'utf8'))
  check(Number.isSafeInteger(previous.pid) && previous.pid > 0, 'Invalid mutation lock owner')
  try { process.kill(previous.pid, 0) } catch (error) {
    check(error.code === 'ESRCH', 'Mutation lock owner cannot be checked')
    rmSync(path)
    return
  }
  throw new Error('Another live CLI mutation holds the project lock')
}

export async function locked(project, action) {
  const directory = confined(project, 'explorer/proposals', { missing: true })
  mkdirSync(directory, { recursive: true })
  confined(project, 'explorer/proposals')
  const path = join(directory, '.mutation.lock')
  if (existsSync(path)) retireDeadLock(path)
  const token = randomUUID()
  const fd = openSync(path, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, 0o600)
  writeFileSync(fd, JSON.stringify({ pid: process.pid, token, created_at: now() }))
  closeSync(fd)
  try { return await action() }
  finally {
    check(!lstatSync(path).isSymbolicLink(), 'Mutation lock changed to symlink')
    const current = JSON.parse(readFileSync(path, 'utf8'))
    if (current.token === token) rmSync(path)
  }
}
