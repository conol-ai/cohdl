import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { createHash } from 'node:crypto'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'

const option = (name, fallback) => process.argv.includes(name) ? process.argv[process.argv.indexOf(name) + 1] : fallback
const base = option('--url', 'http://127.0.0.1:5210')
const run = option('--run', '')
assert.ok(run, 'Existing real provider run required')
const transaction = process.argv.includes('--save-undo')
const mobile = process.argv.includes('--mobile')
const output = resolve(option('--output', '/tmp/cohdl-group-intent-ui'))
mkdirSync(output, { recursive: true })
const sha = bytes => createHash('sha256').update(bytes).digest('hex')
const originalModel = await (await fetch(`${base}/api/model`)).text()
const model = JSON.parse(originalModel)
const viewUrl = `${base}/views/${encodeURIComponent(model.design)}.view.json`
const originalView = await (await fetch(viewUrl)).text()
writeFileSync(resolve(output, 'original-model.json'), originalModel)
writeFileSync(resolve(output, 'original-view.json'), originalView)
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
const page = await browser.newPage()
const records = [], errors = []
page.on('pageerror', error => errors.push(String(error)))
const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}'))
const settle = async () => {
  await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false' && document.querySelector('.react-flow__node') && ![...document.querySelectorAll('.group-intent [role="status"]')].some(element => /\.\.\.$/.test(element.textContent)))
  await new Promise(resolve => setTimeout(resolve, 400))
}
const click = async label => {
  if (mobile && await page.$eval('.group-panel', element => getComputedStyle(element).display === 'none')) { await page.click('[aria-label="Show Groups"]'); await settle() }
  const target = await page.evaluateHandle(label => [...document.querySelectorAll('.group-intent button')].find(button => button.getAttribute('aria-label') === label || button.textContent.trim() === label), label)
  assert.ok(target.asElement(), label)
  await target.asElement().evaluate(element => element.scrollIntoView({ block: 'center' }))
  await target.asElement().click()
  await target.dispose()
  await settle()
  assert.equal(await page.$('.group-intent [role="alert"]'), null)
}
const capture = async name => {
  const bytes = await (await fetch(`${base}/api/model`)).text()
  assert.equal(bytes, originalModel, `${name}: hardware model unchanged`)
  const view = await (await fetch(viewUrl)).text()
  writeFileSync(resolve(output, `${name}-view.json`), view)
  records.push({ name, context: await context(), model_sha256: sha(bytes), view_sha256: sha(view) })
  if (mobile && await page.$eval('.group-panel', element => getComputedStyle(element).display !== 'none')) await page.click('.group-drawer-close')
  await page.screenshot({ path: resolve(output, `${name}.png`) })
  return view
}
const review = async id => {
  if (mobile && await page.$eval('.group-panel', element => getComputedStyle(element).display === 'none')) { await page.click('[aria-label="Show Groups"]'); await settle() }
  await page.$eval('.group-intent', element => { element.open = true })
  await page.$eval('[aria-label="Group proposal run"]', element => { element.focus(); element.select() })
  await page.keyboard.press('Backspace')
  await page.type('[aria-label="Group proposal run"]', id)
  await click('Review')
}
const assertBaseline = async reason => {
  const current = await context()
  for (const key of ['mode', 'view', 'groups', 'scope', 'selection', 'pin', 'net']) assert.deepEqual(current[key], baseline[key], `${reason}: ${key}`)
}
let baseline
try {
  await page.setViewport(mobile ? { width: 390, height: 844 } : { width: 1440, height: 1000 })
  await page.goto(base, { waitUntil: 'networkidle2' })
  await settle()
  baseline = await context()
  await review(run)
  await capture('review')
  await click('Focus')
  const focused = await context()
  assert.equal(focused.mode, 'sch')
  assert.equal(focused.view, 'Intent ESP32 Camera')
  assert.ok(focused.groups.length)
  assert.deepEqual(focused.collapsed, [])
  assert.equal(Number(await page.$eval('.inspection-counts', element => element.dataset.parts)), 32)
  assert.equal(await capture('focused'), originalView)
  const refresh = await browser.newPage()
  await refresh.setViewport(mobile ? { width: 390, height: 844 } : { width: 1440, height: 1000 })
  await refresh.goto(page.url(), { waitUntil: 'networkidle2' })
  await refresh.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false' && document.body.innerText.includes('Requested view is unavailable or temporary'))
  const refreshed = await refresh.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
  assert.equal(refreshed.mode, 'sch')
  assert.equal(refreshed.view, 'Functional')
  assert.ok(refreshed.groups.length)
  await refresh.screenshot({ path: resolve(output, 'unsaved-refresh.png') })
  await refresh.close()
  if (mobile) { await page.click('[aria-label="Show Groups"]'); await settle() }
  await page.type('[aria-label="Group proposal run"]', '_edited')
  await settle()
  await assertBaseline('Editing the run cancels its temporary focus')
  assert.equal(await page.$('[aria-label="Group proposal preview"]'), null)
  assert.equal(await capture('edited-run-restored'), originalView)
  await review(run)
  await click('Focus')
  await click('Cancel temporary focus')
  await assertBaseline('Cancel restores original baseline')
  assert.equal(await capture('cancelled'), originalView)
  await click('Focus')
  assert.equal((await context()).view, 'Intent ESP32 Camera')
  if (transaction) {
    await click('Save')
    const saved = await capture('saved')
    assert.notEqual(saved, originalView)
    assert.equal((await context()).view, 'Intent ESP32 Camera')
    assert.ok(JSON.parse(saved).views.some(view => view.name === 'Intent ESP32 Camera'))
    await page.reload({ waitUntil: 'networkidle2' })
    await settle()
    await review(run)
    assert.ok(await page.$eval('.group-intent', element => element.innerText.includes('Group changes saved')))
    assert.equal(await capture('saved-reloaded-reviewed'), saved)
    await click('Undo saved group change')
    assert.equal(await capture('undone'), originalView)
    assert.equal((await context()).mode, 'sch')
    assert.ok((await context()).groups.length, 'Undo restores a bounded saved Group')
    const undoneParts = Number(await page.$eval('.inspection-counts', element => element.dataset.parts))
    assert.ok(undoneParts > 0 && undoneParts < model.instances.length, 'Reloaded Undo fallback is bounded rather than all components')
  } else {
    await click('Cancel temporary focus')
  }
  assert.deepEqual(errors, [])
  writeFileSync(resolve(output, 'result.json'), JSON.stringify({ status: 'PASS', run, transaction, mobile, records, errors }, null, 2))
  console.log(JSON.stringify({ status: 'PASS', run, transaction, mobile, records: records.length, output }))
} catch (error) {
  await page.screenshot({ path: resolve(output, 'failure.png') })
  writeFileSync(resolve(output, 'result.json'), JSON.stringify({ status: 'FAIL', run, transaction, error: String(error), records, errors }, null, 2))
  throw error
} finally { await browser.close() }
