import assert from 'node:assert/strict'
import { test } from 'node:test'
import { createServer } from 'vite'
import puppeteer from 'puppeteer-core'
import { resolveChromeExecutablePath } from './browser-launch.mjs'

test('camera capture retains applied geometry while a resize and restore happen in one commit', async () => {
  const server = await createServer({ server: { host: '127.0.0.1', port: 0 }, logLevel: 'error' })
  let browser
  try {
    await server.listen()
    browser = await puppeteer.launch({ executablePath: resolveChromeExecutablePath(), headless: true })
    const page = await browser.newPage()
    await page.setViewport({ width: 1280, height: 800 })
    const errors = []
    page.on('pageerror', error => errors.push(error.message))
    await page.goto(`http://127.0.0.1:${server.httpServer.address().port}/test/canvas-fit-fixture.html`)
    await page.waitForFunction(() => window.captureCamera?.()?.size.height === 520)
    const before = await page.evaluate(() => window.setManualCamera())
    await page.evaluate(() => window.relocateSelectedCamera())
    await page.waitForFunction(() => {
      const camera = window.captureCamera().viewport
      return 4000 * camera.zoom + camera.x >= 0 && 4000 * camera.zoom + camera.x < 950
        && 5000 * camera.zoom + camera.y >= 0 && (5000 + 136) * camera.zoom + camera.y <= 520
    }, { timeout: 2000 })
    assert.equal((await page.evaluate(() => window.captureCamera())).viewport.zoom, before.viewport.zoom, 'Relayout recovery keeps manual readable scale')
    await page.evaluate(() => window.resetCameraBounds())
    await page.evaluate(() => window.setManualCamera())
    const during = await page.evaluate(() => window.resizeAndRestore())
    assert.equal(during.size.height, before.size.height, 'The previous camera must keep its applied size until resize is committed')
    await page.waitForFunction(() => window.captureCamera?.()?.size.height === 496 && Math.abs(window.captureCamera().viewport.y + 258) < .001)
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
    await page.evaluate(() => window.queueCameraRestore({ viewport: { x: -100000, y: -100000, zoom: .5 }, size: { width: 950, height: 520 }, autoFit: false }))
    await page.waitForFunction(() => window.captureCamera().viewport.x > -10000, { timeout: 2000 })
    await page.evaluate(() => window.queueCameraRestore({ viewport: { x: -100000, y: -100000, zoom: .002 }, size: { width: 950, height: 520 }, autoFit: false }, true))
    await page.waitForFunction(() => window.captureCamera().viewport.zoom > .01, { timeout: 2000 })
    const tall = await page.evaluate(() => window.selectTallCamera())
    assert.equal(tall.viewport.y, 24)
    await page.evaluate(() => window.resizeCanvas(447))
    await page.waitForFunction(() => window.captureCamera().size.height === 447)
    const withDetails = await page.evaluate(() => window.captureCamera())
    assert.equal(withDetails.viewport.zoom, tall.viewport.zoom)
    assert.ok(withDetails.viewport.y >= 16, 'Shrinking a mobile canvas must retain the selected tall card header, not merely its lower rows')
    const manual = { ...withDetails, viewport: { ...withDetails.viewport, y: -80 } }
    await page.evaluate(snapshot => window.queueCameraRestore(snapshot, true), manual)
    await page.waitForFunction(() => Math.abs(window.captureCamera().viewport.y + 80) < .001)
    assert.deepEqual(await page.evaluate(() => window.captureCamera()), manual, 'A same-size explicit restore preserves a valid manual camera even if the user panned its header away')
    const desktop = await page.evaluate(() => window.selectTallCamera(1060))
    await page.evaluate(() => window.resizeCanvas(700, 730))
    await page.waitForFunction(() => window.captureCamera().size.width === 730)
    const desktopDetails = await page.evaluate(() => window.captureCamera())
    assert.equal(desktopDetails.viewport.zoom, desktop.viewport.zoom)
    assert.ok(desktopDetails.viewport.x >= 16, 'Opening desktop details must retain the selected header left edge even while its right half still intersects')
    const horizontalManual = { ...desktopDetails, viewport: { ...desktopDetails.viewport, x: -80 } }
    await page.evaluate(snapshot => window.queueCameraRestore(snapshot, true), horizontalManual)
    await page.waitForFunction(() => Math.abs(window.captureCamera().viewport.x + 80) < .001)
    assert.deepEqual(await page.evaluate(() => window.captureCamera()), horizontalManual)
    assert.deepEqual(errors, [])
  } finally {
    await browser?.close()
    await server.close()
  }
})
