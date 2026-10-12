import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

const outDir = mkdtempSync(join(tmpdir(), 'cohdl-groups-test-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/views.ts', 'src/inspection.ts', 'src/viewport.ts', 'src/handles.ts'], {
  target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, outDir, strict: true, skipLibCheck: true, types: [],
})
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
const { buildViewGraph, projectGroupUnion, groupMemberPaths } = require(resolve(outDir, 'views.js'))
const { resizeViewport, validViewport, resizeBoardView } = require(resolve(outDir, 'viewport.js'))
const { netEndpoints } = require(resolve(outDir, 'inspection.js'))
const { wireHandles, passiveTerminals } = require(resolve(outDir, 'handles.js'))
const span = { file: 'main.cohdl', line: 1, col: 1 }
const part = (n) => ({ path: `Board::p${n}`, designator: `R${n}`, device_fq: 'fixture::D', impl_traits: [], specs: [], span, docs: [], pins: [
  { logical: 'A', numbers: ['1'], role: 'passive', obligation: 'required', connected: true, nc: false },
  { logical: 'B', numbers: ['2'], role: 'passive', obligation: 'required', connected: true, nc: false },
] })
const net = (name, members, is_gnd = false) => ({ name, is_gnd, span, members: members.map(([n, pin]) => ({ instance_path: `Board::p${n}`, logical_pin: pin, numbers: [pin === 'A' ? '1' : '2'] })) })
const model = () => ({ schema_version: 1, design: 'Board', verdict: 'pass', diagnostics: [], nc: [], footprints: {},
  instances: [1, 2, 3, 4, 5].map(part), subdesigns: [],
  nets: [net('CHAIN', [[1, 'A'], [2, 'A'], [3, 'A']]), net('STAR', [[1, 'B'], [2, 'B'], [3, 'B'], [4, 'B']]), net('GND', [[4, 'A'], [5, 'A']], true)],
  derived: { two_terminal: [], rails: ['GND'], fn_groups: [], bypasses: [] },
})
const view = () => ({ name: 'Blocks', regions: [
  { name: 'Left', members: ['R1', 'R3'] }, { name: 'Middle', members: ['R2'] },
  { name: 'Right', members: ['R4'] }, { name: 'Empty', members: [] },
] })
const freeze = (value) => { Object.freeze(value); for (const child of Object.values(value)) if (child && typeof child === 'object') freeze(child); return value }

test('group union reconnects real endpoints when old chain middle or star hub is hidden', () => {
  const m = freeze(model())
  const before = JSON.stringify(m)
  const full = buildViewGraph(m, null, view())
  const union = projectGroupUnion(m, null, full, ['Left', 'Right'])
  assert.deepEqual(union.graph.nodes.filter((n) => n.inst).map((n) => n.id), ['Board::p1', 'Board::p3', 'Board::p4'])
  for (const name of ['CHAIN', 'STAR']) {
    const edges = union.graph.edges.filter((e) => e.net === name)
    assert.ok(edges.some((e) => [e.source, e.target].includes('Board::p1') && [e.source, e.target].includes('Board::p3')))
    for (const e of edges) assert.ok(union.graph.nodes.some((n) => n.id === e.source) && union.graph.nodes.some((n) => n.id === e.target))
  }
  assert.equal(union.graph.edges.some((e) => e.net === 'GND'), false, 'rails remain stubs')
  assert.deepEqual(union.counts, { parts: 3, totalParts: 5, nets: 3, totalNets: 3 })
  assert.equal(new Set(union.graph.nodes.map((n) => n.id)).size, union.graph.nodes.length)
  assert.equal(JSON.stringify(m), before)
})

test('empty selection restores all, Other is selectable and empty groups recover without stale locations', () => {
  const m = model()
  const full = buildViewGraph(m, null, view())
  const all = projectGroupUnion(m, null, full, [])
  assert.equal(all.counts.parts, 5)
  assert.ok(all.graph.nodes.some((n) => n.id === 'net:STAR'))
  const other = projectGroupUnion(m, null, full, ['Other'])
  assert.deepEqual(other.graph.nodes.filter((n) => n.inst).map((n) => n.id), ['Board::p5'])
  const empty = projectGroupUnion(m, null, full, ['Empty'])
  assert.equal(empty.graph.nodes.length, 0)
  assert.equal(empty.graph.location.size, 0)
  assert.deepEqual(empty.counts, { parts: 0, totalParts: 5, nets: 0, totalNets: 3 })
})

test('overlapping regions select every matching member once, independent of first placement', () => {
  const m = freeze(model())
  const v = view()
  v.regions.push({ name: 'Shared', members: ['R1', 'R2', 'R2'] })
  const full = buildViewGraph(m, null, v, { individual: true })
  assert.equal(full.instanceRegions.get('Board::p1'), 'Left')
  const members = groupMemberPaths(m, v)
  assert.deepEqual([...members.get('Shared')], ['Board::p1', 'Board::p2'])
  for (const groups of [['Shared'], ['Shared', 'Middle'], ['Shared', 'Left']]) {
    const union = projectGroupUnion(m, null, full, groups)
    const expected = new Set(groups.flatMap(name => [...members.get(name)]))
    assert.deepEqual(union.represented, expected)
    assert.equal(union.counts.parts, expected.size)
    assert.equal(new Set(union.graph.nodes.map(n => n.id)).size, union.graph.nodes.length)
  }
})

test('selecting overlapping rail members splits a preexisting aggregate without extra parts', () => {
  const m = model()
  m.nets = [net('VCC', [[1, 'A'], [2, 'A'], [3, 'A']]), net('GND', [[1, 'B'], [2, 'B'], [3, 'B']], true)]
  m.derived = { ...m.derived, two_terminal: ['Board::p1', 'Board::p2', 'Board::p3'], rails: ['VCC', 'GND'] }
  const v = { name: 'Blocks', regions: [{ name: 'Capacitors', members: ['R1', 'R2', 'R3'] }, { name: 'Shared', members: ['R2'] }] }
  const full = buildViewGraph(m, null, v)
  const union = projectGroupUnion(m, null, full, ['Shared'])
  assert.deepEqual([...union.represented], ['Board::p2'])
  assert.equal(union.counts.parts, 1)
  assert.deepEqual(union.graph.nodes.flatMap(n => n.aggMembers ?? []), ['Board::p2'])
})

test('cross-group rail aggregates split by actual member ownership and preserve explicit aggregate rules', () => {
  const m = model()
  m.nets = [net('VCC', [[1, 'A'], [2, 'A'], [3, 'A']]), net('GND', [[1, 'B'], [2, 'B'], [3, 'B']], true)]
  m.derived = { ...m.derived, two_terminal: ['Board::p1', 'Board::p2', 'Board::p3'], rails: ['VCC', 'GND'] }
  const v = { name: 'Blocks', regions: [{ name: 'Left', members: ['R1'] }, { name: 'Right', members: ['R2'] }, { name: 'Shared', members: ['agg:GND + VCC'] }] }
  const full = buildViewGraph(freeze(m), null, v)
  const aggregates = full.graph.nodes.filter((n) => n.kind === 'agg')
  assert.equal(aggregates.length, 3)
  assert.deepEqual(aggregates.map((n) => [full.regions.byNode.get(n.id), n.aggMembers]), [
    ['Left', ['Board::p1']], ['Right', ['Board::p2']], ['Shared', ['Board::p3']],
  ])
  const union = projectGroupUnion(m, null, full, ['Left', 'Right'])
  assert.deepEqual(union.counts, { parts: 2, totalParts: 5, nets: 2, totalNets: 2 })
  assert.equal(union.graph.location.has('Board::p3'), false)
  assert.ok(union.graph.location.get('Board::p1') !== union.graph.location.get('Board::p2'))
  assert.equal(full.instanceRegions.get('Board::p3'), 'Shared')
})

test('host aggregate identity and selected members remain correct when host group is hidden', () => {
  const m = model()
  m.nets = [net('VCC', [[1, 'A'], [2, 'A'], [4, 'A']]), net('GND', [[1, 'B'], [2, 'B']], true)]
  m.derived = { ...m.derived, two_terminal: ['Board::p1', 'Board::p2'], rails: ['VCC', 'GND'], bypasses: [{ cap: 'Board::p1', target: 'Board::p4' }, { cap: 'Board::p2', target: 'Board::p4' }] }
  const v = { name: 'Blocks', regions: [{ name: 'Left', members: ['R1'] }, { name: 'Caps', members: ['agg:host:Board::p4'] }, { name: 'Host', members: ['R4'] }] }
  const full = buildViewGraph(m, null, v)
  const union = projectGroupUnion(m, null, full, ['Left', 'Caps'])
  assert.equal(union.graph.nodes.filter((n) => n.kind === 'agg').length, 2)
  assert.equal(union.graph.edges.length, 0)
  assert.equal(union.counts.parts, 2)
  assert.equal(full.instanceRegions.get('Board::p2'), 'Caps')
})

test('saved viewport resize preserves logical center and zoom, invalid viewports are rejected', () => {
  const before = { x: -85, y: 40, zoom: 0.65 }
  const after = resizeViewport(before, { width: 900, height: 700 }, { width: 390, height: 430 })
  assert.equal(after.zoom, before.zoom)
  assert.equal((900 / 2 - before.x) / before.zoom, (390 / 2 - after.x) / after.zoom)
  assert.equal((700 / 2 - before.y) / before.zoom, (430 / 2 - after.y) / after.zoom)
  assert.equal(validViewport(before), true)
  for (const v of [null, {}, { ...before, x: Infinity }, { ...before, zoom: 0 }, { ...before, zoom: 3 }]) assert.equal(validViewport(v), false)
})

test('physical viewBox resize preserves its actual scale and center', () => {
  const view = { x: -4, y: 3, width: 20, height: 20 }
  const resized = resizeBoardView(view, { width: 800, height: 600 }, { width: 300, height: 400 })
  assert.deepEqual(resized, { x: 1, y: 19 / 3, width: 10, height: 40 / 3 })
  assert.equal(300 / resized.width, 600 / view.height)
  assert.equal(resized.x + resized.width / 2, view.x + view.width / 2)
  assert.equal(resized.y + resized.height / 2, view.y + view.height / 2)
})

test('net inspection retains complete logical and physical endpoints with actual hidden group identity', () => {
  const m = freeze(model())
  const full = buildViewGraph(m, null, view())
  const union = projectGroupUnion(m, null, full, ['Left', 'Right'])
  const endpoints = netEndpoints(m, m.nets[0], full, union.graph)
  assert.deepEqual(endpoints.map((p) => [p.instance_path, p.logical_pin, p.numbers, p.group, p.visible]), [
    ['Board::p1', 'A', ['1'], 'Left', true],
    ['Board::p2', 'A', ['1'], 'Middle', false],
    ['Board::p3', 'A', ['1'], 'Left', true],
  ])
})

test('same-instance logical pins retain their actual detailed and passive wire handles', () => {
  for (const passive of [false, true]) {
    const m = model()
    m.nets = [net('JOIN', [[1, 'A'], [1, 'B'], [2, 'A']])]
    if (passive) m.derived.two_terminal = ['Board::p1']
    const full = buildViewGraph(m, null, view())
    const union = projectGroupUnion(m, null, full, ['Left', 'Middle'])
    const nodes = new Map(union.graph.nodes.map((n) => [n.id, n]))
    const edges = union.graph.edges.filter((e) => e.source === 'Board::p1')
    assert.deepEqual(edges.map((e) => e.sourcePin).sort(), ['A', 'B'])
    for (const detailed of [false, true]) {
      const handles = edges.map((e) => wireHandles(e, nodes, detailed).sourceHandle)
      assert.deepEqual(handles.sort(), passive || detailed ? ['p:A', 'p:B'] : [undefined, undefined])
    }
    if (passive) assert.deepEqual(passiveTerminals(nodes.get('Board::p1')).map((p) => p.id), ['p:A', 'p:B'])
  }
})

test('full endpoint metadata includes multiple logical and physical pins plus hidden Other members', () => {
  const m = model()
  m.nets = [net('JOIN', [[1, 'A'], [1, 'B'], [2, 'A'], [5, 'B']])]
  m.nets[0].members[0].numbers = ['1', '3']
  const full = buildViewGraph(m, null, view())
  const union = projectGroupUnion(m, null, full, ['Left'])
  assert.deepEqual(netEndpoints(m, m.nets[0], full, union.graph).map((p) => [p.logical_pin, p.numbers, p.group, p.visible]), [
    ['A', ['1', '3'], 'Left', true], ['B', ['2'], 'Left', true],
    ['A', ['1'], 'Middle', false], ['B', ['2'], 'Other', false],
  ])
})

test('group classification is flat and scoped union expands real members without hierarchy leakage', () => {
  const m = model()
  m.instances[0].path = 'Board::child::a'
  m.instances[1].path = 'Board::child::b'
  m.subdesigns = [{ path: 'Board::child', definition: 'fixture::Child', span, ports: [], docs: [] }]
  m.nets = [{ ...net('JOIN', []), members: m.instances.slice(0, 2).map((i) => ({ instance_path: i.path, logical_pin: 'A', numbers: ['1'] })) }]
  const scoped = buildViewGraph(m, '', view())
  const union = projectGroupUnion(m, '', scoped, ['Left'])
  assert.deepEqual([...union.represented].sort(), ['Board::child::a', 'Board::p3'])
  assert.equal(union.graph.nodes.some((n) => n.kind === 'subdesign'), false)
  assert.equal(union.counts.parts, 2)
  const other = projectGroupUnion(m, '', scoped, ['Other'])
  assert.deepEqual([...other.represented], ['Board::p5'])
  assert.equal(scoped.instanceRegions.get('Board::child::a'), 'Left')
})
