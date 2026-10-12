import assert from 'node:assert/strict'
import test from 'node:test'
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import ts from 'typescript'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

const out = mkdtempSync(resolve('test/.device-cards-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/nodes.tsx', 'src/partPresentation.ts'], { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true })
assert.deepEqual(ts.getPreEmitDiagnostics(program).map(d => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
require.extensions['.css'] = () => {}
const { DetailedNode, detailedSize, FootprintPreview } = require(resolve(out, 'nodes.js'))
const { wireHandles } = require(resolve(out, 'handles.js'))
const { partPresentation, coreCandidates } = require(resolve(out, 'partPresentation.js'))
const { ReactFlowProvider } = require('@xyflow/react')
const inst = { path: 'Board::U4', designator: 'U4', impl_traits: [], specs: [], part: { mpn: '' }, pins: [{ logical: 'A', numbers: ['1'], nc: false, connected: true, role: 'io' }, { logical: 'B', numbers: ['2'], nc: true, connected: false, role: 'io' }] }
const node = { id: inst.path, kind: 'passive', inst, width: 56, height: 24, pinNets: { A: 'USB_DP' }, railTags: [], netLabels: [], decors: [] }
const render = element => renderToStaticMarkup(React.createElement(ReactFlowProvider, null, element))
test('every actual component has stable detailed geometry, honest local fallback and complete pin identity', () => {
  assert.ok(detailedSize(node).width >= 300)
  const html = render(React.createElement(DetailedNode, { id: node.id, data: { g: node, sourceValue: 'SF32LB525UC6', dark: true }, selected: false }))
  for (const text of ['device-card', 'device-header', 'device-pin-row', 'device-footprint', 'SF32LB525UC6', 'USB_DP', 'NC', 'Footprint unavailable']) assert.ok(html.includes(text), text)
  assert.deepEqual(wireHandles({ source: node.id, target: node.id, sourcePin: 'A', targetPin: 'B' }, new Map([[node.id, node]]), true), { sourceHandle: 'p:A', targetHandle: 'p:B' })
})
test('source identity informs categories and core suggestions without changing an empty MPN', () => {
  assert.equal(partPresentation(inst, 'SF32LB525UC6').category, 'Microcontroller')
  assert.equal(partPresentation({ ...inst, part: { mpn: 'TPD1E1B04DPYR' } }).category, 'ESD protection')
  const result = coreCandidates({ instances: [inst], nets: [{ name: 'USB_DP', members: [{ instance_path: inst.path }] }], derived: { rails: [] } }, new Map([[inst.path, 'SF32LB525UC6']]))
  assert.equal(result[0].instance.path, inst.path)
  assert.match(result[0].evidence, /Source value SF32LB525UC6/)
  assert.equal(inst.part.mpn, '')
})
test('actual local geometry shares fixed SVG sizing, matrices, polygons and honest empty fallback', () => {
  const geo = { pads: [{ number: '1', shape: 'polygon', vertices: [[0, 0], [1, 0], [0, 1]], matrix: [0, 1, -1, 0], rotate: 90, x: 2, y: 3, size: [] }], mount_holes: [{ shape: 'oval', size: [1, 2], x: 0, y: 0, plated: null }] }
  const html = renderToStaticMarkup(React.createElement(FootprintPreview, { geo, dark: true, width: 88, height: 64 }))
  assert.match(html, /width="88"/); assert.match(html, /height="64"/)
  assert.match(html, /matrix\(0 1 -1 0 2 3\)/); assert.match(html, /polygon/); assert.match(html, /source_unspecified/)
  assert.doesNotMatch(html, /NaN|Infinity/)
  const empty = renderToStaticMarkup(React.createElement(FootprintPreview, { geo: { pads: [], mount_holes: [] }, dark: false }))
  assert.match(empty, /Footprint unavailable/)
  const invalid = renderToStaticMarkup(React.createElement(FootprintPreview, { geo: { ...geo, pads: [{ ...geo.pads[0], x: Infinity }] }, dark: false }))
  assert.match(invalid, /Footprint unavailable/)
})
