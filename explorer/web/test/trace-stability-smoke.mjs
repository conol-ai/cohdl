import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { mkdirSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import puppeteer from 'puppeteer-core'
import { resolveChromeExecutablePath } from './browser-launch.mjs'

const args = process.argv.slice(2)
const url = args[args.indexOf('--url') + 1] || 'http://127.0.0.1:5210'
const output = resolve(process.env.COHDL_TRACE_OUTPUT || '/tmp/cohdl-trace-stability')
const light = process.env.COHDL_TRACE_LIGHT === '1'
mkdirSync(output, { recursive: true })
const browser = await puppeteer.launch({ executablePath: resolveChromeExecutablePath(), headless: true })
const records = [], bindings = []
const pause = ms => new Promise(resolve => setTimeout(resolve, ms))
try {
  for (const [label, width, height] of [['desktop', 1280, 800], ['narrow', 640, 844], ['mobile', 390, 844]].filter(([label]) => !process.env.COHDL_TRACE_VIEW || label === process.env.COHDL_TRACE_VIEW)) {
    const page = await browser.newPage()
    await page.setViewport({ width, height })
    const cdp = await page.createCDPSession()
    await cdp.send('Debugger.enable')
    const bound = []
    cdp.on('Debugger.scriptParsed', async event => {
      if (!/\/assets\/index-.*\.js$/.test(event.url)) return
      const { scriptSource: source } = await cdp.send('Debugger.getScriptSource', { scriptId: event.scriptId })
      const marker = source.lastIndexOf('"elk.hierarchyHandling":"INCLUDE_CHILDREN"')
      assert.ok(marker >= 0, 'Native layout marker must exist; missing instrumentation cannot pass')
      const start = source.lastIndexOf('async function', marker)
      const header = source.slice(start, source.indexOf('{', start))
      const argument = /async function\s+\w+\((\w+)/.exec(header)?.[1]
      assert.ok(argument, `Actual layout argument not found: ${header}`)
      const engine = /await (\w+)\.layout/.exec(source.slice(start, marker))?.[1]
      assert.ok(engine, 'Actual ELK instance must be bound for controlled async completion proof')
      const offset = source.indexOf('{', start) + 1
      const lines = source.slice(0, offset).split('\n')
      const location = { scriptId: event.scriptId, lineNumber: lines.length - 1, columnNumber: lines.at(-1).length }
      const point = await cdp.send('Debugger.setBreakpoint', { location, condition: `(window.__trace.layout.push({at:performance.now(),nodes:${argument}.nodes.length,edges:${argument}.edges.length,held:!!window.__holdNext}),window.__holdNext?(window.__holdNext=false,window.__originalLayout=${engine}.layout,${engine}.layout=function(...args){const result=window.__originalLayout.apply(this,args);${engine}.layout=window.__originalLayout;return new Promise(resolve=>setTimeout(()=>resolve(result),300))}):0,false)` })
      const binding = { label, url: event.url, sha256: createHash('sha256').update(source).digest('hex'), offset, header, location, point }
      bound.push(binding); bindings.push(binding)
    })
    let film = '', frame = 0
    cdp.on('Page.screencastFrame', async event => {
      if (film) writeFileSync(resolve(output, `${film}-${String(frame++).padStart(4, '0')}.jpg`), Buffer.from(event.data, 'base64'))
      await cdp.send('Page.screencastFrameAck', { sessionId: event.sessionId }).catch(error => { if (!page.isClosed()) throw error })
    })
    await cdp.send('Page.startScreencast', { format: 'jpeg', quality: 70, maxWidth: 1280, maxHeight: 1080, everyNthFrame: 1 })
    await page.evaluateOnNewDocument(light => {
      const d = window.__trace = { layout: [], mounts: [], unmounts: [], commits: [], history: [], requests: [], sse: [], busy: [], frames: [], cameras: [], clicks: [] }
      document.addEventListener('click', event => { const row = event.target.closest?.('.part-details tr[data-pin]'); if (row) d.clicks.push({ at: performance.now(), pin: row.dataset.pin }) }, true)
      const isDevice = fiber => typeof fiber.type === 'function' && /device-card|part-details/.test(String(fiber.type))
      if (!light) window.__REACT_DEVTOOLS_GLOBAL_HOOK__ = { supportsFiber: true, inject: () => 1,
        onCommitFiberRoot: (_, root) => { window.__root = root; d.commits.push(performance.now()); const visit = fiber => { if (!fiber) return; if (isDevice(fiber) && !fiber.alternate) d.mounts.push({ at: performance.now(), name: fiber.type.name }); visit(fiber.child); visit(fiber.sibling) }; visit(root.current) },
        onCommitFiberUnmount: (_, fiber) => { if (isDevice(fiber)) d.unmounts.push({ at: performance.now(), name: fiber.type.name }) }, onPostCommitFiberRoot() {} }
      const originalFetch = fetch
      window.fetch = (...args) => { d.requests.push({ at: performance.now(), url: String(args[0]) }); return originalFetch(...args) }
      const Native = EventSource
      window.EventSource = class extends Native { constructor(...args) { super(...args); this.addEventListener('message', event => d.sse.push({ at: performance.now(), data: event.data })) } }
      for (const name of ['replaceState', 'pushState']) { const original = history[name]; history[name] = function(...args) { d.history.push({ at: performance.now(), name, url: String(args[2]) }); return original.apply(this, args) } }
      addEventListener('DOMContentLoaded', () => new MutationObserver(mutations => { for (const mutation of mutations) if (mutation.target.classList.contains('explorer-canvas')) d.busy.push({ at: performance.now(), value: mutation.target.getAttribute('aria-busy') }) }).observe(document.body, { subtree: true, attributes: true, attributeFilter: ['aria-busy'] }))
      const sample = () => {
        if (window.__sampling) {
          const canvas = document.querySelector('.explorer-canvas')?.getBoundingClientRect()
          const cards = [...document.querySelectorAll('.device-card')]
          const visible = cards.filter(card => { const r = card.getBoundingClientRect(); return canvas && r.right > canvas.left && r.left < canvas.right && r.bottom > canvas.top && r.top < canvas.bottom }).map(card => card.dataset.instance)
          d.frames.push({ at: performance.now(), visible, busy: document.querySelector('.explorer-canvas')?.getAttribute('aria-busy'), state: document.querySelector('.canvas-state')?.textContent, transform: document.querySelector('.react-flow__viewport') && getComputedStyle(document.querySelector('.react-flow__viewport')).transform })
        }
        requestAnimationFrame(sample)
      }
      requestAnimationFrame(sample)
    }, light)
    const ready = async () => { await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false' && new URL(location.href).searchParams.has('inspect')); await pause(350) }
    const geometry = () => page.evaluate(() => ({ transform: getComputedStyle(document.querySelector('.react-flow__viewport')).transform, nodes: [...document.querySelectorAll('.react-flow__node')].map(node => ({ id: node.dataset.id, style: node.getAttribute('style') })) }))
    const context = () => page.evaluate(() => JSON.parse(new URL(location.href).searchParams.get('inspect')))
    const clickText = async text => { const handle = await page.evaluateHandle(text => [...document.querySelectorAll('button')].find(button => button.textContent.trim() === text || button.getAttribute('aria-label') === text), text); assert.ok(handle.asElement(), text); await handle.asElement().click(); await ready() }
    const search = async value => { await page.$eval('input[aria-label="Search circuit"]', input => { input.focus(); input.select() }); await page.keyboard.press('Backspace'); await page.type('input[aria-label="Search circuit"]', value); await page.keyboard.press('Enter'); await ready(); await page.waitForSelector('.part-details') }
    await page.goto(url, { waitUntil: 'networkidle2' }); await ready()
    assert.equal(bound.length, 1)
    assert.ok(await page.evaluate(() => window.__trace.layout.length > 0), 'Breakpoint must prove at least one real baseline layout')
    const model = await page.evaluate(async () => (await fetch('/api/model')).json())
    assert.equal(model.instances.length, 233)
    await search('C263')
    const bindCamera = async () => { if (!light) await page.evaluate(() => {
      const visit = fiber => { if (!fiber) return; let hook = fiber.memoizedState; while (hook && typeof hook === 'object' && 'memoizedState' in hook) { const api = hook.memoizedState?.current; if (api?.getViewport && api?.setViewport && !api.__traced) { api.__traced = true; for (const name of ['setViewport', 'fitBounds', 'fitView']) { const original = api[name]; api[name] = function(...args) { window.__trace.cameras.push({ at: performance.now(), name }); return original.apply(this, args) } } } hook = hook.next } visit(fiber.child); visit(fiber.sibling) }; visit(window.__root?.current)
    }) }
    await bindCamera()
    const measure = async (name, action, expectedPin, expectedNet) => {
      const before = { geometry: await geometry(), context: await context(), counts: await page.evaluate(() => Object.fromEntries(Object.entries(window.__trace).map(([key, values]) => [key, values.length]))) }
      film = `${label}-${name}`; frame = 0
      await page.evaluate(() => { window.__sampling = true })
      await page.screenshot({ path: resolve(output, `${film}-before.png`) })
      await action(); await pause(500)
      const after = { geometry: await geometry(), context: await context(), events: await page.evaluate(counts => { window.__sampling = false; return Object.fromEntries(Object.entries(window.__trace).map(([key, values]) => [key, values.slice(counts[key])])) }, before.counts) }
      await page.screenshot({ path: resolve(output, `${film}-after.png`) }); film = ''
      records.push({ label, name, light, instrumentation: { react: !light, camera: !light }, before, after }); writeFileSync(resolve(output, 'results.json'), JSON.stringify({ bindings, records }, null, 2))
      assert.equal(after.events.layout.length, 0, `${label}/${name}: trace must not layout`)
      assert.equal(after.events.unmounts.length, 0); assert.equal(after.events.mounts.length, 0)
      assert.equal(after.events.cameras.length, 0); assert.equal(after.events.requests.length, 0)
      assert.ok(after.events.frames.length > 0); assert.ok(after.events.frames.every(f => f.visible.length && f.busy === 'false' && !f.state), `${label}/${name}: continuously visible, no busy overlay`)
      assert.deepEqual(after.geometry, before.geometry)
      assert.equal(after.context.pin, expectedPin); assert.equal(after.context.net, expectedNet)
      assert.ok(after.events.history.length <= 1, 'Trace URL synchronization is bounded')
      if (name.endsWith('-repeat')) assert.equal(after.events.history.length, 0, 'Same-net repeat must not write URL')
      if (name.endsWith('-rapid')) assert.deepEqual(after.events.clicks.map(click => click.pin), ['P2', 'P1', 'P2', 'P1', 'P2'], 'Rapid test must dispatch actual native row clicks in order')
      assert.ok(after.events.history.every(event => event.name === 'replaceState'))
      return after
    }
    const pin = async value => { const row = await page.$(`.part-details tr[data-pin="${value}"]`); assert.ok(row); await row.scrollIntoView(); await row.click() }
    for (const wireMode of (process.env.COHDL_TRACE_PENDING_ONLY ? [] : ['Individual', 'Bundled'])) {
      if (wireMode === 'Bundled') await clickText(wireMode)
      for (const designator of ['C263', 'C259']) {
        await search(designator)
        const instance = model.instances.find(i => i.designator === designator)
        const signal = model.nets.find(net => net.members.some(member => member.instance_path === instance.path && member.logical_pin === 'P1'))?.name ?? ''
        await measure(`${wireMode}-${designator}-gnd`, () => pin('P2'), 'P2', 'GND')
        await measure(`${wireMode}-${designator}-repeat`, () => pin('P2'), 'P2', 'GND')
        await measure(`${wireMode}-${designator}-signal`, () => pin('P1'), 'P1', signal)
        await measure(`${wireMode}-${designator}-rapid`, async () => {
          const row = await page.$('.part-details tr[data-pin="P1"]'); await row.scrollIntoView()
          const points = await page.evaluate(() => ['P1', 'P2'].map(pin => { const r = document.querySelector(`.part-details tr[data-pin="${pin}"]`).getBoundingClientRect(); return { x: r.left + r.width / 2, y: r.top + r.height / 2 } }))
          for (const index of [1, 0, 1, 0, 1]) { await cdp.send('Input.dispatchMouseEvent', { type: 'mousePressed', ...points[index], button: 'left', clickCount: 1 }); await cdp.send('Input.dispatchMouseEvent', { type: 'mouseReleased', ...points[index], button: 'left', clickCount: 1 }) }
        }, 'P2', 'GND')
        await measure(`${wireMode}-${designator}-clear`, () => clickText('Clear trace'), '', '')
      }
    }
    for (const [scopeName, groups] of (process.env.COHDL_TRACE_PENDING_ONLY ? [] : [['multi', ['ESP32 USB', 'Camera', 'SF32 core and clocks']], ['all', []]])) {
      const target = { ...await context(), mode: 'sch', connectionMode: 'individual', groups, collapsed: [], selection: [], pin: '', net: '', viewport: null }
      const next = new URL(url); next.searchParams.set('inspect', JSON.stringify(target))
      await page.goto(next.href, { waitUntil: 'networkidle2' }); await ready(); await search('C259'); await bindCamera()
      const instance = model.instances.find(i => i.designator === 'C259')
      const signal = model.nets.find(net => net.members.some(member => member.instance_path === instance.path && member.logical_pin === 'P1'))?.name ?? ''
      await measure(`${scopeName}-gnd`, () => pin('P2'), 'P2', 'GND')
      await measure(`${scopeName}-repeat`, () => pin('P2'), 'P2', 'GND')
      await measure(`${scopeName}-signal`, () => pin('P1'), 'P1', signal)
      assert.deepEqual((await context()).groups, groups, 'Trace retains exact chosen scope')
    }
    const beforePending = await page.evaluate(() => ({ at: performance.now(), layouts: window.__trace.layout.length }))
    await page.evaluate(() => { window.__holdNext = true; [...document.querySelectorAll('button')].find(button => button.textContent.trim() === 'Bundled').click() })
    await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'true')
    await pin('P2')
    await ready()
    await page.waitForFunction(() => {
      const current = JSON.parse(new URL(location.href).searchParams.get('inspect'))
      return current.pin === 'P2' && current.net === 'GND' && current.viewport !== null
        && document.querySelector('.part-details tr[data-pin="P2"]')?.getAttribute('data-selected') === 'true'
    })
    const pendingResult = { label, name: 'controlled-async-latest-pin', before: beforePending, context: await context(), events: await page.evaluate(before => ({ layouts: window.__trace.layout.slice(before.layouts), clicks: window.__trace.clicks.filter(click => click.at >= before.at), selected: [...document.querySelectorAll('.part-details tr[data-selected=true]')].map(row => row.dataset.pin), busy: document.querySelector('.explorer-canvas').getAttribute('aria-busy') }), beforePending) }
    records.push(pendingResult); writeFileSync(resolve(output, 'results.json'), JSON.stringify({ bindings, records }, null, 2))
    assert.ok(pendingResult.events.layouts.some(layout => layout.held), 'Actual structural completion was held by test-only CDP binding')
    assert.equal(pendingResult.context.pin, 'P2'); assert.equal(pendingResult.context.net, 'GND')
    assert.ok(await page.$('.part-details tr[data-pin="P2"][data-selected="true"]'), 'Latest pin remains visibly highlighted after structural completion')
    await cdp.send('Page.stopScreencast')
    await page.close()
  }
} finally { await browser.close() }
console.log(JSON.stringify({ output, light, cases: records.length, bindings: bindings.map(({ sha256, offset, header }) => ({ sha256, offset, header })), result: 'PASS' }, null, 2))
