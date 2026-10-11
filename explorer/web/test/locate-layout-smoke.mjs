import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runLocateLayoutSmoke(page, { url, output, clickText }) {
  const fixture = await page.evaluate(async () => (await fetch('/api/model')).json())
  const members = fixture.nets.find((n) => n.name === 'CAM_HREF').members
  fixture.layout = { placements: members.map((m, i) => ({ instance: m.instance_path, at: [i * 30, 0], at_mm: [String(i * 30), '0'], rotate: 0, side: i === 0 ? 'top' : 'bottom', matrix: [1, 0, 0, 1] })), board_outline: null, net_classes: [], diff_pairs: [], length_matches: [], placement_hints: [] }
  await page.setRequestInterception(true)
  const intercept = (request) => new URL(request.url()).pathname === '/__locate_layout_fixture.json'
    ? request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(fixture) }) : request.continue()
  page.on('request', intercept)
  const snapshot = () => page.evaluate(() => {
    const svg = document.querySelector('svg[aria-label="Physical component layout in millimetres"]')
    const c = svg.getBoundingClientRect(), v = svg.viewBox.baseVal
    return { side: document.querySelector('select[aria-label="Component side"]').value, scale: svg.getScreenCTM().a, view: { x: v.x, y: v.y, width: v.width, height: v.height }, size: { width: c.width, height: c.height }, components: [...document.querySelectorAll('.board-layout__component')].map((el) => {
      const r = el.getBoundingClientRect(); return { path: el.dataset.instance, visible: r.x >= c.x - 1 && r.right <= c.right + 1 && r.y >= c.y - 1 && r.bottom <= c.bottom + 1 }
    }) }
  })
  const ready = () => page.waitForFunction(() => document.querySelector('svg[aria-label="Physical component layout in millimetres"]') && document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false')
  const records = []
  try {
    for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
      await page.setViewport(viewport)
      await page.goto(`${url}?model=/__locate_layout_fixture.json&nolive=1&mode=layout`, { waitUntil: 'domcontentloaded' }); await ready()
      await new Promise((resolve) => setTimeout(resolve, 250))
      const initial = await snapshot()
      const context = await page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
      context.layoutSide = 'top'
      context.boardViewport = { view: { x: 10000, y: 10000, width: 200, height: 200 }, size: initial.size, autoFit: false }
      const target = new URL(page.url()); target.searchParams.set('inspect', JSON.stringify(context))
      await page.goto(target.href, { waitUntil: 'domcontentloaded' }); await ready(); await new Promise((resolve) => setTimeout(resolve, 250))
      const before = await snapshot()
      await page.type('input[aria-label="Search circuit"]', 'CAM_HREF'); await page.keyboard.press('Enter')
      await ready(); await new Promise((resolve) => setTimeout(resolve, 350))
      const focused = await snapshot()
      await page.screenshot({ path: resolve(output, `locate-layout-${label}-net.png`) })
      writeFileSync(resolve(output, `locate-layout-${label}-net.json`), JSON.stringify({ before, focused }, null, 2))
      assert.equal(focused.side, 'both', 'Explicit net visit reveals placed members on both sides')
      assert.deepEqual(focused.components.map((c) => c.path).sort(), members.map((m) => m.instance_path).sort())
      assert.ok(focused.components.every((c) => c.visible), 'Actual placed endpoint bounds must be within SVG camera')
      await clickText('Back'); await ready(); await new Promise((resolve) => setTimeout(resolve, 250))
      const restored = await snapshot()
      assert.equal(restored.side, before.side)
      assert.ok(Math.abs(restored.scale - before.scale) < 0.001, 'Layout net Back lost actual screen scale')
      for (const [coordinate, dimension] of [['x', 'width'], ['y', 'height']]) assert.ok(Math.abs(restored.view[coordinate] + restored.view[dimension] / 2 - before.view[coordinate] - before.view[dimension] / 2) < 0.001, `Layout net Back lost logical ${coordinate} center`)
      await page.goForward(); await ready(); await new Promise((resolve) => setTimeout(resolve, 250))
      await page.select('select[aria-label="Component side"]', 'top'); await new Promise((resolve) => setTimeout(resolve, 250))
      assert.equal((await snapshot()).side, 'top', 'Consumed net visit must preserve later manual side choice')
      records.push({ label, before, focused, restored })
    }
    writeFileSync(resolve(output, 'locate-layout-inspection.json'), JSON.stringify({ result: 'PASS', fixture: 'Only this browser fixture supplies two explicit opposite-side placements; real PDFExact has no placements', records }, null, 2))
  } finally {
    page.off('request', intercept)
    await page.setRequestInterception(false)
  }
}
