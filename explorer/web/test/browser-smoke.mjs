import assert from 'node:assert/strict'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'

const value = (name, fallback) => { const index = process.argv.indexOf(name); return index < 0 ? fallback : process.argv[index + 1] ?? fallback }
const url = value('--url', 'http://127.0.0.1:5210')
const stage = value('--stage', 'groups')
const output = resolve(value('--output', '/tmp/cohdl-explorer-browser'))
mkdirSync(output, { recursive: true })
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
try {
  const page = await browser.newPage()
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  await page.setViewport({ width: 1280, height: 800 })
  await page.goto(url, { waitUntil: 'domcontentloaded' })
  const ready = async () => {
    await page.waitForSelector('.react-flow__node:not(.react-flow__node-region)', { visible: true })
    await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false')
    await new Promise((r) => setTimeout(r, 180))
  }
  const clickText = async (text) => {
    const button = await page.evaluateHandle((label) => [...document.querySelectorAll('button')].find((el) => el.textContent.trim() === label || el.getAttribute('aria-label') === label)
      ?? [...document.querySelectorAll('.group-selector label')].find((el) => el.textContent.trim() === label)?.querySelector('input'), text)
    assert.ok(await button.asElement(), `Missing control ${text}`)
    await button.asElement().click()
    await button.dispose()
  }
  const geometry = async (label) => {
    const result = await page.evaluate(() => {
      const rect = (el) => { const b = el.getBoundingClientRect(); return { x: b.x, y: b.y, width: b.width, height: b.height, right: b.right, bottom: b.bottom } }
      const canvas = rect(document.querySelector('.explorer-canvas'))
      const toolbar = rect(document.querySelector('.explorer-toolbar'))
      const nodes = [...document.querySelectorAll('.react-flow__node')].map(rect)
      return { canvas, toolbar, nodes, overflow: document.documentElement.scrollWidth > innerWidth }
    })
    assert.ok(result.nodes.length)
    assert.ok(result.canvas.width > 0 && result.canvas.height > 0)
    assert.ok(result.toolbar.bottom <= result.canvas.y + 1, 'Toolbar overlaps canvas')
    assert.equal(result.overflow, false, 'Page overflows viewport')
    for (const node of result.nodes) {
      assert.ok(node.x >= result.canvas.x - 2 && node.right <= result.canvas.right + 2, `Horizontal fit: ${JSON.stringify(node)}`)
      assert.ok(node.y >= result.canvas.y - 2 && node.bottom <= result.canvas.bottom + 2, `Vertical fit: ${JSON.stringify(node)}`)
    }
    await page.screenshot({ path: resolve(output, `${stage}-${label}.png`) })
    writeFileSync(resolve(output, `${stage}-${label}.json`), JSON.stringify(result, null, 2))
    console.log(JSON.stringify({ label, canvas: result.canvas, nodeCount: result.nodes.length, overflow: result.overflow }))
  }
  await ready()
  if (stage === 'union') {
    const { runUnionSmoke } = await import('./union-smoke.mjs')
    await runUnionSmoke(page, { url, output, ready, clickText, geometry })
    const { runLayoutInspectionSmoke } = await import('./layout-inspection-smoke.mjs')
    await runLayoutInspectionSmoke(page, { url, output, ready, clickText })
    assert.deepEqual(errors, [], 'Browser runtime errors')
    console.log(JSON.stringify({ result: 'PASS', stage, output }))
  } else {
  assert.ok(await page.$('.react-flow__node-region'), 'Native group nodes missing')
  await geometry('desktop-all')
  await clickText('Camera')
  await ready()
  await geometry('desktop-camera')
  await page.reload({ waitUntil: 'domcontentloaded' })
  await ready()
  await page.setViewport({ width: 390, height: 844 })
  await ready()
  await geometry('mobile-all')
  await clickText('Camera')
  await ready()
  await geometry('mobile-camera')
  assert.deepEqual(errors, [], 'Browser runtime errors')
  console.log(JSON.stringify({ result: 'PASS', stage, output }))
  }
} finally { await browser.close() }
