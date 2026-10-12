import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runUnionSmoke(page, { url, output, ready, clickText, geometry }) {
  const data = await page.evaluate(async () => {
    const model = await (await fetch('/api/model')).json()
    const config = await (await fetch(`/views/${encodeURIComponent(model.design)}.view.json`)).json()
    return { model, view: config.views[0] }
  })
  const groups = new Map(data.model.instances.map((i) => [i.path, data.view.regions.find((r) => r.members.some((m) => m === i.designator || m === `path:${i.path}`))?.name ?? 'Other']))
  const expected = (names) => {
    const parts = data.model.instances.filter((i) => !names.length || names.includes(groups.get(i.path))).map((i) => i.path).sort()
    return { parts, nets: new Set(data.model.nets.filter((n) => n.members.some((m) => parts.includes(m.instance_path))).map((n) => n.name)).size }
  }
  const inspect = () => page.evaluate(() => {
    const counts = document.querySelector('.inspection-counts').dataset
    const canvas = document.querySelector('.explorer-canvas').getBoundingClientRect()
    const transform = new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform)
    const header = document.querySelector('.react-flow__node.selected .device-header')?.getBoundingClientRect()
    return {
      parts: [...document.querySelectorAll('.react-flow__node')].map((n) => n.dataset.id).filter((id) => !id.startsWith('region:') && !id.startsWith('net:') && !id.startsWith('bundle:') && !id.startsWith('group:')).sort(),
      selected: [...document.querySelectorAll('.react-flow__node.selected')].map((n) => n.dataset.id).sort(),
      groups: [...document.querySelectorAll('.group-selector input:checked')].map((i) => i.parentElement.textContent.trim()).sort(),
      net: [...document.querySelectorAll('.explorer-inspector h3')].map((h) => h.textContent).find((h) => h.startsWith('net ')),
      counts: { ...counts }, viewport: { x: transform.e, y: transform.f, zoom: transform.a, width: canvas.width, height: canvas.height },
      identity: header && { x: header.x - canvas.x, y: header.y - canvas.y, width: header.width, height: header.height },
    }
  })
  const assertUnion = async (names) => {
    const actual = await inspect()
    const want = expected(names)
    assert.deepEqual(actual.parts, want.parts)
    assert.equal(Number(actual.counts.parts), want.parts.length)
    assert.equal(Number(actual.counts.nets), want.nets)
    assert.equal(Number(actual.counts.totalParts), data.model.instances.length)
    assert.equal(Number(actual.counts.totalNets), new Set(data.model.nets.map((n) => n.name)).size)
    assert.deepEqual(actual.groups, [...names].sort())
    return actual
  }
  const snapshots = []
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport)
    await page.goto(url, { waitUntil: 'domcontentloaded' })
    await ready()
    await clickText('Camera')
    await ready()
    await clickText('Camera power')
    await ready()
    await assertUnion(['Camera', 'Camera power'])
    await geometry(`${label}-union`)
    const input = await page.$('input[aria-label="Search circuit"]')
    await input.type('CAM_HREF')
    const result = await page.evaluateHandle(() => [...document.querySelectorAll('button')].find((b) => b.textContent.includes('CAM_HREF') && b.textContent.includes('pins')))
    await result.asElement().click()
    await result.dispose()
    await page.waitForSelector('tr[data-endpoint][data-visible="false"]')
    const net = data.model.nets.find((n) => n.name === 'CAM_HREF')
    const endpoints = await page.$$eval('tr[data-endpoint]', (rows) => rows.map((r) => ({ path: r.dataset.endpoint, group: r.dataset.group, visible: r.dataset.visible === 'true' })))
    assert.deepEqual(endpoints.map((p) => p.path).sort(), net.members.map((m) => m.instance_path).sort())
    for (const endpoint of endpoints) assert.equal(endpoint.group, groups.get(endpoint.path))
    const visible = await page.$('tr[data-endpoint][data-visible="true"] button')
    await visible.click()
    await ready()
    // Controls are programmatic inside React Flow but are still manual user zoom.
    await page.click('.react-flow__controls-zoomin')
    await new Promise((r) => setTimeout(r, 250))
    const controlsOnly = await inspect()
    await (await page.$('tr[data-endpoint][data-visible="false"] button')).click()
    await ready()
    await clickText('Back')
    await ready()
    const controlsRestored = await inspect()
    for (const key of ['x', 'y', 'zoom']) assert.ok(Math.abs(controlsRestored.viewport[key] - controlsOnly.viewport[key]) < 0.01, `Controls-only Back lost ${key}`)
    // Real mouse pan, then zoom: this must stop subsequent auto-fit.
    const canvas = await page.$eval('.explorer-canvas', (el) => { const r = el.getBoundingClientRect(); return { x: r.x, y: r.y, width: r.width, height: r.height } })
    const beforePan = await inspect()
    await page.mouse.move(canvas.x + 120, canvas.y + canvas.height - 35)
    await page.mouse.down({ button: 'middle' })
    await page.mouse.move(canvas.x + 175, canvas.y + canvas.height - 65, { steps: 8 })
    await page.mouse.up({ button: 'middle' })
    const afterPan = await inspect()
    assert.ok(Math.abs(afterPan.viewport.x - beforePan.viewport.x) > 10, 'Mouse pan must move the real viewport')
    await page.click('.react-flow__controls-zoomin')
    await new Promise((r) => setTimeout(r, 250))
    const before = await inspect()
    assert.ok(before.selected.length, 'Expected selected source endpoint')
    const hidden = await page.$('tr[data-endpoint][data-visible="false"] button')
    await hidden.click()
    await ready()
    const destination = await inspect()
    assert.equal(destination.net, 'net CAM_HREF')
    assert.deepEqual(destination.groups, ['ESP32 core'])
    assert.ok(destination.selected.length)
    await clickText('Back')
    await ready()
    const restored = await assertUnion(['Camera', 'Camera power'])
    assert.deepEqual(restored.selected, before.selected)
    assert.equal(restored.net, before.net)
    for (const key of ['x', 'y', 'zoom']) assert.ok(Math.abs(restored.viewport[key] - before.viewport[key]) < 0.01, `Back lost viewport ${key}: ${JSON.stringify({ before, restored })}`)
    await page.setViewport({ width: viewport.width - 25, height: viewport.height - 30 })
    await ready()
    const resized = await inspect()
    for (const [dimension, coordinate] of [['width', 'x'], ['height', 'y']]) {
      const center = (v) => (v[dimension] / 2 - v[coordinate]) / v.zoom
      const identity = before.identity
      const projectedStart = identity && identity[coordinate] + (resized.viewport[dimension] - before.viewport[dimension]) / 2
      const fitsCentered = !identity || projectedStart >= 16 && projectedStart + identity[dimension] <= resized.viewport[dimension] - 16
      if (fitsCentered) assert.ok(Math.abs(center(resized.viewport) - center(before.viewport)) < 0.01, `Resize lost logical ${coordinate} center with visible identity`)
      else {
        assert.ok(resized.identity[coordinate] >= 15, 'Resize recovers selected identity instead of retaining a clipped header')
        if (resized.identity[dimension] <= resized.viewport[dimension] - 32)
          assert.ok(resized.identity[coordinate] + resized.identity[dimension] <= resized.viewport[dimension] - 15, 'Recovered identity stays within canvas')
      }
    }
    assert.equal(resized.viewport.zoom, before.viewport.zoom)
    await clickText('Fit circuit')
    await ready()
    await geometry(`${label}-restored-fit`)
    await clickText('Clear filters')
    await ready()
    await clickText('Expand all groups')
    await ready()
    await assertUnion([])
    await clickText('Other')
    await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false')
    assert.equal(Number((await inspect()).counts.parts), 0)
    await page.waitForSelector('.empty-recovery', { visible: true })
    await page.click('.empty-recovery')
    await ready()
    await clickText('Expand all groups')
    await ready()
    await assertUnion([])
    snapshots.push({ label, endpoints, controlsOnly, controlsRestored, before, destination, restored, resized })
  }
  writeFileSync(resolve(output, 'union-inspection.json'), JSON.stringify({ result: 'PASS', snapshots }, null, 2))
}
