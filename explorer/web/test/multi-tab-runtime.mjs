import assert from 'node:assert/strict'
import { spawn, spawnSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import { cpSync, createWriteStream, existsSync, lstatSync, mkdtempSync, readFileSync, readdirSync, rmSync } from 'node:fs'
import { createServer } from 'node:net'
import { tmpdir } from 'node:os'
import { basename, join, relative } from 'node:path'
import puppeteer from 'puppeteer-core'
import { resolveChromeExecutablePath } from './browser-launch.mjs'

export const sha = bytes => createHash('sha256').update(bytes).digest('hex')
export const pause = ms => new Promise(done => setTimeout(done, ms))
export const bounds = { startup_ms: 15000, request_ms: 6000, readiness_ms: 6000, transition_ms: 30000, overall_ms: 180000 }
export function snapshot(root) {
  const files = {}
  function walk(path) {
    if (basename(path).startsWith('.')) return
    const stat = lstatSync(path)
    assert.equal(stat.isSymbolicLink(), false, `Fixture must not follow symlinks: ${path}`)
    if (stat.isDirectory()) { for (const name of readdirSync(path).sort()) walk(join(path, name)); return }
    if (stat.isFile()) files[relative(root, path)] = sha(readFileSync(path))
  }
  walk(root)
  return files
}
export async function poll(check, label, timeout = bounds.transition_ms) {
  const deadline = Date.now() + timeout
  let latest
  while (Date.now() < deadline) {
    latest = await check()
    if (latest) return latest
    await pause(100)
  }
  throw Error(`${label} timed out after ${timeout}ms; last result: ${JSON.stringify(latest)}`)
}
async function freePort() {
  const listener = createServer()
  await new Promise((done, fail) => { listener.once('error', fail); listener.listen(0, '127.0.0.1', done) })
  const port = listener.address().port
  await new Promise(done => listener.close(done))
  assert.ok(![5199, 5209, 5210].includes(port))
  return port
}
async function stop(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) return
  const exited = new Promise(done => child.once('exit', done))
  child.kill('SIGTERM')
  await Promise.race([exited, pause(2000)])
  if (child.exitCode === null && child.signalCode === null) { child.kill('SIGKILL'); await Promise.race([exited, pause(2000)]) }
  assert.ok(child.exitCode !== null || child.signalCode !== null, 'Owned child did not stop')
}
export function runtime({ code, sourceProject, output, report }) {
  const scratch = mkdtempSync(join(tmpdir(), 'cohdl-multitab-'))
  const project = join(scratch, 'project'), binary = join(scratch, 'cohdl-explorer'), dist = join(code, 'explorer/web/dist')
  let child, browser, launch, log, cleanupPromise, stopped = false
  const owned = { project, url: '', browser: null, async start() {
    report.input_project = { path: sourceProject, sha256: snapshot(sourceProject) }
    cpSync(sourceProject, project, { recursive: true, filter: path => !basename(path).startsWith('.') })
    const originalBinary = join(code, 'explorer/extractor/target/debug/cohdl-explorer')
    cpSync(originalBinary, binary)
    report.binary = { path: originalBinary, sha256: sha(readFileSync(binary)) }
    report.dist_sha256 = snapshot(dist)
    report.code_head = spawnSync('git', ['rev-parse', 'HEAD'], { cwd: code, encoding: 'utf8' }).stdout.trim()
    const port = await freePort()
    assert.equal(stopped, false, 'Interrupted before native launch')
    owned.url = `http://127.0.0.1:${port}`
    const argv = [project, '--serve', '--dist', dist, '--port', String(port)]
    log = createWriteStream(join(output, 'native.log'))
    child = spawn(binary, argv, { env: { ...process.env, COHDL_LIB: join(code, 'lib') }, stdio: ['ignore', 'pipe', 'pipe'] })
    report.child = { pid: child.pid, argv, started_at: new Date().toISOString(), port }
    child.stdout.pipe(log, { end: false }); child.stderr.pipe(log, { end: false })
    child.once('error', error => { report.child.error = String(error) })
    child.once('exit', (status, signal) => Object.assign(report.child, { status, signal, ended_at: new Date().toISOString() }))
    await poll(async () => {
      assert.equal(report.child.error, undefined)
      assert.equal(stopped, false, 'Interrupted during native startup')
      assert.equal(child.exitCode, null, 'Owned native server exited during startup')
      try { const response = await fetch(`${owned.url}/api/model`, { signal: AbortSignal.timeout(1000) }); return response.ok }
      catch { return false }
    }, 'Native HTTP readiness', bounds.startup_ms)
    launch = puppeteer.launch({ executablePath: resolveChromeExecutablePath(), headless: true, timeout: bounds.startup_ms }).then(value => {
      browser = value; owned.browser = value
      report.browser_pid = value.process().pid
      return value
    })
    await launch
  }, cleanup() {
    stopped = true
    cleanupPromise ??= (async () => {
      if (launch) await launch.catch(() => {})
      const close = browser ? await Promise.allSettled([Promise.race([browser.close(), pause(3000)])]) : []
      const outcomes = [...close, ...await Promise.allSettled([stop(browser?.process()), stop(child)])]
      if (log) await new Promise(done => log.end(done))
      try {
        report.input_project_unchanged = report.input_project ? JSON.stringify(snapshot(sourceProject)) === JSON.stringify(report.input_project.sha256) : true
      } finally {
        rmSync(scratch, { recursive: true, force: true })
        report.cleanup = { browser_closed: !browser?.connected, owned_server_exited: !child || child.exitCode !== null || child.signalCode !== null,
          temporary_root_removed: !existsSync(scratch), protected_ports_not_owned: [5199, 5209, 5210] }
      }
      assert.ok(report.input_project_unchanged, 'Original fixture changed')
      for (const outcome of outcomes) if (outcome.status === 'rejected') throw outcome.reason
    })()
    return cleanupPromise
  } }
  return owned
}
