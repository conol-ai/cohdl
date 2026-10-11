import assert from 'node:assert/strict'
import { createServer } from 'node:http'

export async function runRefreshSmoke(page, { url, fixture, config, ready, clickText, camera, context, sameCamera }) {
  const streams = new Set()
  let current = fixture
  let delayNext = false
  let releaseStale = null
  const server = createServer(async (request, response) => {
    const path = new URL(request.url, url).pathname
    if (path === '/api/events') {
      response.writeHead(200, { 'Content-Type': 'text/event-stream', 'Cache-Control': 'no-cache' })
      response.write('data: {"version":1}\n\n')
      streams.add(response)
      request.on('close', () => streams.delete(response))
      return
    }
    if (path === '/api/model' || path === `/views/${fixture.design}.view.json`) {
      const body = JSON.stringify(path === '/api/model' ? current : config)
      if (path === '/api/model' && delayNext) {
        delayNext = false
        releaseStale = () => { response.writeHead(200, { 'Content-Type': 'application/json' }); response.end(body) }
        return
      }
      response.writeHead(200, { 'Content-Type': 'application/json' })
      response.end(body)
      return
    }
    try {
      const upstream = await fetch(new URL(request.url, url))
      response.writeHead(upstream.status, { 'Content-Type': upstream.headers.get('content-type') ?? 'application/octet-stream' })
      response.end(Buffer.from(await upstream.arrayBuffer()))
    } catch { response.writeHead(502); response.end('Fixture proxy unavailable') }
  })
  await new Promise((resolve) => server.listen(0, '127.0.0.1', resolve))
  const target = `http://127.0.0.1:${server.address().port}`
  try {
    await page.goto(target, { waitUntil: 'domcontentloaded' }); await ready()
    await page.type('input[aria-label="Search circuit"]', 'U1.1'); await page.keyboard.press('Enter'); await ready()
    await page.click('.react-flow__controls-zoomin'); await new Promise((r) => setTimeout(r, 250))
    const before = await camera(), historySize = await page.evaluate(() => history.length)
    current = { ...fixture, live_error: 'Updated live fixture source error', diagnostics: [...fixture.diagnostics, { code: 'REFRESHED', severity: 'warning', message: 'Fresh native API response fixture' }] }
    for (const stream of streams) stream.write('data: {"version":2}\n\n')
    await page.waitForFunction(() => document.body.textContent.includes('Updated live fixture source error') || document.body.textContent.includes('source currently fails'))
    await ready(); sameCamera(before, await camera(), 'real SSE model refresh')
    assert.equal(await page.evaluate(() => history.length), historySize, 'Model refresh must not push history')
    await clickText('Issues'); await ready()
    await page.waitForSelector('[data-code="REFRESHED"]')
    await page.click('[data-code="ATTRIBUTE"] button')
    await page.waitForSelector('.issue-source-focus')
    current = { ...fixture, live_error: 'Obsolete delayed model response' }
    delayNext = true
    for (const stream of streams) stream.write('data: {"version":3}\n\n')
    const deadline = Date.now() + 5000
    while (!releaseStale && Date.now() < deadline) await new Promise((resolve) => setTimeout(resolve, 10))
    assert.ok(releaseStale, 'Older API response must be held before the newer request')
    current = { ...fixture, live_error: 'Latest native model response', diagnostics: [{ code: 'LATEST', severity: 'warning', message: 'Latest fixture response wins' }] }
    for (const stream of streams) stream.write('data: {"version":4}\n\n')
    await page.waitForSelector('[data-code="LATEST"]')
    assert.equal(await page.$('.issue-source-focus'), null, 'Resolved diagnostic must clear obsolete source focus')
    releaseStale()
    await new Promise((resolve) => setTimeout(resolve, 350))
    assert.ok(await page.$('[data-code="LATEST"]'), 'Delayed old model must not overwrite latest response')
    assert.equal(await page.evaluate(() => document.body.textContent.includes('Obsolete delayed model response')), false)
    const after = await context()
    return { fixture: 'Explicit local HTTP/SSE navigation fixture using real EventSource/API and production frontend assets', before, after, historySize, outOfOrderResponses: 'latest response retained after delayed stale response', resolvedIssueFocus: 'cleared on actual SSE model replacement' }
  } finally {
    for (const stream of streams) stream.end()
    server.closeAllConnections()
    await new Promise((resolve) => server.close(resolve))
  }
}
