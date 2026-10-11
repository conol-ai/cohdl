import assert from 'node:assert/strict'
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs'
import { join } from 'node:path'
import { isDeepStrictEqual } from 'node:util'
import { bounds, pause, poll, sha } from './multi-tab-runtime.mjs'

export async function ready(tab) {
  await tab.page.waitForFunction(() => document.querySelectorAll('.group-selector input').length >= 26
    && document.querySelector('.inspection-counts')?.getAttribute('data-total-parts') === '233'
    && document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
    && !document.body.innerText.includes('Loading groups...'), { timeout: bounds.readiness_ms, polling: 100 }).catch(error => { throw Error(`Tab ${tab.record.index} completed-layout readiness: ${error}`) })
  await poll(() => !!tab.latest.registry, `Tab ${tab.record.index} document readiness`, bounds.readiness_ms)
  assert.equal(await tab.page.$$eval('.canvas-state[role="alert"]', nodes => nodes.map(node => node.textContent).join('')), '')
}
async function observe(tabs) {
  return Promise.all(tabs.map(async tab => {
    const events = await tab.page.evaluate(() => window.__multitab)
    tab.record.events = events
    return events.messages.at(-1)?.version ?? 0
  }))
}
function content(model) { const { live_error, ...retained } = model; return retained }
export async function transitions(tabs, { project, output, report, save }) {
  const all = check => poll(async () => (await Promise.all(tabs.map(check))).every(Boolean), 'All eight tabs converge')
  const step = async (name, mutate, check) => {
    const previous = await observe(tabs), started_at = new Date().toISOString()
    report.in_progress = { name, stage: 'authoritative state' }; save()
    mutate()
    await all(check)
    report.in_progress.stage = 'new SSE version'; save()
    await all(async (tab, index) => (await tab.page.evaluate(() => window.__multitab.messages.at(-1)?.version ?? 0)) > previous[index])
    report.in_progress.stage = 'completed layouts'; save()
    await Promise.all(tabs.map(ready))
    report.transitions.push({ name, started_at, finished_at: new Date().toISOString(), versions_before: previous, versions_after: await observe(tabs) })
    save()
  }
  await observe(tabs)
  const before = tabs.map(tab => tab.record.requests.filter(request => request.path === '/api/model').length)
  const messages = tabs.map(tab => tab.record.events.messages.length)
  await pause(2400)
  await observe(tabs)
  const after = tabs.map(tab => tab.record.requests.filter(request => request.path === '/api/model').length)
  assert.deepEqual(after, before, 'Same-version reconnect must not reload the model')
  for (let index = 5; index < 8; index++) assert.ok(tabs[index].record.events.messages.length >= messages[index] + 2, `Tab ${index + 1} did not retry twice`)
  report.dedup = { model_requests_before: before, model_requests_after: after, unchanged_versions_retried: true, observation_ms: 2400 }
  const viewPath = join(project, 'views/PDFExact.view.json'), view = JSON.parse(readFileSync(viewPath))
  view.views.find(item => item.name === 'Functional').regions[0].name += ' multitab'
  const changedGroup = view.views.find(item => item.name === 'Functional').regions[0].name
  await step('view watcher reaches admitted and overflow tabs', () => writeFileSync(viewPath, JSON.stringify(view)),
    tab => tab.page.evaluate(name => [...document.querySelectorAll('.group-selector label')].some(label => label.textContent.includes(name)), changedGroup))

  const registryPath = join(project, 'explorer/documents.json'), registry = JSON.parse(readFileSync(registryPath))
  const supportPath = join(project, 'docs/multitab.md'), text = '# Multi-tab evidence\nActual local document update.\n'
  mkdirSync(join(project, 'docs'), { recursive: true })
  const support = { id: 'multitab-support', title: 'Multi-tab evidence', kind: 'markdown', path: 'docs/multitab.md', source_url: null,
    revision: null, sha256: sha(text), pages: null, review: 'unreviewed', uncertainty: 'Disposable real-file regression fixture',
    applicability: [{ instances: ['PDFExact::component_1'], groups: [], relation: 'exact', page: null, section: 'Multi-tab evidence', facts: null, uncertainty: null }] }
  registry.documents.push(support)
  await step('document registry addition reaches every tab', () => { writeFileSync(supportPath, text); writeFileSync(registryPath, JSON.stringify(registry)) },
    tab => tab.latest.registry?.documents.some(document => document.id === support.id && document.status === 'ready'))
  for (const tab of tabs) {
    await tab.page.type('input[aria-label="Search circuit"]', 'SRC1')
    await tab.page.keyboard.press('Enter')
    await tab.page.waitForSelector('[data-document-id="multitab-support"]', { timeout: bounds.readiness_ms })
  }
  const changedText = '# Multi-tab evidence\nActual changed bytes after registry addition.\n'
  support.title = 'Multi-tab updated evidence'; support.sha256 = sha(changedText)
  await step('document bytes and hash recover through native watcher', () => { writeFileSync(supportPath, changedText); writeFileSync(registryPath, JSON.stringify(registry)) },
    tab => tab.latest.registry?.documents.some(document => document.id === support.id && document.actual_sha256 === support.sha256 && document.status === 'ready')
      && tab.page.evaluate(title => document.querySelector('[data-document-id="multitab-support"] summary')?.textContent === title, support.title))
  for (const tab of tabs) {
    const actual = await tab.page.evaluate(async () => {
      const response = await fetch('/api/documents/multitab-support', { signal: AbortSignal.timeout(6000) })
      return { status: response.status, body: await response.text() }
    })
    assert.deepEqual(actual, { status: 200, body: changedText })
    tab.record.changed_document = { status: actual.status, sha256: sha(actual.body) }
  }

  const mainPath = join(project, 'src/main.cohdl'), main = readFileSync(mainPath, 'utf8')
  const devicePath = join(project, 'src/devices/row_1.cohdl'), device = readFileSync(devicePath, 'utf8')
  const originalManufacturer = tabs[0].latest.model.instances.find(instance => instance.path === 'PDFExact::component_1').part.mfr
  const semantic = device.replace('optional P1: 1 [passive]', 'optional P1: 1 [passive], required MULTITAB: 999 [passive]')
  assert.notEqual(semantic, device)
  let retained = content(tabs[0].latest.model)
  await step('semantic failure retains last passing circuit in every tab', () => writeFileSync(devicePath, semantic), tab => {
    const model = tab.latest.model
    return model?.live_error?.includes('E701') && isDeepStrictEqual(content(model), retained)
  })
  for (const tab of tabs) assert.match(await tab.page.$eval('.explorer-toolbar', node => node.parentElement.innerText), /source currently fails to compile/)
  const recovered = device.replace('#[source_unspecified("mpn", "mfr")]', '#[source_unspecified("mpn")]')
    .replace('primary { mfr: "", mpn: "", footprint: FP_1_V1 }', 'primary { mfr: "MultitabFixture", mpn: "", footprint: FP_1_V1 }')
  assert.notEqual(recovered, device)
  await step('valid source change clears errors and publishes current model', () => writeFileSync(devicePath, recovered),
    tab => !tab.latest.model?.live_error && tab.latest.model?.instances.find(instance => instance.path === 'PDFExact::component_1')?.part?.mfr === 'MultitabFixture')
  retained = content(tabs[0].latest.model)
  await step('parse failure retains the newly passing circuit', () => writeFileSync(mainPath, '@\n'),
    tab => tab.latest.model?.live_error?.includes('E001') && tab.latest.model.live_error.includes('main.cohdl')
      && isDeepStrictEqual(content(tab.latest.model), retained))
  await step('restored valid source recovers all tabs', () => { writeFileSync(mainPath, main); writeFileSync(devicePath, device) },
    tab => !tab.latest.model?.live_error && tab.latest.model?.instances.find(instance => instance.path === 'PDFExact::component_1')?.part?.mfr === originalManufacturer)

  const closed_at = new Date().toISOString()
  await tabs[0].page.close()
  const promoted = await poll(() => tabs.slice(5).find(tab => tab.record.responses.some(response => response.path === '/api/events' && response.connection === 'keep-alive' && response.at >= closed_at)), 'Overflow retry promotes into released capacity')
  report.promotion = { closed_tab: 1, closed_at, promoted_tab: promoted.record.index, observed_at: new Date().toISOString() }
  for (const tab of tabs.slice(1)) {
    await ready(tab)
    tab.record.events = await tab.page.evaluate(() => window.__multitab)
    assert.deepEqual(tab.record.events.parse_errors, [])
    if ([6, 8].includes(tab.record.index)) await tab.page.screenshot({ path: join(output, `tab-${tab.record.index}-recovered.png`) })
  }
  save()
}
