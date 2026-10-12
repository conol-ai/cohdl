import assert from 'node:assert/strict'
import puppeteer from 'puppeteer-core'

const argument = process.argv.indexOf('--url')
const base = argument < 0 ? 'http://127.0.0.1:5210' : process.argv[argument + 1]
const model = await (await fetch(`${base}/api/model`)).json()
const config = await (await fetch(`${base}/views/${encodeURIComponent(model.design)}.view.json`)).json()
assert.ok(config.views.find(view => view.name === 'Functional')?.regions.some(region => region.name === 'Camera'))
const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
const errors = []
try {
  for (const failure of ['500', 'malformed', '404']) {
    const page = await browser.newPage()
    page.on('pageerror', error => errors.push(error.message))
    let recovered = false
    await page.setRequestInterception(true)
    page.on('request', request => {
      if (!request.url().includes('/views/')) return request.continue()
      if (recovered) return request.respond({ status: 200, contentType: 'application/json', body: JSON.stringify(config) })
      return request.respond({ status: failure === 'malformed' ? 200 : Number(failure), contentType: 'application/json', body: failure === 'malformed' ? '{broken' : JSON.stringify({ error: 'Configuration unavailable during recovery regression' }) })
    })
    const requested = { version: 2, mode: 'sch', connectionMode: 'individual', scope: null, view: 'Functional', groups: ['Camera'], collapsed: [], selection: [], pin: '', net: '', tab: 'circuit', viewport: null }
    const url = new URL(base)
    url.searchParams.set('inspect', JSON.stringify(requested))
    await page.goto(url.href, { waitUntil: 'networkidle2' })
    await page.waitForFunction(() => [...document.querySelectorAll('button')].some(button => button.textContent === 'Retry groups'))
    await page.waitForFunction(() => document.querySelector('.inspection-range')?.textContent === 'Camera'
      && document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false')
    assert.equal(await page.url(), url.href, `${failure}: failed config must not overwrite the requested URL`)
    assert.equal(await page.$eval('.inspection-range', element => element.textContent), 'Camera')
    recovered = true
    await page.evaluate(() => [...document.querySelectorAll('button')].find(button => button.textContent === 'Retry groups').click())
    const settled = () => {
      const inspect = JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}')
      return inspect.view === 'Functional' && inspect.groups?.length === 1 && inspect.groups[0] === 'Camera'
        && inspect.viewport !== null && document.querySelector('.device-card')
        && document.querySelector('.explorer-canvas')?.getAttribute('aria-busy') === 'false'
        && ![...document.querySelectorAll('button')].some(button => button.textContent === 'Retry groups')
    }
    await page.waitForFunction(settled)
    await page.reload({ waitUntil: 'networkidle2' })
    await page.waitForFunction(settled)
    assert.equal(await page.$eval('.inspection-range', element => element.textContent), 'Camera')
    console.log(`PASS ${failure}: requested Camera preserved through retry and refresh`)
    await page.close()
  }
  const page = await browser.newPage()
  await page.setRequestInterception(true)
  page.on('request', request => request.url().includes('/views/') ? request.respond({ status: 404, contentType: 'application/json', body: '{}' }) : request.continue())
  await page.goto(base, { waitUntil: 'networkidle2' })
  await page.waitForFunction(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}').viewport !== undefined)
  await page.evaluate(() => [...document.querySelectorAll('button')].find(button => button.textContent === 'Overview').click())
  await page.waitForFunction(() => JSON.parse(new URL(location.href).searchParams.get('inspect') ?? '{}').mode === 'overview')
  console.log('PASS no config: viewless project remains navigable')
  await page.close()
  assert.deepEqual(errors, [])
} finally {
  await browser.close()
}
