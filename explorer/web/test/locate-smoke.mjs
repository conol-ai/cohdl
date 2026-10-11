import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runLocateSmoke(page, { url, output, ready, clickText, netOnly = false }) {
  const records = []
  const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
  const camera = () => page.evaluate(() => {
    const r = document.querySelector('.explorer-canvas').getBoundingClientRect()
    const m = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
    return { x: m.e, y: m.f, zoom: m.a, width: r.width, height: r.height }
  })
  const sameCamera = (a, b, label) => {
    assert.ok(Math.abs(a.zoom - b.zoom) < 0.00001, `${label} zoom: ${JSON.stringify({ a, b })}`)
    for (const [dimension, coordinate] of [['width', 'x'], ['height', 'y']])
      assert.ok(Math.abs((a[dimension] / 2 - a[coordinate]) / a.zoom - (b[dimension] / 2 - b[coordinate]) / b.zoom) < 0.02, `${label} center: ${JSON.stringify({ a, b })}`)
  }
  const inspectTarget = (path) => page.evaluate((id) => {
    const node = [...document.querySelectorAll('.react-flow__node.selected')].find((el) => el.dataset.id === id)
      ?? document.querySelector('.react-flow__node.selected')
    const rect = (el) => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height, right: r.right, bottom: r.bottom } }
    return { node: node && rect(node), canvas: rect(document.querySelector('.explorer-canvas')), count: document.querySelector('.inspection-counts').dataset.parts }
  }, path)
  const visibleTarget = async (path, label) => {
    const actual = await inspectTarget(path)
    await page.screenshot({ path: resolve(output, `locate-${label}.png`) })
    writeFileSync(resolve(output, `locate-${label}.json`), JSON.stringify({ ...actual, context: await context(), camera: await camera() }, null, 2))
    assert.equal(Number(actual.count), 233, 'Locator keeps whole real circuit represented')
    assert.ok(actual.node?.width >= 90 && actual.node.height >= 50, `Located target must be inspectable: ${JSON.stringify(actual)}`)
    assert.ok(actual.node.x >= actual.canvas.x - 1 && actual.node.right <= actual.canvas.right + 1 && actual.node.y >= actual.canvas.y - 1 && actual.node.bottom <= actual.canvas.bottom + 1, `Located target must be within canvas: ${JSON.stringify(actual)}`)
    return actual
  }
  const search = async (query) => {
    await page.click('input[aria-label="Search circuit"]', { clickCount: 3 })
    await page.keyboard.press('Backspace')
    await page.type('input[aria-label="Search circuit"]', query)
    await page.keyboard.press('Enter'); await ready()
  }
  const manualDuringLayout = async (action, label) => {
    const before = await camera()
    await page.evaluate(() => {
      window.__manualLocateIntervention = false
      const canvas = document.querySelector('.explorer-canvas')
      const observer = new MutationObserver(() => {
        if (canvas.getAttribute('aria-busy') !== 'true') return
        observer.disconnect(); window.__manualLocateIntervention = true
        document.querySelector('.react-flow__controls-zoomout').click()
      })
      observer.observe(canvas, { attributes: true, attributeFilter: ['aria-busy'] })
    })
    await action()
    await page.waitForFunction(() => window.__manualLocateIntervention)
    await ready(); await new Promise((resolve) => setTimeout(resolve, 500))
    const after = await camera()
    writeFileSync(resolve(output, `locate-${label}-manual.json`), JSON.stringify({ before, after, context: await context() }, null, 2))
    await page.screenshot({ path: resolve(output, `locate-${label}-manual.png`) })
    assert.ok(after.zoom < before.zoom * 0.95, `Late layout must preserve explicit manual zoom: ${JSON.stringify({ before, after })}`)
    assert.equal((await context()).viewport.autoFit, false, 'Manual override must persist its camera policy in URL')
    await page.reload({ waitUntil: 'domcontentloaded' }); await ready()
    sameCamera(after, await camera(), 'Manual override reload')
    return { before, after }
  }
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport)
    await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
    const model = await page.evaluate(async () => (await fetch('/api/model')).json())
    const u1 = model.instances.find((i) => i.designator === 'U1')
    const pin = u1.pins.find((p) => p.numbers.includes('1'))
    const net = model.nets.find((n) => n.members.some((m) => m.instance_path === u1.path && m.logical_pin === pin.logical))
    assert.deepEqual((await context()).groups, [])
    let before, focused, globalPin, bom
    if (!netOnly) {
    await page.click('.react-flow__controls-zoomin'); await new Promise((resolve) => setTimeout(resolve, 250))
    before = await camera()
    await search('U1.1')
    assert.deepEqual((await context()).groups, [])
    assert.deepEqual((await context()).selection, [u1.path])
    assert.equal((await context()).pin, pin.logical); assert.equal((await context()).net, net.name)
    globalPin = await visibleTarget(u1.path, `${label}-global-pin`)
    focused = await camera()
    await clickText('Back'); await ready(); sameCamera(before, await camera(), 'Back restores previous manual camera')
    await page.goForward(); await ready(); sameCamera(focused, await camera(), 'Forward restores focused camera')
    await page.reload({ waitUntil: 'domcontentloaded' }); await ready(); sameCamera(focused, await camera(), 'Reload preserves focused camera')
    await page.setViewport({ width: viewport.width - 20, height: viewport.height - 20 }); await ready(); sameCamera(focused, await camera(), 'Resize preserves focused camera')
    await clickText('Fit circuit'); await ready()
    await clickText('BOM'); await ready()
    const bomCamera = await camera()
    await page.click('button[aria-label="Locate U1"]'); await ready()
    bom = await visibleTarget(u1.path, `${label}-global-bom`)
    await clickText('Back'); await ready(); assert.equal((await context()).tab, 'bom'); sameCamera(bomCamera, await camera(), 'BOM Back camera')
    await clickText('Circuit'); await ready()
    }
    const offscreen = await context()
    offscreen.viewport = { viewport: { x: 10000, y: 10000, zoom: 1 }, size: { width: 950, height: 496 }, autoFit: false }
    const offscreenUrl = new URL(page.url()); offscreenUrl.searchParams.set('inspect', JSON.stringify(offscreen))
    await page.goto(offscreenUrl.href, { waitUntil: 'domcontentloaded' }); await ready()
    await search('CAM_HREF')
    assert.equal((await context()).net, 'CAM_HREF')
    const netCamera = await camera()
    const netBounds = await page.evaluate((paths) => {
      const c = document.querySelector('.explorer-canvas').getBoundingClientRect()
      return [...document.querySelectorAll('.react-flow__node')].filter((n) => paths.includes(n.dataset.id)).map((n) => {
        const r = n.getBoundingClientRect(); return { id: n.dataset.id, visible: r.x >= c.x - 1 && r.right <= c.right + 1 && r.y >= c.y - 1 && r.bottom <= c.bottom + 1 }
      })
    }, model.nets.find((n) => n.name === 'CAM_HREF').members.map((m) => m.instance_path))
    assert.ok(netBounds.length >= 2 && netBounds.every((n) => n.visible), `Net locator must bring actual endpoints into view from offscreen camera: ${JSON.stringify(netBounds)}`)
    const row = await page.evaluateHandle((path) => [...document.querySelectorAll('tr[data-endpoint]')].find((el) => el.dataset.endpoint === path)?.querySelector('button'), u1.path)
    assert.ok(row.asElement(), 'Actual net must expose U1 endpoint')
    await row.asElement().click(); await row.dispose(); await ready()
    const endpoint = await visibleTarget(u1.path, `${label}-global-endpoint`)
    assert.equal((await context()).net, 'CAM_HREF')
    await clickText('Back'); await ready(); sameCamera(netCamera, await camera(), 'Endpoint Back camera')
    await clickText('Fit circuit'); await ready()
    const manualLocate = await manualDuringLayout(() => search('U1.1'), `${label}-locate`)
    await clickText('SCH view'); await ready(); await search('U1.1')
    await clickText('Overview'); await ready(); await clickText('Fit circuit'); await ready()
    const manualRestore = await manualDuringLayout(() => clickText('SCH view'), `${label}-restore`)
    records.push({ label, before, focused, globalPin, bom, endpoint, netCamera, netBounds, manualLocate, manualRestore })
  }
  writeFileSync(resolve(output, 'locate-inspection.json'), JSON.stringify({ result: 'PASS', realModel: 'PDFExact', records }, null, 2))
}
