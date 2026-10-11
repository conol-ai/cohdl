import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'
import puppeteer from 'puppeteer-core'

test('camera capture retains applied geometry while a resize and restore happen in one commit', async () => {
  const server = await createServer({ server: { host: '127.0.0.1', port: 0 }, logLevel: 'error' })
  let browser
  try {
    await server.listen()
    browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
    const page = await browser.newPage()
    await page.setViewport({ width: 1280, height: 800 })
    const errors = []
    page.on('pageerror', error => errors.push(error.message))
    await page.goto(`http://127.0.0.1:${server.httpServer.address().port}/test/canvas-fit-fixture.html`)
    await page.waitForFunction(() => window.captureCamera?.()?.size.height === 520)
    const before = await page.evaluate(() => window.setManualCamera())
    const during = await page.evaluate(() => window.resizeAndRestore())
    assert.equal(during.size.height, before.size.height, 'The previous camera must keep its applied size until resize is committed')
    await page.waitForFunction(() => window.captureCamera?.()?.size.height === 496 && Math.abs(window.captureCamera().viewport.y + 858) < .001)
    const after = await page.evaluate(() => window.captureCamera())
    assert.equal(after.viewport.zoom, before.viewport.zoom)
    assert.equal((after.size.height / 2 - after.viewport.y) / after.viewport.zoom, (before.size.height / 2 - before.viewport.y) / before.viewport.zoom)
    const queued = { viewport: { x: 100, y: 40, zoom: .43 }, size: { width: 950, height: 496 }, autoFit: false }
    const captured = await page.evaluate(snapshot => window.queueCameraRestore(snapshot), queued)
    assert.deepEqual(captured, queued, 'Capture during pending restore must retain the intended camera, never the previous React Flow store')
    await page.waitForFunction(() => Math.abs(new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform).a - .43) < .00001)
    assert.deepEqual(await page.evaluate(() => window.captureCamera()), queued, 'Applied native viewport matches the queued snapshot')
    const resized = await page.evaluate(() => window.resizeAndRestore(520))
    assert.deepEqual(resized, queued)
    await page.waitForFunction(() => window.captureCamera().size.height === 520 && Math.abs(new DOMMatrixReadOnly(getComputedStyle(document.querySelector('.react-flow__viewport')).transform).f - 52) < .001)
    const final = await page.evaluate(() => window.captureCamera())
    assert.equal(final.viewport.zoom, queued.viewport.zoom)
    assert.equal((final.size.height / 2 - final.viewport.y) / final.viewport.zoom, (queued.size.height / 2 - queued.viewport.y) / queued.viewport.zoom)
    assert.deepEqual(errors, [])
  } finally {
    await browser?.close()
    await server.close()
  }
})
