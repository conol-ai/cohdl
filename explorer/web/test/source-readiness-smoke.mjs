import assert from 'node:assert/strict'
import puppeteer from 'puppeteer-core'

const index = process.argv.indexOf('--url')
const base = index < 0 ? 'http://127.0.0.1:5210' : process.argv[index + 1]
const response = await fetch(`${base}/api/documents`)
assert.ok(response.ok, 'Native actual source registry is available')
const registry = await response.text()
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
try {
  const page = await browser.newPage()
  await page.setViewport({ width: 1280, height: 800 })
  let release, onHeld, rejectRegistry = false
  const heldRequest = () => new Promise(resolve => { onHeld = resolve })
  await page.setRequestInterception(true)
  page.on('request', request => {
    if (!request.url().includes('/api/documents') || request.url().includes('/api/documents/')) return request.continue()
    release = () => request.respond({ status: rejectRegistry ? 404 : 200, contentType: 'application/json', body: rejectRegistry ? '{}' : registry })
    onHeld?.()
  })
  const ready = () => page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
    && JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}').viewport !== undefined
    && document.querySelector('.react-flow__node:not(.react-flow__node-region)'))
  const capture = () => page.evaluate(() => {
    const transform = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
    return { x: transform.e, y: transform.f, zoom: transform.a }
  })
  let held = heldRequest()
  await page.goto(base, { waitUntil: 'domcontentloaded' })
  await held
  await page.waitForFunction(() => document.body.textContent.includes('Loading component information...'))
  assert.equal(await page.$eval('.explorer-canvas', element => element.getAttribute('aria-busy')), 'true')
  assert.equal(new URL(page.url()).searchParams.has('inspect'), false, 'Pending source identities must not publish an intermediate camera')
  assert.ok(release, 'Actual source request is held')
  await release()
  await ready()
  await page.click('.react-flow__controls-zoomin')
  await page.waitForFunction(() => {
    const inspect = JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}')
    const transform = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
    return inspect.viewport?.autoFit === false && Math.abs(inspect.viewport.viewport.zoom - transform.a) < .00001
  })
  const before = await capture(), saved = page.url()
  release = null
  held = heldRequest()
  await page.reload({ waitUntil: 'domcontentloaded' })
  await held
  await page.waitForFunction(() => document.body.textContent.includes('Loading component information...'))
  assert.equal(page.url(), saved, 'Reload keeps pending authoritative camera')
  assert.ok(release)
  await release()
  await ready()
  const after = await capture()
  for (const name of ['x', 'y', 'zoom']) assert.ok(Math.abs(before[name] - after[name]) < .02, `Reload ${name} stays deterministic: ${JSON.stringify({ before, after })}`)
  console.log('PASS delayed actual source identity: pending readiness and exact manual-camera reload')
  rejectRegistry = true
  held = heldRequest()
  await page.goto(base, { waitUntil: 'domcontentloaded' })
  await held
  await page.waitForFunction(() => document.body.textContent.includes('Loading component information...'))
  await release()
  await ready()
  console.log('PASS missing source registry: loading settles to honest fallback')
} finally {
  await browser.close()
}
