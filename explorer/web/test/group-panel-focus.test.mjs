import assert from 'node:assert/strict'
import { test } from 'node:test'
import { build } from 'esbuild'
import puppeteer from 'puppeteer-core'

test('mobile Group drawer cycles actual visible AI controls and restores opener focus', async () => {
  const bundle = await build({ stdin: { resolveDir: process.cwd(), loader: 'tsx', contents: `
    import React, { useState } from 'react';
    import { createRoot } from 'react-dom/client';
    import { GroupPanel } from './src/GroupPanel';
    function Fixture() {
      const [open, setOpen] = useState(false);
      return <><button id="opener" onClick={() => setOpen(true)}>Groups</button>
        <GroupPanel names={[]} selected={[]} collapsed={[]} layout={true} open={open}
          onClose={() => setOpen(false)} onToggle={() => {}} onAll={() => {}} onCollapsed={() => {}}>
          <textarea id="intent" aria-label="Group intent" />
          <select id="provider" aria-label="Provider"><option>Configured provider</option></select>
          <a id="document" href="#document">Document</a>
          <details><summary id="advanced">Advanced</summary><input id="closed-details" /></details>
          <div id="custom" tabIndex={0}>Additional action</div>
          <button id="disabled" disabled>Unavailable</button>
          <fieldset disabled><input id="disabled-fieldset" /></fieldset>
          <input id="negative" tabIndex={-1} />
          <div hidden><input id="hidden" /></div>
          <div inert=""><button id="inert">Inactive</button></div>
        </GroupPanel></>;
    }
    createRoot(document.getElementById('root')).render(<Fixture />);
  ` }, bundle: true, write: false, format: 'iife', jsx: 'automatic', loader: { '.css': 'empty' }, logLevel: 'silent' })
  const browser = await puppeteer.launch({ executablePath: process.env.CHROME_PATH ?? '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome', headless: true })
  try {
    const page = await browser.newPage()
    await page.setViewport({ width: 390, height: 844 })
    await page.setContent('<style>.group-panel:not(.is-open){display:none}.group-drawer-close{display:none}</style><div id="root"></div>')
    await page.addScriptTag({ content: bundle.outputFiles[0].text })
    await page.click('#opener')
    await page.waitForFunction(() => document.activeElement?.getAttribute('aria-label') === 'Search groups')
    await page.keyboard.press('Tab')
    assert.equal(await page.evaluate(() => document.activeElement?.id), 'intent', 'Forward wrap skips hidden close button and reaches AI textarea')
    const order = ['provider', 'document', 'advanced', 'custom']
    for (const id of order) {
      await page.keyboard.press('Tab')
      assert.equal(await page.evaluate(() => document.activeElement?.id), id)
    }
    await page.keyboard.press('Tab')
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')), 'Search groups', 'Disabled, hidden, inert, negative tabindex and closed-details controls are skipped')
    await page.focus('#intent')
    await page.keyboard.down('Shift')
    await page.keyboard.press('Tab')
    await page.keyboard.up('Shift')
    assert.equal(await page.evaluate(() => document.activeElement?.getAttribute('aria-label')), 'Search groups', 'Reverse wrap stays in drawer')
    await page.keyboard.press('Escape')
    await page.waitForFunction(() => document.activeElement?.id === 'opener' && !document.querySelector('.group-panel').classList.contains('is-open'))
  } finally { await browser.close() }
})
