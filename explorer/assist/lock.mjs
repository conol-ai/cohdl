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
  const token = randomUUID()
  claimLock(directory, path, token)
  try { return await action() }
  finally {
    check(!lstatSync(path).isSymbolicLink(), 'Mutation lock changed to symlink')
    const current = JSON.parse(readFileSync(path, 'utf8'))
    if (current.token === token) rmSync(path)
  }
}

function claimLock(directory, path, token) {
  const guard = join(directory, '.mutation.guard')
  let fd
  try { fd = openSync(guard, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, 0o600) }
  catch (error) {
    if (error.code === 'EEXIST') throw new Error('Mutation recovery guard is occupied; an interrupted reclamation requires owner verification and explicit guard cleanup')
    throw error
  }
  try {
    writeFileSync(fd, JSON.stringify({ pid: process.pid, token, created_at: now() }))
    // Serialize stale-owner inspection, retirement and new incarnation creation.
    if (existsSync(path)) retireDeadLock(path)
    const owner = openSync(path, constants.O_WRONLY | constants.O_CREAT | constants.O_EXCL | constants.O_NOFOLLOW, 0o600)
    try { writeFileSync(owner, JSON.stringify({ pid: process.pid, token, created_at: now() })) }
    finally { closeSync(owner) }
  } finally { closeSync(fd); rmSync(guard) }
}
