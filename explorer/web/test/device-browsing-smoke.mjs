import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'

const argument = (name, fallback) => {
  const index = process.argv.indexOf(name)
  return index < 0 ? fallback : process.argv[index + 1] ?? fallback
}
const base = argument('--url', 'http://127.0.0.1:5210')
const output = resolve(argument('--output', '/tmp/cohdl-device-browsing'))
mkdirSync(output, { recursive: true })
const browser = await puppeteer.launch({
  executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true,
})
const results = []
const errors = []
let page
const settle = async () => {
  await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
    && document.querySelector('.react-flow__node:not(.react-flow__node-region)'))
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
  await page.goto(`${base}/${path}`, { waitUntil: 'domcontentloaded' })
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
        bounds: rect(card), header: headerRect, headerVisible: headerRect && visible(headerRect),
        headingPixels: header && parseFloat(getComputedStyle(header).fontSize) * zoom,
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
  await page.setViewport({ width: 1280, height: 800 })
  await visit()
  const initial = await context()
  assert.equal(initial.mode, 'sch', 'Clean URL defaults to device SCH')
  assert.deepEqual(initial.collapsed, [], 'Clean URL groups expanded')
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
  page.on('request', async request => {
    if (request.url().includes('/views/')) await new Promise(resolve => setTimeout(resolve, 700))
    await request.continue()
  })
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
  await page.setRequestInterception(false)

  for (const size of [{ width: 1280, height: 800 }, { width: 1920, height: 1080 }, { width: 390, height: 844 }]) {
    await page.setViewport(size)
    for (const groups of [...groupNames.map(group => [group]), ['ESP32 USB', 'Camera']]) {
      const inspect = { ...await context(), mode: 'sch', groups, collapsed: [], selection: [], pin: '', net: '', viewport: null }
      await visit(`?inspect=${encodeURIComponent(JSON.stringify(inspect))}`)
      const name = `${size.width}-${groups.join('-').replaceAll(' ', '_')}`
      assertReadable(await capture(name))
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
  assert.deepEqual(errors, [], 'No runtime errors')
  writeFileSync(resolve(output, 'summary.json'), JSON.stringify({ result: 'PASS', results, errors }, null, 2))
  console.log(JSON.stringify({ result: 'PASS', captures: results.length, output }))
} catch (error) {
  if (page) await capture('failure').catch(() => {})
  writeFileSync(resolve(output, 'failure.json'), JSON.stringify({ error: String(error), errors }, null, 2))
  throw error
} finally {
  await browser.close()
}
