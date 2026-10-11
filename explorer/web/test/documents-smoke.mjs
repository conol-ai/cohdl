import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

export async function runDocumentsSmoke(page, { url, output, ready, clickText }) {
  const records = []
  const search = async query => {
    await page.click('input[aria-label="Search circuit"]', { clickCount: 3 }); await page.keyboard.press('Backspace')
    await page.type('input[aria-label="Search circuit"]', query); await page.keyboard.press('Enter'); await ready()
    await page.waitForFunction(() => document.querySelector('.provenance-inspector')?.getAttribute('aria-busy') === 'false')
  }
  const registryResponse = await fetch(new URL('/api/documents', url))
  assert.equal(registryResponse.status, 200)
  const registry = await registryResponse.json()
  for (const [label, viewport] of [['desktop', { width: 1280, height: 800 }], ['mobile', { width: 390, height: 844 }]]) {
    await page.setViewport(viewport)
    await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready()
    await page.type('input[aria-label="Search circuit"]', 'U4')
    await page.keyboard.press('Enter'); await ready()
    assert.ok(await page.$('.provenance-inspector'), 'Native part inspector must expose applicable document evidence')
    await page.waitForFunction(() => document.querySelector('.provenance-inspector')?.getAttribute('aria-busy') === 'false')
    const state = await page.evaluate(() => ({
      text: document.querySelector('.provenance-inspector').innerText,
      links: [...document.querySelectorAll('.provenance-inspector a')].map(a => ({ text: a.textContent, href: a.getAttribute('href') })),
      count: document.querySelector('.inspection-counts')?.dataset.parts,
      overflow: document.documentElement.scrollWidth > innerWidth,
    }))
    assert.match(state.text, /SF32LB525UC6/)
    assert.match(state.text, /Authored empty/)
    assert.match(state.text, /related/i)
    assert.match(state.text, /unreviewed/i)
    assert.ok(state.links.some(l => l.href.endsWith('#page=4')), 'Exact original schematic page')
    assert.ok(state.links.some(l => l.href.endsWith('#page=10')), 'Exact original BOM page')
    assert.equal(state.overflow, false)
    await page.screenshot({ path: resolve(output, `documents-${label}-u4.png`) })
    await page.click('.source-excerpt summary')
    await page.waitForSelector('[data-source-line="433"]')
    const source = await page.$eval('.source-excerpt', el => el.innerText)
    assert.match(source, /component_215/); assert.match(source, /SHA-256/)
    await page.screenshot({ path: resolve(output, `documents-${label}-source.png`) })
    const pdf = await page.evaluate(async () => {
      const link = document.querySelector('[data-document-id="original-schematic"] a')
      const response = await fetch(link.href)
      return { status: response.status, type: response.headers.get('content-type'), signature: new TextDecoder().decode((await response.arrayBuffer()).slice(0, 5)), href: link.href }
    })
    assert.equal(pdf.status, 200); assert.equal(pdf.type, 'application/pdf'); assert.equal(pdf.signature, '%PDF-'); assert.ok(pdf.href.endsWith('#page=4'))
    await search('U3')
    const air = await page.$eval('[data-document-id="air780eg"]', el => ({ text: el.innerText, localLinks: [...el.querySelectorAll('a')].filter(a => a.getAttribute('href').startsWith('/api/')).length }))
    assert.match(air.text, /No local original/); assert.match(air.text, /Page count unknown/); assert.equal(air.localLinks, 0)
    const support = await page.$eval('[data-document-id="air780eg-evidence"]', async el => {
      const response = await fetch(el.querySelector('a').href)
      return { text: el.innerText, status: response.status, type: response.headers.get('content-type'), body: await response.text() }
    })
    assert.match(support.text, /Local supporting Markdown/); assert.match(support.body, /Air780EG/); assert.equal(support.status, 200); assert.match(support.type, /text\/plain/)
    await page.screenshot({ path: resolve(output, `documents-${label}-missing-original.png`) })
    await search('U1')
    const mini = await page.$eval('[data-document-id="esp32-s3-mini-1u"]', el => el.innerText)
    assert.match(mini, /No local original/); assert.match(mini, /WROOM/)
    await clickText('Clear filters'); await ready(); await clickText('LTE modem and RF'); await ready()
    await page.waitForFunction(() => document.querySelector('.provenance-inspector')?.getAttribute('aria-busy') === 'false')
    const group = await page.$eval('.provenance-inspector', el => el.innerText)
    assert.match(group, /Recorded constraints/); assert.match(group, /1\.5 A/); assert.match(group, /original PDF is missing/)
    await page.screenshot({ path: resolve(output, `documents-${label}-group.png`) })
    await clickText('Clear filters'); await ready(); await clickText('Collapse all groups'); await ready()
    const groupNode = await page.evaluateHandle(() => [...document.querySelectorAll('.react-flow__node')].find(n => n.dataset.id?.startsWith('group:') && n.textContent.includes('LTE modem and RF')))
    assert.ok(groupNode.asElement(), 'Collapsed native LTE group exists')
    await groupNode.asElement().click(); await groupNode.dispose()
    await page.waitForFunction(() => document.querySelector('.provenance-inspector')?.textContent.includes('1.5 A'))
    await page.screenshot({ path: resolve(output, `documents-${label}-collapsed-group.png`) })
    await clickText('Expand all groups'); await ready()
    await search('U4')
    const circuitBefore = await page.evaluate(() => ({ count: document.querySelector('.inspection-counts').dataset.parts, nodes: [...document.querySelectorAll('.react-flow__node')].map(n => n.dataset.id).sort() }))
    let fixture = { status: 422, body: { error: 'Unsupported document registry schema (browser response fixture)' } }
    const intercept = request => request.url().endsWith('/api/documents')
      ? request.respond({ status: fixture.status, contentType: 'application/json', body: JSON.stringify(fixture.body) })
      : request.continue()
    await page.setRequestInterception(true); page.on('request', intercept)
    await clickText('Refresh source evidence')
    await page.waitForFunction(() => document.querySelector('.provenance-inspector [role="alert"]')?.textContent.includes('Unsupported document registry'))
    const circuitAfter = await page.evaluate(() => ({ count: document.querySelector('.inspection-counts').dataset.parts, nodes: [...document.querySelectorAll('.react-flow__node')].map(n => n.dataset.id).sort() }))
    assert.deepEqual(circuitAfter, circuitBefore, 'Document config failure retains the actual circuit context')
    await page.screenshot({ path: resolve(output, `documents-${label}-registry-error-fixture.png`) })
    const invalid = structuredClone(registry)
    const sf32 = invalid.documents.find(d => d.id === 'sf32lb52x')
    sf32.pages = 0; sf32.status = 'invalid'; sf32.issues = ['Invalid page count (browser response fixture)']; sf32.open_url = null
    fixture = { status: 200, body: invalid }; await clickText('Refresh source evidence')
    await page.waitForFunction(() => document.querySelector('[data-document-id="sf32lb52x"]')?.textContent.includes('Invalid page count'))
    assert.ok(await page.$('[data-document-id="original-schematic"] a'), 'Invalid record retains independently valid original citation')
    assert.equal(await page.$('[data-document-id="sf32lb52x"] a'), null)
    page.off('request', intercept); await page.setRequestInterception(false)
    await clickText('Refresh source evidence')
    await page.waitForFunction(() => document.querySelector('[data-document-id="sf32lb52x"] .document-status')?.textContent.includes('Local hash verified'))
    records.push({ label, ...state, source, pdf, air, support: { ...support, body: support.body.slice(0, 200) }, mini, group, responseFixtures: ['unsupported-schema', 'invalid-record', 'actual-recovery'] })
  }
  writeFileSync(resolve(output, 'documents-inspection.json'), JSON.stringify({ result: 'PASS', records }, null, 2))
}
