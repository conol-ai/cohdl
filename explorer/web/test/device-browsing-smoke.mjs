import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'
import { resolveChromeExecutablePath } from './browser-launch.mjs'

const argument = (name, fallback) => {
  const index = process.argv.indexOf(name)
  return index < 0 ? fallback : process.argv[index + 1] ?? fallback
}
const base = argument('--url', 'http://127.0.0.1:5210')
const output = resolve(argument('--output', '/tmp/cohdl-device-browsing'))
mkdirSync(output, { recursive: true })
const browser = await puppeteer.launch({
  executablePath: resolveChromeExecutablePath(), headless: true,
})
const results = []
const errors = []
const failedResponses = []
let page
const settle = async () => {
  await page.waitForFunction(() => {
    const inspect = JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}')
    const range = document.querySelector('.inspection-range')?.textContent
    const expanded = inspect.mode === 'sch' && !inspect.collapsed?.length
    return document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
      && document.querySelector('.react-flow__node:not(.react-flow__node-region)')
      && inspect.version === 2 && inspect.viewport !== null
      && range === (inspect.groups?.length ? `${inspect.groups.join(' + ')} · ${document.querySelector('.inspection-counts')?.dataset.parts} parts` : inspect.mode === 'overview' ? 'System index' : 'All groups')
      && (!expanded || document.querySelectorAll('.device-card').length === Number(document.querySelector('.inspection-counts')?.dataset.parts))
      && !document.body.innerText.includes('Loading groups...')
  })
  await new Promise(resolve => setTimeout(resolve, 350))
}
const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}'))
const command = async label => {
  const handle = await page.evaluateHandle(label => [...document.querySelectorAll('button')].find(button =>
    button.getAttribute('aria-label') === label || button.textContent.trim() === label), label)
  assert.ok(handle.asElement(), `Missing command: ${label}`)
  await handle.asElement().click()
  await handle.dispose()
}
const visit = async (path = '') => {
  await page.goto(`${base}/${path}`, { waitUntil: 'networkidle2' })
  await settle()
}
const capture = async name => {
  const data = await page.evaluate(() => {
    const rect = element => {
      const r = element.getBoundingClientRect()
      return { x: r.x, y: r.y, width: r.width, height: r.height, right: r.right, bottom: r.bottom }
    }
    const canvas = rect(document.querySelector('.explorer-canvas'))
    const visible = r => r.right > canvas.x && r.x < canvas.right && r.bottom > canvas.y && r.y < canvas.bottom
    const transform = getComputedStyle(document.querySelector('.react-flow__viewport')).transform
    const zoom = transform === 'none' ? 1 : Math.abs(new DOMMatrix(transform).a)
    const cards = [...document.querySelectorAll('.device-card')].map(card => {
      const header = card.querySelector('.device-header')
      const pin = card.querySelector('.device-pin-row')
      const headerRect = header && rect(header)
      return {
        id: card.closest('.react-flow__node')?.dataset.id, text: card.innerText,
        bounds: rect(card), header: headerRect, headerVisible: headerRect && visible(headerRect)
          && headerRect.y >= canvas.y && headerRect.bottom <= canvas.bottom
          && headerRect.x >= canvas.x && headerRect.right <= canvas.right,
        headingPixels: header && parseFloat(getComputedStyle(header.firstElementChild ?? header).fontSize) * zoom,
        pinPixels: pin && parseFloat(getComputedStyle(pin).fontSize) * zoom,
        footprint: !!card.querySelector('.device-footprint'),
        footprintText: card.querySelector('.device-footprint')?.textContent,
        pinRows: [...card.querySelectorAll('.device-pin-row')].map(row => row.textContent),
      }
    })
    return { url: location.href, canvas, zoom, cards, viewport: { width: innerWidth, height: innerHeight },
      overflow: document.documentElement.scrollWidth > innerWidth,
      groups: document.querySelector('.group-selector')?.textContent,
      edges: document.querySelectorAll('.react-flow__edge').length }
  })
  await page.screenshot({ path: resolve(output, `${name}.png`) })
  writeFileSync(resolve(output, `${name}.json`), JSON.stringify(data, null, 2))
  results.push({ name, ...data })
  return data
}
const assertReadable = data => {
  assert.equal(data.overflow, false, 'No horizontal document overflow')
  assert.ok(data.canvas.height >= data.viewport.height * (data.viewport.width < 700 ? .4 : .5), 'Canvas keeps useful height')
  assert.ok(data.cards.length > 0, 'Expanded actual device cards')
  assert.ok(data.cards.every(card => card.footprint), 'Every device has a real local preview or honest fallback')
  const primary = data.cards.filter(card => card.headerVisible)
  assert.ok(primary.length, 'At least one complete identity is initially visible')
  assert.ok(primary.some(card => card.headingPixels >= 12 && card.pinPixels >= 9), 'Primary device and pins are readable at entry scale')
}

try {
  page = await browser.newPage()
  page.on('pageerror', error => errors.push(error.message))
  page.on('response', response => { if (response.status() >= 400) failedResponses.push({ url: response.url(), status: response.status() }) })
  await page.setViewport({ width: 1280, height: 800 })
  await visit()
  const initial = await context()
  assert.equal(initial.mode, 'overview', 'Clean URL defaults to system index')
  assert.ok(initial.collapsed.length, 'Clean URL groups indexed')
  assert.equal(initial.view, 'Functional', 'Functional default')
  await capture('clean-1280')

  const model = await page.evaluate(async () => (await fetch('/api/model')).json())
  const config = await page.evaluate(async design => (await fetch(`/views/${encodeURIComponent(design)}.view.json`)).json(), model.design)
  const functional = config.views.find(view => view.name === 'Functional')
  assert.ok(functional, 'Real project has Functional groups')
  const groupNames = ['ESP32 USB', 'Camera', 'SF32 core and clocks']
  for (const group of groupNames) assert.ok(functional.regions.some(region => region.name === group), `Actual ${group} group`)

  // Delay authoritative config to reproduce the original restored-filter race.
  await page.setRequestInterception(true)
  const delayViews = async request => {
    if (request.url().includes('/views/')) await new Promise(resolve => setTimeout(resolve, 700))
    await request.continue()
  }
  page.on('request', delayViews)
  for (const group of groupNames) {
    const legacy = { ...initial, mode: 'overview', groups: [group], collapsed: functional.regions.map(region => region.name) }
    delete legacy.version
    delete legacy.schemaVersion
    await visit(`?inspect=${encodeURIComponent(JSON.stringify(legacy))}`)
    const migrated = await context()
    assert.equal(migrated.mode, 'sch', 'Legacy overview migrates once')
    assert.deepEqual(migrated.groups, [group], 'Late config preserves original group')
    assert.deepEqual(migrated.collapsed, [], 'Legacy groups expand')
    await page.reload({ waitUntil: 'domcontentloaded' }); await settle()
    assert.deepEqual((await context()).groups, [group], 'Refresh retains migrated group')
  }
  page.off('request', delayViews)
  await page.setRequestInterception(false)

  for (const size of [{ width: 1280, height: 800 }, { width: 1920, height: 1080 }, { width: 390, height: 844 }]) {
    await page.setViewport(size)
    for (const groups of [...groupNames.map(group => [group]), ['ESP32 USB', 'Camera']]) {
      const inspect = { ...await context(), mode: 'sch', groups, collapsed: [], selection: [], pin: '', net: '', viewport: null }
      await visit(`?inspect=${encodeURIComponent(JSON.stringify(inspect))}`)
      const name = `${size.width}-${groups.join('-').replaceAll(' ', '_')}`
      const image = await capture(name)
      assertReadable(image)
      if (groups.includes('SF32 core and clocks')) {
        const sf32 = image.cards.find(card => card.text.includes('U4 '))
        assert.ok(sf32?.text.includes('SF32LB525UC6'), 'Source-backed SF32 identity has loaded')
        assert.ok(sf32.headerVisible, 'SF32 core identity header stays fully visible on group entry')
      }
      if (groups.includes('Camera')) assert.ok(image.cards.find(card => card.text.includes('CAM1 '))?.headerVisible, 'Camera identity header stays visible on group entry')
      const expectedParts = model.instances.filter(instance => functional.regions.some(region => groups.includes(region.name)
        && region.members.some(member => member === instance.designator || member === `path:${instance.path}`)))
      assert.deepEqual(image.cards.map(card => card.id).sort(), expectedParts.map(instance => instance.path).sort(), 'Every actual member has its own card, including rail passives')
      for (const instance of expectedParts) {
        const card = image.cards.find(card => card.id === instance.path)
        assert.ok(card.text.includes(instance.designator), 'Actual designator is shown')
        if (instance.part.mpn) assert.ok(card.text.includes(instance.part.mpn), 'Exact known MPN is shown')
        for (const pin of instance.pins) assert.ok(card.pinRows.some(row => row.includes(pin.logical)), `Actual logical pin ${instance.designator}.${pin.logical} is accessible`)
      }
      if ((size.width === 390 || size.width === 1280) && groups.length === 1 && groups[0] === 'SF32 core and clocks') {
        const core = image.cards.find(card => card.text.includes('U4 '))
        const canvas = image.canvas
        await page.mouse.move(canvas.right - 8, canvas.bottom - 30)
        await page.mouse.down({ button: 'middle' })
        await page.mouse.move(canvas.right - 8, canvas.bottom - 20, { steps: 5 })
        await page.mouse.up({ button: 'middle' }); await settle()
        const header = await page.$eval(`.device-card[data-instance="${core.id}"] .device-header`, element => {
          const r = element.getBoundingClientRect(); return { x: r.x + 30, y: r.y + 20 }
        })
        await page.mouse.click(header.x, header.y)
        await page.waitForSelector('.part-details'); await settle()
        const selected = await capture(`${size.width}-SF32-manual-camera-details`)
        assert.ok(selected.cards.find(card => card.id === core.id)?.headerVisible, 'Selected tall-device identity remains visible after details resize')
        assert.equal(selected.zoom, image.zoom, 'Details recovery preserves manual zoom')
      }
    }
  }
  await page.setViewport({ width: 1280, height: 800 })
  await visit()
  const search = await page.$('input[aria-label="Search groups"]')
  assert.ok(search, 'Stable searchable Groups panel')
  await search.type('Camera')
  const checkbox = await page.evaluateHandle(() => [...document.querySelectorAll('.group-selector label')]
    .find(label => label.textContent.includes('Camera'))?.querySelector('input[type="checkbox"]'))
  assert.ok(checkbox.asElement(), 'Search reveals actual Camera checkbox')
  await checkbox.asElement().focus()
  await page.keyboard.press('Space'); await settle()
  assert.deepEqual((await context()).groups, ['Camera'], 'Keyboard checkbox starts subset')
  await search.click({ clickCount: 3 }); await page.keyboard.press('Backspace'); await search.type('USB')
  assert.deepEqual((await context()).groups, ['Camera'], 'Search never silently changes scope')
  await capture('groups-hidden-selection')
  await command('Show all groups'); await settle()
  assert.deepEqual((await context()).groups, [], 'All clears subset to unfiltered circuit')
  await command('Bundled'); await settle()
  const bundled = await context()
  await page.reload({ waitUntil: 'domcontentloaded' }); await settle()
  assert.deepEqual(await context(), bundled, 'Manual wire state and camera survive refresh')
  await command('Overview'); await settle()
  assert.equal((await context()).mode, 'overview')
  await page.reload({ waitUntil: 'domcontentloaded' }); await settle()
  assert.equal((await context()).mode, 'overview', 'New manual Overview survives refresh')
  await command('SCH view'); await settle()
  for (const netName of ['CAM_HREF', 'GND']) {
    const search = await page.$('input[aria-label="Search circuit"]')
    await search.click({ clickCount: 3 }); await page.keyboard.press('Backspace'); await search.type(netName)
    const target = await page.evaluateHandle(name => [...document.querySelectorAll('button')].find(button =>
      button.textContent.includes(name) && button.textContent.includes('pins')), netName)
    assert.ok(target.asElement(), `Actual network search ${netName}`)
    await target.asElement().click(); await target.dispose(); await settle()
    await page.waitForSelector('[data-endpoint]')
    const actual = await page.$$eval('[data-endpoint]', rows => rows.map(row => row.dataset.endpoint).sort())
    assert.deepEqual(actual, model.nets.find(net => net.name === netName).members.map(member => member.instance_path).sort(), 'Signal/rail trace shows complete actual endpoints')
    await capture(`trace-${netName}`)
  }
  assert.deepEqual(errors, [], 'No runtime errors')
  writeFileSync(resolve(output, 'summary.json'), JSON.stringify({ result: 'PASS', results, errors, failedResponses }, null, 2))
  console.log(JSON.stringify({ result: 'PASS', captures: results.length, output }))
} catch (error) {
  if (page) await capture('failure-state').catch(() => {})
  writeFileSync(resolve(output, 'failure.json'), JSON.stringify({ error: String(error), errors, failedResponses }, null, 2))
  throw error
} finally {
  await browser.close()
}
