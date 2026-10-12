import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runReadabilitySmoke(page, { url, output, ready, clickText }) {
  const records = []
  const settle = async () => { await ready(); await new Promise(r => setTimeout(r, 400)) }
  const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
  const shot = async label => {
    assert.equal(await page.evaluate(() => [...document.querySelectorAll('.canvas-state[role="alert"]')].map(el => el.textContent).join('')), '', `${label} must have no layout error`)
    const geometry = await page.evaluate(() => {
      const rect = el => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height, right: r.right, bottom: r.bottom } }
      const matrix = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
      return { canvas: rect(document.querySelector('.explorer-canvas')), zoom: matrix.a, nodes: [...document.querySelectorAll('.react-flow__node')].map(el => ({ id: el.dataset.id, ...rect(el) })),
        labels: [...document.querySelectorAll('.react-flow__edgelabel-renderer button')].map(rect), overflow: document.documentElement.scrollWidth > innerWidth,
        selection: document.querySelectorAll('.react-flow__node.selected').length, dimmed: [...document.querySelectorAll('.react-flow__node > div')].filter(el => Number(getComputedStyle(el).opacity) < .5).length }
    })
    assert.equal(geometry.overflow, false)
    assert.ok(geometry.canvas.width > 0 && geometry.canvas.height > 0)
    await page.screenshot({ path: resolve(output, `readability-${label}.png`) })
    records.push({ label, geometry, context: await context() })
    return geometry
  }
  const search = async value => {
    await page.click('input[aria-label="Search circuit"]', { clickCount: 3 }); await page.keyboard.press('Backspace')
    await page.type('input[aria-label="Search circuit"]', value); await page.keyboard.press('Enter'); await settle()
  }
  const empty = async label => {
    const current = await context()
    assert.deepEqual(current.selection, [], label); assert.equal(current.pin, ''); assert.equal(current.net, '')
    assert.equal(await page.$('.part-details'), null); assert.equal(await page.$('.bundle-inspector'), null)
    assert.equal(await page.$eval('input[aria-label="Search circuit"]', el => el.value), '')
    const geometry = await shot(label); assert.equal(geometry.dimmed, 0)
  }
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport); await page.goto(url, { waitUntil: 'domcontentloaded' }); await settle()
    assert.equal((await context()).mode, 'sch')
    await clickText('Overview'); await settle()
    await clickText('Collapse all groups'); await settle()
    await clickText('Bundled'); await settle()
    const model = await page.evaluate(async () => (await fetch('/api/model')).json())
    assert.equal(model.instances.length, 233)
    assert.equal(await page.evaluate(() => [...document.querySelectorAll('button')].some(el => el.textContent.trim() === 'Source pages')), false)
    assert.ok((await context()).collapsed.length >= 26)
    await shot(`${label}-overview`)
    const bundle = await page.$('.react-flow__edgelabel-renderer button')
    assert.ok(bundle, 'Optional bundled overview has real net bundles')
    await bundle.focus(); await page.keyboard.press('Enter'); await settle()
    assert.ok(await page.$('.bundle-inspector .bundle-network'))
    await shot(`${label}-bundle`)
    const expectedNet = await page.$eval('.bundle-network strong', el => el.textContent)
    await page.click('.bundle-network'); await settle(); assert.equal((await context()).net, expectedNet)
    const traceEdges = await page.$$eval('.react-flow__edge', edges => edges.map(edge => ({ id: edge.dataset.id, d: edge.querySelector('.react-flow__edge-path')?.getAttribute('d'), stroke: edge.querySelector('.react-flow__edge-path')?.style.stroke })))
    assert.ok(traceEdges.some(edge => !edge.id?.startsWith('bundle:') && edge.d && edge.stroke === 'rgb(34, 211, 238)'), `Exact trace renders a selected original SVG edge: ${JSON.stringify(traceEdges)}`)
    await shot(`${label}-trace`)
    await clickText('Clear trace'); await settle()
    await clickText('Display and touch'); await settle(); await empty(`${label}-group`)
    await clickText('SCH view'); await settle(); await search('U6')
    assert.match(await page.$eval('.part-details h3', el => el.textContent), /^U6 Component$/)
    assert.ok(await page.$('.part-footprint svg'))
    assert.equal(await page.evaluate(() => !!document.querySelector('.part-details a[href*="original-schematic"],.part-details a[href*="original-pcb"]')), false)
    await shot(`${label}-details`)
    const u6 = model.instances.find(instance => instance.designator === 'U6')
    const u6Selector = `.react-flow__node[data-id="${u6.path}"]`
    const pin = await page.$(`${u6Selector} .pin-target`)
    assert.ok(pin)
    const identity = await pin.evaluate(el => ({ instance: el.dataset.instance, pin: el.dataset.pin }))
    await pin.focus(); await page.keyboard.press(' '); await settle()
    assert.deepEqual((await context()).selection, [identity.instance]); assert.equal((await context()).pin, identity.pin)
    const assertSelectedVisible = async () => {
      const visible = await page.$eval(u6Selector, node => {
        const rect = node.getBoundingClientRect(), canvas = document.querySelector('.explorer-canvas').getBoundingClientRect()
        return rect.right > canvas.left && rect.left < canvas.right && rect.bottom > canvas.top && rect.top < canvas.bottom
      })
      assert.ok(visible, 'Pin trace keeps the selected actual device on the canvas')
    }
    await assertSelectedVisible()
    await shot(`${label}-keyboard-pin-trace`)
    const pointerPin = async side => {
      const target = await page.evaluateHandle(({ selector, side }) => [...document.querySelectorAll(`${selector} .pin-target`)].find(el => el.style[side] !== ''), { selector: u6Selector, side })
      assert.ok(target.asElement(), `Actual ${side} pin row exists`)
      const wanted = await target.evaluate(el => ({ instance: el.dataset.instance, pin: el.dataset.pin }))
      for (let attempt = 0; attempt < 3; attempt++) {
        const point = await target.evaluate(el => {
          const rect = el.getBoundingClientRect(), canvas = document.querySelector('.explorer-canvas').getBoundingClientRect()
          const x = rect.left + rect.width / 2, y = rect.top + rect.height / 2
          return { x, y, cx: canvas.left + canvas.width / 2, cy: canvas.top + canvas.height / 2,
            visible: x > canvas.left && x < canvas.right && y > canvas.top && y < canvas.bottom }
        })
        if (point.visible) break
        await page.mouse.move(point.cx, point.cy)
        await page.mouse.wheel({ deltaX: point.x - point.cx, deltaY: point.y - point.cy })
        await settle()
      }
      const point = await target.evaluate(el => {
        const rect = el.getBoundingClientRect(), x = rect.left + rect.width / 2, y = rect.top + rect.height / 2
        return { x, y, hit: document.elementFromPoint(x, y)?.closest('.pin-target') === el }
      })
      assert.ok(point.hit, `${side} actual pointer target is visible and unobstructed`)
      await page.mouse.click(point.x, point.y); await settle()
      assert.deepEqual((await context()).selection, [wanted.instance]); assert.equal((await context()).pin, wanted.pin)
      await target.dispose()
      await assertSelectedVisible()
    }
    for (const side of ['left', 'right']) {
      await pointerPin(side)
    }
    await clickText('Clear trace'); await settle()
    await clickText('Bundled'); await settle()
    await pointerPin('left')
    await shot(`${label}-bundled-pin-trace`)
    const row = await page.$('.part-details tr[data-pin]')
    const rowPin = await row.evaluate(el => el.dataset.pin)
    await row.focus(); await page.keyboard.press('Enter'); await settle(); assert.equal((await context()).pin, rowPin)
    await clickText('Mark important'); await settle(); await shot(`${label}-marked`)
    await page.reload({ waitUntil: 'domcontentloaded' }); await settle()
    assert.ok(await page.$('button[aria-label="Unmark important"]'), 'Mark survives reload independently')
    await page.type('input[aria-label="Search circuit"]', 'stale query')
    await clickText('Camera'); await settle(); await empty(`${label}-group-reset`)
    assert.ok(await page.evaluate(() => [...document.querySelectorAll('button')].some(el => el.textContent.trim() === 'Important (1)')))
    await clickText('Back'); await settle(); assert.equal((await context()).pin, rowPin)
    await clickText('Overview'); await settle(); await empty(`${label}-view-reset`)
    await clickText('Important (1)'); await settle(); await shot(`${label}-important-list`)
    await page.click('.important-list .locate-link'); await settle(); assert.equal((await context()).selection.length, 1)
    await page.click('.important-list button[aria-label^="Unmark "]'); await settle()
    await search('C16')
    const railRows = await page.$$eval('.part-details tbody tr', rows => rows.map(row => row.cells[3].textContent))
    assert.ok(railRows.length && railRows.every(state => state === 'simplified rail'), 'C16 wired pins remain inspectable simplified rails')
    for (const [state, predicate] of [['NC', pin => pin.nc], ['open', pin => !pin.nc && !pin.connected]]) {
      const component = model.instances.find(instance => instance.pins.some(predicate))
      const contact = component.pins.find(predicate)
      await search(component.designator)
      const actual = await page.$eval(`.part-details tr[data-pin="${contact.logical}"]`, row => row.cells[3].textContent)
      assert.equal(actual, state)
    }
    await search('U4')
    assert.equal(await page.$eval('.part-value', el => el.textContent), 'SF32LB525UC6')
    assert.ok((await page.$eval('.part-facts', el => el.textContent)).includes('Unspecified'))
    const u4 = model.instances.find(instance => instance.designator === 'U4')
    assert.ok((await page.$eval(`.react-flow__node[data-id="${u4.path}"]`, el => el.textContent)).includes('SF32LB525UC6'), 'Canvas shares the literal source value')
    await clickText('SCH view'); await settle(); await search('U4')
    const heading = await page.$eval(`.react-flow__node[data-id="${u4.path}"] .detailed-heading`, el => {
      const r = el.getBoundingClientRect(), parent = el.parentElement.getBoundingClientRect()
      return { text: el.textContent, title: el.title, width: r.width, height: r.height, contained: r.left >= parent.left - 1 && r.right <= parent.right + 1 && r.top >= parent.top - 1 && r.bottom <= parent.bottom + 1 }
    })
    assert.ok(heading.text.includes('SF32LB525UC6')); assert.ok(heading.title.includes('SF32LB525UC6'))
    assert.ok(heading.width > 0 && heading.height > 0 && heading.contained, `SCH source heading stays inside component: ${JSON.stringify(heading)}`)
    await shot(`${label}-u4-sch-source-value`)
    records.push({ label: `${label}-u4-sch-heading`, heading })
  }
  const { runImportantStorageSmoke } = await import('./important-storage-smoke.mjs')
  await runImportantStorageSmoke(page, { url, output, ready, clickText })
  writeFileSync(resolve(output, 'readability-results.json'), JSON.stringify({ result: 'PASS', records }, null, 2))
}
