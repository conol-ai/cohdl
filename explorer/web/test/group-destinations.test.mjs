import assert from 'node:assert/strict'
import test from 'node:test'
import { cpSync, mkdtempSync, rmSync, writeFileSync } from 'node:fs'
import { createRequire } from 'node:module'
import { resolve } from 'node:path'
import ts from 'typescript'
import React from 'react'
import { renderToStaticMarkup } from 'react-dom/server'

const out = mkdtempSync(resolve('test/.group-destinations-'))
process.once('exit', () => rmSync(out, { recursive: true, force: true }))
writeFileSync(resolve(out, 'package.json'), JSON.stringify({ type: 'commonjs' }))
const program = ts.createProgram(['src/groupIndex.ts', 'src/GroupIndexContent.tsx', 'src/OutsideDestinations.tsx', 'src/search.ts'], { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, jsx: ts.JsxEmit.ReactJSX, outDir: out, strict: true, skipLibCheck: true })
assert.deepEqual(ts.getPreEmitDiagnostics(program).map(d => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
cpSync('src/OutsideDestinations.css', resolve(out, 'OutsideDestinations.css'))
require.extensions['.css'] = () => {}
const { groupIndex } = require(resolve(out, 'groupIndex.js'))
const { destinationGroups, OutsideDestinations } = require(resolve(out, 'OutsideDestinations.js'))
const { searchCircuit } = require(resolve(out, 'search.js'))
const { GroupIndexContent } = require(resolve(out, 'GroupIndexContent.js'))
const part = (path, designator) => ({ path, designator, device_fq: 'source::Unknown', impl_traits: [], specs: [], part: { mpn: '' }, pins: [{ logical: 'P2', numbers: ['2', 'EP'], connected: true, nc: false, role: 'io' }] })
const instances = [part('board::u4', 'U4'), part('board::c263', 'C263'), part('board::outside', 'J9')]
const member = instance => ({ instance_path: instance.path, logical_pin: 'P2', numbers: ['2', 'EP'] })
const ground = { name: 'GND', is_gnd: true, members: instances.map(member) }
const signal = { name: 'USB_DP', is_gnd: false, members: [member(instances[0]), member(instances[2])] }
const model = { instances, nets: [ground, signal], derived: { rails: ['GND'] }, footprints: {} }
const regions = new Map([[instances[0].path, 'SF32'], [instances[1].path, 'SF32'], [instances[2].path, 'USB']])
const values = new Map([[instances[0].path, 'SF32LB525UC6']])
test('group index derives exact members, source-backed cores and only real crossing interfaces', () => {
  const before = JSON.stringify(model)
  const index = groupIndex(model, regions, values)
  assert.equal(index.get('SF32').parts, 2)
  assert.match(index.get('SF32').cores[0].label, /U4.*SF32LB525UC6/)
  assert.deepEqual(index.get('SF32').interfaces, ['USB_DP'])
  assert.deepEqual(index.get('SF32').rails, ['GND'])
  assert.equal(index.get('USB').parts, 1)
  assert.equal(index.get('USB').cores.length, 0)
  assert.equal(JSON.stringify(model), before)
})
test('source value search locates an actual empty-MPN device and its exact pins', () => {
  assert.equal(searchCircuit(model, 'SF32LB525UC6', values).find(hit => hit.kind === 'part').id, instances[0].path)
  assert.match(searchCircuit(model, 'SF32LB525UC6', values).find(hit => hit.kind === 'part').label, /U4.*SF32LB525UC6/)
  const hit = searchCircuit(model, 'SF32LB525UC6 P2', values).find(hit => hit.kind === 'pin')
  assert.equal(hit.id, instances[0].path)
  assert.equal(hit.pin, 'P2')
  assert.equal(instances[0].part.mpn, '')
})
test('destinations retain every original endpoint and separate local from outside groups without graph changes', () => {
  const visible = new Set([instances[0].path, instances[1].path])
  const groups = destinationGroups(model, ground, regions, visible, values)
  assert.equal(groups.length, 2)
  assert.equal(groups.find(group => group.local).endpoints.length, 2)
  const outside = groups.find(group => !group.local)
  assert.equal(outside.group, 'USB')
  assert.deepEqual(outside.endpoints[0].member, ground.members[2])
  assert.equal(groups.flatMap(group => group.endpoints).length, ground.members.length)
  assert.deepEqual([...visible], [instances[0].path, instances[1].path])
  const html = renderToStaticMarkup(React.createElement(OutsideDestinations, { model, net: ground, instanceRegions: regions, visible, sourceValues: values, onGroup() {}, onEndpoint() {} }))
  for (const text of ['GND', 'SF32', 'USB', 'Local', 'Outside', 'J9.P2', '2, EP', '<details', 'outside-destination-list']) assert.ok(html.includes(text), text)
  assert.match(html, /Open USB group/)
})
test('high fanout stays in bounded disclosed groups while preserving all source tuples', () => {
  const net = { ...ground, members: Array.from({ length: 250 }, (_, index) => ({ instance_path: `unresolved::${index}`, logical_pin: `P${index}`, numbers: [`${index}`] })) }
  const groups = destinationGroups(model, net, new Map(), new Set())
  assert.equal(groups[0].endpoints.length, 250)
  assert.deepEqual(groups[0].endpoints.map(endpoint => endpoint.member), net.members)
  const html = renderToStaticMarkup(React.createElement(OutsideDestinations, { model, net, instanceRegions: new Map(), visible: new Set(), onGroup() {}, onEndpoint() {} }))
  assert.equal((html.match(/<details/g) ?? []).length, 1)
  assert.match(html, /250 endpoints/)
  assert.doesNotMatch(html, /<details[^>]* open/)
})

test('index and destination commands retain explicit real group and exact endpoint intent', () => {
  const calls = []
  const entry = groupIndex(model, regions, values).get('SF32')
  const content = GroupIndexContent({ entry, onOpen: name => calls.push(['group', name]) })
  const html = renderToStaticMarkup(content)
  for (const text of ['group-index-identity', 'group-index-cores', 'group-index-interfaces', 'SF32LB525UC6', 'USB_DP', 'View inside SF32']) assert.ok(html.includes(text), text)
  const click = { stopPropagation() {} }
  content.props.children.at(-1).props.onClick(click)
  const destinations = OutsideDestinations({ model, net: ground, instanceRegions: regions, visible: new Set([instances[0].path]), onGroup: name => calls.push(['group', name]), onEndpoint: (...args) => calls.push(['endpoint', ...args]) })
  const visit = element => {
    if (!element?.props) return
    if (element.props.className === 'destination-group-command') element.props.onClick()
    if (element.props.className === 'destination-endpoint') element.props.onClick()
    for (const child of [element.props.children].flat(Infinity)) visit(child)
  }
  visit(destinations)
  assert.deepEqual(calls[0], ['group', 'SF32'])
  assert.ok(calls.some(call => JSON.stringify(call) === JSON.stringify(['group', 'USB'])))
  assert.ok(calls.some(call => JSON.stringify(call) === JSON.stringify(['endpoint', instances[2].path, 'P2', 'GND'])))
})
