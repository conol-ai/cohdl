import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'
import { resolveChromeExecutablePath } from './browser-launch.mjs'

const option = (name, fallback) => process.argv.includes(name) ? process.argv[process.argv.indexOf(name) + 1] : fallback
const base = option('--url', 'http://127.0.0.1:5210')
const output = resolve(option('--output', '/tmp/cohdl-inspection-usability'))
mkdirSync(output, { recursive: true })
const model = await (await fetch(`${base}/api/model`)).json()
const registry = await (await fetch(`${base}/api/documents`)).json()
const config = await (await fetch(`${base}/views/${encodeURIComponent(model.design)}.view.json`)).json()
const browser = await puppeteer.launch({ executablePath: resolveChromeExecutablePath(), headless: true })
const results = [], errors = []
let page
const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}'))
const settle = async () => {
  await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false' && document.querySelector('.react-flow__node') && JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}').version === 2)
  await new Promise(resolve => setTimeout(resolve, 250))
}
const click = async label => {
  const element = await page.evaluateHandle(label => [...document.querySelectorAll('button')].find(button => button.getAttribute('aria-label') === label || button.textContent.trim() === label), label)
  assert.ok(element.asElement(), `Missing ${label}`)
  await element.asElement().click()
  await element.dispose()
  await settle()
}
const capture = async name => {
  const dimensions = await page.evaluate(() => ({ overflow: document.documentElement.scrollWidth > innerWidth, canvas: document.querySelector('.explorer-canvas').getBoundingClientRect().toJSON(), cards: document.querySelectorAll('.device-card').length }))
  assert.equal(dimensions.overflow, false)
  assert.ok(dimensions.canvas.width > 100 && dimensions.canvas.height > 150)
  await page.screenshot({ path: resolve(output, `${name}.png`) })
  results.push({ name, context: await context(), ...dimensions })
}
const select = async reference => {
  await page.$eval('[aria-label="Search circuit"]', (input, value) => { const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; setter.call(input, value); input.dispatchEvent(new Event('input', { bubbles: true })) }, reference)
  await page.waitForSelector('[data-search-kind="part"]')
  await page.click('[data-search-kind="part"]')
  await settle()
}
try {
  for (const viewport of [{ width: 1440, height: 1000 }, { width: 390, height: 844 }]) {
    page = await browser.newPage()
    page.on('pageerror', error => errors.push(String(error)))
    await page.setViewport(viewport)
    const prefix = viewport.width < 700 ? 'mobile' : 'desktop'
    await page.goto(base, { waitUntil: 'networkidle2' })
    await settle()
    const fresh = await context()
    assert.equal(fresh.mode, 'sch')
    assert.ok(fresh.groups.length)
    assert.deepEqual(fresh.collapsed, [])
    assert.ok(await page.$('.device-card .device-footprint'))
    assert.equal(await page.evaluate(() => [...document.querySelectorAll('button')].some(button => button.textContent.trim() === 'Layout' || button.textContent.trim() === 'Show in layout')), false)
    await capture(`${prefix}-fresh-inside`)
    if (viewport.width < 700) await click('Show Groups')
    await page.click('[aria-label="ESP32 groups"]')
    await settle()
    const selected = (await context()).groups
    const definition = config.views.find(view => view.name === fresh.view)
    const esp32 = definition.navigation.find(parent => parent.name === 'ESP32').groups
    assert.ok(esp32.every(name => selected.includes(name)))
    assert.equal(await page.$eval('[aria-label="ESP32 groups"]', input => input.checked), true)
    assert.equal(await page.$eval('[aria-label="Power groups"]', input => input.indeterminate), true)
    const cameraPower = await page.$$('.group-control[data-group="Camera power"] input')
    assert.equal(cameraPower.length, 2)
    for (const checkbox of cameraPower) assert.equal(await checkbox.evaluate(input => input.checked), true)
    const expected = new Set(definition.regions.filter(region => selected.includes(region.name)).flatMap(region => region.members).flatMap(reference => model.instances.filter(instance => instance.designator === reference || `path:${instance.path}` === reference).map(instance => instance.path)))
    assert.equal(Number(await page.$eval('.inspection-counts', element => element.dataset.parts)), expected.size)
    await click('Hide ESP32 groups')
    await page.type('[aria-label="Search groups"]', 'Camera power')
    assert.ok(await page.$('.group-branch[aria-label="ESP32"] [data-group="Camera power"]'))
    await page.$eval('[aria-label="Search groups"]', input => { const setter = Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set; setter.call(input, ''); input.dispatchEvent(new Event('input', { bubbles: true })) })
    assert.equal(await page.$('.group-branch[aria-label="ESP32"] .group-children'), null)
    if (viewport.width < 700) await page.keyboard.press('Escape')
    await capture(`${prefix}-group-tree`)
    await click('Back')
    assert.deepEqual((await context()).groups, fresh.groups)
    const legacy = new URL(base)
    legacy.searchParams.set('inspect', JSON.stringify({ ...fresh, mode: 'overview', collapsed: fresh.groups }))
    await page.goto(legacy.href, { waitUntil: 'networkidle2' })
    await settle()
    assert.equal((await context()).mode, 'overview')
    await click('View inside')
    assert.equal((await context()).mode, 'sch')
    assert.deepEqual((await context()).groups, fresh.groups)
    await capture(`${prefix}-legacy-inside`)
    await page.evaluate(() => {
      window.panelProbe = { nodes: [...document.querySelectorAll('.react-flow__node')], frames: 0, blank: 0, busy: 0, running: true }
      const sample = () => { const p = window.panelProbe; if (!p.running) return; p.frames++; if (!document.querySelector('.react-flow__node')) p.blank++; if (document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') !== 'false') p.busy++; requestAnimationFrame(sample) }
      requestAnimationFrame(sample)
    })
    const panelCamera = await page.$eval('.react-flow__viewport', element => element.getAttribute('style'))
    await click('Hide toolbar')
    assert.equal(await page.$eval('.explorer-toolbar', element => getComputedStyle(element).display), 'none')
    await capture(`${prefix}-toolbar-hidden`)
    await click('Show toolbar')
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')), 'Search circuit')
    if (viewport.width > 700) {
      await click('Hide Groups')
      assert.equal(await page.$eval('.group-panel', element => getComputedStyle(element).display), 'none')
      await capture(`${prefix}-groups-hidden`)
      await click('Show Groups')
    } else {
      await click('Show Groups')
      assert.equal(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')), 'Search groups')
      await page.keyboard.press('Escape')
      await settle()
    }
    const panelProbe = await page.evaluate(() => { const p = window.panelProbe; p.running = false; return { frames: p.frames, blank: p.blank, busy: p.busy, retained: p.nodes.every(node => node.isConnected) } })
    assert.ok(panelProbe.frames > 0)
    assert.deepEqual({ blank: panelProbe.blank, busy: panelProbe.busy, retained: panelProbe.retained }, { blank: 0, busy: 0, retained: true })
    assert.equal(await page.$eval('.react-flow__viewport', element => element.getAttribute('style')), panelCamera)
    results.push({ name: `${prefix}-panel-continuity`, ...panelProbe })
    await select('U1')
    const corePath = (await context()).selection[0]
    await page.$eval('.explorer-inspector', element => { element.scrollTop = 0 })
    await page.$eval('.part-details .pin-target[data-pin="P56"]', element => element.scrollIntoView({ block: 'center' }))
    await page.click('.part-details .pin-target[data-pin="P56"]')
    await settle()
    assert.equal((await context()).pin, 'P56')
    assert.equal(await page.$eval('.part-network', element => element.dataset.net), 'GND')
    assert.ok(await page.$('.pin-target[data-pin="P53"][data-traced="true"][data-selected="false"]'))
    const states = await page.$$eval('.part-details .pin-table tbody tr', rows => rows.map(row => row.lastElementChild.textContent))
    assert.ok(states.every(state => state !== 'exact trace'))
    assert.ok(await page.$eval('.part-network', element => !!(element.compareDocumentPosition(document.querySelector('.pin-table')) & Node.DOCUMENT_POSITION_FOLLOWING)))
    await capture(`${prefix}-network-first`)
    const camera = await page.$eval('.react-flow__viewport', element => element.getAttribute('style'))
    for (const pin of ['P56', 'P53', 'P56']) {
      await page.$eval(`.part-details .pin-target[data-pin="${pin}"]`, element => element.scrollIntoView({ block: 'center' }))
      await page.click(`.part-details .pin-target[data-pin="${pin}"]`)
      await settle()
      assert.equal((await context()).pin, pin)
      assert.equal(await page.$eval('.react-flow__viewport', element => element.getAttribute('style')), camera)
      assert.deepEqual(await page.$$eval('.part-details .pin-table tbody tr', rows => rows.map(row => row.lastElementChild.textContent)), states)
    }
    const outside = await page.$('.part-network .destination-group[data-local="false"]')
    assert.ok(outside)
    await outside.evaluate(element => { element.open = true })
    const endpoint = await outside.$('.destination-endpoint')
    await endpoint.evaluate(element => element.scrollIntoView({ block: 'center' }))
    await endpoint.click()
    await settle()
    assert.notEqual((await context()).selection[0], corePath)
    await click('Back')
    assert.equal((await context()).selection[0], corePath)
    assert.equal((await context()).pin, 'P56')
    await click('Clear trace')
    assert.deepEqual(await page.$$eval('.part-details .pin-table tbody tr', rows => rows.map(row => row.lastElementChild.textContent)), states)
    for (const state of ['NC', 'open']) {
      const part = model.instances.find(instance => instance.pins.some(pin => state === 'NC' ? pin.nc : !pin.connected && !pin.nc))
      assert.ok(part)
      const pin = part.pins.find(pin => state === 'NC' ? pin.nc : !pin.connected && !pin.nc)
      await select(part.designator)
      await page.$eval(`.part-details .pin-target[data-pin="${pin.logical}"]`, element => element.scrollIntoView({ block: 'center' }))
      await page.click(`.part-details .pin-target[data-pin="${pin.logical}"]`)
      await settle()
      assert.equal(await page.$eval('.part-network', element => element.dataset.state), state)
      await capture(`${prefix}-${state}`)
    }
    const ready = registry.documents.find(document => document.status === 'ready' && !document.id.startsWith('original-') && document.applicability.some(a => a.instances.length))
    const missing = registry.documents.find(document => document.status === 'missing' && document.applicability.some(a => a.instances.length))
    for (const [document, status] of [[ready, 'ready'], [missing, 'missing']]) {
      assert.ok(document)
      const part = model.instances.find(instance => instance.path === document.applicability.find(a => a.instances.length).instances[0])
      await select(part.designator)
      const selector = `.part-documents [data-document-id="${document.id}"]`
      await page.waitForSelector(selector)
      if (status === 'ready') assert.ok(await page.$(`${selector} a[href^="/api/documents/"]`))
      else assert.equal(await page.$(`${selector} a[href^="/api/documents/"]`), null)
      await page.$eval('.part-documents', element => element.scrollIntoView({ block: 'center' }))
      await capture(`${prefix}-documents-${status}`)
    }
    await click('BOM')
    await page.type('[aria-label="Search BOM"]', 'C')
    const rows = await page.$$('.bom-panel tbody tr')
    assert.ok(rows.length >= 3)
    for (const row of rows.slice(0, 3)) {
      const path = await row.evaluate(element => element.dataset.instance)
      const button = await row.$('button')
      await button.evaluate(element => element.scrollIntoView({ block: 'nearest' }))
      const scroll = await page.$eval('.bom-panel .inspection-lists', element => element.scrollTop)
      await button.click()
      await settle()
      assert.equal((await context()).tab, 'bom')
      assert.equal((await context()).selection[0], path)
      assert.equal(await page.$eval('[aria-label="Search BOM"]', element => element.value), 'C')
      assert.equal(await page.$eval('.bom-panel .inspection-lists', element => element.scrollTop), scroll)
      assert.ok(await page.$(`.bom-panel tr[data-instance="${path}"][aria-selected="true"]`))
      assert.equal(await page.$eval('.part-details', element => element.dataset.instance), path)
    }
    await capture(`${prefix}-continuous-bom`)
    await click('Clear filters')
    assert.equal((await context()).mode, 'sch')
    assert.ok((await context()).groups.length)
    assert.deepEqual((await context()).collapsed, [])
    await page.close()
  }
  assert.deepEqual(errors, [])
  writeFileSync(resolve(output, 'result.json'), JSON.stringify({ status: 'PASS', parts: model.instances.length, results, errors }, null, 2))
  console.log(JSON.stringify({ status: 'PASS', parts: model.instances.length, captures: results.length, output }))
} catch (error) {
  if (page && !page.isClosed()) await page.screenshot({ path: resolve(output, 'failure.png') })
  writeFileSync(resolve(output, 'result.json'), JSON.stringify({ status: 'FAIL', error: String(error), results, errors }, null, 2))
  throw error
} finally {
  await browser.close()
}
