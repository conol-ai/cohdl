import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'

const argument = (name, fallback) => {
  const index = process.argv.indexOf(name)
  return index < 0 ? fallback : process.argv[index + 1] ?? fallback
}
const base = argument('--url', 'http://127.0.0.1:5210')
const output = resolve(argument('--output', '/tmp/cohdl-scoped-hierarchy'))
mkdirSync(output, { recursive: true })
const model = await (await fetch(`${base}/api/model`)).json()
const config = await (await fetch(`${base}/views/${encodeURIComponent(model.design)}.view.json`)).json()
const functional = config.views.find(view => view.name === 'Functional')
assert.ok(functional)
const groupFor = instance => functional.regions.find(region => region.members.some(member =>
  member === instance.designator || member === `path:${instance.path}`))?.name ?? 'Other'
const browser = await puppeteer.launch({
  executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true,
})
const records = [], errors = []
let page
const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}'))
const settle = async () => {
  await page.waitForFunction(() => {
    const inspect = JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}')
    return inspect.version === 2 && inspect.viewport
      && document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
      && document.querySelector('.react-flow__node:not(.react-flow__node-region)')
      && !document.body.innerText.includes('Loading groups...')
  })
  await new Promise(resolve => setTimeout(resolve, 300))
}
const command = async label => {
  const target = await page.evaluateHandle(label => [...document.querySelectorAll('button')].find(button =>
    button.getAttribute('aria-label') === label || button.textContent.trim() === label), label)
  assert.ok(target.asElement(), `Missing command ${label}`)
  await target.asElement().click()
  await target.dispose()
  await settle()
}
const visit = async inspect => {
  const url = new URL(base)
  if (inspect) url.searchParams.set('inspect', JSON.stringify(inspect))
  await page.goto(url.href, { waitUntil: 'networkidle2' })
  await settle()
}
const capture = async name => {
  const data = await page.evaluate(() => {
    const rect = element => {
      const r = element.getBoundingClientRect()
      return { x: r.x, y: r.y, right: r.right, bottom: r.bottom, width: r.width, height: r.height }
    }
    const canvas = rect(document.querySelector('.explorer-canvas'))
    const zoom = Math.abs(new DOMMatrix(getComputedStyle(document.querySelector('.react-flow__viewport')).transform).a)
    return {
      canvas, zoom, width: innerWidth, height: innerHeight,
      overflow: document.documentElement.scrollWidth > innerWidth,
      count: Number(document.querySelector('.inspection-counts')?.dataset.parts),
      range: document.querySelector('.inspection-range')?.textContent,
      index: [...document.querySelectorAll('.group-index-identity')].map(element => element.textContent),
      cards: [...document.querySelectorAll('.device-card')].map(card => {
        const header = card.querySelector('.device-header'), pin = card.querySelector('.device-pin-row')
        return { id: card.dataset.instance, text: card.textContent, header: rect(header),
          headingPixels: parseFloat(getComputedStyle(header.firstElementChild).fontSize) * zoom,
          pinPixels: pin ? parseFloat(getComputedStyle(pin).fontSize) * zoom : null,
          footprint: !!card.querySelector('.device-footprint'),
          pins: [...card.querySelectorAll('.device-pin-row')].map(row => row.textContent) }
      }),
    }
  })
  assert.equal(data.overflow, false, `${name}: document does not overflow`)
  assert.ok(data.canvas.height >= data.height * (data.width < 700 ? .4 : .5), `${name}: useful canvas height`)
  await page.screenshot({ path: resolve(output, `${name}.png`) })
  const record = { name, context: await context(), ...data }
  records.push(record)
  writeFileSync(resolve(output, `${name}.json`), JSON.stringify(record, null, 2))
  return record
}
const assertInternal = (data, groups) => {
  const expected = model.instances.filter(instance => groups.includes(groupFor(instance)))
  assert.deepEqual(data.cards.map(card => card.id).sort(), expected.map(instance => instance.path).sort(), 'Actual Group union only')
  assert.equal(data.count, expected.length, 'Outside destinations do not inflate represented count')
  assert.ok(data.cards.every(card => card.footprint), 'Every internal device has local geometry or honest fallback')
  const readable = data.cards.filter(card => card.header.x >= data.canvas.x && card.header.right <= data.canvas.right
    && card.header.y >= data.canvas.y && card.header.bottom <= data.canvas.bottom)
  assert.ok(readable.some(card => card.headingPixels >= 12 && card.pinPixels >= 9), 'Actual primary identity and pins readable')
  for (const instance of expected) {
    const card = data.cards.find(card => card.id === instance.path)
    assert.ok(card.text.includes(instance.designator))
    if (instance.part.mpn) assert.ok(card.text.includes(instance.part.mpn))
    for (const pin of instance.pins) assert.ok(card.pins.some(row => row.includes(pin.logical)))
  }
}
const toggle = async group => {
  const mobile = await page.evaluate(() => innerWidth < 700)
  if (mobile) await command('Open groups')
  const search = await page.$('input[aria-label="Search groups"]')
  await search.focus(); await search.evaluate(element => element.select())
  await page.keyboard.press('Backspace'); await search.type(group)
  const input = await page.evaluateHandle(group => [...document.querySelectorAll('.group-selector label')]
    .find(label => label.querySelector('span')?.textContent === group)?.querySelector('input[type="checkbox"]'), group)
  assert.ok(input.asElement(), `Actual checkbox ${group}`)
  await input.asElement().click(); await input.dispose(); await settle()
  if (mobile) await command('Close groups')
}
const search = async text => {
  await page.focus('input[aria-label="Search circuit"]')
  await page.$eval('input[aria-label="Search circuit"]', element => element.select())
  await page.keyboard.press('Backspace')
  await page.type('input[aria-label="Search circuit"]', text)
  await page.keyboard.press('Enter'); await settle()
}

try {
  page = await browser.newPage()
  page.on('pageerror', error => errors.push(error.message))
  for (const viewport of [{ width: 1280, height: 800 }, { width: 1920, height: 1080 }, { width: 390, height: 844 }]) {
    await page.setViewport(viewport); await visit()
    const initial = await context()
    assert.equal(initial.mode, 'overview', 'Clean entry is a bounded system index')
    assert.deepEqual(initial.groups, [])
    assert.ok(initial.collapsed.length >= functional.regions.length)
    const index = await capture(`${viewport.width}-index`)
    assert.equal(index.cards.length, 0, 'System index does not expand all233 devices')
    assert.ok(index.index.some(text => text.includes('SF32LB525UC6')), 'Index includes real source-backed core identity')
    assert.ok(index.index.some(text => text.includes('ESP32')), 'Index includes real ESP32 identity')
    const entry = await page.$('.group-index-identity[data-group="Camera"] .group-entry')
    assert.ok(entry, 'System index has a direct Camera entry')
    await entry.focus()
    await page.keyboard.press('Enter'); await settle()
    assert.deepEqual((await context()).groups, ['Camera'], 'Index entry supports keyboard navigation')
    assertInternal(await capture(`${viewport.width}-index-keyboard-Camera`), ['Camera'])
    for (const groups of [['ESP32 USB'], ['Camera'], ['SF32 core and clocks'], ['ESP32 USB', 'Camera']]) {
      await visit()
      for (const group of groups) await toggle(group)
      const inspect = await context()
      assert.equal(inspect.mode, 'sch'); assert.deepEqual(inspect.groups, groups); assert.deepEqual(inspect.collapsed, [])
      assertInternal(await capture(`${viewport.width}-${groups.join('-').replaceAll(' ', '_')}`), groups)
    }
    await visit(); await toggle('Camera'); await toggle('Camera')
    assert.equal((await context()).mode, 'overview', 'Last uncheck returns to system index')
    assert.deepEqual((await context()).groups, [])
    assert.equal(await page.$$eval('.device-card', cards => cards.length), 0)
  }
  for (const viewport of [{ width: 1280, height: 800 }, { width: 390, height: 844 }]) {
    await page.setViewport(viewport); await visit(); await toggle('SF32 core and clocks')
    await search('SF32LB525UC6')
    const u4 = model.instances.find(instance => instance.designator === 'U4')
    assert.deepEqual((await context()).selection, [u4.path], 'Source-only identity search locates actual U4')
    const canvas = await page.$eval('.react-flow__pane', element => {
      const r = element.getBoundingClientRect()
      return { x: r.x + 35, y: r.y + 35 }
    })
    await page.mouse.move(canvas.x, canvas.y); await page.mouse.down()
    await page.mouse.move(canvas.x + 55, canvas.y + 35, { steps: 6 }); await page.mouse.up(); await settle()
    const panned = await context()
    await page.setViewport({ width: viewport.width, height: viewport.height - 40 }); await settle()
    assert.deepEqual((await context()).selection, [u4.path], 'U4 stays selected through manual pan and details resize')
    assert.equal((await context()).viewport.viewport.zoom, panned.viewport.viewport.zoom, 'Resize preserves manual zoom')
    await capture(`${viewport.width}-U4-pan-resize`)
    await page.setViewport(viewport); await settle()
    await search('C263')
    const row = await page.$('.part-details tr[data-pin="P2"]')
    assert.ok(row); await row.evaluate(element => element.scrollIntoView({ block: 'nearest' }))
    await row.click(); await settle()
    assert.equal((await context()).net, 'GND')
    assertInternal(await capture(`${viewport.width}-scoped-GND`), ['SF32 core and clocks'])
    await page.waitForSelector('.outside-destinations')
    const expected = model.nets.find(net => net.name === 'GND').members.filter(member =>
      groupFor(model.instances.find(instance => instance.path === member.instance_path)) !== 'SF32 core and clocks')
    const destinations = await page.$eval('.outside-destinations', element => element.textContent)
    assert.ok(destinations.includes('GND') && destinations.includes('Camera'))
    const actual = await page.$$eval('.outside-destinations li[data-visible="false"]', rows => rows.map(row => ({
      instance_path: row.dataset.instance, logical_pin: row.dataset.pin, numbers: JSON.parse(row.dataset.physical),
    })))
    const tuple = member => JSON.stringify([member.instance_path, member.logical_pin, member.numbers])
    assert.deepEqual(actual.map(tuple).sort(), expected.map(tuple).sort(), 'Every outside destination is an exact original endpoint')
    assert.equal(await page.$$eval('.outside-destinations details[open]', rows => rows.length), 0, 'High-fanout destinations start bounded')
    records.push({ name: `${viewport.width}-outside-GND`, expected, actual, text: destinations })
    const before = await context()
    await page.$eval('.outside-destinations details[data-group="Camera"][data-local="false"] summary', element => element.click())
    const destination = await page.evaluateHandle(() => [...document.querySelectorAll('.outside-destinations button')]
      .find(button => button.getAttribute('aria-label') === 'Open Camera group'))
    assert.ok(destination.asElement(), 'Explicit outside Group destination')
    await destination.asElement().evaluate(element => element.scrollIntoView({ block: 'nearest' }))
    await destination.asElement().click(); await destination.dispose(); await settle()
    assert.deepEqual((await context()).groups, ['Camera'])
    assertInternal(await capture(`${viewport.width}-outside-Camera`), ['Camera'])
    await command('Back')
    const restored = await context()
    assert.deepEqual(restored.groups, before.groups); assert.equal(restored.net, before.net)
    assert.deepEqual(restored.selection, before.selection); assert.equal(restored.pin, before.pin)
    assert.deepEqual(restored.viewport.viewport, before.viewport.viewport, 'Back restores exact valid camera')
    await page.reload({ waitUntil: 'networkidle2' }); await settle()
    assert.deepEqual((await context()).groups, before.groups)
    assert.equal((await context()).mode, 'sch', 'Manual v2 Group detail remains after reload')
  }
  assert.deepEqual(errors, [])
  writeFileSync(resolve(output, 'summary.json'), JSON.stringify({ result: 'PASS', records, errors }, null, 2))
  console.log(JSON.stringify({ result: 'PASS', captures: records.filter(record => record.canvas).length, output }))
} catch (error) {
  if (page) await page.screenshot({ path: resolve(output, 'failure.png') }).catch(() => {})
  writeFileSync(resolve(output, 'failure.json'), JSON.stringify({ result: 'FAIL', error: error.stack, records, errors }, null, 2))
  throw error
} finally {
  await browser.close()
}
