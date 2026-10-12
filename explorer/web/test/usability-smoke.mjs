import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'

const arg = (name, fallback) => process.argv.includes(name) ? process.argv[process.argv.indexOf(name) + 1] : fallback
const url = arg('--url', 'http://127.0.0.1:5210')
const output = resolve(arg('--output', '/tmp/cohdl-usability'))
mkdirSync(output, { recursive: true })
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
const records = [], errors = []
const page = await browser.newPage()
page.on('pageerror', e => errors.push(e.message))
const ready = async () => { await page.waitForSelector('.react-flow__node:not(.react-flow__node-region)'); await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'); await new Promise(r => setTimeout(r, 300)) }
const click = async text => { const el = await page.evaluateHandle(text => [...document.querySelectorAll('button')].find(e => e.getAttribute('aria-label') === text || e.textContent.trim() === text), text); assert.ok(el.asElement(), text); await el.asElement().evaluate(e => e.click()); await ready() }
const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
const shot = async name => {
  const geometry = await page.evaluate(() => {
    const c = document.querySelector('.explorer-canvas').getBoundingClientRect()
    const rect = e => { const r = e.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height } }
    const visible = [...document.querySelectorAll('.react-flow__node:not(.react-flow__node-region)')].filter(e => { const r = e.getBoundingClientRect(); return r.right > c.x && r.x < c.right && r.bottom > c.y && r.y < c.bottom })
    return { canvas: rect(document.querySelector('.explorer-canvas')), visible: visible.map(e => ({ id: e.dataset.id, ...rect(e) })), ratio: c.height / innerHeight, overflow: document.documentElement.scrollWidth > innerWidth, dim: [...document.querySelectorAll('.react-flow__node > div')].filter(e => Number(getComputedStyle(e).opacity) < .5).length,
      headings: [...document.querySelectorAll('.explorer-inspector h3')].map(e => Number.parseFloat(getComputedStyle(e).fontSize)) }
  })
  assert.ok(geometry.visible.length, `${name}: actual circuit visible`)
  assert.ok(geometry.ratio >= (page.viewport().width < 680 ? .4 : .5), `${name}: useful canvas ${geometry.ratio}`)
  assert.equal(geometry.overflow, false)
  assert.ok(geometry.headings.every(size => size >= 14))
  // Decode a screenshot of an actual node in-browser, excluding dotted grid pixels.
  const actual = geometry.visible.filter(n => !n.id.startsWith('bundle:')).sort((a, b) => b.width * b.height - a.width * a.height)[0]
  assert.ok(actual, `${name}: actual component foreground target`)
  const node = await page.$(`.react-flow__node[data-id="${actual.id}"]`)
  const png = await node.screenshot({ encoding: 'base64' })
  const foreground = await page.evaluate(async base64 => { const img = new Image(); img.src = `data:image/png;base64,${base64}`; await img.decode(); const c = document.createElement('canvas'); c.width = img.width; c.height = img.height; const ctx = c.getContext('2d'); ctx.drawImage(img, 0, 0); const p = ctx.getImageData(0, 0, c.width, c.height).data; let count = 0; for (let i = 0; i < p.length; i += 4) if (p[i + 3] > 0 && Math.max(p[i], p[i + 1], p[i + 2]) - Math.min(p[i], p[i + 1], p[i + 2]) > 20) count++; return count }, png)
  assert.ok(foreground > 20, `${name}: rendered node foreground ${foreground}`)
  await page.screenshot({ path: resolve(output, `${name}.png`) })
  records.push({ name, geometry, foreground, context: await context() })
}
try {
  for (const [name, width, height] of [['desktop', 1280, 800], ['wide', 1920, 1080], ['mobile', 390, 844]]) {
    await page.setViewport({ width, height }); await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
    await page.evaluate(() => { for (const key of Object.keys(localStorage)) if (key.startsWith('cohdl:important:')) localStorage.removeItem(key) }); await page.goto(url); await ready()
    assert.equal(await page.$eval('.inspection-counts', e => e.dataset.totalParts), '233')
    await shot(`${name}-overview`)
    await click('Fit circuit')
    const module = await page.evaluateHandle(() => [...document.querySelectorAll('.react-flow__node')].find(e => e.textContent.includes('Motor drive')))
    const moduleBox = await module.asElement().boundingBox(), canvasBox = await (await page.$('.explorer-canvas')).boundingBox()
    await page.mouse.move(canvasBox.x + canvasBox.width / 2, canvasBox.y + canvasBox.height / 2)
    await page.mouse.wheel({ deltaX: (moduleBox.x + moduleBox.width / 2 - canvasBox.x - canvasBox.width / 2) * 2, deltaY: (moduleBox.y + moduleBox.height / 2 - canvasBox.y - canvasBox.height / 2) * 2 })
    await ready()
    while (await page.evaluate(() => new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform).a < .72)) { await page.click('.react-flow__controls-zoomin'); await ready() }
    assert.ok(module.asElement()); await module.asElement().click(); await ready()
    assert.match(await page.$eval('.explorer-inspector', e => e.textContent), /Motor drive/)
    await shot(`${name}-details`)
    await click('View inside'); assert.deepEqual((await context()).groups, ['Motor drive']); await shot(`${name}-inside`)
    await click('Back'); assert.deepEqual((await context()).groups, []); assert.ok((await context()).collapsed.includes('Motor drive')); assert.ok(await page.$('.view-inside')); await shot(`${name}-back-to-module`)
    await click('View inside')
    const titleSize = await page.evaluate(() => { const node = [...document.querySelectorAll('.react-flow__node-part')].find(e => !e.dataset.id.startsWith('bundle:')); const title = node.querySelector('[title]'); return Number.parseFloat(getComputedStyle(title).fontSize) * new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform).a })
    assert.ok(titleSize >= 10, `Small-group title readable ${titleSize}`)
    const bundle = await page.$('.react-flow__edgelabel-renderer button, .react-flow__node[data-id^="bundle:"]')
    assert.ok(bundle, 'Motor group has exact network bundles')
    await bundle.evaluate(e => e.click()); await ready(); assert.ok(await page.$('.bundle-network')); await shot(`${name}-bundle`); await page.$eval('.bundle-network', e => e.click()); await ready(); assert.ok((await context()).net); await shot(`${name}-trace`)
    await page.$eval('.groups-disclosure', e => e.open = true)
    const options = await page.$$eval('.group-selector input', els => els.length); assert.ok(options >= 26)
    const scroll = await page.$eval('.group-selector', el => { el.scrollTop = el.scrollHeight; const last = el.querySelector('.group-control:last-child').getBoundingClientRect(), box = el.getBoundingClientRect(); return { reachable: last.bottom <= box.bottom + 2 && last.top >= box.top, overflow: el.scrollHeight > el.clientHeight, scroll: el.scrollTop } }); assert.ok(scroll.reachable); if (scroll.overflow) assert.ok(scroll.scroll > 0)
    await page.evaluate(() => { const el = [...document.querySelectorAll('.group-selector label')].find(e => e.textContent.trim() === 'Display and touch'); el.querySelector('input').click() }); await ready()
    assert.equal((await context()).net, ''); assert.deepEqual((await context()).selection, []); assert.equal((await context()).pin, '')
    await page.$eval('.groups-disclosure', e => e.open = false); await shot(`${name}-another-group`)
    assert.equal(records.at(-1).geometry.dim, 0)
    await click('Back'); await click('Clear filters'); await click('Overview'); await shot(`${name}-back-overview`)
    await click('Important (0)'); await shot(`${name}-suggestions`)
    const candidate = await page.$('.core-candidate .context-command'); assert.ok(candidate); await candidate.evaluate(e => e.click()); await ready(); await page.reload(); await ready(); await click('Important (1)'); assert.ok(await page.$('.important-item'))
    await shot(`${name}-important-reload`); await page.$eval('.important-item .icon-tool', e => e.click()); await ready(); await page.reload(); await ready(); await click('Important (0)'); assert.equal(await page.$('.important-item'), null); assert.ok(await page.$('.core-candidate')); await click('Close important parts'); await shot(`${name}-sidebar-close`)
    const canvas = await page.$('.explorer-canvas'); const box = await canvas.boundingBox(); await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.wheel({ deltaY: 90 }); await ready()
    const valid = (await context()).viewport; await page.reload(); await ready(); const restored = (await context()).viewport; assert.equal(restored.viewport.zoom, valid.viewport.zoom); assert.ok(Math.abs(restored.viewport.x - valid.viewport.x) < 1 && Math.abs(restored.viewport.y - valid.viewport.y) < 1)
    await shot(`${name}-valid-camera-reload`)
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.wheel({ deltaY: 100000 }); await ready(); await page.reload(); await ready(); await shot(`${name}-offscreen-recovered`)
    const nextBox = await (await page.$('.explorer-canvas')).boundingBox(); await page.mouse.move(nextBox.x + nextBox.width / 2, nextBox.y + nextBox.height / 2); await page.mouse.wheel({ deltaY: 100000 }); await ready()
    await page.setViewport({ width: width - 20, height: height - 20 }); await ready(); await shot(`${name}-resize`)
  }
  assert.deepEqual(errors, [])
  const scenarioEvidence = { 'F34.1': ['details', 'inside', 'back-to-module'], 'F34.2': ['another-group', 'valid-camera-reload', 'offscreen-recovered', 'resize'], 'F34.3': ['overview', 'inside', 'details'], 'F34.4': ['suggestions', 'important-reload', 'sidebar-close'], 'F34.5': ['bundle', 'trace', 'details'] }
  writeFileSync(resolve(output, 'report.json'), JSON.stringify({ result: 'PASS', scenarios: Object.entries(scenarioEvidence).map(([id, steps]) => ({ id, result: 'PASS', evidence: records.filter(r => steps.some(step => r.name.endsWith(`-${step}`))).map(r => r.name) })), records, errors }, null, 2))
  console.log(JSON.stringify({ result: 'PASS', output, screenshots: records.length }))
} catch (e) { await page.screenshot({ path: resolve(output, 'failure.png') }); writeFileSync(resolve(output, 'failure.json'), JSON.stringify({ error: String(e), records, errors }, null, 2)); throw e }
finally { await browser.close() }
