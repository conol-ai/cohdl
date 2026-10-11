import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runLayoutInspectionSmoke(page, { url, output, ready, clickText }) {
  const fixture = await page.evaluate(async () => await (await fetch('/api/model')).json())
  const members = fixture.nets.find((n) => n.name === 'CAM_HREF').members
  // PDFExact declares no placements. Only this browser fixture supplies coordinates.
  fixture.layout = { placements: members.map((m, i) => ({ instance: m.instance_path, at: [i * 30, 0], at_mm: [String(i * 30), '0'], rotate: 0, side: 'top', matrix: [1, 0, 0, 1] })), board_outline: null, net_classes: [], diff_pairs: [], length_matches: [], placement_hints: [] }
  await page.setRequestInterception(true)
  const intercept = (request) => request.url().endsWith('/__m6_layout_fixture.json')
    ? request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(fixture) }) : request.continue()
  page.on('request', intercept)
  const snapshot = () => page.evaluate(() => {
    const svg = document.querySelector('svg[aria-label="Physical component layout in millimetres"]')
    const view = svg.viewBox.baseVal, rect = svg.getBoundingClientRect()
    const selected = [...document.querySelectorAll('.board-layout__component')].filter((el) => el.querySelector('rect')?.getAttribute('fill') === '#22d3ee22')
    return { view: { x: view.x, y: view.y, width: view.width, height: view.height }, scale: svg.getScreenCTM().a,
      size: { width: rect.width, height: rect.height }, selected: selected.map((el) => el.dataset.instance),
      rendered: selected.map((el) => { const b = el.getBoundingClientRect(); return { x: b.x, y: b.y, width: b.width, height: b.height } }),
      endpoints: [...document.querySelectorAll('tr[data-endpoint]')].map((r) => ({ path: r.dataset.endpoint, group: r.dataset.group, visible: r.dataset.visible })),
      overflow: document.documentElement.scrollWidth > innerWidth }
  })
  const records = []
  try {
    for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
      await page.setViewport(viewport)
      await page.goto(`${url}?model=/__m6_layout_fixture.json&nolive=1`, { waitUntil: 'domcontentloaded' })
      await ready()
      await clickText('Camera')
      await ready()
      await page.type('input[aria-label="Search circuit"]', 'CAM_HREF')
      await page.evaluate(() => [...document.querySelectorAll('button')].find((b) => b.textContent.includes('CAM_HREF') && b.textContent.includes('pins')).click())
      await clickText('Layout')
      await page.waitForSelector('tr[data-endpoint][data-visible="false"] button')
      await (await page.$('tr[data-endpoint][data-visible="true"] button')).click()
      await page.waitForSelector('svg[aria-label="Physical component layout in millimetres"]')
      await page.click('button[aria-label="Zoom into layout"]')
      await new Promise((r) => setTimeout(r, 200))
      const before = await snapshot()
      assert.equal(before.endpoints.length, members.length)
      assert.ok(before.endpoints.some((e) => e.group === 'ESP32 core' && e.visible === 'false'))
      assert.ok(before.size.width > 0 && before.size.height > 0)
      assert.equal(before.overflow, false)
      assert.ok(before.selected.length)
      assert.ok((await page.$eval('.board-layout', (el) => el.textContent)).includes('9 without placement'), 'Layout unplaced list must respect the selected group')
      await (await page.$('tr[data-endpoint][data-visible="false"] button')).click()
      await new Promise((r) => setTimeout(r, 200))
      const destination = await snapshot()
      assert.ok(destination.endpoints.some((e) => e.group === 'ESP32 core' && e.visible === 'true'))
      await clickText('Back')
      await new Promise((r) => setTimeout(r, 200))
      const restored = await snapshot()
      assert.deepEqual(restored.selected, before.selected)
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
      records.push({ label, before, destination, restored, resized })
    }
  } finally {
    page.off('request', intercept)
    await page.setRequestInterception(false)
  }
  writeFileSync(resolve(output, 'layout-inspection-fixture.json'), JSON.stringify({ result: 'PASS', fixture: 'PDFExact with two test-only placements', records }, null, 2))
}
