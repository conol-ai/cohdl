import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runGroupsSmoke(page, { url, output, ready, clickText, geometry }) {
  const records = []
  const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport)
    await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
    const model = await page.evaluate(async () => (await fetch('/api/model')).json())
    const view = await page.evaluate(async design => {
      const config = await (await fetch(`/views/${encodeURIComponent(design)}.view.json`)).json()
      const inspection = JSON.parse(new URL(location.href).searchParams.get('inspect'))
      return config.views.find(view => view.name === inspection.view)
    }, model.design)
    const groups = await page.$$eval('.react-flow__node', nodes => nodes.filter(node => node.dataset.id.startsWith('group:')).map(node => JSON.parse(decodeURIComponent(node.dataset.id.slice(6)))[1]))
    assert.deepEqual([...groups].sort(), view.regions.map(region => region.name).sort(), 'Default overview represents actual native view modules')
    assert.equal(Number(await page.$eval('.inspection-counts', el => el.dataset.parts)), model.instances.length)
    const overview = await context()
    assert.deepEqual(overview.groups, [])
    assert.ok(groups.every(group => overview.collapsed.includes(group)))
    assert.deepEqual(overview.selection, []); assert.equal(overview.net, '')
    await geometry(`${label}-all`, { verticalFit: label === 'desktop' })
    let pan
    if (label === 'mobile') {
      const last = await page.$$eval('.react-flow__node', nodes => nodes.filter(node => node.dataset.id.startsWith('group:')).sort((a, b) => b.getBoundingClientRect().bottom - a.getBoundingClientRect().bottom)[0].dataset.id)
      const bounds = () => page.evaluate(id => {
        const canvas = document.querySelector('.explorer-canvas').getBoundingClientRect()
        const node = [...document.querySelectorAll('.react-flow__node')].find(node => node.dataset.id === id).getBoundingClientRect()
        return { canvas: { x: canvas.x, y: canvas.y, right: canvas.right, bottom: canvas.bottom }, node: { x: node.x, y: node.y, right: node.right, bottom: node.bottom }, visible: node.y >= canvas.y && node.bottom <= canvas.bottom }
      }, last)
      const before = await bounds()
      await page.mouse.move(before.canvas.right - 4, before.canvas.y + 20)
      for (let attempt = 0; attempt < 3 && !(await bounds()).visible; attempt++) {
        const current = await bounds()
        await page.mouse.wheel({ deltaY: Math.max(0, current.node.bottom - current.canvas.bottom + 20) * 2 })
        await new Promise(resolve => setTimeout(resolve, 250))
      }
      const after = await bounds()
      assert.ok(after.visible, 'Final collapsed module is reachable through native vertical pan')
      assert.ok(after.node.x >= after.canvas.x && after.node.right <= after.canvas.right)
      await page.screenshot({ path: resolve(output, 'groups-mobile-final-module-pan.png') })
      pan = { last, before, after, input: 'Native mouse wheel over canvas' }
    }
    await clickText('Camera'); await ready()
    const camera = await context()
    assert.deepEqual(camera.groups, ['Camera'])
    assert.deepEqual(camera.selection, []); assert.equal(camera.pin, ''); assert.equal(camera.net, '')
    assert.ok(!camera.collapsed.includes('Camera'), 'Selected module expands its actual components')
    const expected = model.instances.filter(instance => view.regions.find(region => region.members.some(member => member === instance.designator || member === `path:${instance.path}`))?.name === 'Camera').map(instance => instance.path).sort()
    const actual = await page.$$eval('.react-flow__node', nodes => nodes.map(node => node.dataset.id).filter(id => !/^(region:|net:|bundle:|group:)/.test(id)).sort())
    assert.deepEqual(actual, expected, 'Selected Camera displays its real member identities')
    await geometry(`${label}-camera`)
    await page.reload({ waitUntil: 'domcontentloaded' }); await ready()
    assert.deepEqual((await context()).groups, ['Camera'], 'Reload preserves explicit module range')
    records.push({ label, overview, groups, camera, expected, pan })
  }
  writeFileSync(resolve(output, 'groups-inspection.json'), JSON.stringify({ result: 'PASS', records }, null, 2))
}
