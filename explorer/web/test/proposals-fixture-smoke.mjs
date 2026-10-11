import assert from 'node:assert/strict'
import { cpSync, mkdirSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createServer } from 'node:net'
import { tmpdir } from 'node:os'
import { resolve } from 'node:path'
import { spawn } from 'node:child_process'
import puppeteer from 'puppeteer-core'
import { runProposalRecoverySmoke } from './proposals-smoke.mjs'

const root = mkdtempSync(resolve(tmpdir(), 'cohdl-proposal-browser-fixture-'))
const output = resolve(process.argv[2] ?? '/tmp/cohdl-proposals-fixtures')
mkdirSync(output, { recursive: true })
cpSync(resolve('../extractor/tests/fixtures/parameterized'), resolve(root, 'project'), { recursive: true })
const listener = createServer()
await new Promise(resolve => listener.listen(0, '127.0.0.1', resolve))
const port = listener.address().port
await new Promise(resolve => listener.close(resolve))
const server = spawn(resolve('../extractor/target/debug/cohdl-explorer'), [resolve(root, 'project'), '--serve', '--dist', resolve('dist'), '--port', String(port)], { stdio: ['ignore', 'ignore', 'pipe'] })
let serverLog = ''
server.stderr.on('data', chunk => { serverLog += chunk.toString() })
const url = `http://127.0.0.1:${port}`
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
const errors = []
try {
  const deadline = Date.now() + 10000
  for (;;) {
    try { const response = await fetch(`${url}/api/model`); if (response.ok) break } catch {}
    assert.ok(Date.now() < deadline, `Fixture server startup failed: ${serverLog}`)
    await new Promise(resolve => setTimeout(resolve, 50))
  }
  const page = await browser.newPage()
  page.on('pageerror', error => errors.push(error.message))
  const ready = async () => {
    await page.waitForSelector('.react-flow__node:not(.react-flow__node-region)', { visible: true })
    await page.waitForFunction(() => document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false')
  }
  const clickText = async label => {
    const button = await page.evaluateHandle(label => [...document.querySelectorAll('button')].find(el => el.textContent.trim() === label || el.getAttribute('aria-label') === label), label)
    assert.ok(button.asElement(), `Missing control ${label}`)
    await button.asElement().click(); await button.dispose()
  }
  const retained = { schema: 1, status: 'proposed', freshness: 'fresh', issues: [], artifacts: [], run: {
    id: 'declared-latest-fixture', kind: 'plan', provider: null, bundle: null, validation: null, receipt: null,
    proposal: { schema: 1, kind: 'plan', title: 'Declared latest response fixture', rationale: 'Response ordering regression only', uncertainty: 'Not real AI', changes: [], citations: [{ document: null, page: null, section: null, source: { path: 'src/main.cohdl', line: 1, sha256: 'a'.repeat(64) } }] },
  } }
  for (const [width, height] of [[1280, 800], [390, 844]]) {
    await page.setViewport({ width, height })
    await runProposalRecoverySmoke(page, { url, output, ready, clickText }, retained)
    await page.click('.proposal-citation .source-excerpt summary')
    await page.waitForFunction(() => document.querySelector('.proposal-citation [role="alert"]')?.textContent.includes('Current source differs from cited SHA-256'))
    await page.waitForSelector('.proposal-citation pre[aria-label="CoHDL source excerpt"]')
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth), false)
    await page.screenshot({ path: resolve(output, `declared-proposals-${width}.png`), fullPage: true })
  }
  assert.deepEqual(errors, [])
  const result = { result: 'PASS', fixture: true, realProvider: false, viewports: [1280, 390], states: ['empty', 'error', 'rejected', 'pending'], obsoleteResponseIgnored: true, sourceHashMismatchExplicit: true, errors }
  writeFileSync(resolve(output, 'result.json'), JSON.stringify(result, null, 2))
  console.log(JSON.stringify(result))
} finally {
  await browser.close()
  server.kill()
  await new Promise(resolve => server.once('exit', resolve))
  rmSync(root, { recursive: true, force: true })
  writeFileSync(resolve(output, 'server.log'), serverLog)
}
