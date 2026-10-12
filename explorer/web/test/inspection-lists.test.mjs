import assert from 'node:assert/strict'
import { test } from 'node:test'
import { cpSync, existsSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'
import ts from 'typescript'

const out = mkdtempSync(resolve('test/.inspection-render-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/InspectionLists.tsx'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS,
  jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true,
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
if (existsSync('src/InspectionLists.css')) cpSync('src/InspectionLists.css', resolve(out, 'InspectionLists.css'))
require.extensions['.css'] = () => {}
const { InspectionLists } = require(resolve(out, 'InspectionLists.js'))
const span = { file: 'src/main.cohdl', line: 42, col: 5 }
const model = {
  instances: [
    { path: 'Board::empty', designator: 'U4', device_fq: 'source::Device', impl_traits: [], part: { mpn: '', footprint: 'source::FP' }, specs: [], span },
    { path: 'Board::unknown', device_fq: 'source::Unknown', impl_traits: [], specs: [{ name: 'resistance', value: '10kohm' }], span },
    { path: 'Board::empty-reference', designator: '', device_fq: 'source::Device', impl_traits: [], specs: [], span },
  ],
  diagnostics: [
    { code: 'E42', severity: 'error', message: 'Pin conflict', span },
    { code: 'W01', severity: 'warning', message: 'Unknown source location' },
  ],
  live_error: 'Actual failed edit; last good circuit retained',
}
function buttons(element) {
  if (!React.isValidElement(element)) return []
  return [
    ...(element.type === 'button' ? [element] : []),
    ...React.Children.toArray(element.props.children).flatMap(buttons),
  ]
}

test('BOM retains empty versus absent fields and locates exact real instance identities', () => {
  const locations = []
  const element = InspectionLists({ model, tab: 'bom', onLocate: (hit) => locations.push(hit), onIssue: () => {} })
  const html = renderToStaticMarkup(element)
  assert.match(html, /Not specified/)
  assert.match(html, /Unknown/)
  assert.match(html, /10kohm/)
  assert.doesNotMatch(html, />Board::unknown</)
  assert.doesNotMatch(html, /Locate Board::empty-reference/)
  assert.match(html, /Reference not specified/)
  const targets = buttons(element)
  assert.equal(targets.length, model.instances.length)
  targets[0].props.onClick()
  assert.deepEqual(locations, [{ kind: 'part', id: 'Board::empty', label: 'U4', sub: '' }])
})

test('Issues expose last-good live errors, retain unlocated diagnostics and locate authored spans', () => {
  const located = []
  const element = InspectionLists({ model, tab: 'issues', onLocate: () => {}, onIssue: (issue) => located.push(issue) })
  const html = renderToStaticMarkup(element)
  assert.match(html, /Actual failed edit; last good circuit retained/)
  assert.match(html, /Unknown source location/)
  assert.match(html, /Source location unavailable/)
  assert.match(html, /src\/main.cohdl:42:5/)
  const targets = buttons(element)
  assert.equal(targets.length, 1)
  targets[0].props.onClick()
  assert.deepEqual(located, [model.diagnostics[0]])
})

test('Empty BOM and Issues remain explicit and recoverable', () => {
  const empty = { instances: [], diagnostics: [] }
  for (const [tab, message] of [['bom', /No parts/], ['issues', /No issues/]]) {
    const element = InspectionLists({ model: empty, tab, onLocate: () => {}, onIssue: () => {} })
    assert.match(renderToStaticMarkup(element), message)
    assert.equal(buttons(element).length, 0)
  }
})
