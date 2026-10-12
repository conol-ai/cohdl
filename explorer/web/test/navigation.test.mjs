import assert from 'node:assert/strict'
import { after, test } from 'node:test'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { createRequire } from 'node:module'
import ts from 'typescript'

const outDir = mkdtempSync(join(tmpdir(), 'cohdl-navigation-test-'))
after(() => rmSync(outDir, { recursive: true, force: true }))
const program = ts.createProgram(['src/navigation.ts', 'src/search.ts', 'src/views.ts', 'src/handles.ts'], { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS, outDir, strict: true, skipLibCheck: true, types: [] })
assert.deepEqual(ts.getPreEmitDiagnostics(program).map((d) => ts.flattenDiagnosticMessageText(d.messageText, '\n')), [])
program.emit()
const require = createRequire(import.meta.url)
const { buildViewGraph, projectGroupUnion, collapseGroups, groupNodeId } = require(resolve(outDir, 'views.js'))
const { searchCircuit, inspectionIssues } = require(resolve(outDir, 'search.js'))
const { parseInspectionUrl, inspectionUrl, contextKey } = require(resolve(outDir, 'navigation.js'))
const { wireHandles } = require(resolve(outDir, 'handles.js'))
const span = { file: 'src/main.cohdl', line: 5, col: 2 }
const part = (path, designator) => ({ path, designator, device_fq: 'fixture::Device', impl_traits: [], specs: [], docs: [], span, pins: [
  { logical: 'SIGNAL', numbers: ['1', '3'], role: 'passive', obligation: 'required', connected: true, nc: false },
  { logical: 'RETURN', numbers: ['2'], role: 'passive', obligation: 'required', connected: true, nc: false },
  { logical: 'NC', numbers: ['4'], role: 'passive', obligation: 'optional', connected: false, nc: true },
] })
const model = () => ({ schema_version: 1, design: 'Board', verdict: 'pass', diagnostics: [], instances: [part('Board::a', 'U1'), part('Board::b', 'U2'), part('Board::c', 'U3')], subdesigns: [], nc: [{ instance_path: 'Board::a', logical_pin: 'NC' }], nets: [
  { name: 'BUS', is_gnd: false, span, members: [['Board::a', 'SIGNAL', ['1', '3']], ['Board::a', 'RETURN', ['2']], ['Board::b', 'SIGNAL', ['1', '3']], ['Board::c', 'SIGNAL', ['1', '3']]].map(([instance_path, logical_pin, numbers]) => ({ instance_path, logical_pin, numbers })) },
  { name: 'GND', is_gnd: true, span, members: ['Board::b', 'Board::c'].map((instance_path) => ({ instance_path, logical_pin: 'RETURN', numbers: ['2'] })) },
  { name: 'INTERNAL', is_gnd: false, span, members: [{ instance_path: 'Board::a', logical_pin: 'NC', numbers: ['4'] }] },
], footprints: {}, derived: { two_terminal: [], rails: ['GND'], fn_groups: [], bypasses: [] } })
const view = { name: '功能 / & boundaries', regions: [{ name: 'Left / & 空间', members: ['U1'] }, { name: 'Right', members: ['U2'] }] }
const config = { schema_version: 1, design: 'Board', views: [view] }
const freeze = (v) => { Object.freeze(v); for (const c of Object.values(v)) if (c && typeof c === 'object') freeze(c); return v }

test('device browsing defaults expanded SCH and migrates legacy overview exactly once', () => {
  const m = model()
  const clean = parseInspectionUrl(new URL('http://localhost/'), m, config)
  assert.equal(clean.mode, 'sch')
  assert.equal(clean.connectionMode, 'individual')
  assert.deepEqual(clean.collapsed, [])
  const legacy = { ...clean, mode: 'overview', groups: ['Right'], collapsed: ['Right'], selection: ['Board::b'], viewport: { viewport: { x: 5, y: 5, zoom: 1 }, size: { width: 400, height: 400 }, autoFit: false } }
  const oldUrl = new URL('http://localhost/')
  oldUrl.searchParams.set('inspect', JSON.stringify(legacy))
  const migrated = parseInspectionUrl(oldUrl, m, config)
  assert.equal(migrated.mode, 'sch')
  assert.deepEqual(migrated.groups, ['Right'])
  assert.deepEqual(migrated.selection, ['Board::b'])
  assert.deepEqual(migrated.collapsed, [])
  assert.equal(migrated.viewport, null)
  const manual = { ...legacy, connectionMode: 'bundled' }
  assert.deepEqual(parseInspectionUrl(inspectionUrl(oldUrl, manual), m, config), manual)
  assert.equal(parseInspectionUrl(new URL('http://localhost/?mode=overview'), m, config).mode, 'overview')
})

test('device projection represents rail-only components individually through group union', () => {
  const m = model()
  m.derived.two_terminal = ['Board::b', 'Board::c']
  m.derived.rails = ['BUS', 'GND']
  const full = buildViewGraph(m, null, view, { individual: true })
  const union = projectGroupUnion(m, '', full, ['Right'])
  assert.equal(union.graph.location.get('Board::b'), 'Board::b')
  assert.equal(union.graph.nodes.filter(n => n.inst?.path === 'Board::b').length, 1)
  assert.ok(!full.graph.nodes.some(n => n.aggMembers))
})

test('collapsed boundaries retain actual logical/physical endpoints and never inflate represented counts', () => {
  const m = freeze(model()), before = JSON.stringify(m)
  const full = buildViewGraph(m, null, view)
  const union = projectGroupUnion(m, null, full, [])
  const collapsed = collapseGroups(m, union, full, [view.regions[0].name, 'Right'])
  assert.deepEqual(collapsed.counts, union.counts)
  assert.deepEqual([...collapsed.represented], [...union.represented])
  const left = collapsed.graph.nodes.find((n) => n.id === groupNodeId(view.name, view.regions[0].name))
  assert.deepEqual(left.aggMembers, ['Board::a'])
  assert.deepEqual(left.ports.map((p) => [p.net, p.member.instance_path, p.member.logical_pin, p.member.numbers]), [
    ['BUS', 'Board::a', 'SIGNAL', ['1', '3']], ['BUS', 'Board::a', 'RETURN', ['2']],
  ])
  const byId = new Map(collapsed.graph.nodes.map((n) => [n.id, n]))
  const edges = collapsed.graph.edges.filter((e) => e.source === left.id)
  assert.equal(edges.length, 2)
  assert.equal(new Set(edges.map((e) => wireHandles(e, byId, true).sourceHandle)).size, 2)
  assert.ok(collapsed.graph.edges.every((e) => e.net === 'BUS'))
  assert.equal(JSON.stringify(m), before)
  const selected = projectGroupUnion(m, null, full, [view.regions[0].name, 'Other'])
  const hiddenHub = collapseGroups(m, selected, full, [view.regions[0].name])
  assert.ok(hiddenHub.graph.edges.some((e) => e.net === 'BUS' && e.source === left.id))
})

test('search resolves logical and every physical number, duplicate labels and NC without inventing a net', () => {
  const m = model()
  m.nets = m.nets.filter((n) => n.name !== 'INTERNAL')
  assert.ok(searchCircuit(m, 'U1 SIGNAL').some((h) => h.kind === 'pin' && h.id === 'Board::a' && h.pin === 'SIGNAL' && h.net === 'BUS'))
  assert.ok(searchCircuit(m, 'U1 3').some((h) => h.kind === 'pin' && h.pin === 'SIGNAL'))
  assert.ok(searchCircuit(m, 'U1.4').some((h) => h.kind === 'pin' && h.pin === 'NC' && !h.net))
  assert.deepEqual(searchCircuit(m, 'U1.1').filter((h) => h.kind === 'pin').map((h) => h.pin), ['SIGNAL'])
  assert.equal(searchCircuit(m, 'U1.1')[0].kind, 'pin')
  assert.deepEqual(searchCircuit(m, 'U1.4').filter((h) => h.kind === 'pin').map((h) => h.pin), ['NC'])
  m.instances[0].pins[0].numbers.push('A1', 'EP')
  m.instances[0].part = { fq: 'fixture::Part', mpn: 'EXACT MULTI WORD PART' }
  assert.equal(searchCircuit(m, 'MULTI WORD')[0].kind, 'part')
  assert.deepEqual(searchCircuit(m, 'U1.a1').filter((h) => h.kind === 'pin').map((h) => h.pin), ['SIGNAL'])
  assert.deepEqual(searchCircuit(m, 'U1.EP').filter((h) => h.kind === 'pin').map((h) => h.pin), ['SIGNAL'])
  assert.equal(searchCircuit(m, 'SIGNAL').filter((h) => h.kind === 'pin').length, 3)
  assert.deepEqual(searchCircuit(m, 'missing-component'), [])
})

test('collapse splits a hierarchy block spanning groups without duplicate or escaped real members', () => {
  const m = model()
  const paths = new Map([['Board::a', 'Board::child::a'], ['Board::b', 'Board::child::b']])
  m.instances.forEach((i) => { i.path = paths.get(i.path) ?? i.path })
  m.nets.forEach((n) => n.members.forEach((p) => { p.instance_path = paths.get(p.instance_path) ?? p.instance_path }))
  m.subdesigns = [{ path: 'Board::child', definition: 'fixture::Child', span, ports: [], docs: [] }]
  const full = buildViewGraph(m, null, view)
  const scoped = buildViewGraph(m, '', view)
  const union = projectGroupUnion(m, '', scoped, [])
  const folded = collapseGroups(m, union, full, [view.regions[0].name])
  assert.deepEqual(folded.counts, union.counts)
  assert.deepEqual([...folded.represented].sort(), [...union.represented].sort())
  for (const path of union.represented) {
    const nodes = folded.graph.nodes.filter((n) => n.inst?.path === path || n.aggMembers?.includes(path))
    assert.equal(nodes.length, 1, path)
    assert.equal(folded.graph.location.get(path), nodes[0].id)
  }
  assert.ok(!folded.graph.nodes.some((n) => n.subdesign))
})

test('scoped collapse rebuilds rail aggregates from represented members and aggregate selection survives URLs', () => {
  const m = model()
  m.instances[0].path = 'Board::child::a'
  m.instances[2].path = 'Board::child::inner::c'
  m.nets = [
    { name: 'GND', is_gnd: true, span, members: [m.instances[0], m.instances[1]].map((i) => ({ instance_path: i.path, logical_pin: 'SIGNAL', numbers: ['1', '3'] })) },
    { name: 'BUS', is_gnd: false, span, members: [{ instance_path: m.instances[2].path, logical_pin: 'SIGNAL', numbers: ['1', '3'] }] },
  ]
  m.derived.two_terminal = m.instances.slice(0, 2).map((i) => i.path)
  m.subdesigns = ['Board::child', 'Board::child::inner'].map((path) => ({ path, definition: 'fixture::Child', span, ports: [], docs: [] }))
  const definition = { name: 'Scoped', regions: [{ name: 'Rail', members: ['U1', 'U2'] }, { name: 'Logic', members: ['U3'] }] }
  const full = buildViewGraph(m, null, definition)
  const union = projectGroupUnion(m, 'Board::child', buildViewGraph(m, 'Board::child', definition), [])
  const folded = collapseGroups(m, union, full, ['Logic'])
  const aggregate = folded.graph.nodes.find((n) => n.id.startsWith('agg:'))
  assert.deepEqual(aggregate.aggMembers, [m.instances[0].path])
  assert.ok(!folded.graph.location.has(m.instances[1].path))
  const cfg = { ...config, views: [definition] }
  const base = parseInspectionUrl(new URL('http://localhost/'), m, cfg)
  const selected = { ...base, selection: [aggregate.id] }
  assert.deepEqual(parseInspectionUrl(inspectionUrl(new URL('http://localhost/'), selected), m, cfg).selection, selected.selection)
})

test('Issues retain diagnostics and separate last-good live-source failure, including missing spans', () => {
  const m = model()
  m.live_error = 'actual compiler failure without span'
  m.diagnostics = [{ code: 'E1', severity: 'error', message: 'pin conflict', span }, { code: 'W1', severity: 'warning', message: 'unknown source location' }]
  const issues = inspectionIssues(m)
  assert.equal(issues.length, 3)
  assert.ok(issues.some((i) => i.message === m.live_error && !i.span))
  assert.ok(issues.some((i) => i.code === 'W1' && !i.span))
})

test('URL round trips source identities and native cameras, context keys distinguish root from flat', () => {
  const m = model()
  const context = { ...parseInspectionUrl(new URL('http://localhost/'), m, config), mode: 'sch', scope: null, view: view.name, groups: [view.regions[0].name], collapsed: [view.regions[0].name], selection: ['Board::a'], net: 'BUS', pin: 'SIGNAL', tab: 'circuit', layoutFrame: 'board', layoutSide: 'bottom', viewport: { viewport: { x: -40.5, y: 87.2, zoom: 0.37 }, size: { width: 390, height: 240.5 }, autoFit: false }, boardViewport: { view: { x: -4, y: 2, width: 30, height: 20 }, size: { width: 500, height: 400 }, autoFit: false } }
  const url = inspectionUrl(new URL('http://localhost/?nolive=1&model=/original.json'), context)
  assert.equal(url.searchParams.get('model'), '/original.json')
  assert.deepEqual(parseInspectionUrl(url, m, config), context)
  assert.notEqual(contextKey({ ...context, scope: '' }), contextKey(context))
  assert.notEqual(contextKey({ ...context, view: '' }), contextKey(context))
})

test('URL validation prunes removed source identities and rejects malformed or unsupported cameras', () => {
  const m = model()
  const base = parseInspectionUrl(new URL('http://localhost/'), m, config)
  for (const viewport of [{ viewport: { x: null, y: 0, zoom: 1 }, size: { width: 390, height: 200 }, autoFit: false }, { viewport: { x: 0, y: 0, zoom: 3 }, size: { width: 390, height: 200 }, autoFit: false }]) {
    const input = { ...base, scope: 'removed', groups: ['missing'], collapsed: ['missing'], selection: ['missing'], net: 'missing', pin: 'missing', viewport, boardViewport: { view: { x: 0, y: 0, width: -3, height: 2 }, size: { width: 390, height: 200 }, autoFit: false } }
    const actual = parseInspectionUrl(inspectionUrl(new URL('http://localhost/'), input), m, config)
    assert.deepEqual(actual.groups, [])
    assert.deepEqual(actual.collapsed, [])
    assert.deepEqual(actual.selection, [])
    assert.equal(actual.scope, null)
    assert.equal(actual.net, '')
    assert.equal(actual.pin, '')
    assert.equal(actual.viewport, null)
    assert.equal(actual.boardViewport, null)
  }
  assert.deepEqual(parseInspectionUrl(new URL('http://localhost/?inspect=%7Bbroken'), m, config), base)
})

test('Proposals tab round trips without changing circuit selection or cameras', () => {
  const m = model()
  const context = { ...parseInspectionUrl(new URL('http://localhost/'), m, config), tab: 'proposals', selection: ['Board::a'], net: 'BUS' }
  assert.deepEqual(parseInspectionUrl(inspectionUrl(new URL('http://localhost/'), context), m, config), context)
})
