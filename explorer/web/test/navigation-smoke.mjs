import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runNavigationSmoke(page, { url, output, ready, clickText, geometry }) {
  const data = await page.evaluate(async () => {
    const model = await (await fetch('/api/model')).json()
    const config = await (await fetch(`/views/${model.design}.view.json`)).json()
    return { model, config }
  })
  const settle = async () => { await ready(); await new Promise((r) => setTimeout(r, 100)) }
  const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
  const camera = () => page.evaluate(() => {
    const rect = document.querySelector('.explorer-canvas').getBoundingClientRect()
    const t = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
    return { x: t.e, y: t.f, zoom: t.a, width: rect.width, height: rect.height }
  })
  const sameCamera = (a, b, label) => {
    assert.ok(Math.abs(a.zoom - b.zoom) < 0.00001, `${label} zoom: ${JSON.stringify({ a, b })}`)
    for (const [dimension, coordinate] of [['width', 'x'], ['height', 'y']])
      assert.ok(Math.abs((a[dimension] / 2 - a[coordinate]) / a.zoom - (b[dimension] / 2 - b[coordinate]) / b.zoom) < 0.02, `${label} ${coordinate}: ${JSON.stringify({ a, b })}`)
  }
  const search = async (query) => {
    await page.$eval('input[aria-label="Search circuit"]', (el) => { el.value = ''; el.dispatchEvent(new Event('input', { bubbles: true })) })
    await page.click('input[aria-label="Search circuit"]', { clickCount: 3 })
    await page.keyboard.press('Backspace')
    await page.type('input[aria-label="Search circuit"]', query)
    await page.keyboard.press('Enter')
    await settle()
  }
  const records = []
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport)
    await page.goto(url, { waitUntil: 'domcontentloaded' }); await settle()
    const allCounts = await page.$eval('.inspection-counts', (el) => ({ ...el.dataset }))
    await clickText('Collapse all groups'); await settle()
    assert.deepEqual(await page.$eval('.inspection-counts', (el) => ({ ...el.dataset })), allCounts)
    assert.ok((await context()).collapsed.length >= data.config.views[0].regions.length)
    const collapsedOverview = await page.evaluate(() => {
      const nodes = [...document.querySelectorAll('.react-flow__node-boundary')]
      const transform = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
      return { groups: nodes.length, zoom: transform.a, minWidth: Math.min(...nodes.map((n) => n.getBoundingClientRect().width)), titlePixels: Math.min(...nodes.map((n) => parseFloat(getComputedStyle(n.querySelector('[title]')).fontSize) * transform.a)) }
    })
    if (label === 'desktop') {
      assert.ok(collapsedOverview.minWidth >= 100, `Whole-system groups too small: ${JSON.stringify(collapsedOverview)}`)
      assert.ok(collapsedOverview.titlePixels >= 8, `Whole-system labels too small: ${JSON.stringify(collapsedOverview)}`)
    }
    writeFileSync(resolve(output, `navigation-${label}-all-collapsed.json`), JSON.stringify(collapsedOverview, null, 2))
    await page.screenshot({ path: resolve(output, `navigation-${label}-all-collapsed.png`) })
    await clickText('Expand all groups'); await settle()
    assert.deepEqual((await context()).collapsed, [])
    assert.deepEqual(await page.$eval('.inspection-counts', (el) => ({ ...el.dataset })), allCounts)
    await clickText('Camera'); await settle()
    await clickText('Camera power'); await settle()
    const beforeCollapse = await context()
    const counts = await page.$eval('.inspection-counts', (el) => ({ ...el.dataset }))
    await clickText('Collapse Camera'); await settle()
    assert.deepEqual(await page.$eval('.inspection-counts', (el) => ({ ...el.dataset })), counts)
    assert.ok(await page.$('.react-flow__node-boundary'), 'Actual collapsed group missing')
    assert.ok((await context()).collapsed.includes('Camera'))
    await page.evaluate((view) => [...document.querySelectorAll('.react-flow__node')].find((el) => el.dataset.id === `group:${encodeURIComponent(JSON.stringify([view, 'Camera']))}`).click(), data.config.views[0].name)
    await page.waitForSelector('tr[data-boundary-port]')
    assert.ok((await page.$eval('.boundary-pin-list', (el) => el.textContent)).includes('CAM'))
    await page.screenshot({ path: resolve(output, `navigation-${label}-collapse.png`) })
    const visiblePaths = data.model.instances.filter((i) => data.config.views[0].regions.filter((r) => ['Camera', 'Camera power'].includes(r.name)).some((r) => r.members.includes(i.designator) || r.members.includes(`path:${i.path}`))).map((i) => i.path)
    const hiddenNet = data.model.nets.find((n) => n.members.length && n.members.every((m) => !visiblePaths.includes(m.instance_path)))
    await search(hiddenNet.name)
    const netFound = await context()
    assert.equal(netFound.net, hiddenNet.name)
    assert.ok(netFound.groups.some((g) => !['Camera', 'Camera power'].includes(g)), 'An exclusively hidden net must reveal a real group')
    const target = data.model.instances.find((i) => i.designator === 'U1')
    const pin = target.pins.find((p) => p.numbers.length)
    await search(`U1.${pin.numbers[0]}`)
    const found = await context()
    assert.deepEqual(found.selection, [target.path]); assert.equal(found.pin, pin.logical)
    assert.ok(await page.$(`tr[data-pin="${pin.logical}"][data-selected="true"]`))
    assert.ok(found.groups.includes('ESP32 core'), 'Search must reveal hidden group')
    await page.type('input[aria-label="Search circuit"]', 'NoSuchComponentName')
    assert.ok((await page.$eval('.explorer-toolbar', (el) => el.textContent)).includes('No results'))
    await clickText('Clear search')
    assert.equal(await page.$eval('input[aria-label="Search circuit"]', (el) => el.value), '')
    await page.type('input[aria-label="Search circuit"]', 'NoSuchComponentName')
    await page.keyboard.press('Escape')
    assert.equal(await page.$eval('input[aria-label="Search circuit"]', (el) => el.value), '')
    await clickText('BOM'); await settle()
    const u4 = data.model.instances.find((i) => i.designator === 'U4')
    const u4Row = await page.evaluate((path) => [...document.querySelectorAll('tr[data-instance]')].find((el) => el.dataset.instance === path).textContent, u4.path)
    assert.ok(u4Row.includes('Not specified'), 'Actual empty MPN must stay empty')
    await page.click('button[aria-label="Locate U4"]'); await settle()
    assert.deepEqual((await context()).selection, [u4.path])
    await clickText('Issues'); await settle()
    assert.ok((await page.$eval('.explorer-inspector', (el) => el.textContent)).includes('No issues'))
    // Arrow keys move the actual tab and its focus; Space activates BOM locate.
    await page.focus('[role="tab"][aria-selected="true"]')
    await page.keyboard.press('Home'); await settle()
    assert.equal((await context()).tab, 'circuit')
    await page.click('.react-flow__controls-zoomin'); await new Promise((r) => setTimeout(r, 250))
    const overview = await camera()
    await clickText('SCH view'); await settle()
    await page.click('.react-flow__controls-zoomout'); await new Promise((r) => setTimeout(r, 250))
    const schematic = await camera()
    await clickText('Overview'); await settle(); sameCamera(overview, await camera(), 'mode overview')
    await clickText('SCH view'); await settle(); sameCamera(schematic, await camera(), 'mode schematic')
    await clickText('Overview'); await settle()
    const firstView = data.config.views[0].name, secondView = data.config.views[1].name
    await clickText(secondView); await settle()
    await page.click('.react-flow__controls-zoomin'); await new Promise((r) => setTimeout(r, 250))
    const otherView = await camera()
    await clickText(firstView); await settle(); sameCamera(overview, await camera(), 'view original')
    await clickText(secondView); await settle(); sameCamera(otherView, await camera(), 'view second')
    const beforeHistory = await context(), historySize = await page.evaluate(() => history.length)
    await page.goBack(); await settle(); assert.equal((await context()).view, firstView)
    await page.goForward(); await settle(); assert.deepEqual((await context()).selection, beforeHistory.selection)
    assert.equal(await page.evaluate(() => history.length), historySize, 'popstate must not push history')
    const shared = page.url(), savedCamera = await camera()
    await clickText('Copy inspection link')
    assert.equal(new URL(shared).searchParams.get('model'), null)
    await page.reload({ waitUntil: 'domcontentloaded' }); await settle()
    assert.equal((await context()).view, secondView); sameCamera(savedCamera, await camera(), 'reload')
    await page.setViewport({ width: viewport.width - 20, height: viewport.height - 20 }); await settle()
    sameCamera(savedCamera, await camera(), 'resize')
    await clickText('Fit circuit'); await settle(); await geometry(`${label}-navigation-fit`)
    records.push({ label, beforeCollapse, netFound, found, overview, schematic, otherView, beforeHistory, shared, historySize })
    console.log(JSON.stringify({ label, navigation: 'actual model PASS' }))
  }
  const { runNavigationFixtureSmoke } = await import('./navigation-fixture-smoke.mjs')
  await runNavigationFixtureSmoke(page, { url, output, ready: settle, clickText, sameCamera, camera, context, search })
  writeFileSync(resolve(output, 'navigation-inspection.json'), JSON.stringify({ result: 'PASS', actualModel: data.model.design, records }, null, 2))
}
