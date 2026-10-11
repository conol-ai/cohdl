import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runNavigationFixtureSmoke(page, { url, output, ready, clickText, sameCamera, camera, context, search }) {
  const long = 'ExtremelyLongUnbrokenCircuitIdentity'.repeat(6)
  const span = (line) => ({ file: `fixture/${long}.cohdl`, line, col: 1 })
  const paths = ['Fixture::child::a', 'Fixture::child::b', 'Fixture::c']
  const instance = (path, index) => ({ path, designator: `U${index + 1}`, device_fq: 'fixture::Device',
    part: { fq: 'fixture::Part', mpn: index ? '' : long, footprint: 'fixture::FP', mfr: '' }, impl_traits: [], specs: [], docs: [], span: span(index + 1), pins: [
      { logical: 'SIGNAL', numbers: ['1', '3', 'A1'], role: 'passive', obligation: 'required', connected: true, nc: false },
      { logical: 'RETURN', numbers: ['2'], role: 'passive', obligation: 'required', connected: true, nc: false },
      { logical: 'NC', numbers: ['4'], role: 'passive', obligation: 'optional', connected: false, nc: true },
    ] })
  const member = (index, pin) => ({ instance_path: paths[index], logical_pin: pin, numbers: pin === 'SIGNAL' ? ['1', '3', 'A1'] : ['2'] })
  const fixture = { schema_version: 1, design: 'NavigationFixture', verdict: 'fail', live_error: `Actual watch failure fixture ${long}`,
    instances: paths.map(instance), nets: [
      { name: 'BUS', is_gnd: false, span: span(20), members: [member(0, 'SIGNAL'), member(0, 'RETURN'), member(1, 'SIGNAL'), member(2, 'SIGNAL')] },
      { name: 'GND', is_gnd: true, span: span(21), members: [member(1, 'RETURN'), member(2, 'RETURN')] },
    ], nc: [{ instance_path: paths[0], logical_pin: 'NC' }],
    diagnostics: [{ code: 'LOCATED', severity: 'error', message: long, span: span(1) }, { code: 'NET_SOURCE', severity: 'warning', message: 'net source fixture', span: span(20) }, { code: 'UNLOCATED', severity: 'warning', message: 'No authored span fixture' }, { code: 'ATTRIBUTE', severity: 'warning', message: 'Attribute source fixture', span: span(50) }],
    subdesigns: [{ path: 'Fixture::child', definition: 'fixture::Child', span: span(40), ports: [{ name: 'BUS', net: 'BUS', connected: true, obligation: 'required' }], docs: [] }],
    derived: { two_terminal: [], rails: ['GND'], fn_groups: [], bypasses: [] },
    footprints: { 'fixture::FP': { pads: [{ number: '1', shape: 'rect', x: 0, y: 0, size: [1, 1], rotate: 0, layer: 'top_copper', pth: false }], mount_holes: [] } },
    layout: { placements: paths.map((path, index) => ({ instance: path, at: [index * 20, 0], at_mm: [String(index * 20), '0'], rotate: 0, side: index ? 'bottom' : 'top', matrix: [1, 0, 0, 1] })), board_outline: null, net_classes: [], diff_pairs: [], length_matches: [], placement_hints: [] },
  }
  const config = { schema_version: 1, design: fixture.design, views: [
    { name: `Functional ${long}`, regions: [{ name: long, members: ['U1'] }, { name: 'Right', members: ['U2'] }] },
    { name: 'Source / & 空间', regions: [{ name: 'Page 1', members: ['U1', 'U2'] }] },
  ] }
  await page.setRequestInterception(true)
  const intercept = (request) => new URL(request.url()).pathname === '/__navigation_fixture.json'
    ? request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(fixture) })
    : new URL(request.url()).pathname === '/views/NavigationFixture.view.json'
      ? request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(config) }) : request.continue()
  page.on('request', intercept)
  const records = []
  try {
    for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
      await page.setViewport(viewport)
      await page.goto(`${url}?model=/__navigation_fixture.json&nolive=1`, { waitUntil: 'domcontentloaded' }); await ready()
      const flat = await context(); assert.equal(flat.scope, null)
      await clickText(`Collapse ${long}`); await ready()
      assert.equal(Number(await page.$eval('.inspection-counts', (el) => el.dataset.parts)), 3)
      await search('U1.a1')
      assert.deepEqual((await context()).selection, [paths[0]])
      assert.equal((await context()).pin, 'SIGNAL')
      assert.ok(!(await context()).collapsed.includes(long))
      await search('U1.4')
      assert.equal((await context()).net, ''); assert.equal((await context()).pin, 'NC')
      assert.ok(await page.$('tr[data-pin="NC"][data-selected="true"]'))
      await page.click('.react-flow__controls-zoomin'); await new Promise((r) => setTimeout(r, 250))
      const flatCamera = await camera()
      await page.select('#hierarchy-scope', ''); await ready()
      assert.equal((await context()).scope, '')
      await page.click('.react-flow__controls-zoomout'); await new Promise((r) => setTimeout(r, 250))
      const rootCamera = await camera()
      await page.select('#hierarchy-scope', 'all'); await ready(); sameCamera(flatCamera, await camera(), 'flat scope')
      await page.select('#hierarchy-scope', ''); await ready(); sameCamera(rootCamera, await camera(), 'root scope')
      await page.select('#hierarchy-scope', 'all'); await ready()
      await clickText('Issues'); await ready()
      const issuesText = await page.$eval('.explorer-inspector', (el) => el.textContent)
      assert.ok(issuesText.includes(fixture.live_error)); assert.ok(issuesText.includes('Source location unavailable'))
      await page.click('button[aria-label="Locate ATTRIBUTE source"]')
      assert.ok((await page.$eval('.issue-source-focus', (el) => el.textContent)).includes('No component is declared'))
      assert.equal(await page.$eval('.issue-source-focus', (el) => {
        const r = el.getBoundingClientRect(), p = el.parentElement.getBoundingClientRect()
        return document.activeElement === el && r.top >= p.top && r.top < p.bottom
      }), true, 'Unmatched diagnostic focus must be visible')
      await clickText('BOM'); await ready()
      await clickText('Issues'); await ready()
      assert.equal(await page.$('.issue-source-focus'), null, 'Inspection context change clears previous unmatched source focus')
      await page.click('button[aria-label="Locate NET_SOURCE source"]'); await ready()
      assert.equal((await context()).net, 'BUS')
      await clickText('Issues'); await ready()
      await page.focus('button[aria-label="Locate LOCATED source"]'); await page.keyboard.press('Space'); await ready()
      assert.deepEqual((await context()).selection, [paths[0]])
      await clickText('BOM'); await ready()
      const shape = await page.evaluate(() => {
        const canvas = document.querySelector('.explorer-canvas').getBoundingClientRect(), inspector = document.querySelector('.explorer-inspector').getBoundingClientRect()
        return { canvas: { width: canvas.width, height: canvas.height }, inspector: { width: inspector.width, height: inspector.height }, overflow: document.documentElement.scrollWidth > innerWidth }
      })
      assert.equal(shape.overflow, false); assert.ok(shape.canvas.width >= 300 && shape.canvas.height >= 200)
      await page.screenshot({ path: resolve(output, `navigation-${label}-long-bom-fixture.png`) })
      await page.focus('button[aria-label="Locate U2"]'); await page.keyboard.press('Enter'); await ready()
      await clickText('Layout'); await page.waitForSelector('svg[aria-label="Physical component layout in millimetres"]')
      await new Promise((r) => setTimeout(r, 200))
      await page.select('select[aria-label="Component side"]', 'bottom')
      await page.click('button[aria-label="Zoom into layout"]'); await new Promise((r) => setTimeout(r, 200))
      const board = await context(); assert.equal(board.layoutSide, 'bottom'); assert.equal(board.boardViewport.autoFit, false)
      await clickText('Overview'); await ready()
      await clickText('Layout'); await page.waitForSelector('svg[aria-label="Physical component layout in millimetres"]'); await new Promise((r) => setTimeout(r, 200))
      const restored = await context(); assert.deepEqual(restored.boardViewport, board.boardViewport)
      const length = await page.evaluate(() => history.length)
      await page.reload({ waitUntil: 'domcontentloaded' }); await page.waitForSelector('svg[aria-label="Physical component layout in millimetres"]'); await new Promise((r) => setTimeout(r, 300))
      assert.deepEqual((await context()).boardViewport, board.boardViewport); assert.equal((await context()).layoutSide, 'bottom')
      assert.equal(await page.evaluate(() => history.length), length)
      await page.screenshot({ path: resolve(output, `navigation-${label}-layout-reload-fixture.png`) })
      await clickText('Overview'); await ready()
      // Several queued transitions must finish at the final source view, then
      // return to the already-saved native camera rather than an obsolete fit.
      await page.evaluate((names) => { for (const name of names) [...document.querySelectorAll('button')].find((b) => b.textContent.trim() === name).click() }, [config.views[1].name, config.views[0].name, config.views[1].name])
      await ready(); assert.equal((await context()).view, config.views[1].name)
      await clickText(config.views[0].name); await ready()
      const recovered = await context()
      assert.equal(recovered.view, config.views[0].name)
      records.push({ label, fixture: 'Explicit synthetic navigation model, original live model unchanged', flat, flatCamera, rootCamera, shape, board, restored, recovered })
      console.log(JSON.stringify({ label, navigation: 'declared fixture PASS' }))
    }
  } finally { page.off('request', intercept); await page.setRequestInterception(false) }
  const { runRefreshSmoke } = await import('./navigation-refresh-smoke.mjs')
  const refresh = await runRefreshSmoke(page, { url, fixture, config, ready, clickText, camera, context, sameCamera })
  writeFileSync(resolve(output, 'navigation-fixture.json'), JSON.stringify({ result: 'PASS', records, refresh }, null, 2))
}
