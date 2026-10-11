import assert from 'node:assert/strict'
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { bounds, runtime, sha } from './multi-tab-runtime.mjs'
import { ready, transitions } from './multi-tab-transitions.mjs'

const option = (name, fallback) => { const index = process.argv.indexOf(name); return index < 0 ? fallback : process.argv[index + 1] ?? fallback }
const here = dirname(fileURLToPath(import.meta.url)), code = resolve(here, '../../..')
const sourceProject = resolve(option('--project', join(code, '../cohdl-doc-explorer-usability/explorer/runtime/PDFExact')))
const output = resolve(option('--output', `/tmp/cohdl-multitab-${Date.now()}`))
mkdirSync(output, { recursive: true })
const report = { result: 'RUNNING', started_at: new Date().toISOString(), bounds, fixture: 'Copied actual PDFExact 233-part/26-group project; native compiler/HTTP/SSE and real Chromium in one shared browser context',
  source_sha256: Object.fromEntries(['multi-tab-smoke.mjs', 'multi-tab-runtime.mjs', 'multi-tab-transitions.mjs'].map(name => [name, sha(readFileSync(join(here, name)))])), tabs: [], transitions: [] }
const save = () => writeFileSync(join(output, 'result.json'), JSON.stringify(report, null, 2) + '\n')
const owned = runtime({ code, sourceProject, output, report }), tabs = []
let timer, interrupted = false
async function run() {
  await owned.start()
  const configuredGroups = JSON.parse(readFileSync(join(owned.project, 'views/PDFExact.view.json'))).views.find(view => view.name === 'Functional').regions.map(region => region.name).sort()
  assert.equal(configuredGroups.length, 26)
  const context = owned.browser.defaultBrowserContext()
  report.single_browser_context = true
  for (let index = 1; index <= 8; index++) {
    const page = await context.newPage()
    const record = { index, started_at: new Date().toISOString(), requests: [], responses: [], errors: [], body_errors: [], models: [] }
    report.tabs.push(record)
    const tab = { page, record, latest: { model: null, registry: null } }; tabs.push(tab)
    page.setDefaultTimeout(bounds.request_ms); page.setDefaultNavigationTimeout(bounds.request_ms)
    await page.setViewport({ width: 1280, height: 800 })
    await page.evaluateOnNewDocument(() => {
      window.__multitab = { messages: [], opens: [], reconnects: [], parse_errors: [] }
      const NativeEventSource = window.EventSource
      window.EventSource = class extends NativeEventSource {
        constructor(...args) {
          super(...args)
          this.addEventListener('open', () => window.__multitab.opens.push(performance.now()))
          this.addEventListener('error', () => window.__multitab.reconnects.push({ at: performance.now(), state: this.readyState }))
          this.addEventListener('message', event => {
            try { window.__multitab.messages.push({ version: JSON.parse(event.data).version, at: performance.now() }) }
            catch (error) { window.__multitab.parse_errors.push(String(error)) }
          })
        }
      }
    })
    page.on('pageerror', error => record.errors.push(error.message))
    page.on('request', request => {
      const path = new URL(request.url()).pathname
      if (path.startsWith('/api/') || path.startsWith('/views/')) record.requests.push({ path, at: new Date().toISOString() })
    })
    page.on('response', response => {
      const path = new URL(response.url()).pathname
      if (!path.startsWith('/api/') && !path.startsWith('/views/')) return
      record.responses.push({ path, status: response.status(), at: new Date().toISOString(), connection: response.headers().connection, length: response.headers()['content-length'] })
      if (!['/api/model', '/api/documents'].includes(path) || response.status() !== 200) return
      void response.json().then(body => {
        if (path === '/api/documents') { tab.latest.registry = body; return }
        tab.latest.model = body
        record.models.push({ at: new Date().toISOString(), sha256: sha(JSON.stringify(body)), instances: body.instances.length, verdict: body.verdict, live_error: body.live_error ?? null })
      }).catch(error => record.body_errors.push({ path, error: String(error) }))
    })
    await page.goto(owned.url, { waitUntil: 'domcontentloaded' })
    await ready(tab)
    record.initial = await page.evaluate(() => ({ group_controls: document.querySelectorAll('.group-selector input').length,
      modules: [...document.querySelectorAll('.react-flow__node[data-id^="group:"]')].map(node => JSON.parse(decodeURIComponent(node.dataset.id.slice(6)))[1]).sort(), parts: document.querySelector('.inspection-counts').dataset.totalParts,
      busy: document.querySelector('.explorer-canvas').getAttribute('aria-busy') }))
    assert.deepEqual(record.initial.modules, configuredGroups); assert.equal(record.initial.parts, '233'); assert.equal(record.initial.busy, 'false')
    for (const path of ['/api/model', '/api/project', '/api/documents', '/views/PDFExact.view.json']) assert.ok(record.responses.some(response => response.path === path && response.status === 200), `Tab ${index}: ${path} did not progress`)
    record.events = await page.evaluate(() => window.__multitab)
    await page.screenshot({ path: join(output, `tab-${index}.png`) })
    record.ready_at = new Date().toISOString()
    save()
  }
  const file = join(owned.project, 'explorer/documents/original-schematic.pdf'), expectedFile = sha(readFileSync(file))
  for (const tab of tabs) {
    const actual = await tab.page.evaluate(async path => {
      const response = await fetch(`/api/file?p=${encodeURIComponent(path)}`, { signal: AbortSignal.timeout(6000) })
      const bytes = await response.arrayBuffer()
      return { status: response.status, type: response.headers.get('content-type'), sha256: [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map(byte => byte.toString(16).padStart(2, '0')).join('') }
    }, file)
    assert.deepEqual(actual, { status: 200, type: 'application/pdf', sha256: expectedFile })
    tab.record.file = actual
  }
  await transitions(tabs, { project: owned.project, output, report, save })
  for (const tab of tabs) assert.deepEqual(tab.record.errors, [], `Tab ${tab.record.index} runtime errors`)
  report.result = 'PASS'
}
async function interrupt(signal) {
  if (interrupted) return
  interrupted = true; report.result = 'INTERRUPTED'; report.signal = signal
  await owned.cleanup(); report.finished_at = new Date().toISOString(); save(); process.exit(130)
}
process.once('SIGINT', () => { void interrupt('SIGINT') }); process.once('SIGTERM', () => { void interrupt('SIGTERM') })
try {
  await Promise.race([run(), new Promise((_, fail) => { timer = setTimeout(() => fail(Error(`Overall timeout ${bounds.overall_ms}ms`)), bounds.overall_ms) })])
} catch (error) {
  report.result = 'FAIL'; report.error = String(error); report.error_stack = error.stack; process.exitCode = 1
  for (const tab of tabs) if (!tab.page.isClosed()) {
    tab.record.failure_ui = await tab.page.evaluate(() => ({ text: document.body.innerText.slice(0, 2000), groups: document.querySelectorAll('.group-selector input').length, busy: document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') })).catch(() => null)
    await tab.page.screenshot({ path: join(output, `tab-${tab.record.index}-failure.png`) }).catch(() => {})
  }
} finally {
  clearTimeout(timer)
  try { await owned.cleanup() } catch (error) { report.result = 'FAIL'; report.cleanup_error = String(error); process.exitCode = 1 }
  report.finished_at = new Date().toISOString(); save()
  console.log(JSON.stringify({ result: report.result, output, bounds, tabs: report.tabs.length, transitions: report.transitions.length, cleanup: report.cleanup, error: report.error }))
}
