import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runImportantStorageSmoke(page, { url, output, ready, clickText }) {
  const identity = await (await fetch(new URL('/api/project', url))).json()
  const model = await (await fetch(new URL('/api/model', url))).json()
  const instance = model.instances.find(i => i.designator === 'U6')
  const key = `cohdl:important:1:${identity.id}:${encodeURIComponent(identity.design)}`
  const seed = await page.evaluateOnNewDocument(({ key, path }) => localStorage.setItem(key, JSON.stringify([path])), { key, path: instance.path })
  const pending = []
  let hold = true
  const intercept = request => {
    if (hold && new URL(request.url()).pathname === '/api/project') pending.push(request)
    else void request.continue()
  }
  await page.setRequestInterception(true); page.on('request', intercept)
  await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
  await page.type('input[aria-label="Search circuit"]', 'U6'); await page.keyboard.press('Enter'); await ready()
  assert.ok(pending.length, 'Project identity must be delayed for this real race')
  await clickText('Mark important'); await clickText('Unmark important')
  hold = false
  await Promise.all(pending.map(request => request.continue().catch(() => {})))
  await page.waitForFunction(() => [...document.querySelectorAll('button')].some(el => el.textContent.trim() === 'Important (0)'))
  await new Promise(r => setTimeout(r, 250))
  assert.deepEqual(await page.evaluate(key => JSON.parse(localStorage.getItem(key)), key), [], 'Pending unmark must not be resurrected by saved marks')
  page.off('request', intercept); await page.setRequestInterception(false)
  await page.removeScriptToEvaluateOnNewDocument(seed.identifier)

  const denied = await page.evaluateOnNewDocument(() => {
    Object.defineProperty(window, 'localStorage', { configurable: true, get() { throw new DOMException('Denied by test', 'SecurityError') } })
  })
  await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
  await page.type('input[aria-label="Search circuit"]', 'U6'); await page.keyboard.press('Enter'); await ready()
  await clickText('Mark important')
  assert.ok(await page.$('button[aria-label="Unmark important"]'), 'Denied storage still supports session marks')
  await clickText('Unmark important'); await page.removeScriptToEvaluateOnNewDocument(denied.identifier)

  await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
  await page.evaluate(({ key, path }) => localStorage.setItem(key, JSON.stringify([path])), { key, path: instance.path })
  const writesDenied = await page.evaluateOnNewDocument(() => {
    Storage.prototype.setItem = function () { throw new DOMException('Writes denied by test', 'QuotaExceededError') }
    const NativeEventSource = window.EventSource
    window.EventSource = class extends NativeEventSource { constructor(...args) { super(...args); window.__marksEventSource = this } }
  })
  const liveUrl = new URL(url); liveUrl.searchParams.delete('nolive')
  await page.goto(liveUrl.href, { waitUntil: 'domcontentloaded' }); await ready()
  await page.type('input[aria-label="Search circuit"]', 'U6'); await page.keyboard.press('Enter'); await ready()
  await page.waitForSelector('button[aria-label="Unmark important"]')
  await clickText('Unmark important')
  const refreshed = page.waitForResponse(response => new URL(response.url()).pathname === '/api/project')
  await page.evaluate(() => window.__marksEventSource.onmessage(new MessageEvent('message', { data: JSON.stringify({ version: 987654321 }) })))
  await refreshed; await new Promise(r => setTimeout(r, 300)); await ready()
  assert.equal(await page.$('button[aria-label="Unmark important"]'), null, 'A failed storage write must not resurrect an unmarked part after same-project live model refresh')
  await page.removeScriptToEvaluateOnNewDocument(writesDenied.identifier)
  await page.evaluate(key => localStorage.removeItem(key), key)
  writeFileSync(resolve(output, 'important-storage.json'), JSON.stringify({ result: 'PASS', pendingRequests: pending.length,
    delayedIdentityUnmark: 'PASS', deniedStorageSessionMarks: 'PASS', failedWriteLiveRefresh: 'PASS', liveRefreshTrigger: 'Browser event fixture invokes actual SSE handler and fetches the real native model and project identity', projectIdentity: identity }, null, 2))
}
