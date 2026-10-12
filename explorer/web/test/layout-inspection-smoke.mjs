import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runLayoutInspectionSmoke(page, { url, output, ready, clickText }) {
  const fixture = await page.evaluate(async () => await (await fetch('/api/model')).json())
  const members = fixture.nets.find((n) => n.name === 'CAM_HREF').members
  // PDFExact declares no placements. Only this browser fixture supplies coordinates.
  fixture.layout = { placements: members.map((m, i) => ({ instance: m.instance_path, at: [i * 30, 0], at_mm: [String(i * 30), '0'], rotate: 0, side: i === 0 ? 'top' : 'bottom', matrix: [1, 0, 0, 1] })), board_outline: null, net_classes: [], diff_pairs: [], length_matches: [], placement_hints: [] }
  await page.setRequestInterception(true)
  const intercept = (request) => request.url().endsWith('/__m6_layout_fixture.json')
    ? request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(fixture) }) : request.continue()
  page.on('request', intercept)
  const snapshot = () => page.evaluate(() => {
    const svg = document.querySelector('svg[aria-label="Physical component layout in millimetres"]')
    const view = svg.viewBox.baseVal, rect = svg.getBoundingClientRect()
    const selected = [...document.querySelectorAll('.board-layout__component')].filter((el) => el.querySelector('rect')?.getAttribute('fill') === '#22d3ee22')
    return { view: { x: view.x, y: view.y, width: view.width, height: view.height }, scale: svg.getScreenCTM().a,
      side: document.querySelector('select[aria-label="Component side"]').value,
      paths: [...document.querySelectorAll('.board-layout__component')].map((el) => el.dataset.instance),
      groups: [...document.querySelectorAll('.group-selector input:checked')].map((el) => el.parentElement.textContent.trim()),
      net: [...document.querySelectorAll('.explorer-inspector h3')].map((el) => el.textContent).find((text) => text.startsWith('net ')),
      size: { width: rect.width, height: rect.height }, selected: selected.map((el) => el.dataset.instance),
      rendered: selected.map((el) => { const b = el.getBoundingClientRect(); return { x: b.x, y: b.y, width: b.width, height: b.height } }),
      endpoints: [...document.querySelectorAll('[data-endpoint]')].map((r) => ({ path: r.dataset.endpoint, group: r.closest('.destination-group').dataset.group, visible: r.dataset.visible })),
      overflow: document.documentElement.scrollWidth > innerWidth }
  })
  const records = []
  const trace = async () => {
    await page.click('input[aria-label="Search circuit"]', { clickCount: 3 })
    await page.keyboard.press('Backspace')
    await page.type('input[aria-label="Search circuit"]', 'CAM_HREF')
    await page.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent.includes('CAM_HREF') && b.textContent.includes('pins')).click())
    await page.waitForSelector('[data-endpoint]')
  }
  const clickEndpoint = async (visible) => {
    const button = await page.$(`[data-endpoint][data-visible="${visible}"] button`)
    assert.ok(button, `Missing ${visible ? 'local' : 'outside'} endpoint`)
    await button.evaluate(element => { element.closest('.destination-group').open = true })
    await button.click()
    await button.dispose()
  }
  try {
    for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
      await page.setViewport(viewport)
      await page.goto(`${url}?model=/__m6_layout_fixture.json&nolive=1`, { waitUntil: 'domcontentloaded' })
      await ready()
      await clickText('Layout')
      await page.waitForSelector('select[aria-label="Component side"]')
      await trace()
      await page.select('select[aria-label="Component side"]', 'top')
      await page.waitForFunction(() => document.querySelectorAll('.board-layout__component').length === 1)
      const top = await snapshot()
      assert.deepEqual(top.paths, [members[0].instance_path])
      assert.equal(top.endpoints.find((e) => e.path === members[1].instance_path).visible, 'false', 'Bottom endpoint must be hidden by Top filter')
      await clickText('Camera')
      assert.equal((await snapshot()).net, undefined, 'User group changes clear the old trace')
      await trace()
      await page.waitForSelector('[data-endpoint][data-visible="false"] button')
      await clickEndpoint(true)
      await page.waitForSelector('svg[aria-label="Physical component layout in millimetres"]')
      await page.click('button[aria-label="Zoom into layout"]')
      await new Promise((r) => setTimeout(r, 200))
      const before = await snapshot()
      assert.equal(before.endpoints.length, members.length)
      assert.ok(before.endpoints.some((e) => e.group === 'ESP32 core' && e.visible === 'false'))
      assert.ok(before.size.width > 0 && before.size.height > 0)
      assert.equal(before.overflow, false)
      assert.ok(before.selected.length)
      assert.equal(before.side, 'top')
      assert.ok((await page.$eval('.board-layout', (el) => el.textContent)).includes('9 without placement'), 'Layout unplaced list must respect the selected group')
      await clickEndpoint(false)
      await new Promise((r) => setTimeout(r, 200))
      const destination = await snapshot()
      assert.ok(destination.endpoints.some((e) => e.group === 'ESP32 core' && e.visible === 'true'))
      assert.equal(destination.side, 'both', 'Endpoint navigation must reveal the bottom side')
      assert.equal(destination.net, before.net)
      await clickText('Back')
      await new Promise((r) => setTimeout(r, 200))
      const restored = await snapshot()
      assert.deepEqual(restored.selected, before.selected)
      assert.equal(restored.side, before.side)
      for (const key of ['x', 'y', 'width', 'height']) assert.ok(Math.abs(restored.rendered[0][key] - before.rendered[0][key]) < 0.01, `Layout Back moved rendered component ${key}`)
      assert.ok(Math.abs(restored.scale - before.scale) < 0.001)
      assert.deepEqual(restored.endpoints, before.endpoints)
      for (const key of ['x', 'y', 'width', 'height']) assert.ok(Math.abs(restored.view[key] - before.view[key]) < 0.001, `Layout Back lost ${key}: ${JSON.stringify({ before, restored })}`)
      await page.setViewport({ width: viewport.width - 25, height: viewport.height - 30 })
      await new Promise((r) => setTimeout(r, 200))
      const resized = await snapshot()
      assert.ok(Math.abs(resized.scale - before.scale) < 0.001, 'Layout resize lost scale')
      for (const [coord, dimension] of [['x', 'width'], ['y', 'height']]) assert.ok(Math.abs(resized.view[coord] + resized.view[dimension] / 2 - before.view[coord] - before.view[dimension] / 2) < 0.001, 'Layout resize lost center')
      assert.equal(resized.overflow, false)
      await page.screenshot({ path: resolve(output, `union-${label}-layout-fixture.png`) })
      await clickText('Camera')
      await trace()
      await page.select('select[aria-label="Component side"]', 'bottom')
      await page.waitForFunction(() => document.querySelectorAll('.board-layout__component').length === 1)
      const bottom = await snapshot()
      assert.deepEqual(bottom.paths, [members[1].instance_path])
      assert.equal(bottom.endpoints.find((e) => e.path === members[0].instance_path).visible, 'false', 'Top endpoint must be hidden by Bottom filter')
      await clickEndpoint(true)
      assert.deepEqual((await snapshot()).groups, [], 'Locating a local bottom endpoint retains the all-group union despite SCH collapse state')
      await page.click('button[aria-label="Zoom into layout"]')
      await new Promise((r) => setTimeout(r, 200))
      const bottomBefore = await snapshot()
      await page.select('select[aria-label="Component side"]', 'top')
      await new Promise((r) => setTimeout(r, 200))
      const afterNavigationFilter = await snapshot()
      assert.equal(afterNavigationFilter.side, 'top', 'Consumed endpoint navigation must not override a later side choice')
      assert.deepEqual(afterNavigationFilter.paths, [members[0].instance_path])
      await page.select('select[aria-label="Component side"]', 'bottom')
      await page.waitForFunction(() => document.querySelectorAll('.board-layout__component').length === 1)
      await clickEndpoint(false)
      await new Promise((r) => setTimeout(r, 200))
      const bottomDestination = await snapshot()
      assert.equal(bottomDestination.side, 'both')
      assert.deepEqual(bottomDestination.groups, [], 'A side-only reveal must retain the all-group union')
      assert.equal(bottomDestination.net, bottomBefore.net)
      await clickText('Back')
      await new Promise((r) => setTimeout(r, 200))
      const bottomRestored = await snapshot()
      assert.equal(bottomRestored.side, 'bottom')
      assert.deepEqual(bottomRestored.selected, bottomBefore.selected)
      assert.deepEqual(bottomRestored.paths, bottomBefore.paths)
      assert.deepEqual(bottomRestored.endpoints, bottomBefore.endpoints)
      for (const key of ['x', 'y', 'width', 'height']) assert.ok(Math.abs(bottomRestored.view[key] - bottomBefore.view[key]) < 0.001, `Bottom Back lost ${key}`)
      assert.ok(Math.abs(bottomRestored.scale - bottomBefore.scale) < 0.001)
      await page.screenshot({ path: resolve(output, `union-${label}-layout-sides-fixture.png`) })
      await clickText('Clear filters')
      await ready()
      assert.equal(await page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')).layoutSide), 'both', 'Clear filters must restore both physical sides when returning to the system index')
      records.push({ label, top, before, destination, restored, resized, bottom, bottomBefore, afterNavigationFilter, bottomDestination, bottomRestored })
    }
  } finally {
    page.off('request', intercept)
    await page.setRequestInterception(false)
  }
  writeFileSync(resolve(output, 'layout-inspection-fixture.json'), JSON.stringify({ result: 'PASS', fixture: 'PDFExact with two test-only placements on opposite sides; source net topology unchanged', records }, null, 2))
}
