import assert from 'node:assert/strict'
import test from 'node:test'
import { cpSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import ts from 'typescript'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

const out = mkdtempSync(resolve('test/.readability-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/partPresentation.ts', 'src/important.ts', 'src/pinConnection.ts', 'src/PinTarget.tsx', 'src/PartDetails.tsx'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true,
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map(d => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
for (const file of ['ProvenanceInspector.css', 'OutsideDestinations.css', 'PartDetails.css']) {
  cpSync(`src/${file}`, resolve(out, file))
}
require.extensions['.css'] = () => {}
const { partPresentation, coreCandidates } = require(resolve(out, 'partPresentation.js'))
const { marksKey, readMarks, writeMarks } = require(resolve(out, 'important.js'))
const { pinConnection } = require(resolve(out, 'pinConnection.js'))
const { PinTarget } = require(resolve(out, 'PinTarget.js'))
const { PartDetails } = require(resolve(out, 'PartDetails.js'))
const { DetailedNode } = require(resolve(out, 'nodes.js'))
const { ReactFlowProvider } = require('@xyflow/react')
const part = { path: 'Board::SRC7', designator: 'SRC7', device_fq: 'source::DEVICE_7', impl_traits: ['source::SourceComponent'], specs: [], part: { fq: 'source::part', mpn: '' }, pins: [], docs: [], span: { file: 'src/main.cohdl', line: 1, col: 1 } }
test('core suggestions require real identity/category and rank nonrail connectivity without confirming marks', () => {
  const ic = { ...part, path: 'ic', part: { mpn: 'STM32F405RGT6' } }
  const unknown = { ...part, path: 'unknown', device_fq: 'DEVICE_STM32' }
  const passive = { ...part, path: 'r', impl_traits: ['std::Resistor'] }
  const model = { instances: [unknown, passive, ic], nets: [{ name: 'VCC', is_gnd: false, members: [{ instance_path: 'ic' }] }, { name: 'SDA', members: [{ instance_path: 'ic' }, { instance_path: 'r' }] }], derived: { rails: ['VCC'] } }
  const candidates = coreCandidates(model)
  assert.deepEqual(candidates.map(c => c.instance.path), ['ic'])
  assert.match(candidates[0].evidence, /STM32.*1 signal/)
  assert.equal(coreCandidates({ ...model, instances: [{ ...ic, specs: [{ name: 'resistance', value: '1ohm' }] }] }).length, 0)
})
test('identity uses declared electrical facts and never designator prefixes or internal device names', () => {
  assert.deepEqual(partPresentation(part), { reference: 'SRC7', category: 'Component', value: '', mpn: '', title: 'SRC7 Component', subtitle: '' })
  const resistor = partPresentation({ ...part, designator: 'C99', specs: [{ name: 'resistance', value: '0ohm' }] })
  assert.equal(resistor.category, 'Resistor'); assert.equal(resistor.value, '0ohm')
  assert.equal(partPresentation({ ...part, specs: [{ name: 'capacitance', value: '2.2uF' }] }).category, 'Capacitor')
  assert.equal(partPresentation({ ...part, impl_traits: ['std::Inductor'] }).category, 'Inductor')
  assert.equal(partPresentation({ ...part, part: { fq: 'p', mpn: 'Real-Part' } }).subtitle, 'Real-Part')
})
test('marks are project and design scoped, bounded, filtered and resilient to denied storage', () => {
  const a = marksKey('a'.repeat(64), 'Board'), b = marksKey('b'.repeat(64), 'Board')
  assert.notEqual(a, b); assert.notEqual(a, marksKey('a'.repeat(64), 'Other'))
  assert.equal(marksKey('', 'Board'), null)
  const allowed = new Set(['Board::one', 'Board::two'])
  for (const value of ['bad', '{}', JSON.stringify(['removed', 4, 'Board::one', 'Board::one']), ' '.repeat(65537)]) {
    const result = readMarks({ getItem: () => value }, a, allowed)
    assert.deepEqual([...result], value.startsWith('[') ? ['Board::one'] : [])
  }
  assert.deepEqual([...readMarks({ getItem() { throw Error('denied') } }, a, allowed)], [])
  assert.equal(writeMarks({ setItem() { throw Error('denied') } }, a, new Set(['Board::one'])), false)
  let stored
  assert.equal(writeMarks({ setItem(key, value) { stored = { key, value } } }, a, new Set(['Board::two'])), true)
  assert.deepEqual(JSON.parse(stored.value), ['Board::two'])
  const unicodePath = '部'.repeat(1024)
  assert.deepEqual([...readMarks({ getItem: () => JSON.stringify(Array(30).fill(unicodePath)) }, a, new Set([unicodePath]))], [], 'UTF-8 byte bound includes non-ASCII source paths')
})
test('all pin connection explanations use full actual membership and distinguish open and NC', () => {
  const pin = { logical: 'P1', numbers: ['1'], connected: true, nc: false, role: 'io', obligation: 'optional' }
  const net = { name: 'SDA', is_gnd: false, members: [{ instance_path: part.path, logical_pin: 'P1', numbers: ['1'] }, { instance_path: 'Board::other', logical_pin: 'P2', numbers: ['2'] }], span: part.span }
  const model = { instances: [part], nets: [net], derived: { rails: [] } }
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path]), '').state, 'outside range')
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path, 'Board::other']), '').state, 'bundled connection')
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path, 'Board::other']), '', undefined, 'individual').state, 'individual connection')
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path, 'Board::other']), 'SDA', undefined, 'individual').state, 'individual connection')
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path]), 'SDA').state, 'outside range')
  assert.equal(pinConnection({ ...model, derived: { rails: ['SDA'] } }, part.path, pin, new Set(), '').state, 'simplified rail')
  assert.equal(pinConnection({ ...model, nets: [{ ...net, members: net.members.slice(0, 1) }] }, part.path, pin, new Set(), '').state, 'single endpoint')
  assert.equal(pinConnection(model, part.path, { ...pin, logical: 'NC', connected: false, nc: true }, new Set(), '').state, 'NC')
  assert.equal(pinConnection(model, part.path, { ...pin, logical: 'OPEN', connected: false }, new Set(), '').state, 'open')
  const internal = { ...model, nets: [{ ...net, members: [net.members[0], { ...net.members[1], instance_path: part.path }] }] }
  assert.equal(pinConnection(internal, part.path, pin, new Set([part.path]), '').state, 'internal connection')
  assert.equal(pinConnection(model, part.path, pin, new Set([part.path, 'Board::other']), '', new Map([[part.path, 'group:one'], ['Board::other', 'group:one']])).state, 'internal connection')
})

test('part details connection states follow the explicit wire mode', () => {
  const pin = { logical: 'SDA', numbers: ['1'], connected: true, nc: false, role: 'io', obligation: 'optional' }
  const instance = { ...part, pins: [pin] }
  const net = { name: 'SDA', members: [{ instance_path: part.path, logical_pin: 'SDA', numbers: ['1'] }, { instance_path: 'other', logical_pin: 'IN', numbers: ['2'] }] }
  const props = { model: { instances: [instance], nets: [net], footprints: {}, derived: { rails: [] } }, instance, dark: false, resource: { registry: null, loading: false, error: '', refresh() {} }, view: '', groups: [], visible: new Set([part.path, 'other']), selectedPin: '', selectedNet: '', marked: false, onMark() {}, onPin() {} }
  const render = connectionMode => renderToStaticMarkup(React.createElement(PartDetails, { ...props, connectionMode }))
  assert.match(render('individual'), /individual connection/)
  assert.doesNotMatch(render('individual'), /bundled connection/)
  assert.match(render('bundled'), /bundled connection/)
})

test('SCH detailed heading uses the prepared source-backed subtitle when actual MPN is empty', () => {
  const node = { id: part.path, kind: 'ic', title: 'SRC7 Component', sub: 'SF32LB525UC6', inst: part, pinNets: {}, railTags: [], width: 300, height: 50, pinsTotal: 0, pinsConnected: 0 }
  const html = renderToStaticMarkup(React.createElement(ReactFlowProvider, null, React.createElement(DetailedNode, { id: part.path, data: { g: node, onPin() {} }, selected: false })))
  assert.match(html, /SF32LB525UC6/)
})

test('pin target pointer and keyboard activation retain independent exact half-row identity', () => {
  const calls = []
  const identity = { instance: part.path, pin: 'LEFT', net: 'SDA' }
  const target = PinTarget({ identity, selected: false, children: 'left pin', onActivate: pin => calls.push(pin) })
  let stopped = 0, prevented = 0
  const event = key => ({ key, stopPropagation() { stopped++ }, preventDefault() { prevented++ } })
  target.props.onClick(event()); target.props.onKeyDown(event('Enter')); target.props.onKeyDown(event(' ')); target.props.onKeyDown(event('Tab'))
  assert.deepEqual(calls, [identity, identity, identity]); assert.equal(stopped, 3); assert.equal(prevented, 2)
  assert.equal(target.props.role, 'button'); assert.equal(target.props.tabIndex, 0)
})

test('concise part details show source value and honest empty MPN, pins before source support', () => {
  const instance = { ...part, pins: [{ logical: 'OPEN', numbers: ['1'], connected: false, nc: false, role: 'io', obligation: 'optional' }] }
  const registry = { schema: 1, documents: [{ id: 'original-schematic', title: 'Original schematic', status: 'missing', applicability: [{ relation: 'exact', instances: [part.path], groups: [], facts: { value: 'SF32LB525UC6', mpn: '' } }] }], constraints: [] }
  const html = renderToStaticMarkup(React.createElement(PartDetails, { model: { instances: [instance], nets: [], footprints: {}, derived: { rails: [] } }, instance, dark: false,
    resource: { registry, loading: false, error: '', refresh() {} }, view: '', groups: [], visible: new Set(), selectedPin: '', selectedNet: '', marked: false, onMark() {}, onPin() {} }))
  assert.match(html, /SRC7 Microcontroller/); assert.match(html, /SF32LB525UC6/); assert.match(html, /Unspecified/)
  assert.doesNotMatch(html, /No applicable document records|Source evidence/)
  assert.doesNotMatch(html, /Original schematic/); assert.match(html, /open/)
  assert.ok(html.indexOf('DEVICE_7') > html.indexOf('Technical details'))
})

test('selected network precedes connections, keeps exact pin distinct and directly exposes truthful documents', () => {
  const pins = ['P53', 'P56'].map(logical => ({ logical, numbers: [logical.slice(1)], connected: true, nc: false }))
  const instance = { ...part, pins }
  const net = { name: 'GND', members: [...pins.map(pin => ({ instance_path: part.path, logical_pin: pin.logical, numbers: pin.numbers })), { instance_path: 'outside', logical_pin: 'GND', numbers: ['2'] }] }
  const document = { id: 'chip-sheet', title: 'Chip datasheet', kind: 'pdf', status: 'ready', open_url: '/api/documents/chip-sheet', pages: 4, review: 'unreviewed', issues: [], applicability: [{ relation: 'exact', instances: [part.path], groups: [], page: 2 }] }
  const props = { model: { instances: [instance], nets: [net], footprints: {}, derived: { rails: ['GND'] } }, instance, dark: false, resource: { registry: { documents: [document], constraints: [] }, loading: false, error: '', refresh() {} }, view: '', groups: [], visible: new Set([part.path]), instanceRegions: new Map([[part.path, 'Core'], ['outside', 'Power']]), selectedPin: 'P56', selectedNet: 'GND', marked: false, onMark() {}, onPin() {}, onGroup() {}, onEndpoint() {} }
  const html = renderToStaticMarkup(React.createElement(PartDetails, props))
  assert.ok(html.indexOf('Selected network') < html.indexOf('<h4>Connections'))
  assert.match(html, /3 endpoints.*2 local.*1 outside/)
  assert.match(html, /data-pin="P53" data-selected="false" data-traced="true"/)
  assert.match(html, /data-pin="P56" data-selected="true" data-traced="true"/)
  assert.doesNotMatch(html, /exact trace/)
  assert.ok(html.indexOf('Documents / Datasheet') < html.indexOf('<h4>Connections'))
  assert.match(html, /href="\/api\/documents\/chip-sheet#page=2"/)
  const clear = renderToStaticMarkup(React.createElement(PartDetails, { ...props, selectedPin: '', selectedNet: '' }))
  assert.equal((html.match(/simplified rail/g) ?? []).length, (clear.match(/simplified rail/g) ?? []).length)
  const missing = renderToStaticMarkup(React.createElement(PartDetails, { ...props, resource: { ...props.resource, registry: null } }))
  assert.match(missing, /No associated documents/)
  assert.doesNotMatch(missing, /href="\/api\/documents\//)
  for (const [status, label] of [['missing', 'Missing local document'], ['invalid', 'Unavailable']]) {
    const unavailable = { ...document, status, open_url: '/api/documents/chip-sheet', applicability: [{ ...document.applicability[0], relation: 'related' }] }
    const html = renderToStaticMarkup(React.createElement(PartDetails, { ...props, resource: { ...props.resource, registry: { documents: [unavailable], constraints: [] } } }))
    assert.match(html, new RegExp(label))
    assert.match(html, /Related evidence/)
    assert.doesNotMatch(html, /href="\/api\/documents\//)
  }
  for (const [state, label] of [[{ loading: true }, 'Checking documents'], [{ error: 'offline' }, 'Documents unavailable: offline']]) {
    const html = renderToStaticMarkup(React.createElement(PartDetails, { ...props, resource: { ...props.resource, ...state } }))
    assert.match(html, new RegExp(label))
  }
  for (const [logical, nc, label] of [['NC', true, 'NC'], ['OPEN', false, 'open']]) {
    const openPart = { ...instance, pins: [{ logical, numbers: ['9'], connected: false, nc }] }
    const openHtml = renderToStaticMarkup(React.createElement(PartDetails, { ...props, instance: openPart, selectedPin: logical, selectedNet: '' }))
    assert.match(openHtml, new RegExp(`data-state="${label}"`))
  }
})

test('unselected parts keep connections directly below identity without an empty network slot', () => {
  const instance = { ...part, pins: [{ logical: 'P1', numbers: ['1'], connected: true, nc: false }] }
  const html = renderToStaticMarkup(React.createElement(PartDetails, {
    model: { instances: [instance], nets: [], footprints: {}, derived: { rails: [] } }, instance, dark: false,
    resource: { registry: null, loading: false, error: '', refresh() {} }, view: '', groups: [], visible: new Set([part.path]),
    selectedPin: '', selectedNet: '', marked: false, onMark() {}, onPin() {},
  }))
  assert.doesNotMatch(html, /Selected network|No network selected/)
  assert.ok(html.indexOf('Documents / Datasheet') < html.indexOf('Connections'))
})

test('datasheet cards deduplicate stable ids and hashes while evidence stays collapsed', () => {
  const instance = { ...part, pins: [] }
  const applicability = [{ relation: 'exact', instances: [part.path], groups: [], page: 1 }]
  const first = { id: 'sheet-a', title: 'Family datasheet', kind: 'pdf', status: 'ready', open_url: '/api/documents/sheet-a', pages: 2, review: 'reviewed', issues: [], applicability, sha256: 'a'.repeat(64) }
  const duplicateHash = { ...first, id: 'sheet-b', open_url: '/api/documents/sheet-b' }
  const html = renderToStaticMarkup(React.createElement(PartDetails, {
    model: { instances: [instance], nets: [], footprints: {}, derived: { rails: [] } }, instance, dark: false,
    resource: { registry: { schema: 1, documents: [first, duplicateHash], constraints: [] }, loading: false, error: '', refresh() {} },
    view: '', groups: [], visible: new Set([part.path]), selectedPin: '', selectedNet: '', marked: false, onMark() {}, onPin() {},
  }))
  assert.equal((html.match(/data-document-id="sheet-a"/g) ?? []).length, 1)
  assert.doesNotMatch(html, /data-document-id="sheet-b"/)
  assert.match(html, /<details class="provenance-inspector"/)
  assert.doesNotMatch(html, /<details class="provenance-inspector"[^>]* open/)
})
