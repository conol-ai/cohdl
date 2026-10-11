import { spawn } from 'node:child_process'
import { now } from './io.mjs'

export function runProcess(command, { cwd, input, timeout = 180000, limit = 2 * 1024 * 1024, signal } = {}) {
  return new Promise((resolve) => {
    const evidence = { command, cwd, started_at: now(), finished_at: null, exit_code: null, signal: null, error: null }
    const chunks = { stdout: [], stderr: [] }
    let size = 0
    let killTimer
    const child = spawn(command[0], command.slice(1), { cwd, shell: false, detached: process.platform !== 'win32', stdio: ['pipe', 'pipe', 'pipe'] })
    const kill = (value) => {
      try { if (process.platform === 'win32') child.kill(value); else process.kill(-child.pid, value) } catch (error) { if (error.code !== 'ESRCH') evidence.error ??= error.message }
    }
    const stop = (message) => {
      if (evidence.error !== null) return
      evidence.error = message
      kill('SIGTERM')
      killTimer = setTimeout(() => kill('SIGKILL'), 250)
    }
    const capture = (name, chunk) => {
      const allowed = Math.max(0, limit - size)
      chunks[name].push(chunk.subarray(0, allowed))
      size += chunk.length
      if (size > limit) stop('Subprocess output limit exceeded')
    }
    child.stdout.on('data', (chunk) => capture('stdout', chunk))
    child.stderr.on('data', (chunk) => capture('stderr', chunk))
    child.on('error', (error) => { evidence.error ??= error.message })
    child.stdin.on('error', (error) => { if (error.code !== 'EPIPE') stop(error.message) })
    const abort = () => stop('Subprocess cancelled')
    signal?.addEventListener('abort', abort, { once: true })
    const timer = setTimeout(() => stop('Subprocess timeout exceeded'), timeout)
    child.on('close', (code, value) => {
      clearTimeout(timer)
      clearTimeout(killTimer)
      signal?.removeEventListener('abort', abort)
      evidence.finished_at = now()
      evidence.exit_code = code
      evidence.signal = value
      resolve({ evidence, stdout: Buffer.concat(chunks.stdout), stderr: Buffer.concat(chunks.stderr) })
    })
    if (signal?.aborted) abort()
    child.stdin.end(input)
  })
}
