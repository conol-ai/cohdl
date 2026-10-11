import assert from 'node:assert/strict'
import { writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

const pause = ms => new Promise(resolve => setTimeout(resolve, ms))
const context = async page => page.evaluate(() => {
  const value = JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}')
  return Object.fromEntries(['mode', 'scope', 'view', 'groups', 'collapsed', 'selection', 'pin', 'net', 'layoutFrame', 'layoutSide'].map(key => [key, value[key]]))
})

async function inspectReal(page, { url, output, ready, clickText }, width, height, retained) {
  await page.setViewport({ width, height })
  await page.goto(url, { waitUntil: 'domcontentloaded' })
  await ready()
  const before = await context(page)
  await clickText('Proposals')
  await page.waitForSelector('.proposal-picker select')
  await page.select('.proposal-picker select', retained.run.id)
  await page.waitForFunction(id => document.querySelector('.proposal-id')?.textContent.startsWith(id) && !document.querySelector('.proposal-review [role="alert"]'), retained.run.id)
  const expected = retained.run.proposal
  const observed = await page.evaluate(() => ({
    title: document.querySelector('.proposal-title h3')?.textContent,
    old: [...document.querySelectorAll('[data-diff="old"]')].map(el => el.textContent),
    new: [...document.querySelectorAll('[data-diff="new"]')].map(el => el.textContent),
    text: document.querySelector('.proposal-review')?.textContent,
    links: [...document.querySelectorAll('.proposal-review a')].map(a => a.getAttribute('href')),
    state: document.querySelector('.proposal-state')?.dataset,
  }))
  assert.equal(observed.title, expected.title)
  assert.deepEqual(observed.old, expected.changes.map(c => c.old))
  assert.deepEqual(observed.new, expected.changes.map(c => c.new))
  assert.ok(observed.text.includes(expected.rationale))
  assert.ok(observed.text.includes(expected.uncertainty || 'None stated'))
  assert.ok(observed.text.includes('Compiler verdict: pass'))
  assert.ok(observed.text.includes('Apply: applied'))
  assert.ok(observed.text.includes('Revert: reverted'))
  assert.ok(observed.text.includes('Fresh apply validation: PASS'))
  assert.equal(observed.state.status, 'reverted')
  assert.equal(observed.state.freshness, 'fresh')
  for (const citation of expected.citations) {
    if (citation.source) assert.ok(observed.text.includes(`${citation.source.path}:${citation.source.line}`))
    if (citation.document) assert.ok(observed.links.some(href => href?.startsWith(`/api/documents/${citation.document}`)))
  }
  const source = await page.$('.proposal-citation .source-excerpt summary')
  if (source) {
    await source.click()
    await page.waitForSelector('.proposal-citation pre[aria-label="CoHDL source excerpt"]')
  }
  await page.evaluate(() => { document.querySelector('.proposal-review').scrollIntoView(); document.querySelector('.explorer-inspector').scrollTop = 0 })
  const bounds = await page.evaluate(() => {
    const inspector = document.querySelector('.explorer-inspector').getBoundingClientRect()
    return { overflow: document.documentElement.scrollWidth > innerWidth, inspector: { x: inspector.x, width: inspector.width, height: inspector.height }, clipped: [...document.querySelectorAll('.proposal-review button, .proposal-review select')].some(el => el.getBoundingClientRect().right > inspector.right + 1) }
  })
  assert.equal(bounds.overflow, false)
  assert.equal(bounds.clipped, false)
  await page.screenshot({ path: resolve(output, `proposals-real-${width}.png`), fullPage: true })
  await clickText('Back to Circuit')
  await ready()
  assert.deepEqual(await context(page), before, 'Proposal review changed circuit context')
  writeFileSync(resolve(output, `proposals-real-${width}.json`), JSON.stringify({ id: retained.run.id, observed, bounds, before, after: await context(page) }, null, 2))
}

async function recovery(page, { url, output, ready, clickText }, retained) {
  const realSummary = { id: retained.run.id, title: retained.run.proposal.title, kind: retained.run.kind, status: retained.status, freshness: retained.freshness, issues: retained.issues }
  const invalid = { id: 'declared-invalid-fixture', title: 'Declared invalid fixture', kind: null, status: 'rejected', freshness: 'unknown', issues: ['Malformed run.json'] }
  const pending = { id: 'declared-pending-fixture', title: 'Declared pending fixture', kind: 'plan', status: 'pending', freshness: 'fresh', issues: [] }
  let mode = 'empty'
  let delayed = false
  const handle = async request => {
    const path = new URL(request.url()).pathname
    if (!path.startsWith('/api/proposals')) return request.continue()
    if (mode === 'error') return request.respond({ status: 503, contentType: 'application/json', body: JSON.stringify({ error: 'Declared recovery fixture unavailable' }) })
    if (path === '/api/proposals') {
      const runs = mode === 'empty' ? [] : [invalid, pending, realSummary]
      return request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify({ schema: 1, runs }) })
    }
    const id = path.split('/')[3]
    if (id === retained.run.id) return request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(retained) })
    const summary = id === pending.id ? pending : invalid
    if (delayed && id === pending.id) await pause(600)
    return request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify({ schema: 1, status: summary.status, freshness: summary.freshness, issues: summary.issues, artifacts: [], run: { id, proposal: null, bundle: null, validation: null, receipt: null } }) }).catch(() => {})
  }
  await page.setRequestInterception(true)
  page.on('request', handle)
  try {
    await page.goto(url, { waitUntil: 'domcontentloaded' }); await ready(); await clickText('Proposals')
    await page.waitForFunction(() => document.querySelector('.proposal-review')?.textContent.includes('No proposals'))
    mode = 'error'; await clickText('Refresh proposals')
    await page.waitForFunction(() => document.querySelector('.proposal-review [role="alert"]')?.textContent.includes('Declared recovery fixture unavailable'))
    mode = 'records'; await clickText('Refresh proposals')
    await page.waitForFunction(() => document.querySelector('.proposal-state')?.dataset.status === 'rejected')
    await page.select('.proposal-picker select', pending.id)
    await page.waitForFunction(() => document.querySelector('.proposal-state')?.dataset.status === 'pending')
    delayed = true
    await page.select('.proposal-picker select', invalid.id)
    await page.waitForFunction(() => document.querySelector('.proposal-state')?.dataset.status === 'rejected')
    await page.select('.proposal-picker select', pending.id)
    await pause(100)
    await page.select('.proposal-picker select', retained.run.id)
    await page.waitForFunction(id => document.querySelector('.proposal-id')?.textContent.startsWith(id), retained.run.id)
    await pause(800)
    assert.ok((await page.$eval('.proposal-id', el => el.textContent)).startsWith(retained.run.id), 'Obsolete proposal response replaced selected run')
    writeFileSync(resolve(output, 'proposals-declared-recovery.json'), JSON.stringify({ fixture: true, states: ['empty', 'error', 'rejected', 'pending'], obsoleteResponseIgnored: true }, null, 2))
  } finally {
    page.off('request', handle)
    await page.setRequestInterception(false)
  }
}

export async function runProposalsSmoke(page, options) {
  const list = await fetch(new URL('/api/proposals', options.url)).then(r => r.json())
  assert.equal(list.schema, 1)
  let retained
  for (const summary of list.runs) {
    if (summary.status !== 'reverted' || summary.freshness !== 'fresh') continue
    const detail = await fetch(new URL(`/api/proposals/${summary.id}`, options.url)).then(r => r.json())
    if (detail.run.proposal?.changes.length && detail.run.provider?.engine === 'codex' && detail.run.provider.command.includes('exec') && detail.run.validation?.decision === 'PASS' && detail.run.receipt?.revert?.outcome === 'reverted') { retained = detail; break }
  }
  assert.ok(retained, 'A real persisted Codex proposal with compiler PASS and guarded revert receipt is required')
  await inspectReal(page, options, 1280, 800, retained)
  await inspectReal(page, options, 390, 844, retained)
  await recovery(page, options, retained)
}
